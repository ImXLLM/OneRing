//! طبقه‌بند قطعی هویت کاربر — پورت `10x/engine/identity_rl.py` (پوشهٔ ۰۲ گنجینه).
//!
//! نسخهٔ پایتون Q-Learning با اکتشاف تصادفی بود؛ پورت کیان «قطعی» است —
//! همان فضای برچسب‌ها (۷ معمارهٔ کاربر)، همان سبدسازی ویژگی‌ها، همان
//! جوایز رویدادمحور، اما با:
//! * طبقه‌بند [`classify`] — برندهٔ بیشینه روی امتیازهای انباشته (بدون RNG)؛
//! * هش وضعیت [`hash_state`] — FNV-1a ۳۲بیتیِ همیشه‌پایدار (جایگزین
//!   `blake2b % 100000` پایتون که وابسته به کتابخانهٔ خارجی بود).
#![forbid(unsafe_code)]

use std::fmt;

/// برچسب‌های هویتی — همان `ACTIONS` پایتون، به‌صورت enum بسته.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UserArchetype {
    /// تازه‌وارد — سیگنال‌های کم
    NewUser,
    /// تماشاگر — می‌چرخد، قصد خرید کم
    Browser,
    /// کارکنِ مأموریت — اعتبار را با تسک جمع می‌کند
    TaskEarner,
    /// خریدار بازگشتی
    ReturningBuyer,
    /// پرارزش — برداشت/خرید بزرگ
    HighValue,
    /// همراه — تعامل عمیق با دستیار و محصولات
    Supporter,
    /// در آستانهٔ ریزش — بازگشت خرج یا اعتراض
    ChurnRisk,
}

impl UserArchetype {
    /// برچسب متنی — همان رشته‌های `ACTIONS` پایتون (سازگاری سیم).
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

    /// همهٔ برچسب‌ها به ترتیب پایدار — همان ترتیب `ACTIONS` پایتون.
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

/// بردار ویژگی‌های گسسته‌شدهٔ رفتار کاربر — همان ستون‌های `_BUCKETS` پایتون.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UserFeatures {
    /// بازدید (چت + خرید — نمای بازگشت)
    pub visits: u32,
    /// تعداد خرید
    pub purchases: u32,
    /// تسک‌های انجام‌شده
    pub tasks: u32,
    /// برداشت‌های موفق
    pub withdraws: u32,
    /// سطح چت
    pub chat_level: u32,
    /// سبد زمانی (ساعت / ۶)
    pub session_bucket: u32,
    /// تازگی — ۱ اگر سن حساب < ۲ روز
    pub freshness: u32,
}

/// مرزهای سبد — عیناً `_BUCKETS` پایتون.
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
    /// رشتهٔ وضعیت گسسته — عیناً همان قالب `_state` پایتون:
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

/// هش وضعیت قطعی — FNV-1a ۳۲بیتی (پایدار در همهٔ نسخه‌ها و سکوها؛ جایگزین
/// `hashlib.blake2b % 100000` پایتون). برای شمول در جدول Q مقیاس‌پذیر.
#[must_use]
pub fn hash_state(state: &str) -> u32 {
    let mut h: u32 = 0x811C_9DC5;
    for b in state.as_bytes() {
        h ^= u32::from(*b);
        h = h.wrapping_mul(0x0100_0193);
    }
    h
}

/// جایز رویدادها — عیناً `_EVENT_REWARD` پایتون؛ غایب یعنی صفر.
///
/// خروجی: `(برچسب، امتیاز)` جفت‌های رویداد.
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

/// امتیاز برچسب‌ها را از دنبالهٔ رویدادها می‌سازد (بدون نرخ یادگیری — جمعِ ساده
/// جوایز؛ نتایج با Q-Learning هم‌راستاست چون بیشینهٔ همان جوایز را برمی‌گزیند).
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

/// طبقه‌بند قطعی — برندهٔ بیشینهٔ امتیازها با اولویت ترتیب پایدار.
///
/// بدون هیچ رویداد (پروندهٔ نو) → [`UserArchetype::NewUser`] (همان
/// `profile_label` پایتون). تسک‌محور بودن بر خرید بازگشتی، در حالت تساوی
/// با ترتیب رویدادها حل می‌شود — ورودی دنباله همیشه قطعی است.
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
    // اولین برچسب در ترتیب پایدار که به بیشینه رسیده — قطعی و تکرارپذیر
    totals
        .iter()
        .find(|(_, v)| (*v - best).abs() < f64::EPSILON)
        .map_or(UserArchetype::NewUser, |(a, _)| *a)
}

/// اطمینان برنده — نسبت امتیاز برنده به مجموع قدر مطلق‌ها (۰..=۱؛ عینِ حس
/// `confidence` پایتون اما قطعی و بدون softmax سرریزپذیر).
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

    /// رشتهٔ وضعیت — قالب عین پایتون و سبدسازی سقف‌دار.
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

    /// هش وضعیت — پایدار، دترمینیستیتیک، بدون برخورد در فضای تست.
    #[test]
    fn hash_state_is_stable_and_collides_rarely() {
        let s = "visits:0|purchases:0|tasks:0|withdraws:0|chat_level:0|session_bucket:0|freshness:0";
        assert_eq!(hash_state(s), hash_state(s));
        let empty = hash_state("");
        let base = hash_state(s);
        assert_ne!(empty, base);
        // چندصد وضعیت متمایز → برخورد کمتر از ۲٪ (فضای ۲^۳۲)
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

    /// طبقه‌بند — پرچم‌های رفتاری متمایز به برچسب درست می‌رسند.
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
        // خریدار بازگشتی: چند خرید + ویزیت — بر خریدار بازگشتی می‌چربد
        assert_eq!(
            classify(&["purchase", "visit", "purchase"]),
            UserArchetype::ReturningBuyer
        );
        // رویداد ناشناخته → هیچ امتیازی → NewUser
        assert_eq!(classify(&["nope"]), UserArchetype::NewUser);
    }

    /// قطعیت — همان دنباله، همان برچسب و همان اطمینان.
    #[test]
    fn classification_is_deterministic() {
        let events = vec!["purchase", "withdraw", "chat_message"];
        assert_eq!(classify(&events), classify(&events));
        assert!((confidence(&events) - confidence(&events)).abs() < f64::EPSILON);
        assert!(confidence(&["purchase"]) > confidence(&[]));
        // ترکیب خرید + برداشت → HighValue (برندهٔ بیشینه)
        assert_eq!(classify(&["purchase", "withdraw"]), UserArchetype::HighValue);
    }

    /// برچسب‌ها و رشته‌ها — سازگاری سیم با پایتون.
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
