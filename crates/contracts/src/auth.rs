//! قراردادهای هویت M3 — احراز Telegram initData، نشست‌ها و واریز TON.
//!
//! هویت هرگز از بدنهٔ درخواست مشتق نمی‌شود — فقط از `initData` امضاشدهٔ
//! `Telegram.WebApp` (طرح رسمی HMAC در [`crate::security`]). نشست توکنی
//! اپاک است و با `Authorization: Bearer` می‌آید.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::common::Din;

/// درخواست احراز تلگرام — رشتهٔ کامل `initData` خام از `Telegram.WebApp`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TelegramAuthRequest {
    /// `window.Telegram.WebApp.initData` — بدون هیچ دست‌کاری
    pub init_data: String,
}

/// نمای کاربر مینی‌اپ — هویت تلگرام متصل به حساب دفتر دوطرفه.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserView {
    /// شناسه پایدار (`citizen:{telegram_id}`)
    pub id: String,
    /// شناسهٔ عددی تلگرام (رشته — حفاظ 2^53−1)
    pub telegram_id: String,
    /// یوزرنیم تلگرام بدون `@` — اختیاری (کلید غایب روی سیم)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// حساب دفتر متصل به این هویت
    pub account_id: String,
    /// آدرس ولت TON متصل — `null` صریح تا وقتی TON Connect وصل شود
    pub ton_address: Option<String>,
    /// زمان ثبت ISO
    pub created_at: String,
}

/// پاسخ احراز موفق — توکن نشست + نمای کاربر.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthResponse {
    /// توکن نشست (۶۴ hex) — در `Authorization: Bearer` برگردانده می‌شود
    pub token: String,
    /// انقضای توکن ISO
    pub expires_at: String,
    /// کاربر احرازشده
    pub user: UserView,
}

/// ردیف تاریخچهٔ کیف روی وایر — مبلغ علامت‌دار (بدهکار منفی).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WalletTxView {
    /// شناسه تراکنش دفتر
    pub tx_id: String,
    /// مبلغ علامت‌دار به دیین — منفی یعنی خروج از حساب
    pub amount: i64,
    /// یادداشت — `null` صریح اگر ثبت نشده
    pub memo: Option<String>,
    /// زمان ISO
    pub at: String,
}

/// وضعیت من کامل — هویت + تراز مشتق‌شده + تاریخچهٔ اخیر.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MeResponse {
    /// کاربر احرازشده
    pub user: UserView,
    /// تراز آزاد دیین (SUM دفتر — ADR-003)
    pub balance: Din,
    /// دیین قفل‌شده در اسکروهای خریدار
    pub escrow_locked: Din,
    /// ۱۰ ردیف آخر دفتر من
    pub recent_txs: Vec<WalletTxView>,
}

/// درخواست اتصال آدرس TON به هویت — از TON Connect می‌آید.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SetTonAddressRequest {
    /// آدرس wallet TON (فرمت پایهٔ ۵۱ کاراکتری EQ/UQ یا فرندلی UQ..)
    pub address: String,
}

/// درخواست ورود دمو/توسعه‌دهنده — فقط در حالت dev (`POST /auth/dev-login`).
///
/// جایگزین تلگرام نیست؛ فقط برای تستِ محلیِ حلقهٔ کامل (سازنده → انتشار →
/// رفرال → خرید → اسکرو → برداشت) در مرورگر بدون تلگرام. همهٔ فیلدها غیر از
/// `tgUserId` اختیاری‌اند با پیش‌فرض‌های معقول؛ `firstName` فقط نمایشی است و
/// ذخیره نمی‌شود (هویت = `citizen:{tgUserId}`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct DevLoginRequest {
    /// شناسهٔ عددی تلگرام آزمایشی — فقط ارقام (۱..=۲۰ رقم)؛ پیش‌فرض پرسونای سازنده
    pub tg_user_id: String,
    /// یوزرنیم آزمایشی اختیاری — بدون `@`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// نام نمایشی اختیاری — فقط برای نمایش؛ روی هویت ذخیره نمی‌شود
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    /// پارامتر شروع وایرال اختیاری (`p_<productId>_r_<referrer>`)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_param: Option<String>,
}

/// پیش‌فرض `DevLoginRequest` — پرسونای سازندهٔ دمو (۱۰۰۱).
impl Default for DevLoginRequest {
    fn default() -> Self {
        Self {
            tg_user_id: "1001".to_owned(),
            username: None,
            first_name: None,
            start_param: None,
        }
    }
}

/// مقدار پیش‌فرض شیرِ اعتبار آزمایشی — ۵٬۰۰۰ KIAN به micro.
pub const FAUCET_DEFAULT_MICRO: i64 = 5_000_000_000;

/// سقف هر برداشت از شیر آزمایشی — ۵۰٬۰۰۰ KIAN به micro.
pub const FAUCET_MAX_MICRO: i64 = 50_000_000_000;

/// درخواست شیرِ اعتبار آزمایشی (`POST /wallet/faucet`) — فقط حالت dev.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct FaucetRequest {
    /// مبلغ به micro-KIAN — پیش‌فرض ۵٬۰۰۰ KIAN؛ سقف ۵۰٬۰۰۰ KIAN
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount_micro: Option<i64>,
}

/// درخواست ثبت واریز TON — اثبات انتقال از سمت کاربر.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TonDepositCreateRequest {
    /// هش تراکنش TON (۶۴ hex یا base64 — طول ۸..=۱۲۸)
    pub tx_hash: String,
    /// آدرس فرستنده — همان ولت TON Connect
    pub wallet_address: String,
    /// مبلغ TON به‌صورت رشتهٔ دهدهی (مثل "0.50") — هرگز float روی سیم
    pub amount_ton: String,
}

/// نمای واریز TON — وضعیت رفت‌وبرگشت کامل.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TonDepositView {
    /// شناسه (`dep:{uuid}`)
    pub id: String,
    /// هش تراکنش زنجیره
    pub tx_hash: String,
    /// آدرس فرستنده
    pub wallet_address: String,
    /// مبلغ TON رشتهٔ دهدهی
    pub amount_ton: String,
    /// `PENDING` / `CONFIRMED` / `REJECTED`
    pub status: String,
    /// زمان ثبت ISO
    pub created_at: String,
    /// زمان تصمیم ISO — `null` صریح تا وقتی ادمین تعیین تکلیف کند
    pub confirmed_at: Option<String>,
}

/// فهرست واریزهای من.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TonDepositListResponse {
    /// ردیف‌ها — جدیدترین اول
    pub items: Vec<TonDepositView>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{from_str, to_string};

    #[test]
    fn me_response_null_ton_address_is_explicit() {
        let me = MeResponse {
            user: UserView {
                id: "citizen:1".into(),
                telegram_id: "1".into(),
                username: None,
                account_id: "citizen:1".into(),
                ton_address: None,
                created_at: "2026-01-01T00:00:00.000Z".into(),
            },
            balance: Din::new(100).unwrap(),
            escrow_locked: Din::new(0).unwrap(),
            recent_txs: vec![],
        };
        let json = to_string(&me).unwrap();
        assert!(json.contains("\"tonAddress\":null"), "explicit null — {json}");
        assert!(!json.contains("username"), "optional absent — {json}");
    }

    #[test]
    fn dev_login_request_defaults_and_camel_case() {
        // بدنهٔ خالی → پرسونای پیش‌فرض سازنده
        let req: DevLoginRequest = from_str("{}").unwrap();
        assert_eq!(req.tg_user_id, "1001");
        assert_eq!(req.username, None);
        // شکل سیمی camelCase + فیلدهای غایب
        let full = DevLoginRequest {
            tg_user_id: "2002".into(),
            username: Some("buyer_demo".into()),
            first_name: Some("خریدار (Buyer)".into()),
            start_param: Some("p_prod1_r_3003".into()),
        };
        let json = to_string(&full).unwrap();
        assert!(json.contains("\"tgUserId\""), "{json}");
        assert!(json.contains("\"firstName\""), "{json}");
        assert!(json.contains("\"startParam\""), "{json}");
        assert!(json.contains("\"username\""), "{json}");
    }

    #[test]
    fn faucet_request_defaults_to_five_kian() {
        let req: FaucetRequest = from_str("{}").unwrap();
        assert_eq!(req.amount_micro, None);
        let parsed: FaucetRequest =
            from_str(r#"{"amountMicro":5000000000}"#).unwrap();
        assert_eq!(parsed.amount_micro, Some(5_000_000_000));
    }

    #[test]
    fn deposit_view_roundtrip_keeps_null_confirmed_at() {
        let raw = r#"{"id":"dep:1","txHash":"abc","walletAddress":"UQ","amountTon":"0.5","status":"PENDING","createdAt":"2026-01-01T00:00:00.000Z","confirmedAt":null}"#;
        let view: TonDepositView = from_str(raw).unwrap();
        assert_eq!(view.status, "PENDING");
        assert_eq!(view.confirmed_at, None);
    }
}
