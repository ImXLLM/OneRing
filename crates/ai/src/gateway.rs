//! gateway — multi-lane pool + fallback + quality gate + redaction + attempt logging.
//!
//! Architecture ported from `brain.py` v1.2 of Hermes (`ShopifyHermes`) — `battle-tested`:
//! * **Flat pool**: every entry is one `(url, key, model)` combination; logical slots and
//!   Hermes-style providers are expanded by the "every model × every key" Cartesian product, and
//!   duplicate triples are dropped (order preserved).
//! * **`last_good` stickiness**: the last winner is tried first next time
//!   (`AtomicUsize` + `Relaxed`) — the exact `brain.py` scheme: `[last_good] + rest`.
//! * **`looks_broken` quality gate**: an empty/corrupt/garbled/wrong-language reply = failure
//!   for that lane; the first "broken but present" reply is stashed as least-bad and
//!   returned only when everything fails (chat) or falls through to local synthesis (builder).
//!
//! Contract: input is redacted here (before any lane sees it), a lane without complete
//! config (url+key+model) is excluded outright, a malformed packet per lane = that lane fails and
//! moves on to the next. No code path ever sends raw text or a key outward.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::redaction::{redact, RedactionFinding};

/// Logical slot names — fallback order.
pub const SLOT_NAMES: [&str; 3] = ["primary", "secondary", "tertiary"];

/// Output token cap for generating a complete HTML artifact in the product builder.
///
/// Live `OpenRouter` benchmark evidence (24 November 2026): reasoning-heavy models on
/// free lanes draw their `reasoning_tokens` share from this same cap; at 4096 either
/// the content comes back empty (`finish_reason=length` spent on thinking) or the document is cut
/// off midway. 16384 fits a complete document plus the CAST JSON block with safe headroom
/// and costs zero on free lanes.
pub const BUILDER_MAX_TOKENS: u32 = 16_384;

/// `OpenRouter` base — default for the `KIAN_AI_OPENROUTER_*` Hermes-style provider (no URL).
pub const OPENROUTER_BASE_URL: &str = "https://openrouter.ai/api/v1";

/// `Coarena` base — daily free credit, `OpenAI`-compatible (discovered in Hermes).
pub const COARENA_BASE_URL: &str = "https://coarena.ai/api/v1";

/// Default `9router` base — common proxy for `oc/*` lanes.
pub const ROUTER9_BASE_URL: &str = "https://9router-production-6f2225a3.up.railway.app/v1";

/// Configuration of one pool lane.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotConfig {
    /// Lane name — a logical slot name (`SLOT_NAMES`) or a Hermes-style provider
    /// (`coarena`/`openrouter`/`router9`) — reported on the wire as provider
    pub name: String,
    /// `OpenAI`-compatible URL base (`/chat/completions` is appended)
    pub url: String,
    /// API key — never logged or sent on the wire
    pub key: String,
    /// Model name
    pub model: String,
}

impl SlotConfig {
    /// A lane is enabled only with all three of url+key+model complete — half-configured = off.
    #[must_use]
    pub fn enabled(&self) -> bool {
        !self.url.is_empty() && !self.key.is_empty() && !self.model.is_empty()
    }
}

/// `brain.py` Cartesian product: "for every model, for every key" — one pool entry for
/// each combination. Empty items (whitespace / leading empty separator) are ignored.
///
/// Exact order: model outer, key inner — i.e. `mA×(k1,k2), mB×(k1,k2)`.
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

/// Pool insertion with duplicate `(url, key, model)` triple removal — first order kept.
fn push_unique(pool: &mut Vec<SlotConfig>, entry: SlotConfig) {
    if !pool
        .iter()
        .any(|x| x.url == entry.url && x.key == entry.key && x.model == entry.model)
    {
        pool.push(entry);
    }
}

/// Gateway configuration.
///
/// `slots` is the **executable flat pool** (the `pool` in `brain.py`) — after
/// Cartesian expansion, each member is one `(url, key, model)` combination. [`AiConfig::pool`]
/// exposes the same slice under a clearer name; `/ai/status` groups members by name
/// and reports the pool size separately.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiConfig {
    /// Flat pool in fallback order (logical slots ← Hermes-style providers)
    pub slots: Vec<SlotConfig>,
    /// Per-user daily token cap
    pub daily_cap_tokens: i64,
}

impl AiConfig {
    /// Flat pool slice — a clearer name for the same [`AiConfig::slots`].
    #[must_use]
    pub fn pool(&self) -> &[SlotConfig] {
        &self.slots
    }

    /// Read from the environment — fail-closed: without env the pool is empty.
    ///
    /// Logical slots are built from `KIAN_AI_{PRIMARY|SECONDARY|TERTIARY}_{URL,KEY,MODEL}`
    /// (or the single-gateway shortcuts); `_KEY` and `_MODEL` (and the shortcuts)
    /// may be **comma-separated lists** and are expanded by the `brain.py` Cartesian
    /// product. Hermes-style providers, when present, are appended
    /// to the end of the pool (Hermes latency order: coarena ← openrouter ← router9):
    /// * `KIAN_AI_COARENA_URL` (default [`COARENA_BASE_URL`]) + `KIAN_AI_COARENA_KEYS`
    ///   (or single-key `KIAN_AI_COARENA_KEY`) + `KIAN_AI_COARENA_MODELS`
    /// * `KIAN_AI_OPENROUTER_URL` (default [`OPENROUTER_BASE_URL`]) + `KIAN_AI_OPENROUTER_KEYS`
    ///   (or `KIAN_AI_OPENROUTER_KEY`) + `KIAN_AI_OPENROUTER_MODELS`
    /// * `KIAN_AI_ROUTER9_URL` (default [`ROUTER9_BASE_URL`]) + `KIAN_AI_ROUTER9_KEYS`
    ///   (or `KIAN_AI_ROUTER9_KEY`) + `KIAN_AI_ROUTER9_MODELS`
    ///
    /// Daily cap from `KIAN_AI_DAILY_TOKENS` (default 50,000).
    ///
    /// "Single-gateway" shortcuts for multi-model routers (e.g. 9router):
    /// * `KIAN_AI_BASE_URL` + `KIAN_AI_API_KEY` + `KIAN_AI_MODEL` → primary slot
    /// * `KIAN_AI_FALLBACK_MODEL` (+ optional `KIAN_AI_FALLBACK_URL` / `KIAN_AI_FALLBACK_KEY`)
    ///   → secondary slot on the same gateway.
    #[must_use]
    pub fn from_env() -> Self {
        Self::from_env_reader(|k| std::env::var(k).ok().filter(|v| !v.trim().is_empty()))
    }

    /// Injectable version of [`AiConfig::from_env`] — the env reader comes from outside so
    /// tests can prove the pool and shortcut order without mutating process-global variables.
    ///
    /// Priority per field: slot-specific variable ← single-gateway shortcut ← off (empty string).
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
        // Logical slots with raw values (possibly comma lists) — enabled ones are expanded
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
                // Complete slot — "every model × every key" Cartesian product into the flat pool
                for entry in expand_models_times_keys(&s.name, &s.url, &s.key, &s.model) {
                    push_unique(&mut pool, entry);
                }
            } else {
                // Incomplete/empty slot — kept in the pool itself with off flags so that
                // /ai/status still shows the same three honest rows as before;
                // execution only touches enabled members. No dedup —
                // the distinct primary/secondary/tertiary names must all stay present.
                pool.push(s);
            }
        }
        // Hermes-style providers — extra lanes at the end of the pool
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

/// One packet message — role + content (not redacted before the gateway; redaction happens here).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketMessage {
    /// `system|user|assistant`
    pub role: String,
    /// Text
    pub content: String,
}

/// Packet sent to a pool lane.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatPacket {
    /// Lane model
    pub model: String,
    /// Messages — user messages always post-redaction
    pub messages: Vec<PacketMessage>,
    /// Optional output token cap (e.g. `Some(4096)` for full HTML generation)
    pub max_tokens: Option<u32>,
}

/// Successful lane packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionPacket {
    /// Response text
    pub content: String,
    /// Prompt tokens
    pub prompt_tokens: i64,
    /// Completion tokens
    pub completion_tokens: i64,
}

/// One attempt failure — for the honest all-failed response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttemptFailure {
    /// Lane name (logical slot or Hermes-style provider)
    pub slot: String,
    /// Short detail — never includes request text or keys
    pub detail: String,
}

/// Gateway error — typed; raw upstream text never bubbles up.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GatewayError {
    /// No lane configured — 503 `AI_DISABLED`
    #[error("AI gateway disabled — no complete slot configured")]
    Disabled,
    /// All enabled lanes failed (and no acceptable least-bad existed) — 502 `AI_UPSTREAM`
    #[error("all AI slots failed")]
    AllSlotsFailed {
        /// Per-attempt failures in attempt order
        attempts: Vec<AttemptFailure>,
    },
}

/// Status of one lane group (logical slot or Hermes-style provider) for the wire — no keys.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotStatus {
    /// Group number in pool order (1-based)
    pub slot: u8,
    /// Name
    pub name: String,
    /// Group models — comma-separated list (first-seen order kept, no duplicates)
    pub model: String,
    /// Enabled? (at least one group member enabled)
    pub enabled: bool,
}

/// Successful chat result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatResult {
    /// Response text
    pub content: String,
    /// Winning lane
    pub provider: String,
    /// Winning model
    pub model: String,
    /// Prompt tokens
    pub prompt_tokens: i64,
    /// Completion tokens
    pub completion_tokens: i64,
    /// Total
    pub total_tokens: i64,
    /// Input redaction finding categories (no duplicates, first-seen order)
    pub redactions: Vec<String>,
}

/// Outcome of one completion — the error is a short secret-free string (for `AttemptFailure.detail`).
pub type CompletionOutcome = Result<CompletionPacket, String>;

/// Model client — one level above HTTP so tests can be scripted.
#[async_trait::async_trait]
pub trait ModelClient: Send + Sync {
    /// One completion — the error is a short secret-free string (for `AttemptFailure.detail`).
    ///
    /// # Errors
    /// Network/upstream/malformed-packet failure — as a descriptive string.
    async fn complete(&self, slot: &SlotConfig, packet: &ChatPacket) -> CompletionOutcome;
}

/// Counts Persian/Arabic characters (`\u{0600}..=\u{06FF}`).
#[must_use]
pub fn persian_char_count(text: &str) -> usize {
    text.chars().filter(|c| ('\u{0600}'..='\u{06FF}').contains(c)).count()
}

/// Strips fenced code blocks (```` ```...``` ````) — to gauge the language of the "actual text"
/// of the reply, not the code inside the block (HTML/CSS is naturally Latin-heavy).
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

/// Removes hex color tokens (`#RGB`/`#RRGGBB`/`#RRGGBBAA`) before the run scan —
/// real CSS colors of the artifact (e.g. `#ffffff`) are not language degeneration.
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
                i = j; // complete color token — removed
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// Are there `min` identical consecutive alphabetic characters?
///
/// A non-alphabetic character (whitespace, punctuation, digit, ``` ```` ```, `---`, `====`) breaks the run
/// and hex colors (`#000000`, `#ffffff`) are removed before the scan —
/// so Markdown separators and CSS stay intact while "hehehehe" or "aaaaaa" is caught.
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

/// Quality gate in the style of `brain.py::_looks_broken` — a broken reply = lane failure.
///
/// * Rule 1 (empty/too short): trimmed reply text < 5 characters — in builder mode
///   < 64 (a complete HTML artifact is never that short).
/// * Rule 2 (encoding corruption): presence of `U+FFFD`.
/// * Rule 3 (run degeneration): >= 5 identical consecutive alphabetic characters.
/// * Rule 4 (language mismatch):
///   * Plain chat: Persian prompt (>= 10 Persian/Arabic characters) but, after code blocks are stripped,
///     more than 60% of the reply's letters are ASCII Latin → wrong-language reply.
///   * Builder: the reply must carry an `<html` or `<!doctype html` tag (case-insensitive)
///     and, for a Persian prompt, at least 5 Persian characters in the reply —
///     HTML/CSS is naturally Latin-heavy and must not be rejected.
#[must_use]
pub fn looks_broken(prompt: &str, reply: &str, is_builder: bool) -> bool {
    // 1) Empty / too short
    let min_chars = if is_builder { 64 } else { 5 };
    if reply.trim().chars().count() < min_chars {
        return true;
    }
    // 2) Encoding corruption
    if reply.contains('\u{FFFD}') {
        return true;
    }
    // 3) Alphabetic character run degeneration
    if has_identical_alpha_run(reply, 5) {
        return true;
    }
    // 4) Language mismatch
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

/// AI gateway — redaction → flat pool with last-winner stickiness → quality gate →
/// least-bad stash → typed result.
pub struct AiGateway {
    /// Boot configuration — `slots` is the executable flat pool
    pub config: AiConfig,
    client: Arc<dyn ModelClient>,
    /// Index of the last winning lane (`brain.py::last_good`) — tried first next time
    last_good: AtomicUsize,
}

impl std::fmt::Debug for AiGateway {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // The client is deliberately not exposed — the config suffices and keys never appear in trace
        f.debug_struct("AiGateway").field("config", &self.config).finish_non_exhaustive()
    }
}

impl AiGateway {
    /// Build — the client is injected (production HTTP or a scripted test client).
    #[must_use]
    pub fn new(config: AiConfig, client: Arc<dyn ModelClient>) -> Self {
        Self { config, client, last_good: AtomicUsize::new(0) }
    }

    /// Gateway enabled? At least one pool member is complete.
    #[must_use]
    pub fn enabled(&self) -> bool {
        self.config.slots.iter().any(SlotConfig::enabled)
    }

    /// Executable flat pool slice — same as `config.pool()`.
    #[must_use]
    pub fn pool(&self) -> &[SlotConfig] {
        self.config.pool()
    }

    /// Executable pool size — number of enabled `(url, key, model)` members after the
    /// Cartesian expansion — for `/ai/status` (off members of incomplete slots are not counted).
    #[must_use]
    pub fn pool_size(&self) -> usize {
        self.config.slots.iter().filter(|s| s.enabled()).count()
    }

    /// `brain.py` execution plan: last winner first, the rest in natural pool order.
    ///
    /// Exactly `[last_good] + [i for i in range(len(pool)) if i != last_good]`
    /// when `last_good` is valid; otherwise the natural order.
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

    /// Pool group statuses for the wire — no keys at all.
    ///
    /// Pool members are grouped by name (first-seen order kept); each group's
    /// distinct models are comma-joined and a group is enabled = at least one member enabled.
    /// For a three-slot single-model pool the output is one-to-one with the previous behavior.
    #[must_use]
    pub fn slot_statuses(&self) -> Vec<SlotStatus> {
        let mut out: Vec<SlotStatus> = Vec::new();
        for s in &self.config.slots {
            if let Some(group) = out.iter_mut().find(|g| g.name == s.name) {
                if !group.model.split(',').any(|m| m.trim() == s.model) {
                    group.model.push(',');
                    group.model.push_str(&s.model);
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

    /// Chat — user messages are redacted, pool lanes are tried in plan order.
    ///
    /// # Errors
    /// [`GatewayError::Disabled`] with no enabled lane; [`GatewayError::AllSlotsFailed`]
    /// with per-attempt details when all enabled lanes failed and no acceptable least-bad existed.
    pub async fn chat(&self, messages: Vec<PacketMessage>) -> Result<ChatResult, GatewayError> {
        self.chat_with_max_tokens(messages, None).await
    }

    /// Multi-turn chat with an optional output token cap (e.g. `Some(4096)` for HTML code generation).
    ///
    /// The secret scanner runs on `user` messages so user keys/tokens never leave the system;
    /// system prompts and HTML code from previous `assistant` turns are left untouched.
    ///
    /// # Errors
    /// [`GatewayError::Disabled`] with no enabled lane; [`GatewayError::AllSlotsFailed`]
    /// with per-attempt details when all enabled lanes failed.
    pub async fn chat_with_max_tokens(
        &self,
        messages: Vec<PacketMessage>,
        max_tokens: Option<u32>,
    ) -> Result<ChatResult, GatewayError> {
        self.chat_with_quality(messages, max_tokens, false).await
    }

    /// Full gateway core with a `brain.py`-style quality gate.
    ///
    /// `is_builder = true` means an HTML artifact generation turn — the reply must carry an HTML document
    /// tag and the 64-character minimum applies; in chat mode a
    /// wrong-language reply (Persian prompt, Latin-heavy reply) is rejected.
    ///
    /// Least-bad stash (`least_bad`): the first HTTP 200 reply with non-empty text that
    /// fails the quality gate is not accepted immediately — it is stashed and the
    /// next lane is tried. If every lane failed:
    /// * Plain chat → the least-bad is returned (availability over quality — same as `brain.py`).
    /// * Builder → the least-bad is returned only if it carries an extractable HTML
    ///   document (`<!DOCTYPE html` or `<html`); otherwise [`GatewayError::AllSlotsFailed`]
    ///   propagates so the builder engine falls through to deterministic local synthesis.
    ///
    /// # Errors
    /// [`GatewayError::Disabled`] with no enabled lane; [`GatewayError::AllSlotsFailed`]
    /// when no lane produced an acceptable reply.
    // Allowed length for the lane orchestration function — step-order readability outranks splitting it up
    #[allow(clippy::too_many_lines)]
    pub async fn chat_with_quality(
        &self,
        messages: Vec<PacketMessage>,
        max_tokens: Option<u32>,
        is_builder: bool,
    ) -> Result<ChatResult, GatewayError> {
        // 1) redaction — on user messages before any lane sees them; categories are reported without content
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

        // 2) Enabled pool + execution plan (last winner first)
        let pool: Vec<&SlotConfig> = self.config.slots.iter().filter(|s| s.enabled()).collect();
        if pool.is_empty() {
            return Err(GatewayError::Disabled);
        }
        let plan = self.plan(pool.len());

        // Language reference for the quality gate — the last user message (the `ref` in brain.py)
        let prompt_ref = redacted
            .iter()
            .rev()
            .find(|m| m.role == "user")
            .or_else(|| redacted.last())
            .map_or("", |m| m.content.as_str());

        // 3) Pool scan — quality gate + least-bad stash
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
                    // Strict packet: empty content or negative tokens = malformed packet
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
                    // Winner — stickiness: this lane is tried first next time
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

        // 4) All failed — acceptable least-bad or typed error
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

    // ── Pool Cartesian product rule ─────────────────────────────────────────────

    /// Key list × model list expands exactly in "every model, every key" order and
    /// duplicate triples are dropped (first order kept).
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
                // Unconfigured secondary/tertiary slots — stay honestly off
                ("", "", ""),
                ("", "", ""),
            ],
            "model-outer × key-inner, dedup preserves first order"
        );
        assert!(cfg.slots.iter().take(4).all(|s| s.name == "primary"));
        assert_eq!(cfg.pool().iter().filter(|s| s.enabled()).count(), 4);
    }

    /// Hermes-style providers are appended to the end of the pool (Hermes order:
    /// coarena ← openrouter ← router9); they carry default URLs and accept a single key too.
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
                // Unconfigured logical slots — honestly off, not dropped
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

    /// Without Hermes providers and with single-value slots, the pool = previous behavior
    /// (three slots in the same order) — full backward compatibility.
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

    /// An incomplete slot (no key) is not expanded and reports as an off group; the
    /// multiple models of one group appear comma-joined in the status.
    #[test]
    fn status_groups_pool_entries_by_name() {
        let cfg = cfg_from(&[
            ("KIAN_AI_PRIMARY_URL", "https://a/v1"),
            ("KIAN_AI_PRIMARY_KEY", "k1"),
            ("KIAN_AI_PRIMARY_MODEL", "m1,m2,m3"),
            ("KIAN_AI_TERTIARY_URL", "https://t/v1"),
            ("KIAN_AI_TERTIARY_MODEL", "tm1,tm2"), // no key — off
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

    // ── last_good stickiness ──────────────────────────────────────────────────

    /// Lane 0 fails and lane 1 succeeds on the first call; the second call hits lane 1
    /// directly and never touches lane 0 (the `[last_good] + rest` plan).
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

        // Call 1: primary fails → secondary wins
        let r1 = gw.chat(msgs.clone()).await.expect("call 1 must succeed");
        assert_eq!(r1.provider, "secondary");
        assert_eq!(r1.model, "m2");
        assert_eq!(scripted.called_slots(), vec!["primary", "secondary"]);

        // Call 2: straight to secondary — primary is never touched
        let r2 = gw.chat(msgs.clone()).await.expect("call 2 must succeed");
        assert_eq!(r2.provider, "secondary");
        assert_eq!(
            scripted.called_slots(),
            vec!["primary", "secondary", "secondary"],
            "sticky last_good must skip the broken entry entirely"
        );
    }

    /// Triple-slot multi-model lane: the mid-pool winner becomes sticky — the plan starts
    /// at the winner and then continues with the rest in natural order.
    #[tokio::test]
    async fn sticky_plan_covers_whole_pool_in_natural_order_after_winner() {
        // Pool: primary×3 models; lane 1 (m2) wins
        let scripted = Arc::new(ScriptedClient::new(vec![
            Err("upstream status 502".into()), // m1
            Ok(CompletionPacket {
                content: "\u{67e}\u{627}\u{633}\u{62e} \u{62a}\u{645}\u{6cc}\u{632} \u{641}\u{627}\u{631}\u{633}\u{6cc} \u{628}\u{627} \u{62c}\u{632}\u{626}\u{6cc}\u{627}\u{62a} \u{6a9}\u{627}\u{641}\u{6cc}".into(),
                prompt_tokens: 4,
                completion_tokens: 6,
            }), // m2 wins
            Ok(CompletionPacket {
                content: "\u{67e}\u{627}\u{633}\u{62e} \u{62a}\u{645}\u{6cc}\u{632} \u{641}\u{627}\u{631}\u{633}\u{6cc} \u{628}\u{627} \u{62c}\u{632}\u{626}\u{6cc}\u{627}\u{62a} \u{6a9}\u{627}\u{641}\u{6cc}".into(),
                prompt_tokens: 4,
                completion_tokens: 6,
            }), // m3
            Ok(CompletionPacket {
                content: "\u{67e}\u{627}\u{633}\u{62e} \u{62a}\u{645}\u{6cc}\u{632} \u{641}\u{627}\u{631}\u{633}\u{6cc} \u{628}\u{627} \u{62c}\u{632}\u{626}\u{6cc}\u{627}\u{62a} \u{6a9}\u{627}\u{641}\u{6cc}".into(),
                prompt_tokens: 4,
                completion_tokens: 6,
            }), // Call 2: m2
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
            content: "\u{6cc}\u{6a9} \u{633}\u{648}\u{627}\u{644} \u{6a9}\u{627}\u{645}\u{644}\u{627}\u{64b} \u{641}\u{627}\u{631}\u{633}\u{6cc} \u{628}\u{631}\u{627}\u{6cc} \u{67e}\u{627}\u{633}\u{62e} \u{641}\u{627}\u{631}\u{633}\u{6cc}".into(),
        }];
        let r1 = gw.chat(msgs.clone()).await.expect("call 1");
        assert_eq!(r1.model, "m2", "m1 failed, m2 won");
        assert_eq!(scripted.called_slots(), vec!["primary", "primary"], "two tries in call 1");
        let r2 = gw.chat(msgs.clone()).await.expect("call 2");
        assert_eq!(r2.model, "m2", "sticky winner m2 first — m3/m1 untouched");
        assert_eq!(scripted.called_slots().len(), 3, "only one request in call 2");
    }

    // ── Quality gate ──────────────────────────────────────────────────────────

    /// Rules 1 and 2: empty/too short (chat <5) and the encoding corruption character.
    #[test]
    fn quality_gate_catches_empty_short_and_corrupt() {
        assert!(looks_broken("\u{633}\u{644}\u{627}\u{645}", "   ", false), "whitespace-only");
        assert!(looks_broken("\u{633}\u{644}\u{627}\u{645}", "ab", false), "2 chars < 5");
        assert!(looks_broken("hi", "ok!", false), "3 chars < 5");
        assert!(
            !looks_broken("\u{633}\u{644}\u{627}\u{645}", "\u{633}\u{644}\u{627}\u{645} \u{62f}\u{648}\u{633}\u{62a} \u{645}\u{646}", false),
            "exactly at/over the 5-char floor passes"
        );
        assert!(looks_broken("hi", "\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}", false));
        // Builder mode: cap 64
        assert!(looks_broken("\u{633}\u{644}\u{627}\u{645}", "\u{67e}\u{627}\u{633}\u{62e} \u{6a9}\u{648}\u{62a}\u{627}\u{647}", true), "builder floor is 64");
        assert!(
            !looks_broken(
                "hi",
                "<!DOCTYPE html><html><body><main><h1>Profit Calc Tool</h1><p>Full page body here</p></main></body></html>",
                true
            ),
            "valid-size builder doc passes"
        );
    }

    /// Rule 3: a run of >=5 identical alphabetic characters is caught; Markdown separators,
    /// hex colors, and code blocks stay intact.
    #[test]
    fn quality_gate_char_run_vs_separators() {
        assert!(looks_broken("hi", "\u{647}\u{647}\u{647}\u{647}\u{647}\u{647} \u{686}\u{647} \u{62c}\u{627}\u{644}\u{628}", false), "Persian degeneration");
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

    /// Chat rule 4: a Latin-heavy reply to a Persian prompt is rejected; a Persian reply and
    /// a code-heavy reply inside a fence with Persian commentary are accepted.
    #[test]
    fn quality_gate_language_mismatch_chat() {
        let fa_prompt = "\u{627}\u{6cc}\u{646} \u{6cc}\u{6a9} \u{67e}\u{631}\u{633}\u{634} \u{6a9}\u{627}\u{645}\u{644}\u{627}\u{64b} \u{641}\u{627}\u{631}\u{633}\u{6cc} \u{627}\u{633}\u{62a} \u{6a9}\u{647} \u{628}\u{6cc}\u{634} \u{627}\u{632} \u{62f}\u{647} \u{646}\u{648}\u{6cc}\u{633}\u{647}\u{654} \u{641}\u{627}\u{631}\u{633}\u{6cc} \u{62f}\u{627}\u{631}\u{62f}";
        assert!(
            looks_broken(fa_prompt, "Sure! Here is a complete English answer about the topic", false),
            "English-only reply to a Persian question is broken"
        );
        assert!(
            !looks_broken(fa_prompt, "\u{67e}\u{627}\u{633}\u{62e} \u{6a9}\u{627}\u{645}\u{644} \u{641}\u{627}\u{631}\u{633}\u{6cc} \u{628}\u{627} \u{62a}\u{648}\u{636}\u{6cc}\u{62d} \u{6a9}\u{627}\u{641}\u{6cc} \u{628}\u{631}\u{627}\u{6cc} \u{6a9}\u{627}\u{631}\u{628}\u{631}", false),
            "Persian reply passes"
        );
        assert!(
            !looks_broken(
                fa_prompt,
                "\u{62a}\u{648}\u{636}\u{6cc}\u{62d} \u{641}\u{627}\u{631}\u{633}\u{6cc}:\n```html\n<html><body>latin heavy content</body></html>\n```\n\u{67e}\u{627}\u{6cc}\u{627}\u{646} \u{62a}\u{648}\u{636}\u{6cc}\u{62d}",
                false
            ),
            "fenced code is exempt from the language ratio"
        );
        assert!(
            !looks_broken("hi there friend", "Sure! Here is a complete English answer", false),
            "non-Persian prompt skips the language rule"
        );
    }

    /// Builder rule 4: an HTML document is required; with a Persian prompt, at least 5 Persian
    /// characters as well; without a Persian prompt the document tag alone suffices.
    #[test]
    fn quality_gate_builder_mode_requires_html_and_persian() {
        let fa = "\u{6cc}\u{6a9} \u{645}\u{627}\u{634}\u{6cc}\u{646} \u{62d}\u{633}\u{627}\u{628} \u{633}\u{648}\u{62f} \u{627}\u{631}\u{632} \u{62f}\u{6cc}\u{62c}\u{6cc}\u{62a}\u{627}\u{644} \u{628}\u{627} \u{62a}\u{645} \u{62a}\u{6cc}\u{631}\u{647} \u{628}\u{633}\u{627}\u{632}";
        let good = "<!DOCTYPE html><html lang=\"fa\" dir=\"rtl\"><body><h1>\u{645}\u{627}\u{634}\u{6cc}\u{646} \u{62d}\u{633}\u{627}\u{628} \u{633}\u{648}\u{62f} \u{645}\u{631}\u{6a9}\u{628}</h1><script>let x=1;</script></body></html>";
        assert!(!looks_broken(fa, good, true), "valid Persian RTL artifact passes");
        assert!(
            looks_broken(fa, "\u{62a}\u{648}\u{636}\u{6cc}\u{62d} \u{6a9}\u{648}\u{62a}\u{627}\u{647} \u{628}\u{62f}\u{648}\u{646} \u{647}\u{6cc}\u{686} \u{6a9}\u{62f} \u{627}\u{686} \u{62a}\u{6cc} \u{627}\u{645} \u{627}\u{644}\u{6cc} \u{62f}\u{631} \u{67e}\u{627}\u{633}\u{62e} \u{6a9}\u{627}\u{631}\u{628}\u{631}", true),
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
            looks_broken(fa, "<html><body>\u{6a9}\u{648}\u{62a}\u{627}\u{647}</body></html>", true),
            "31 chars < 64 builder floor → broken"
        );
    }

    // ── Least-bad stash ───────────────────────────────────────────────────

    /// Lane 0 is broken (English reply to a Persian question) and lane 1 is clean → lane 1 wins.
    /// If lane 1 returns 429 instead of a clean reply, the stashed least-bad is salvaged for chat.
    #[tokio::test]
    async fn least_bad_stash_prefers_clean_and_salvages_when_all_fail() {
        let fa_q = "\u{6cc}\u{6a9} \u{67e}\u{631}\u{633}\u{634} \u{6a9}\u{627}\u{645}\u{644}\u{627}\u{64b} \u{641}\u{627}\u{631}\u{633}\u{6cc} \u{628}\u{627} \u{628}\u{6cc}\u{634} \u{627}\u{632} \u{62f}\u{647} \u{646}\u{648}\u{6cc}\u{633}\u{647}\u{654} \u{641}\u{627}\u{631}\u{633}\u{6cc} \u{628}\u{631}\u{627}\u{6cc} \u{6af}\u{6cc}\u{62a} \u{632}\u{628}\u{627}\u{646}";
        let garbled = "Totally unrelated English answer here for a Persian question";
        let clean = "\u{67e}\u{627}\u{633}\u{62e} \u{62a}\u{645}\u{6cc}\u{632} \u{641}\u{627}\u{631}\u{633}\u{6cc} \u{628}\u{627} \u{62c}\u{632}\u{626}\u{6cc}\u{627}\u{62a} \u{6a9}\u{627}\u{641}\u{6cc} \u{628}\u{631}\u{627}\u{6cc} \u{6a9}\u{627}\u{631}\u{628}\u{631}";
        let mk = |c: &str| Ok(CompletionPacket {
            content: c.to_owned(),
            prompt_tokens: 6,
            completion_tokens: 9,
        });

        // Case A: broken ← clean; clean wins
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

        // Case B: broken ← 429; the stashed least-bad is salvaged for chat
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

    /// Builder mode: a broken reply carrying an extractable HTML document → the least-bad is returned
    /// (the builder engine extracts it); a broken reply without HTML → an error so that
    /// deterministic local synthesis runs instead.
    #[tokio::test]
    async fn builder_least_bad_salvages_only_real_html_documents() {
        let fa_q = "\u{6cc}\u{6a9} \u{627}\u{628}\u{632}\u{627}\u{631} \u{641}\u{631}\u{648}\u{634}\u{6af}\u{627}\u{647}\u{6cc} \u{628}\u{627} \u{631}\u{627}\u{628}\u{637} \u{641}\u{627}\u{631}\u{633}\u{6cc} \u{628}\u{633}\u{627}\u{632}";
        let corrupt_doc = "<!DOCTYPE html><html dir=\"rtl\"><body><main><h1>\u{FFFD}\u{FFFD}\u{FFFD}\u{627}\u{628}\u{632}\u{627}\u{631} \u{641}\u{631}\u{648}\u{634}\u{FFFD}\u{FFFD}</h1></main></body></html>";
        let no_html = "\u{62a}\u{648}\u{636}\u{6cc}\u{62d} \u{6a9}\u{648}\u{62a}\u{627}\u{647} \u{628}\u{62f}\u{648}\u{646} \u{647}\u{6cc}\u{686} \u{633}\u{646}\u{62f} \u{627}\u{686} \u{62a}\u{6cc} \u{627}\u{645} \u{627}\u{644}\u{6cc} \u{62f}\u{631} \u{67e}\u{627}\u{633}\u{62e} \u{6a9}\u{627}\u{631}\u{628}\u{631}";
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

        // Corrupt HTML document → salvage for the builder
        let scripted = Arc::new(ScriptedClient::new(vec![mk(corrupt_doc)]));
        let gw = AiGateway::new(cfg_from(pairs), scripted.clone());
        let msgs = vec![PacketMessage { role: "user".into(), content: fa_q.to_owned() }];
        let r = gw
            .chat_with_quality(msgs.clone(), Some(BUILDER_MAX_TOKENS), true)
            .await
            .expect("doctype-bearing least-bad must be salvaged for builder");
        assert_eq!(r.content, corrupt_doc);

        // No HTML → typed error so BuilderEngine runs local synthesis
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
        assert!(attempts[0].detail.contains("quality gate"), "attempts: {attempts:?}");
    }
}
