//! قراردادهای وایرِ Wave 10 (Commit #30) — اتاق جنگِ بنیان‌گذارِ حاکم و
//! موتورِ شبیه‌سازیِ واقعیِ زمانیِ «روزِ ۰ → روزِ ۱».
//!
//! DTOهای روی وایر برای چهار اندپوینتِ تازه:
//! - `GET  /api/v1/war-room/state` — وضعیتِ کاملِ روزِ صفر + سناریویِ
//!   کانونیِ ۲۴ تیک ([`WarRoomStateResponse`])؛
//! - `POST /api/v1/war-room/step` — پیشرویِ علّیِ تک‌گام
//!   ([`WarRoomStepRequest`], [`WarRoomStepResponse`])؛
//! - `POST /api/v1/war-room/directive` — دستورِ تاکتیکیِ رئیس + سه پاسخِ
//!   در-شخصیت ([`WarRoomDirectiveRequest`], [`WarRoomDirectiveResponse`])؛
//! - `GET  /api/v1/war-room/explain-simple` — توضیحِ «۰ تا ۱۰۰» برای
//!   سوادِ صفر ([`WarRoomExplainResponse`]).

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// نمایِ یک پرسونایِ هم‌بنیان‌گذار روی وایر.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FounderView {
    /// شناسهٔ وایر — `elon_musk` | `cz_binance` | `jeff_bezos` | `chairman`.
    pub persona_id: String,
    /// هندلِ X.
    pub handle: String,
    /// نامِ نمایشی — انگلیسی.
    pub display_name_en: String,
    /// نامِ نمایشی — فارسی.
    pub display_name_fa: String,
    /// مأموریت — انگلیسی.
    pub mandate_en: String,
    /// مأموریت — فارسی.
    pub mandate_fa: String,
    /// رنگِ accent به هگز.
    pub accent_hex: String,
    /// عضوِ شبکهٔ ۲۴/۷؟
    pub is_co_founder: bool,
}

/// نمایِ دفترِ ولتِ سازمان روی وایر — همهٔ ارقامِ صادقانهٔ روزِ صفر.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WalletLedgerView {
    /// بذرِ کریپتو (`µUSDC`) — `33_000_000`.
    pub crypto_seed_micros: i64,
    /// بذرِ نقد (سنت) — `1_500`.
    pub cash_seed_cents: i64,
    /// ذخیرهٔ کاغذیِ قفل‌شده (سنت) — `35_000`.
    pub paper_reserve_locked_cents: i64,
    /// درآمدِ خالصِ پلتفرم (`µUSDC`) — از صفر.
    pub earned_revenue_micros: i64,
    /// پرداخت‌هایِ ۸۰٪ سازنده (`µUSDC`).
    pub creator_payouts_micros: i64,
    /// پرداخت‌هایِ آفیلیت (`µUSDC`).
    pub affiliate_payouts_micros: i64,
    /// سطلِ مالیاتِ بازار (`µUSDC`).
    pub stream_marketplace_tax_micros: i64,
    /// سطلِ روتینگِ آفیلیت (`µUSDC`).
    pub stream_affiliate_routing_micros: i64,
    /// سطلِ بوستِ ویژه (`µUSDC`).
    pub stream_featured_boost_micros: i64,
    /// سطلِ اختلافِ تسویه (`µUSDC`).
    pub stream_settlement_spread_micros: i64,
    /// کاربرانِ فعال.
    pub active_users: i64,
    /// قیفِ بازدید.
    pub funnel_visits: i64,
    /// قیفِ ثبت‌نام.
    pub funnel_signups: i64,
    /// قیفِ خرید.
    pub funnel_purchases: i64,
    /// درآمد به رشتهٔ دلار — `$9.55`.
    pub earned_revenue_usd: String,
    /// بذرِ کریپتو به رشتهٔ دلار — `$33.00`.
    pub crypto_seed_usd: String,
    /// بذرِ نقد به رشتهٔ دلار — `$15.00`.
    pub cash_seed_usd: String,
    /// ذخیرهٔ کاغذی به رشتهٔ دلار — `$350.00`.
    pub paper_reserve_usd: String,
}

/// نمایِ یک تیکِ شبیه‌سازی روی وایر.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WarRoomTickView {
    /// شمارهٔ تیک.
    pub tick_index: i64,
    /// ثانیهٔ شبیه‌سازی.
    pub simulation_second: i64,
    /// شناسهٔ سخنران.
    pub speaker_id: String,
    /// نامِ نمایشیِ سخنران — انگلیسی.
    pub speaker_name_en: String,
    /// نامِ نمایشیِ سخنران — فارسی.
    pub speaker_name_fa: String,
    /// رنگِ accent سخنران.
    pub speaker_accent_hex: String,
    /// پیامِ فارسی.
    pub message_fa: String,
    /// پیامِ انگلیسی.
    pub message_en: String,
    /// فرمانِ ترمینال — در صورتِ اجرا.
    pub terminal_cmd: Option<String>,
    /// پستِ اجتماعی به JSON — در صورتِ انتشار.
    pub social_post_json: Option<String>,
    /// دلتایِ خزانه (`µUSDC`).
    pub wallet_delta_usdc_micros: i64,
    /// جمعِ ولت پس از تیک (`µUSDC`).
    pub wallet_total_usdc_micros: i64,
    /// مرحلهٔ فعال.
    pub active_pipeline_stage: String,
    /// جریانِ درآمدِ فعال.
    pub revenue_stream: String,
    /// حجمِ ناخالص (`µUSDC`).
    pub sale_volume_micros: i64,
    /// پرداختِ سازنده (`µUSDC`).
    pub creator_payout_micros: i64,
    /// فروشِ با رفرال؟
    pub referred: bool,
}

/// پاسخِ `GET /api/v1/war-room/state` — وضعیتِ کاملِ اتاقِ جنگ.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WarRoomStateResponse {
    /// امضایِ اتاقِ جنگ.
    pub signature: String,
    /// مخزنِ هدف.
    pub github_repo: String,
    /// هشِ کوتاهِ کمیتِ شاخص.
    pub github_commit_short: String,
    /// شناسهٔ زنجیرهٔ Base.
    pub base_chain_id: i64,
    /// قراردادِ USDC بومیِ Base.
    pub base_usdc_contract: String,
    /// نمادِ استارزِ تلگرام.
    pub telegram_stars_currency: String,
    /// ساعتِ شبیه‌سازی — ثانیهٔ سپری‌شده.
    pub elapsed_seconds: i64,
    /// نشانگرِ تیکِ بعدی.
    pub tick_cursor: i64,
    /// درصدِ پیشرفت.
    pub progress_percent: i64,
    /// پخشِ خودکار روشن؟
    pub playing: bool,
    /// ضریبِ سرعت — ۱ | ۲ | ۵.
    pub speed_multiplier: i64,
    /// دفترِ ولتِ سازمان.
    pub wallet: WalletLedgerView,
    /// پرسوناهایِ اتاقِ جنگ.
    pub founders: Vec<FounderView>,
    /// سناریویِ کانونیِ ۲۴ تیک — منبعِ واحدِ حقیقتِ اپِ HTML.
    pub scenario: Vec<WarRoomTickView>,
    /// ثابت‌هایِ منشور برایِ UI (bps).
    pub charter_unreferred_take_bps: i64,
    /// `8000 bps` سهمِ همیشگیِ سازنده.
    pub charter_creator_bps: i64,
    /// سقفِ ضدِ-جهش (ثانیه).
    pub max_tick_delta_seconds: i64,
    /// کلِ صادقانهٔ درآمدِ روزِ اول (`µUSDC`) — `9_550_000`.
    pub day1_total_earned_micros: i64,
}

/// درخواستِ `POST /api/v1/war-room/step`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WarRoomStepRequest {
    /// تعدادِ تیک‌هایِ اجراشده تا حالا — `0..=24`.
    pub tick_cursor: i64,
    /// ضریبِ سرعتِ انتخابی — فقط ۱، ۲ و ۵.
    pub speed_multiplier: i64,
}

/// پاسخِ `POST /api/v1/war-room/step` — تیکِ اجراشده + وضعیتِ تازه.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WarRoomStepResponse {
    /// تیکِ اجراشده — کامل.
    pub executed: WarRoomTickView,
    /// نشانگرِ تیک پس از اجرا.
    pub tick_cursor: i64,
    /// درصدِ پیشرفت.
    pub progress_percent: i64,
    /// ثانیهٔ شبیه‌سازی.
    pub elapsed_seconds: i64,
    /// دفترِ ولت پس از اجرا.
    pub wallet: WalletLedgerView,
    /// سناریو تمام شد؟
    pub scenario_exhausted: bool,
}

/// درخواستِ `POST /api/v1/war-room/directive`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WarRoomDirectiveRequest {
    /// متنِ دستورِ رئیس — فارسی یا انگلیسی، سقفِ ۲۰۰۰ نویسه.
    pub directive_text: String,
    /// لوکالِ پاسخ‌ها — `fa` | `en`.
    pub locale: String,
    /// تیکِ فعالِ زمانِ تزریق (برایِ ثبتِ علّی).
    pub triggered_tick: i64,
}

/// یک پاسخِ در-شخصیت روی وایر.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FounderReplyView {
    /// شناسهٔ پاسخ‌دهنده.
    pub speaker_id: String,
    /// نامِ انگلیسی.
    pub speaker_name_en: String,
    /// نامِ فارسی.
    pub speaker_name_fa: String,
    /// رنگِ accent.
    pub accent_hex: String,
    /// پاسخِ فارسی.
    pub message_fa: String,
    /// پاسخِ انگلیسی.
    pub message_en: String,
    /// فرمانِ ترمینالِ پیشنهادی.
    pub terminal_cmd: Option<String>,
    /// پستِ اجتماعیِ پیشنهادی به JSON.
    pub social_post_json: Option<String>,
}

/// پاسخِ `POST /api/v1/war-room/directive`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WarRoomDirectiveResponse {
    /// متنِ دستور — پس از پالایش.
    pub directive_text: String,
    /// قصدِ تشخیص‌داده‌شده.
    pub intent_id: String,
    /// برچسبِ فارسیِ قصد.
    pub intent_label_fa: String,
    /// برچسبِ انگلیسیِ قصد.
    pub intent_label_en: String,
    /// سه پاسخِ در-شخصیت: ایلان، CZ، جف.
    pub replies: Vec<FounderReplyView>,
    /// تیکِ فعالِ زمانِ تزریق.
    pub triggered_tick: i64,
    /// ثانیهٔ شبیه‌سازی.
    pub simulation_second: i64,
}

/// پاسخِ `GET /api/v1/war-room/explain-simple` — «۰ تا ۱۰۰» برای سوادِ صفر.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WarRoomExplainResponse {
    /// امضایِ اتاقِ جنگ.
    pub signature: String,
    /// لوکالِ بسته — `fa` | `en`.
    pub locale: String,
    /// سرخط — فارسی.
    pub headline_fa: String,
    /// سرخط — انگلیسی.
    pub headline_en: String,
    /// منشورِ پول — فارسی.
    pub charter_fa: String,
    /// منشورِ پول — انگلیسی.
    pub charter_en: String,
    /// چهار گام — شناسه‌ها به ترتیبِ الف تا د.
    pub step_ids: Vec<String>,
    /// عنوانِ فارسیِ هر گام.
    pub step_titles_fa: Vec<String>,
    /// عنوانِ انگلیسیِ هر گام.
    pub step_titles_en: Vec<String>,
    /// بدنهٔ فارسیِ هر گام.
    pub step_bodies_fa: Vec<String>,
    /// بدنهٔ انگلیسیِ هر گام.
    pub step_bodies_en: Vec<String>,
}
