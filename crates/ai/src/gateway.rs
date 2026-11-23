//! gateway — استخرِ چندلِین + فال‌بک + گیت کیفیت + redaction + ثبت تلاش‌ها.
//!
//! معماری پورت‌شده از `brain.py` نسخهٔ ۱٫۲ هرمس (ShopifyHermes) — battle-tested:
//! * **استخر تخت**: هر ورودی یک ترکیب `(url, key, model)` است؛ اسلات‌های منطقی و
//!   تامین‌کننده‌های سبک هرمس با ضرب دکارتیِ «هر مدل × هر کلید» باز می‌شوند و
//!   سه‌تایی‌های تکراری حذف می‌شوند (حفظ ترتیب).
//! * **چسبندگی `last_good`**: برندهٔ آخر دفعهٔ بعد اول امتحان می‌شود
//!   (`AtomicUsize` + `Relaxed`) — طرح دقیق `brain.py`: `[last_good] + بقیه`.
//! * **گیت کیفیت `looks_broken`**: پاسخ خالی/خراب/بی‌معنی/اشتباه‌زبان = شکست
//!   همان لِین؛ اولین پاسخِ «خراب اما موجود» به‌عنوان کم‌ضررترین ذخیره می‌شود و
//!   فقط وقتی همه‌جا شکست، برگردانده می‌شود (چت) یا به سنتز محلی می‌رسد (سازنده).
//!
//! قرارداد: ورودی اینجا redact می‌شود (قبل از دیدن هر لِین)، لِینِ بدون کانفیگ
//! کامل (url+key+model) کلاً حذف است، پاکت خراب هر لِین = شکست همان لِین و رفتن
//! به بعدی. هیچ مسیری متن خام یا کلید را به بیرون نمی‌فرستد.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::redaction::{redact, RedactionFinding};

/// نام‌های اسلات منطقی — ترتیب فال‌بک.
pub const SLOT_NAMES: [&str; 3] = ["primary", "secondary", "tertiary"];

/// سقف توکن خروجی برای تولید آرتیفکت HTML کامل در سازندهٔ محصول.
///
/// شواهد بنچمارک زندهٔ OpenRouter (۲۴ نوامبر ۲۰۲۶): مدل‌های reasoning-محورِ
/// لِین رایگان، سهم `reasoning_tokens` را از همین سقف می‌خورند؛ در ۴۰۹۶ یا
/// محتوا خالی می‌ماند (`finish_reason=length` روی فکر کردن) یا سند وسط راه
/// بریده می‌شود. ۱۶۳۸۴ سند کامل + بلوک CAST JSON را با فاصلهٔ امن جا می‌دهد
/// و روی لِین‌های رایگان هزینهٔ صفر دارد.
pub const BUILDER_MAX_TOKENS: u32 = 16_384;

/// پایهٔ `OpenRouter` — پیش‌فرض تامین‌کنندهٔ سبک `KIAN_AI_OPENROUTER_*` (بدون URL).
pub const OPENROUTER_BASE_URL: &str = "https://openrouter.ai/api/v1";

/// پایهٔ `Coarena` — اعتبار رایگان روزانه، سازگار با `OpenAI` (کشف‌شده در هرمس).
pub const COARENA_BASE_URL: &str = "https://coarena.ai/api/v1";

/// پایهٔ پیش‌فرض ۹روتر — پروکسی رایجِ لِین‌های `oc/*`.
pub const ROUTER9_BASE_URL: &str = "https://9router-production-6f2225a3.up.railway.app/v1";

/// پیکربندی یک لِین استخر.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotConfig {
    /// نام لِین — نام اسلات منطقی (`SLOT_NAMES`) یا تامین‌کنندهٔ سبک هرمس
    /// (`coarena`/`openrouter`/`router9`) — روی سیم به‌عنوان provider گزارش می‌شود
    pub name: String,
    /// پایهٔ URL سازگار با `OpenAI` (`/chat/completions` append می‌شود)
    pub url: String,
    /// کلید API — هرگز لاگ یا روی سیم نمی‌رود
    pub key: String,
    /// نام مدل
    pub model: String,
}

impl SlotConfig {
    /// لِین فقط با هر سهٔ url+key+model کامل فعال است — نیمه‌کانفیگ = خاموش.
    #[must_use]
    pub fn enabled(&self) -> bool {
        !self.url.is_empty() && !self.key.is_empty() && !self.model.is_empty()
    }
}

/// ضرب دکارتیِ `brain.py`: «برای هر مدل، برای هر کلید» — یک ورودی استخر برای
/// هر ترکیب. ورودی‌های خالی (فاصله/جداشنندهٔ سرِ خالی) نادیده گرفته می‌شوند.
///
/// ترتیب دقیق: مدلِ بیرونی، کلیدِ درونی — یعنی `mA×(k1,k2), mB×(k1,k2)`.
#[must_use]
pub fn expand_models_times_keys(name: &str, url: &str, keys: &str, models: &str) -> Vec<SlotConfig> {
    let mut out = Vec::new();
    for model in models.split(',').map(str::trim).filter(|m| !m.is_empty()) {
        for key in keys.split(',').map(str::trim).filter(|k| !k.is_empty()) {
            out.push(SlotConfig {
                name: name.to_owned(),
                url: url.to_owned(),
                key: key.to_owned(),
                model: model.to_owned(),
            });
        }
    }
    out
}

/// افزودن به استخر با حذف سه‌تایی تکراریِ `(url, key, model)` — حفظ اولین ترتیب.
fn push_unique(pool: &mut Vec<SlotConfig>, entry: SlotConfig) {
    if !pool
        .iter()
        .any(|x| x.url == entry.url && x.key == entry.key && x.model == entry.model)
    {
        pool.push(entry);
    }
}

/// پیکربندی گیت‌وی.
///
/// `slots` **استخرِ تختِ اجرایی** است (همان `pool` در `brain.py`) — پس از
/// بازکردن ضرب دکارتی، هر عضو یک ترکیب `(url, key, model)` است. [`AiConfig::pool`]
/// همین برش را با نام روشن می‌دهد؛ `/ai/status` عضوها را بر اساس نام گروه می‌کند
/// و اندازهٔ استخر را جداگانه گزارش می‌کند.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiConfig {
    /// استخر تخت به ترتیب فال‌بک (اسلات‌های منطقی ← تامین‌کننده‌های سبک هرمس)
    pub slots: Vec<SlotConfig>,
    /// سقف روزانهٔ توکن هر کاربر
    pub daily_cap_tokens: i64,
}

impl AiConfig {
    /// برش استخر تخت — نامی روشن‌تر برای همان [`AiConfig::slots`].
    #[must_use]
    pub fn pool(&self) -> &[SlotConfig] {
        &self.slots
    }

    /// خواندن از محیط — fail-closed: بدون env استخر خالی.
    ///
    /// اسلات‌های منطقی از `KIAN_AI_{PRIMARY|SECONDARY|TERTIARY}_{URL,KEY,MODEL}`
    /// (یا میان‌برهای تک‌گیت‌وی) ساخته می‌شوند؛ `_KEY` و `_MODEL` (و میان‌برها)
    /// می‌توانند **فهرست جداشده با کاما** باشند و با ضرب دکارتیِ `brain.py` باز
    /// می‌شوند. تامین‌کننده‌های سبک هرمس اگر حاضر باشند به انتهای استخر افزوده
    /// می‌شوند (ترتیب لاتنسیِ هرمس: coarena ← openrouter ← router9):
    /// * `KIAN_AI_COARENA_URL` (پیش‌فرض [`COARENA_BASE_URL`]) + `KIAN_AI_COARENA_KEYS`
    ///   (یا تک‌کلیدی `KIAN_AI_COARENA_KEY`) + `KIAN_AI_COARENA_MODELS`
    /// * `KIAN_AI_OPENROUTER_URL` (پیش‌فرض [`OPENROUTER_BASE_URL`]) + `KIAN_AI_OPENROUTER_KEYS`
    ///   (یا `KIAN_AI_OPENROUTER_KEY`) + `KIAN_AI_OPENROUTER_MODELS`
    /// * `KIAN_AI_ROUTER9_URL` (پیش‌فرض [`ROUTER9_BASE_URL`]) + `KIAN_AI_ROUTER9_KEYS`
    ///   (یا `KIAN_AI_ROUTER9_KEY`) + `KIAN_AI_ROUTER9_MODELS`
    ///
    /// سقف روزانه از `KIAN_AI_DAILY_TOKENS` (پیش‌فرض ۵۰٬۰۰۰).
    ///
    /// میان‌برهای «تک‌گیت‌وی» برای روترهای چندمدلی (مثل ۹روتر):
    /// * `KIAN_AI_BASE_URL` + `KIAN_AI_API_KEY` + `KIAN_AI_MODEL` → اسلات primary
    /// * `KIAN_AI_FALLBACK_MODEL` (+ اختیاری `KIAN_AI_FALLBACK_URL` / `KIAN_AI_FALLBACK_KEY`)
    ///   → اسلات secondary روی همان گیت‌وی.
    #[must_use]
    pub fn from_env() -> Self {
        Self::from_env_reader(|k| std::env::var(k).ok().filter(|v| !v.trim().is_empty()))
    }

    /// نسخهٔ تزریق‌پذیر [`AiConfig::from_env`] — خوانندهٔ env از بیرون می‌آید تا
    /// تست‌ها بدون دست‌کاری متغیر سراسری فرایند، استخر و ترتیب میان‌برها را اثبات کنند.
    ///
    /// اولویت هر فیلد: متغیر اختصاصیِ اسلات ← میان‌بر تک‌گیت‌وی ← خاموش (رشتهٔ خالی).
    #[must_use]
    pub fn from_env_reader(get: impl Fn(&str) -> Option<String>) -> Self {
        let base_url = get("KIAN_AI_BASE_URL");
        let api_key = get("KIAN_AI_API_KEY");
        let slot = |name: &str, url: Option<String>, key: Option<String>, model: Option<String>| {
            SlotConfig {
                name: name.to_owned(),
                url: url.unwrap_or_default(),
                key: key.unwrap_or_default(),
                model: model.unwrap_or_default(),
            }
        };
        // اسلات‌های منطقی با مقادیر خام (شاید فهرست کاما) — فعال‌ها باز می‌شوند
        let logical = vec![
            slot(
                SLOT_NAMES[0],
                get("KIAN_AI_PRIMARY_URL").or_else(|| base_url.clone()),
                get("KIAN_AI_PRIMARY_KEY").or_else(|| api_key.clone()),
                get("KIAN_AI_PRIMARY_MODEL").or_else(|| get("KIAN_AI_MODEL")),
            ),
            slot(
                SLOT_NAMES[1],
                get("KIAN_AI_SECONDARY_URL")
                    .or_else(|| get("KIAN_AI_FALLBACK_URL"))
                    .or_else(|| base_url.clone()),
                get("KIAN_AI_SECONDARY_KEY")
                    .or_else(|| get("KIAN_AI_FALLBACK_KEY"))
                    .or_else(|| api_key.clone()),
                get("KIAN_AI_SECONDARY_MODEL").or_else(|| get("KIAN_AI_FALLBACK_MODEL")),
            ),
            slot(
                SLOT_NAMES[2],
                get("KIAN_AI_TERTIARY_URL"),
                get("KIAN_AI_TERTIARY_KEY"),
                get("KIAN_AI_TERTIARY_MODEL"),
            ),
        ];
        let mut pool: Vec<SlotConfig> = Vec::new();
        for s in logical {
            if s.enabled() {
                // اسلات کامل — ضرب دکارتیِ «هر مدل × هر کلید» در استخر تخت
                for entry in expand_models_times_keys(&s.name, &s.url, &s.key, &s.model) {
                    push_unique(&mut pool, entry);
                }
            } else {
                // اسلات ناقص/خالی — خودش با flags خاموش در استخر می‌ماند تا
                // /ai/status همانند گذشته سه ردیف صادقانه را نشان دهد؛
                // اجرا فقط عضوهای enabled را لمس می‌کند. بدون dedup —
                // نام‌های متفاوتِ primary/secondary/tertiary باید همه حاضر بمانند.
                pool.push(s);
            }
        }
        // تامین‌کننده‌های سبک هرمس — لِین‌های اضافه در انتهای استخر
        // (name, url_var, default_url, keys_var, key_var, models_var)
        let providers: [(&str, &str, &str, &str, &str, &str); 3] = [
            (
                "coarena",
                "KIAN_AI_COARENA_URL",
                COARENA_BASE_URL,
                "KIAN_AI_COARENA_KEYS",
                "KIAN_AI_COARENA_KEY",
                "KIAN_AI_COARENA_MODELS",
            ),
            (
                "openrouter",
                "KIAN_AI_OPENROUTER_URL",
                OPENROUTER_BASE_URL,
                "KIAN_AI_OPENROUTER_KEYS",
                "KIAN_AI_OPENROUTER_KEY",
                "KIAN_AI_OPENROUTER_MODELS",
            ),
            (
                "router9",
                "KIAN_AI_ROUTER9_URL",
                ROUTER9_BASE_URL,
                "KIAN_AI_ROUTER9_KEYS",
                "KIAN_AI_ROUTER9_KEY",
                "KIAN_AI_ROUTER9_MODELS",
            ),
        ];
        for (name, url_var, default_url, keys_var, key_var, models_var) in providers {
            let Some(models) = get(models_var) else { continue };
            let Some(keys) = get(keys_var).or_else(|| get(key_var)) else { continue };
            let url = get(url_var).unwrap_or_else(|| default_url.to_owned());
            for entry in expand_models_times_keys(name, &url, &keys, &models) {
                push_unique(&mut pool, entry);
            }
        }
        Self {
            slots: pool,
            daily_cap_tokens: get("KIAN_AI_DAILY_TOKENS").and_then(|v| v.parse().ok()).unwrap_or(50_000),
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

/// پاکت ارسالی به لِین استخر.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatPacket {
    /// مدل لِین
    pub model: String,
    /// پیام‌ها — پیام‌های کاربر همیشه پس از redaction
    pub messages: Vec<PacketMessage>,
    /// سقف توکن خروجی اختیاری (مثلاً `Some(4096)` برای تولید HTML کامل)
    pub max_tokens: Option<u32>,
}

/// پاکت موفق لِین.
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
    /// نام لِین (اسلات منطقی یا تامین‌کنندهٔ سبک هرمس)
    pub slot: String,
    /// جزئیات کوتاه — هرگز شامل متن درخواست/کلید نیست
    pub detail: String,
}

/// خطای گیت‌وی — تایپ‌شده؛ متن خام آپ‌استریم هرگز بالا نمی‌آید.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GatewayError {
    /// هیچ لِینی کانفیگ نشده — 503 `AI_DISABLED`
    #[error("AI gateway disabled — no complete slot configured")]
    Disabled,
    /// همهٔ لِین‌های فعال شکست خوردند (و کم‌ضررینِ قابل‌قبولی نبود) — 502 `AI_UPSTREAM`
    #[error("all AI slots failed")]
    AllSlotsFailed {
        /// ریز شکست‌ها به ترتیب تلاش
        attempts: Vec<AttemptFailure>,
    },
}

/// وضعیت یک گروه لِین (اسلات منطقی یا تامین‌کنندهٔ سبک هرمس) برای سیم — بدون کلید.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotStatus {
    /// شمارهٔ گروه در ترتیب استخر (۱ مبنا)
    pub slot: u8,
    /// نام
    pub name: String,
    /// مدل‌های گروه — فهرست جداشده با کاما (حفظ ترتیب اولین مشاهده، بدون تکرار)
    pub model: String,
    /// فعال؟ (حداقل یک عضو گروه فعال)
    pub enabled: bool,
}

/// نتیجهٔ موفق گفت‌وگو.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatResult {
    /// متن پاسخ
    pub content: String,
    /// لِین برنده
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

/// شمارش نویسه‌های فارسی/عربی (`\u{0600}..=\u{06FF}`).
#[must_use]
pub fn persian_char_count(text: &str) -> usize {
    text.chars().filter(|c| ('\u{0600}'..='\u{06FF}').contains(c)).count()
}

/// حذف بلوک‌های کد محصور (```` ```...``` ````) — برای سنجش زبانِ «متنِ واقعی»
/// پاسخ، نه کدِ داخل بلوک (HTML/CSS طبیعتاً لاتین‌غالب است).
#[must_use]
pub fn strip_fenced_code_blocks(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_fence = false;
    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
            out.push('\n');
            continue;
        }
        if !in_fence {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

/// حذف توکن‌های رنگ هگز (`#RGB`/`#RRGGBB`/`#RRGGBBAA`) پیش از اسکن زنجیره —
/// رنگ‌های CSS واقعیِ آرتیفکت (مثل `#ffffff`) تباهی زبان نیستند.
fn scrub_hex_colors(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0usize;
    while i < chars.len() {
        if chars[i] == '#' {
            let mut j = i + 1;
            while j < chars.len() && chars[j].is_ascii_hexdigit() && j - i <= 8 {
                j += 1;
            }
            if (3..=8).contains(&(j - i - 1)) {
                i = j; // توکن کامل رنگ — حذف
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// آیا `min` نویسهٔ الفباییِ یکسانِ پشت‌سرهم هست؟
///
/// نویسهٔ غیرالفبایی (فاصله، نقطه‌گذاری، رقم، ``` ```` ```، `---`، `====`) زنجیره
/// را می‌شکند و رنگ‌های هگز (`#000000`، `#ffffff`) قبل از اسکن حذف می‌شوند —
/// پس جداکننده‌های مارک‌داون و CSS سالم می‌مانند و «هههههه» یا «aaaaaa» گرفته می‌شود.
fn has_identical_alpha_run(text: &str, min: usize) -> bool {
    let mut prev: Option<char> = None;
    let mut run = 0usize;
    for c in scrub_hex_colors(text).chars() {
        if c.is_alphabetic() {
            if prev == Some(c) {
                run += 1;
                if run >= min {
                    return true;
                }
            } else {
                run = 1;
            }
            prev = Some(c);
        } else {
            run = 0;
            prev = None;
        }
    }
    false
}

/// گیت کیفیت به سبک `brain.py::_looks_broken` — پاسخ خراب = شکست لِین.
///
/// * قانون ۱ (خالی/خیلی کوتاه): متن تریم‌شدهٔ پاسخ < ۵ نویسه — در حالت سازنده
///   < ۶۴ (آرتیفکت HTML کامل هرگز این‌قدر کوتاه نیست).
/// * قانون ۲ (خرابی انکودینگ): وجود `U+FFFD`.
/// * قانون ۳ (تباهی زنجیره): ≥ ۵ نویسهٔ الفبایی یکسان پشت‌سرهم.
/// * قانون ۴ (ناهماهنگی زبان):
///   * چت عادی: پرامپت فارسی (≥ ۱۰ نویسهٔ فارسی/عربی) اما پس از حذف بلوک‌های کد،
///     بیش از ۶۰٪ حروفِ پاسخ لاتینِ ASCII → پاسخ اشتباه‌زبان.
///   * سازنده: پاسخ باید تگ `<html` یا `<!doctype html` داشته باشد (بی‌حس به
///     بزرگی/کوچکی حروف) و اگر پرامپت فارسی است، حداقل ۵ نویسهٔ فارسی در پاسخ باشد —
///     HTML/CSS طبیعتاً لاتین‌غالب است و نباید رد شود.
#[must_use]
pub fn looks_broken(prompt: &str, reply: &str, is_builder: bool) -> bool {
    // ۱) خالی / خیلی کوتاه
    let min_chars = if is_builder { 64 } else { 5 };
    if reply.trim().chars().count() < min_chars {
        return true;
    }
    // ۲) خرابی انکودینگ
    if reply.contains('\u{FFFD}') {
        return true;
    }
    // ۳) تباهی زنجیرهٔ نویسهٔ الفبایی
    if has_identical_alpha_run(reply, 5) {
        return true;
    }
    // ۴) ناهماهنگی زبان
    let prompt_persian = persian_char_count(prompt) >= 10;
    if is_builder {
        let lower = reply.to_lowercase();
        if !(lower.contains("<!doctype html") || lower.contains("<html")) {
            return true;
        }
        if prompt_persian && persian_char_count(reply) < 5 {
            return true;
        }
        return false;
    }
    if prompt_persian {
        let stripped = strip_fenced_code_blocks(reply);
        let letters: Vec<char> = stripped.chars().filter(|c| c.is_alphabetic()).collect();
        if !letters.is_empty() {
            let latin = letters.iter().filter(|c| c.is_ascii_alphabetic()).count();
            if latin * 10 > letters.len() * 6 {
                return true;
            }
        }
    }
    false
}

/// گیت‌وی AI — redaction → استخر تخت با چسبندگی برندهٔ آخر → گیت کیفیت →
/// ذخیرهٔ کم‌ضررترین → نتیجهٔ تایپ‌شده.
pub struct AiGateway {
    /// پیکربندی بوت — `slots` استخر تخت اجرایی است
    pub config: AiConfig,
    client: Arc<dyn ModelClient>,
    /// شاخص لِین برندهٔ آخر (`brain.py::last_good`) — دفعهٔ بعد اول امتحان می‌شود
    last_good: AtomicUsize,
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
        Self { config, client, last_good: AtomicUsize::new(0) }
    }

    /// گیت‌وی فعال؟ حداقل یک عضو استخر کامل.
    #[must_use]
    pub fn enabled(&self) -> bool {
        self.config.slots.iter().any(SlotConfig::enabled)
    }

    /// برش استخر تخت اجرایی — همان `config.pool()`.
    #[must_use]
    pub fn pool(&self) -> &[SlotConfig] {
        self.config.pool()
    }

    /// اندازهٔ استخر اجرایی — تعداد عضوهای فعال `(url, key, model)` پس از ضرب
    /// دکارتی — برای `/ai/status` (عضوهای خاموشِ اسلات‌های ناقص شمرده نمی‌شوند).
    #[must_use]
    pub fn pool_size(&self) -> usize {
        self.config.slots.iter().filter(|s| s.enabled()).count()
    }

    /// طرح اجرای `brain.py`: برندهٔ آخر اول، بقیه به ترتیب طبیعی استخر.
    ///
    /// دقیقاً `[last_good] + [i for i in range(len(pool)) if i != last_good]`
    /// وقتی `last_good` معتبر باشد؛ در غیر این صورت ترتیب طبیعی.
    fn plan(&self, pool_len: usize) -> Vec<usize> {
        let lg = self.last_good.load(Ordering::Relaxed);
        let mut plan = Vec::with_capacity(pool_len);
        if lg < pool_len {
            plan.push(lg);
        }
        for i in 0..pool_len {
            if i != lg {
                plan.push(i);
            }
        }
        plan
    }

    /// وضعیت گروه‌های استخر برای سیم — بدون هیچ کلیدی.
    ///
    /// عضوهای استخر بر اساس نام گروه می‌شوند (حفظ ترتیب اولین مشاهده)؛ مدل‌های
    /// متمایز هر گروه با کاما می‌آیند و فعال بودن گروه = فعال بودن حداقل یک عضو.
    /// برای استخرِ سه‌اسلاتهٔ تک‌مدلی، خروجی با رفتار قبلی یک‌به‌یک برابر است.
    #[must_use]
    pub fn slot_statuses(&self) -> Vec<SlotStatus> {
        let mut out: Vec<SlotStatus> = Vec::new();
        for s in &self.config.slots {
            if let Some(group) = out.iter_mut().find(|g| g.name == s.name) {
                if !group.model.split(',').any(|m| m.trim() == s.model) {
                    group.model.push_str(&format!(",{}", s.model));
                }
                group.enabled |= s.enabled();
            } else {
                out.push(SlotStatus {
                    slot: u8::try_from(out.len() + 1).unwrap_or(u8::MAX),
                    name: s.name.clone(),
                    model: s.model.clone(),
                    enabled: s.enabled(),
                });
            }
        }
        out
    }

    /// گفت‌وگو — پیام‌های کاربر redact می‌شوند، لِین‌های استخر به ترتیب طرح امتحان می‌شوند.
    ///
    /// # Errors
    /// [`GatewayError::Disabled`] بدون لِین فعال؛ [`GatewayError::AllSlotsFailed`]
    /// با ریز تلاش‌ها وقتی همهٔ لِین‌های فعال شکست خوردند و کم‌ضررینِ قابل‌قبولی نبود.
    pub async fn chat(&self, messages: Vec<PacketMessage>) -> Result<ChatResult, GatewayError> {
        self.chat_with_max_tokens(messages, None).await
    }

    /// گفت‌وگوی چندنوبتی با سقف توکن خروجی اختیاری (مثلاً `Some(4096)` برای تولید کد HTML).
    ///
    /// اسکنر رازها روی پیام‌های `user` اجرا می‌شود تا کلید/توکن کاربر هرگز به بیرون نرود؛
    /// پرامپت سیستمی و کد HTML خروجیِ نوبت‌های قبلی `assistant` دست‌نخورده می‌مانند.
    ///
    /// # Errors
    /// [`GatewayError::Disabled`] بدون لِین فعال؛ [`GatewayError::AllSlotsFailed`]
    /// با ریز تلاش‌ها وقتی همهٔ لِین‌های فعال شکست خوردند.
    pub async fn chat_with_max_tokens(
        &self,
        messages: Vec<PacketMessage>,
        max_tokens: Option<u32>,
    ) -> Result<ChatResult, GatewayError> {
        self.chat_with_quality(messages, max_tokens, false).await
    }

    /// هستهٔ کامل گیت‌وی با گیت کیفیت به سبک `brain.py`.
    ///
    /// `is_builder = true` یعنی نوبتِ تولید آرتیفکت HTML — پاسخ باید تگ سند HTML
    /// داشته باشد و قانون کوتاهیِ ۶۴ نویسه روی آن اعمال می‌شود؛ در حالت چت، پاسخ
    /// اشتباه‌زبان (پرامپت فارسی، پاسخ لاتین‌غالب) رد می‌شود.
    ///
    /// استخرِ کم‌ضررترین (`least_bad`): اولین پاسخِ HTTP 200 با متنِ غیروخالی که
    /// گیت کیفیت را رد می‌کند، بلافاصله پذیرفته نمی‌شود — ذخیره می‌شود و لِین
    /// بعدی امتحان می‌شود. اگر همهٔ لِین‌ها شکست خوردند:
    /// * چت عادی → کم‌ضررترین برگردانده می‌شود (دسترس‌پذیری بر کیفیت — همان `brain.py`).
    /// * سازنده → کم‌ضررترین فقط وقتی برگردانده می‌شود که سند HTML قابل‌استخراج
    ///   (`<!DOCTYPE html` یا `<html`) داشته باشد؛ وگرنه [`GatewayError::AllSlotsFailed`]
    ///   بالا می‌رود تا موتور سازنده به سنتز محلیِ تعیینی برسد.
    ///
    /// # Errors
    /// [`GatewayError::Disabled`] بدون لِین فعال؛ [`GatewayError::AllSlotsFailed`]
    /// وقتی هیچ لِینی پاسخ قابل‌قبول نداد.
    pub async fn chat_with_quality(
        &self,
        messages: Vec<PacketMessage>,
        max_tokens: Option<u32>,
        is_builder: bool,
    ) -> Result<ChatResult, GatewayError> {
        // ۱) redaction — روی پیام‌های کاربر قبل از دیدن هر لِین؛ دسته‌ها بدون محتوا گزارش می‌شوند
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

        // ۲) استخر فعال + طرح اجرا (برندهٔ آخر اول)
        let pool: Vec<&SlotConfig> = self.config.slots.iter().filter(|s| s.enabled()).collect();
        if pool.is_empty() {
            return Err(GatewayError::Disabled);
        }
        let plan = self.plan(pool.len());

        // مرجع زبان برای گیت کیفیت — آخرین پیام کاربر (همان `ref` در brain.py)
        let prompt_ref = redacted
            .iter()
            .rev()
            .find(|m| m.role == "user")
            .or_else(|| redacted.last())
            .map(|m| m.content.as_str())
            .unwrap_or("");

        // ۳) اسکن استخر — گیت کیفیت + ذخیرهٔ کم‌ضررترین
        let mut attempts: Vec<AttemptFailure> = Vec::new();
        let mut least_bad: Option<(usize, CompletionPacket)> = None;
        for idx in plan {
            let slot = pool[idx];
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
                    if looks_broken(prompt_ref, &done.content, is_builder) {
                        attempts.push(AttemptFailure {
                            slot: slot.name.clone(),
                            detail: "quality gate — garbled/low-quality reply (skipped)".into(),
                        });
                        if least_bad.is_none() {
                            least_bad = Some((idx, done));
                        }
                        continue;
                    }
                    // برنده — چسبندگی: دفعهٔ بعد همین لِین اول امتحان می‌شود
                    self.last_good.store(idx, Ordering::Relaxed);
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

        // ۴) همه‌جا شکست — کم‌ضررترینِ قابل‌قبول یا خطای تایپ‌شده
        if let Some((idx, done)) = least_bad {
            let salvage = if is_builder {
                let lower = done.content.to_lowercase();
                lower.contains("<!doctype html") || lower.contains("<html")
            } else {
                true
            };
            if salvage {
                let slot = pool[idx];
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
        }
        Err(GatewayError::AllSlotsFailed { attempts })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::ScriptedClient;
    use std::collections::HashMap;

    fn cfg_from(pairs: &[(&str, &str)]) -> AiConfig {
        let env: HashMap<&str, String> =
            pairs.iter().map(|(k, v)| (*k, (*v).to_owned())).collect();
        AiConfig::from_env_reader(|k| env.get(k).cloned())
    }

    // ── قانون ضرب دکارتی استخر ─────────────────────────────────────────────

    /// فهرستِ کلید × فهرستِ مدل دقیقاً به ترتیب «هر مدل، هر کلید» باز می‌شود و
    /// سه‌تایی تکراری حذف می‌شود (حفظ اولین ترتیب).
    #[test]
    fn cross_product_expands_models_x_keys_ordered_with_dedup() {
        let cfg = cfg_from(&[
            ("KIAN_AI_PRIMARY_URL", "https://or.example/v1"),
            ("KIAN_AI_PRIMARY_KEY", "k1,k2,k1"),
            ("KIAN_AI_PRIMARY_MODEL", "mA,mB"),
        ]);
        let triples: Vec<(&str, &str, &str)> = cfg
            .slots
            .iter()
            .map(|s| (s.url.as_str(), s.key.as_str(), s.model.as_str()))
            .collect();
        assert_eq!(
            triples,
            vec![
                ("https://or.example/v1", "k1", "mA"),
                ("https://or.example/v1", "k2", "mA"),
                ("https://or.example/v1", "k1", "mB"),
                ("https://or.example/v1", "k2", "mB"),
                // اسلات‌های secondary/tertiaryِ کانفیگ‌نشده — صادقانه خاموش می‌مانند
                ("", "", ""),
                ("", "", ""),
            ],
            "model-outer × key-inner, dedup preserves first order"
        );
        assert!(cfg.slots.iter().take(4).all(|s| s.name == "primary"));
        assert_eq!(cfg.pool().iter().filter(|s| s.enabled()).count(), 4);
    }

    /// تامین‌کننده‌های سبک هرمس به انتهای استخر افزوده می‌شوند (ترتیب هرمس:
    /// coarena ← openrouter ← router9)، URL پیش‌فرض دارند و تک‌کلیدی هم می‌پذیرند.
    #[test]
    fn hermes_style_providers_append_in_brain_py_order() {
        let cfg = cfg_from(&[
            ("KIAN_AI_COARENA_KEY", "coa-1"),
            ("KIAN_AI_COARENA_MODELS", "gpt-6-luna,gemini-3-8-flash"),
            ("KIAN_AI_OPENROUTER_KEYS", "orA,orB"),
            ("KIAN_AI_OPENROUTER_MODELS", "x:free"),
            ("KIAN_AI_ROUTER9_URL", "https://r9.example/v1"),
            ("KIAN_AI_ROUTER9_KEY", "sk-r9"),
            ("KIAN_AI_ROUTER9_MODELS", "oc/a,oc/b"),
        ]);
        let flat: Vec<(&str, &str, &str, &str)> = cfg
            .slots
            .iter()
            .map(|s| (s.name.as_str(), s.url.as_str(), s.key.as_str(), s.model.as_str()))
            .collect();
        assert_eq!(
            flat,
            vec![
                // اسلات‌های منطقیِ کانفیگ‌نشده — صادقانه خاموش، بدون حذف
                ("primary", "", "", ""),
                ("secondary", "", "", ""),
                ("tertiary", "", "", ""),
                ("coarena", COARENA_BASE_URL, "coa-1", "gpt-6-luna"),
                ("coarena", COARENA_BASE_URL, "coa-1", "gemini-3-8-flash"),
                ("openrouter", OPENROUTER_BASE_URL, "orA", "x:free"),
                ("openrouter", OPENROUTER_BASE_URL, "orB", "x:free"),
                ("router9", "https://r9.example/v1", "sk-r9", "oc/a"),
                ("router9", "https://r9.example/v1", "sk-r9", "oc/b"),
            ],
            "providers append after logical slots in Hermes latency order"
        );
    }

    /// بدون تامین‌کنندهٔ هرمس و با اسلات‌های تک‌مقداری، استخر = رفتار قبلی
    /// (سه اسلات به همان ترتیب) — سازگاری کامل به عقب.
    #[test]
    fn single_value_slots_keep_legacy_layout() {
        let cfg = cfg_from(&[
            ("KIAN_AI_PRIMARY_URL", "https://a/v1"),
            ("KIAN_AI_PRIMARY_KEY", "k1"),
            ("KIAN_AI_PRIMARY_MODEL", "m1"),
            ("KIAN_AI_SECONDARY_URL", "https://b/v1"),
            ("KIAN_AI_SECONDARY_KEY", "k2"),
            ("KIAN_AI_SECONDARY_MODEL", "m2"),
        ]);
        assert_eq!(cfg.slots.len(), 3);
        assert_eq!(cfg.slots[0].name, "primary");
        assert_eq!(cfg.slots[1].name, "secondary");
        assert!(cfg.slots[1].enabled());
        assert!(!cfg.slots[2].enabled(), "unconfigured tertiary stays disabled");
    }

    /// اسلات ناقص (بدون کلید) باز نمی‌شود و گروه خاموش گزارش می‌شود؛ مدل‌های
    /// چندتاییِ یک گروه در وضعیت با کاما می‌آیند.
    #[test]
    fn status_groups_pool_entries_by_name() {
        let cfg = cfg_from(&[
            ("KIAN_AI_PRIMARY_URL", "https://a/v1"),
            ("KIAN_AI_PRIMARY_KEY", "k1"),
            ("KIAN_AI_PRIMARY_MODEL", "m1,m2,m3"),
            ("KIAN_AI_TERTIARY_URL", "https://t/v1"),
            ("KIAN_AI_TERTIARY_MODEL", "tm1,tm2"), // بدون کلید — خاموش
        ]);
        let st = AiGateway::new(cfg, Arc::new(ScriptedClient::default())).slot_statuses();
        assert_eq!(st.len(), 3);
        assert_eq!(st[0].name, "primary");
        assert_eq!(st[0].model, "m1,m2,m3");
        assert!(st[0].enabled);
        assert_eq!(st[1].name, "secondary");
        assert_eq!(st[1].model, "");
        assert!(!st[1].enabled, "unconfigured secondary stays disabled");
        assert_eq!(st[2].name, "tertiary");
        assert_eq!(st[2].model, "tm1,tm2");
        assert!(!st[2].enabled, "no key → group disabled");
    }

    // ── چسبندگی last_good ──────────────────────────────────────────────────

    /// لِین ۰ شکست و لِین ۱ موفق در فراخوان اول؛ فراخوان دوم مستقیم لِین ۱ را
    /// می‌زند و اصلاً لِین ۰ را لمس نمی‌کند (طرح `[last_good] + بقیه`).
    #[tokio::test]
    async fn last_good_sticky_entry_wins_next_call_without_retrying_broken() {
        let scripted = Arc::new(ScriptedClient::new(vec![
            Err("upstream status 502".into()),
            Ok(CompletionPacket {
                content: "all systems go now".into(),
                prompt_tokens: 5,
                completion_tokens: 7,
            }),
            Ok(CompletionPacket {
                content: "sticky winner again".into(),
                prompt_tokens: 5,
                completion_tokens: 7,
            }),
        ]));
        let gw = AiGateway::new(
            cfg_from(&[
                ("KIAN_AI_PRIMARY_URL", "https://a/v1"),
                ("KIAN_AI_PRIMARY_KEY", "k1"),
                ("KIAN_AI_PRIMARY_MODEL", "m1"),
                ("KIAN_AI_SECONDARY_URL", "https://b/v1"),
                ("KIAN_AI_SECONDARY_KEY", "k2"),
                ("KIAN_AI_SECONDARY_MODEL", "m2"),
            ]),
            scripted.clone(),
        );
        let msgs = vec![PacketMessage { role: "user".into(), content: "hello there".into() }];

        // فراخوان ۱: primary شکست → secondary برنده
        let r1 = gw.chat(msgs.clone()).await.expect("call 1 must succeed");
        assert_eq!(r1.provider, "secondary");
        assert_eq!(r1.model, "m2");
        assert_eq!(scripted.called_slots(), vec!["primary", "secondary"]);

        // فراخوان ۲: مستقیم secondary — primary هرگز لمس نمی‌شود
        let r2 = gw.chat(msgs.clone()).await.expect("call 2 must succeed");
        assert_eq!(r2.provider, "secondary");
        assert_eq!(
            scripted.called_slots(),
            vec!["primary", "secondary", "secondary"],
            "sticky last_good must skip the broken entry entirely"
        );
    }

    /// اسلات ۳تایی چندمدلی: برندهٔ وسط استخر چسبنده می‌شود — طرح از برنده شروع
    /// می‌شود و بعد بقیه به ترتیب طبیعی ادامه می‌دهند.
    #[tokio::test]
    async fn sticky_plan_covers_whole_pool_in_natural_order_after_winner() {
        // استخر: primary×3 مدل؛ لِین ۱ (m2) برنده می‌شود
        let scripted = Arc::new(ScriptedClient::new(vec![
            Err("upstream status 502".into()), // m1
            Ok(CompletionPacket {
                content: "پاسخ تمیز فارسی با جزئیات کافی".into(),
                prompt_tokens: 4,
                completion_tokens: 6,
            }), // m2 برنده
            Ok(CompletionPacket {
                content: "پاسخ تمیز فارسی با جزئیات کافی".into(),
                prompt_tokens: 4,
                completion_tokens: 6,
            }), // m3
            Ok(CompletionPacket {
                content: "پاسخ تمیز فارسی با جزئیات کافی".into(),
                prompt_tokens: 4,
                completion_tokens: 6,
            }), // فراخوان ۲: m2
        ]));
        let gw = AiGateway::new(
            cfg_from(&[
                ("KIAN_AI_PRIMARY_URL", "https://a/v1"),
                ("KIAN_AI_PRIMARY_KEY", "k1"),
                ("KIAN_AI_PRIMARY_MODEL", "m1,m2,m3"),
            ]),
            scripted.clone(),
        );
        let msgs = vec![PacketMessage {
            role: "user".into(),
            content: "یک سوال کاملاً فارسی برای پاسخ فارسی".into(),
        }];
        let r1 = gw.chat(msgs.clone()).await.expect("call 1");
        assert_eq!(r1.model, "m2", "m1 failed, m2 won");
        assert_eq!(scripted.called_slots(), vec!["primary", "primary"], "two tries in call 1");
        let r2 = gw.chat(msgs.clone()).await.expect("call 2");
        assert_eq!(r2.model, "m2", "sticky winner m2 first — m3/m1 untouched");
        assert_eq!(scripted.called_slots().len(), 3, "only one request in call 2");
    }

    // ── گیت کیفیت ──────────────────────────────────────────────────────────

    /// قانون ۱ و ۲: خالی/خیلی کوتاه (چت <۵) و نویسهٔ خرابی انکودینگ.
    #[test]
    fn quality_gate_catches_empty_short_and_corrupt() {
        assert!(looks_broken("سلام", "   ", false), "whitespace-only");
        assert!(looks_broken("سلام", "ab", false), "2 chars < 5");
        assert!(looks_broken("hi", "ok!", false), "3 chars < 5");
        assert!(
            !looks_broken("سلام", "سلام دوست من", false),
            "exactly at/over the 5-char floor passes"
        );
        assert!(looks_broken("hi", "\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}", false));
        // حالت سازنده: سقف ۶۴
        assert!(looks_broken("سلام", "پاسخ کوتاه", true), "builder floor is 64");
        assert!(
            !looks_broken(
                "hi",
                "<!DOCTYPE html><html><body><main><h1>Profit Calc Tool</h1><p>Full page body here</p></main></body></html>",
                true
            ),
            "valid-size builder doc passes"
        );
    }

    /// قانون ۳: زنجیرهٔ ≥۵ الفبایی یکسان گرفته می‌شود؛ جداکننده‌های مارک‌داون و
    /// رنگ هگز و بلوک‌های کد سالم می‌مانند.
    #[test]
    fn quality_gate_char_run_vs_separators() {
        assert!(looks_broken("hi", "هههههه چه جالب", false), "Persian degeneration");
        assert!(looks_broken("hi", "aaaaaa", false), "latin degeneration");
        assert!(!looks_broken("hi", "aaaa cc bb", false), "4-run is under the floor");
        assert!(
            !looks_broken("hi", "----- ===== ----- -----", false),
            "punctuation runs must never trip the gate"
        );
        assert!(
            !looks_broken("hi", "background:#000000; color:#ffffff;", false),
            "hex colors must never trip the gate"
        );
        assert!(
            !looks_broken("hi", "```html\n<article>\n```\n-----", false),
            "fence markers are punctuation"
        );
    }

    /// قانون ۴ چت: پاسخ لاتین‌غالب به پرامپت فارسی رد می‌شود؛ پاسخ فارسی و
    /// پاسخِ کد-محورِ داخل fence با توضیح فارسی قبول است.
    #[test]
    fn quality_gate_language_mismatch_chat() {
        let fa_prompt = "این یک پرسش کاملاً فارسی است که بیش از ده نویسهٔ فارسی دارد";
        assert!(
            looks_broken(fa_prompt, "Sure! Here is a complete English answer about the topic", false),
            "English-only reply to a Persian question is broken"
        );
        assert!(
            !looks_broken(fa_prompt, "پاسخ کامل فارسی با توضیح کافی برای کاربر", false),
            "Persian reply passes"
        );
        assert!(
            !looks_broken(
                fa_prompt,
                "توضیح فارسی:\n```html\n<html><body>latin heavy content</body></html>\n```\nپایان توضیح",
                false
            ),
            "fenced code is exempt from the language ratio"
        );
        assert!(
            !looks_broken("hi there friend", "Sure! Here is a complete English answer", false),
            "non-Persian prompt skips the language rule"
        );
    }

    /// قانون ۴ سازنده: سند HTML لازم است؛ با پرامپت فارسی حداقل ۵ نویسهٔ فارسی
    /// هم؛ بدون پرامپت فارسی فقط تگ سند کافی است.
    #[test]
    fn quality_gate_builder_mode_requires_html_and_persian() {
        let fa = "یک ماشین حساب سود ارز دیجیتال با تم تیره بساز";
        let good = "<!DOCTYPE html><html lang=\"fa\" dir=\"rtl\"><body><h1>ماشین حساب سود مرکب</h1><script>let x=1;</script></body></html>";
        assert!(!looks_broken(fa, good, true), "valid Persian RTL artifact passes");
        assert!(
            looks_broken(fa, "توضیح کوتاه بدون هیچ کد اچ تی ام الی در پاسخ کاربر", true),
            "no HTML tag → broken"
        );
        let latin_doc = "<!DOCTYPE html><html><body><h1>Profit Calc</h1><script>let x=1;</script></body></html>";
        assert!(
            looks_broken(fa, latin_doc, true),
            "HTML without any Persian chars for a Persian prompt → broken"
        );
        assert!(
            !looks_broken("build a profit calculator app", latin_doc, true),
            "non-Persian prompt skips the Persian-chars rule"
        );
        assert!(
            looks_broken(fa, "<html><body>کوتاه</body></html>", true),
            "31 chars < 64 builder floor → broken"
        );
    }

    // ── ذخیرهٔ کم‌ضررترین ───────────────────────────────────────────────────

    /// لِین ۰ خراب (پاسخ انگلیسی به پرسش فارسی) و لِین ۱ تمیز → لِین ۱ برنده است.
    /// اگر لِین ۱ به‌جای پاسخ تمیز ۴۲۹ بدهد، کم‌ضررینِ ذخیره‌شده برای چت برمی‌گردد.
    #[tokio::test]
    async fn least_bad_stash_prefers_clean_and_salvages_when_all_fail() {
        let fa_q = "یک پرسش کاملاً فارسی با بیش از ده نویسهٔ فارسی برای گیت زبان";
        let garbled = "Totally unrelated English answer here for a Persian question";
        let clean = "پاسخ تمیز فارسی با جزئیات کافی برای کاربر";
        let mk = |c: &str| Ok(CompletionPacket {
            content: c.to_owned(),
            prompt_tokens: 6,
            completion_tokens: 9,
        });

        // حالت الف: خراب ← تمیز؛ تمیز برنده است
        let scripted = Arc::new(ScriptedClient::new(vec![mk(garbled), mk(clean)]));
        let gw = AiGateway::new(
            cfg_from(&[
                ("KIAN_AI_PRIMARY_URL", "https://a/v1"),
                ("KIAN_AI_PRIMARY_KEY", "k1"),
                ("KIAN_AI_PRIMARY_MODEL", "m1"),
                ("KIAN_AI_SECONDARY_URL", "https://b/v1"),
                ("KIAN_AI_SECONDARY_KEY", "k2"),
                ("KIAN_AI_SECONDARY_MODEL", "m2"),
            ]),
            scripted.clone(),
        );
        let msgs = vec![PacketMessage { role: "user".into(), content: fa_q.to_owned() }];
        let r = gw.chat(msgs.clone()).await.expect("clean entry must win");
        assert_eq!(r.provider, "secondary");
        assert_eq!(r.content, clean);
        assert_eq!(scripted.called_slots(), vec!["primary", "secondary"]);

        // حالت ب: خراب ← ۴۲۹؛ کم‌ضررین ذخیره‌شده برای چت برگردانده می‌شود
        let scripted = Arc::new(ScriptedClient::new(vec![
            mk(garbled),
            Err("upstream status 429 Too Many Requests".into()),
        ]));
        let gw = AiGateway::new(
            cfg_from(&[
                ("KIAN_AI_PRIMARY_URL", "https://a/v1"),
                ("KIAN_AI_PRIMARY_KEY", "k1"),
                ("KIAN_AI_PRIMARY_MODEL", "m1"),
                ("KIAN_AI_SECONDARY_URL", "https://b/v1"),
                ("KIAN_AI_SECONDARY_KEY", "k2"),
                ("KIAN_AI_SECONDARY_MODEL", "m2"),
            ]),
            scripted.clone(),
        );
        let r = gw.chat(msgs.clone()).await.expect("least-bad must be salvaged for chat");
        assert_eq!(r.provider, "primary", "salvaged from the first (garbled) entry");
        assert_eq!(r.content, garbled);
        assert_eq!(scripted.called_slots(), vec!["primary", "secondary"]);
        assert!(
            gw.last_good.load(Ordering::Relaxed) == 0,
            "salvaged least-bad must not update stickiness"
        );
    }

    /// حالت سازنده: پاسخِ خرابِ دارای سند HTML قابل استخراج → کم‌ضررین برمی‌گردد
    /// (موتور سازنده آن را استخراج می‌کند)؛ پاسخ خرابِ بدون HTML → خطا تا سنتز
    /// محلیِ تعیینی اجرا شود.
    #[tokio::test]
    async fn builder_least_bad_salvages_only_real_html_documents() {
        let fa_q = "یک ابزار فروشگاهی با رابط فارسی بساز";
        let corrupt_doc = "<!DOCTYPE html><html dir=\"rtl\"><body><main><h1>\u{FFFD}\u{FFFD}\u{FFFD}ابزار فروش\u{FFFD}\u{FFFD}</h1></main></body></html>";
        let no_html = "توضیح کوتاه بدون هیچ سند اچ تی ام الی در پاسخ کاربر";
        let mk = |c: &str| Ok(CompletionPacket {
            content: c.to_owned(),
            prompt_tokens: 8,
            completion_tokens: 12,
        });
        let pairs = &[
            ("KIAN_AI_PRIMARY_URL", "https://a/v1"),
            ("KIAN_AI_PRIMARY_KEY", "k1"),
            ("KIAN_AI_PRIMARY_MODEL", "m1"),
        ];

        // سند HTML خراب → نجات برای سازنده
        let scripted = Arc::new(ScriptedClient::new(vec![mk(corrupt_doc)]));
        let gw = AiGateway::new(cfg_from(pairs), scripted.clone());
        let msgs = vec![PacketMessage { role: "user".into(), content: fa_q.to_owned() }];
        let r = gw
            .chat_with_quality(msgs.clone(), Some(BUILDER_MAX_TOKENS), true)
            .await
            .expect("doctype-bearing least-bad must be salvaged for builder");
        assert_eq!(r.content, corrupt_doc);

        // بدون HTML → خطای تایپ‌شده تا BuilderEngine سنتز محلی را اجرا کند
        let scripted = Arc::new(ScriptedClient::new(vec![mk(no_html)]));
        let gw = AiGateway::new(cfg_from(pairs), scripted.clone());
        let err = gw
            .chat_with_quality(msgs, Some(BUILDER_MAX_TOKENS), true)
            .await
            .expect_err("non-HTML least-bad must NOT be salvaged for builder");
        let GatewayError::AllSlotsFailed { attempts } = err else {
            panic!("expected AllSlotsFailed");
        };
        assert_eq!(attempts.len(), 1);
        assert!(attempts[0].detail.contains("quality gate"), "attempts: {:?}", attempts);
    }
}
