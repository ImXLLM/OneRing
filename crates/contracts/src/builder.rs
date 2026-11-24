//! قراردادهای وایر استودیوی ساخت محصول HTML با هوش مصنوعی (فاز ۴).
//!
//! همهٔ مسیرهای این ماژول با نشست Bearer شهروند کار می‌کنند:
//! `POST /api/v1/builder/sessions`، `GET /api/v1/builder/sessions`،
//! `GET /api/v1/builder/sessions/{id}`، `POST /api/v1/builder/sessions/{id}/chat`
//! و `POST /api/v1/builder/sessions/{id}/publish`.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::common::Din;
use crate::market::ProductView;

/// یک پیام در تاریخچهٔ گفتگوی سازندهٔ محصول.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BuilderMessageView {
    /// شناسهٔ افزایشی پیام
    pub id: i64,
    /// نقش (`user` / `assistant`)
    pub role: String,
    /// متن پیام
    pub content: String,
    /// زمان ثبت (Unix epoch ثانیه)
    pub created_at: i64,
}

/// خلاصهٔ یک جلسه برای فهرست جلسات کاربر.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BuilderSessionSummary {
    /// شناسهٔ جلسه (`bldr:{uuid}`)
    pub id: String,
    /// مرحله (`discovery` / `ready` / `published`)
    pub stage: String,
    /// عنوان پیشنهادی
    pub title: String,
    /// توضیح فروش پیشنهادی
    pub description: String,
    /// قیمت پیشنهادی به دیین
    pub suggested_price: i64,
    /// نرخ پورسانت معرف (`0..=5000`)
    pub affiliate_bps: u16,
    /// آیا کد HTML تولید شده است؟
    pub has_html: bool,
    /// شناسهٔ محصول منتشرشده (در صورت انتشار)
    pub published_product_id: Option<String>,
    /// زمان ایجاد (Unix epoch ثانیه)
    pub created_at: i64,
    /// زمان آخرین تغییر (Unix epoch ثانیه)
    pub updated_at: i64,
}

/// نمای کامل یک جلسهٔ سازنده (همراه تاریخچهٔ پیام‌ها، کد HTML و پیش‌نمایش).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BuilderSessionView {
    /// شناسهٔ جلسه (`bldr:{uuid}`)
    pub id: String,
    /// مرحله (`discovery` / `ready` / `published`)
    pub stage: String,
    /// عنوان پیشنهادی محصول
    pub title: String,
    /// توضیح فروش (فرمول CAST)
    pub description: String,
    /// قیمت پیشنهادی به دیین
    pub suggested_price: i64,
    /// نرخ پورسانت معرف (`0..=5000`)
    pub affiliate_bps: u16,
    /// کد کامل HTML تک‌صفحه‌ای تولیدشده
    pub generated_html: String,
    /// نسخهٔ پیش‌نمایش امن HTML (≤ ۵۰ KB)
    pub preview_html: String,
    /// شناسهٔ محصول منتشرشده در بازار (در صورت انتشار)
    pub published_product_id: Option<String>,
    /// تاریخچهٔ پیام‌های گفتگو به ترتیب زمانی
    pub messages: Vec<BuilderMessageView>,
    /// زمان ایجاد (Unix epoch ثانیه)
    pub created_at: i64,
    /// زمان آخرین تغییر (Unix epoch ثانیه)
    pub updated_at: i64,
}

/// فهرست جلسات سازندهٔ کاربر.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BuilderSessionListResponse {
    /// جلسات کاربر — جدیدترین اول
    pub items: Vec<BuilderSessionSummary>,
}

/// درخواست ساخت جلسهٔ جدید استودیو (بدنهٔ خالی `{}` نیز مجاز است).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateBuilderSessionRequest {
    /// عنوان یا ایدهٔ اولیهٔ اختیاری
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_idea: Option<String>,
}

/// درخواست ارسال پیام جدید در جلسهٔ سازنده.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BuilderChatRequest {
    /// پیام کاربر (۱..=۴۰۰۰ کاراکتر)
    pub message: String,
    /// اجبار به تولید فوری کد HTML بدون پرسیدن سوال اضافه
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_generate: Option<bool>,
}

/// پاسخ یک نوبت چت با استودیوی ساخت محصول.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BuilderChatResponse {
    /// پاسخ متنی دستیار AI
    pub reply: String,
    /// وضعیت به‌روزرسانی‌شدهٔ جلسه (شامل پیام‌ها و کد HTML جدید)
    pub session: BuilderSessionView,
    /// آیا مولد تعاملی جایگزین داخلی اجرا شده است؟
    pub used_fallback: bool,
    /// نام اسلات ارائه‌دهنده (در صورت استفاده از LLM خارجی)
    pub provider: Option<String>,
    /// مجموع توکن مصرف‌شده در این نوبت
    pub total_tokens: i64,
}

/// درخواست انتشار یک‌کلیکهٔ محصول ساخته‌شده در بازار کیان.
///
/// همهٔ فیلدها اختیاری‌اند؛ اگر ارسال نشوند از مقادیر پیشنهادی AI روی جلسه استفاده می‌شود.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PublishBuilderRequest {
    /// ویرایش اختیاری عنوان محصول
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// ویرایش اختیاری توضیح فروش
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// ویرایش اختیاری قیمت به دیین
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_din: Option<Din>,
    /// ویرایش اختیاری نرخ پورسانت معرف (`0..=5000`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub affiliate_bps: Option<u16>,
    /// دستهٔ اختیاری محصول
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}

/// پاسخ انتشار یک‌کلیکه — شامل محصول منتشرشده و پارامتر لینک رفرال تلگرام.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PublishBuilderResponse {
    /// محصول ثبت‌شده در بازار
    pub product: ProductView,
    /// شناسهٔ جلسهٔ سازنده
    pub session_id: String,
    /// پارامتر آمادهٔ لینک وایرال و رفرال مینی‌اپ (`p_{product_id}_r_{tg_id}`)
    pub share_start_param: String,
    /// اگر جلسه قبلاً منتشر شده بود، `true` و همان محصول بازگردانده می‌شود
    pub already_published: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{from_str, to_string};

    #[test]
    fn builder_chat_and_publish_dtos_round_trip() {
        let req: BuilderChatRequest =
            from_str(r#"{"message":"یک ماشین‌حساب بساز","forceGenerate":true}"#).unwrap();
        assert_eq!(req.force_generate, Some(true));
        let pub_req: PublishBuilderRequest = from_str("{}").unwrap();
        assert_eq!(pub_req, PublishBuilderRequest::default());

        let summary = BuilderSessionSummary {
            id: "bldr:1".into(),
            stage: "ready".into(),
            title: "ماشین‌حساب".into(),
            description: "توضیح".into(),
            suggested_price: 250,
            affiliate_bps: 1_500,
            has_html: true,
            published_product_id: None,
            created_at: 1_700_000_000,
            updated_at: 1_700_000_100,
        };
        let json = to_string(&summary).unwrap();
        assert!(json.contains("\"hasHtml\":true"));
        assert!(json.contains("\"affiliateBps\":1500"));
        assert!(json.contains("\"publishedProductId\":null"));
    }
}
