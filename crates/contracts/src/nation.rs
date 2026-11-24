//! قراردادهای وایر پارلمان M5 — لایحه، رأی، بستن و اجرا (حاکمیت HITL).
//!
//! شکل سیمی camelCase با null صریح؛ شناسه‌ها همان الگوی اسکرو (`bill:…`).
//! منطق گذارها در `kian-nation` — اینجا فقط DTO با deriveهای utoipa.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::common::Din;

/// نوع لایحه روی سیم — `escrowRelease` / `treasuryMove`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum BillKind {
    /// حل اختلاف اسکرو — اجرا یعنی release/refund با `approvedByAdmin: true`.
    EscrowRelease,
    /// جابه‌جایی خزانه — اجرا یعنی posting دوطرفهٔ خزانه → مقصد.
    TreasuryMove,
}

/// نتیجهٔ پیشنهادی لایحهٔ حل اختلاف — `release` / `refund`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum EscrowResolution {
    /// تسویهٔ ۹۰/۱۰ به نفع فروشنده (مادهٔ ۲).
    Release,
    /// بازگشت کامل به خریدار، بدون مالیات.
    Refund,
}

/// درخواست باز کردن لایحه — `createBillRequest`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateBillRequest {
    /// نوع لایحه
    pub kind: BillKind,
    /// موضوع: شناسهٔ اسکرو یا حساب مقصد (۱..=۱۲۸)
    #[schema(max_length = 128)]
    pub subject: String,
    /// مبلغ به دیین — برای `EscrowRelease` اجباری؛ برای `TreasuryMove` اجباری و > ۰
    #[serde(default)]
    #[schema(nullable = false)]
    pub amount_din: Option<Din>,
    /// نتیجهٔ پیشنهادی — فقط برای `EscrowRelease` اجباری
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<EscrowResolution>,
    /// عنوان لایحه (۱..=۲۰۰)
    #[schema(max_length = 200)]
    pub title: String,
}

/// درخواست رأی — رأی‌دهنده از نشست bearer می‌آید (هویت هرگز از بدنه).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CastVoteRequest {
    /// موافق؟
    pub approve: bool,
}

/// نمای لایحه روی سیم — `billView`؛ شمارش‌ها شفاف (مادهٔ ۶).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BillView {
    /// شناسه لایحه (`bill:…`)
    pub id: String,
    /// نوع لایحه
    pub kind: BillKind,
    /// موضوع (اسکرو/حساب)
    pub subject: String,
    /// مبلغ دیین (۰ اگر بی‌مبلغ)
    pub amount_din: i64,
    /// نتیجهٔ پیشنهادی — فقط حل اختلاف
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<EscrowResolution>,
    /// عنوان
    pub title: String,
    /// وضعیت: `OPEN` / `PASSED` / `REJECTED` / `EXECUTED`
    pub status: String,
    /// دلیل قطعی پس از بستن: `passed` / `quorum_not_met` / `majority_no` / `human_approval_missing`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision_reason: Option<String>,
    /// زمان باز شدن
    pub opened_at: String,
    /// زمان بستن — تا قبل از تکلیف null صریح
    pub closed_at: Option<String>,
    /// زمان اجرا — تا قبل از اجرا null صریح
    pub executed_at: Option<String>,
    /// رأی موافق
    pub votes_yes: i64,
    /// رأی مخالف
    pub votes_no: i64,
    /// رأی انسانی موافق (گیت مادهٔ ۷)
    pub human_votes_yes: i64,
    /// آیا مادهٔ ۷ این لایحه را انسانی می‌خواهد؟ (مبلغ ≥ ۵۰,۰۰۰ Đ)
    pub hitl_required: bool,
}

/// پاسخ فهرست لایحه‌ها — `billListResponse`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BillListResponse {
    /// لایحه‌ها به ترتیب زمانی (جدیدترین اول)
    pub items: Vec<BillView>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{from_str, to_string};

    #[test]
    fn bill_wire_shape_is_camel_case() {
        let json = r#"{"kind":"escrowRelease","subject":"esc:1","amountDin":60000,"resolution":"release","title":"حل اختلاف"}"#;
        let req: CreateBillRequest = from_str(json).unwrap();
        assert_eq!(req.kind, BillKind::EscrowRelease);
        assert_eq!(req.resolution, Some(EscrowResolution::Release));
        assert_eq!(req.amount_din.map(Din::get), Some(60_000));
        assert_eq!(to_string(&req).unwrap(), json);
    }

    #[test]
    fn bill_view_omits_absent_optionals() {
        let view = BillView {
            id: "bill:1".into(),
            kind: BillKind::TreasuryMove,
            subject: "citizen:7".into(),
            amount_din: 10_000,
            resolution: None,
            title: "جابه‌جایی خزانه".into(),
            status: "OPEN".into(),
            decision_reason: None,
            opened_at: "2026-11-21T00:00:00.000Z".into(),
            closed_at: None,
            executed_at: None,
            votes_yes: 2,
            votes_no: 1,
            human_votes_yes: 1,
            hitl_required: false,
        };
        let s = to_string(&view).unwrap();
        assert!(!s.contains("resolution"), "resolution absent, not null");
        assert!(s.contains("\"closedAt\":null"), "closedAt explicit null");
        assert!(!s.contains("\"decisionReason\""));
    }
}
