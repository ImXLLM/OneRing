//! `agent_reach` — اتصال‌دهندهٔ وبِ صفر-هزینهٔ ناوگان + هندلر امنیت نگهبان.
//!
//! پورت‌های این ماژول:
//! - [`Panniantong/Agent-Reach`] (`agent_reach/channels/web.py`) — خوانندهٔ
//!   Jina Reader و جست‌وجوی Exa hosted MCP بدون کلید، با تشخیص دیوار چالش
//!   antِibot روی ۴٬۰۹۶ بایت اول؛ کاوشگر (`scout`) با این بک‌اندها اینترنت
//!   را بدون هیچ هزینهٔ API می‌خواند.
//! - [`androoAGI/starnet`] (`sidecar/subagents.js` + `capability/capGate.js`)
//!   — تماشاگر توقف ساب‌ایجنت (`StallWatcher`: سکوت ۴۵۰ ثانیه بیرون ابزار /
//!   ۱٬۲۰۰ ثانیه درون ابزار) و تضعیف یکنواخت قابلیت‌ها برای فرزند
//!   (`attenuate`: فرزند = والد ∩ زیرمجموعه، هرگز اجتماع).
//!
//! و هندلر امنیت/راستی‌آزمایی نگهبان ([`GuardianSecurityHandler`]) روی همین
//! دو قطعه سیم می‌شود — ناوگان کیان با این، **۷/۷ فعال** است.
//!
//! [`Panniantong/Agent-Reach`]: https://github.com/Panniantong/Agent-Reach
//! [`androoAGI/starnet`]: https://github.com/androoAGI/starnet
#![forbid(unsafe_code)]

use serde_json::Value;

// ── ۱) بک‌اندهای صفر-هزینه (پورت Agent-Reach) ────────────────────────────────

/// پیشوند خوانندهٔ Jina — «تمیزترین متن وب» بدون کلید.
pub const JINA_READER_PREFIX: &str = "https://r.jina.ai/";

/// Exa hosted MCP — جست‌وجوی معنایی/خواندن صفحهٔ رایگان (JSON-RPC 2.0).
pub const EXA_MCP_URL: &str = "https://mcp.exa.ai/mcp";

/// سقف نویسه‌های `web_fetch_exa` — عین `web.py::_exa_read`.
pub const EXA_MAX_FETCH_CHARS: u64 = 20_000;

/// بایت‌های اسکن antِibot — عین `_ANTIBOT_SCAN_BYTES = 4096`.
pub const ANTIBOT_SCAN_BYTES: usize = 4_096;

/// سقف بدنهٔ Jina Reader — عین `_MAX_RESPONSE_BYTES = 5MB`.
pub const JINA_MAX_RESPONSE_BYTES: usize = 5 * 1024 * 1024;

/// ساخت نشانی Jina Reader برای هر URL عمومی — `https://r.jina.ai/{url}`.
#[must_use]
pub fn jina_reader_url(target: &str) -> String {
    format!("{JINA_READER_PREFIX}{target}")
}

/// بار JSON-RPC جست‌وجوی معنایی Exa (`web_search_exa`) — عین `_exa_search`:
/// آرگومان‌ها `query` + `objective` + `numResults`.
#[must_use]
pub fn exa_search_payload(query: &str, num_results: usize) -> Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "web_search_exa",
            "arguments": {
                "query": query,
                "objective": query,
                "numResults": num_results
            }
        }
    })
}

/// بار JSON-RPC خواندن صفحه با Exa (`web_fetch_exa`) — عین `_exa_read`:
/// آرگومان‌ها `urls` تک‌عضوی + `maxCharacters: 20000`.
#[must_use]
pub fn exa_fetch_payload(url: &str) -> Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "web_fetch_exa",
            "arguments": {
                "urls": [url],
                "maxCharacters": EXA_MAX_FETCH_CHARS
            }
        }
    })
}

/// دیوار چالش antِibot؟ — پورت `_is_antibot_page` + گسترش قرارداد کیان.
///
/// چهار خانوادهٔ چالش (پورت upstream + Turnstile + دیوار ۴۰۳/۵۰۳):
/// 1. **Cloudflare** — «Just a moment…»، «Attention Required! | Cloudflare»،
///    `cf-chl` و مسیر `/cdn-cgi/challenge-platform/`.
/// 2. **Turnstile** — ویجت چالش `challenges.cloudflare.com/turnstile`.
/// 3. **Captcha** — «captcha»، هشدار Jina «requiring captcha»، «prove your
///    humanity»، «performing/checking security verification».
/// 4. **دیوار وضعیت HTTP** — «403 Forbidden» / «503 Service Unavailable».
///
/// فقط `ANTIBOT_SCAN_BYTES` بایت اول بدنه اسکن می‌شود (عین upstream) —
/// چالش‌ها همیشه سرِ پاسخ‌اند. مرز نویسهٔ UTF-8 حفظ می‌شود.
#[must_use]
pub fn is_antibot_challenge_page(title: &str, body: &str) -> bool {
    let mut end = ANTIBOT_SCAN_BYTES.min(body.len());
    while end > 0 && !body.is_char_boundary(end) {
        end -= 1;
    }
    let hay = format!("{}\n{}", title.to_lowercase(), body[..end].to_lowercase());

    let cloudflare = hay.contains("just a moment")
        || hay.contains("attention required! | cloudflare")
        || hay.contains("/cdn-cgi/challenge-platform/")
        || hay.contains("cf-chl");
    let turnstile = hay.contains("turnstile");
    let captcha = hay.contains("captcha")
        || hay.contains("prove your humanity")
        || (hay.contains("warning:") && hay.contains("requiring captcha"));
    let verification_wall = hay.contains("performing security verification")
        || hay.contains("checking your browser")
        || hay.contains("security verification");
    let http_wall = hay.contains("403 forbidden") || hay.contains("503 service unavailable");

    cloudflare || turnstile || captcha || verification_wall || http_wall
}

// ── ۲) تماشاگر توقف ساب‌ایجنت (پورت starnet/subagents.js) ────────────────────

/// سقف سکوت بیرون ابزار — `DEFAULT_STALL_MS = 450000` (۴۵۰ ثانیه).
pub const DEFAULT_IDLE_TIMEOUT_SECS: u64 = 450;

/// سقف سکوت درون ابزار — `DEFAULT_IN_TOOL_STALL_MS = 1200000` (۱٬۲۰۰ ثانیه)؛
/// یک build/تست واقعی سکوت طولانیِ مشروع دارد.
pub const DEFAULT_HARD_WALL_SECS: u64 = 1_200;

/// تصمیم تماشاگر برای یک ساب‌ایجنت.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StallDecision {
    /// زنده — نویزِ پیشرفت (توکن/ابزار/heartbeat) دیده می‌شود یا زیر سقف است.
    Alive,
    /// متوقف — سکوت از سقف گذشت؛ host ران را «stale» علامت می‌زند و
    /// فرزندانش به‌صورت آبشاری لغو می‌شوند (عین `checkStalls` استارنت).
    Stalled {
        /// ثانیه‌های سکوت در لحظهٔ حکم.
        quiet_secs: u64,
        /// آیا سکوت «درون ابزار» بود (سقف سخت به‌جای سقف idle)؟
        in_tool: bool,
    },
}

/// تماشاگر توقف — پورت سیاست `checkStalls` استارنت با ساعت تزریقی
/// (قطعی و آفلاین‌تست‌پذیر؛ هیچ `Instant::now()` پنهانی نیست).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SubagentStallWatchdog {
    idle_timeout_secs: u64,
    hard_wall_secs: u64,
}

impl Default for SubagentStallWatchdog {
    fn default() -> Self {
        Self::new(DEFAULT_IDLE_TIMEOUT_SECS, DEFAULT_HARD_WALL_SECS)
    }
}

impl SubagentStallWatchdog {
    /// ساخت با سقف‌ها؛ سقف سخت هرگز زیر سقف idle نمی‌رود
    /// (عین `inToolStallMs = Math.max(stallMs, knob)` سطر ۸۰).
    #[must_use]
    pub fn new(idle_timeout_secs: u64, hard_wall_secs: u64) -> Self {
        Self {
            idle_timeout_secs,
            hard_wall_secs: hard_wall_secs.max(idle_timeout_secs),
        }
    }

    /// سقف‌ها (idle, hard) — برای گزارش نگهبان.
    #[must_use]
    pub const fn limits(&self) -> (u64, u64) {
        (self.idle_timeout_secs, self.hard_wall_secs)
    }

    /// حکم: ثانیه‌های سکوت `quiet_secs` با سقفِ متناسب مقایسه می‌شود —
    /// درون ابزار سقف سخت، بیرون ابزار سقف idle (عین `checkStalls`).
    #[must_use]
    pub fn evaluate(&self, quiet_secs: u64, in_tool: bool) -> StallDecision {
        let limit = if in_tool {
            self.hard_wall_secs
        } else {
            self.idle_timeout_secs
        };
        if quiet_secs >= limit {
            StallDecision::Stalled { quiet_secs, in_tool }
        } else {
            StallDecision::Alive
        }
    }
}

// ── ۳) تضعیف یکنواخت قابلیت‌ها (پورت starnet/capGate.js) ─────────────────────

/// تضعیف یکنواخت — قابلیت‌های فرزند = والد ∩ زیرمجموعهٔ مجاز.
///
/// قانون capGate: «هرگز اجتماع؛ تضعیف یکنواخت، چیدمان اتاق را سقفِ سخت
/// اقتدار می‌کند» — فرزند هرگز ابزاری که والد نداشت نمی‌گیرد و زنجیرهٔ
/// فرزند-از-فرزند فقط کوچک‌تر می‌شود. ترتیب خروجی = ترتیب والد؛ تکرار ندارد.
#[must_use]
pub fn attenuate_capabilities(parent: &[String], allowed_subset: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::with_capacity(parent.len());
    for cap in parent {
        if allowed_subset.contains(cap) && !out.contains(cap) {
            out.push(cap.clone());
        }
    }
    out
}

// ── ۴) هندلر امنیت/راستی‌آزمایی نگهبان (فعال‌سازی ناوگان ۷/۷) ─────────────────

/// حکم راستی‌آزمایی سطح ابزار توسط نگهبان.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceVerdict {
    /// سطح تمیز — محتوا واقعی است.
    Clean,
    /// دیوار چالش — antِibot؛ ابزار باید به بک‌اند بعدی (Exa) بیفتد،
    /// هرگز محتوا را به‌جای پاسخ جعل نکند (اصل صادق‌بودن).
    ChallengeWall,
}

/// هندلر امنیت/راستی‌آزمایی نگهبان — نقطهٔ سیم‌کشی نقش `guardian`.
///
/// سه وظیفهٔ نگهبان، همه قطعی و تست‌شده:
/// 1. راستی‌آزمایی سطحِ خروجی ابزار وب (چالش antِibot → `ChallengeWall`)؛
/// 2. حکم توقف ساب‌ایجنت با تماشاگر استارنت؛
/// 3. صدور قابلیت فرزند با تضعیف یکنواخت (هیچ ارتقایی ممکن نیست).
#[derive(Debug, Clone, Copy, Default)]
pub struct GuardianSecurityHandler {
    _private: (),
}

impl GuardianSecurityHandler {
    /// راستی‌آزمایی سطح صفحهٔ خوانده‌شده — نگهبان محتوای جعلیِ دیوار چالش را
    /// رد می‌کند تا کاوشگر به‌جای «پاسخ»، چالش Cloudflare را فروخته نکند.
    #[must_use]
    pub fn verify_surface(&self, title: &str, body: &str) -> SurfaceVerdict {
        if is_antibot_challenge_page(title, body) {
            SurfaceVerdict::ChallengeWall
        } else {
            SurfaceVerdict::Clean
        }
    }

    /// حکم توقف ساب‌ایجنت — همان [`SubagentStallWatchdog::evaluate`].
    #[must_use]
    pub fn evaluate_liveness(
        &self,
        watchdog: &SubagentStallWatchdog,
        quiet_secs: u64,
        in_tool: bool,
    ) -> StallDecision {
        watchdog.evaluate(quiet_secs, in_tool)
    }

    /// صدور قابلیت برای فرزند — همیشه تضعیف، هرگز ارتقا.
    #[must_use]
    pub fn issue_child_capabilities(
        &self,
        parent: &[String],
        allowed_subset: &[String],
    ) -> Vec<String> {
        attenuate_capabilities(parent, allowed_subset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── بک‌اندها ──

    #[test]
    fn jina_reader_url_builds_correct_prefix() {
        assert_eq!(
            jina_reader_url("https://example.com/post"),
            "https://r.jina.ai/https://example.com/post"
        );
        assert!(jina_reader_url("x").starts_with(JINA_READER_PREFIX));
    }

    #[test]
    fn exa_search_payload_matches_upstream_shape() {
        let p = exa_search_payload("yt-dlp github repository", 5);
        assert_eq!(p["jsonrpc"], "2.0");
        assert_eq!(p["id"], 1);
        assert_eq!(p["method"], "tools/call");
        assert_eq!(p["params"]["name"], "web_search_exa");
        assert_eq!(p["params"]["arguments"]["query"], "yt-dlp github repository");
        assert_eq!(p["params"]["arguments"]["objective"], "yt-dlp github repository");
        assert_eq!(p["params"]["arguments"]["numResults"], 5);
    }

    #[test]
    fn exa_fetch_payload_matches_upstream_shape() {
        let p = exa_fetch_payload("https://example.com");
        assert_eq!(p["params"]["name"], "web_fetch_exa");
        assert_eq!(p["params"]["arguments"]["urls"][0], "https://example.com");
        assert_eq!(p["params"]["arguments"]["maxCharacters"], 20_000);
    }

    #[test]
    fn antibot_detects_all_four_challenge_families() {
        // Cloudflare «Just a moment…»
        assert!(is_antibot_challenge_page(
            "Just a moment...",
            "<html><head><title>Just a moment...</title></head></html>"
        ));
        // دیوار Attention Required + مسیر challenge-platform
        assert!(is_antibot_challenge_page(
            "Attention Required! | Cloudflare",
            "ray id: 8f0b · /cdn-cgi/challenge-platform/h/b/orchestrate"
        ));
        // Turnstile
        assert!(is_antibot_challenge_page(
            "Login",
            "<script src=\"https://challenges.cloudflare.com/turnstile/v0/api.js\"></script>"
        ));
        // Captcha
        assert!(is_antibot_challenge_page("Security Check", "Please complete the CAPTCHA below"));
        // هشدار Jina
        assert!(is_antibot_challenge_page(
            "target",
            "warning: target page requires captcha\nMarkdown Content:"
        ));
        // دیوار ۴۰۳/۵۰۳
        assert!(is_antibot_challenge_page("Error", "<h1>403 Forbidden</h1>"));
        assert!(is_antibot_challenge_page("Error", "<h1>503 Service Unavailable</h1>"));
        // اثبات انسانیت
        assert!(is_antibot_challenge_page("Gate", "Prove your humanity to continue"));
    }

    #[test]
    fn antibot_lets_clean_content_through() {
        assert!(!is_antibot_challenge_page(
            "KIAN Documentation",
            "# KIAN\n\nمحتوای تمیز دربارهٔ طراحی بازار و اسکرو..."
        ));
        // متن درون‌محتوایی دربارهٔ ۴۰۳ هنوز صفحهٔ ۴۰۳ نیست — اما سیاستِ ما
        // محافظه‌کار است: هر ذکر «403 forbidden» در ۴۰۹۶ بایت اول = دیوار.
        // (fail-closed: بهتر است به Exa بیفتد تا محتوای جعلی بخواند)
        assert!(is_antibot_challenge_page("x", "403 forbidden"));
    }

    #[test]
    fn antibot_scan_is_bounded_and_char_boundary_safe() {
        // چالش پس از ۴۰۹۶ بایت — دیده نمی‌شود (سر پاسخ است که مهم است)
        let padding = "a".repeat(ANTIBOT_SCAN_BYTES + 64);
        assert!(!is_antibot_challenge_page(
            "clean",
            &format!("{padding}just a moment")
        ));
        // محتوای چندبایتی داخل پنجره — تطبیق همچنان کار می‌کند
        let fa = "سلام ".repeat(100); // ≈ ۹۰۰ بایت چندبایتی
        assert!(is_antibot_challenge_page("x", &format!("{fa}just a moment")));
        // مرز ۴۰۹۶ وسط نویسهٔ ۴بایتی: برش به مرز نویسه پس‌نشینی می‌شود —
        // نه پانیک، نه تطبیقِ ناقص بیرون پنجره
        let prefix = "a".repeat(ANTIBOT_SCAN_BYTES - 3);
        let body = format!("{prefix}🦊just a moment");
        assert!(!is_antibot_challenge_page("clean", &body));
    }

    // ── تماشاگر توقف ──

    #[test]
    fn watchdog_defaults_match_starnet_constants() {
        let w = SubagentStallWatchdog::default();
        assert_eq!(w.limits(), (450, 1_200));
    }

    #[test]
    fn watchdog_alive_below_limit_stalled_at_limit() {
        let w = SubagentStallWatchdog::default();
        assert_eq!(w.evaluate(449, false), StallDecision::Alive);
        assert_eq!(
            w.evaluate(450, false),
            StallDecision::Stalled { quiet_secs: 450, in_tool: false }
        );
        // درون ابزار سقف سخت ۱٬۲۰۰ است — ۴۵۰ ثانیه سکوتِ درون build مشروع است.
        assert_eq!(w.evaluate(450, true), StallDecision::Alive);
        assert_eq!(w.evaluate(1_199, true), StallDecision::Alive);
        assert_eq!(
            w.evaluate(1_200, true),
            StallDecision::Stalled { quiet_secs: 1_200, in_tool: true }
        );
    }

    #[test]
    fn watchdog_hard_wall_never_below_idle() {
        let w = SubagentStallWatchdog::new(600, 300);
        assert_eq!(w.limits(), (600, 600), "hard wall clamps to idle floor");
    }

    // ── تضعیف قابلیت ──

    #[test]
    fn attenuate_is_intersection_never_union() {
        let parent = vec!["web.read".to_owned(), "db.write".to_owned(), "shell.exec".to_owned()];
        let subset = vec!["web.read".to_owned(), "db.admin".to_owned()];
        let child = attenuate_capabilities(&parent, &subset);
        assert_eq!(child, vec!["web.read".to_owned()], "db.admin must never leak in");
        // ترتیب والد حفظ می‌شود حتی اگر زیرمجموعه به‌هم‌ریخته باشد.
        let subset = vec!["shell.exec".to_owned(), "web.read".to_owned()];
        let child = attenuate_capabilities(&parent, &subset);
        assert_eq!(child, vec!["web.read".to_owned(), "shell.exec".to_owned()]);
        // زیرمجموعهٔ خالی → هیچ.
        assert!(attenuate_capabilities(&parent, &[]).is_empty());
    }

    #[test]
    fn attenuate_is_monotonic_down_a_spawn_chain() {
        let root = vec!["a".to_owned(), "b".to_owned(), "c".to_owned(), "d".to_owned()];
        let child = attenuate_capabilities(&root, &["a".to_owned(), "b".to_owned(), "c".to_owned()]);
        let grandchild = attenuate_capabilities(&child, &["b".to_owned(), "c".to_owned(), "e".to_owned()]);
        assert_eq!(grandchild, vec!["b".to_owned(), "c".to_owned()]);
        // هر فرزند زیرمجموعهٔ والد — هیچ زنجیره‌ای بزرگ نمی‌شود.
        for cap in &grandchild {
            assert!(child.contains(cap));
            assert!(root.contains(cap));
        }
        // تکرار در زیرمجموعه، تکرار در خروجی نمی‌سازد.
        let dup = attenuate_capabilities(&root, &["a".to_owned(), "a".to_owned()]);
        assert_eq!(dup, vec!["a".to_owned()]);
    }

    // ── هندلر نگهبان ──

    #[test]
    fn guardian_handler_wires_all_three_duties() {
        let guardian = GuardianSecurityHandler::default();
        let watchdog = SubagentStallWatchdog::default();

        // ۱) سطح چالش → رد؛ سطح تمیز → پذیرش.
        assert_eq!(
            guardian.verify_surface("Just a moment...", "<title>Just a moment...</title>"),
            SurfaceVerdict::ChallengeWall
        );
        assert_eq!(
            guardian.verify_surface("KIAN Market", "# products list"),
            SurfaceVerdict::Clean
        );

        // ۲) حکم توقف — مدارشکن در سقف.
        assert_eq!(
            guardian.evaluate_liveness(&watchdog, 900, true),
            StallDecision::Alive
        );
        assert!(matches!(
            guardian.evaluate_liveness(&watchdog, 1_200, true),
            StallDecision::Stalled { .. }
        ));

        // ۳) صدور فرزند — فقط تضعیف.
        let parent = vec!["shield.audit".to_owned(), "ledger.write".to_owned()];
        assert_eq!(
            guardian.issue_child_capabilities(&parent, &["shield.audit".to_owned()]),
            vec!["shield.audit".to_owned()]
        );
    }
}
