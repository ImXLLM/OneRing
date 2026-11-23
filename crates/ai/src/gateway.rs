//! gateway — فال‌بک سه‌اسلات + redaction + ثبت تلاش‌ها.
//!
//! قرارداد: ورودی اینجا redact می‌شود (قبل از دیدن هر اسلات)، اسلاتِ بدون
//! کانفیگ کامل (url+key+model) کلاً حذف است، پاکت خرابِ هر اسلات = شکستِ همان
//! اسلات و رفتن به بعدی. هیچ مسیری متن خام را به بیرون نمی‌فرستد.

use std::sync::Arc;

use crate::redaction::{redact, RedactionFinding};

/// نام‌های اسلات — ترتیبِ فال‌بک.
pub const SLOT_NAMES: [&str; 3] = ["primary", "secondary", "tertiary"];

/// سقف توکن خروجی برای تولید آرتیفکت HTML کامل در سازندهٔ محصول.
pub const BUILDER_MAX_TOKENS: u32 = 4_096;

/// پیکربندی یک اسلات مدل.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotConfig {
    /// نام اسلات — از [`SLOT_NAMES`]
    pub name: String,
    /// پایهٔ URL سازگار با `OpenAI` (`/chat/completions` append می‌شود)
    pub url: String,
    /// کلید API — هرگز لاگ یا روی سیم نمی‌رود
    pub key: String,
    /// نام مدل
    pub model: String,
}

impl SlotConfig {
    /// اسلات فقط با هر سهٔ url+key+model کامل فعال است — نیمه‌کانفیگ = خاموش.
    #[must_use]
    pub fn enabled(&self) -> bool {
        !self.url.is_empty() && !self.key.is_empty() && !self.model.is_empty()
    }
}

/// پیکربندی گیت‌وی.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiConfig {
    /// اسلات‌ها به ترتیب فال‌بک
    pub slots: Vec<SlotConfig>,
    /// سقف روزانهٔ توکن هر کاربر
    pub daily_cap_tokens: i64,
}

impl AiConfig {
    /// خواندن از محیط — fail-closed: بدون env فقط اسلات‌های خاموش.
    ///
    /// `KIAN_AI_PRIMARY_URL` / `KIAN_AI_PRIMARY_KEY` / `KIAN_AI_PRIMARY_MODEL`
    /// و بعد SECONDARY و TERTIARY؛ سقف روزانه `KIAN_AI_DAILY_TOKENS`
    /// (پیش‌فرض ۵۰٬۰۰۰).
    #[must_use]
    pub fn from_env() -> Self {
        let env = |k: &str| std::env::var(k).ok().filter(|v| !v.trim().is_empty());
        let slots = SLOT_NAMES
            .iter()
            .map(|name| {
                let upper = name.to_uppercase();
                SlotConfig {
                    name: (*name).to_owned(),
                    url: env(&format!("KIAN_AI_{upper}_URL")).unwrap_or_default(),
                    key: env(&format!("KIAN_AI_{upper}_KEY")).unwrap_or_default(),
                    model: env(&format!("KIAN_AI_{upper}_MODEL")).unwrap_or_default(),
                }
            })
            .collect();
        Self {
            slots,
            daily_cap_tokens: env("KIAN_AI_DAILY_TOKENS").and_then(|v| v.parse().ok()).unwrap_or(50_000),
        }
    }
}

/// یک پیام پاکت — نقش + محتوا (قبل از گیت‌وی redact نشده؛ اینجا انجام می‌شود).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketMessage {
    /// `system|user|assistant`
    pub role: String,
    /// متن
    pub content: String,
}

/// پاکت ارسالی به اسلات.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatPacket {
    /// مدل اسلات
    pub model: String,
    /// پیام‌ها — پیام‌های کاربر همیشه پس از redaction
    pub messages: Vec<PacketMessage>,
    /// سقف توکن خروجی اختیاری (مثلاً `Some(4096)` برای تولید HTML کامل)
    pub max_tokens: Option<u32>,
}

/// پاکت موفق اسلات.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionPacket {
    /// متن پاسخ
    pub content: String,
    /// توکن ورودی
    pub prompt_tokens: i64,
    /// توکن خروجی
    pub completion_tokens: i64,
}

/// شکست یک تلاش — برای پاسخ صادقانهٔ همه‌جا-شکست.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttemptFailure {
    /// نام اسلات
    pub slot: String,
    /// جزئیات کوتاه — هرگز شامل متن درخواست/کلید نیست
    pub detail: String,
}

/// خطای گیت‌وی — تایپ‌شده؛ متن خام آپ‌استریم هرگز بالا نمی‌آید.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GatewayError {
    /// هیچ اسلاتی کانفیگ نشده — 503 `AI_DISABLED`
    #[error("AI gateway disabled — no complete slot configured")]
    Disabled,
    /// همهٔ اسلات‌های فعال شکست خوردند — 502 `AI_UPSTREAM`
    #[error("all AI slots failed")]
    AllSlotsFailed {
        /// ریز شکست‌ها به ترتیب تلاش
        attempts: Vec<AttemptFailure>,
    },
}

/// وضعیت یک اسلات برای سیم — بدون کلید.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotStatus {
    /// شمارهٔ اسلات (۱ مبنا)
    pub slot: u8,
    /// نام
    pub name: String,
    /// مدل کانفیگ‌شده — خاموش یعنی رشتهٔ خالی
    pub model: String,
    /// فعال؟
    pub enabled: bool,
}

/// نتیجهٔ موفق گفت‌وگو.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatResult {
    /// متن پاسخ
    pub content: String,
    /// اسلات برنده
    pub provider: String,
    /// مدل برنده
    pub model: String,
    /// توکن ورودی
    pub prompt_tokens: i64,
    /// توکن خروجی
    pub completion_tokens: i64,
    /// مجموع
    pub total_tokens: i64,
    /// دستهٔ یافته‌های redaction ورودی (بدون تکرار، به ترتیب اولین مشاهده)
    pub redactions: Vec<String>,
}

/// نتیجهٔ یک کامل‌سازی — خطا رشتهٔ کوتاه بی‌راز (برای `AttemptFailure.detail`).
pub type CompletionOutcome = Result<CompletionPacket, String>;

/// کلاینت مدل — یک سطح بالاتر از HTTP تا تست‌ها اسکریپت‌شو باشند.
#[async_trait::async_trait]
pub trait ModelClient: Send + Sync {
    /// یک کامل‌سازی — خطا رشتهٔ کوتاه بی‌راز (برای `AttemptFailure.detail`).
    ///
    /// # Errors
    /// شکست شبکه/آپ‌استریم/پاکت خراب — به‌صورت رشتهٔ توضیحی.
    async fn complete(&self, slot: &SlotConfig, packet: &ChatPacket) -> CompletionOutcome;
}

/// گیت‌وی AI — redaction → فال‌بک ترتیبی → نتیجهٔ تایپ‌شده.
pub struct AiGateway {
    /// پیکربندی بوت
    pub config: AiConfig,
    client: Arc<dyn ModelClient>,
}

impl std::fmt::Debug for AiGateway {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // کلاینت عمداً فاش نمی‌شود — پیکربندی کافی است و کلیدها در trace هرگز
        f.debug_struct("AiGateway").field("config", &self.config).finish_non_exhaustive()
    }
}

impl AiGateway {
    /// ساخت — کلاینت تزریقی می‌شود (HTTP تولیدی یا اسکریپت‌شوی تست).
    #[must_use]
    pub fn new(config: AiConfig, client: Arc<dyn ModelClient>) -> Self {
        Self { config, client }
    }

    /// گیت‌وی فعال؟ حداقل یک اسلات کامل.
    #[must_use]
    pub fn enabled(&self) -> bool {
        self.config.slots.iter().any(SlotConfig::enabled)
    }

    /// وضعیت سه اسلات برای سیم — بدون هیچ کلیدی.
    #[must_use]
    pub fn slot_statuses(&self) -> Vec<SlotStatus> {
        self.config
            .slots
            .iter()
            .enumerate()
            .map(|(i, s)| SlotStatus {
                slot: u8::try_from(i + 1).unwrap_or(1),
                name: s.name.clone(),
                model: s.model.clone(),
                enabled: s.enabled(),
            })
            .collect()
    }

    /// گفت‌وگو — پیام‌های کاربر redact می‌شوند، اسلات‌ها به ترتیب امتحان می‌شوند.
    ///
    /// # Errors
    /// [`GatewayError::Disabled`] بدون اسلات فعال؛ [`GatewayError::AllSlotsFailed`]
    /// با ریز تلاش‌ها وقتی همهٔ اسلات‌های فعال شکست خوردند.
    pub async fn chat(&self, messages: Vec<PacketMessage>) -> Result<ChatResult, GatewayError> {
        self.chat_with_max_tokens(messages, None).await
    }

    /// گفت‌وگوی چندنوبتی با سقف توکن خروجی اختیاری (مثلاً `Some(4096)` برای تولید کد HTML).
    ///
    /// اسکنر رازها روی پیام‌های `user` اجرا می‌شود تا کلید/توکن کاربر هرگز به بیرون نرود؛
    /// پرامپت سیستمی و کد HTML خروجیِ نوبت‌های قبلی `assistant` دست‌نخورده می‌مانند.
    ///
    /// # Errors
    /// [`GatewayError::Disabled`] بدون اسلات فعال؛ [`GatewayError::AllSlotsFailed`]
    /// با ریز تلاش‌ها وقتی همهٔ اسلات‌های فعال شکست خوردند.
    pub async fn chat_with_max_tokens(
        &self,
        messages: Vec<PacketMessage>,
        max_tokens: Option<u32>,
    ) -> Result<ChatResult, GatewayError> {
        // ۱) redaction — روی پیام‌های کاربر قبل از دیدن هر اسلات؛ دسته‌ها بدون محتوا گزارش می‌شوند
        let mut findings: Vec<RedactionFinding> = Vec::new();
        let redacted: Vec<PacketMessage> = messages
            .into_iter()
            .map(|m| {
                if m.role == "user" {
                    let report = redact(&m.content);
                    findings.extend(report.findings);
                    PacketMessage { role: m.role, content: report.text }
                } else {
                    m
                }
            })
            .collect();

        let mut seen_kinds: Vec<String> = Vec::new();
        for f in &findings {
            let kind = f.kind.to_owned();
            if !seen_kinds.contains(&kind) {
                seen_kinds.push(kind);
            }
        }

        // ۲) فال‌بک ترتیبی
        let mut attempts: Vec<AttemptFailure> = Vec::new();
        let slots: Vec<&SlotConfig> =
            self.config.slots.iter().filter(|s| s.enabled()).collect();
        if slots.is_empty() {
            return Err(GatewayError::Disabled);
        }
        for slot in slots {
            let packet = ChatPacket {
                model: slot.model.clone(),
                messages: redacted.clone(),
                max_tokens,
            };
            match self.client.complete(slot, &packet).await {
                Ok(done) => {
                    // پاکت سخت‌گیرانه: محتوای خالی یا توکن منفی = پاکت خراب
                    if done.content.trim().is_empty()
                        || done.prompt_tokens < 0
                        || done.completion_tokens < 0
                    {
                        attempts.push(AttemptFailure {
                            slot: slot.name.clone(),
                            detail: "malformed packet — empty content or negative tokens".into(),
                        });
                        continue;
                    }
                    return Ok(ChatResult {
                        content: done.content,
                        provider: slot.name.clone(),
                        model: slot.model.clone(),
                        prompt_tokens: done.prompt_tokens,
                        completion_tokens: done.completion_tokens,
                        total_tokens: done.prompt_tokens.saturating_add(done.completion_tokens),
                        redactions: seen_kinds,
                    });
                }
                Err(detail) => {
                    tracing::warn!(slot = %slot.name, "AI slot failed: {detail}");
                    attempts.push(AttemptFailure { slot: slot.name.clone(), detail });
                }
            }
        }
        Err(GatewayError::AllSlotsFailed { attempts })
    }
}
