//! قراردادهای وایر کیف پول چهارزنجیره M5 — متادیتا زنجیره، آدرس واریز کاربر و
//! واریز USDT (اثبات کاربر → تأیید ادمین → ضرب مادهٔ ۳ با نرخ USDT).
//!
//! همان قراردادهای M3 برای واریز TON — این‌بار روی چهار زنجیره؛ اعتبارسنجی
//! آدرس با چک‌سام‌های رسمی در `kian-wallet` (سرور؛ کلاینت هرگز اعتماد نمی‌شود).

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// نمای یک زنجیره — `chainView` (متادیتای عمومی؛ بدون هیچ secret).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChainView {
    /// برچسب زنجیره: TRX / SOL / TON / BSC
    pub chain: String,
    /// نام فارسی
    pub fa_name: String,
    /// قرارداد کانونیک USDT روی این زنجیره
    pub usdt_contract: String,
    /// واحد on-chain USDT (۶ روی TRX/SOL/TON؛ ۱۸ روی BSC)
    pub usdt_onchain_decimals: u32,
}

/// پاسخ فهرست زنجیره‌ها — `chainsResponse`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChainsResponse {
    /// هر چهار زنجیره به ترتیب ثابت
    pub items: Vec<ChainView>,
}

/// درخواست ثبت/به‌روزرسانی آدرس واریز کاربر برای یک زنجیره.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SetDepositAddressRequest {
    /// زنجیره: TRX / SOL / TON / BSC
    #[schema(max_length = 4)]
    pub chain: String,
    /// آدرس — سمت سرور با چک‌سام رسمی زنجیره اعتبارسنجی می‌شود
    #[schema(max_length = 96)]
    pub address: String,
}

/// نمای آدرس ثبت‌شده کاربر — `depositAddressView`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DepositAddressView {
    /// زنجیره
    pub chain: String,
    /// آدرس
    pub address: String,
    /// زمان ثبت ISO
    pub created_at: String,
}

/// پاسخ فهرست آدرس‌های من — `myAddressesResponse`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MyAddressesResponse {
    /// آدرس‌های ثبت‌شده کاربر (حداکثر چهار زنجیره)
    pub items: Vec<DepositAddressView>,
}

/// درخواست ثبت واریز USDT — اثبات کاربر تا راستی‌آزمایی ادمین.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UsdtDepositCreateRequest {
    /// زنجیره: TRX / SOL / TON / BSC
    #[schema(max_length = 4)]
    pub chain: String,
    /// هش تراکنش زنجیره — UNIQUE روی (chain, txHash)
    #[schema(min_length = 8, max_length = 128)]
    pub tx_hash: String,
    /// آدرس فرستنده — با قواعد همان زنجیره اعتبارسنجی می‌شود
    #[schema(max_length = 96)]
    pub from_address: String,
    /// مبلغ USDT رشتهٔ دهدهی با ≤۶ رقم اعشار (مقیاس سیم: micro-USDT)
    #[schema(max_length = 24)]
    pub amount_usdt: String,
}

/// نمای واریز USDT روی سیم — `usdtDepositView`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UsdtDepositView {
    /// شناسه (`udep:…`)
    pub id: String,
    /// زنجیره
    pub chain: String,
    /// هش تراکنش
    pub tx_hash: String,
    /// آدرس فرستنده
    pub from_address: String,
    /// مبلغ رشتهٔ دهدهی USDT
    pub amount_usdt: String,
    /// PENDING / CONFIRMED / REJECTED
    pub status: String,
    /// زمان ثبت ISO
    pub created_at: String,
    /// زمان تصمیم — تا قبل از تکلیف null صریح
    pub confirmed_at: Option<String>,
}

/// پاسخ فهرست واریزهای USDT — `usdtDepositListResponse`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UsdtDepositListResponse {
    /// واریزهای من (۵۰ آخر)
    pub items: Vec<UsdtDepositView>,
}

/// نمای خزانه یک زنجیره — فقط آدرس تنظیم‌شده از env؛ خالی یعنی تنظیم‌نشده.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TreasuryChainView {
    /// زنجیره
    pub chain: String,
    /// آدرس خزانه — رشتهٔ خالی یعنی تنظیم‌نشده (شفاف، نه مخفی)
    pub address: String,
    /// آیا آدرس با قواعد زنجیره معتبر است؟
    pub configured: bool,
}

/// پاسخ شفافیت خزانه — `treasuryResponse` (مادهٔ ۶؛ عمومی).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TreasuryResponse {
    /// وضعیت خزانه هر چهار زنجیره
    pub items: Vec<TreasuryChainView>,
    /// سیاست حراست — متن قراردادی ثابت
    pub movement_policy: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{from_str, to_string};

    #[test]
    fn usdt_deposit_wire_shape_is_camel_case() {
        let json = r#"{"chain":"BSC","txHash":"0xabc123def4567890","fromAddress":"0x55d398326f99059fF775485246999027B3197955","amountUsdt":"12.5"}"#;
        let req: UsdtDepositCreateRequest = from_str(json).unwrap();
        assert_eq!(req.chain, "BSC");
        assert_eq!(req.amount_usdt, "12.5");
        assert_eq!(to_string(&req).unwrap(), json);
    }

    #[test]
    fn usdt_deposit_view_keeps_explicit_null() {
        let view = UsdtDepositView {
            id: "udep:1".into(),
            chain: "TON".into(),
            tx_hash: "abc12345".into(),
            from_address: "EQCxE6mUtQJKFnGfaROTKOt1lZbDiiX1kCixRv7Nw2Id_sDs".into(),
            amount_usdt: "1.5".into(),
            status: "PENDING".into(),
            created_at: "2026-11-21T00:00:00.000Z".into(),
            confirmed_at: None,
        };
        let s = to_string(&view).unwrap();
        assert!(
            s.contains("\"confirmedAt\":null"),
            "explicit null until decided"
        );
    }
}
