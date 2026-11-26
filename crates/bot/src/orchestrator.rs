//! The pure orchestrator kernel — commands, rate limiting with an injected clock, message truncation.
//!
//! Rules:
//! * **Zero I/O** — data comes only from [`KernelPort`]; output is only [`BotAction`].
//! * Per-chat rate limit with an injected clock (`now_ms`) — offline-testable; one warning
//!   per window (it replies to spam without producing spam itself).
//! * Message truncation on a character boundary — the Telegram hard cap is 4096.
//! * The bot token never enters any text (explicit test; the token is not here at all).
//! * Citizen commands (`/balance`, `/me`) take identity from the trusted channel —
//!   initData is HMAC-verified and the official `POST /auth/telegram` path is taken; never
//!   from the user body.
//! * The MAIRON sovereign suite ([`crate::sovereign`]) serves `/mairon`, `/trinity`,
//!   `/quant`, `/treasury`, `/warroom` and `/hud` as pure, deterministic formatters.

use std::collections::{HashMap, VecDeque};

use crate::port::KernelPort;
use crate::sovereign;
use crate::update::BotUpdate;

/// The Telegram message-length hard cap.
pub const TELEGRAM_MAX_CHARS: usize = 4096;

/// A message with optional link buttons — the only action the bot takes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BotAction {
    /// Destination chat.
    pub chat_id: i64,
    /// Text (international English, no secrets).
    pub text: String,
    /// Inline link buttons (one glass row — mini-app and market entry).
    pub links: Vec<LinkButton>,
}

/// One inline link button.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkButton {
    /// Button label.
    pub label: String,
    /// URL (https or t.me).
    pub url: String,
}

/// Bot policy — comes from the shell env; no secrets in any text.
#[derive(Debug, Clone)]
pub struct BotPolicy {
    /// Bot username (without `@`) — for `/cmd@Bot` normalization.
    pub bot_username: Option<String>,
    /// Mini-app URL — the entry button.
    pub miniapp_url: String,
    /// Rate-limit window in milliseconds.
    pub rate_window_ms: u64,
    /// Max messages per window, per chat.
    pub rate_max_per_window: u32,
    /// Reply length cap (<= 4096).
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

/// The pure bot kernel — no I/O, no secrets in output.
pub struct BotCore {
    policy: BotPolicy,
    /// Recent send times per chat.
    stamps: HashMap<i64, VecDeque<u64>>,
    /// Last rate warning per chat — one warning per window.
    last_warn: HashMap<i64, u64>,
}

impl BotCore {
    /// Build with a policy.
    #[must_use]
    pub fn new(policy: BotPolicy) -> Self {
        Self { policy, stamps: HashMap::new(), last_warn: HashMap::new() }
    }

    /// The policy (for display).
    #[must_use]
    pub fn policy(&self) -> &BotPolicy {
        &self.policy
    }

    /// Process one update — the heart of the bot. Pure: every side effect is a `BotAction` only.
    ///
    /// `now_ms` comes from the injected shell clock (offline-testable).
    pub async fn handle(
        &mut self,
        update: &BotUpdate,
        now_ms: u64,
        port: &dyn KernelPort,
    ) -> Vec<BotAction> {
        // 1) per-chat rate limit — one warning per window
        match self.rate_verdict(update.chat_id, now_ms) {
            RateVerdict::Reject { warn: Some((chat, retry_ms)) } => {
                let secs = (retry_ms / 1000).max(1);
                return vec![Action::text(
                    chat,
                    format!("Easy there — try again after {secs} seconds. /help"),
                )];
            }
            RateVerdict::Reject { warn: None } => return Vec::new(),
            RateVerdict::Allow => {}
        }

        // 2) command routing (with @Bot-suffix normalization)
        let cmd = update.normalized_command(self.policy.bot_username.as_deref());
        let actions = match cmd.as_deref() {
            Some("start") => vec![self.welcome(update.chat_id, None)],
            Some("help") => vec![Action::help(update.chat_id)],
            Some("id") => vec![Action::ident(update.chat_id, update)],
            Some("link") => vec![self.link(update.chat_id)],
            Some("economy") => vec![self.economy(update.chat_id, port).await],
            Some("balance") => vec![self.balance(update.chat_id, update, now_ms, port).await],
            Some("me") => vec![self.me(update.chat_id, update, now_ms, port).await],
            // the MAIRON sovereign suite — deterministic, zero-I/O formatters
            Some(c) if sovereign::is_sovereign_command(c) => vec![Action::text(
                update.chat_id,
                sovereign::format_sovereign_command(c).unwrap_or_default(),
            )],
            Some(_) => vec![Action::text(
                update.chat_id,
                "I did not recognize that command 🤔 — run /help to see the command list.",
            )],
            // a plain text message = first the instant FAQ responder (zero tokens — a port of the Hermes bot;
            // vault folder 03); if nothing hooks → the golden opportunity to pitch the same engaging
            // /start welcome
            None => {
                if let Some(answer) = kian_hermes::match_instant_faq(&update.text) {
                    vec![Action::text(
                        update.chat_id,
                        format!("⚡ Instant answer — zero tokens:\n\n{answer}"),
                    )]
                } else {
                    vec![self.welcome(update.chat_id, Some("Got your message! 👋"))]
                }
            }
        };
        // 3) truncation on a character boundary — the policy cap, the Telegram hard cap
        actions
            .into_iter()
            .map(|mut a| {
                a.text = clip(a.text, self.policy.reply_max_chars);
                a
            })
            .collect()
    }

    // ── rate limit ─────────────────────────────────────────────────────────────

    /// The per-chat rate verdict — a sliding window with the injected clock.
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
        // over the cap — warn only once per window
        let warned = self.last_warn.get(&chat_id).is_some_and(|w| now_ms - *w <= window);
        if warned {
            return RateVerdict::Reject { warn: None };
        }
        self.last_warn.insert(chat_id, now_ms);
        let retry_ms = stamps.front().map_or(window, |t| window.saturating_sub(now_ms.saturating_sub(*t)));
        RateVerdict::Reject { warn: Some((chat_id, retry_ms)) }
    }

    // ── commands ─────────────────────────────────────────────────────────────

    /// The welcome message — the platform value proposition + glass entry buttons.
    ///
    /// Sent for `/start` with no preamble, and for a plain text message with a short greeting;
    /// in both cases the direct mini-app and market entry buttons
    /// float under the message so the user enters with a single tap.
    fn welcome(&self, chat_id: i64, preface: Option<&str>) -> BotAction {
        let mut text = String::new();
        if let Some(p) = preface {
            text.push_str(p);
            text.push_str("\n\n");
        }
        text.push_str(
            concat!(
                "✨ Welcome to MAIRON — the sovereign founder OS!\n\n",
                "1. Build a web app or HTML tool with AI in 2 minutes ✨\n",
                "2. Share your storefront link and earn crypto 💎\n",
                "3. Or share product links and earn up to 20% instant commission 🔗\n\n",
                "Start right now 👇 (guide: /help)"
            ),
        );
        let market_url = format!("{}/market", self.policy.miniapp_url.trim_end_matches('/'));
        Action::with_links(
            chat_id,
            text,
            vec![
                LinkButton {
                    label: "🚀 Open the MAIRON mini-app".to_owned(),
                    url: self.policy.miniapp_url.clone(),
                },
                LinkButton { label: "🛒 MAIRON Market".to_owned(), url: market_url },
            ],
        )
    }

    /// `/link` — mini-app link + button.
    fn link(&self, chat_id: i64) -> BotAction {
        Action::with_links(
            chat_id,
            format!("MAIRON mini-app:\n{}", self.policy.miniapp_url),
            vec![LinkButton {
                label: "Open the mini-app".to_owned(),
                url: self.policy.miniapp_url.clone(),
            }],
        )
    }

    /// `/economy` — the national economy from the kernel (Article 6).
    async fn economy(&self, chat_id: i64, port: &dyn KernelPort) -> BotAction {
        match port.economy().await {
            Ok(e) => Action::text(
                chat_id,
                format!(
                    "MAIRON economy (Article 6):\n\
                     circulating: {} Đ\n\
                     treasury: {} Đ\n\
                     escrow locked: {} Đ\n\
                     total minted: {} Đ — total burned: {} Đ\n\
                     accounts: {} — open escrows: {}\n\n\
                     Source: the kernel double-entry ledger — no number is hand-edited.",
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

    /// `/balance` — a citizen session via the official identity path, then the balance.
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
                    "Your balance:\n\
                     available: {} Đ\n\
                     escrow locked: {} Đ\n\n\
                     Full wallet and market live in the mini-app.",
                    num(m.balance),
                    num(m.escrow_locked),
                ),
            ),
            Err(e) => Action::text(chat_id, e.user_text()),
        }
    }

    /// `/me` — the citizen profile.
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
                    .map_or_else(|| "not set (use /me inside the mini-app)".to_owned(), str::to_owned);
                Action::text(
                    chat_id,
                    format!(
                        "Your profile:\n\
                         account: {}\n\
                         username: @{}\n\
                         TON address: {}",
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

// ── small constructors ──────────────────────────────────────────────────────────

enum RateVerdict {
    Allow,
    /// `warn: Some` only once per window; silent afterwards.
    Reject { warn: Option<(i64, u64)> },
}

/// Safe truncation on a character boundary — never splits a character in half.
fn clip(text: String, cap: usize) -> String {
    if text.chars().count() <= cap {
        return text;
    }
    let mut s: String = text.chars().take(cap.saturating_sub(1)).collect();
    s.push('…');
    s
}

/// Private action constructor — truncation happens in `handle`.
struct Action;

impl Action {
    fn text(chat_id: i64, text: impl Into<String>) -> BotAction {
        BotAction { chat_id, text: text.into(), links: Vec::new() }
    }

    fn with_links(chat_id: i64, text: String, links: Vec<LinkButton>) -> BotAction {
        BotAction { chat_id, text, links }
    }

    /// `/help` — the command list (no policy needed).
    fn help(chat_id: i64) -> BotAction {
        Self::text(
            chat_id,
            concat!(
                "MAIRON bot commands:\n",
                "/start — welcome and the mini-app entry\n",
                "/mairon — the sovereign OS overview (IMPERIUM laws)\n",
                "/trinity — the 3-avatar build pipeline (IQ 136/139/150, REA >= 85)\n",
                "/quant — the causal CCXT desk (8 keyless venues, Sharpe >= 1.20)\n",
                "/treasury — Base-8453 settlement (USDC, $0xKIA, XTR, 4-stream split)\n",
                "/warroom — the 24-tick founder simulation (chairman directives)\n",
                "/hud — the neural topology and Grok voice modes\n",
                "/balance — your balance and escrow lock (citizen session)\n",
                "/me — my profile (account, TON address)\n",
                "/economy — the national economy (Article 6)\n",
                "/link — mini-app link\n",
                "/id — my Telegram id\n\n",
                "Identity comes only from Telegram; no password or key is ever entered here."
            ),
        )
    }

    /// `/id` — the Telegram id (no kernel).
    fn ident(chat_id: i64, update: &BotUpdate) -> BotAction {
        let name = update.username.as_deref().unwrap_or("—");
        Self::text(
            chat_id,
            format!("Your Telegram id: {}\nusername: @{}", update.from_id, name),
        )
    }
}

/// Number formatting with thousands separators (locale-neutral).
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

// ── tests — all offline with FakePort ─────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::port::{BotEconomy, BotError, BotMe};
    use crate::sovereign;
    use async_trait::async_trait;
    use std::sync::Mutex;

    /// The scripted port — zero network.
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
        assert!(start[0].text.contains("MAIRON"));
        // value proposition + market button
        assert!(start[0].text.contains("AI in 2 minutes"));
        assert!(start[0].text.contains("20% instant commission"));
        assert!(start[0].links.iter().any(|l| l.url.ends_with("/market")));
        let help = core.handle(&update("help"), NOW + 1, &port).await;
        assert!(help[0].text.contains("/balance"));
        assert!(help[0].text.contains("/mairon"), "help lists the sovereign suite");
    }

    #[tokio::test]
    async fn balance_uses_session_path_and_reports_honestly() {
        let mut core = BotCore::new(BotPolicy::default());
        let port = FakePort::ok();
        let out = core.handle(&update("balance"), NOW, &port).await;
        assert!(out[0].text.contains("500"));
        assert!(out[0].text.contains("250"));
        // kernel down/identity disabled → an honest message, not a fake number
        let mut core2 = BotCore::new(BotPolicy::default());
        let broken = FakePort::broken();
        let out2 = core2.handle(&update("balance"), NOW, &broken).await;
        assert!(out2[0].text.contains("disabled"), "honest identity-disabled message");
    }

    #[tokio::test]
    async fn economy_formats_thousands_and_handles_failure() {
        let mut core = BotCore::new(BotPolicy::default());
        let port = FakePort::ok();
        let out = core.handle(&update("economy"), NOW, &port).await;
        assert!(out[0].text.contains("1,234,567"), "thousands separator");
        assert!(out[0].text.contains("Article 6"));
        let mut core2 = BotCore::new(BotPolicy::default());
        let out2 = core2.handle(&update("economy"), NOW, &FakePort::broken()).await;
        assert!(out2[0].text.contains("not reachable"));
    }

    #[tokio::test]
    async fn rate_limit_warns_once_then_goes_silent() {
        let mut core = BotCore::new(BotPolicy { rate_max_per_window: 3, ..BotPolicy::default() });
        let port = FakePort::ok();
        // messages 1..3 allowed
        for i in 0..3 {
            let out = core.handle(&update("id"), NOW + i, &port).await;
            assert_eq!(out.len(), 1, "msg {i} must pass");
        }
        // the fourth — one warning
        let warn = core.handle(&update("id"), NOW + 4, &port).await;
        assert_eq!(warn.len(), 1);
        assert!(warn[0].text.contains("Easy there"));
        // the fifth and sixth — silent
        assert_eq!(core.handle(&update("id"), NOW + 5, &port).await, [] as [BotAction; 0]);
        assert_eq!(core.handle(&update("id"), NOW + 6, &port).await, [] as [BotAction; 0]);
        // a new window → allowed again
        let again = core.handle(&update("id"), NOW + 61_000, &port).await;
        assert_eq!(again.len(), 1);
    }

    #[tokio::test]
    async fn unknown_command_and_plain_text_get_hints() {
        let mut core = BotCore::new(BotPolicy::default());
        let port = FakePort::ok();
        let unknown = core.handle(&update("nope"), NOW, &port).await;
        assert!(unknown[0].text.contains("did not recognize"));
        let plain = core
            .handle(
                &BotUpdate {
                    update_id: 2,
                    chat_id: 100,
                    from_id: 777,
                    username: None,
                    command: None,
                    args: Vec::new(),
                    text: "hello there".to_owned(),
                },
                NOW + 1,
                &port,
            )
            .await;
        // a plain text message → the full welcome with mini-app buttons (not a dry reply)
        assert!(plain[0].text.contains("Welcome to MAIRON"), "friendly welcome on plain text");
        assert!(plain[0].text.contains("20% instant commission"));
        assert!(!plain[0].links.is_empty(), "welcome carries miniapp buttons");
    }

    #[tokio::test]
    async fn long_outputs_are_truncated_at_char_boundary() {
        let mut core = BotCore::new(BotPolicy { reply_max_chars: 40, ..BotPolicy::default() });
        let port = FakePort::ok();
        let out = core.handle(&update("economy"), NOW, &port).await;
        assert!(out[0].text.chars().count() <= 40, "must truncate to policy cap");
        // character boundary — never splits a character in half
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
        assert!(out[0].text.contains("Your balance"));
    }

    #[tokio::test]
    async fn actions_never_carry_secrets() {
        // the token is not in the orchestrator at all — this test locks the future contract too
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

    /// A FAQ question as plain text → an instant zero-token reply; an unrelated message → the regular welcome.
    #[tokio::test]
    async fn plain_text_questions_get_instant_faq_answers() {
        let mut core = BotCore::new(BotPolicy::default());
        // a text message (no command) with the escrow keyword → the instant canned reply
        let mut u = update("");
        u.command = None;
        u.text = "how do escrow-protected purchases work?".to_owned();
        let out = core.handle(&u, NOW, &FakePort::ok()).await;
        assert_eq!(out.len(), 1);
        assert!(out[0].text.contains("escrow"), "instant FAQ: {}", out[0].text);
        assert!(out[0].text.contains("zero tokens"), "the FAQ wrapper is English");
        // an unrelated message → the regular welcome path (not FAQ)
        let mut u2 = update("");
        u2.command = None;
        u2.text = "hello, how are you?".to_owned();
        let out2 = core.handle(&u2, NOW, &FakePort::ok()).await;
        assert_eq!(out2.len(), 1);
        assert!(!out2[0].text.contains("escrow"), "an unrelated message must not hit the FAQ");
        // the FAQ reply stays under the Telegram truncation cap
        assert!(out[0].text.chars().count() <= 4096);
    }

    // ── the MAIRON sovereign suite through the router ────────────────────────

    /// Every sovereign command routes through `handle` with its formatted payload.
    #[tokio::test]
    async fn sovereign_commands_route_through_the_core() {
        let mut core = BotCore::new(BotPolicy::default());
        let port = FakePort::ok();
        let expectations: [(&str, &str); 6] = [
            ("mairon", "IMPERIUM"),
            ("trinity", "150"),
            ("quant", "binance-spot"),
            ("treasury", "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913"),
            ("warroom", "24 hourly causal ticks"),
            ("hud", "180"),
        ];
        for (i, (cmd, marker)) in expectations.iter().enumerate() {
            let out = core.handle(&update(cmd), NOW + i as u64, &port).await;
            assert_eq!(out.len(), 1, "{cmd} replies exactly once");
            assert!(
                out[0].text.contains(marker),
                "{cmd} must carry its sovereign marker ({marker}): {}",
                out[0].text
            );
        }
    }

    /// `/mairon@Bot` suffix routing works for the sovereign suite too.
    #[tokio::test]
    async fn sovereign_commands_support_bot_suffix() {
        let mut core = BotCore::new(BotPolicy {
            bot_username: Some("MaironBot".to_owned()),
            ..BotPolicy::default()
        });
        let port = FakePort::ok();
        let mut u = update("treasury");
        u.command = Some("treasury@maironbot".to_owned());
        let out = core.handle(&u, NOW, &port).await;
        assert!(out[0].text.contains("Base-8453"));
    }

    /// Sovereign replies never leak secrets and always fit the cap.
    #[tokio::test]
    async fn sovereign_replies_are_clean_and_clipped() {
        let mut core = BotCore::new(BotPolicy::default());
        let port = FakePort::ok();
        for (i, cmd) in sovereign::KEYLESS_VENUES.iter().enumerate().take(3) {
            let _ = cmd; // venue ids are not commands; iterate sovereign commands instead
            let cmd = ["mairon", "trinity", "quant", "treasury", "warroom", "hud"][i];
            let out = core.handle(&update(cmd), NOW + i as u64, &port).await;
            assert_eq!(out.len(), 1);
            assert!(!out[0].text.contains("ABC-TOKEN"), "no secrets in {cmd}");
            assert!(out[0].text.chars().count() <= 4096, "{cmd} fits the cap");
        }
        // and all six explicitly
        for (i, cmd) in ["trinity", "quant", "treasury", "warroom", "hud", "mairon"]
            .iter()
            .enumerate()
        {
            let out = core.handle(&update(cmd), NOW + 100 + i as u64, &port).await;
            assert_eq!(out.len(), 1);
            assert!(!out[0].text.contains("ABC-TOKEN"));
        }
    }

    /// The rate limit also gates the sovereign suite (no bypass).
    #[tokio::test]
    async fn sovereign_commands_respect_the_rate_limit() {
        let mut core = BotCore::new(BotPolicy { rate_max_per_window: 2, ..BotPolicy::default() });
        let port = FakePort::ok();
        assert_eq!(core.handle(&update("mairon"), NOW, &port).await.len(), 1);
        assert_eq!(core.handle(&update("trinity"), NOW + 1, &port).await.len(), 1);
        let warn = core.handle(&update("quant"), NOW + 2, &port).await;
        assert_eq!(warn.len(), 1);
        assert!(warn[0].text.contains("Easy there"), "rate limit applies to sovereign cmds");
    }
}
