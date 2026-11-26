//! `agent_reach` — the fleet's zero-cost web connector + the guardian security handler.
//!
//! This module's ports:
//! - [`Panniantong/Agent-Reach`] (`agent_reach/channels/web.py`) — the Jina Reader
//!   reader and keyless Exa hosted MCP search, with antibot challenge-wall detection
//!   over the first 4,096 bytes; the `scout` reads the internet with these backends
//!   at zero API cost.
//! - [`androoAGI/starnet`] (`sidecar/subagents.js` + `capability/capGate.js`)
//!   — the subagent stall watcher (`StallWatcher`: 450 seconds of silence outside tools /
//!   1,200 seconds inside tools) and uniform capability attenuation for the
//!   child (`attenuate`: child = parent ∩ subset, never a union).
//!
//! And the guardian security/verification handler ([`GuardianSecurityHandler`]) wires onto these
//! same two pieces — with this, the Kian fleet is **7/7 active**.
//!
//! [`Panniantong/Agent-Reach`]: https://github.com/Panniantong/Agent-Reach
//! [`androoAGI/starnet`]: https://github.com/androoAGI/starnet
#![forbid(unsafe_code)]

use serde_json::Value;

// ── 1) Zero-cost backends (port of Agent-Reach) ────────────────────────────────

/// The Jina reader prefix — "the cleanest web text" without a key.
pub const JINA_READER_PREFIX: &str = "https://r.jina.ai/";

/// Exa hosted MCP — free semantic search/page reading (JSON-RPC 2.0).
pub const EXA_MCP_URL: &str = "https://mcp.exa.ai/mcp";

/// `web_fetch_exa` character cap — identical to `web.py::_exa_read`.
pub const EXA_MAX_FETCH_CHARS: u64 = 20_000;

/// Antibot scan bytes — identical to `_ANTIBOT_SCAN_BYTES = 4096`.
pub const ANTIBOT_SCAN_BYTES: usize = 4_096;

/// Jina Reader body cap — identical to `_MAX_RESPONSE_BYTES = 5MB`.
pub const JINA_MAX_RESPONSE_BYTES: usize = 5 * 1024 * 1024;

/// Builds the Jina Reader address for any public URL — `https://r.jina.ai/{url}`.
#[must_use]
pub fn jina_reader_url(target: &str) -> String {
    format!("{JINA_READER_PREFIX}{target}")
}

/// The Exa semantic-search JSON-RPC payload (`web_search_exa`) — identical to `_exa_search`:
/// arguments `query` + `objective` + `numResults`.
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

/// The Exa page-read JSON-RPC payload (`web_fetch_exa`) — identical to `_exa_read`:
/// a single-element `urls` argument + `maxCharacters: 20000`.
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

/// Antibot challenge wall? — port of `_is_antibot_page` + the Kian contract extension.
///
/// Four challenge families (upstream port + Turnstile + the 403/503 wall):
/// 1. **Cloudflare** — "Just a moment…", "Attention Required! | Cloudflare",
///    `cf-chl` and the `/cdn-cgi/challenge-platform/` path.
/// 2. **Turnstile** — the `challenges.cloudflare.com/turnstile` challenge widget.
/// 3. **Captcha** — "captcha", the Jina "requiring captcha" warning, "prove your
///    humanity", "performing/checking security verification".
/// 4. **HTTP status wall** — "403 Forbidden" / "503 Service Unavailable".
///
/// Only the body's first `ANTIBOT_SCAN_BYTES` bytes are scanned (identical to upstream) —
/// challenges always sit at the head of the response. The UTF-8 character boundary is preserved.
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

// ── 2) Subagent stall watcher (port of starnet/subagents.js) ────────────────────

/// Outside-tool silence cap — `DEFAULT_STALL_MS = 450000` (450 seconds).
pub const DEFAULT_IDLE_TIMEOUT_SECS: u64 = 450;

/// Inside-tool silence cap — `DEFAULT_IN_TOOL_STALL_MS = 1200000` (1,200 seconds);
/// a real build/test legitimately stays silent for a long time.
pub const DEFAULT_HARD_WALL_SECS: u64 = 1_200;

/// The watcher's decision for one subagent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StallDecision {
    /// Alive — progress noise (tokens/tools/heartbeat) is seen, or it is under the cap.
    Alive,
    /// Stalled — silence passed the cap; the host marks the run "stale" and
    /// cancels its children in a cascade (identical to starnet's `checkStalls`).
    Stalled {
        /// Seconds of silence at verdict time.
        quiet_secs: u64,
        /// Was the silence "inside a tool" (the hard cap instead of the idle cap)?
        in_tool: bool,
    },
}

/// Stall watcher — port of starnet's `checkStalls` policy with an injected clock
/// (deterministic and offline-testable; no hidden `Instant::now()`).
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
    /// Build with the caps; the hard cap never drops below the idle cap
    /// (identical to `inToolStallMs = Math.max(stallMs, knob)` at line 80).
    #[must_use]
    pub fn new(idle_timeout_secs: u64, hard_wall_secs: u64) -> Self {
        Self {
            idle_timeout_secs,
            hard_wall_secs: hard_wall_secs.max(idle_timeout_secs),
        }
    }

    /// The caps (idle, hard) — for the guardian's report.
    #[must_use]
    pub const fn limits(&self) -> (u64, u64) {
        (self.idle_timeout_secs, self.hard_wall_secs)
    }

    /// Verdict: the `quiet_secs` silence seconds are compared against the matching cap —
    /// inside tools the hard cap, outside tools the idle cap (identical to `checkStalls`).
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

// ── 3) Uniform capability attenuation (port of starnet/capGate.js) ─────────────────────

/// Uniform attenuation — child capabilities = parent ∩ allowed subset.
///
/// The capGate rule: "never a union; uniform attenuation makes the room layout a hard
/// ceiling" — a child never gains a tool the parent lacked, and the
/// child-of-child chain only shrinks. Output order = parent order; no duplicates.
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

// ── 4) Guardian security/verification handler (7/7 fleet activation) ─────────────────

/// The guardian's tool-level verification verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceVerdict {
    /// Clean level — the content is real.
    Clean,
    /// Challenge wall — antibot; the tool must fall through to the next backend (Exa),
    /// never fake content as an answer (the honesty principle).
    ChallengeWall,
}

/// The guardian security/verification handler — the wiring point of the `guardian` role.
///
/// The guardian's three duties, all deterministic and tested:
/// 1. Tool-output-level verification of the web (an antibot challenge → `ChallengeWall`);
/// 2. Subagent stall verdict via the starnet watcher;
/// 3. Child capability issuance with uniform attenuation (no elevation is possible).
#[derive(Debug, Clone, Copy, Default)]
pub struct GuardianSecurityHandler {
    _private: (),
}

impl GuardianSecurityHandler {
    /// Read-page-level verification — the guardian rejects challenge-wall fake content
    /// so the scout never sells a Cloudflare challenge as an "answer".
    #[must_use]
    pub fn verify_surface(&self, title: &str, body: &str) -> SurfaceVerdict {
        if is_antibot_challenge_page(title, body) {
            SurfaceVerdict::ChallengeWall
        } else {
            SurfaceVerdict::Clean
        }
    }

    /// Subagent stall verdict — the same [`SubagentStallWatchdog::evaluate`].
    #[must_use]
    pub fn evaluate_liveness(
        &self,
        watchdog: &SubagentStallWatchdog,
        quiet_secs: u64,
        in_tool: bool,
    ) -> StallDecision {
        watchdog.evaluate(quiet_secs, in_tool)
    }

    /// Child capability issuance — always attenuation, never elevation.
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

    // ── backends ──

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
        // The Attention Required wall + the challenge-platform path
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
        // The Jina warning
        assert!(is_antibot_challenge_page(
            "target",
            "warning: target page requires captcha\nMarkdown Content:"
        ));
        // The 403/503 wall
        assert!(is_antibot_challenge_page("Error", "<h1>403 Forbidden</h1>"));
        assert!(is_antibot_challenge_page("Error", "<h1>503 Service Unavailable</h1>"));
        // The humanity proof
        assert!(is_antibot_challenge_page("Gate", "Prove your humanity to continue"));
    }

    #[test]
    fn antibot_lets_clean_content_through() {
        assert!(!is_antibot_challenge_page(
            "KIAN Documentation",
            "# KIAN\n\n\u{645}\u{62d}\u{62a}\u{648}\u{627}\u{6cc} \u{62a}\u{645}\u{6cc}\u{632} \u{62f}\u{631}\u{628}\u{627}\u{631}\u{647}\u{654} \u{637}\u{631}\u{627}\u{62d}\u{6cc} \u{628}\u{627}\u{632}\u{627}\u{631} \u{648} \u{627}\u{633}\u{6a9}\u{631}\u{648}..."
        ));
        // In-content text about 403 is not yet a 403 page — but our
        // policy is conservative: any mention of "403 forbidden" in the first 4,096 bytes = a wall.
        // (fail-closed: better to fall through to Exa than to read fake content)
        assert!(is_antibot_challenge_page("x", "403 forbidden"));
    }

    #[test]
    fn antibot_scan_is_bounded_and_char_boundary_safe() {
        // A challenge after 4,096 bytes — not seen (the response head is what matters)
        let padding = "a".repeat(ANTIBOT_SCAN_BYTES + 64);
        assert!(!is_antibot_challenge_page(
            "clean",
            &format!("{padding}just a moment")
        ));
        // Multi-byte content inside the window — matching still works
        let fa = "\u{633}\u{644}\u{627}\u{645} ".repeat(100); // ≈ 900 bytes of multi-byte content
        assert!(is_antibot_challenge_page("x", &format!("{fa}just a moment")));
        // The 4,096 boundary mid-4-byte-character: the cut retreats to a character boundary —
        // no panic, no partial match outside the window
        let prefix = "a".repeat(ANTIBOT_SCAN_BYTES - 3);
        let body = format!("{prefix}🦊just a moment");
        assert!(!is_antibot_challenge_page("clean", &body));
    }

    // ── stall watcher ──

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
        // Inside a tool the hard cap is 1,200 — 450 seconds of silence inside a build is legitimate.
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

    // ── capability attenuation ──

    #[test]
    fn attenuate_is_intersection_never_union() {
        let parent = vec!["web.read".to_owned(), "db.write".to_owned(), "shell.exec".to_owned()];
        let subset = vec!["web.read".to_owned(), "db.admin".to_owned()];
        let child = attenuate_capabilities(&parent, &subset);
        assert_eq!(child, vec!["web.read".to_owned()], "db.admin must never leak in");
        // Parent order is preserved even if the subset is scrambled.
        let subset = vec!["shell.exec".to_owned(), "web.read".to_owned()];
        let child = attenuate_capabilities(&parent, &subset);
        assert_eq!(child, vec!["web.read".to_owned(), "shell.exec".to_owned()]);
        // An empty subset → nothing.
        assert_eq!(attenuate_capabilities(&parent, &[]), [] as [std::string::String; 0]);
    }

    #[test]
    fn attenuate_is_monotonic_down_a_spawn_chain() {
        let root = vec!["a".to_owned(), "b".to_owned(), "c".to_owned(), "d".to_owned()];
        let child = attenuate_capabilities(&root, &["a".to_owned(), "b".to_owned(), "c".to_owned()]);
        let grandchild = attenuate_capabilities(&child, &["b".to_owned(), "c".to_owned(), "e".to_owned()]);
        assert_eq!(grandchild, vec!["b".to_owned(), "c".to_owned()]);
        // Every child is a subset of its parent — no chain ever grows.
        for cap in &grandchild {
            assert!(child.contains(cap));
            assert!(root.contains(cap));
        }
        // Duplicates in the subset do not create duplicates in the output.
        let dup = attenuate_capabilities(&root, &["a".to_owned(), "a".to_owned()]);
        assert_eq!(dup, vec!["a".to_owned()]);
    }

    // ── guardian handler ──

    #[test]
    fn guardian_handler_wires_all_three_duties() {
        let guardian = GuardianSecurityHandler::default();
        let watchdog = SubagentStallWatchdog::default();

        // 1) Challenge level → reject; clean level → accept.
        assert_eq!(
            guardian.verify_surface("Just a moment...", "<title>Just a moment...</title>"),
            SurfaceVerdict::ChallengeWall
        );
        assert_eq!(
            guardian.verify_surface("KIAN Market", "# products list"),
            SurfaceVerdict::Clean
        );

        // 2) Stall verdict — the circuit breaker trips at the cap.
        assert_eq!(
            guardian.evaluate_liveness(&watchdog, 900, true),
            StallDecision::Alive
        );
        assert!(matches!(
            guardian.evaluate_liveness(&watchdog, 1_200, true),
            StallDecision::Stalled { .. }
        ));

        // 3) Child issuance — attenuation only.
        let parent = vec!["shield.audit".to_owned(), "ledger.write".to_owned()];
        assert_eq!(
            guardian.issue_child_capabilities(&parent, &["shield.audit".to_owned()]),
            vec!["shield.audit".to_owned()]
        );
    }
}
