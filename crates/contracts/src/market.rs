//! قراردادهای بازار M3 — محصول‌ها، خرید با اسکرو ۸ حالته و سفارش‌های من.
//!
//! خرید = ساخت امانت + تأمین آن در «یک» فرمان اتمی: Din خریدار قفل می‌شود و
//! فروشنده تحویل می‌دهد؛ مادهٔ ۸ منشور خودش مالیات ۱۰٪ را در `release` اعمال می‌کند.
//! محتوای تحویل (`deliveryPayload`) هرگز روی وایر عمومی نمی‌آید.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::common::{Din, EscrowStatus, Id, IdempotencyKey};
use crate::escrows::EscrowView;

/// سقف حجم محتوای تحویل محصول (۱۵۰ کیلوبایت برای آرتیفکت HTML تک‌صفحه‌ای تعاملی).
pub const MAX_DELIVERY_PAYLOAD_BYTES: usize = 150_000;

/// سقف حجم پیش‌نمایش عمومی HTML پیش از خرید (۵۰ کیلوبایت).
pub const MAX_PREVIEW_HTML_BYTES: usize = 50_000;

/// درخواست ثبت محصول در بازار.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateProductRequest {
    /// عنوان (۱..=۱۲۰ کاراکتر)
    pub title: String,
    /// توضیح (۱..=۲۰۰۰ کاراکتر)
    pub description: String,
    /// قیمت به دیین — حداقل ۱
    pub price_din: Din,
    /// دسته اختیاری
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// پیش‌نمایش امن HTML برای صفحهٔ محصول پیش از خرید (حداکثر ۵۰ KB)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview_html: Option<String>,
    /// درصد پورسانت معرف بر حسب نقاط پایه (پیش‌فرض ۱۰۰۰ = ۱۰٪، سقف ۵۰۰۰ = ۵۰٪)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub affiliate_bps: Option<u16>,
    /// نوع آرتیفکت تک‌صفحه‌ای (پیش‌فرض `"html"`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact_kind: Option<String>,
    /// محتوای تحویل کامل (حداکثر ۱۵۰ KB) — فقط به خریدار همان سفارش داده می‌شود
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivery_payload: Option<String>,
}

/// نمای عمومی محصول — بدون محتوای تحویل (`deliveryPayload`) و بدون نیاز به نشست.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProductView {
    /// شناسه (`prod:{uuid}`)
    pub id: String,
    /// حساب فروشنده (`citizen:{tg_id}`)
    pub seller_id: String,
    /// عنوان
    pub title: String,
    /// توضیح
    pub description: String,
    /// قیمت به دیین
    pub price_din: Din,
    /// دسته — `null` صریح اگر ثبت نشده
    pub category: Option<String>,
    /// پیش‌نمایش عمومی HTML
    pub preview_html: String,
    /// نرخ پورسانت معرف بر حسب نقاط پایه (`0..=5000`)
    pub affiliate_bps: u16,
    /// نوع آرتیفکت (`html` و…)
    pub artifact_kind: String,
    /// فعال بودن برای خرید
    pub active: bool,
    /// زمان ایجاد ISO
    pub created_at: String,
    /// نمرهٔ REA مایرون مُهر‌شده روی لیستینگ (`0..=100`؛ `0` = هنوز مُهر نشده)
    pub mairon_score: u8,
    /// نشان مرئی مایرون (خالی = مُهر نشده)
    pub mairon_badge: String,
}

/// فهرست محصولات فعال بازار.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProductListResponse {
    /// محصولات — جدیدترین اول
    pub items: Vec<ProductView>,
}

/// درخواست خرید — ساخت امانت + قفل Din خریدار در یک فرمان اتمی.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseRequest {
    /// محصول هدف
    pub product_id: Id,
    /// معرف اختیاری (`tg_id` عددی مثل `"12345"` یا `"citizen:12345"`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referrer: Option<String>,
    /// کلید یکسان‌سازی اختیاری — retry دوبار پول نمی‌گیرد
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<IdempotencyKey>,
}

/// پاسخ تحویل محتوای کامل محصول — فقط برای خریدار یا فروشندهٔ اسکرو پرداخت‌شده.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct EscrowDeliveryView {
    /// شناسه اسکرو
    pub escrow_id: String,
    /// شناسه محصول مرتبط
    pub product_id: String,
    /// عنوان محصول
    pub title: String,
    /// نوع آرتیفکت (`html` و…)
    pub artifact_kind: String,
    /// وضعیت فعلی اسکرو (`FUNDED` / `DELIVERED` / `RELEASED`)
    pub state: EscrowStatus,
    /// محتوای کامل محصول پولی
    pub delivery_payload: String,
}

/// پاسخ خرید — امانت CREATED→FUNDED در همان لحظه.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseResponse {
    /// امانت تأمین‌شده (FUNDED)
    pub escrow: EscrowView,
    /// تراکنش قفل Din — `null` صریح در replay بدون tx
    pub tx_id: Option<String>,
    /// آیا این پخش مجدد بود؟ (پول دوباره قفل نشد)
    pub replayed: bool,
}

/// سفارش‌های من + محصولات من — یک نگاه به کل حضور در بازار.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MyMarketResponse {
    /// محصول‌هایی که فروخته‌ام
    pub products: Vec<ProductView>,
    /// اسکروهایی که خریدار یا فروشندهٔ آن‌ها هستم
    pub orders: Vec<EscrowView>,
}

// ── مُهر مایرون و رأی ۵-ستارهٔ شهروندان (Wave 3 — Trinity marketplace gate) ──

/// پاسخ تأیید و انتشار دو-کلیدی (`POST /market/products/{id}/mairon-confirm`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MaironConfirmResponse {
    /// شناسهٔ محصول
    pub product_id: String,
    /// نمرهٔ REA مایرون (`0..=100`)
    pub mairon_rea_score: u8,
    /// نشان مرئی مُهرشده روی لیستینگ
    pub mairon_visible_badge: String,
    /// تأیید صریح سازنده — کلید دوم دروازهٔ دو-کلیدی
    pub creator_confirmed: bool,
    /// آیا لیستینگ با مُهر منتشر شد؟
    pub published: bool,
}

/// درخواست ثبت رأی شهروند روی محصول (`POST /market/products/{id}/reviews`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProductReviewSubmitRequest {
    /// ستاره — فقط `1..=5`
    pub stars: u8,
    /// نظر صحیح‌سنجی‌شده (`1..=2000` کاراکتر پس از trim)
    pub comment: String,
}

/// نمای یک رأی روی وایر.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProductReviewView {
    /// شناسهٔ یکتای رأی (`rev:{uuid}`)
    pub review_id: String,
    /// شناسهٔ محصول
    pub product_id: String,
    /// حساب شهروند رأی‌دهنده (`citizen:{tg_id}`)
    pub reviewer_id: String,
    /// ستاره (`1..=5`)
    pub stars: u8,
    /// نظر
    pub comment: String,
    /// خرید تاییدشده (اتصال به امانت تسویه‌شده؛ ضد-سوریل)
    pub verified_purchase: bool,
    /// عکس فوری نمرهٔ REA مایرون هنگام رأی
    pub mairon_score_at_review: u8,
    /// زمان ثبت ISO
    pub created_at: String,
}

/// خلاصهٔ رتبهٔ ۵-ستارهٔ بیزی + هیستوگرام + نمرهٔ رتبهٔ ترکیبی
/// (`GET /market/products/{id}/reviews`) — دو لایهٔ فنی و اجتماعی کنار هم.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProductRatingSummaryResponse {
    /// شناسهٔ محصول
    pub product_id: String,
    /// نمرهٔ فنی مایرون (`0..=100`)
    pub mairon_rea_score: u8,
    /// نشان مرئی مایرون
    pub mairon_visible_badge: String,
    /// شمار همهٔ رأی‌ها
    pub total_reviews: u32,
    /// شمار رأی‌های خرید تاییدشده
    pub verified_reviews: u32,
    /// میانگین حسابی ستاره‌ها (`0.0..=5.0`)
    pub average_stars: f64,
    /// میانگین بیزی (پیشین `C = 5.0` و `m = 4.20`)
    pub bayesian_stars: f64,
    /// هیستوگرام دقیق `[1★..5★]`
    pub star_histogram: Vec<u32>,
    /// نمایش پنج‌ستارهٔ دوزبانه
    pub five_star_display: String,
    /// نمرهٔ رتبهٔ ترکیبی: `0.60 * (bayesian_stars * 20) + 0.40 * mairon_score`
    pub combined_rank_score: f64,
    /// همهٔ رأی‌های محصول — جدیدترین اول
    pub reviews: Vec<ProductReviewView>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{from_str, to_string};

    #[test]
    fn product_view_category_null_is_explicit() {
        let p = ProductView {
            id: "prod:1".into(),
            seller_id: "citizen:1".into(),
            title: "دستور پخت".into(),
            description: "راجع به چیزی".into(),
            price_din: Din::new(500).unwrap(),
            category: None,
            preview_html: "<h1>دمو</h1>".into(),
            affiliate_bps: 1_000,
            artifact_kind: "html".into(),
            active: true,
            created_at: "2026-01-01T00:00:00.000Z".into(),
            mairon_score: 92,
            mairon_badge: "⚒️ Mairon REA Certified: 92/100 (Forge-Grade)".into(),
        };
        let json = to_string(&p).unwrap();
        assert!(json.contains("\"category\":null"), "explicit null — {json}");
        assert!(json.contains("\"affiliateBps\":1000"), "affiliateBps — {json}");
        assert!(json.contains("\"artifactKind\":\"html\""), "artifactKind — {json}");
        assert!(json.contains("\"previewHtml\":\"<h1>دمو</h1>\""), "previewHtml — {json}");
    }

    #[test]
    fn purchase_request_rejects_bad_id() {
        assert!(from_str::<PurchaseRequest>(r#"{"productId":"bad id!"}"#).is_err());
        assert!(from_str::<PurchaseRequest>(r#"{"productId":"prod:1"}"#).is_ok());
        let with_ref: PurchaseRequest =
            from_str(r#"{"productId":"prod:1","referrer":"12345"}"#).unwrap();
        assert_eq!(with_ref.referrer.as_deref(), Some("12345"));
    }
}
