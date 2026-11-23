//! اولیه‌های احراز (خالص، بدون وبرام) — پورت `server/src/auth.ts` + `guards.ts`.
//!
//! قواعد fail-closed:
//! * بررسی کلید ادمین با مقایسهٔ زمان‌ثابت (`subtle`)؛ کلیدِ تنظیم‌نشدهٔ سرور =
//!   ادمین **غیرفعال**، هرگز باز.
//! * Telegram initData طبق طرح رسمی `WebApp` `HMAC-SHA256`.
//! * rate-limit پنجرهٔ لغزان درون‌حافظه‌ای — همان معناشناسی TS (توزیع فاز ۴
//!   به‌عنوان middleware آکسوم، با همان بدنهٔ 429).

use std::collections::HashMap;
use std::fmt::Write as _;
use std::sync::{Mutex, PoisonError};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::Sha256;
use subtle::ConstantTimeEq;

// ── X-KIAN-Key ───────────────────────────────────────────────────────────────

/// بررسی کلید ادمین — مقایسهٔ زمان‌ثابت.
///
/// * کلید سرور `None` یا کوتاه‌تر از حداقل (۱۶ کاراکتر؛ ۸ برای کلیدهای
///   دمو با پیشوند صریح `dev-`) → `false` (ادمین غیرفعال).
/// * هدر غایب/خالی → `false`.
/// * طول‌های متفاوت → `false` (بدون مقایسهٔ بایت‌به‌بایت — نشت زمانی ندارد).
#[must_use]
pub fn verify_kian_key(header_value: Option<&str>, server_key: Option<&str>) -> bool {
    let Some(server_key) = server_key else { return false };
    // کلید ادمین تولیدی حداقل ۱۶ کاراکتر؛ پیشوند `dev-` صریحاً کلید دمو/توسعه
    // اعلام می‌کند (مثل `dev-admin-key`) و با حداقل ۸ کاراکتر پذیرفته می‌شود —
    // فقط برای تستِ محلی؛ در استقرار واقعی کلید بلند تولید کنید.
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

// ── Telegram initData (طرح رسمی WebApp) ─────────────────────────────────────

/// حداکثر عمر مجاز `auth_date` در `initData` (۲۴ ساعت = ۸۶٬۴۰۰ ثانیه).
pub const INIT_DATA_MAX_AGE_SECS: i64 = 86_400;

/// حداکثر انحراف مجاز ساعت به آینده برای `auth_date` (۵ دقیقه = ۳۰۰ ثانیه).
pub const INIT_DATA_MAX_FUTURE_SECS: i64 = 300;

/// هویت مشتق‌شده از initData.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelegramIdentity {
    /// شناسهٔ عددی تلگرام به‌صورت رشته (`"123"`)
    pub user_id: String,
    /// یوزرنیم اختیاری
    pub username: Option<String>,
    /// پارامتر اختیاری `start_param` از لینک مستقیم مینی‌اپ (برای محصول/رفرال)
    pub start_param: Option<String>,
}

/// رکورد کاربر داخل initData (`user` = JSON کدشده در فرم).
#[derive(Debug, Deserialize)]
struct RawTelegramUser {
    id: Option<f64>,
    username: Option<String>,
}

/// اشتقاق کلید رسمی Telegram MiniApp: `HMAC-SHA256(key = "WebAppData", data = bot_token)`.
fn webapp_secret_key(bot_token: &str) -> [u8; 32] {
    let mut mac = Hmac::<Sha256>::new_from_slice(b"WebAppData")
        .unwrap_or_else(|_| unreachable!("WebAppData is a valid HMAC key"));
    mac.update(bot_token.as_bytes());
    let bytes = mac.finalize().into_bytes();
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    out
}

/// زمان فعلی Unix بر حسب ثانیه.
fn current_unix_secs() -> Option<i64> {
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();
    i64::try_from(secs).ok()
}

/// اعتبارسنجی رسمی Telegram `WebApp` initData (HMAC-SHA256) با ساعت فعلی سیستم.
///
/// برای تست‌های قطعی با ساعت تزریقی از [`verify_telegram_init_data_at`] استفاده کنید.
#[must_use]
pub fn verify_telegram_init_data(init_data: &str, bot_token: &str) -> Option<TelegramIdentity> {
    let now_secs = current_unix_secs()?;
    verify_telegram_init_data_at(init_data, bot_token, now_secs)
}

/// اعتبارسنجی رسمی Telegram `WebApp` initData (HMAC-SHA256) در زمان `now_secs`:
///
/// 1. `hash` را از پارامترها جدا کن.
/// 2. بقیهٔ پارامترها (از جمله `start_param` در صورت وجود) را به ترتیب حروف الفبا
///    sort کن و با `\n` بچسبان.
/// 3. `secret = HMAC-SHA256("WebAppData", bot_token)`،
///    `computed = HMAC-SHA256(secret, data_check_string)`.
/// 4. مقایسهٔ زمان‌ثابت با `hash` (hex).
/// 5. بررسی تازگی `auth_date` (حداکثر ۲۴ ساعت گذشته و ۵ دقیقه آینده — ضد Replay).
///
/// هر گام شکست → `None` (fail-closed).
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
    // خروجی HMAC-SHA256 همیشه ۳۲ بایت است — کلید معتبر HMAC
    let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(&secret) else {
        secret.fill(0);
        return None;
    };
    mac.update(data_check_string.as_bytes());
    let computed = mac.finalize().into_bytes();
    secret.fill(0); // پاک‌سازی کلید مشتق‌شده از حافظه

    // مقایسهٔ زمان‌ثابت روی بایت‌های hex‌گشایی‌شده
    let expected = hex::decode(&hash).ok()?;
    if expected.len() != computed.len() {
        return None;
    }
    if expected.ct_eq(computed.as_slice()).unwrap_u8() != 1 {
        return None;
    }

    // بررسی تازگی auth_date — جلوگیری از Replay Attack
    let auth_date_raw = params.iter().find(|(k, _)| k == "auth_date").map(|(_, v)| v.as_str())?;
    let auth_date: i64 = auth_date_raw.parse().ok()?;
    let age = now_secs.checked_sub(auth_date)?;
    if age > INIT_DATA_MAX_AGE_SECS || age < -INIT_DATA_MAX_FUTURE_SECS {
        return None;
    }

    // کاربر: JSON داخل پارامتر user — id باید صحیح امن باشد
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
    #[allow(clippy::cast_possible_truncation)] // id پیش‌تر: is_finite، fract==0، ≤2^53−1
    Some(TelegramIdentity { user_id: format!("{}", id as i64), username, start_param })
}

/// ساخت initData معتبر برای تست‌ها — پورت `makeTestInitData` با `auth_date` زنده.
#[must_use]
pub fn make_test_init_data(bot_token: &str, user: &str, username: Option<&str>) -> String {
    let id: i64 = user.parse().unwrap_or(42);
    let now = current_unix_secs().unwrap_or(1_760_000_000);
    build_bot_init_data(bot_token, id, username, now)
}

/// ضرب initData معتبر از سمت بات (M6) — نقطهٔ تنها ضرب هویت تلگرام.
///
/// باتِ طول‌کشی/وبهوک، دارندهٔ `KIAN_BOT_TOKEN` است؛ آپدیت‌ها مستقیم از کانال
/// معتمد تلگرام می‌آیند. برای نشست شهروند، بات همان مسیر رسمی مینی‌اپ را
/// طی می‌کند: initData با همین الگوریتم رسمی `WebAppData` HMAC ضرب می‌شود و
/// `POST /auth/telegram` آن را مثل مینی‌اپ تأیید می‌کند — **هیچ مسیر جانبی برای هویت وجود ندارد**.
///
/// `auth_date` (Unix epoch بر حسب ثانیه) از ساعت تزریقی شِل می‌آید — آفلاین‌تست‌پذیر.
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

/// گریز JSON برای username — نقل‌قول و بک‌اسلش خطرناک را خنثی می‌کند تا
/// خروجی همیشه JSON-safe بماند (هویت هرگز از رشتهٔ خام کاربر عبور نمی‌کند).
fn escape_json(s: &str) -> String {
    s.chars().filter(|c| *c != '"' && *c != '\\').collect()
}

/// کدگذاری فرم ساده — مثل `URLSearchParams.toString()` واقعی تلگرام:
/// فقط unreserved (`A-Za-z0-9-_.`) خام می‌ماند؛ `{}`, `"`, `:` و بقیه
/// percent-encoded می‌شوند — خروجی همیشه در JSON امن است (مثل initData واقعی).
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

/// تجزیهٔ فرم‌encoded — بدون وابستگی؛ + به فاصلهٔ `URLSearchParams` نگاشت نمی‌شود مگر در query.
fn parse_urlencoded(s: &str) -> Vec<(String, String)> {
    s.split('&')
        .filter(|p| !p.is_empty())
        .map(|pair| {
            let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
            (urldecode(k), urldecode(v))
        })
        .collect()
}

/// کدگشایی درصدی.
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

// ── Rate limit — پنجرهٔ لغزان درون‌حافظه‌ای ──────────────────────────────────

/// اندازهٔ پنجره (۶۰ ثانیه) — مثل TS.
pub const RATE_WINDOW_SECS: u64 = 60;

/// limiter پنجرهٔ لغزان — یکی به ازای سرور (درون Mutex فاز ۴).
///
/// * تماس‌گیرندگان معتمد (بات/داشبورد — دارای کلید ادمین) باکت مشترک `admin` می‌گیرند.
/// * بقیه با ip واقعی کلید می‌خورند (X-Forwarded-For در فاز ۴).
/// * نگه‌داشت حافظه: بیش از ۱۰٬۰۰۰ کلید → پاک‌سازی (مثل TS).
#[derive(Debug, Default)]
pub struct RateLimiter {
    windows: HashMap<String, Vec<Instant>>,
    limit: usize,
}

impl RateLimiter {
    /// limiter جدید با سقف درخواست در دقیقه.
    #[must_use]
    pub fn new(limit: usize) -> Self {
        Self { windows: HashMap::new(), limit }
    }

    /// ثبت hit و تصمیم — `Ok(())` مجاز، `Err(HttpFailure)` یعنی 429 آماده.
    ///
    /// # Errors
    /// [`HttpFailure::rate_limited`] وقتی تعداد hits در پنجرهٔ ۶۰ثانیه‌ای به سقف برسد.
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
        // اصلاح صحت (تست بار ۲۰۲۵-۱۱): پیش‌تر بیش از ۱۰٬۰۰۰ کلید →
        // `windows.clear()` که سقف نرخِ *همهٔ* کاربران را یک‌باره می‌ریخت
        // (روز شلوغ = دورزدن دسته‌جمعی قواعد). حالا فقط باکت‌های خاموش —
        // که آخرین hit‌شان بیرون پنجره است — تخلیه می‌شوند: کاربر فعال هرگز
        // ریست نمی‌شود و حافظه به اندازهٔ مجموعهٔ فعال باقی می‌ماند.
        if self.windows.len() > 10_000 {
            let cutoff = now - window;
            self.windows.retain(|_, h| h.last().is_some_and(|t| *t > cutoff));
        }
        Ok(())
    }

    /// ریست کامل (تست‌ها).
    pub fn reset(&mut self) {
        self.windows.clear();
    }
}

/// تعداد شاردها — تقسیم قفل با FNV-1a؛ در RPS بالا contention را ~۱۶ برابر
/// می‌کاهد (اصلاح تست بار ۲۰۲۵-۱۱: Mutex سراسری تک‌نقطهٔ داغ بود).
pub const RATE_SHARD_COUNT: usize = 16;

/// limiter نرخ شاردشده — `RATE_SHARD_COUNT` پنجرهٔ لغزان مستقل، هرکدام با
/// Mutex خودش؛ کلید با hash به shard ثابت می‌رود (همان کلید = همان shard،
/// پس سقف per-key دقیقاً مثل قبل است). `&self` — Sync و بدون قفل بیرونی.
#[derive(Debug)]
pub struct ShardedLimiter {
    shards: Vec<Mutex<RateLimiter>>,
}

impl ShardedLimiter {
    /// limiter شاردشده — هر shard همان سقف را دارد.
    #[must_use]
    pub fn new(limit: usize) -> Self {
        Self {
            shards: (0..RATE_SHARD_COUNT)
                .map(|_| Mutex::new(RateLimiter::new(limit)))
                .collect(),
        }
    }

    /// شارد ثابت برای کلید — FNV-1a: سریع، بدون وابستگی، توزیع یکنواخت.
    fn shard_for(&self, key: &str) -> &Mutex<RateLimiter> {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325; // FNV offset basis
        for b in key.as_bytes() {
            h ^= u64::from(*b);
            h = h.wrapping_mul(0x0000_0100_0000_01b3); // FNV prime
        }
        let idx = usize::try_from(h % self.shards.len() as u64).unwrap_or(0);
        &self.shards[idx]
    }

    /// ثبت hit و تصمیم — همان قرارداد [`RateLimiter::check`].
    ///
    /// # Errors
    /// [`HttpFailure::rate_limited`] وقتی سقف shardِ کلید پر شود.
    pub fn check(&self, key: &str) -> Result<(), crate::error::HttpFailure> {
        let mut shard = self.shard_for(key)
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        shard.check(key)
    }

    /// ریست همهٔ شاردها (تست‌ها).
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
        // توکن دیگر → fail-closed (امضا فقط با توکن دارنده معتبر است)
        assert!(verify_telegram_init_data_at(&data, "7000000000:OTHER", now).is_none());
        // بدون username → هویت بدون نام، همچنان معتبر
        let bare = build_bot_init_data(bot, 8, None, now);
        let id2 = verify_telegram_init_data_at(&bare, bot, now).expect("bare initData verifies");
        assert_eq!(id2.user_id, "8");
        assert!(id2.username.is_none());
        // username خطرناک — نقل‌قول/بک‌اسلش هرگز به JSON عبور نمی‌کند (فقط حذف می‌شوند)
        let evil = build_bot_init_data(bot, 9, Some("a\"},\"is_bot\":true"), now);
        let id3 =
            verify_telegram_init_data_at(&evil, bot, now).expect("escaped username verifies");
        assert_eq!(id3.username.as_deref(), Some("a},is_bot:true"));
        // خروجی همیشه JSON-safe: بدون {، }، " خام (همان قواعد make_test_init_data)
        assert!(!evil.contains('{') && !evil.contains('"') && !evil.contains('}'));
    }

    #[test]
    fn kia_key_fail_closed_on_missing_or_short_server_key() {
        assert!(!verify_kian_key(Some(KEY), None)); // سرور بدون کلید → هرگز باز
        assert!(!verify_kian_key(Some(KEY), Some("short"))); // < ۱۶ کاراکتر
        assert!(!verify_kian_key(None, Some(KEY))); // هدر غایب
        assert!(!verify_kian_key(Some(""), Some(KEY))); // هدر خالی
        assert!(verify_kian_key(Some(KEY), Some(KEY)));
        assert!(!verify_kian_key(Some("kia-admin-key-0123456788"), Some(KEY)));
        assert!(!verify_kian_key(Some(KEY), Some(&KEY[..15])));

        // کلید دمو با پیشوند صریح `dev-` — حداقل ۸ کاراکتر (تست محلی امشب)
        const DEV: &str = "dev-admin-key"; // ۱۳ کاراکتر
        assert!(verify_kian_key(Some(DEV), Some(DEV)));
        assert!(!verify_kian_key(Some("dev-admin-keY"), Some(DEV))); // هم‌طول اما متفاوت
        assert!(!verify_kian_key(Some(DEV), Some("dev-adm"))); // < ۸ کاراکتر دمو
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

    /// خروجی helper باید مثل initData واقعی تلگرام باشد: بدون هیچ کاراکتر خام
    /// `{`, `}`, `"` (JSON-safe) و کاملاً percent-encoded — تا داخل بدنهٔ JSON
    /// بدون شکست جا شود و با `URLSearchParams` واقعی سازگار باشد.
    #[test]
    fn make_test_init_data_output_is_urlencoded_and_json_safe() {
        let data = make_test_init_data("123:tok", "42", Some("sara"));
        assert!(!data.contains('{'), "raw braces break JSON bodies");
        assert!(!data.contains('"'), "raw quotes break JSON bodies");
        assert!(data.contains("user=%7B"), "user field must be percent-encoded like real Telegram");
        assert!(verify_telegram_init_data(&data, "123:tok").is_some(), "still verifies");
        // طرح رسمی WebAppData — initData کاملاً کدنویسی‌شده با start_param هم verify می‌شود
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
        // منقضی (> ۸۶٬۴۰۰ ثانیه قبل)
        let expired = build_bot_init_data(bot, 10, Some("old"), now - INIT_DATA_MAX_AGE_SECS - 1);
        assert!(verify_telegram_init_data_at(&expired, bot, now).is_none());
        // روی مرز ۲۴ ساعت — هنوز معتبر
        let on_edge = build_bot_init_data(bot, 10, Some("edge"), now - INIT_DATA_MAX_AGE_SECS);
        assert!(verify_telegram_init_data_at(&on_edge, bot, now).is_some());
        // بیش از ۵ دقیقه در آینده (> ۳۰۰ ثانیه)
        let future = build_bot_init_data(bot, 10, Some("fut"), now + INIT_DATA_MAX_FUTURE_SECS + 1);
        assert!(verify_telegram_init_data_at(&future, bot, now).is_none());
        // روی مرز ۵ دقیقه آینده — معتبر
        let skew_ok = build_bot_init_data(bot, 10, Some("skew"), now + INIT_DATA_MAX_FUTURE_SECS);
        assert!(verify_telegram_init_data_at(&skew_ok, bot, now).is_some());

        // فرمول قدیمی Login Widget (SHA256(bot_token)) باید رد شود
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
        // دستکاری hash
        let len = data.len();
        data.replace_range(len - 2..len, if data.ends_with("aa") { "bb" } else { "aa" });
        assert!(verify_telegram_init_data(&data, bot).is_none());
        // توکن اشتباه
        let data2 = make_test_init_data(bot, "99", None);
        assert!(verify_telegram_init_data(&data2, "7000000000:OTHER").is_none());
        // hash غایب
        let data3 = data2.split("&hash=").next().unwrap_or("").to_string();
        assert!(verify_telegram_init_data(&data3, bot).is_none());
        // user بدون id (با امضای معتبر WebAppData و auth_date تازه)
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
        // ادمین باکت خودش را دارد — تماس معتمد متأثر از ip عمومی نیست
        assert!(lim.check("admin").is_ok());
        lim.reset();
    }

    /// اصلاح تست بار: تخلیهٔ فقط باکت‌های خاموش — کاربرِ فعال هرگز ریست نمی‌شود.
    #[test]
    fn rate_limiter_eviction_never_resets_active_buckets() {
        let mut lim = RateLimiter::new(5);
        // کاربر فعال تا سقف پر می‌شود؛ بعد طوفان کلیدها، sweep را فعال می‌کند
        for _ in 0..5 {
            assert!(lim.check("active-user").is_ok());
        }
        for i in 0..11_000usize {
            let k = format!("ip:10.{i}");
            assert!(lim.check(&k).is_ok());
        }
        assert!(lim.windows.len() > 10_000, "sweep must have been triggered");
        // رفتار قدیمی (windows.clear()) اینجا سقف را ریست می‌کرد و is_ok می‌داد؛
        // رفتار اصلاح‌شده: کاربر فعال همچنان سقف‌خورده است
        assert!(lim.check("active-user").is_err(), "active bucket must never be reset");
        // باکت‌های فعال نگه داشته شدند (اینجا همه فعال‌اند — فقط خاموش‌ها می‌رفتند)
        assert!(lim.windows.len() > 10_000);
        lim.reset();
    }

    /// limiter شارد: همان کلید به همان shard می‌رود (سقف per-key حفظ می‌شود).
    #[test]
    fn sharded_limiter_keeps_per_key_limit() {
        let lim = ShardedLimiter::new(3);
        assert!(lim.check("user-A").is_ok());
        assert!(lim.check("user-A").is_ok());
        assert!(lim.check("user-A").is_ok());
        let err = lim.check("user-A").unwrap_err();
        assert_eq!(err.status, 429);
        // کلید دیگر در shard دیگر یا همان — در هر حال سقف مستقل دارد
        assert!(lim.check("user-B").is_ok());
        lim.reset();
        // بعد از reset، کلید A از نو شمرده می‌شود
        assert!(lim.check("user-A").is_ok());
    }

    /// توزیع FNV — کلیدهای متفاوت روی چند shard پخش می‌شوند (نباید همه به یکی بروند).
    #[test]
    fn sharded_limiter_distributes_keys_across_shards() {
        let lim = ShardedLimiter::new(10);
        let mut shards_used = std::collections::HashSet::new();
        for i in 0..256usize {
            let k = format!("ip:10.0.0.{i}");
            let mut h: u64 = 0xcbf2_9ce4_8422_2325;
            for b in k.as_bytes() {
                h ^= u64::from(*b);
                h = h.wrapping_mul(0x0000_0100_0000_01b3);
            }
            shards_used.insert((h % RATE_SHARD_COUNT as u64) as usize);
        }
        assert!(
            shards_used.len() >= RATE_SHARD_COUNT / 2,
            "256 keys must spread across at least half the shards, got {}",
            shards_used.len()
        );
    }
}
