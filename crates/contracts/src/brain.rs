//! قراردادهای وایر M4 «مغز + پنل عملیات» — گیت‌وی AI، موتور هرمس، ناوگان
//! A2A/MCP و اندپوینت‌های ادمین.
//!
//! قواعد فاز ۳ سر جایش است: `camelCase`، `null` صریح برای nullable، کلیدِ غایب
//! برای optional، اعداد صحیح تا `2^53−1`. حساس‌ترین نکتهٔ امنیتی این ماژول:
//! **خروجی AI هرگز کلید/راز برنمی‌گرداند** — `redactions` فهرست آنچه redact شد
//! را گزارش می‌کند، نه خودِ راز را؛ و کارت A2A هیچ نشست یا کلیدی فاش نمی‌کند.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

use crate::common::{Din, Id};
use crate::market::ProductView;

// ── گیت‌وی AI ───────────────────────────────────────────────────────────────

/// یک پیام در گفت‌وگوی AI — نقش‌های استاندارد `system|user|assistant`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AiMessage {
    /// نقش پیام
    pub role: String,
    /// متن پیام — قبل از ارسال به هر اسلات، redaction می‌شود
    pub content: String,
}

/// درخواست گفت‌وگو با گیت‌وی AI (فال‌بک سه‌اسلات).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AiChatRequest {
    /// پیام‌های گفت‌وگو — حداقل ۱
    pub messages: Vec<AiMessage>,
    /// مهارت هرمس اختیاری — غایب یعنی مسیریابی خودکار
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_id: Option<Id>,
}

/// پاسخ گیت‌وی — اسلات برنده + مصرف دقیق.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AiChatResponse {
    /// متن پاسخ (خروجی سپر هم روی آن می‌گذرد)
    pub content: String,
    /// ارائه‌دهندهٔ برنده — «primary/secondary/tertiary»
    pub provider: String,
    /// مدل استفاده‌شده
    pub model: String,
    /// توکن ورودی
    pub prompt_tokens: i64,
    /// توکن خروجی
    pub completion_tokens: i64,
    /// مجموع توکن
    pub total_tokens: i64,
    /// دستهٔ یافته‌های redaction (مثل `private_key_hex`) — خودِ راز هرگز نمی‌آید
    pub redactions: Vec<String>,
    /// باقی‌ماندهٔ بودجهٔ روزانهٔ کاربر به توکن
    pub remaining_tokens: i64,
}

/// مصرف امروز کاربر در گیت‌وی — مبنای پاکت سخت بودجه.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AiUsageResponse {
    /// مجموع توکن امروز (UTC)
    pub total_tokens: i64,
    /// تعداد فراخوانی‌های امروز
    pub calls: i64,
    /// سقف روزانهٔ پیکربندی‌شده
    pub daily_cap: i64,
}

/// یک اسلات مدل در گیت‌وی — صادقانه: اسلات بدون env کانفیگ `enabled: false`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AiSlotView {
    /// شمارهٔ اسلات — ۱ تا ۳
    pub slot: u8,
    /// نام اسلات (`primary`/`secondary`/`tertiary`)
    pub name: String,
    /// مدل کانفیگ‌شده — بدون env رشتهٔ خالی
    pub model: String,
    /// فعال = URL و KEY هر دو تنظیم شده‌اند
    pub enabled: bool,
}

/// وضعیت گیت‌وی AI — چیپ صادقانهٔ health و داشبورد ادمین.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AiStatusResponse {
    /// گیت‌وی فعال؟ حداقل یک اسلات کانفیگ‌شده
    pub enabled: bool,
    /// سقف روزانهٔ توکن هر کاربر
    pub daily_cap: i64,
    /// سه اسلات به ترتیب فال‌بک
    pub slots: Vec<AiSlotView>,
}

// ── موتور هرمس ──────────────────────────────────────────────────────────────

/// نتیجهٔ سپر دفاعی روی ورودی/خروجی مهارت.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ShieldView {
    /// `ALLOW` / `REDACTED` / `BLOCKED`
    pub verdict: String,
    /// دلیل مسدودسازی یا redaction — برای ALLOW `null` صریح
    pub reason: Option<String>,
}

/// نمای عمومی یک مهارت هرمس — قالب پرامپت هرگز روی سیم عمومی نمی‌آید.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HermesSkillView {
    /// شناسهٔ مهارت (`wallet.balance` و…)
    pub id: String,
    /// عنوان فارسی
    pub title: String,
    /// دسته (`wallet`/`market`/`brain`/…)
    pub category: String,
    /// توضیح
    pub description: String,
    /// `kernel` (اجرای قطعی روی کرنل) یا `llm` (قالب + گیت‌وی)
    pub kind: String,
    /// برآورد توکن برای بودجه
    pub estimate_tokens: i64,
}

/// فهرست مهارت‌های هرمس.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HermesSkillListResponse {
    /// مهارت‌های فعال — جدیدترین مرتب‌سازی بر اساس شناسه
    pub items: Vec<HermesSkillView>,
}

/// درخواست اجرای مهارت — بدون `skillId` مسیریابی خودکار کلیدواژه‌ای.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HermesRunRequest {
    /// شناسهٔ مهارت — غایب یعنی خودکار
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_id: Option<Id>,
    /// ورودی کاربر
    pub input: String,
}

/// پاسخ اجرای مهارت — شامل حکم سپر و امتیاز هویتی پس از این ران.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HermesRunResponse {
    /// شناسهٔ ران (`hrun:{uuid}`)
    pub run_id: Id,
    /// مهارت اجراشده — در مسیریابی خودکار، مهارتِ انتخاب‌شده گزارش می‌شود
    pub skill_id: String,
    /// خروجی مهارت
    pub output: String,
    /// حکم سپر روی ورودی
    pub shield: ShieldView,
    /// دستهٔ یافته‌های redaction ورودی/خروجی
    pub redactions: Vec<String>,
    /// ارائه‌دهنده — برای مهارت‌های kernel `null` صریح
    pub provider: Option<String>,
    /// مدل — برای مهارت‌های kernel `null` صریح
    pub model: Option<String>,
    /// توکن مصرفی (مهارت kernel → ۰)
    pub tokens: i64,
    /// امتیاز هویتی ۰..=۱۰۰ پس از این ران (RL هویت صادقانه از شمارش ران‌ها)
    pub identity_score: i64,
}

/// یک ران گذشتهٔ من — خروجی کامل فقط در خودِ ران؛ فهرست پیش‌نمایش کوتاه است.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HermesRunView {
    /// شناسهٔ ران
    pub id: Id,
    /// مهارت
    pub skill_id: String,
    /// حکم سپر (`ALLOW`/`REDACTED`/`BLOCKED`)
    pub shield: String,
    /// توکن مصرفی
    pub tokens: i64,
    /// نتیجهٔ ممیزی (`hermes.ok|blocked|failed`)
    pub outcome: String,
    /// زمان ISO
    pub at: String,
}

/// فهرست ران‌های من + امتیاز هویتی فعلی.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HermesRunListResponse {
    /// ران‌ها — جدیدترین اول
    pub items: Vec<HermesRunView>,
    /// امتیاز هویتی ۰..=۱۰۰
    pub identity_score: i64,
}

// ── ناوگان A2A/MCP ──────────────────────────────────────────────────────────

/// یک ایجنت در ناوگان — وضعیت صادقانه (`active`/`registered`/`disabled`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentView {
    /// شناسه ایجنت (`agent:{role}`)
    pub id: String,
    /// نام
    pub name: String,
    /// نقش — یکی از ۷ نقش ناوگان
    pub role: String,
    /// وضعیت — هیچ ایجنتی دروغ نمی‌گوید
    pub status: String,
    /// توضیح نقش
    pub description: String,
}

/// ناوگان + ابزارهای MCP — نمای عمومی شفاف.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FleetResponse {
    /// ۷ ایجنت ناوگان
    pub agents: Vec<AgentView>,
    /// گیت‌وی A2A فعال؟ (ادمین کلید دارد)
    pub a2a_enabled: bool,
    /// ابزارهای MCP ثبت‌شده
    pub mcp_tools: Vec<McpToolView>,
}

/// ابزار MCP — allowlist بسته؛ ابزار ناشناخته هرگز invoke نمی‌شود.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct McpToolView {
    /// نام ابزار (`kernel.economy` و…)
    pub name: String,
    /// توضیح
    pub description: String,
    /// آرگومان‌های لازم به ترتیب
    pub args: Vec<String>,
    /// فقط ادمین؟ (readهای عمومی false)
    pub admin_only: bool,
}

/// فرستندهٔ کارت A2A.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentCardProvider {
    /// سازمان
    pub organization: String,
    /// نشانی
    pub url: String,
}

/// یک مهارت در کارت A2A — از فهرست مهارت‌های هرمس می‌آید.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentCardSkill {
    /// شناسه
    pub id: String,
    /// نام
    pub name: String,
    /// توضیح
    pub description: String,
    /// برچسب‌ها
    pub tags: Vec<String>,
}

/// کارت کشف A2A — `/.well-known/agent-card.json`؛ هیچ کلیدی در آن نیست.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentCardResponse {
    /// نام ایجنت
    pub name: String,
    /// توضیح
    pub description: String,
    /// URL سرویس
    pub url: String,
    /// نسخهٔ کیان
    pub version: String,
    /// نسخهٔ پروتکل
    pub protocol_version: String,
    /// سازمان
    pub provider: AgentCardProvider,
    /// مهارت‌ها (از هرمس)
    pub skills: Vec<AgentCardSkill>,
}

/// ارسال پیام A2A بین ایجنت‌های ناوگان — فقط ادمین (گیت‌وی A2A).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct A2aSendRequest {
    /// فرستنده — باید ایجنت ثبت‌شده باشد
    pub from: Id,
    /// گیرنده — باید ایجنت ثبت‌شده باشد
    pub to: Id,
    /// نوع پیام (`handoff`/`query`/`result`)
    pub kind: String,
    /// بدنه
    pub body: String,
}

/// یک پیام A2A.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct A2aMessageView {
    /// شناسه پیام (`a2a:{uuid}`)
    pub id: Id,
    /// فرستنده
    pub from_agent: String,
    /// گیرنده
    pub to_agent: String,
    /// نوع
    pub kind: String,
    /// بدنه
    pub body: String,
    /// زمان ISO
    pub at: String,
}

/// صندوق ورودی یک ایجنت.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct A2aInboxResponse {
    /// پیام‌ها — جدیدترین اول
    pub items: Vec<A2aMessageView>,
}

/// فراخوانی ابزار MCP — فقط ابزارهای allowlist با آرگومان‌های دقیق.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct McpInvokeRequest {
    /// نام ابزار
    pub tool: Id,
    /// آرگومان‌ها — JSON object
    pub arguments: Value,
}

/// نتیجهٔ ابزار MCP — هر شکلی که ابزار برگرداند؛ خطا از پاکت استاندارد می‌آید.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct McpInvokeResponse {
    /// نام ابزار
    pub tool: String,
    /// نتیجه
    pub result: Value,
}

// ── پنل ادمین ───────────────────────────────────────────────────────────────

/// یک نقطهٔ سری درآمد ۱۴ روزه.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RevenuePoint {
    /// روز (YYYY-MM-DD)
    pub day: String,
    /// دیین درآمد همان روز
    pub din: Din,
}

/// فروش به ازای فروشنده.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SellerSalesView {
    /// حساب فروشنده
    pub seller_id: String,
    /// مجموع فروش به دیین
    pub din: Din,
}

/// فروش به ازای محصول.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProductSalesView {
    /// شناسهٔ محصول
    pub product_id: String,
    /// عنوان — اگر محصول حذف شده `null` صریح
    pub title: Option<String>,
    /// مجموع فروش به دیین
    pub din: Din,
}

/// نمای کلی ادمین — همهٔ شمارنده‌ها از کوئری‌های زندهٔ دیتابیس.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminOverviewResponse {
    /// کاربران تلگرام
    pub users: i64,
    /// حساب‌های دفتر
    pub accounts: i64,
    /// امانت‌ها
    pub escrows: i64,
    /// درخواست‌های HITL در انتظار
    pub hitl_pending: i64,
    /// کل ضرب‌شده (مادهٔ ۳)
    pub minted_all_time: Din,
    /// کل سوزانده‌شده
    pub burned_all_time: Din,
    /// کل فراخوانی‌های AI
    pub ai_calls: i64,
    /// سری درآمد ۱۴ روزه
    pub revenue_series: Vec<RevenuePoint>,
    /// فروش به ازای فروشنده
    pub sales_by_seller: Vec<SellerSalesView>,
    /// فروش به ازای محصول
    pub sales_by_product: Vec<ProductSalesView>,
}

/// یک درخواست HITL برای صف ادمین.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminHitlView {
    /// شناسه درخواست
    pub id: Id,
    /// نوع (`ESCROW_RELEASE`)
    pub kind: String,
    /// امانت مربوط
    pub escrow_id: String,
    /// مبلغ
    pub amount: Din,
    /// درخواست‌دهنده
    pub requester_id: String,
    /// یادداشت درخواست — غایب یعنی ثبت نشده
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// وضعیت (`PENDING`/`APPROVED`/`REJECTED`/`EXPIRED`)
    pub status: String,
    /// زمان ایجاد ISO
    pub created_at: String,
    /// انقضا ISO
    pub expires_at: String,
    /// تصمیم‌گیرنده — تا تصمیم غایب
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decided_by: Option<String>,
    /// زمان تصمیم — تا تصمیم غایب
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decided_at: Option<String>,
    /// یادداشت تصمیم — غایب یعنی ثبت نشده
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision_note: Option<String>,
}

/// فهرست صف HITL ادمین.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminHitlListResponse {
    /// درخواست‌ها — جدیدترین اول
    pub items: Vec<AdminHitlView>,
}

/// تصمیم ادمین روی HITL — تأیید یعنی تسویهٔ اتمی در همان فرمان.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HitlDecisionRequest {
    /// یادداشت تصمیم اختیاری
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// نتیجهٔ تصمیم HITL.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HitlDecisionResponse {
    /// شناسهٔ درخواست
    pub id: Id,
    /// وضعیت پس از تصمیم (`APPROVED`/`REJECTED`)
    pub status: String,
    /// زمان تصمیم ISO
    pub decided_at: String,
    /// آیا تسویهٔ پولی انجام شد؟ (فقط approve)
    pub settled: bool,
    /// شناسهٔ تراکنش تسویه — در replay هم همان tx اصلی
    pub tx_id: Option<String>,
    /// آیا تسویه یک replay بود؟
    pub replayed: bool,
}

/// یک ردیف ممیزی برای دنبالهٔ ادمین.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminAuditEntry {
    /// شناسه
    pub id: Id,
    /// عامل
    pub actor: String,
    /// اقدام
    pub action: String,
    /// هدف — غایب یعنی بدون هدف
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    /// متادیتا JSON رشته‌ای — غایب یعنی بدون متادیتا
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<String>,
    /// زمان ISO
    pub at: String,
}

/// دنبالهٔ ممیزی ادمین.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminAuditListResponse {
    /// ردیف‌ها — جدیدترین اول
    pub items: Vec<AdminAuditEntry>,
}

/// یک واریز TON در نمای ادمین — با کاربرش (برخلاف نمای کاربر M3).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminDepositView {
    /// شناسه واریز
    pub id: Id,
    /// کاربر ثبت‌کننده
    pub user_id: String,
    /// هش تراکنش زنجیره (UNIQUE — درس replay)
    pub tx_hash: String,
    /// آدرس فرستنده
    pub wallet_address: String,
    /// مبلغ TON رشتهٔ دهدهی
    pub amount_ton: String,
    /// وضعیت (`PENDING`/`CONFIRMED`/`REJECTED`)
    pub status: String,
    /// زمان ثبت ISO
    pub created_at: String,
    /// زمان تصمیم ISO — تا قبل از تکلیف غایب
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decided_at: Option<String>,
    /// تصمیم‌گیرنده — تا قبل از تکلیف غایب
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decided_by: Option<String>,
}

/// فهرست واریزهای TON برای ادمین (همهٔ کاربران، قابل فیلتر وضعیت).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminDepositListResponse {
    /// واریزها — جدیدترین اول
    pub items: Vec<AdminDepositView>,
}

/// فهرست محصولات برای ادمین — شامل غیرفعال‌ها (برخلاف نمای عمومی بازار).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminProductListResponse {
    /// همهٔ محصولات — جدیدترین اول
    pub items: Vec<ProductView>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{from_str, to_string};

    /// قرارداد وایر: پاسخ AI با null صریح و camelCase دور می‌زند.
    #[test]
    fn ai_chat_response_round_trips_camel_case() {
        let res = AiChatResponse {
            content: "سلام".into(),
            provider: "primary".into(),
            model: "kian-model".into(),
            prompt_tokens: 12,
            completion_tokens: 34,
            total_tokens: 46,
            redactions: vec!["private_key_hex".into()],
            remaining_tokens: 49_954,
        };
        let wire = to_string(&res).unwrap();
        assert!(wire.contains("\"promptTokens\""), "camelCase wire: {wire}");
        assert!(wire.contains("\"remainingTokens\""), "camelCase wire: {wire}");
        let back: AiChatResponse = from_str(&wire).unwrap();
        assert_eq!(back, res);
    }

    /// سپر دفاعی: verdict روی سیم `SCREAMING_SNAKE_CASE` است — مثل وضعیت اسکرو.
    #[test]
    fn shield_verdict_is_screaming_snake() {
        let blocked = ShieldView { verdict: "BLOCKED".into(), reason: Some("injection".into()) };
        let wire = to_string(&blocked).unwrap();
        assert!(wire.contains("\"verdict\":\"BLOCKED\""), "wire: {wire}");
        let allow: ShieldView =
            from_str(&to_string(&ShieldView { verdict: "ALLOW".into(), reason: None }).unwrap())
                .unwrap();
        assert_eq!(allow.reason, None);
    }

    /// کارت A2A — کشف عمومی؛ نسخهٔ پروتکل و مهارت‌ها حتماً حاضرند.
    #[test]
    fn agent_card_shape_is_a2a_like() {
        let card = AgentCardResponse {
            name: "KIAN Kernel".into(),
            description: "Money kernel + Hermes brain".into(),
            url: "http://127.0.0.1:8088/api/v1".into(),
            version: "0.1.0".into(),
            protocol_version: "a2a/1.0-kian".into(),
            provider: AgentCardProvider { organization: "KiaAgentX".into(), url: "https://kiaagentx.io".into() },
            skills: vec![AgentCardSkill {
                id: "wallet.balance".into(),
                name: "تراز من".into(),
                description: "خوانش قطعی دفتر".into(),
                tags: vec!["kernel".into()],
            }],
        };
        let wire = to_string(&card).unwrap();
        assert!(wire.contains("\"protocolVersion\""), "wire: {wire}");
        assert!(wire.contains("\"skills\""), "wire: {wire}");
        assert!(!wire.to_lowercase().contains("key"), "no credential marker in card: {wire}");
    }

    /// فراخوانی MCP — arguments آزاد (object) ولی tool تایپ Id دارد.
    #[test]
    fn mcp_invoke_accepts_json_arguments() {
        let req: McpInvokeRequest = from_str(
            r#"{"tool":"kernel.economy","arguments":{"deep":true}}"#,
        )
        .unwrap();
        assert_eq!(req.tool.get(), "kernel.economy");
        assert_eq!(req.arguments["deep"], serde_json::json!(true));
    }
}
