//! `loop_guard` — گارد حلقهٔ بات‌به‌بات (پورت `gateway/bot_loop_guard.py` گنجینهٔ
//! `NousResearch` — Wave 2 - Step 2).
//!
//! دو پروفایل هرمس که همدیگر را جواب می‌دهند هرگز خودشان متوقف نمی‌شوند؛
//! `{PLATFORM}_ALLOW_BOTS` فقط ورود را می‌گذراند. این گارد سرعتِ پیام‌های
//! بات‌نوشتهٔ هر گفتگو را می‌شمارد و با سه سنجهٔ مستقل، طوفانِ پاسخِ بی‌پایان را
//! می‌خفه‌کند:
//! ۱. **هوپهای پیاپی بات** — از آخرین پیامِ انسان، بیش از
//!    [`MAX_CONSECUTIVE_BOT_TURNS`] پیام باتِ پیاپی در یک گفتگو یعنی حلقه؛
//!    پیامِ انسان شمارنده را صفر می‌کند (پورت `chain_admit`).
//! ۲. **بارِ تکراری** — همان هشِ متن از همان فرستنده در پنجرهٔ لغزان
//!    ([`DEFAULT_WINDOW_SECS`] ثانیه) = پینگ تکراری؛ رد (پورت `duplicate`).
//! ۳. **پینگ‌پنگ** — متنِ یکسان که بین دو فرستندهٔ بات تاب می‌خورد
//!    (A→B→A با هشِ یکسان) = رد (پورت `reply_to_reply`/`max_hops`).
//!
//! گارد کاملاً قطعی است: زمان از بیرون تزریق می‌شود و تست بدون خواب/شانس است.
//! پیامِ انسان هرگز سرکوب نمی‌شود (fail-open برای انسان).
#![forbid(unsafe_code)]

/// سقفِ پیش‌فرضِ پیام باتِ پیاپی پس از آخرین پیام انسان (پورت `max_hops`،
/// بسته‌تر برای کیان: ۳).
pub const MAX_CONSECUTIVE_BOT_TURNS: u32 = 3;

/// پنجرهٔ لغزانِ پیش‌فرض بر حسب ثانیه — هش‌ها و سرعت داخل این پنجره معتبرند.
pub const DEFAULT_WINDOW_SECS: u64 = 60;

/// سقفِ پیام از یک `(chat, sender)` در پنجره — سیلابِ خالص هم سرکوب می‌شود.
pub const MAX_EVENTS_PER_WINDOW: usize = 5;

/// دلیلِ سرکوب — برای لاگ و تله‌متری.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopReason {
    /// بیش از سقفِ پیام باتِ پیاپی پس از آخرین پیام انسان.
    MaxConsecutiveBotTurns,
    /// همان متن از همان فرستنده در پنجرهٔ لغزان.
    DuplicatePayload,
    /// تاب‌خوردنِ متنِ یکسان بین دو فرستندهٔ بات (A→B→A).
    PingPong,
    /// سیلاب از یک فرستنده در پنجره.
    RateLimit,
}

/// تصمیم گارد برای یک پیام ورودی.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopGuardDecision {
    Allow,
    Suppress(LoopReason),
}

/// یک رخداد ورودی در تاریخچهٔ گفتگو.
#[derive(Debug, Clone, PartialEq, Eq)]
struct GuardEvent {
    sender: String,
    hash: u64,
    at: u64,
}

/// گارد حلقهٔ بات‌ها — حالتِ هر گفتگو: شمارندهٔ هوپِ باتِ پیاپی، تاریخچهٔ
///bounded رخدادها و شمارندهٔ سرعتِ هر `(chat, sender)`.
#[derive(Debug, Default)]
pub struct BotLoopGuard {
    window_secs: u64,
    max_consecutive_bot_turns: u32,
    /// شمارندهٔ پیام باتِ پیاپی به‌ازای هر گفتگو — پیام انسان صفرش می‌کند.
    bot_turns: std::collections::HashMap<String, u32>,
    /// آخرین رخداد باتِ مجاز هر گفتگو — برای انقضای زنجیرهٔ خاموش
    /// (پورت «chain updated» گنجینه: زنجیرهٔ ساکتِ گذشته از پنجره می‌میرد).
    last_bot_at: std::collections::HashMap<String, u64>,
    /// تاریخچهٔ bounded هر گفتگو — آخرین `HISTORY_CAP` رخداد.
    history: std::collections::HashMap<String, Vec<GuardEvent>>,
    /// مهر زمانیِ رخدادهای هر `(chat, sender)` برای سنجش سرعت.
    velocity: std::collections::HashMap<(String, String), Vec<u64>>,
}

/// سقف تاریخچهٔ هر گفتگو — حافظهٔ bounded؛ پنجرهٔ لغزان هرگز به این نمی‌رسد.
const HISTORY_CAP: usize = 32;

impl BotLoopGuard {
    /// گارد با پنجرهٔ و سقفِ پیش‌فرض.
    #[must_use]
    pub fn new() -> Self {
        Self::with_limits(DEFAULT_WINDOW_SECS, MAX_CONSECUTIVE_BOT_TURNS)
    }

    /// گارد با حدودِ صریح — برای تست و تنظیمِ هر پلتفرم.
    #[must_use]
    pub fn with_limits(window_secs: u64, max_consecutive_bot_turns: u32) -> Self {
        Self {
            window_secs: window_secs.max(1),
            max_consecutive_bot_turns: max_consecutive_bot_turns.max(1),
            bot_turns: std::collections::HashMap::new(),
            last_bot_at: std::collections::HashMap::new(),
            history: std::collections::HashMap::new(),
            velocity: std::collections::HashMap::new(),
        }
    }

    /// یک پیام ورودی را می‌سنجد و تصمیم می‌دهد.
    ///
    /// ترتیبِ سنجه‌ها (پورت ترتیبِ `chain_admit`): انسان → صفرِ هوپ؛ بات →
    /// سیلاب، هوپِ پیاپی، بارِ تکراری، پینگ‌پنگ. رخدادِ مجاز ثبت می‌شود؛ رخدادِ
    /// سرکوب‌شده ثبت نمی‌شود (پیامِ خفه‌شده سمتِ مقصد نمی‌رود و نباید در
    /// تاریخچه و سرعت حساب شود).
    pub fn observe(
        &mut self,
        chat_id: &str,
        sender_id: &str,
        is_bot: bool,
        text: &str,
        now_secs: u64,
    ) -> LoopGuardDecision {
        if !is_bot {
            // انسان — fail-open: زنجیرهٔ بات تمام و شمارنده صفر (پورت `human`)
            self.bot_turns.insert(chat_id.to_owned(), 0);
            return LoopGuardDecision::Allow;
        }

        // ── پاک‌سازیِ پنجرهٔ لغزان (قطعی با زمانِ تزریقی) ──
        self.sweep(chat_id, sender_id, now_secs);

        // ── سنجهٔ ۱: سیلاب از یک فرستنده در پنجره ──
        let vel_key = (chat_id.to_owned(), sender_id.to_owned());
        if self
            .velocity
            .get(&vel_key)
            .is_some_and(|stamps| stamps.len() >= MAX_EVENTS_PER_WINDOW)
        {
            return LoopGuardDecision::Suppress(LoopReason::RateLimit);
        }

        // ── انقضای زنجیرهٔ خاموش — گپِ بیش از پنجره = زنجیرهٔ جدید
        // (پورت «now - chain.updated > cooldown_seconds» گنجینه)
        let stale = self
            .last_bot_at
            .get(chat_id)
            .is_none_or(|&t| now_secs.saturating_sub(t) > self.window_secs);
        if stale {
            self.bot_turns.insert(chat_id.to_owned(), 0);
        }

        // ── سنجهٔ ۲: هوپِ پیاپی بات پس از آخرین پیام انسان ──
        let turns = self.bot_turns.entry(chat_id.to_owned()).or_insert(0);
        if *turns >= self.max_consecutive_bot_turns {
            return LoopGuardDecision::Suppress(LoopReason::MaxConsecutiveBotTurns);
        }

        let hash = payload_hash(text);
        let history = self.history.entry(chat_id.to_owned()).or_default();

        // ── سنجهٔ ۳: پینگ‌پنگ — تابِ همان هش بین دو فرستندهٔ بات (A→B→A)؛
        // امضای دقیقِ طوفانِ بات‌به‌بات، مقدم بر تکرارِ ساده ──
        if history.len() >= 2 {
            let last = &history[history.len() - 1];
            let prev = &history[history.len() - 2];
            if last.sender != sender_id
                && last.hash == hash
                && prev.sender == sender_id
                && prev.hash == hash
            {
                return LoopGuardDecision::Suppress(LoopReason::PingPong);
            }
        }

        // ── سنجهٔ ۴: بارِ تکراری از همان فرستنده در پنجره ──
        if history.iter().any(|e| e.sender == sender_id && e.hash == hash) {
            return LoopGuardDecision::Suppress(LoopReason::DuplicatePayload);
        }

        // ── مجاز — ثبت ──
        *turns += 1;
        self.last_bot_at.insert(chat_id.to_owned(), now_secs);
        if history.len() >= HISTORY_CAP {
            history.remove(0);
        }
        history.push(GuardEvent { sender: sender_id.to_owned(), hash, at: now_secs });
        self.velocity
            .entry(vel_key)
            .or_default()
            .push(now_secs);
        LoopGuardDecision::Allow
    }

    /// رخدادهای کهنه‌تر از پنجره را از سرعت و تاریخچه می‌خواباند (قطعی).
    fn sweep(&mut self, chat_id: &str, sender_id: &str, now_secs: u64) {
        let cutoff = now_secs.saturating_sub(self.window_secs);
        let vel_key = (chat_id.to_owned(), sender_id.to_owned());
        if let Some(stamps) = self.velocity.get_mut(&vel_key) {
            stamps.retain(|&t| t > cutoff);
            if stamps.is_empty() {
                self.velocity.remove(&vel_key);
            }
        }
        if let Some(hist) = self.history.get_mut(chat_id) {
            hist.retain(|e| e.at > cutoff);
        }
    }

    /// تعداد هوپِ پیاپیِ فعلی یک گفتگو (تله‌متری و تست).
    #[must_use]
    pub fn consecutive_bot_turns(&self, chat_id: &str) -> u32 {
        self.bot_turns.get(chat_id).copied().unwrap_or(0)
    }
}

/// هش پایدار متن — فروکاستِ فاصله‌ها (پورت `sha256(" ".join(text.split()))`
/// گنجینه با خروجی سبک‌تر): چند فاصله/خط جدید = یک فاصله؛ خالی = ۰.
fn payload_hash(text: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325; // FNV-1a offset basis
    let mut last_ws = true;
    for ch in text.chars() {
        if ch.is_whitespace() {
            if last_ws {
                continue; // فاصله‌های پیاپی = یکی
            }
            last_ws = true;
            hash = mix(hash, b' ');
        } else {
            last_ws = false;
            let mut buf = [0u8; 4];
            let enc = ch.encode_utf8(&mut buf);
            for b in enc.bytes() {
                hash = mix(hash, b);
            }
        }
    }
    hash
}

/// یک بایت را به هش FNV-1a می‌آمیزد.
fn mix(mut hash: u64, byte: u8) -> u64 {
    hash ^= u64::from(byte);
    hash.wrapping_mul(0x0000_0100_0000_01b3)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// چهارمین پیام باتِ پیاپی سرکوب می‌شود؛ پیام انسان زنجیره را صفر می‌کند.
    #[test]
    fn consecutive_bot_turns_are_capped_and_human_resets() {
        let mut g = BotLoopGuard::new();
        assert_eq!(g.observe("c", "bot-a", true, "پیام یک", 1), LoopGuardDecision::Allow);
        assert_eq!(g.observe("c", "bot-a", true, "پیام دو", 2), LoopGuardDecision::Allow);
        assert_eq!(g.observe("c", "bot-a", true, "پیام سه", 3), LoopGuardDecision::Allow);
        assert_eq!(
            g.observe("c", "bot-a", true, "پیام چهار", 4),
            LoopGuardDecision::Suppress(LoopReason::MaxConsecutiveBotTurns)
        );
        // انسان زنجیره را می‌شکند — دوباره سه هوپ مجاز
        assert_eq!(g.observe("c", "human-1", false, "خودت جواب بده", 5), LoopGuardDecision::Allow);
        assert_eq!(g.consecutive_bot_turns("c"), 0);
        assert_eq!(g.observe("c", "bot-b", true, "سوال اول", 6), LoopGuardDecision::Allow);
        assert_eq!(g.observe("c", "bot-b", true, "سوال دوم", 7), LoopGuardDecision::Allow);
        assert_eq!(g.observe("c", "bot-b", true, "سوال سوم", 8), LoopGuardDecision::Allow);
        assert_eq!(
            g.observe("c", "bot-b", true, "سوال چهارم", 9),
            LoopGuardDecision::Suppress(LoopReason::MaxConsecutiveBotTurns)
        );
    }

    /// همان متن از همان فرستنده در پنجره = بار تکراری؛ بعد از پنجره دوباره مجاز.
    #[test]
    fn duplicate_payload_is_suppressed_within_window_then_allowed() {
        let mut g = BotLoopGuard::new();
        assert_eq!(g.observe("c", "bot", true, "وضعیت چطوره؟", 10), LoopGuardDecision::Allow);
        // فاصله‌ها نرمال می‌شوند — همین متن با فاصله‌های متفاوت تکراری است
        assert_eq!(
            g.observe("c", "bot", true, "وضعیت   چطوره؟", 20),
            LoopGuardDecision::Suppress(LoopReason::DuplicatePayload)
        );
        assert_eq!(
            g.observe("c", "bot", true, "وضعیت چطوره؟", 30),
            LoopGuardDecision::Suppress(LoopReason::DuplicatePayload)
        );
        // پنجرهٔ ۶۰ ثانیه گذشت — متن همان مجاز می‌شود (دورِ جدید)
        assert_eq!(g.observe("c", "bot", true, "وضعیت چطوره؟", 10 + 61 + 30), LoopGuardDecision::Allow);
    }

    /// پینگ‌پنگ A→B→A با متنِ یکسان سرکوب می‌شود — جوهرِ طوفان بات‌به‌بات.
    #[test]
    fn ping_pong_between_two_bots_is_suppressed() {
        let mut g = BotLoopGuard::new();
        let msg = "سلام، وضعیت چطوره؟";
        assert_eq!(g.observe("c", "a", true, msg, 1), LoopGuardDecision::Allow);
        assert_eq!(g.observe("c", "b", true, msg, 2), LoopGuardDecision::Allow);
        // A دوباره همان متن — تاب کامل شد؛ پینگ‌پنگ مقدم بر تکرارِ ساده است
        assert_eq!(
            g.observe("c", "a", true, msg, 3),
            LoopGuardDecision::Suppress(LoopReason::PingPong)
        );
        // متنِ متفاوت از همان A — نه تکراری است نه پینگ‌پنگ؛ هوپِ سوم مجاز
        assert_eq!(g.observe("c", "a", true, "متن دیگر", 4), LoopGuardDecision::Allow);
        assert_eq!(g.consecutive_bot_turns("c"), 3);
        // هوپِ چهارم — سقف
        assert_eq!(
            g.observe("c", "a", true, "متن دیگرِ تازه", 5),
            LoopGuardDecision::Suppress(LoopReason::MaxConsecutiveBotTurns)
        );
    }

    /// سیلاب از یک فرستنده در پنجره — حتی وقتی هوپ‌ها با پیام انسان ریست می‌شوند.
    #[test]
    fn velocity_flood_is_suppressed() {
        let mut g = BotLoopGuard::new();
        // پنج پیامِ متمایزِ بات، هر کدام پس از یک پیام انسان — هوپ‌ها ریست می‌شوند
        // ولی سرعتِ (chat, spam) در پنجره انباشته می‌شود
        let mut t = 1u64;
        for i in 0..MAX_EVENTS_PER_WINDOW {
            assert_eq!(g.observe("c", "human", false, "سوال تازه", t), LoopGuardDecision::Allow);
            t += 1;
            assert_eq!(
                g.observe("c", "spam", true, &format!("پاسخ شمارهٔ {i}"), t),
                LoopGuardDecision::Allow
            );
            t += 10;
        }
        // ششمین پیام از همان فرستنده — هنوز در پنجره، سیلاب سرکوب می‌شود
        assert_eq!(
            g.observe("c", "spam", true, "پاسخ تازهٔ دیگر", t),
            LoopGuardDecision::Suppress(LoopReason::RateLimit)
        );
        // فرستندهٔ دیگرِ همان گفتگو — سیلابِ او نیست و هوپ‌ها ریست شده‌اند
        assert_eq!(g.observe("c", "other", true, "پیام مستقل", t + 1), LoopGuardDecision::Allow);
        // انسان ریست می‌کند؛ پس از گذشتن پنجره، فرستندهٔ اول هم جا باز می‌کند
        assert_eq!(g.observe("c", "human", false, "کافی است", t + 2), LoopGuardDecision::Allow);
        assert_eq!(
            g.observe("c", "spam", true, "پاسخ بعد از پنجره", t + 200),
            LoopGuardDecision::Allow
        );
    }

    /// گفتگوهای جدا مستقل‌اند — گاردِ یک گفتگو دیگری را نمی‌گیرد.
    #[test]
    fn conversations_are_isolated() {
        let mut g = BotLoopGuard::new();
        assert_eq!(g.observe("c1", "bot", true, "مشترک", 1), LoopGuardDecision::Allow);
        assert_eq!(g.observe("c2", "bot", true, "مشترک", 2), LoopGuardDecision::Allow);
        assert_eq!(g.consecutive_bot_turns("c1"), 1);
        assert_eq!(g.consecutive_bot_turns("c2"), 1);
    }

    /// حدودِ صریح — پنجره و سقفِ هوپ قابل تنظیم؛ سقفِ ۱ یعنی فقط یک هوپ،
    /// و زنجیرهٔ خاموشِ گذشته از پنجره می‌میرد (پورت chain decay).
    #[test]
    fn explicit_limits_shape_the_guard() {
        let mut g = BotLoopGuard::with_limits(10, 1);
        assert_eq!(g.observe("c", "a", true, "اول", 1), LoopGuardDecision::Allow);
        assert_eq!(
            g.observe("c", "a", true, "دوم", 2),
            LoopGuardDecision::Suppress(LoopReason::MaxConsecutiveBotTurns)
        );
        // گپِ خاموشیِ بیش از پنجره — زنجیرهٔ قبلی منقضی و متنِ «اول» هم دوباره مجاز
        assert_eq!(g.observe("c", "a", true, "اول", 1 + 11), LoopGuardDecision::Allow);
    }

    /// انسان هرگز سرکوب نمی‌شود — حتی با متن تکراری و سیلاب.
    #[test]
    fn humans_are_never_suppressed() {
        let mut g = BotLoopGuard::new();
        for i in 0..10 {
            assert_eq!(
                g.observe("c", "human", false, "همان سوال", i as u64 + 1),
                LoopGuardDecision::Allow,
                "پیام انسان هرگز سرکوب نمی‌شود"
            );
        }
    }

    /// هش پایدار — فاصله‌های پیاپی فرومی‌ریزند و متن‌های معنایی برابر هش می‌شوند.
    #[test]
    fn payload_hash_normalizes_whitespace() {
        assert_eq!(payload_hash("وضعیت چطوره؟"), payload_hash("وضعیت   \nچطوره؟"));
        assert_ne!(payload_hash("وضعیت چطوره؟"), payload_hash("وضعیت چطوره؟؟"));
        assert_eq!(payload_hash(""), payload_hash("   \t\n  "));
    }
}
