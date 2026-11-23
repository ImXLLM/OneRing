//! گارد SQL امن — تنها نقطهٔ audit انسانیِ SQL داینامیک در `kian-db`.
//!
//! sqlx 0.9 هر رشتهٔ SQL غیر literal را با `E0277` مسدود می‌کند تا تزریق
//! ممکن صریحاً audit شود. در KIAN همهٔ ۸ نقطهٔ SQL داینامیک (فقط
//! `queries.rs`) از این یک گذرگاه می‌روند تا audit یک‌جا و قابل جست‌وجو
//! باشد: `rg "audited\(" crates/db/src`.
//!
//! قرارداد audit (هر تغییر اینجا باید این پنج بند را دوباره تأیید کند):
//! ۱. رشتهٔ پویا فقط از «قطعات ثابت داخل crate» ساخته می‌شود — نام جدول/ستون
//!    از whitelist enumها می‌آید (`HitlRequest`، `Product`، `AuditLog`، …)؛
//! ۲. هر مقدارِ وابسته به ورودی کاربر حتماً با `.bind()` پاس می‌شود و هرگز
//!    با `format!` داخل SQL درج نمی‌شود؛
//! ۳. فیلترهای وضعیت از enumهای سری‌شوندهٔ داخل crate می‌آیند (`as_str`
//!    روی `HitlStatus`/`ProductStatus` — نه رشتهٔ خام کلاینت)؛
//! ۴. مسیرهای فایل بکاپ فقط درون `data/backups/` با نام تولیدی سرور
//!    ساخته می‌شوند (بدون ورودی مسیر از کلاینت)؛
//! ۵. تست‌های یکپارچگی `integration_tests.rs` مسیرهای تزریق کلاسیک
//!    (`' OR 1=1 --`) را روی همین گذرگاه‌ها رد می‌کنند.
//!
//! پورت فلسفهٔ گارد `sqlsafe` در مخزن 10x — درس لگاسیِ محافظان امنیتی.
#![forbid(unsafe_code)]

use sqlx::AssertSqlSafe;

/// audit انسانی رشتهٔ SQL داینامیک — تنها راه عبور رشتهٔ غیر literal از گیت.
///
/// # قرارداد فراخوانی
/// فراخواننده باید بندهای ۱–۳ سند بالا را برای `sql` تضمین کند؛ این تابع
/// هیچ اعتبارسنجی زمان اجرا اضافه نمی‌کند چون قاعده «درجِ هرگزِ ورودی کاربر
/// در SQL» در سطح همهٔ فراخوانی‌ها برقرار است.
pub(crate) fn audited(sql: String) -> AssertSqlSafe<String> {
    AssertSqlSafe(sql)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// تقدسِ قرارداد — خروجی همان ورودی است؛ گارد فقط audit نقطه‌ای است.
    #[test]
    fn audited_passes_through_the_string_verbatim() {
        let sql = r#"SELECT "id" FROM "HitlRequest" WHERE "status" = ?"#;
        let wrapped = audited(sql.to_owned());
        assert_eq!(wrapped.0, sql);
    }
}
