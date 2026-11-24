//! قراردادهای کیف پول — ضرب و سوزاندن (ماده‌های ۳ و ۴ منشور) — پورت `wallet.ts`.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::common::{Din, Id, IdempotencyKey};

/// درخواست ضرب — `mintRequestSchema` (سقف سالانه در فاز ۴ اعمال می‌شود).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MintRequest {
    /// حساب مقصد
    pub account_id: Id,
    /// مبلغ دیین
    pub amount: Din,
    /// یادداشت اختیاری (≤۲۰۰)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(max_length = 200)]
    pub memo: Option<String>,
    /// کلید یکسان‌سازی اختیاری
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<IdempotencyKey>,
}

/// درخواست سوزاندن — `burnRequestSchema` (overdraft ناممکن است).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BurnRequest {
    /// حساب مبدأ
    pub account_id: Id,
    /// مبلغ دیین
    pub amount: Din,
    /// یادداشت اختیاری (≤۲۰۰)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(max_length = 200)]
    pub memo: Option<String>,
    /// کلید یکسان‌سازی اختیاری
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<IdempotencyKey>,
}

/// پاسخ جابه‌جایی کیف پول — `walletMoveResponseSchema`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WalletMoveResponse {
    /// شناسهٔ تراکنش
    pub tx_id: String,
    /// حساب
    pub account_id: String,
    /// موجودی پس از عملیات
    #[schema(minimum = -9_007_199_254_740_991_i64, maximum = 9_007_199_254_740_991_i64)]
    pub balance: i64,
    /// آیا این پاسخ replay بود؟
    pub replayed: bool,
}

/// درخواست ثبت برداشت کریپتویی توسط شهروند (`POST /api/v1/wallet/withdrawals`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateWithdrawalRequest {
    /// زنجیره مقصد (`TON` / `TRX` / `SOL` / `BSC`)
    pub chain: String,
    /// دارایی مقصد (`TON` یا `USDT`)
    pub asset: String,
    /// آدرس کیف پول مقصد (چک‌سام رسمی توسط `kian-wallet`)
    pub destination_address: String,
    /// مبلغ برداشتی به دیین (حداقل ۱)
    pub amount_din: Din,
}

/// درخواست تأیید پرداخت برداشت توسط ادمین (`POST /api/v1/wallet/withdrawals/{id}/confirm`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmWithdrawalRequest {
    /// هش تراکنش پرداخت‌شده روی زنجیره (۸..=۱۲۸ کاراکتر)
    pub tx_hash: String,
}

/// نمای یک درخواست برداشت روی سیم.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawalRequestView {
    /// شناسه (`wdr:{uuid}`)
    pub id: String,
    /// حساب شهروند (`citizen:{tg_id}`)
    pub account_id: String,
    /// زنجیره (`TON`/`TRX`/`SOL`/`BSC`)
    pub chain: String,
    /// دارایی (`TON`/`USDT`)
    pub asset: String,
    /// آدرس مقصد
    pub destination_address: String,
    /// مبلغ به دیین
    pub amount_din: Din,
    /// هش تراکنش پرداخت — تا پیش از تأیید `null`
    pub tx_hash: Option<String>,
    /// وضعیت (`PENDING` / `CONFIRMED` / `REJECTED`)
    pub status: String,
    /// زمان ایجاد (Unix epoch ثانیه)
    pub created_at: i64,
    /// زمان تصمیم ادمین (Unix epoch ثانیه)
    pub resolved_at: Option<i64>,
}

/// فهرست درخواست‌های برداشت.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawalListResponse {
    /// درخواست‌ها — جدیدترین اول
    pub items: Vec<WithdrawalRequestView>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{from_str, to_string};

    #[test]
    fn mint_request_wire_shape_matches_ts() {
        let json = r#"{"accountId":"user:ali","amount":500,"memo":"welcome","idempotencyKey":"mint-ali-1"}"#;
        let req: MintRequest = from_str(json).unwrap();
        assert_eq!(req.amount, Din::new(500).unwrap());
        assert_eq!(
            req.idempotency_key.as_ref().map(ToString::to_string),
            Some("mint-ali-1".into())
        );
        assert_eq!(to_string(&req).unwrap(), json);
    }

    #[test]
    fn burn_request_omits_optionals() {
        let req = BurnRequest {
            account_id: Id::new("user:x").unwrap(),
            amount: Din::new(250).unwrap(),
            memo: None,
            idempotency_key: None,
        };
        assert_eq!(to_string(&req).unwrap(), r#"{"accountId":"user:x","amount":250}"#);
    }
}
