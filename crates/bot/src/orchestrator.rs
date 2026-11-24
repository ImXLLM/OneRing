//! هستهٔ خالص orchestrator — فرمان‌ها، سقف نرخ با ساعت تزریقی، برش پیام.
//!
//! قواعد:
//! * **صفر I/O** — داده فقط از [`KernelPort`] می‌آید؛ خروجی فقط [`BotAction`].
//! * سقف نرخ per-chat با ساعت تزریقی (`now_ms`) — آفلاین‌تست‌پذیر؛ هشدار یک‌بار
//!   در هر پنجره (پاسخ به اسپم، خودش اسپم نمی‌سازد).
//! * برش پیام روی مرز کاراکتر — سقف سخت تلگرام ۴۰۹۶.
//! * توکن بات هرگز وارد متن‌ها نمی‌شود (تست صریح؛ توکن اساساً اینجا نیست).
//! * فرمان‌های شهروند (`/balance`، `/me`) هویت را از کانال معتمد می‌گیرند —
//!   initData ضرب می‌شود و مسیر رسمی `POST /auth/telegram` طی می‌شود؛ از
//!   بدنهٔ کاربر هرگز.

use std::collections::{HashMap, VecDeque};

use crate::port::KernelPort;
use crate::update::BotUpdate;

/// سقف سخت طول پیام تلگرام.
pub const TELEGRAM_MAX_CHARS: usize = 4096;

/// یک پیام با دکمه‌های لینک اختیاری — تنها کنشی که بات می‌کند.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BotAction {
    /// چت مقصد.
    pub chat_id: i64,
    /// متن (RTL فارسی، بدون راز).
    pub text: String,
    /// دکمه‌های inline لینک (یک ردیف شیشه‌ای — ورود به مینی‌اپ و بازار).
    pub links: Vec<LinkButton>,
}

/// دکمهٔ لینک inline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkButton {
    /// برچسب دکمه.
    pub label: String,
    /// URL (https یا t.me).
    pub url: String,
}

/// سیاست بات — از env شِل می‌آید؛ هیچ رازی در متن‌ها نیست.
#[derive(Debug, Clone)]
pub struct BotPolicy {
    /// نام کاربری بات (بدون `@`) — برای نرمال‌سازی `/cmd@Bot`.
    pub bot_username: Option<String>,
    /// URL مینی‌اپ — دکمهٔ ورود.
    pub miniapp_url: String,
    /// پنجرهٔ سقف نرخ به میلی‌ثانیه.
    pub rate_window_ms: u64,
    /// بیشینهٔ پیام در هر پنجره per-chat.
    pub rate_max_per_window: u32,
    /// سقف طول پاسخ (≤ ۴۰۹۶).
    pub reply_max_chars: usize,
}

impl Default for BotPolicy {
    fn default() -> Self {
        Self {
            bot_username: None,
            miniapp_url: "http://localhost:3000".to_owned(),
            rate_window_ms: 60_000,
            rate_max_per_window: 20,
            reply_max_chars: TELEGRAM_MAX_CHARS,
        }
    }
}

/// هستهٔ خالص بات — بدون I/O، بدون راز در خروجی.
pub struct BotCore {
    policy: BotPolicy,
    /// زمان‌های ارسال اخیر per-chat.
    stamps: HashMap<i64, VecDeque<u64>>,
    /// آخرین هشدار نرخ per-chat — یک هشدار در هر پنجره.
    last_warn: HashMap<i64, u64>,
}

impl BotCore {
    /// ساخت با سیاست.
    #[must_use]
    pub fn new(policy: BotPolicy) -> Self {
        Self { policy, stamps: HashMap::new(), last_warn: HashMap::new() }
    }

    /// سیاست (نمایشی).
    #[must_use]
    pub fn policy(&self) -> &BotPolicy {
        &self.policy
    }

    /// پردازش یک آپدیت — قلب بات. خالص: هر اثر جانبی فقط `BotAction`.
    ///
    /// `now_ms` از ساعت تزریقی شِل می‌آید (آفلاین‌تست‌پذیر).
    pub async fn handle(
        &mut self,
        update: &BotUpdate,
        now_ms: u64,
        port: &dyn KernelPort,
    ) -> Vec<BotAction> {
        // ۱) سقف نرخ per-chat — هشدار یک‌بار در پنجره
        match self.rate_verdict(update.chat_id, now_ms) {
            RateVerdict::Reject { warn: Some((chat, retry_ms)) } => {
                let secs = (retry_ms / 1000).max(1);
                return vec![Action::text(
                    chat,
                    format!("کمی آرام‌تر — پس از {secs} ثانیه دوباره بپرسید. /help"),
                )];
            }
            RateVerdict::Reject { warn: None } => return Vec::new(),
            RateVerdict::Allow => {}
        }

        // ۲) مسیریابی فرمان (با نرمال‌سازی پسوند @Bot)
        let cmd = update.normalized_command(self.policy.bot_username.as_deref());
        let actions = match cmd.as_deref() {
            Some("start") => vec![self.welcome(update.chat_id, None)],
            Some("help") => vec![Action::help(update.chat_id)],
            Some("id") => vec![Action::ident(update.chat_id, update)],
            Some("link") => vec![self.link(update.chat_id)],
            Some("economy") => vec![self.economy(update.chat_id, port).await],
            Some("balance") => vec![self.balance(update.chat_id, update, now_ms, port).await],
            Some("me") => vec![self.me(update.chat_id, update, now_ms, port).await],
            Some(_) => vec![Action::text(
                update.chat_id,
                "این دستور را ناشناخته بودم 🤔 — با /help فهرست فرمان‌ها را ببین.",
            )],
            // پیام متنی عادی = نخست پاسخ‌گر فوری FAQ (صفر توکن — پورت بات هرمس؛
            // پوشهٔ ۰۳ گنجینه)؛ اگر هیچ قلابی نخورد → فرصت طلایی معرفی همان
            // خوش‌آمدِ جذابِ /start
            None => {
                if let Some(answer) = kian_hermes::match_instant_faq(&update.text) {
                    vec![Action::text(update.chat_id, answer.to_owned())]
                } else {
                    vec![self.welcome(update.chat_id, Some("پیامت را گرفتم! 👋"))]
                }
            }
        };
        // ۳) برش روی مرز کاراکتر — سقف سیاست، سقف سخت تلگرام
        actions
            .into_iter()
            .map(|mut a| {
                a.text = clip(a.text, self.policy.reply_max_chars);
                a
            })
            .collect()
    }

    // ── سقف نرخ ─────────────────────────────────────────────────────────────

    /// حکم نرخ per-chat — پنجرهٔ لغزان با ساعت تزریقی.
    fn rate_verdict(&mut self, chat_id: i64, now_ms: u64) -> RateVerdict {
        let window = self.policy.rate_window_ms;
        let max = usize::try_from(self.policy.rate_max_per_window).unwrap_or(usize::MAX);
        let stamps = self.stamps.entry(chat_id).or_default();
        while stamps.front().is_some_and(|t| now_ms.saturating_sub(*t) > window) {
            stamps.pop_front();
        }
        if stamps.len() < max {
            stamps.push_back(now_ms);
            return RateVerdict::Allow;
        }
        // عبور از سقف — هشدار فقط یک‌بار در هر پنجره
        let warned = self.last_warn.get(&chat_id).is_some_and(|w| now_ms - *w <= window);
        if warned {
            return RateVerdict::Reject { warn: None };
        }
        self.last_warn.insert(chat_id, now_ms);
        let retry_ms = stamps.front().map_or(window, |t| window.saturating_sub(now_ms.saturating_sub(*t)));
        RateVerdict::Reject { warn: Some((chat_id, retry_ms)) }
    }

    // ── فرمان‌ها ─────────────────────────────────────────────────────────────

    /// پیام خوش‌آمد سه‌خطی — ارزش پیشنهادی پلتفرم + دکمه‌های شیشه‌ای ورود.
    ///
    /// برای `/start` بدون پیش‌گفتار و برای پیام متنی عادی با یک سلام کوتاه
    /// فرستاده می‌شود؛ در هر دو حالت دکمه‌های مستقیم ورود به مینی‌اپ و بازار
    /// پایین پیام شناور است تا کاربر با یک تپ وارد شود.
    fn welcome(&self, chat_id: i64, preface: Option<&str>) -> BotAction {
        let mut text = String::new();
        if let Some(p) = preface {
            text.push_str(p);
            text.push_str("\n\n");
        }
        text.push_str(
            concat!(
                "✨ به پادشاهی KIAN خوش آمدی!\n\n",
                "۱. با هوش مصنوعی در ۱ دقیقه وب‌اپ و ابزار HTML بساز ✨\n",
                "۲. لینک فروشت را پخش کن و کریپتو بگیر 💎\n",
                "۳. یا لینک محصولات بقیه را پخش کن و تا ۵۰٪ پورسانت آنی بگیر 🔗\n\n",
                "همین حالا شروع کن 👇 (راهنما: /help)"
            ),
        );
        let market_url = format!("{}/market", self.policy.miniapp_url.trim_end_matches('/'));
        Action::with_links(
            chat_id,
            text,
            vec![
                LinkButton {
                    label: "🚀 ورود به مینی‌اپ KIAN".to_owned(),
                    url: self.policy.miniapp_url.clone(),
                },
                LinkButton { label: "🛒 بازار KIAN".to_owned(), url: market_url },
            ],
        )
    }

    /// `/link` — لینک مینی‌اپ + دکمه.
    fn link(&self, chat_id: i64) -> BotAction {
        Action::with_links(
            chat_id,
            format!("مینی‌اپ KIAN:\n{}", self.policy.miniapp_url),
            vec![LinkButton {
                label: "باز کردن مینی‌اپ".to_owned(),
                url: self.policy.miniapp_url.clone(),
            }],
        )
    }

    /// `/economy` — اقتصاد ملی از کرنل (مادهٔ ۶).
    async fn economy(&self, chat_id: i64, port: &dyn KernelPort) -> BotAction {
        match port.economy().await {
            Ok(e) => Action::text(
                chat_id,
                format!(
                    "اقتصاد ملی KIAN (مادهٔ ۶):\n\
                     در گردش: {} Đ\n\
                     خزانه: {} Đ\n\
                     قفل اسکرو: {} Đ\n\
                     کل ضرب: {} Đ — کل سوزانده: {} Đ\n\
                     حساب‌ها: {} — امانت باز: {}\n\n\
                     منبع: دفتر دوطرفهٔ کرنل — هیچ عددی دست‌ساز نیست.",
                    num(e.circulating),
                    num(e.treasury),
                    num(e.escrow_locked),
                    num(e.minted_all),
                    num(e.burned_all),
                    num(e.accounts),
                    num(e.open_escrows),
                ),
            ),
            Err(e) => Action::text(chat_id, e.user_text()),
        }
    }

    /// `/balance` — نشست شهروند از مسیر رسمی هویت، سپس تراز.
    async fn balance(
        &self,
        chat_id: i64,
        update: &BotUpdate,
        now_ms: u64,
        port: &dyn KernelPort,
    ) -> BotAction {
        let now_secs = i64::try_from(now_ms / 1000).unwrap_or(i64::MAX);
        match port.me_via_session(update.from_id, update.username.as_deref(), now_secs).await {
            Ok(m) => Action::text(
                chat_id,
                format!(
                    "تراز شما:\n\
                     آزاد: {} Đ\n\
                     قفل اسکرو: {} Đ\n\n\
                     جزئیات ولت و بازار در مینی‌اپ.",
                    num(m.balance),
                    num(m.escrow_locked),
                ),
            ),
            Err(e) => Action::text(chat_id, e.user_text()),
        }
    }

    /// `/me` — پروندهٔ شهروند.
    async fn me(
        &self,
        chat_id: i64,
        update: &BotUpdate,
        now_ms: u64,
        port: &dyn KernelPort,
    ) -> BotAction {
        let now_secs = i64::try_from(now_ms / 1000).unwrap_or(i64::MAX);
        match port.me_via_session(update.from_id, update.username.as_deref(), now_secs).await {
            Ok(m) => {
                let ton = m
                    .ton_address
                    .as_deref()
                    .map_or_else(|| "ثبت نشده (/me در مینی‌اپ)".to_owned(), str::to_owned);
                Action::text(
                    chat_id,
                    format!(
                        "پروندهٔ شما:\n\
                         حساب دفتر: {}\n\
                         username: @{}\n\
                         آدرس TON: {}",
                        m.account_id,
                        m.username.as_deref().unwrap_or("—"),
                        ton,
                    ),
                )
            }
            Err(e) => Action::text(chat_id, e.user_text()),
        }
    }
}

// ── سازنده‌های کوچک ──────────────────────────────────────────────────────────

enum RateVerdict {
    Allow,
    /// `warn: Some` فقط یک‌بار در پنجره؛ بعدش بی‌صدا.
    Reject { warn: Option<(i64, u64)> },
}

/// برش امن روی مرز کاراکتر — هرگز پنیکربیتی نمی‌شکند.
fn clip(text: String, cap: usize) -> String {
    if text.chars().count() <= cap {
        return text;
    }
    let mut s: String = text.chars().take(cap.saturating_sub(1)).collect();
    s.push('…');
    s
}

/// سازندهٔ پرایوت اکشن‌ها — برش در `handle` انجام می‌شود.
struct Action;

impl Action {
    fn text(chat_id: i64, text: impl Into<String>) -> BotAction {
        BotAction { chat_id, text: text.into(), links: Vec::new() }
    }

    fn with_links(chat_id: i64, text: String, links: Vec<LinkButton>) -> BotAction {
        BotAction { chat_id, text, links }
    }

    /// `/help` — فهرست فرمان‌ها (بدون نیاز به سیاست).
    fn help(chat_id: i64) -> BotAction {
        Self::text(
            chat_id,
            concat!(
                "فرمان‌های بات KIAN:\n",
                "/start — خوش‌آمد و ورود به مینی‌اپ\n",
                "/balance — تراز Din و قفل اسکرو (نشست شهروند)\n",
                "/me — پروندهٔ من (حساب دفتر، آدرس TON)\n",
                "/economy — اقتصاد ملی (مادهٔ ۶)\n",
                "/link — لینک مینی‌اپ\n",
                "/id — شناسهٔ تلگرام من\n\n",
                "هویت فقط از تلگرام می‌آید؛ رمز و کلیدی اینجا جایی وارد نمی‌شود."
            ),
        )
    }

    /// `/id` — شناسهٔ تلگرام (بدون کرنل).
    fn ident(chat_id: i64, update: &BotUpdate) -> BotAction {
        let name = update.username.as_deref().unwrap_or("—");
        Self::text(
            chat_id,
            format!("شناسهٔ تلگرام شما: {}\nusername: @{}", update.from_id, name),
        )
    }
}

/// قالب‌بندی عدد با جداکنندهٔ هزارگان (بی‌طرفِ لوکال — برای RTL همخوان).
fn num(n: i64) -> String {
    let s = n.abs().to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    let bytes = s.as_bytes();
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(char::from(*b));
    }
    if n < 0 {
        out.insert(0, '-');
    }
    out
}

// ── تست‌ها — همهٔ آفلاین با FakePort ─────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::port::{BotEconomy, BotError, BotMe};
    use async_trait::async_trait;
    use std::sync::Mutex;

    /// پورت اسکریپت‌شده — صفر شبکه.
    struct FakePort {
        economy: Mutex<Result<BotEconomy, BotError>>,
        me: Mutex<Result<BotMe, BotError>>,
    }

    impl FakePort {
        fn ok() -> Self {
            Self {
                economy: Mutex::new(Ok(BotEconomy {
                    circulating: 1_234_567,
                    treasury: 100_000,
                    escrow_locked: 5_000,
                    minted_all: 2_000_000,
                    burned_all: 10,
                    accounts: 7,
                    open_escrows: 2,
                })),
                me: Mutex::new(Ok(BotMe {
                    username: Some("kia".to_owned()),
                    account_id: "citizen:777".to_owned(),
                    ton_address: Some("UQ…".to_owned()),
                    balance: 500,
                    escrow_locked: 250,
                })),
            }
        }

        fn broken() -> Self {
            Self {
                economy: Mutex::new(Err(BotError::Transport("offline".to_owned()))),
                me: Mutex::new(Err(BotError::Kernel {
                    code: "IDENTITY_DISABLED".to_owned(),
                    message: "no token".to_owned(),
                })),
            }
        }
    }

    #[async_trait]
    impl KernelPort for FakePort {
        async fn economy(&self) -> Result<BotEconomy, BotError> {
            self.economy.lock().unwrap().clone()
        }
        async fn me_via_session(
            &self,
            _tg: i64,
            _username: Option<&str>,
            _now: i64,
        ) -> Result<BotMe, BotError> {
            self.me.lock().unwrap().clone()
        }
    }

    fn update(cmd: &str) -> BotUpdate {
        BotUpdate {
            update_id: 1,
            chat_id: 100,
            from_id: 777,
            username: Some("kia".to_owned()),
            command: (!cmd.is_empty()).then(|| cmd.to_owned()),
            args: Vec::new(),
            text: format!("/{cmd}"),
        }
    }

    const NOW: u64 = 1_760_000_000_000;

    #[tokio::test]
    async fn start_and_help_are_static_and_linked() {
        let mut core = BotCore::new(BotPolicy::default());
        let port = FakePort::ok();
        let start = core.handle(&update("start"), NOW, &port).await;
        assert_eq!(start.len(), 1);
        assert!(!start[0].links.is_empty(), "start must carry miniapp buttons");
        assert!(start[0].text.contains("KIAN"));
        // سه‌خط ارزش پیشنهادی + دکمهٔ بازار
        assert!(start[0].text.contains("هوش مصنوعی"));
        assert!(start[0].text.contains("۵۰٪ پورسانت"));
        assert!(start[0].links.iter().any(|l| l.url.ends_with("/market")));
        let help = core.handle(&update("help"), NOW + 1, &port).await;
        assert!(help[0].text.contains("/balance"));
    }

    #[tokio::test]
    async fn balance_uses_session_path_and_reports_honestly() {
        let mut core = BotCore::new(BotPolicy::default());
        let port = FakePort::ok();
        let out = core.handle(&update("balance"), NOW, &port).await;
        assert!(out[0].text.contains("500"));
        assert!(out[0].text.contains("250"));
        // کرنل خاموش/هویت غیرفعال → پیام صادقانه، نه عدد جعلی
        let mut core2 = BotCore::new(BotPolicy::default());
        let broken = FakePort::broken();
        let out2 = core2.handle(&update("balance"), NOW, &broken).await;
        assert!(out2[0].text.contains("غیرفعال"), "honest identity-disabled message");
    }

    #[tokio::test]
    async fn economy_formats_thousands_and_handles_failure() {
        let mut core = BotCore::new(BotPolicy::default());
        let port = FakePort::ok();
        let out = core.handle(&update("economy"), NOW, &port).await;
        assert!(out[0].text.contains("1,234,567"), "thousands separator");
        assert!(out[0].text.contains("مادهٔ ۶"));
        let mut core2 = BotCore::new(BotPolicy::default());
        let out2 = core2.handle(&update("economy"), NOW, &FakePort::broken()).await;
        assert!(out2[0].text.contains("کرنل در دسترس نیست"));
    }

    #[tokio::test]
    async fn rate_limit_warns_once_then_goes_silent() {
        let mut core = BotCore::new(BotPolicy { rate_max_per_window: 3, ..BotPolicy::default() });
        let port = FakePort::ok();
        // پیام‌های ۱..۳ مجاز
        for i in 0..3 {
            let out = core.handle(&update("id"), NOW + i, &port).await;
            assert_eq!(out.len(), 1, "msg {i} must pass");
        }
        // چهارمین — هشدار یک‌بار
        let warn = core.handle(&update("id"), NOW + 4, &port).await;
        assert_eq!(warn.len(), 1);
        assert!(warn[0].text.contains("آرام‌تر"));
        // پنجم و ششم — بی‌صدا
        assert_eq!(core.handle(&update("id"), NOW + 5, &port).await, [] as [BotAction; 0]);
        assert_eq!(core.handle(&update("id"), NOW + 6, &port).await, [] as [BotAction; 0]);
        // پنجرهٔ جدید → باز مجاز
        let again = core.handle(&update("id"), NOW + 61_000, &port).await;
        assert_eq!(again.len(), 1);
    }

    #[tokio::test]
    async fn unknown_command_and_plain_text_get_hints() {
        let mut core = BotCore::new(BotPolicy::default());
        let port = FakePort::ok();
        let unknown = core.handle(&update("nope"), NOW, &port).await;
        assert!(unknown[0].text.contains("ناشناخته"));
        let plain = core
            .handle(
                &BotUpdate {
                    update_id: 2,
                    chat_id: 100,
                    from_id: 777,
                    username: None,
                    command: None,
                    args: Vec::new(),
                    text: "سلام".to_owned(),
                },
                NOW + 1,
                &port,
            )
            .await;
        // پیام متنی عادی → خوش‌آمد کامل با دکمه‌های مینی‌اپ (نه پاسخ خشک)
        assert!(plain[0].text.contains("خوش آمدی"), "friendly welcome on plain text");
        assert!(plain[0].text.contains("پورسانت"));
        assert!(!plain[0].links.is_empty(), "welcome carries miniapp buttons");
    }

    #[tokio::test]
    async fn long_outputs_are_truncated_at_char_boundary() {
        let mut core = BotCore::new(BotPolicy { reply_max_chars: 40, ..BotPolicy::default() });
        let port = FakePort::ok();
        let out = core.handle(&update("economy"), NOW, &port).await;
        assert!(out[0].text.chars().count() <= 40, "must truncate to policy cap");
        // مرز کاراکتر — پنیکربیتی نمی‌شکند
        let _ = out[0].text.chars().last().expect("non-empty");
    }

    #[tokio::test]
    async fn bot_suffix_normalization_routes_commands() {
        let mut core = BotCore::new(BotPolicy {
            bot_username: Some("KiaBot".to_owned()),
            ..BotPolicy::default()
        });
        let port = FakePort::ok();
        let mut u = update("balance");
        u.command = Some("balance@kiabot".to_owned());
        let out = core.handle(&u, NOW, &port).await;
        assert!(out[0].text.contains("تراز شما"));
    }

    #[tokio::test]
    async fn actions_never_carry_secrets() {
        // توکن اساساً در orchestrator نیست — این تست قرارداد آینده را هم قفل می‌کند
        const TOKEN: &str = "123456:ABC-TOKEN-SHOULD-NEVER-LEAK";
        let mut core = BotCore::new(BotPolicy::default());
        let port = FakePort::ok();
        let mut all = Vec::new();
        for cmd in ["start", "help", "id", "link", "economy", "balance", "me", "nope"] {
            all.extend(core.handle(&update(cmd), NOW, &port).await);
        }
        for a in all {
            assert!(!a.text.contains(TOKEN));
            assert!(!a.text.contains("ABC-TOKEN"));
            for l in &a.links {
                assert!(!l.url.contains(TOKEN));
            }
        }
    }

    #[test]
    fn thousands_formatting_is_correct() {
        assert_eq!(num(1_234_567), "1,234,567");
        assert_eq!(num(-42), "-42");
        assert_eq!(num(0), "0");
        assert_eq!(num(999), "999");
        assert_eq!(num(1000), "1,000");
    }

    /// پیام متنی پرسش FAQ → پاسخ فوری صفرتوکنی؛ پیام بی‌ربط → خوش‌آمد عادی.
    #[tokio::test]
    async fn plain_text_questions_get_instant_faq_answers() {
        let mut core = BotCore::new(BotPolicy::default());
        // پیام متنی (بدون فرمان) با کلیدواژهٔ اسکرو → پاسخ ثابت فوری
        let mut u = update("");
        u.command = None;
        u.text = "خرید با امنیت اسکرو چطوره؟".to_owned();
        let out = core.handle(&u, NOW, &FakePort::ok()).await;
        assert_eq!(out.len(), 1);
        assert!(out[0].text.contains("اسکرو"), "پاسخ فوری: {}", out[0].text);
        assert!(out[0].text.contains("پارلمان"));
        // پیام بی‌ربط → مسیر خوش‌آمد عادی (نه FAQ)
        let mut u2 = update("");
        u2.command = None;
        u2.text = "سلام خوبی؟".to_owned();
        let out2 = core.handle(&u2, NOW, &FakePort::ok()).await;
        assert_eq!(out2.len(), 1);
        assert!(!out2[0].text.contains("اسکرو"), "بی‌ربط نباید FAQ بگیرد");
        // پاسخ FAQ زیر سقف برش تلگرام می‌ماند
        assert!(out[0].text.chars().count() <= 4096);
    }
}
