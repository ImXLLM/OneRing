//! هندلرهای HTTP — هر هندلر سه گام دارد: امنیت (گارد ادمین/نرخ از middleware
//! و [`require_admin`])، اعتبارسنجی بدنهٔ وایر ([`parse_body`] — مثل zod در
//! لبهٔ HTTP)، و اجرای دامنه از مسیر خالص + [`V3Store`].
//!
//! هیچ هندلری مستقیم JSON نمی‌سازد — پاسخ‌ها از DTOهای فاز ۳ ساخته می‌شوند تا
//! قرارداد هرگز از هندلر فاصله نگیرد (ADR-005).

pub mod accounts;
pub mod admin;
pub mod agents;
pub mod ai;
pub mod arda;
pub mod auth;
pub mod bot;
pub mod builder;
pub mod chains;
pub mod deposits;
pub mod escrows;
pub mod hermes;
pub mod market;
pub mod monopoly;
pub mod nation;
pub mod quant;
pub mod studio;
pub mod system;
pub mod wallet;

use serde::de::DeserializeOwned;
use sha2::{Digest, Sha256};

use crate::error::ApiFailure;

/// پالایش نویسه‌های وایر — هر نویسهٔ بیرون از `^[A-Za-z0-9:_.-]+$` به `_`
/// تبدیل می‌شود تا خروجی همیشه الگوی `Id` را پاس کند (D2 — بازرسی فارنزیک).
fn sanitize_wire_chars(input: &str) -> String {
    input
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, ':' | '_' | '.' | '-') {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// پسوند قطعی ۱۶-رقمی SHA-256 برای شناسه‌های سرریز — تابع خالص از ورودی،
/// پس برای یک `raw_id` ثابت همیشه همان پسوند تولید می‌شود (بدون رانش اتمی).
fn deterministic_id_suffix(raw_id: &str) -> String {
    let digest = Sha256::digest(raw_id.as_bytes());
    let hex = format!("{digest:x}");
    hex.chars().take(16).collect()
}

/// D2 — شناسهٔ وایرِ امن با پیشوند — جایگزینِ `format!("{prefix}:{id}")`.
///
/// شناسهٔ DB-مبدأ می‌تواند تا ۱۲۸ کاراکتر باشد؛ افزودن پیشوندِ `hitl:` بر
/// آن، سقف `Id::new` را می‌شکند و `.expect` در `admin.rs` کل سرور را
/// panic می‌کند. این تابع تضمین می‌کند خروجی در `1..=128` کاراکتر و منطبق
/// بر الگوی `^[A-Za-z0-9:_.-]+$` است:
/// - اگر `prefix:sanitized` جا شود، همان برگردانده می‌شود؛
/// - وگرنه سرِ برش‌خوردهٔ ≤۱۰۰ کاراکتر + `_{sha256_hex16}` قطعی.
#[must_use]
pub fn safe_prefixed_wire_id(prefix: &str, raw_id: &str) -> String {
    let p = sanitize_wire_chars(prefix);
    let body = sanitize_wire_chars(raw_id);
    let direct = format!("{p}:{body}");
    if (1..=128).contains(&direct.chars().count()) {
        return direct;
    }
    // سرِ برش‌خورده: حداکثر ۱۰۰ نویسه از `prefix:body` + پسوند ۱۷ نویسه‌ای
    // (`_` + ۱۶ hex) → کل همیشه ≤ ۱۱۷ و ≥ ۱ — همیشه معتبر برای `Id::new`.
    let head: String = direct.chars().take(100).collect();
    format!("{head}_{}", deterministic_id_suffix(raw_id))
}

/// D2 — تبدیل امنِ هر شناسهٔ DB-مبدأ به `Id` بدون هیچ `.expect` روی دادهٔ
/// ناهمگون. رشتهٔ خالی/سفید → شناسهٔ ایستای `id:{fallback_tag}`؛ نویسه‌ها
/// پالایش می‌شوند؛ سرریز طول با پسوند قطعی ۱۶-hex نجات می‌یابد — سرور هرگز
/// روی ردیفِ خرابِ DB از کار نمی‌افتد.
///
/// `raw` به‌صورت `impl Into<String>` پذیرفته می‌شود — فراخوانیِ با `String`
/// (شناسه‌هایِ DB-مبدأ) بدون تغییر کار می‌کند.
///
/// # Panics
/// تنها در مسیرِ غیرقابلِ رسیدنِ fallback ایستا — وقتی حتی
/// `id:sanitized-fallback` (که الگو و طولش ثابت و معتبر است) رد شود؛
/// برای هر ورودیِ واقعی غیرِممکن است.
#[must_use]
pub fn safe_wire_id(raw: impl Into<String>, fallback_tag: &str) -> kian_contracts::Id {
    let raw = raw.into();
    let tag = sanitize_wire_chars(fallback_tag);
    let static_fallback = || {
        kian_contracts::Id::new(format!("id:{tag}")).unwrap_or_else(|| {
            kian_contracts::Id::new("id:sanitized-fallback").expect("valid static fallback")
        })
    };
    if raw.trim().is_empty() {
        return static_fallback();
    }
    let sanitized = sanitize_wire_chars(&raw);
    if let Some(id) = kian_contracts::Id::new(sanitized.clone()) {
        return id;
    }
    // پس از پالایش، تنها حالتِ نامعتبر سرریزِ طول است — نجاتِ قطعی:
    let salvaged = safe_prefixed_wire_id("id", &sanitized);
    kian_contracts::Id::new(salvaged).unwrap_or_else(static_fallback)
}

/// اعتبارسنجی بدنهٔ وایر — همان نقطهٔ zod در TS: بدنهٔ نامعتبر → 400
/// `INVALID_REQUEST` با شمارش issue (serde یک خطای ریشه‌ای می‌دهد — حداقلِ
/// صادقانه ۱). بدنهٔ خالی برای فرم‌های all-optional مثل `{}` تفسیر می‌شود.
///
/// # Errors
/// [`HttpFailure::invalid_request`] — وضعیت ۴۰۰ با `meta.issues`.
pub fn parse_body<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, ApiFailure> {
    let raw: &[u8] = if bytes.iter().all(u8::is_ascii_whitespace) {
        b"{}"
    } else {
        bytes
    };
    serde_json::from_slice(raw).map_err(|_| kian_contracts::HttpFailure::invalid_request(1).into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use kian_contracts::FundEscrowRequest;

    #[test]
    fn empty_body_becomes_default_object_for_optional_forms() {
        let req: FundEscrowRequest = parse_body(b"").unwrap();
        assert_eq!(req, FundEscrowRequest::default());
    }

    #[test]
    fn invalid_body_is_invalid_request_400() {
        let err = parse_body::<FundEscrowRequest>(b"{\"idempotencyKey\":\"short\"}").unwrap_err();
        assert_eq!(err.status(), 400);
        assert_eq!(err.envelope().error.code, "INVALID_REQUEST");
    }

    // ── D2 — بازرسی فارنزیک: شناسهٔ وایر امن ────────────────────────────────

    /// شناسهٔ ۱۲۸-کاراکتری اسکرو (سقف قانونی `Id::new`) با پیشوند `hitl:`
    /// می‌شکند ۱۳۳ — باید قطعی و بدون panic نجات یابد و `Id::new` پاس شود.
    #[test]
    fn hitl_wire_id_survives_128_char_escrow_id_without_panicking() {
        let escrow_id = format!("escrow:{}", "a1".repeat(61)); // ۱۲۹ کاراکتر، سپس برش به سقفِ قانونی ۱۲۸
        let escrow_id = escrow_id.chars().take(128).collect::<String>();
        assert_eq!(escrow_id.chars().count(), 128);
        let hitl = safe_prefixed_wire_id("hitl", &escrow_id);
        assert!(
            (1..=128).contains(&hitl.chars().count()),
            "HITL wire id must stay within 1..=128, got {}",
            hitl.chars().count()
        );
        let parsed = kian_contracts::Id::new(hitl.clone());
        assert!(parsed.is_some(), "sanitized HITL id must parse as wire Id: {hitl}");
        // قطعیت: ورودی ثابت → خروجی ثابت
        assert_eq!(hitl, safe_prefixed_wire_id("hitl", &escrow_id));
        // پیشوندِ معنایی حفظ می‌شود تا اپراتور HITL هنوز آن را می‌شناسد
        assert!(hitl.starts_with("hitl:"));
    }

    #[test]
    fn short_ids_pass_through_unchanged() {
        assert_eq!(safe_prefixed_wire_id("hitl", "escrow-123"), "hitl:escrow-123");
        assert_eq!(safe_prefixed_wire_id("hitl", "a".repeat(122).as_str()).chars().count(), 127);
    }

    #[test]
    fn hostile_chars_are_sanitized_and_oversize_ids_recovered_deterministically() {
        let hostile = "escrow/id با فاصله و یونیکد!";
        let safe = safe_prefixed_wire_id("hitl", hostile);
        assert!(safe.starts_with("hitl:"));
        assert!(kian_contracts::Id::new(safe).is_some());
        // سرریز ۵۰۰ کاراکتری → ≤ ۱۲۸ با پسوند قطعی ۱۶-hex
        let huge = "x".repeat(500);
        let salvaged = safe_wire_id(huge.clone(), "audit");
        assert!(salvaged.get().chars().count() <= 128);
        assert!(salvaged.get().chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, ':' | '_' | '.' | '-')));
        assert_eq!(safe_wire_id(huge, "audit"), salvaged, "salvage must be deterministic");
        // رشتهٔ تهی → fallback ایستای معتبر، بدون panic
        assert_eq!(safe_wire_id(String::new(), "deposit").get(), "id:deposit");
    }
}
