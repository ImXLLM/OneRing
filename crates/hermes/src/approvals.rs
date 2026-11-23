//! `approvals` — حکمرانی HITL شش‌طبقه‌ای با صدای دوگانه + گاردهای ساب‌ایجنت و روتین.
//!
//! پورت Wave 2 از گنجینهٔ CopilotKit/OpenBot:
//! - `shared/approval-outcomes.ts` → [`ApprovalOutcome`]: هر تصمیم دو صدا دارد —
//!   جمله‌ای برای مدل (`outcome_for_model`) که صادق گزارش کند و دنبال دور زدن
//!   نرود، و جمله‌ای برای انسان (`outcome_for_person_fa`) که رک و کوتاه است.
//!   تغییر یکی هرگز دیگری را پشت سر نمی‌گذارد چون هر دو کنار هم تعریف و تست‌اند.
//! - `docs/approvals.md` + `shared/what-if.ts` → [`simulate_approval_policy`]:
//!   شبیه‌ساز سیاستِ قطعی «چه می‌شد اگر…» — همان ترتیب لایه‌ها که محصول اعمال
//!   می‌کند، بدون شبکه و بدون شانس.
//! - `shared/subagent.ts` → [`deterministic_subagent_thread_id`]: thread
//!   ساب‌ایجنت «مشتق» است نه «ضرب‌شده» — از والد + فراخوانی، به‌صورت UUIDv8، تا
//!   رونوشت ساب‌ایجنت بعد از ری‌استارت هم پیدا شود و هرگز تولد دوباره رخ ندهد.
//! - `docs/routines.md` → [`RoutineFatigueGuard`]: ضد اسپم روتین — کف ۱۵ دقیقه،
//!   سقف ۲۰ روتین فعال، مدارشکن پس از ۱۰ شکست پیاپی، و پرش از پنجره‌های از دست
//!   رفتهٔ کرون بعد از ری‌استارت (به‌جای شلیک انبوه).
//!
//! الگوی «جعبهٔ راز» (secret box) هم همین‌جا لایه‌بندی شده: مدیریت اعتبارنامه و
//! دسترسی هرگز خودکار تأیید نمی‌شود — کار خود انسان است.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use sha2::{Digest, Sha256};

/// آستانهٔ تأیید انسانی از منشور کیان (مادهٔ ۷) — ۵۰٬۰۰۰ دینار.
pub use kian_domain::charter::Charter;

/// شش طبقهٔ اقدامی که طبق `docs/approvals.md` «در هر نسخه می‌پرسند» — ترتیب
/// نمایش، همان ترتیب اولویت سیاست است.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ApprovalCategory {
    /// فرستادن پیام/میل/دعوت به دیگران.
    SendMessages,
    /// خرید، پرداخت یا جابه‌جایی پول.
    MakePurchasesOrTransfers,
    /// حذف یا تغییر دادهٔ موجود.
    DeleteOrModifyData,
    /// انتشار محتوای عمومی (پست، توییت، انتشار محصول).
    PublishPublicContent,
    /// مدیریت اعتبارنامه یا دسترسی (رمز، توکن، مجوز) — جعبهٔ راز.
    ManageCredentialsOrPermissions,
    /// ثبت فرم یا درخواست رسمی.
    SubmitFormsOrApplications,
}

impl ApprovalCategory {
    /// برچسب فارسی برای کارت تأیید و صفحهٔ سیاست.
    #[must_use]
    pub const fn label_fa(self) -> &'static str {
        match self {
            Self::SendMessages => "ارسال پیام به دیگران",
            Self::MakePurchasesOrTransfers => "خرید یا جابه‌جایی پول",
            Self::DeleteOrModifyData => "حذف یا تغییر داده",
            Self::PublishPublicContent => "انتشار محتوای عمومی",
            Self::ManageCredentialsOrPermissions => "مدیریت اعتبارنامه یا دسترسی",
            Self::SubmitFormsOrApplications => "ثبت فرم یا درخواست",
        }
    }
}

/// نتیجهٔ دوزبانهٔ یک کارت تأیید — صدای مدل و صدای انسان کنار هم.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ApprovalOutcome {
    /// انسان اجازه داد.
    Approved,
    /// انسان صریحاً رد کرد — مدل حق تلاش دوباره، پیرایه یا دور زدن ندارد.
    DeclinedByPerson,
    /// مهلت تصمیم تمام شد و کاری انجام نشد.
    TimedOut,
}

impl ApprovalOutcome {
    /// جمله‌ای که مدل می‌خواند — می‌گوید چه کسی تصمیم گرفت و بعد چه کند،
    /// تا گزارشش صادق باشد و دنبال راه دور نگردد (پورت OpenBot).
    #[must_use]
    pub const fn outcome_for_model(self) -> &'static str {
        match self {
            Self::Approved => {
                "APPROVED: The person allowed this action. Proceed, and report the result truthfully in your answer."
            }
            Self::DeclinedByPerson => {
                "DECLINED_BY_PERSON: The user explicitly declined this action. Do not retry, rephrase, or work around it."
            }
            Self::TimedOut => {
                "TIMED_OUT: The approval request expired without a decision, so it was not done. Ask again only if it is still needed, or say in your answer that it timed out."
            }
        }
    }

    /// جمله‌ای که انسان می‌خواند — رک، کوتاه، بدون کلمه‌نویسی برای مدل.
    #[must_use]
    pub const fn outcome_for_person_fa(self) -> &'static str {
        match self {
            Self::Approved => "انجام شد.",
            Self::DeclinedByPerson => "درخواست توسط شما رد شد.",
            Self::TimedOut => "زمان تأیید به پایان رسید و کاری انجام نشد.",
        }
    }
}

/// زمینهٔ یک اقدام فرضی برای شبیه‌ساز سیاست — همان چیپ‌های صفحهٔ what-if.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalContext {
    /// مبلغ مالی به دینار (None یعنی اقدام بی‌مبلغ).
    pub amount_din: Option<i64>,
    /// ابزار مخرب برچسب‌خورده توسط اپ — مستقل از نام ابزار می‌پرسد.
    pub destructive: bool,
    /// توضیح کوتاه اقدام برای کارت (مثلاً «انتقال ۲ میلیون دیناری به کیف X»).
    pub detail: String,
}

impl ApprovalContext {
    /// زمینهٔ سادهٔ بی‌مبلغ و غیرمخرب.
    #[must_use]
    pub fn plain(detail: impl Into<String>) -> Self {
        Self { amount_din: None, destructive: false, detail: detail.into() }
    }
}

/// حکم سیاست برای یک اقدام فرضی — سه‌گانهٔ قطعی.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PolicyVerdict {
    /// بدون کارت اجرا می‌شود.
    AutoApprove,
    /// کارت تأیید انسانی می‌خواهد (دو صدا).
    RequiresHumanConfirmation,
    /// سیاست به‌کلی رد می‌کند — حتی انسان هم از این مسیر پاس نمی‌دهد.
    Blocked,
}

impl PolicyVerdict {
    /// دلیل یک‌جمله‌ای فارسی برای کارت what-if و لاگ ممیزی.
    #[must_use]
    pub const fn reason_fa(self) -> &'static str {
        match self {
            Self::AutoApprove => "سیاست این اقدام را بدون کارت تأیید اجرا می‌کند.",
            Self::RequiresHumanConfirmation => "این اقدام کارت تأیید انسانی می‌خواهد.",
            Self::Blocked => "سیاست این اقدام را به‌کلی می‌بندد — کارِ خود انسان است.",
        }
    }
}

/// شبیه‌ساز سیاست «چه می‌شد اگر…» — تابعِ خالص و قطعی (پورت what-if-check).
///
/// ترتیب لایه‌ها همان ترتیبی است که محصول اعمال می‌کند؛ اولین جوابِ قطعی برنده است:
/// 1. **جعبهٔ راز:** `ManageCredentialsOrPermissions` همیشه [`PolicyVerdict::Blocked`]
///    است — رمز/توکن را فقط خود انسان وارد می‌کند و هیچ پاسخی از طرف او
///    نمی‌تواند بنشیند (الگوی «خودم انجامش می‌دهم» در OpenBot).
/// 2. **مخرب برچسب‌خورده:** `destructive` بدون توجه به نام ابزار کارت می‌خواهد.
/// 3. **دروازهٔ منشور کیان:** خرید/انتقال بالای آستانهٔ HITL منشور (۵۰٬۰۰۰ Đ)
///    تأیید انسانی می‌خواهد؛ زیر آستانه، لِین اسکروی خودکار آزاد می‌شود
///    ([`PolicyVerdict::AutoApprove`]) و مبلغِ منفی به‌کلی بسته است.
/// 4. بقیهٔ طبقه‌ها همیشه کارت می‌خواهند — ارسال، حذف، انتشار و ثبت فرم.
///
/// هیچ ورودی‌ای باعث I/O یا شانس نمی‌شود؛ همین تابع هم برای دکمهٔ «چه می‌شد
/// اگر…» و هم برای گیت واقعی اجرا می‌شود تا شبیه‌ساز و واقعیت هرگز اختلاف
/// نظر نداشته باشند.
#[must_use]
pub fn simulate_approval_policy(category: ApprovalCategory, context: &ApprovalContext) -> PolicyVerdict {
    // ۱. جعبهٔ راز — حتی انسان هم از این گیت پاس نمی‌شود.
    if matches!(category, ApprovalCategory::ManageCredentialsOrPermissions) {
        return PolicyVerdict::Blocked;
    }
    // ۲. ابزار مخربِ برچسب‌خورده — کارت می‌خواهد، هر اسمی که داشته باشد.
    if context.destructive {
        return PolicyVerdict::RequiresHumanConfirmation;
    }
    // ۳. دروازهٔ پول — آستانهٔ منشور کیان.
    if matches!(category, ApprovalCategory::MakePurchasesOrTransfers) {
        return match context.amount_din {
            None => PolicyVerdict::RequiresHumanConfirmation,
            // مبلغ نامعتبر (منفی یا بیرون از مرز امن 2^53−1) به‌کلی بسته است.
            Some(amount) => match kian_domain::Din::new(amount) {
                Ok(din) if kian_domain::requires_hitl(din) => PolicyVerdict::RequiresHumanConfirmation,
                Ok(_) => PolicyVerdict::AutoApprove,
                Err(_) => PolicyVerdict::Blocked,
            },
        };
    }
    // ۴. بقیهٔ طبقه‌ها همیشه کارت انسانی می‌خواهند.
    PolicyVerdict::RequiresHumanConfirmation
}

/// آیا رشتهٔ شکلِ استاندارد UUID است (۳۶ نویسه، خط‌تیره در ۸/۱۳/۱۸/۲۳)؟
fn looks_like_uuid(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 36 {
        return false;
    }
    for (index, b) in bytes.iter().enumerate() {
        match index {
            8 | 13 | 18 | 23 => {
                if *b != b'-' {
                    return false;
                }
            }
            _ => {
                if !b.is_ascii_hexdigit() {
                    return false;
                }
            }
        }
    }
    true
}

/// thread قطعی ساب‌ایجنت — مشتق، نه ضرب‌شده (پورت `subagentThreadId`).
///
/// هش SHA-256 روی `subagent\n{parent_thread_id}\n{tool_call_id}`؛ اگر والد
/// UUID است، ۶ بایت اول با بایت‌های والد جایگزین می‌شود تا ساب‌ایجنت همان
/// اثر انگشت استقرار گفتگوی خودش را حمل کند. نسخه روی ۸ قفل می‌شود (UUIDv8:
/// این بیت‌ها تصادفی نیستند) و واریانت RFC هم رعایت می‌شود.
///
/// هر دو سمت (سرور و کلاینت) همین derivation را می‌دانند، پس کلاینت می‌تواند
/// رونوشت ساب‌ایجنت را «قبل از رسیدن نتیجه» پیدا کند و بعد از ری‌استارت هم
/// همان thread بازیابی می‌شود — تولد دوباره و spawning تکراری ممکن نیست.
///
/// هیچ‌چیز اینجا راز نیست؛ دانستن شناسه به هیچ‌چیز نمی‌رسد.
#[must_use]
pub fn deterministic_subagent_thread_id(parent_thread_id: &str, tool_call_id: &str) -> String {
    let digest = Sha256::digest(
        format!("subagent\n{parent_thread_id}\n{tool_call_id}").as_bytes(),
    );
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    if looks_like_uuid(parent_thread_id) {
        let hex: String = parent_thread_id.chars().filter(|c| *c != '-').collect();
        for (index, byte) in bytes.iter_mut().take(6).enumerate() {
            *byte = u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16).unwrap_or(0);
        }
    }
    bytes[6] = (bytes[6] & 0x0f) | 0x80; // نسخه ۸
    bytes[8] = (bytes[8] & 0x3f) | 0x80; // واریانت RFC
    let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    format!(
        "{}-{}-{}-{}-{}",
        &hex[..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    )
}

/// حکم گارد خستگی روتین برای یک پنجرهٔ زمان‌بندی.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FireDecision {
    /// شلیک مجاز است.
    Fire,
    /// کف ۱۵ دقیقه هنوز تمام نشده — به نوبت بعد موکول شود.
    Throttled,
    /// سقف روتین‌های فعال پر است.
    CapReached,
    /// مدارشکن باز است — روتین پس از شکست‌های پیاپی خودکار خاموش شده.
    DisabledAfterFailures,
    /// پنجرهٔ کرون بعد از ری‌استارت از دست رفته — پرش، نه شلیک انبوه.
    SkippedMissedWindow,
}

impl FireDecision {
    /// توضیح فارسی برای لاگ و صفحهٔ روتین‌ها.
    #[must_use]
    pub const fn reason_fa(self) -> &'static str {
        match self {
            Self::Fire => "روتین در پنجرهٔ خودش شلیک می‌کند.",
            Self::Throttled => "فاصلهٔ حداقلی ۱۵ دقیقه هنوز تمام نشده است.",
            Self::CapReached => "سقف ۲۰ روتین فعال پر است؛ تا خالی شدن صبر کن.",
            Self::DisabledAfterFailures => "روتین پس از ۱۰ شکست پیاپی خودکار خاموش شد.",
            Self::SkippedMissedWindow => "پنجرهٔ از دست رفته پس از ری‌استارت پرش می‌شود، نه شلیک انبوه.",
        }
    }
}

/// گارد ضد اسپم روتین — پورت سیاست روتین‌های OpenBot با ثابت‌های مأموریت:
/// - [`Self::MIN_INTERVAL_SECS`] = ۹۰۰ (کف ۱۵ دقیقه بین دو شلیک یک روتین)؛
/// - [`Self::MAX_ACTIVE_ROUTINES`] = ۲۰ (سقف روتین‌های فعال)؛
/// - [`Self::MAX_CONSECUTIVE_FAILURES`] = ۱۰ (مدارشکن: خاموشی خودکار).
///
/// گارد کاملاً قطعی است: زمان را مصرف‌کننده می‌فرستد (تست‌پذیری بدون خواب)،
/// و پرش پنجرهٔ از دست رفته بعد از ری‌استارت به‌جای شلیک انبوهِ همهٔ
/// پنجره‌های عقب‌افتاده انجام می‌شود.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RoutineFatigueGuard {
    /// وضعیت هر روتین به شناسه‌اش.
    state: BTreeMap<String, RoutineState>,
}

/// وضعیت داخلی یک روتین.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct RoutineState {
    /// آخرین شلیک مجاز (ثانیهٔ epoch).
    last_fired_secs: Option<u64>,
    /// شمارش شکست پیاپی بدون موفقیت.
    consecutive_failures: u32,
}

impl RoutineFatigueGuard {
    /// کف فاصلهٔ دو شلیک یک روتین: ۱۵ دقیقه.
    pub const MIN_INTERVAL_SECS: u64 = 900;
    /// سقف روتین‌های فعال هم‌زمان.
    pub const MAX_ACTIVE_ROUTINES: usize = 20;
    /// آستانهٔ مدارشکن: خاموشی خودکار پس از این تعداد شکست پیاپی.
    pub const MAX_CONSECUTIVE_FAILURES: u32 = 10;

    /// گارد تازه و خالی.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// حکم این پنجرهٔ زمان‌بندی برای روتین داده‌شده.
    ///
    /// ترتیب لایه‌ها (اولین جواب قطعی برنده است):
    /// 1. مدارشکن باز → [`FireDecision::DisabledAfterFailures`]؛
    /// 2. پنجرهٔ از دست رفته (زمان‌بندیِ گذشته از الان، بعد از ری‌استارت) →
    ///    [`FireDecision::SkippedMissedWindow`] — پرش، نه شلیک انبوه؛
    /// 3. سقف روتین‌های فعال → [`FireDecision::CapReached`]؛
    /// 4. کف ۱۵ دقیقه → [`FireDecision::Throttled`]؛
    /// 5. وگرنه [`FireDecision::Fire`] و ثبت زمان شلیک.
    pub fn evaluate(
        &mut self,
        routine_id: &str,
        now_secs: u64,
        scheduled_for_secs: u64,
        active_count: usize,
    ) -> FireDecision {
        let state = self.state.entry(routine_id.to_owned()).or_default();
        if state.consecutive_failures >= Self::MAX_CONSECUTIVE_FAILURES {
            return FireDecision::DisabledAfterFailures;
        }
        if scheduled_for_secs < now_secs {
            // بعد از ری‌استارت، همهٔ پنجره‌های عقب‌افتاده یک‌جا شلیک نمی‌شوند؛
            // روتین منتظر پنجرهٔ بعدی زمان‌بندی‌شده می‌ماند.
            return FireDecision::SkippedMissedWindow;
        }
        if active_count >= Self::MAX_ACTIVE_ROUTINES {
            return FireDecision::CapReached;
        }
        if let Some(last) = state.last_fired_secs
            && now_secs.saturating_sub(last) < Self::MIN_INTERVAL_SECS
        {
            return FireDecision::Throttled;
        }
        state.last_fired_secs = Some(now_secs);
        FireDecision::Fire
    }

    /// ثبت موفقیت — شمارش شکست پیاپی صفر می‌شود.
    pub fn record_success(&mut self, routine_id: &str) {
        let state = self.state.entry(routine_id.to_owned()).or_default();
        state.consecutive_failures = 0;
    }

    /// ثبت شکست — با رسیدن به آستانه، مدارشکن باز می‌شود (خاموشی خودکار).
    pub fn record_failure(&mut self, routine_id: &str) {
        let state = self.state.entry(routine_id.to_owned()).or_default();
        state.consecutive_failures = state.consecutive_failures.saturating_add(1);
    }

    /// آیا مدارشکن این روتین باز است؟
    #[must_use]
    pub fn is_disabled(&self, routine_id: &str) -> bool {
        self.state
            .get(routine_id)
            .is_some_and(|s| s.consecutive_failures >= Self::MAX_CONSECUTIVE_FAILURES)
    }

    /// شمارش شکست پیاپی فعلی (برای صفحهٔ روتین‌ها).
    #[must_use]
    pub fn consecutive_failures(&self, routine_id: &str) -> u32 {
        self.state
            .get(routine_id)
            .map_or(0, |s| s.consecutive_failures)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// صدای مدل و صدای انسان کنار هم تعریف شده‌اند — ردِ انسان برای مدل صریح
    /// «دور نزن» است و برای انسان یک جملهٔ کوتاه فارسی.
    #[test]
    fn dual_voice_outcomes_never_leak_model_voice_to_person() {
        let declined = ApprovalOutcome::DeclinedByPerson;
        assert_eq!(
            declined.outcome_for_model(),
            "DECLINED_BY_PERSON: The user explicitly declined this action. Do not retry, rephrase, or work around it."
        );
        assert_eq!(declined.outcome_for_person_fa(), "درخواست توسط شما رد شد.");
        assert!(declined.outcome_for_model().starts_with("DECLINED_BY_PERSON"));
        assert!(!declined.outcome_for_person_fa().contains("DECLINED"));

        assert!(ApprovalOutcome::Approved.outcome_for_model().starts_with("APPROVED"));
        assert!(ApprovalOutcome::TimedOut.outcome_for_model().starts_with("TIMED_OUT"));
        assert_eq!(ApprovalOutcome::TimedOut.outcome_for_person_fa(), "زمان تأیید به پایان رسید و کاری انجام نشد.");
    }

    /// ترتیب لایه‌های شبیه‌ساز: جعبهٔ راز می‌بندد، مخرب کارت می‌خواهد، پول از
    /// دروازهٔ منشور می‌گذرد و بقیه همیشه کارت انسانی می‌گیرند.
    #[test]
    fn what_if_policy_layers_match_the_product_order() {
        // جعبهٔ راز — حتی مبلغ صفر و غیرمخرب هم بسته است.
        assert_eq!(
            simulate_approval_policy(ApprovalCategory::ManageCredentialsOrPermissions, &ApprovalContext::plain("تغییر رمز کیف")),
            PolicyVerdict::Blocked
        );
        // مخرب برچسب‌خورده — هر طبقه‌ای که باشد کارت می‌خواهد.
        let destructive = ApprovalContext { amount_din: Some(100), destructive: true, detail: "پاک‌سازی آرشیو".into() };
        assert_eq!(
            simulate_approval_policy(ApprovalCategory::DeleteOrModifyData, &destructive),
            PolicyVerdict::RequiresHumanConfirmation
        );
        // پول: زیر آستانهٔ منشور خودکار، روی آستانه انسانی، منفی بسته.
        let buy_small = ApprovalContext { amount_din: Some(Charter::HITL_THRESHOLD_DIN.as_i64() - 1), destructive: false, detail: "خرید محصول".into() };
        assert_eq!(
            simulate_approval_policy(ApprovalCategory::MakePurchasesOrTransfers, &buy_small),
            PolicyVerdict::AutoApprove
        );
        let buy_big = ApprovalContext { amount_din: Some(Charter::HITL_THRESHOLD_DIN.as_i64()), destructive: false, detail: "انتقال بزرگ".into() };
        assert_eq!(
            simulate_approval_policy(ApprovalCategory::MakePurchasesOrTransfers, &buy_big),
            PolicyVerdict::RequiresHumanConfirmation
        );
        let buy_negative = ApprovalContext { amount_din: Some(-1), destructive: false, detail: "مبلغ نامعتبر".into() };
        assert_eq!(
            simulate_approval_policy(ApprovalCategory::MakePurchasesOrTransfers, &buy_negative),
            PolicyVerdict::Blocked
        );
        let buy_unknown = ApprovalContext { amount_din: None, destructive: false, detail: "مبلغ نامعلوم".into() };
        assert_eq!(
            simulate_approval_policy(ApprovalCategory::MakePurchasesOrTransfers, &buy_unknown),
            PolicyVerdict::RequiresHumanConfirmation
        );
        // بقیهٔ طبقه‌ها همیشه کارت انسانی می‌خواهند.
        for category in [
            ApprovalCategory::SendMessages,
            ApprovalCategory::DeleteOrModifyData,
            ApprovalCategory::PublishPublicContent,
            ApprovalCategory::SubmitFormsOrApplications,
        ] {
            assert_eq!(
                simulate_approval_policy(category, &ApprovalContext::plain("اقدام فرضی")),
                PolicyVerdict::RequiresHumanConfirmation,
                "{category:?}"
            );
        }
    }

    /// UUIDv8 ساب‌ایجنت: قطعی، شکل درست، اثر انگشت والد، و حساس به فراخوانی.
    #[test]
    fn subagent_thread_id_is_deterministic_uuidv8_with_parent_fingerprint() {
        let parent = "11111111-2222-3333-4444-555555555555";
        let a = deterministic_subagent_thread_id(parent, "call-1");
        let b = deterministic_subagent_thread_id(parent, "call-1");
        assert_eq!(a, b, "same parent + call → same thread, no duplicate spawn");

        // شکل UUIDv8: نسخه روی ۸ و واریانت RFC.
        assert_eq!(a.len(), 36);
        assert_eq!(a.chars().nth(14), Some('8'), "version nibble must be 8: {a}");
        assert!(
            matches!(a.chars().nth(19), Some('8' | '9' | 'a' | 'b')),
            "variant nibble must be RFC: {a}"
        );
        let c = deterministic_subagent_thread_id(parent, "call-2");
        assert_ne!(a, c, "different tool call → different thread");

        // ۶ بایت اول از والد به ارث می‌رسد (اثر انگشت استقرار)؛ نیم‌نویسهٔ
        // نسخهٔ ۸ روی گروه سوم قفل می‌شود (۸ در نیم‌نویسهٔ بالا).
        assert!(a.starts_with("11111111-2222-8"), "parent fingerprint + version 8: {a}");

        // والدهای غیر UUID هم کار می‌کنند و قطعی می‌مانند.
        let d1 = deterministic_subagent_thread_id("chat:fa:900", "c1");
        let d2 = deterministic_subagent_thread_id("chat:fa:900", "c1");
        assert_eq!(d1, d2);
        assert_eq!(d1.chars().nth(14), Some('8'));
    }

    /// کف ۱۵ دقیقه: ۸۹۹ ثانیه بعد خفه می‌شود، ۹۰۰ ثانیه بعد شلیک می‌کند.
    #[test]
    fn routine_fatigue_enforces_fifteen_minute_floor() {
        let mut guard = RoutineFatigueGuard::new();
        assert_eq!(guard.evaluate("r1", 1_000, 1_000, 0), FireDecision::Fire);
        assert_eq!(guard.evaluate("r1", 1_000 + 899, 1_899, 0), FireDecision::Throttled);
        let fired = guard.evaluate("r1", 1_000 + 900, 1_900, 0);
        assert_eq!(fired, FireDecision::Fire);
        assert_eq!(fired.reason_fa(), "روتین در پنجرهٔ خودش شلیک می‌کند.");
        // همان لحظه دوباره؟ خفه — کف ۱۵ دقیقه از شلیکِ تازه می‌شمارد.
        assert_eq!(guard.evaluate("r1", 1_900, 1_900, 0), FireDecision::Throttled);
        // روتین دوم مستقل است.
        assert_eq!(guard.evaluate("r2", 1_001, 1_001, 0), FireDecision::Fire);
    }

    /// سقف ۲۰ روتین فعال و پرش پنجرهٔ از دست رفته بعد از ری‌استارت.
    #[test]
    fn routine_cap_and_missed_window_skip_work() {
        let mut guard = RoutineFatigueGuard::new();
        assert_eq!(guard.evaluate("fresh", 5_000, 5_000, RoutineFatigueGuard::MAX_ACTIVE_ROUTINES), FireDecision::CapReached);
        assert_eq!(
            guard.evaluate("fresh", 5_000, 5_000, RoutineFatigueGuard::MAX_ACTIVE_ROUTINES - 1),
            FireDecision::Fire,
            "19 active leaves one seat"
        );
        // پنجرهٔ عقب‌افتاده بعد از ری‌استارت: پرش، حتی اگر کف فاصله هم گذشته باشد.
        assert_eq!(guard.evaluate("fresh", 100_000, 90_000, 0), FireDecision::SkippedMissedWindow);
    }

    /// مدارشکن: پس از ۱۰ شکست پیاپی خاموش می‌شود؛ یک موفقیت شمارش را صفر می‌کند.
    #[test]
    fn circuit_breaker_auto_disables_after_ten_failures() {
        let mut guard = RoutineFatigueGuard::new();
        for round in 0..RoutineFatigueGuard::MAX_CONSECUTIVE_FAILURES - 1 {
            guard.record_failure("r1");
            assert_eq!(guard.consecutive_failures("r1"), round + 1);
            assert!(!guard.is_disabled("r1"));
        }
        guard.record_failure("r1");
        assert!(guard.is_disabled("r1"), "10th failure opens the breaker");
        assert_eq!(guard.evaluate("r1", 50_000, 50_000, 0), FireDecision::DisabledAfterFailures);
        // موفقیت مدار را می‌بندد و روتین دوباره شلیک می‌کند.
        guard.record_success("r1");
        assert!(!guard.is_disabled("r1"));
        assert_eq!(guard.evaluate("r1", 50_000, 50_000, 0), FireDecision::Fire);
    }

    /// شش طبقه و سه حکم — سری‌شدن قرارداد SCREAMING_SNAKE را رعایت می‌کند.
    #[test]
    fn categories_and_verdicts_serialize_stable() {
        let json = serde_json::to_string(&ApprovalCategory::MakePurchasesOrTransfers).expect("serialize");
        assert_eq!(json, "\"MAKE_PURCHASES_OR_TRANSFERS\"");
        let json = serde_json::to_string(&PolicyVerdict::RequiresHumanConfirmation).expect("serialize");
        assert_eq!(json, "\"REQUIRES_HUMAN_CONFIRMATION\"");
        let json = serde_json::to_string(&ApprovalOutcome::DeclinedByPerson).expect("serialize");
        assert_eq!(json, "\"DECLINED_BY_PERSON\"");
        assert_eq!(ApprovalCategory::ManageCredentialsOrPermissions.label_fa(), "مدیریت اعتبارنامه یا دسترسی");
    }
}
