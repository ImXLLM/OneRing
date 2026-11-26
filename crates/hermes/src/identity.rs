//! Deterministic user-identity classifier — port of `10x/engine/identity_rl.py` (vault folder 02).
//!
//! The Python version was Q-Learning with stochastic exploration; the Kian port is "deterministic" —
//! the same label space (7 user archetypes), the same feature bucketing, the same
//! event-driven rewards, but with:
//! * the [`classify`] classifier — the argmax winner over accumulated scores (no RNG);
//! * the [`hash_state`] state hash — an always-stable 32-bit FNV-1a (replacing
//!   the Python `blake2b % 100000` that depended on an external library).
#![forbid(unsafe_code)]

use std::fmt;

/// Identity labels — the Python `ACTIONS`, as a closed enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UserArchetype {
    /// Newcomer — few signals
    NewUser,
    /// Spectator — browses, little purchase intent
    Browser,
    /// Mission worker — earns credits through tasks
    TaskEarner,
    /// Repeat buyer
    ReturningBuyer,
    /// High-value — large withdrawals/purchases
    HighValue,
    /// Companion — deep engagement with the assistant and products
    Supporter,
    /// Churn risk — refunds or complaints
    ChurnRisk,
}

impl UserArchetype {
    /// Text label — the Python `ACTIONS` strings (wire compatibility).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NewUser => "new_user",
            Self::Browser => "browser",
            Self::TaskEarner => "task_earner",
            Self::ReturningBuyer => "returning_buyer",
            Self::HighValue => "high_value",
            Self::Supporter => "supporter",
            Self::ChurnRisk => "churn_risk",
        }
    }

    /// All labels in stable order — the Python `ACTIONS` order.
    #[must_use]
    pub const fn all() -> [Self; 7] {
        [
            Self::NewUser,
            Self::Browser,
            Self::TaskEarner,
            Self::ReturningBuyer,
            Self::HighValue,
            Self::Supporter,
            Self::ChurnRisk,
        ]
    }
}

impl fmt::Display for UserArchetype {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Discretized user-behavior feature vector — the Python `_BUCKETS` columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UserFeatures {
    /// Visits (chat + purchase — returning view)
    pub visits: u32,
    /// Purchase count
    pub purchases: u32,
    /// Completed tasks
    pub tasks: u32,
    /// Successful withdrawals
    pub withdraws: u32,
    /// Chat level
    pub chat_level: u32,
    /// Time bucket (hour / 6)
    pub session_bucket: u32,
    /// Freshness — 1 if the account age < 2 days
    pub freshness: u32,
}

/// Bucket edges — exactly the Python `_BUCKETS`.
type FeatureExtractor = fn(&UserFeatures) -> u32;

const BUCKETS: [(&str, u32, FeatureExtractor); 7] = [
    ("visits", 4, |f| f.visits),
    ("purchases", 4, |f| f.purchases),
    ("tasks", 4, |f| f.tasks),
    ("withdraws", 3, |f| f.withdraws),
    ("chat_level", 4, |f| f.chat_level),
    ("session_bucket", 4, |f| f.session_bucket),
    ("freshness", 2, |f| f.freshness),
];

impl UserFeatures {
    /// Discrete state string — exactly the Python `_state` format:
    /// `visits:0|purchases:0|tasks:0|withdraws:0|chat_level:0|session_bucket:0|freshness:0`
    #[must_use]
    pub fn state_string(&self) -> String {
        let parts: Vec<String> = BUCKETS
            .iter()
            .map(|(k, b, get)| {
                let v = get(self).min(b - 1);
                format!("{k}:{v}")
            })
            .collect();
        parts.join("|")
    }
}

/// Deterministic state hash — 32-bit FNV-1a (stable across versions and platforms; replaces
/// the Python `hashlib.blake2b % 100000`). Suitable for inclusion in a scalable Q table.
#[must_use]
pub fn hash_state(state: &str) -> u32 {
    let mut h: u32 = 0x811C_9DC5;
    for b in state.as_bytes() {
        h ^= u32::from(*b);
        h = h.wrapping_mul(0x0100_0193);
    }
    h
}

/// Event rewards — exactly the Python `_EVENT_REWARD`; absent means zero.
///
/// Output: `(label, score)` event pairs.
#[must_use]
pub fn event_rewards(event: &str) -> &'static [(UserArchetype, f64)] {
    match event {
        "purchase" => &[
            (UserArchetype::ReturningBuyer, 5.0),
            (UserArchetype::HighValue, 2.0),
            (UserArchetype::Supporter, 1.0),
        ],
        "withdraw" => &[
            (UserArchetype::HighValue, 5.0),
            (UserArchetype::ReturningBuyer, 1.0),
        ],
        "task_done" => &[(UserArchetype::TaskEarner, 4.0), (UserArchetype::NewUser, 0.5)],
        "visit" => &[
            (UserArchetype::ReturningBuyer, 1.0),
            (UserArchetype::Supporter, 0.5),
            (UserArchetype::NewUser, 0.3),
        ],
        "chat_message" => &[
            (UserArchetype::Supporter, 1.5),
            (UserArchetype::ReturningBuyer, 0.5),
        ],
        "refund" => &[
            (UserArchetype::ChurnRisk, 3.0),
            (UserArchetype::HighValue, -1.0),
        ],
        "chargeback" => &[
            (UserArchetype::ChurnRisk, 5.0),
            (UserArchetype::TaskEarner, -1.0),
        ],
        "mystery_box" => &[
            (UserArchetype::TaskEarner, 1.5),
            (UserArchetype::ReturningBuyer, -0.5),
        ],
        _ => &[],
    }
}

/// Builds label scores from the event sequence (no learning rate — a plain sum of
/// rewards; results align with Q-Learning since the argmax of the same rewards is chosen).
fn score(events: &[&str]) -> Vec<(UserArchetype, f64)> {
    let mut totals: Vec<(UserArchetype, f64)> =
        UserArchetype::all().iter().map(|&a| (a, 0.0)).collect();
    for ev in events {
        for (label, r) in event_rewards(ev) {
            if let Some(slot) = totals.iter_mut().find(|(a, _)| a == label) {
                slot.1 += r;
            }
        }
    }
    totals
}

/// Deterministic classifier — the argmax winner over scores with stable-order priority.
///
/// No events at all (a fresh profile) → [`UserArchetype::NewUser`] (the Python
/// `profile_label`). Mission-worker vs repeat-buyer ties are resolved
/// by event order — the sequence input is always deterministic.
#[must_use]
pub fn classify(events: &[&str]) -> UserArchetype {
    if events.is_empty() {
        return UserArchetype::NewUser;
    }
    let totals = score(events);
    let best = totals
        .iter()
        .map(|(_, v)| *v)
        .fold(f64::NEG_INFINITY, f64::max);
    // The first label in stable order to reach the max — deterministic and reproducible
    totals
        .iter()
        .find(|(_, v)| (*v - best).abs() < f64::EPSILON)
        .map_or(UserArchetype::NewUser, |(a, _)| *a)
}

/// Winner confidence — the winner score over the sum of absolute values (0..=1; the spirit of
/// the Python `confidence` but deterministic and without an overflow-prone softmax).
#[must_use]
pub fn confidence(events: &[&str]) -> f64 {
    let totals = score(events);
    let sum: f64 = totals.iter().map(|(_, v)| v.abs()).sum();
    if sum <= f64::EPSILON {
        return 0.0;
    }
    let best = totals.iter().map(|(_, v)| *v).fold(0.0_f64, f64::max);
    (best / sum).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// State string — a Python-identical format with clamped bucketing.
    #[test]
    fn state_string_matches_python_format() {
        let f = UserFeatures::default();
        assert_eq!(
            f.state_string(),
            "visits:0|purchases:0|tasks:0|withdraws:0|chat_level:0|session_bucket:0|freshness:0"
        );
        let big = UserFeatures {
            visits: 99,
            purchases: 3,
            tasks: 50,
            withdraws: 7,
            chat_level: 100,
            session_bucket: 9,
            freshness: 1,
        };
        assert_eq!(
            big.state_string(),
            "visits:3|purchases:3|tasks:3|withdraws:2|chat_level:3|session_bucket:3|freshness:1"
        );
    }

    /// State hash — stable, deterministic, collision-free over the test space.
    #[test]
    fn hash_state_is_stable_and_collides_rarely() {
        let s = "visits:0|purchases:0|tasks:0|withdraws:0|chat_level:0|session_bucket:0|freshness:0";
        assert_eq!(hash_state(s), hash_state(s));
        let empty = hash_state("");
        let base = hash_state(s);
        assert_ne!(empty, base);
        // Several hundred distinct states → under 2% collisions (a 2^32 space)
        let mut seen = std::collections::HashSet::new();
        let mut collisions = 0;
        for i in 0..200u32 {
            let st = format!("{s}|v{i}");
            if !seen.insert(hash_state(&st)) {
                collisions += 1;
            }
        }
        assert!(collisions <= 4, "collisions: {collisions}");
    }

    /// Classifier — distinct behavioral flags land on the right label.
    #[test]
    fn classify_matches_expected_archetypes() {
        assert_eq!(classify(&[]), UserArchetype::NewUser);
        assert_eq!(classify(&["purchase"]), UserArchetype::ReturningBuyer);
        assert_eq!(classify(&["withdraw"]), UserArchetype::HighValue);
        assert_eq!(classify(&["task_done"]), UserArchetype::TaskEarner);
        assert_eq!(classify(&["chat_message"]), UserArchetype::Supporter);
        assert_eq!(classify(&["refund"]), UserArchetype::ChurnRisk);
        assert_eq!(classify(&["chargeback"]), UserArchetype::ChurnRisk);
        assert_eq!(classify(&["visit"]), UserArchetype::ReturningBuyer);
        assert_eq!(classify(&["mystery_box"]), UserArchetype::TaskEarner);
        // Repeat buyer: several purchases + visits — weighted toward a returning buyer
        assert_eq!(
            classify(&["purchase", "visit", "purchase"]),
            UserArchetype::ReturningBuyer
        );
        // Unknown event → no score → NewUser
        assert_eq!(classify(&["nope"]), UserArchetype::NewUser);
    }

    /// Determinism — the same sequence, the same label, the same confidence.
    #[test]
    fn classification_is_deterministic() {
        let events = vec!["purchase", "withdraw", "chat_message"];
        assert_eq!(classify(&events), classify(&events));
        assert!((confidence(&events) - confidence(&events)).abs() < f64::EPSILON);
        assert!(confidence(&["purchase"]) > confidence(&[]));
        // A purchase + withdrawal combination → HighValue (the argmax winner)
        assert_eq!(classify(&["purchase", "withdraw"]), UserArchetype::HighValue);
    }

    /// Labels and strings — wire compatibility with Python.
    #[test]
    fn archetypes_serialize_to_python_labels() {
        let labels: Vec<&str> = UserArchetype::all().iter().map(|a| a.as_str()).collect();
        assert_eq!(
            labels,
            vec![
                "new_user",
                "browser",
                "task_earner",
                "returning_buyer",
                "high_value",
                "supporter",
                "churn_risk"
            ]
        );
        assert_eq!(UserArchetype::Browser.to_string(), "browser");
    }
}
