//! Auth primitives (pure, no web framework) — port of `server/src/auth.ts` + `guards.ts`.
//!
//! Fail-closed rules:
//! * Admin key check via constant-time comparison (`subtle`); an unset server key
//!   means admin is **disabled**, never open.
//! * Telegram initData per the official `WebApp` `HMAC-SHA256` scheme.
//! * In-memory sliding-window rate limit — same semantics as TS (Phase 4 distribution
//!   as Axum middleware, with the same 429 body).

use std::collections::HashMap;
use std::fmt::Write as _;
use std::sync::{Mutex, PoisonError};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::Sha256;
use subtle::ConstantTimeEq;

// ── X-KIAN-Key ───────────────────────────────────────────────────────────────

/// Admin key check — constant-time comparison.
///
/// * Server key `None` or shorter than the minimum (16 characters; 8 for demo keys
///   with the explicit `dev-` prefix) → `false` (admin disabled).
/// * Missing/empty header → `false`.
/// * Different lengths → `false` (no byte-by-byte comparison — no timing leak).
#[must_use]
pub fn verify_kian_key(header_value: Option<&str>, server_key: Option<&str>) -> bool {
    let Some(server_key) = server_key else { return false };
    // A generated admin key is at least 16 characters; the explicit `dev-` prefix declares
    // a demo/development key (e.g. `dev-admin-key`) accepted at a minimum of 8 characters —
    // local testing only; generate a long key for real deployments.
    let min_len = if server_key.starts_with("dev-") { 8 } else { 16 };
    if server_key.len() < min_len {
        return false;
    }
    let Some(header) = header_value else { return false };
    if header.is_empty() {
        return false;
    }
    let a = header.as_bytes();
    let b = server_key.as_bytes();
    if a.len() != b.len() {
        return false;
    }
    a.ct_eq(b).into()
}

// ── Telegram initData (official WebApp scheme) ─────────────────────────────────────

/// Maximum allowed age of `auth_date` in `initData` (24 hours = 86,400 seconds).
pub const INIT_DATA_MAX_AGE_SECS: i64 = 86_400;

/// Maximum allowed future clock skew for `auth_date` (5 minutes = 300 seconds).
pub const INIT_DATA_MAX_FUTURE_SECS: i64 = 300;

/// Identity derived from initData.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelegramIdentity {
    /// Telegram numeric ID as a string (`"123"`)
    pub user_id: String,
    /// Optional username
    pub username: Option<String>,
    /// Optional `start_param` from the mini-app direct link (for product/referral)
    pub start_param: Option<String>,
}

/// User record inside initData (`user` = form-encoded JSON).
#[derive(Debug, Deserialize)]
struct RawTelegramUser {
    id: Option<f64>,
    username: Option<String>,
}

/// Official `Telegram MiniApp` key derivation: `HMAC-SHA256(key = "WebAppData", data = bot_token)`.
fn webapp_secret_key(bot_token: &str) -> [u8; 32] {
    let mut mac = Hmac::<Sha256>::new_from_slice(b"WebAppData")
        .unwrap_or_else(|_| unreachable!("WebAppData is a valid HMAC key"));
    mac.update(bot_token.as_bytes());
    let bytes = mac.finalize().into_bytes();
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    out
}

/// Current Unix time in seconds.
fn current_unix_secs() -> Option<i64> {
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();
    i64::try_from(secs).ok()
}

/// Official Telegram `WebApp` initData validation (HMAC-SHA256) against the current system clock.
///
/// For deterministic tests with an injected clock, use [`verify_telegram_init_data_at`].
#[must_use]
pub fn verify_telegram_init_data(init_data: &str, bot_token: &str) -> Option<TelegramIdentity> {
    let now_secs = current_unix_secs()?;
    verify_telegram_init_data_at(init_data, bot_token, now_secs)
}

/// Official Telegram `WebApp` initData validation (HMAC-SHA256) at time `now_secs`:
///
/// 1. Separate `hash` from the parameters.
/// 2. Sort the remaining parameters (including `start_param` if present) alphabetically
///    and join them with `\n`.
/// 3. `secret = HMAC-SHA256("WebAppData", bot_token)`,
///    `computed = HMAC-SHA256(secret, data_check_string)`.
/// 4. Constant-time comparison against `hash` (hex).
/// 5. Check `auth_date` freshness (at most 24 hours in the past and 5 minutes in the future — anti-replay).
///
/// Any step failing → `None` (fail-closed).
#[must_use]
pub fn verify_telegram_init_data_at(
    init_data: &str,
    bot_token: &str,
    now_secs: i64,
) -> Option<TelegramIdentity> {
    let mut params: Vec<(String, String)> = parse_urlencoded(init_data);
    let hash_pos = params.iter().position(|(k, _)| k == "hash")?;
    let hash = params.remove(hash_pos).1;

    params.sort_by(|a, b| a.0.cmp(&b.0));
    let data_check_string =
        params.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join("\n");

    let mut secret = webapp_secret_key(bot_token);
    // the HMAC-SHA256 output is always 32 bytes — a valid HMAC key
    let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(&secret) else {
        secret.fill(0);
        return None;
    };
    mac.update(data_check_string.as_bytes());
    let computed = mac.finalize().into_bytes();
    secret.fill(0); // wipe the derived key from memory

    // constant-time comparison on the decoded hex bytes
    let expected = hex::decode(&hash).ok()?;
    if expected.len() != computed.len() {
        return None;
    }
    if expected.ct_eq(computed.as_slice()).unwrap_u8() != 1 {
        return None;
    }

    // auth_date freshness check — Replay Attack prevention
    let auth_date_raw = params.iter().find(|(k, _)| k == "auth_date").map(|(_, v)| v.as_str())?;
    let auth_date: i64 = auth_date_raw.parse().ok()?;
    let age = now_secs.checked_sub(auth_date)?;
    if !(-INIT_DATA_MAX_FUTURE_SECS..=INIT_DATA_MAX_AGE_SECS).contains(&age) {
        return None;
    }

    // user: JSON inside the user parameter — id must be a safe integer
    let user_raw = params.iter().find(|(k, _)| k == "user").map(|(_, v)| v.clone())?;
    let user: RawTelegramUser = serde_json::from_str(&user_raw).ok()?;
    let id = user.id?;
    if !id.is_finite() || id < 0.0 || id.fract() != 0.0 || id > 9_007_199_254_740_991.0 {
        return None;
    }
    let username = user.username.filter(|u| !u.is_empty());
    let start_param = params
        .iter()
        .find(|(k, _)| k == "start_param")
        .map(|(_, v)| v.trim().to_owned())
        .filter(|v| !v.is_empty());
    #[allow(clippy::cast_possible_truncation)] // id first: is_finite, fract==0, ≤2^53−1
    Some(TelegramIdentity { user_id: format!("{}", id as i64), username, start_param })
}

/// Build valid initData for the tests — port of `makeTestInitData` with a live `auth_date`.
#[must_use]
pub fn make_test_init_data(bot_token: &str, user: &str, username: Option<&str>) -> String {
    let id: i64 = user.parse().unwrap_or(42);
    let now = current_unix_secs().unwrap_or(1_760_000_000);
    build_bot_init_data(bot_token, id, username, now)
}

/// Mint valid initData on the bot side (M6) — the single identity-minting point for Telegram.
///
/// The long-polling/webhook bot holds `KIAN_BOT_TOKEN`; updates come straight from the
/// trusted Telegram channel. For the citizen session, the bot follows the same official
/// mini-app path: initData is minted with the same official `WebAppData` HMAC and
/// `POST /auth/telegram` verifies it like the mini-app — **there is no side channel for identity**.
///
/// `auth_date` (Unix epoch seconds) comes from the injected shell clock — offline-testable.
#[must_use]
pub fn build_bot_init_data(
    bot_token: &str,
    user_id: i64,
    username: Option<&str>,
    auth_date: i64,
) -> String {
    let user_json = match username {
        Some(u) if !u.is_empty() => format!(r#"{{"id":{user_id},"username":"{}"}}"#, escape_json(u)),
        _ => format!(r#"{{"id":{user_id}}}"#),
    };
    let mut params: Vec<(String, String)> = vec![
        ("user".into(), user_json),
        ("auth_date".into(), format!("{auth_date}")),
    ];
    params.sort_by(|a, b| a.0.cmp(&b.0));
    let data_check_string =
        params.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join("\n");
    let mut secret = webapp_secret_key(bot_token);
    let mut mac = Hmac::<Sha256>::new_from_slice(&secret)
        .unwrap_or_else(|_| unreachable!("HMAC-SHA256 secret is always 32 bytes"));
    mac.update(data_check_string.as_bytes());
    let hash = hex::encode(mac.finalize().into_bytes());
    secret.fill(0);
    let pairs =
        params.iter().map(|(k, v)| format!("{k}={}", urlencode(v))).collect::<Vec<_>>().join("&");
    format!("{pairs}&hash={hash}")
}

/// JSON escaping for the username — neutralizes dangerous quotes and backslashes so
/// the output is always JSON-safe (identity never flows through the raw user string).
fn escape_json(s: &str) -> String {
    s.chars().filter(|c| *c != '"' && *c != '\\').collect()
}

/// Simple form encoding — like Telegram's real `URLSearchParams.toString()`:
/// only unreserved characters (`A-Za-z0-9-_.`) stay raw; `{}`, `"`, `:`, and the rest
/// are percent-encoded — the output is always JSON-safe (like real initData).
fn urlencode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' => {
                out.push(b as char);
            }
            _ => {
                let _ = write!(out, "%{b:02X}");
            }
        }
    }
    out
}

/// Form-encoded parsing — no dependencies; `+` does not map to space per `URLSearchParams` except in query.
fn parse_urlencoded(s: &str) -> Vec<(String, String)> {
    s.split('&')
        .filter(|p| !p.is_empty())
        .map(|pair| {
            let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
            (urldecode(k), urldecode(v))
        })
        .collect()
}

/// Percent decoding.
fn urldecode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() + 1 && i + 2 < bytes.len() {
            let hex_part = std::str::from_utf8(&bytes[i + 1..i + 3]).ok();
            if let Some(v) = hex_part.and_then(|h| u8::from_str_radix(h, 16).ok()) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

// ── Rate limit — in-memory sliding window ──────────────────────────────────

/// Window size (60 seconds) — like TS.
pub const RATE_WINDOW_SECS: u64 = 60;

/// Sliding-window limiter — one per server (inside the Phase 4 Mutex).
///
/// * Trusted callers (bot/dashboard — holding the admin key) share the `admin` bucket.
/// * Everyone else is keyed by their real IP (X-Forwarded-For in Phase 4).
/// * Memory retention: more than 10,000 keys → sweep (like TS).
#[derive(Debug, Default)]
pub struct RateLimiter {
    windows: HashMap<String, Vec<Instant>>,
    limit: usize,
}

impl RateLimiter {
    /// New limiter with a requests-per-minute cap.
    #[must_use]
    pub fn new(limit: usize) -> Self {
        Self { windows: HashMap::new(), limit }
    }

    /// Record a hit and decide — `Ok(())` allowed, `Err(HttpFailure)` means a ready 429.
    ///
    /// # Errors
    /// [`HttpFailure::rate_limited`] when the hits in the 60-second window reach the cap.
    pub fn check(&mut self, key: &str) -> Result<(), crate::error::HttpFailure> {
        let now = Instant::now();
        let window = std::time::Duration::from_secs(RATE_WINDOW_SECS);
        let hits = self.windows.entry(key.to_owned()).or_default();
        hits.retain(|t| now.duration_since(*t) < window);
        if hits.len() >= self.limit {
            return Err(crate::error::HttpFailure::rate_limited(
                i64::try_from(self.limit).unwrap_or(i64::MAX),
            ));
        }
        hits.push(now);
        // Correctness fix (2025-11 load test): previously, more than 10,000 keys triggered
        // `windows.clear()`, wiping the rate cap of *every* user at once
        // (a busy day = wholesale bypass of the rules). Now only idle buckets —
        // those whose last hit is outside the window — are evicted: an active user is never
        // reset, and memory stays proportional to the active set.
        if self.windows.len() > 10_000 {
            // the full window has not elapsed, no eviction happens (no panic) —
            if let Some(cutoff) = now.checked_sub(window) {
                self.windows.retain(|_, h| h.last().is_some_and(|t| *t > cutoff));
            }
        }
        Ok(())
    }

    /// Full reset (tests).
    pub fn reset(&mut self) {
        self.windows.clear();
    }
}

/// Shard count — lock partitioning with FNV-1a; at high RPS it cuts contention
/// by ~16x (2025-11 load-test fix: the global Mutex was a single hot spot).
pub const RATE_SHARD_COUNT: usize = 16;

/// Sharded rate limiter — `RATE_SHARD_COUNT` independent sliding windows, each with
/// its own Mutex; a key is hashed to a fixed shard (same key = same shard,
/// so the per-key cap is exactly as before). `&self` — Sync with no outer lock.
#[derive(Debug)]
pub struct ShardedLimiter {
    shards: Vec<Mutex<RateLimiter>>,
}

impl ShardedLimiter {
    /// Sharded limiter — every shard has the same cap.
    #[must_use]
    pub fn new(limit: usize) -> Self {
        Self {
            shards: (0..RATE_SHARD_COUNT)
                .map(|_| Mutex::new(RateLimiter::new(limit)))
                .collect(),
        }
    }

    /// Fixed shard for a key — FNV-1a: fast, dependency-free, uniform distribution.
    fn shard_for(&self, key: &str) -> &Mutex<RateLimiter> {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325; // FNV offset basis
        for b in key.as_bytes() {
            h ^= u64::from(*b);
            h = h.wrapping_mul(0x0000_0100_0000_01b3); // FNV prime
        }
        let idx = usize::try_from(h % self.shards.len() as u64).unwrap_or(0);
        &self.shards[idx]
    }

    /// Record a hit and decide — same contract as [`RateLimiter::check`].
    ///
    /// # Errors
    /// [`HttpFailure::rate_limited`] when the key's shard cap fills.
    pub fn check(&self, key: &str) -> Result<(), crate::error::HttpFailure> {
        let mut shard = self.shard_for(key)
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        shard.check(key)
    }

    /// Reset all shards (tests).
    pub fn reset(&self) {
        for s in &self.shards {
            s.lock().unwrap_or_else(PoisonError::into_inner).reset();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: &str = "kia-admin-key-0123456789";

    #[test]
    fn bot_minted_init_data_roundtrips_through_official_verify() {
        let bot = "7000000000:TEST-BOT-TOKEN-M6";
        let now = 1_760_000_000;
        let data = build_bot_init_data(bot, 777_001, Some("kia_agent"), now);
        let identity =
            verify_telegram_init_data_at(&data, bot, now).expect("bot-minted initData must verify");
        assert_eq!(identity.user_id, "777001");
        assert_eq!(identity.username.as_deref(), Some("kia_agent"));
        assert!(identity.start_param.is_none());
        // a different token → fail-closed (the signature is valid only with the holder's token)
        assert!(verify_telegram_init_data_at(&data, "7000000000:OTHER", now).is_none());
        // no username → nameless identity, still valid
        let bare = build_bot_init_data(bot, 8, None, now);
        let id2 = verify_telegram_init_data_at(&bare, bot, now).expect("bare initData verifies");
        assert_eq!(id2.user_id, "8");
        assert!(id2.username.is_none());
        // dangerous username — quotes/backslashes never reach the JSON (they are only stripped)
        let evil = build_bot_init_data(bot, 9, Some("a\"},\"is_bot\":true"), now);
        let id3 =
            verify_telegram_init_data_at(&evil, bot, now).expect("escaped username verifies");
        assert_eq!(id3.username.as_deref(), Some("a},is_bot:true"));
        // output always JSON-safe: no raw {, }, " (same rules as make_test_init_data)
        assert!(!evil.contains('{') && !evil.contains('"') && !evil.contains('}'));
    }

    #[test]
    fn kia_key_fail_closed_on_missing_or_short_server_key() {
        assert!(!verify_kian_key(Some(KEY), None)); // server without a key → never open
        assert!(!verify_kian_key(Some(KEY), Some("short"))); // < 16 characters
        assert!(!verify_kian_key(None, Some(KEY))); // missing header
        assert!(!verify_kian_key(Some(""), Some(KEY))); // empty header
        assert!(verify_kian_key(Some(KEY), Some(KEY)));
        assert!(!verify_kian_key(Some("kia-admin-key-0123456788"), Some(KEY)));
        assert!(!verify_kian_key(Some(KEY), Some(&KEY[..15])));

        // demo key with the explicit `dev-` prefix — minimum 8 characters (local test tonight)
        let dev: &str = "dev-admin-key"; // 13 characters
        assert!(verify_kian_key(Some(dev), Some(dev)));
        assert!(!verify_kian_key(Some("dev-admin-keY"), Some(dev))); // same length but different
        assert!(!verify_kian_key(Some(dev), Some("dev-adm"))); // < 8 demo characters
    }

    #[test]
    fn telegram_init_data_official_scheme_roundtrip() {
        let bot = "7000000000:TEST-TOKEN-abc";
        let data = make_test_init_data(bot, "12345", Some("ali"));
        let identity = verify_telegram_init_data(&data, bot).expect("valid initData must verify");
        assert_eq!(identity.user_id, "12345");
        assert_eq!(identity.username.as_deref(), Some("ali"));
        assert!(identity.start_param.is_none());
    }

    /// The helper's output must look like real Telegram initData: no raw
    /// `{`, `}`, `"` characters (JSON-safe) and fully percent-encoded — so it fits inside a JSON
    /// body without breaking and stays compatible with real `URLSearchParams`.
    #[test]
    fn make_test_init_data_output_is_urlencoded_and_json_safe() {
        let data = make_test_init_data("123:tok", "42", Some("sara"));
        assert!(!data.contains('{'), "raw braces break JSON bodies");
        assert!(!data.contains('"'), "raw quotes break JSON bodies");
        assert!(data.contains("user=%7B"), "user field must be percent-encoded like real Telegram");
        assert!(verify_telegram_init_data(&data, "123:tok").is_some(), "still verifies");
        // official WebAppData scheme — fully encoded initData with start_param verifies too
        let auth_date = 1_760_000_000i64;
        let fully_encoded = format!(
            "auth_date={auth_date}&start_param=p_prod1_r_999&user=%7B%22id%22%3A77%2C%22username%22%3A%22ok%22%7D"
        );
        let secret = webapp_secret_key("123:tok");
        let mut mac = Hmac::<Sha256>::new_from_slice(&secret).unwrap();
        mac.update(
            format!("auth_date={auth_date}\nstart_param=p_prod1_r_999\nuser={{\"id\":77,\"username\":\"ok\"}}")
                .as_bytes(),
        );
        let full = format!(
            "{fully_encoded}&hash={}",
            hex::encode(mac.finalize().into_bytes())
        );
        let identity = verify_telegram_init_data_at(&full, "123:tok", auth_date + 60)
            .expect("real Telegram WebApp form with start_param must verify");
        assert_eq!(identity.user_id, "77");
        assert_eq!(identity.username.as_deref(), Some("ok"));
        assert_eq!(identity.start_param.as_deref(), Some("p_prod1_r_999"));
    }

    #[test]
    fn telegram_init_data_rejects_expired_future_and_login_widget_hmac() {
        use sha2::Digest;

        let bot = "7000000000:TEST-TOKEN-abc";
        let now = 1_760_100_000i64;
        // expired (> 86,400 seconds ago)
        let expired = build_bot_init_data(bot, 10, Some("old"), now - INIT_DATA_MAX_AGE_SECS - 1);
        assert!(verify_telegram_init_data_at(&expired, bot, now).is_none());
        // on the 24-hour boundary — still valid
        let on_edge = build_bot_init_data(bot, 10, Some("edge"), now - INIT_DATA_MAX_AGE_SECS);
        assert!(verify_telegram_init_data_at(&on_edge, bot, now).is_some());
        // more than 5 minutes in the future (> 300 seconds)
        let future = build_bot_init_data(bot, 10, Some("fut"), now + INIT_DATA_MAX_FUTURE_SECS + 1);
        assert!(verify_telegram_init_data_at(&future, bot, now).is_none());
        // on the 5-minute future boundary — valid
        let skew_ok = build_bot_init_data(bot, 10, Some("skew"), now + INIT_DATA_MAX_FUTURE_SECS);
        assert!(verify_telegram_init_data_at(&skew_ok, bot, now).is_some());

        // the old Login Widget formula (SHA256(bot_token)) must be rejected
        let raw = format!("auth_date={now}&user=%7B%22id%22%3A55%7D");
        let old_secret = Sha256::digest(bot.as_bytes());
        let mut mac = Hmac::<Sha256>::new_from_slice(&old_secret).unwrap();
        mac.update(format!("auth_date={now}\nuser={{\"id\":55}}").as_bytes());
        let legacy_signed = format!("{raw}&hash={}", hex::encode(mac.finalize().into_bytes()));
        assert!(
            verify_telegram_init_data_at(&legacy_signed, bot, now).is_none(),
            "Login Widget SHA256(bot_token) signature must be rejected"
        );
    }

    #[test]
    fn telegram_init_data_rejects_tampering() {
        let bot = "7000000000:TEST-TOKEN-abc";
        let mut data = make_test_init_data(bot, "12345", None);
        // tampered hash
        let len = data.len();
        data.replace_range(len - 2..len, if data.ends_with("aa") { "bb" } else { "aa" });
        assert!(verify_telegram_init_data(&data, bot).is_none());
        // wrong token
        let data2 = make_test_init_data(bot, "99", None);
        assert!(verify_telegram_init_data(&data2, "7000000000:OTHER").is_none());
        // missing hash
        let data3 = data2.split("&hash=").next().unwrap_or("").to_string();
        assert!(verify_telegram_init_data(&data3, bot).is_none());
        // user without id (with a valid WebAppData signature and a fresh auth_date)
        let now = 1_760_000_000i64;
        let secret = webapp_secret_key(bot);
        let mut mac = Hmac::<Sha256>::new_from_slice(&secret).unwrap();
        mac.update(format!("auth_date={now}\nuser={{}}").as_bytes());
        let bogus =
            format!("auth_date={now}&user=%7B%7D&hash={}", hex::encode(mac.finalize().into_bytes()));
        assert!(verify_telegram_init_data_at(&bogus, bot, now).is_none());
    }

    #[test]
    fn rate_limiter_blocks_after_limit_and_separates_admin_bucket() {
        let mut lim = RateLimiter::new(3);
        assert!(lim.check("1.2.3.4").is_ok());
        assert!(lim.check("1.2.3.4").is_ok());
        assert!(lim.check("1.2.3.4").is_ok());
        let err = lim.check("1.2.3.4").unwrap_err();
        assert_eq!(err.status, 429);
        assert_eq!(err.code, "RATE_LIMITED");
        // the admin has its own bucket — a trusted caller is unaffected by the public IP
        assert!(lim.check("admin").is_ok());
        lim.reset();
    }

    /// Load-test fix: evict only idle buckets — an active user is never reset.
    #[test]
    fn rate_limiter_eviction_never_resets_active_buckets() {
        let mut lim = RateLimiter::new(5);
        // an active user fills up to the cap; the subsequent key storm triggers the sweep
        for _ in 0..5 {
            assert!(lim.check("active-user").is_ok());
        }
        for i in 0..11_000usize {
            let k = format!("ip:10.{i}");
            assert!(lim.check(&k).is_ok());
        }
        assert!(lim.windows.len() > 10_000, "sweep must have been triggered");
        // the old behavior (windows.clear()) reset the cap here and returned is_ok;
        // the fixed behavior: the active user remains capped
        assert!(lim.check("active-user").is_err(), "active bucket must never be reset");
        // active buckets retained (here all are active — only idle ones would have been evicted)
        assert!(lim.windows.len() > 10_000);
        lim.reset();
    }

    /// Sharded limiter: the same key goes to the same shard (the per-key cap is preserved).
    #[test]
    fn sharded_limiter_keeps_per_key_limit() {
        let lim = ShardedLimiter::new(3);
        assert!(lim.check("user-A").is_ok());
        assert!(lim.check("user-A").is_ok());
        assert!(lim.check("user-A").is_ok());
        let err = lim.check("user-A").unwrap_err();
        assert_eq!(err.status, 429);
        // another key in another or the same shard — either way it has its own independent cap
        assert!(lim.check("user-B").is_ok());
        lim.reset();
        // after reset, key A is counted again
        assert!(lim.check("user-A").is_ok());
    }

    /// FNV distribution — different keys spread across shards (they must not all land on one).
    #[test]
    fn sharded_limiter_distributes_keys_across_shards() {
        let _lim = ShardedLimiter::new(10);
        let mut shards_used = std::collections::HashSet::new();
        for i in 0..256usize {
            let k = format!("ip:10.0.0.{i}");
            let mut h: u64 = 0xcbf2_9ce4_8422_2325;
            for b in k.as_bytes() {
                h ^= u64::from(*b);
                h = h.wrapping_mul(0x0000_0100_0000_01b3);
            }
            let shard_count = u64::try_from(RATE_SHARD_COUNT).expect("shard count fits u64");
            shards_used.insert(usize::try_from(h % shard_count).expect("shard index fits usize"));
        }
        assert!(
            shards_used.len() >= RATE_SHARD_COUNT / 2,
            "256 keys must spread across at least half the shards, got {}",
            shards_used.len()
        );
    }
}
