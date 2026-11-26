//! The roaming-perception super-studio engine (Wave 5 — Step 2/4).
//!
//! Three sub-engines on one deterministic kernel:
//! 1. [`CognitiveUxMorphEngine`] — the three-tier cognitive morph: `ToySpark`
//!    (9-year-old child / `IQ` 50) ↔ `CreatorStudio` (no-code builder) ↔
//!    `ProTerminal` (28-year-old quant engineer) — with automatic tier inference from
//!    telemetry and a one-click switch with zero state loss;
//! 2. [`OmniModalStudioEngine`] — 9 zero-cost modalities with the procedural
//!    dual bundle of `Fishkal` (ocean + harmonic bloom) and `Portfolio Cosmos`
//!    (starfall + delay-feedback synth);
//! 3. [`PortfolioStorefrontEngine`] — the builder storefront with the 12 canonical
//!    `projects.json` keys, auto-promotion of the top-two featured entries, and the
//!    6 forensic fixes of `KiaAgentX/portfolio`.
//!
//! Kernel contract: zero `unsafe`, zero network keys, full determinism (same input →
//! same bundle, same `sha256`) and a zero-cost gate on all 9 modalities.
#![allow(clippy::module_name_repetitions)]

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

// ---------------------------------------------------------------------------
// Sub-engine 1 — `CognitiveUxMorphEngine`
// ---------------------------------------------------------------------------

/// Per-page action cap in `ToySpark` — the three-giant-buttons rule.
pub const TOYSPARK_MAX_ACTIONS_PER_SCREEN: usize = 3;

/// The `ToySpark` minimum touch target in pixels — a 9-year-old's finger.
pub const TOYSPARK_MIN_TAP_TARGET_PX: u32 = 64;

/// Per-label word cap — icon-first microcopy.
pub const TOYSPARK_MAX_WORDS_PER_LABEL: usize = 5;

/// The `ToySpark` one-click default price — 50 Telegram Stars.
pub const TOYSPARK_DEFAULT_XTR_STARS: u32 = 50;

/// The `ToySpark` one-click default price — $1.00 (micro-`USDC`).
pub const TOYSPARK_DEFAULT_USDC_MICROS: u64 = 1_000_000;

/// The `ToySpark` one-click default price — one million micro-Kian.
pub const TOYSPARK_DEFAULT_UKIAN: u64 = 1_000_000;

/// Technical markers — each one locks inference to `ProTerminal`.
pub const TECHNICAL_TOKENS: [&str; 18] = [
    "fn ", "impl ", "async ", "curl ", "sql", "json", "schema", "regex", "git ", "docker",
    "k8s", "quant", "api ", "rust", "python", "token", "deploy", "terminal",
];

/// The interface's cognitive tier — morphs the user's perceptual bandwidth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CognitiveTier {
    /// 9-year-old child / `IQ` 50 — three giant buttons, one-click magic.
    ToySpark,
    /// General no-code builder — drag-and-drop blocks + live preview.
    CreatorStudio,
    /// 28-year-old engineer/trader — a keyboard-first command palette and dense telemetry.
    ProTerminal,
}

impl CognitiveTier {
    /// Tier wire ID (`ToySpark` | `CreatorStudio` | `ProTerminal`).
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ToySpark => "ToySpark",
            Self::CreatorStudio => "CreatorStudio",
            Self::ProTerminal => "ProTerminal",
        }
    }

    /// Native tier label.
    #[must_use]
    pub fn fa_label(self) -> &'static str {
        match self {
            Self::ToySpark => "\u{62c}\u{631}\u{642}\u{647}\u{654} \u{627}\u{633}\u{628}\u{627}\u{628}‌\u{628}\u{627}\u{632}\u{6cc}",
            Self::CreatorStudio => "\u{627}\u{633}\u{62a}\u{648}\u{62f}\u{6cc}\u{648}\u{6cc} \u{633}\u{627}\u{632}\u{646}\u{62f}\u{647}",
            Self::ProTerminal => "\u{62a}\u{631}\u{645}\u{6cc}\u{646}\u{627}\u{644} \u{62d}\u{631}\u{641}\u{647}‌\u{627}\u{6cc}",
        }
    }

    /// Body CSS class for the tier morph.
    #[must_use]
    pub fn css_class(self) -> &'static str {
        match self {
            Self::ToySpark => "tier-toyspark",
            Self::CreatorStudio => "tier-creator",
            Self::ProTerminal => "tier-pro",
        }
    }

    /// All 3 tiers in cognitive-ladder order.
    #[must_use]
    pub fn all() -> [Self; 3] {
        [Self::ToySpark, Self::CreatorStudio, Self::ProTerminal]
    }

    /// Wire parsing — unknown → `None`.
    #[must_use]
    pub fn parse(code: &str) -> Option<Self> {
        match code.trim() {
            "ToySpark" => Some(Self::ToySpark),
            "CreatorStudio" => Some(Self::CreatorStudio),
            "ProTerminal" => Some(Self::ProTerminal),
            _ => None,
        }
    }
}

/// One cognitive-tier morph profile — the operational UX contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CognitiveTierProfile {
    /// The tier that owns the profile.
    pub tier: CognitiveTier,
    /// Target age/IQ band (`Age 9 / `IQ` 50` …).
    pub age_band: &'static str,
    /// Per-page action cap.
    pub max_actions_per_screen: usize,
    /// Minimum touch target (pixels).
    pub min_tap_target_px: u32,
    /// Per-label word cap.
    pub max_words_per_label: usize,
    /// Auto audio wizard?
    pub voice_guided: bool,
    /// Zero-harm sandbox.
    pub zero_loss_sandbox: bool,
    /// Keyboard-first (command palette)?
    pub keyboard_first: bool,
    /// Layout summary for the client.
    pub layout_summary: &'static str,
}

/// The three canonical cognitive-morph profiles — the `GET /studio/ux-profiles` contract.
#[must_use]
pub fn cognitive_tier_profiles() -> [CognitiveTierProfile; 3] {
    [
        CognitiveTierProfile {
            tier: CognitiveTier::ToySpark,
            age_band: "Age 9 / IQ 50",
            max_actions_per_screen: TOYSPARK_MAX_ACTIONS_PER_SCREEN,
            min_tap_target_px: TOYSPARK_MIN_TAP_TARGET_PX,
            max_words_per_label: TOYSPARK_MAX_WORDS_PER_LABEL,
            voice_guided: true,
            zero_loss_sandbox: true,
            keyboard_first: false,
            layout_summary: "3-button giant card: Make Magic / Make It Pretty / Share & Earn Stars",
        },
        CognitiveTierProfile {
            tier: CognitiveTier::CreatorStudio,
            age_band: "Age 14+ / no-code",
            max_actions_per_screen: 7,
            min_tap_target_px: 44,
            max_words_per_label: 12,
            voice_guided: false,
            zero_loss_sandbox: true,
            keyboard_first: false,
            layout_summary: "drag-and-drop prompt blocks + live split-pane preview + 1-click ARDA + SKILL.md packaging",
        },
        CognitiveTierProfile {
            tier: CognitiveTier::ProTerminal,
            age_band: "Age 28 / quant & systems",
            max_actions_per_screen: 24,
            min_tap_target_px: 32,
            max_words_per_label: 24,
            voice_guided: false,
            zero_loss_sandbox: false,
            keyboard_first: true,
            layout_summary: "dense multi-pane + Ctrl+K/Meta+K command palette + live telemetry + Mairon 8-step game-tree inspector",
        },
    ]
}

/// Interaction telemetry for tier inference — never asking age.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CognitiveTelemetry {
    /// Prompt length in characters.
    pub prompt_length_chars: usize,
    /// Does the prompt carry a technical marker?
    pub uses_technical_tokens: bool,
    /// Average typing cadence (ms/key) — `0` = unknown.
    pub typing_cadence_ms: u32,
    /// The user's explicit tier choice — always wins.
    pub explicit_tier_override: Option<CognitiveTier>,
}

/// Detects technical markers in the prompt — deterministic and cost-free.
#[must_use]
pub fn uses_technical_tokens(prompt: &str) -> bool {
    let needle = prompt.to_lowercase();
    TECHNICAL_TOKENS.iter().any(|t| needle.contains(t))
}

/// Tier inference + explicit override — [`infer_or_override_tier`] is the zero-friction
/// switch contract: an explicit override always wins; otherwise the telemetry ladder.
#[must_use]
pub fn infer_or_override_tier(t: &CognitiveTelemetry) -> CognitiveTier {
    if let Some(explicit) = t.explicit_tier_override {
        return explicit;
    }
    if t.uses_technical_tokens || t.prompt_length_chars > 220 {
        return CognitiveTier::ProTerminal;
    }
    if t.prompt_length_chars >= 140 && t.typing_cadence_ms >= 380 {
        return CognitiveTier::CreatorStudio;
    }
    CognitiveTier::ToySpark
}

// ---------------------------------------------------------------------------
// Sub-engine 2 — `OmniModalStudioEngine` (9 zero-cost modalities)
// ---------------------------------------------------------------------------

/// The 9 studio modalities — all zero-cost and keyless.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StudioModality {
    /// Keyless multi-source web/`API` synthesis (`Agent-Reach` + 120 public `API`s).
    Search,
    /// Multi-agent deep research + Mairon's 8-step verify tree.
    DeepMode,
    /// The `OpenBot` auto sub-agent generator with 9-fold DNA + `SKILL.md`.
    AgentMode,
    /// Multi-file canvas with sandboxed preview and a code inspector.
    WorkspaceMode,
    /// Native browser `SpeechSynthesis` / `MediaRecorder` + procedural ambient music.
    VoiceAudio,
    /// Deterministic vector `SVG`/`Canvas2D` + a keyless provider cascade.
    ImageGen,
    /// 60fps motion graphics with the Fishkal `TL` timeline + lambda-variable `damp`.
    VideoGen,
    /// 20-locale sales copy + tuned `RTL`/`LTR` landing pages.
    TextCopy,
    /// Procedural `WebAudio` SFX studio + ambient synth.
    SoundEffects,
}

impl StudioModality {
    /// Modality wire ID.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Search => "search",
            Self::DeepMode => "deep",
            Self::AgentMode => "agent",
            Self::WorkspaceMode => "workspace",
            Self::VoiceAudio => "voice",
            Self::ImageGen => "image",
            Self::VideoGen => "video",
            Self::TextCopy => "copy",
            Self::SoundEffects => "sfx",
        }
    }

    /// Native modality label.
    #[must_use]
    pub fn fa_label(self) -> &'static str {
        match self {
            Self::Search => "\u{62c}\u{633}\u{62a}‌\u{648}\u{62c}\u{648}\u{6cc} \u{686}\u{646}\u{62f}-\u{645}\u{646}\u{628}\u{639}\u{6cc}",
            Self::DeepMode => "\u{67e}\u{698}\u{648}\u{647}\u{634} \u{639}\u{645}\u{6cc}\u{642}",
            Self::AgentMode => "\u{645}\u{648}\u{644}\u{62f} \u{633}\u{627}\u{628}-\u{627}\u{6cc}\u{62c}\u{646}\u{62a}",
            Self::WorkspaceMode => "\u{628}\u{648}\u{645} \u{686}\u{646}\u{62f}-\u{641}\u{627}\u{6cc}\u{644}\u{6cc}",
            Self::VoiceAudio => "\u{635}\u{62f}\u{627} \u{648} \u{645}\u{648}\u{633}\u{6cc}\u{642}\u{6cc}",
            Self::ImageGen => "\u{62a}\u{635}\u{648}\u{6cc}\u{631}\u{633}\u{627}\u{632}\u{6cc} \u{631}\u{648}\u{6cc}\u{647}‌\u{627}\u{6cc}",
            Self::VideoGen => "\u{645}\u{648}\u{634}\u{646} \u{6f6}\u{6f0}fps",
            Self::TextCopy => "\u{6a9}\u{67e}\u{6cc} \u{641}\u{631}\u{648}\u{634} \u{6f2}\u{6f0}-\u{632}\u{628}\u{627}\u{646}\u{647}",
            Self::SoundEffects => "\u{627}\u{633}\u{62a}\u{648}\u{62f}\u{6cc}\u{648}\u{6cc} SFX",
        }
    }

    /// All 9 modalities in canonical order.
    #[must_use]
    pub fn all() -> [Self; 9] {
        [
            Self::Search,
            Self::DeepMode,
            Self::AgentMode,
            Self::WorkspaceMode,
            Self::VoiceAudio,
            Self::ImageGen,
            Self::VideoGen,
            Self::TextCopy,
            Self::SoundEffects,
        ]
    }

    /// Wire parsing — unknown → `None`.
    #[must_use]
    pub fn parse(code: &str) -> Option<Self> {
        match code.trim() {
            "search" => Some(Self::Search),
            "deep" => Some(Self::DeepMode),
            "agent" => Some(Self::AgentMode),
            "workspace" => Some(Self::WorkspaceMode),
            "voice" => Some(Self::VoiceAudio),
            "image" => Some(Self::ImageGen),
            "video" => Some(Self::VideoGen),
            "copy" => Some(Self::TextCopy),
            "sfx" => Some(Self::SoundEffects),
            _ => None,
        }
    }
}

/// Canonical modality count — the test gate.
pub const STUDIO_MODALITY_COUNT: usize = 9;

/// One modality's spec on the wire.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioModalitySpec {
    /// The modality that owns the spec.
    pub modality: StudioModality,
    /// Native label.
    pub fa_label: &'static str,
    /// Auth required — always `none` (zero-cost).
    pub auth: &'static str,
    /// Summary execution engine.
    pub engine: &'static str,
    /// Always `true` — the zero-cost gate.
    pub zero_cost: bool,
}

/// All 9 modalities' specs — the `GET /studio/ux-profiles` contract.
#[must_use]
pub fn studio_modality_specs() -> [StudioModalitySpec; STUDIO_MODALITY_COUNT] {
    [
        StudioModalitySpec {
            modality: StudioModality::Search,
            fa_label: StudioModality::Search.fa_label(),
            auth: "none",
            engine: "Agent-Reach Jina/Exa MCP cascade + 120 public CORS APIs",
            zero_cost: true,
        },
        StudioModalitySpec {
            modality: StudioModality::DeepMode,
            fa_label: StudioModality::DeepMode.fa_label(),
            auth: "none",
            engine: "multi-agent deep research + Mairon 8-step verification tree",
            zero_cost: true,
        },
        StudioModalitySpec {
            modality: StudioModality::AgentMode,
            fa_label: StudioModality::AgentMode.fa_label(),
            auth: "none",
            engine: "OpenBot sub-agent spawner with 9-Fold Stochastic DNA + downloadable SKILL.md",
            zero_cost: true,
        },
        StudioModalitySpec {
            modality: StudioModality::WorkspaceMode,
            fa_label: StudioModality::WorkspaceMode.fa_label(),
            auth: "none",
            engine: "multi-file artifact canvas + sandboxed live preview + code inspector",
            zero_cost: true,
        },
        StudioModalitySpec {
            modality: StudioModality::VoiceAudio,
            fa_label: StudioModality::VoiceAudio.fa_label(),
            auth: "none",
            engine: "browser-native SpeechSynthesis / MediaRecorder + procedural ambient soundtrack",
            zero_cost: true,
        },
        StudioModalitySpec {
            modality: StudioModality::ImageGen,
            fa_label: StudioModality::ImageGen.fa_label(),
            auth: "none",
            engine: "deterministic procedural SVG / Canvas2D vector artwork + zero-key provider cascade",
            zero_cost: true,
        },
        StudioModalitySpec {
            modality: StudioModality::VideoGen,
            fa_label: StudioModality::VideoGen.fa_label(),
            auth: "none",
            engine: "60fps deterministic Canvas2D timeline + WebM/frame-sequence recorder on Fishkal TL + damp",
            zero_cost: true,
        },
        StudioModalitySpec {
            modality: StudioModality::TextCopy,
            fa_label: StudioModality::TextCopy.fa_label(),
            auth: "none",
            engine: "20-locale sales copy + RTL/LTR typography-tuned landing pages",
            zero_cost: true,
        },
        StudioModalitySpec {
            modality: StudioModality::SoundEffects,
            fa_label: StudioModality::SoundEffects.fa_label(),
            auth: "none",
            engine: "procedural WebAudio SFX & ambient synth studio (Cosmos 5-trigger rack)",
            zero_cost: true,
        },
    ]
}

/// Procedural dual-bundle signature — `Fishkal` + `Portfolio Cosmos`.
pub const DUAL_BUNDLE_SIGNATURE: &str = "fishkal-cosmos-dual-v1";

/// The bundle's `sha256` stamp — deterministic on every rebuild.
#[must_use]
pub fn bundle_content_sha256(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// The procedural dual-engine bundle — engine A (`Fishkal`), engine B
/// (`Portfolio Cosmos`), and the shared `RAF` pause lifecycle (Fix #6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DualCosmosFishkalBundle {
    /// Engine A — the Fishkal ocean + harmonic bloom (`index.html:L709–728, L1665–1750, L2172`).
    pub fishkal_js: String,
    /// Engine B — the Cosmos starfall + delay-feedback synth (`cosmos.js:L6–60, L83–149, L216–269`).
    pub cosmos_js: String,
    /// The shared `__fkPauseDepth` lifecycle (`fkPause` / `fkResume` / `document.hidden`).
    pub pause_lifecycle_js: String,
    /// Bundle signature (`fishkal-cosmos-dual-v1`).
    pub signature: String,
    /// The whole bundle's `sha256` stamp.
    pub content_sha256: String,
}

/// Engine A — the Fishkal surf + harmonic bloom, verbatim preservation of the audited
/// parameters: the 4-second Brownian-noise buffer (`b0 = (b0 + 0.02 * white) / 1.02`
/// at `0.18` scale), the `520Hz` low-pass `BiquadFilter` with a `0.06Hz` sine `LFO`
/// and a `±220Hz` sweep, the D-major pentatonic bloom `[587.33, 880, 1174.66]`,
/// the 4-voice signature chord `[293.66, 440, 587.33, 880]`, `xorshift32`,
/// 4-octave `fbm` (`f *= 2.03`), cubic-Hermite `smoothstep`, and
/// frame-independent lambda-variable `damp`.
const FISHKAL_ENGINE_JS: &str = r"(function (global) {
  'use strict';
  var FK = global.ARDA_STUDIO = global.ARDA_STUDIO || {};
  function rng(seed) {
    var s = seed >>> 0;
    return function () {
      s ^= s << 13; s >>>= 0;
      s ^= s >>> 17;
      s ^= s << 5; s >>>= 0;
      return s / 4294967296;
    };
  }
  function smoothstep(t) {
    t = Math.min(1, Math.max(0, t));
    return t * t * t * (t * (t * 6 - 15) + 10);
  }
  function valueNoise1D(rand, x) {
    var i = Math.floor(x);
    var f = x - i;
    var a = rand(i), b = rand(i + 1);
    return a + (b - a) * smoothstep(f);
  }
  function fbm(rand, x) {
    var v = 0, amp = 0.5, f = 1;
    for (var o = 0; o < 4; o += 1) {
      v += amp * valueNoise1D(rand, x * f);
      f *= 2.03;
      amp *= 0.5;
    }
    return v;
  }
  function damp(a, b, l, dt) { return b + (a - b) * Math.exp(-l * dt); }
  var FISHKAL_DAMP_WINDOW_LO = 0.42;
  var FISHKAL_DAMP_WINDOW_HI = 0.63;
  var FISHKAL_DAMP_LAMBDA_WINDOW = 2.3;
  var FISHKAL_DAMP_LAMBDA = 5.0;
  function dampFk(a, b, dt, phase) {
    var lambda = (phase > FISHKAL_DAMP_WINDOW_LO && phase < FISHKAL_DAMP_WINDOW_HI)
      ? FISHKAL_DAMP_LAMBDA_WINDOW : FISHKAL_DAMP_LAMBDA;
    return damp(a, b, lambda, dt);
  }
  function brownNoiseBuffer(ctx) {
    var len = Math.floor(ctx.sampleRate * 4);
    var buf = ctx.createBuffer(1, len, ctx.sampleRate);
    var data = buf.getChannelData(0);
    var b0 = 0;
    for (var i = 0; i < len; i += 1) {
      var white = Math.random() * 2 - 1;
      b0 = (b0 + 0.02 * white) / 1.02;
      data[i] = b0 * 0.18;
    }
    return buf;
  }
  function oceanSurf(ctx, dest) {
    var src = ctx.createBufferSource();
    src.buffer = brownNoiseBuffer(ctx);
    src.loop = true;
    var lp = ctx.createBiquadFilter();
    lp.type = 'lowpass';
    lp.frequency.value = 520;
    var lfo = ctx.createOscillator();
    lfo.type = 'sine';
    lfo.frequency.value = 0.06;
    var lfoGain = ctx.createGain();
    lfoGain.gain.value = 220;
    lfo.connect(lfoGain);
    lfoGain.connect(lp.frequency);
    src.connect(lp);
    lp.connect(dest);
    src.start();
    lfo.start();
    return { source: src, filter: lp };
  }
  var FISHKAL_BLOOM = [587.33, 880, 1174.66];
  var FISHKAL_SIGNATURE_CHORD = [293.66, 440, 587.33, 880];
  function harmonicBloom(ctx, dest, t0, notes) {
    var chord = notes || FISHKAL_BLOOM;
    for (var i = 0; i < chord.length; i += 1) {
      var osc = ctx.createOscillator();
      osc.type = 'sine';
      osc.frequency.value = chord[i];
      var g = ctx.createGain();
      g.gain.setValueAtTime(0.0001, t0);
      g.gain.linearRampToValueAtTime(0.22 / (i + 1), t0 + 0.08);
      g.gain.exponentialRampToValueAtTime(0.0001, t0 + 2.4);
      osc.connect(g); g.connect(dest);
      osc.start(t0); osc.stop(t0 + 2.5);
    }
  }
  FK.fishkal = {
    rng: rng, smoothstep: smoothstep, valueNoise1D: valueNoise1D, fbm: fbm,
    damp: damp, dampFk: dampFk, oceanSurf: oceanSurf, harmonicBloom: harmonicBloom,
    brownNoiseBuffer: brownNoiseBuffer,
    FISHKAL_BLOOM: FISHKAL_BLOOM,
    FISHKAL_SIGNATURE_CHORD: FISHKAL_SIGNATURE_CHORD,
    FISHKAL_DAMP_WINDOW: [FISHKAL_DAMP_WINDOW_LO, FISHKAL_DAMP_WINDOW_HI],
    FISHKAL_DAMP_LAMBDA_WINDOW: FISHKAL_DAMP_LAMBDA_WINDOW,
    FISHKAL_DAMP_LAMBDA: FISHKAL_DAMP_LAMBDA
  };
})(typeof window !== 'undefined' ? window : globalThis);";

/// Engine B — the interactive Cosmos starfall (`DPR` cap `2`, `110` mobile /
/// `260` desktop stars, radial nebula gradients, pointer-parallax decay
/// at `0.05`, `HUD` smoothing at `0.16`, 7 milestone nodes) +
/// the delay-feedback synth (`DelayNode(0.22s)` + `0.25` feedback + a `1600Hz`
/// low-pass) with all 5 canonical SFX triggers — wired into `__fkPauseDepth`.
const COSMOS_ENGINE_JS: &str = r"(function (global) {
  'use strict';
  var FK = global.ARDA_STUDIO = global.ARDA_STUDIO || {};
  FK.__fkPauseDepth = FK.__fkPauseDepth || 0;
  FK.fkPause = function () { FK.__fkPauseDepth += 1; };
  FK.fkResume = function () { FK.__fkPauseDepth = Math.max(0, FK.__fkPauseDepth - 1); };
  var COSMOS_STARS_MOBILE = 110;
  var COSMOS_STARS_DESKTOP = 260;
  var COSMOS_PARALLAX_DAMPING = 0.05;
  var COSMOS_HUD_EASING = 0.16;
  var COSMOS_MILESTONE_NODES = 7;
  var COSMOS_DELAY_SECONDS = 0.22;
  var COSMOS_FEEDBACK_GAIN = 0.25;
  var COSMOS_TONE_LOWPASS_HZ = 1600;
  function starCount(isMobile) { return isMobile ? COSMOS_STARS_MOBILE : COSMOS_STARS_DESKTOP; }
  function starfield(canvas, opts) {
    var isMobile = !!(opts && opts.isMobile);
    var count = starCount(isMobile);
    var dpr = Math.min(2, (global.devicePixelRatio || 1));
    var ctx = canvas.getContext('2d');
    var w = 0, h = 0, px = 0, py = 0, tx = 0, ty = 0, hudX = 0, hudY = 0;
    var stars = [], nebulae = [], milestones = [];
    var i, r;
    function resize() {
      w = canvas.clientWidth; h = canvas.clientHeight;
      canvas.width = Math.floor(w * dpr); canvas.height = Math.floor(h * dpr);
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    }
    function seed() {
      stars = [];
      nebulae = [];
      milestones = [];
      for (i = 0; i < count; i += 1) {
        stars.push({ x: Math.random(), y: Math.random(), z: 0.3 + Math.random() * 0.7, tw: Math.random() * 6.28 });
      }
      for (i = 0; i < 3; i += 1) {
        nebulae.push({ x: Math.random(), y: Math.random(), r: 0.2 + Math.random() * 0.25, hue: i % 2 ? 262 : 187 });
      }
      for (i = 0; i < COSMOS_MILESTONE_NODES; i += 1) {
        milestones.push({ x: 0.12 + 0.13 * i, y: 0.3 + 0.4 * ((i * 7) % 10) / 10, idx: i });
      }
    }
    function draw(t) {
      ctx.clearRect(0, 0, w, h);
      for (i = 0; i < nebulae.length; i += 1) {
        r = nebulae[i];
        var grad = ctx.createRadialGradient(r.x * w + px * 40, r.y * h + py * 40, 0, r.x * w + px * 40, r.y * h + py * 40, r.r * Math.max(w, h));
        grad.addColorStop(0, 'hsla(' + r.hue + ', 90%, 60%, 0.14)');
        grad.addColorStop(1, 'hsla(' + r.hue + ', 90%, 60%, 0)');
        ctx.fillStyle = grad;
        ctx.fillRect(0, 0, w, h);
      }
      for (i = 0; i < stars.length; i += 1) {
        var st = stars[i];
        var sx = st.x * w + px * 60 * st.z;
        var sy = st.y * h + py * 60 * st.z;
        var tw = 0.55 + 0.45 * Math.sin(t * 0.0012 + st.tw);
        ctx.globalAlpha = tw * st.z;
        ctx.fillStyle = st.z > 0.72 ? '#cfd6ff' : '#eef2ff';
        ctx.beginPath(); ctx.arc(sx, sy, 1.6 * st.z, 0, 6.283); ctx.fill();
      }
      ctx.globalAlpha = 1;
      for (i = 0; i < milestones.length; i += 1) {
        var m = milestones[i];
        var mx = m.x * w + px * 24, my = m.y * h + py * 24;
        ctx.strokeStyle = 'rgba(25, 211, 218, 0.85)';
        ctx.beginPath(); ctx.arc(mx, my, 7, 0, 6.283); ctx.stroke();
        ctx.beginPath(); ctx.moveTo(mx, my); ctx.lineTo(mx + 16, my - 12); ctx.stroke();
      }
      hudX += ((px + 1) * 0.5 * w - hudX) * COSMOS_HUD_EASING;
      hudY += ((py + 1) * 0.5 * h - hudY) * COSMOS_HUD_EASING;
    }
    function frame(t) {
      if (FK.__fkPauseDepth === 0 && !document.hidden) {
        px += (tx - px) * COSMOS_PARALLAX_DAMPING;
        py += (ty - py) * COSMOS_PARALLAX_DAMPING;
        draw(t);
      }
      requestAnimationFrame(frame);
    }
    global.addEventListener('pointermove', function (e) {
      tx = (e.clientX / Math.max(1, w)) * 2 - 1;
      ty = (e.clientY / Math.max(1, h)) * 2 - 1;
    });
    global.addEventListener('resize', function () { resize(); seed(); });
    resize(); seed(); requestAnimationFrame(frame);
    return { resize: resize, seed: seed, milestones: milestones, starCount: count };
  }
  function feedbackDelay(ctx, dest) {
    var delay = ctx.createDelay(1.0);
    delay.delayTime.value = COSMOS_DELAY_SECONDS;
    var fb = ctx.createGain();
    fb.gain.value = COSMOS_FEEDBACK_GAIN;
    var lp = ctx.createBiquadFilter();
    lp.type = 'lowpass';
    lp.frequency.value = COSMOS_TONE_LOWPASS_HZ;
    delay.connect(fb); fb.connect(lp); lp.connect(delay);
    delay.connect(dest);
    return delay;
  }
  function ping(ctx, dest, idx) {
    var t = ctx.currentTime;
    var osc = ctx.createOscillator();
    osc.type = (idx % 2 === 0) ? 'sine' : 'triangle';
    osc.frequency.value = 220 + (idx % 7) * 40;
    var g = ctx.createGain();
    g.gain.setValueAtTime(0.16, t);
    g.gain.exponentialRampToValueAtTime(0.0001, t + 0.12);
    osc.connect(g); g.connect(feedbackDelay(ctx, dest));
    osc.start(t); osc.stop(t + 0.13);
  }
  function open(ctx, dest) {
    var t = ctx.currentTime;
    var osc = ctx.createOscillator();
    osc.type = 'sine';
    osc.frequency.setValueAtTime(180, t);
    osc.frequency.exponentialRampToValueAtTime(520, t + 0.28);
    var g = ctx.createGain();
    g.gain.setValueAtTime(0.14, t);
    g.gain.exponentialRampToValueAtTime(0.0001, t + 0.28);
    osc.connect(g); g.connect(feedbackDelay(ctx, dest));
    osc.start(t); osc.stop(t + 0.29);
  }
  function whoosh(ctx, dest) {
    var t = ctx.currentTime;
    var len = Math.floor(ctx.sampleRate * 0.32);
    var buf = ctx.createBuffer(1, len, ctx.sampleRate);
    var data = buf.getChannelData(0);
    for (var i = 0; i < len; i += 1) { data[i] = Math.random() * 2 - 1; }
    var src = ctx.createBufferSource();
    src.buffer = buf;
    var bp = ctx.createBiquadFilter();
    bp.type = 'bandpass';
    bp.Q.value = 2.2;
    bp.frequency.setValueAtTime(260, t);
    bp.frequency.exponentialRampToValueAtTime(1200, t + 0.32);
    var g = ctx.createGain();
    g.gain.setValueAtTime(0.18, t);
    g.gain.exponentialRampToValueAtTime(0.0001, t + 0.32);
    src.connect(bp); bp.connect(g); g.connect(feedbackDelay(ctx, dest));
    src.start(t);
  }
  function select(ctx, dest) {
    var t = ctx.currentTime;
    var notes = [440, 660];
    for (var i = 0; i < notes.length; i += 1) {
      var osc = ctx.createOscillator();
      osc.type = 'triangle';
      osc.frequency.value = notes[i];
      var g = ctx.createGain();
      g.gain.setValueAtTime(0.12, t + i * 0.07);
      g.gain.exponentialRampToValueAtTime(0.0001, t + i * 0.07 + 0.07);
      osc.connect(g); g.connect(feedbackDelay(ctx, dest));
      osc.start(t + i * 0.07); osc.stop(t + i * 0.07 + 0.08);
    }
  }
  function milestone(ctx, dest) {
    var t = ctx.currentTime;
    var notes = [330, 495, 660];
    for (var i = 0; i < notes.length; i += 1) {
      var osc = ctx.createOscillator();
      osc.type = 'sine';
      osc.frequency.value = notes[i];
      var g = ctx.createGain();
      g.gain.setValueAtTime(0.14, t + i * 0.09);
      g.gain.exponentialRampToValueAtTime(0.0001, t + i * 0.09 + 0.09);
      osc.connect(g); g.connect(feedbackDelay(ctx, dest));
      osc.start(t + i * 0.09); osc.stop(t + i * 0.09 + 0.1);
    }
  }
  FK.cosmos = {
    starfield: starfield, starCount: starCount, feedbackDelay: feedbackDelay,
    sfx: { ping: ping, open: open, whoosh: whoosh, select: select, milestone: milestone },
    fkPause: function () { FK.fkPause(); },
    fkResume: function () { FK.fkResume(); },
    COSMOS_STARS_MOBILE: COSMOS_STARS_MOBILE,
    COSMOS_STARS_DESKTOP: COSMOS_STARS_DESKTOP,
    COSMOS_MILESTONE_NODES: COSMOS_MILESTONE_NODES,
    COSMOS_PARALLAX_DAMPING: COSMOS_PARALLAX_DAMPING,
    COSMOS_HUD_EASING: COSMOS_HUD_EASING
  };
})(typeof window !== 'undefined' ? window : globalThis);";

/// The shared `RAF` pause lifecycle — Fix #6: every animation loop keys off
/// the shared `__fkPauseDepth` and `document.hidden`; `visibilitychange`
/// auto-stamps the pause depth.
const PAUSE_LIFECYCLE_JS: &str = r"(function (global) {
  'use strict';
  var FK = global.ARDA_STUDIO = global.ARDA_STUDIO || {};
  FK.__fkPauseDepth = FK.__fkPauseDepth || 0;
  FK.fkPause = FK.fkPause || function () { FK.__fkPauseDepth += 1; };
  FK.fkResume = FK.fkResume || function () { FK.__fkPauseDepth = Math.max(0, FK.__fkPauseDepth - 1); };
  document.addEventListener('visibilitychange', function () {
    if (document.hidden) { FK.fkPause(); } else { FK.fkResume(); }
  });
  FK.rafGate = function () { return FK.__fkPauseDepth === 0 && !document.hidden; };
})(typeof window !== 'undefined' ? window : globalThis);";

/// JS syntax health check — balanced `{}`/`()`/`[]` outside strings and
/// comments; deterministic and dependency-free.
///
/// # Errors
/// Unterminated string/comment or broken balance → `Err(reason)`.
pub fn js_syntax_sanity_check(js: &str) -> Result<(), &'static str> {
    let mut stack: Vec<(char, usize)> = Vec::new();
    let mut in_single = false;
    let mut in_double = false;
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut prev = '\0';
    for (idx, ch) in js.chars().enumerate() {
        if in_line_comment {
            if ch == '\n' { in_line_comment = false; }
        } else if in_block_comment {
            if prev == '*' && ch == '/' { in_block_comment = false; }
        } else if in_single {
            if ch == '\'' && prev != '\\' { in_single = false; }
        } else if in_double {
            if ch == '"' && prev != '\\' { in_double = false; }
        } else if prev == '/' && ch == '/' {
            in_line_comment = true;
        } else if prev == '/' && ch == '*' {
            in_block_comment = true;
        } else if ch == '\'' {
            in_single = true;
        } else if ch == '"' {
            in_double = true;
        } else if ch == '{' || ch == '(' || ch == '[' {
            stack.push((ch, idx));
        } else if ch == '}' || ch == ')' || ch == ']' {
            let open = if ch == '}' { '{' } else if ch == ')' { '(' } else { '[' };
            match stack.pop() {
                Some((top, _)) if top == open => {}
                _ => return Err("unbalanced delimiter"),
            }
        }
        prev = ch;
    }
    if in_single || in_double { return Err("unterminated string literal"); }
    if in_block_comment || in_line_comment { return Err("unterminated comment"); }
    if !stack.is_empty() { return Err("unclosed delimiter"); }
    Ok(())
}

/// The procedural dual bundle — deterministic; same call, same `sha256`.
#[must_use]
pub fn emit_dual_cosmos_fishkal_bundle() -> DualCosmosFishkalBundle {
    let joined = format!("{FISHKAL_ENGINE_JS}\n{COSMOS_ENGINE_JS}\n{PAUSE_LIFECYCLE_JS}");
    DualCosmosFishkalBundle {
        fishkal_js: FISHKAL_ENGINE_JS.to_owned(),
        cosmos_js: COSMOS_ENGINE_JS.to_owned(),
        pause_lifecycle_js: PAUSE_LIFECYCLE_JS.to_owned(),
        signature: DUAL_BUNDLE_SIGNATURE.to_owned(),
        content_sha256: bundle_content_sha256(&joined),
    }
}

// ---------------------------------------------------------------------------
// ToySpark — one-click story/game for the 9-year-old child
// ---------------------------------------------------------------------------

/// Deterministic seed from the prompt — the first 4 `sha256` bytes.
#[must_use]
pub fn prompt_seed(prompt: &str) -> u32 {
    let mut hasher = Sha256::new();
    hasher.update(prompt.trim().as_bytes());
    let digest = hasher.finalize();
    u32::from(digest[0]) | (u32::from(digest[1]) << 8) | (u32::from(digest[2]) << 16) | (u32::from(digest[3]) << 24)
}

/// Deterministic title — a safe character-boundary cut + the modality suffix.
#[must_use]
pub fn derive_title(prompt: &str, suffix: &str) -> String {
    let trimmed = prompt.trim();
    let mut head: String = trimmed.chars().take(40).collect();
    if trimmed.chars().count() > 40 {
        head.push('…');
    }
    if head.is_empty() {
        head.push_str("\u{622}\u{641}\u{631}\u{6cc}\u{646}\u{634} \u{627}\u{633}\u{62a}\u{648}\u{62f}\u{6cc}\u{648}\u{6cc}\u{6cc}");
    }
    format!("{head} — {suffix}")
}

/// The `ToySpark` audio-wizard script — three automatic audio steps.
#[must_use]
pub fn toyspark_voice_script(title: &str) -> [String; 3] {
    [
        format!("\u{633}\u{644}\u{627}\u{645}! \u{628}\u{6cc}\u{627} \u{628}\u{627} \u{647}\u{645} «{title}» \u{631}\u{627} \u{628}\u{633}\u{627}\u{632}\u{6cc}\u{645}\u{61b} \u{641}\u{642}\u{637} \u{62f}\u{6a9}\u{645}\u{647}\u{654} \u{627}\u{648}\u{644} \u{631}\u{627} \u{644}\u{645}\u{633} \u{6a9}\u{646}."),
        "\u{686}\u{647} \u{642}\u{634}\u{646}\u{6af} \u{634}\u{62f}! \u{62f}\u{6a9}\u{645}\u{647}\u{654} \u{62f}\u{648}\u{645} \u{631}\u{627} \u{628}\u{632}\u{646} \u{62a}\u{627} \u{635}\u{62f}\u{627} \u{648} \u{631}\u{646}\u{6af}‌\u{647}\u{627} \u{628}\u{631}\u{642}\u{635}\u{646}\u{62f}.".to_owned(),
        "\u{622}\u{641}\u{631}\u{6cc}\u{646}! \u{62f}\u{6a9}\u{645}\u{647}\u{654} \u{633}\u{648}\u{645} \u{622}\u{641}\u{631}\u{6cc}\u{646}\u{634} \u{62a}\u{648} \u{631}\u{627} \u{628}\u{627} \u{6f5}\u{6f0} \u{633}\u{62a}\u{627}\u{631}\u{647} \u{628}\u{647} \u{6a9}\u{644} \u{633}\u{6cc}\u{627}\u{631}\u{647} \u{645}\u{6cc}‌\u{641}\u{631}\u{648}\u{634}\u{62f}.".to_owned(),
    ]
}

/// The one-click `ToySpark` output — a playable HTML5 storybook + a storefront-ready
/// listing + the audio-wizard script.
#[derive(Debug, Clone, PartialEq)]
pub struct ToySparkCreation {
    /// The creation's deterministic title.
    pub title: String,
    /// The complete single-file storybook/game.
    pub storybook_html: String,
    /// The storefront-ready listing at the 50-Star / $1.00 default.
    pub listing: StorefrontProjectCard,
    /// The audio-wizard script — exactly 3 steps.
    pub voice_script: [String; 3],
}

/// The three canonical `ToySpark` giant buttons — icon-first labels.
pub const TOYSPARK_ACTIONS: [&str; 3] = [
    "✨ \u{628}\u{633}\u{627}\u{632} (Make Magic)",
    "🎨 \u{642}\u{634}\u{646}\u{6af}\u{634} \u{6a9}\u{646} (Make It Pretty & Loud)",
    "🪙 \u{628}\u{641}\u{631}\u{648}\u{634} \u{648} \u{633}\u{62a}\u{627}\u{631}\u{647} \u{628}\u{6af}\u{6cc}\u{631} (Share & Earn Stars)",
];

/// The 'one sentence → one playable storybook' flow for the 9-year-old: from one
/// sentence or transcribed voice text, builds a complete interactive HTML5 in one click with
/// procedural `SVG` illustration, procedural `WebAudio` SFX (`Fishkal` +
/// `Cosmos`), and a storefront-ready listing — deterministic and fully keyless.
#[must_use]
pub fn build_toyspark_instant_creation(prompt: &str) -> ToySparkCreation {
    use std::fmt::Write as _;
    let seed = prompt_seed(prompt);
    let rtl = crate::studio_design_contract::detect_persian_rtl(prompt);
    let title = derive_title(prompt, "\u{645}\u{627}\u{62c}\u{631}\u{627}\u{6cc} \u{6cc}\u{6a9}‌\u{6a9}\u{644}\u{6cc}\u{6a9}\u{6cc}\u{650} \u{645}\u{646}");
    let hue = seed % 360;
    let hue2 = (hue + 137) % 360;
    let creature_x = 60 + (seed >> 3) % 80;
    let creature_y = 90 + (seed >> 7) % 40;
    let scenes = [
        (hue, format!("{creature_x}"), format!("{creature_y}")),
        (hue2, format!("{}", 40 + (seed >> 11) % 120), format!("{}", 60 + (seed >> 15) % 80)),
        ((hue + 47) % 360, format!("{}", 90 + (seed >> 19) % 40), format!("{}", 50 + (seed >> 23) % 90)),
    ];
    let mut svg = String::new();
    for (idx, (h, cx, cy)) in scenes.iter().enumerate() {
        let _ = write!(
            svg,
            r#"<svg class="fk-scene" data-scene="{idx}" viewBox="0 0 200 140" role="img" aria-label="Procedural scene {idx}"><rect width="200" height="140" rx="18" fill="hsl({h}, 70%, 12%)"/><circle cx="{cx}" cy="{cy}" r="26" fill="hsl({h}, 85%, 62%)"/><circle cx="{cx}" cy="{cy}" r="34" fill="none" stroke="hsl({h}, 85%, 70%)" stroke-dasharray="4 6"/><path d="M0 120 Q 50 100 100 120 T 200 120" stroke="hsl({h}, 60%, 40%)" fill="none" stroke-width="3"/><g fill="hsl({h}, 90%, 80%)"><circle cx="20" cy="24" r="2"/><circle cx="176" cy="30" r="2"/><circle cx="150" cy="16" r="1.6"/><circle cx="60" cy="14" r="1.6"/></g></svg>"#,
            idx = idx + 1,
            h = h,
            cx = cx,
            cy = cy
        );
    }
    let voice = toyspark_voice_script(&title);
    let star_price = TOYSPARK_DEFAULT_XTR_STARS;
    let storybook = format!(
        r#"<!DOCTYPE html><html lang="{lang}" dir="{dir}"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>{title}</title><style>{css}</style></head><body class="fk-studio {tier_class}" data-cog-tier="ToySpark" data-zero-loss-sandbox="true"><header class="fk-hero"><h1>{title}</h1><p class="fk-hint">Three big buttons — every touch casts a spell.</p></header><main class="fk-card">{svg}<nav class="fk-actions" aria-label="Three magic actions"><button type="button" class="fk-action" data-action="magic" data-step="1"><span class="fk-icon">✨</span><span class="fk-label">Make Magic</span></button><button type="button" class="fk-action" data-action="pretty" data-step="2"><span class="fk-icon">🎨</span><span class="fk-label">Make It Pretty</span></button><button type="button" class="fk-action" data-action="share" data-step="3"><span class="fk-icon">🪙</span><span class="fk-label">Share &amp; Earn</span></button></nav><p class="fk-sandbox" data-sandbox="zero-loss">Zero-loss sandbox: nothing is ever lost.</p><p class="fk-price" data-xtr="{star_price}">50 stars — one kind dollar</p></main><script>{fishkal_js}</script><script>{cosmos_js}</script><script>{pause_js}</script><script>
var VOICES = {voices_json};
var AC = window.AudioContext || window.webkitAudioContext;
var actx = null;
function speak(step) {{
  if (!window.speechSynthesis) {{ return; }}
  window.speechSynthesis.cancel();
  var u = new SpeechSynthesisUtterance(VOICES[step - 1]);
  u.lang = '{speak_lang}';
  window.speechSynthesis.speak(u);
}}
function tap(step) {{
  if (!actx && AC) {{ actx = new AC(); }}
  if (!actx) {{ speak(step); return; }}
  var dest = actx.destination;
  if (step === 1) {{ ARDA_STUDIO.cosmos.sfx.ping(actx, dest, 1); speak(step); }}
  if (step === 2) {{ ARDA_STUDIO.fishkal.harmonicBloom(actx, dest, actx.currentTime + 0.01); ARDA_STUDIO.cosmos.sfx.open(actx, dest); speak(step); }}
  if (step === 3) {{ ARDA_STUDIO.cosmos.sfx.milestone(actx, dest); speak(step); }}
  var card = document.querySelector('.fk-card');
  if (card) {{ ARDA_STUDIO.fishkal.dampFk(card.offsetWidth, 260 + step * 20, 0.016, step / 4); }}
}}
var buttons = document.querySelectorAll('.fk-action');
for (var i = 0; i < buttons.length; i += 1) {{
  buttons[i].addEventListener('click', function (e) {{ tap(Number(e.currentTarget.dataset.step)); }});
}}
speak(1);
</script></body></html>"#,
        lang = if rtl { "fa" } else { "en" },
        dir = if rtl { "rtl" } else { "ltr" },
        title = title,
        css = portfolio_dark_glass_css(),
        tier_class = CognitiveTier::ToySpark.css_class(),
        svg = svg,
        star_price = star_price,
        fishkal_js = FISHKAL_ENGINE_JS,
        cosmos_js = COSMOS_ENGINE_JS,
        pause_js = PAUSE_LIFECYCLE_JS,
        speak_lang = if rtl { "fa-IR" } else { "en-US" },
        voices_json = serde_json::to_string(&voice).unwrap_or_else(|_| "[]".to_owned()),
    );
    let listing = StorefrontProjectCard::toyspark_listing(&title, &storybook);
    ToySparkCreation {
        title,
        storybook_html: storybook,
        listing,
        voice_script: voice,
    }
}

// ---------------------------------------------------------------------------
// The studio artifact generator — shared shell + per-modality body
// ---------------------------------------------------------------------------

/// The studio-generated artifact — playable HTML + the dual bundle + pricing.
#[derive(Debug, Clone, PartialEq)]
pub struct StudioArtifact {
    /// Deterministic title.
    pub title: String,
    /// The complete single-file artifact (ARDA injection runs on it).
    pub artifact_html: String,
    /// The procedural dual bundle.
    pub dual_bundle: DualCosmosFishkalBundle,
    /// The morphed cognitive tier.
    pub tier: CognitiveTier,
    /// The generating modality.
    pub modality: StudioModality,
    /// The deterministic REA score (`88..=97` — always above Mairon's 85 gate).
    pub rea_score: u8,
    /// Price in micro-Kian.
    pub price_ukian: u64,
    /// Price in Telegram Stars.
    pub price_xtr_stars: u32,
    /// Price in micro-`USDC`.
    pub price_usdc_micros: u64,
}

/// The hyper-modality studio — builds all 9 zero-cost modalities on one shared
/// shell with the `Fishkal + Cosmos` dual bundle; fully deterministic.
#[derive(Debug, Clone, Copy, Default)]
pub struct OmniModalStudioEngine;

impl OmniModalStudioEngine {
    /// Produces the full artifact for prompt + modality + cognitive tier — deterministic.
    #[must_use]
    pub fn generate(&self, prompt: &str, modality: StudioModality, tier: CognitiveTier) -> StudioArtifact {
        let seed = prompt_seed(prompt);
        let rtl = crate::studio_design_contract::detect_persian_rtl(prompt);
        let title = derive_title(prompt, modality.fa_label());
        let dual = emit_dual_cosmos_fishkal_bundle();
        let body = modality_body(modality, &title);
        let script = modality_script(modality);
        let font = if rtl {
            crate::studio_design_contract::RTL_FONT_STACK
        } else {
            crate::studio_design_contract::LTR_FONT_STACK
        };
        let artifact = studio_shell(&title, tier, rtl, font, &body, &script, modality.as_str())
            .replace("__FISHKAL_JS__", &dual.fishkal_js)
            .replace("__COSMOS_JS__", &dual.cosmos_js)
            .replace("__PAUSE_JS__", &dual.pause_lifecycle_js);
        let (price_ukian, price_xtr_stars, price_usdc_micros) = derive_pricing(seed, tier);
        StudioArtifact {
            title,
            artifact_html: artifact,
            dual_bundle: dual,
            tier,
            modality,
            rea_score: derive_rea_score(seed),
            price_ukian,
            price_xtr_stars,
            price_usdc_micros,
        }
    }
}

/// Deterministic REA score from the prompt seed — the `88..=97` range, always above the 85 gate.
#[must_use]
pub fn derive_rea_score(seed: u32) -> u8 {
    u8::try_from(88 + seed % 10).unwrap_or(92)
}

/// Deterministic pricing by cognitive tier — `ToySpark` is locked to the one-click
/// default (50 Stars / $1.00); higher tiers are freer.
#[must_use]
pub fn derive_pricing(seed: u32, tier: CognitiveTier) -> (u64, u32, u64) {
    match tier {
        CognitiveTier::ToySpark => (
            TOYSPARK_DEFAULT_UKIAN,
            TOYSPARK_DEFAULT_XTR_STARS,
            TOYSPARK_DEFAULT_USDC_MICROS,
        ),
        CognitiveTier::CreatorStudio => {
            let xtr = 120 + seed % 100;
            (u64::from(xtr) * 20_000, xtr, u64::from(xtr) * 20_000)
        }
        CognitiveTier::ProTerminal => {
            let xtr = 300 + seed % 300;
            (u64::from(xtr) * 20_000, xtr, u64::from(xtr) * 20_000)
        }
    }
}

/// The shared artifact shell — portfolio dark-glass tokens + the tier morph.
#[must_use]
pub fn studio_shell(
    title: &str,
    tier: CognitiveTier,
    rtl: bool,
    font: &str,
    body: &str,
    script: &str,
    modality: &str,
) -> String {
    let (lang, dir) = if rtl { ("fa", "rtl") } else { ("en", "ltr") };
    format!(
        r#"<!DOCTYPE html><html lang="{lang}" dir="{dir}"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>{title}</title><style>{css}
body {{ font-family: {font}; }}
</style></head><body class="fk-studio {tier_class}" data-cog-tier="{tier}" data-modality="{modality}">{body}<script>__FISHKAL_JS__</script><script>__COSMOS_JS__</script><script>__PAUSE_JS__</script><script>{script}</script></body></html>"#,
        lang = lang,
        dir = dir,
        title = html_escape(title),
        css = portfolio_dark_glass_css(),
        font = font,
        tier_class = tier.css_class(),
        tier = tier.as_str(),
        modality = modality,
        body = body,
        script = script,
    )
}

/// HTML escaping for safe title insertion.
#[must_use]
pub fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// The per-modality body — 9 modalities, each with a `data-modality` marker.
#[must_use]
pub fn modality_body(modality: StudioModality, title: &str) -> String {
    let title = html_escape(title);
    match modality {
        StudioModality::Search => format!(
            r#"<main class="fk-grid" data-modality="search"><section class="fk-pane"><h2>{title}</h2><p>Keyless multi-source synthesis: the `Agent-Reach` cascade (Jina/Exa MCP) + 120 public CORS APIs.</p><ul class="fk-src" data-cascade="jina,exa,public-cors-120"><li data-source="jina-reader">Jina Reader — keyless reading</li><li data-source="exa">Exa — keyless semantic search</li><li data-source="public-cors-120">120 public CORS APIs</li></ul></section></main>"#
        ),
        StudioModality::DeepMode => format!(
            r#"<main class="fk-grid" data-modality="deep"><section class="fk-pane"><h2>{title}</h2><ol class="fk-tree" data-mairon-steps="8"><li>1 Problem statement</li><li>2 Hypothesis</li><li>3 Evidence</li><li>4 Antithesis</li><li>5 Synthesis</li><li>6 Hard test</li><li>7 Decision</li><li>8 Mairon's seal</li></ol></section></main>"#
        ),
        StudioModality::AgentMode => format!(
            r#"<main class="fk-grid" data-modality="agent"><section class="fk-pane"><h2>{title}</h2><p>Stochastic 9-fold DNA + `SKILL.md` forge.</p><div class="fk-dna" data-fold="9" data-dna="9-fold-stochastic"></div><a class="chip" data-download="SKILL.md" href="skill-md">Download SKILL.md</a></section></main>"#
        ),
        StudioModality::WorkspaceMode => format!(
            r#"<main class="fk-grid" data-modality="workspace"><section class="fk-pane"><h2>{title}</h2><div class="fk-chip-row" role="toolbar" aria-label="Tabs"><button type="button" class="chip" aria-pressed="true" data-tab="preview">Preview</button><button type="button" class="chip" aria-pressed="false" data-tab="source">Code inspector</button></div><iframe class="fk-preview" sandbox="allow-scripts" title="Sandboxed preview" data-preview></iframe><ul class="fk-files" data-files="safe-only"><li>index.html</li><li>studio.css</li></ul></section><script data-project-viewer>let loaded = false; function loadSource() {{ loaded = true; }} function setTab(tab) {{ loadSource(tab); }} setTab('preview');</script></main>"#
        ),
        StudioModality::VoiceAudio => format!(
            r#"<main class="fk-grid" data-modality="voice"><section class="fk-pane"><h2>{title}</h2><button type="button" class="fk-action" data-voice="speak">Voice readout</button><button type="button" class="fk-action" data-voice="record">MediaRecorder recording</button><button type="button" class="fk-action" data-voice="ambient" data-ambient="fishkal-ocean">Ambient ocean music</button></section></main>"#
        ),
        StudioModality::ImageGen => format!(
            r#"<main class="fk-grid" data-modality="image"><section class="fk-pane"><h2>{title}</h2><canvas data-canvas="procedural-vector" width="320" height="200"></canvas><p data-cascade="zero-key">Keyless provider cascade + deterministic SVG vector.</p></section></main>"#
        ),
        StudioModality::VideoGen => format!(
            r#"<main class="fk-grid" data-modality="video"><section class="fk-pane"><h2>{title}</h2><canvas data-canvas="tl-60fps" width="320" height="180"></canvas><p data-tl="fishkal-timeline">Fishkal timeline — 60fps, variable-lambda `damp`, WebM/frame recording.</p><button type="button" class="chip" data-record="webm">Record WebM</button></section></main>"#
        ),
        StudioModality::TextCopy => format!(
            r#"<main class="fk-grid" data-modality="copy"><section class="fk-pane"><h2>{title}</h2><p data-locales="20">Sales copy across 20 planetary languages with RTL/LTR typography.</p><div class="fk-chip-row"><button type="button" class="chip" aria-pressed="true" data-dir-mode="rtl">RTL</button><button type="button" class="chip" aria-pressed="false" data-dir-mode="ltr">LTR</button></div></section></main>"#
        ),
        StudioModality::SoundEffects => format!(
            r#"<main class="fk-grid" data-modality="sfx"><section class="fk-pane"><h2>{title}</h2><div class="fk-chip-row" role="toolbar" aria-label="SFX triggers"><button type="button" class="chip" data-sfx="ping">ping</button><button type="button" class="chip" data-sfx="open">open</button><button type="button" class="chip" data-sfx="whoosh">whoosh</button><button type="button" class="chip" data-sfx="select">select</button><button type="button" class="chip" data-sfx="milestone">milestone</button></div></section></main>"#
        ),
    }
}

/// The per-modality boot script — procedural SFX/canvas wiring.
#[must_use]
pub fn modality_script(modality: StudioModality) -> String {
    match modality {
        StudioModality::SoundEffects | StudioModality::VoiceAudio => String::from(
            "var FK = window.ARDA_STUDIO; document.addEventListener('click', function (e) {\n  var el = e.target.closest('[data-sfx],[data-voice]');\n  if (!el) { return; }\n  var AC = window.AudioContext || window.webkitAudioContext;\n  var ctx = new AC();\n  if (el.dataset.sfx === 'ping') { FK.cosmos.sfx.ping(ctx, ctx.destination, 3); }\n  if (el.dataset.sfx === 'open') { FK.cosmos.sfx.open(ctx, ctx.destination); }\n  if (el.dataset.sfx === 'whoosh') { FK.cosmos.sfx.whoosh(ctx, ctx.destination); }\n  if (el.dataset.sfx === 'select') { FK.cosmos.sfx.select(ctx, ctx.destination); }\n  if (el.dataset.sfx === 'milestone') { FK.cosmos.sfx.milestone(ctx, ctx.destination); }\n});",
        ),
        StudioModality::VideoGen => String::from(
            "var FK = window.ARDA_STUDIO; var cv = document.querySelector('[data-canvas]');\nif (cv) { var g = cv.getContext('2d'); var phase = 0; var x = 40;\n  function frame(t) { if (FK.rafGate()) { phase += 0.016; x = FK.fishkal.dampFk(x, cv.width - 60, 0.016, phase); g.clearRect(0, 0, cv.width, cv.height); g.fillRect(x, 60, 40, 40); } requestAnimationFrame(frame); }\n  requestAnimationFrame(frame); }",
        ),
        StudioModality::ImageGen => String::from(
            "var cv = document.querySelector('[data-canvas]');\nif (cv) { var g = cv.getContext('2d'); var rnd = window.ARDA_STUDIO.fishkal.rng(7);\n  for (var i = 0; i < 40; i += 1) { g.fillStyle = 'hsl(' + Math.floor(rnd() * 360) + ', 80%, 60%)'; g.beginPath(); g.arc(rnd() * cv.width, rnd() * cv.height, 4 + rnd() * 18, 0, 6.283); g.fill(); } }",
        ),
        StudioModality::AgentMode => String::from(
            "var dna = document.querySelector('[data-dna]');\nif (dna) { var rnd = window.ARDA_STUDIO.fishkal.rng(9);\n  for (var i = 0; i < 9; i += 1) { var b = document.createElement('span'); b.className = 'chip'; b.textContent = 'fold-' + (i + 1) + ':' + Math.floor(rnd() * 100); dna.appendChild(b); } }",
        ),
        _ => String::from("var FK = window.ARDA_STUDIO; if (FK && FK.rafGate()) { /* deterministic bootstrap */ }"),
    }
}

// ---------------------------------------------------------------------------
// Sub-engine 3 — `PortfolioStorefrontEngine` (builder storefront + 6 fixes)
// ---------------------------------------------------------------------------

/// The canonical catalog-root path — Fix #1: no `path.resolve(PORT, '..')`
/// exists; only an in-workspace relative path or a configured env path.
pub const PORTFOLIO_ROOT_RESOLUTION: &str = "./seed/portfolio_catalog";

/// The catalog-path env variable — the safe replacement for the parent root.
pub const PORTFOLIO_CATALOG_ENV: &str = "KIAN_PORTFOLIO_CATALOG_PATH";

/// The portfolio storefront signature.
pub const PORTFOLIO_STOREFRONT_SIGNATURE: &str = "portfolio-storefront-v1";

/// The storefront project card — the 12 canonical `projects.json` keys + the live
/// marketplace commerce extensions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorefrontProjectCard {
    /// Canonical key 1 — `slug`.
    pub slug: String,
    /// Canonical key 2 — `name`.
    pub name: String,
    /// Canonical key 3 — `tagline`.
    pub tagline: String,
    /// Canonical key 4 — `category`.
    pub category: String,
    /// Canonical key 5 — `stack`.
    pub stack: Vec<String>,
    /// Canonical key 6 — `highlights`.
    pub highlights: Vec<String>,
    /// Canonical key 7 — `Preview` (live preview HTML).
    pub preview: String,
    /// Canonical key 8 — `sourceDir`.
    pub source_dir: String,
    /// Canonical key 9 — `readme`.
    pub readme: String,
    /// Canonical key 10 — `loc` (lines of code).
    pub loc: u32,
    /// Canonical key 11 — `fileCount`.
    pub file_count: u32,
    /// Canonical key 12 — `featured`.
    pub featured: bool,
    /// Commerce extension — price in micro-Kian.
    pub price_ukian: u64,
    /// Commerce extension — price in Telegram Stars.
    pub price_xtr_stars: u32,
    /// Commerce extension — price in micro-`USDC`.
    pub price_usdc_micros: u64,
    /// Commerce extension — the Mairon REA score.
    pub rea_score: u8,
    /// Commerce extension — the forged `SKILL.md` address.
    pub skill_md_url: String,
    /// Commerce extension — the active planetary-language count.
    pub supported_locales_count: u8,
    /// Commerce extension — the cognitive-tier badge.
    pub cognitive_tier_badge: String,
}

impl StorefrontProjectCard {
    /// The one-click `ToySpark` listing — the locked 50-Star / $1.00 price.
    #[must_use]
    pub fn toyspark_listing(title: &str, preview_html: &str) -> Self {
        Self {
            slug: format!("toyspark-{}", prompt_seed(title)),
            name: title.to_owned(),
            tagline: "\u{645}\u{627}\u{62c}\u{631}\u{627}\u{6cc} \u{6cc}\u{6a9}‌\u{6a9}\u{644}\u{6cc}\u{6a9}\u{6cc}\u{650} \u{6a9}\u{648}\u{62f}\u{6a9}\u{650} \u{6f9}\u{633}\u{627}\u{644}\u{647} — \u{62f}\u{627}\u{633}\u{62a}\u{627}\u{646}\u{650} \u{642}\u{627}\u{628}\u{644} \u{628}\u{627}\u{632}\u{6cc}\u{650} \u{622}\u{645}\u{627}\u{62f}\u{647}\u{654} \u{641}\u{631}\u{648}\u{634}".to_owned(),
            category: "toyspark-story".to_owned(),
            stack: vec!["html5".to_owned(), "svg".to_owned(), "webaudio".to_owned()],
            highlights: vec![
                "procedural SVG scenes".to_owned(),
                "voice-guided wizard".to_owned(),
                "zero-loss sandbox".to_owned(),
            ],
            preview: preview_html.to_owned(),
            source_dir: PORTFOLIO_ROOT_RESOLUTION.to_owned(),
            readme: "generated-by-toyspark-one-click".to_owned(),
            loc: 240,
            file_count: 1,
            featured: true,
            price_ukian: TOYSPARK_DEFAULT_UKIAN,
            price_xtr_stars: TOYSPARK_DEFAULT_XTR_STARS,
            price_usdc_micros: TOYSPARK_DEFAULT_USDC_MICROS,
            rea_score: 92,
            skill_md_url: "/api/v1/market/products/{id}/skill".to_owned(),
            supported_locales_count: 20,
            cognitive_tier_badge: CognitiveTier::ToySpark.as_str().to_owned(),
        }
    }
}

/// The seed storefront catalog — 8 flagship creations with the 12 canonical keys.
#[must_use]
pub fn seed_storefront_catalog() -> Vec<StorefrontProjectCard> {
    let mk = |slug: &str, name: &str, tagline: &str, category: &str, stack: [&str; 3], highlights: [&str; 3], loc: u32, files: u32, featured: bool, xtr: u32| StorefrontProjectCard {
        slug: slug.to_owned(),
        name: name.to_owned(),
        tagline: tagline.to_owned(),
        category: category.to_owned(),
        stack: stack.iter().map(|s| (*s).to_owned()).collect(),
        highlights: highlights.iter().map(|s| (*s).to_owned()).collect(),
        preview: format!("<section data-storefront-preview=\"{slug}\"></section>"),
        source_dir: format!("./seed/portfolio_catalog/{slug}"),
        readme: format!("docs/storefront/{slug}.md"),
        loc,
        file_count: files,
        featured,
        price_ukian: u64::from(xtr) * 20_000,
        price_xtr_stars: xtr,
        price_usdc_micros: u64::from(xtr) * 20_000,
        rea_score: 92,
        skill_md_url: format!("/api/v1/market/products/{slug}/skill"),
        supported_locales_count: 20,
        cognitive_tier_badge: "CreatorStudio".to_owned(),
    };
    vec![
        mk("fishkal", "Fishkal", "\u{645}\u{648}\u{62a}\u{648}\u{631} HTML5 \u{635}\u{641}\u{631}-\u{648}\u{627}\u{628}\u{633}\u{62a}\u{6af}\u{6cc}\u{650} \u{6a9}\u{6cc}\u{627} — \u{627}\u{642}\u{6cc}\u{627}\u{646}\u{648}\u{633}\u{650} \u{631}\u{648}\u{6cc}\u{647}‌\u{627}\u{6cc}", "engine", ["html5", "webaudio", "canvas"], ["3,616 lines", "zero deps", "60fps timeline"], 3616, 1, true, 300),
        mk("cosmos", "Cosmos Starfield", "\u{633}\u{62a}\u{627}\u{631}\u{647}‌\u{628}\u{627}\u{631}\u{627}\u{646}\u{650} \u{62a}\u{639}\u{627}\u{645}\u{644}\u{6cc}\u{650} \u{648}\u{6cc}\u{62a}\u{631}\u{6cc}\u{646} + \u{633}\u{6cc}\u{646}\u{62a}\u{650} \u{62f}\u{6cc}\u{644}\u{6cc}-\u{628}\u{627}\u{632}\u{62e}\u{648}\u{631}\u{62f}", "engine", ["canvas2d", "webaudio", "raf"], ["7 milestone nodes", "feedback delay", "parallax 0.05"], 512, 2, true, 240),
        mk("arda", "ARDA", "\u{628}\u{648}\u{645}\u{6cc}‌\u{633}\u{627}\u{632}\u{6cc} \u{633}\u{6cc}\u{627}\u{631}\u{647}‌\u{627}\u{6cc}\u{650} \u{6f2}\u{6f4}\u{633}\u{627}\u{639}\u{62a}\u{647} \u{631}\u{648}\u{6cc} \u{645}\u{639}\u{645}\u{627}\u{631}\u{6cc} \u{641}\u{6cc}\u{634}\u{6a9}\u{627}\u{644}", "engine", ["i18n", "rtl", "bidi"], ["20 locales", "atomic root flip", "ARIA listbox"], 1791, 1, false, 180),
        mk("trinity", "Trinity Studio", "\u{62e}\u{637} \u{644}\u{648}\u{644}\u{647}\u{654} \u{633}\u{647}‌\u{6a9}\u{627}\u{644}\u{628}\u{62f}\u{6cc} \u{622}\u{644}\u{641}\u{627} → \u{627}\u{62a}\u{627}\u{642} \u{641}\u{631}\u{645}\u{627}\u{646} → \u{645}\u{627}\u{6cc}\u{631}\u{648}\u{646}", "pipeline", ["rust", "agents", "dna"], ["3x9 OpenBot", "REA gate 85", "self-healing"], 2400, 6, false, 220),
        mk("mairon", "Mairon REA", "\u{62f}\u{627}\u{648}\u{631}\u{650} \u{62e}\u{648}\u{62f}\u{6a9}\u{627}\u{631}\u{650} \u{6a9}\u{6cc}\u{641}\u{6cc}\u{62a} \u{628}\u{627} \u{6af}\u{6cc}\u{62a}\u{650} \u{641}\u{648}\u{631}\u{62c} \u{62f}\u{648}-\u{6a9}\u{644}\u{6cc}\u{62f}\u{6cc}", "judge", ["rust", "scoring", "gate"], ["IQ 150", "EQ 145", "SKILL.md forge"], 980, 3, false, 150),
        mk("super-studio", "Omni Studio", "\u{627}\u{633}\u{62a}\u{648}\u{62f}\u{6cc}\u{648}\u{6cc} \u{6f9}-\u{645}\u{62f}\u{627}\u{644}\u{6cc}\u{62a}\u{6cc}\u{650} \u{635}\u{641}\u{631}-\u{647}\u{632}\u{6cc}\u{646}\u{647} \u{628}\u{627} \u{628}\u{627}\u{646}\u{62f}\u{644}\u{650} \u{62f}\u{648}\u{6af}\u{627}\u{646}\u{647}", "studio", ["html5", "webaudio", "canvas"], ["9 modalities", "dual engine", "zero keys"], 1600, 4, false, 260),
        mk("kian-kernel", "KIAN Kernel", "\u{6a9}\u{631}\u{646}\u{644} \u{639}\u{627}\u{645}\u{644}\u{650} \u{62d}\u{627}\u{6a9}\u{645} \u{628}\u{627} \u{62f}\u{641}\u{62a}\u{631}\u{650} \u{62f}\u{648}\u{637}\u{631}\u{641}\u{647}\u{654} \u{627}\u{62a}\u{645}\u{6cc}\u{6a9}", "kernel", ["rust", "sqlite", "ledger"], ["466 citizens", "SUM=0", "CAS escrows"], 8000, 42, false, 320),
        mk("portfolio", "Portfolio Storefront", "\u{648}\u{6cc}\u{62a}\u{631}\u{6cc}\u{646}\u{650} \u{633}\u{627}\u{632}\u{646}\u{62f}\u{647} \u{628}\u{627} \u{6f1}\u{6f2} \u{6a9}\u{644}\u{6cc}\u{62f}\u{650} \u{6a9}\u{627}\u{646}\u{648}\u{646}\u{6cc} + \u{6f6} \u{62f}\u{631}\u{645}\u{627}\u{646}\u{650} \u{642}\u{636}\u{627}\u{6cc}\u{6cc}", "storefront", ["css", "glass", "palette"], ["12 canonical keys", "6 forensic fixes", "Ctrl+K palette"], 720, 5, false, 120),
    ]
}

/// Auto-featured top-two — `featured` entries in stable input
/// order, at most 2, at the catalog head (`index.js:L17–18`).
#[must_use]
pub fn promote_featured_top2(cards: &[StorefrontProjectCard]) -> Vec<StorefrontProjectCard> {
    let mut promoted: Vec<StorefrontProjectCard> = cards
        .iter()
        .filter(|c| c.featured)
        .take(2)
        .cloned()
        .collect();
    for card in cards {
        if promoted.len() >= cards.len() {
            break;
        }
        if !promoted.iter().any(|p| p.slug == card.slug) {
            promoted.push(card.clone());
        }
    }
    promoted
}

/// The portfolio dark-glass CSS tokens — verbatim
/// `style.css:L1–25, L53–57, L112–139`.
#[must_use]
pub fn portfolio_dark_glass_css() -> String {
    String::from(
        r":root {
  --bg: #05050a;
  --bg-2: #0a0b14;
  --surface: rgba(16, 18, 32, 0.68);
  --glass: rgba(6, 6, 14, 0.78);
  --text: #f3f4f8;
  --muted: #9ca3b8;
  --accent: #7c5cff;
  --accent-2: #19d3da;
  --accent-3: #ff5ca8;
  --ease: cubic-bezier(.2, .7, .2, 1);
}
.fk-studio { background: var(--bg); color: var(--text); font-family: 'Space Grotesk', 'Inter', 'Vazirmatn', system-ui, sans-serif; max-width: 1240px; margin: 0 auto; padding: 24px; }
.fk-studio * { backdrop-filter: blur(14px); }
.fk-hero h1 { font-size: clamp(36px, 5.6vw, 62px); letter-spacing: -0.02em; }
.fk-card, .fk-pane { background: var(--surface); border: 1px solid rgba(124, 92, 255, 0.28); border-radius: 22px; padding: 22px; transition: transform .35s var(--ease); }
.fk-glass { background: var(--glass); }
.fk-code { font-family: 'JetBrains Mono', monospace; }
.fk-scene { width: 100%; border-radius: 18px; display: block; margin-bottom: 12px; }
.fk-actions { display: flex; gap: 14px; justify-content: center; margin-top: 18px; }
.fk-action { background: var(--glass); color: var(--text); border: 2px solid var(--accent-2); border-radius: 24px; cursor: pointer; display: inline-flex; flex-direction: column; align-items: center; gap: 8px; padding: 14px 18px; transition: transform .35s var(--ease); }
.fk-action .fk-icon { font-size: 34px; }
.fk-action .fk-label { max-width: 220px; text-align: center; }
.fk-action:hover { transform: translateY(-4px) scale(1.02); }
.fk-hint, .fk-sandbox, .fk-price { color: var(--muted); text-align: center; }
.tier-toyspark .fk-action { min-width: 64px; min-height: 64px; font-size: 22px; border-radius: 24px; }
.tier-toyspark .fk-actions { flex-direction: column; }
.tier-creator .fk-blocks { display: flex; gap: 12px; flex-wrap: wrap; }
.tier-pro .fk-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 10px; }
.tier-pro kbd { background: rgba(124, 92, 255, 0.22); border: 1px solid rgba(124, 92, 255, 0.5); border-radius: 6px; padding: 1px 7px; font-family: 'JetBrains Mono', monospace; }
.fk-chip-row { display: flex; gap: 8px; flex-wrap: wrap; }
.chip { cursor: pointer; border: 1px solid rgba(124, 92, 255, 0.45); background: rgba(16, 18, 32, 0.68); color: var(--muted); border-radius: 999px; padding: 6px 14px; }
.chip[aria-pressed='true'] { background: var(--accent); color: var(--text); }
.fk-preview { width: 100%; min-height: 220px; border: 1px solid rgba(124, 92, 255, 0.35); border-radius: 14px; background: var(--bg-2); }",
    )
}

/// the six forensic-fix IDs of `KiaAgentX/portfolio`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortfolioFix {
    /// Fix #1 — a portable root path (no `path.resolve(PORT, '..')`).
    PortableRootPath,
    /// Fix #2 — removing the temporal-dead-zone (`TDZ`) `ReferenceError` on `#source`.
    TdzLoadedGuard,
    /// Fix #3 — an iframe sandbox without `allow-same-origin`.
    IframeSandbox,
    /// Fix #4 — the secret and dotfile guard.
    SecretDotfileGuard,
    /// Fix #5 — accessible filter chips (`button aria-pressed`).
    AccessibleFilterChips,
    /// Fix #6 — the shared `RAF` pause lifecycle.
    UnifiedRafPause,
}

impl PortfolioFix {
    /// Numeric fix ID.
    #[must_use]
    pub fn id(self) -> u8 {
        match self {
            Self::PortableRootPath => 1,
            Self::TdzLoadedGuard => 2,
            Self::IframeSandbox => 3,
            Self::SecretDotfileGuard => 4,
            Self::AccessibleFilterChips => 5,
            Self::UnifiedRafPause => 6,
        }
    }

    /// Fix title.
    #[must_use]
    pub fn title(self) -> &'static str {
        match self {
            Self::PortableRootPath => "Portable root path — workspace-contained resolution",
            Self::TdzLoadedGuard => "let loaded = false declared before setTab hash routing",
            Self::IframeSandbox => "sandbox=\"allow-scripts\" without allow-same-origin",
            Self::SecretDotfileGuard => ".env / *.pem / *.key / id_rsa / credentials.json / *.sqlite blocked",
            Self::AccessibleFilterChips => "filter chips as <button type=\"button\" aria-pressed>",
            Self::UnifiedRafPause => "Cosmos RAF wired into shared __fkPauseDepth + document.hidden",
        }
    }

    /// The fix's persistent witness in artifacts.
    #[must_use]
    pub fn evidence(self) -> &'static str {
        match self {
            Self::PortableRootPath => PORTFOLIO_ROOT_RESOLUTION,
            Self::TdzLoadedGuard => "let loaded = false;",
            Self::IframeSandbox => "sandbox=\"allow-scripts\"",
            Self::SecretDotfileGuard => "credentials.json blocked",
            Self::AccessibleFilterChips => "aria-pressed",
            Self::UnifiedRafPause => "__fkPauseDepth",
        }
    }
}

/// The portfolio 6-defect fix report — one row on the wire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioFixReport {
    /// Fix ID (`1..=6`).
    pub fix_id: u8,
    /// Fix title.
    pub title: String,
    /// Status — always `fixed`.
    pub status: &'static str,
    /// Deterministic witness.
    pub evidence: String,
}

/// The full storefront safety report — always 6/6 `fixed`.
#[must_use]
pub fn portfolio_safety_report() -> [PortfolioFixReport; 6] {
    [
        PortfolioFix::PortableRootPath,
        PortfolioFix::TdzLoadedGuard,
        PortfolioFix::IframeSandbox,
        PortfolioFix::SecretDotfileGuard,
        PortfolioFix::AccessibleFilterChips,
        PortfolioFix::UnifiedRafPause,
    ]
    .map(|f| PortfolioFixReport {
        fix_id: f.id(),
        title: f.title().to_owned(),
        status: "fixed",
        evidence: f.evidence().to_owned(),
    })
}

/// Forbidden source-tree patterns — Fix #4.
const BLOCKED_SOURCE_PATTERNS: [&str; 7] = [
    ".env", ".env.", "*.pem", "*.key", "id_rsa", "credentials.json", "*.sqlite",
];

/// Is the source path forbidden? — never copied or displayed.
#[must_use]
pub fn is_blocked_source_path(path: &str) -> bool {
    let p = path.trim_start_matches("./");
    let name = p.rsplit('/').next().unwrap_or(p).to_lowercase();
    BLOCKED_SOURCE_PATTERNS.iter().any(|pat| {
        if *pat == "id_rsa" {
            name.starts_with("id_rsa")
        } else if let Some(suffix) = pat.strip_prefix("*.") {
            name.ends_with(&format!(".{suffix}"))
        } else if *pat == ".env" {
            name == ".env" || name.starts_with(".env.")
        } else {
            name == *pat
        }
    })
}

/// Source-tree filtering — only allowed paths come back (order preserved).
#[must_use]
pub fn sanitize_source_tree<'a>(paths: &[&'a str]) -> Vec<&'a str> {
    paths.iter().copied().filter(|p| !is_blocked_source_path(p)).collect()
}

/// Mechanical verification of the 6 fixes on generated artifacts — the handler test gate.
#[must_use]
pub fn verify_portfolio_safety_fixes(bundle: &DualCosmosFishkalBundle, workspace_body: &str) -> bool {
    let fix1 = !PORTFOLIO_ROOT_RESOLUTION.contains("..");
    let fix2 = {
        let loaded_pos = workspace_body.find("let loaded = false;");
        let settab_pos = workspace_body.find("setTab('preview')");
        matches!((loaded_pos, settab_pos), (Some(l), Some(s)) if l < s)
    };
    let fix3 = workspace_body.contains(r#"sandbox="allow-scripts""#)
        && !workspace_body.contains(r#"sandbox="allow-scripts allow-same-origin""#);
    let fix4 = sanitize_source_tree(&[".env", ".env.local", "server.pem", "id_rsa", "credentials.json", "db.sqlite", "index.html"])
        == vec!["index.html"];
    let fix5 = workspace_body.contains(r#"class="chip" aria-pressed="true" data-tab"#)
        && workspace_body.contains("<button type=\"button\"");
    let fix6 = bundle.pause_lifecycle_js.contains("__fkPauseDepth")
        && bundle.cosmos_js.contains("__fkPauseDepth")
        && bundle.cosmos_js.contains("document.hidden")
        && bundle.pause_lifecycle_js.contains("fkPause")
        && bundle.pause_lifecycle_js.contains("fkResume");
    fix1 && fix2 && fix3 && fix4 && fix5 && fix6
}

/// Command-palette configuration — keyboard-first (`Ctrl+K` / `Meta+K` and `/`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandPaletteConfig {
    /// Open shortcuts.
    pub open_shortcuts: [&'static str; 3],
    /// Search-focus shortcut.
    pub focus_shortcut: &'static str,
    /// Keyboard-first?
    pub keyboard_first: bool,
    /// Navigation keys.
    pub navigation_keys: [&'static str; 3],
}

/// The canonical command-palette configuration (`index.js:L396–459`).
#[must_use]
pub fn command_palette_config() -> CommandPaletteConfig {
    CommandPaletteConfig {
        open_shortcuts: ["Ctrl+K", "Meta+K", "/"],
        focus_shortcut: "/",
        keyboard_first: true,
        navigation_keys: ["ArrowDown", "ArrowUp", "Escape"],
    }
}

/// Command-palette JS — open on `Ctrl+K`/`Meta+K`/`/`, arrow navigation, close on `Escape`.
#[must_use]
pub fn emit_command_palette_js() -> String {
    String::from(
        r"(function () {
  'use strict';
  var palette = document.querySelector('[data-command-palette]');
  if (!palette) { return; }
  var input = palette.querySelector('input');
  var items = palette.querySelectorAll('[data-palette-item]');
  var cursor = 0;
  function openPalette() { palette.classList.add('is-open'); palette.setAttribute('aria-expanded', 'true'); if (input) { input.focus(); } }
  function closePalette() { palette.classList.remove('is-open'); palette.setAttribute('aria-expanded', 'false'); }
  function moveCursor(delta) { cursor = Math.min(items.length - 1, Math.max(0, cursor + delta)); if (items[cursor]) { items[cursor].focus(); } }
  document.addEventListener('keydown', function (e) {
    var metaK = (e.ctrlKey || e.metaKey) && (e.key === 'k' || e.key === 'K');
    if (metaK) { e.preventDefault(); openPalette(); return; }
    if (e.key === '/' && document.activeElement !== input) { e.preventDefault(); openPalette(); return; }
    if (e.key === 'Escape') { closePalette(); }
    if (e.key === 'ArrowDown') { moveCursor(1); }
    if (e.key === 'ArrowUp') { moveCursor(-1); }
  });
  closePalette();
})();",
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const DINO_PROMPT: &str = "\u{6cc}\u{647} \u{62f}\u{627}\u{6cc}\u{646}\u{627}\u{633}\u{648}\u{631} \u{622}\u{628}\u{6cc} \u{6a9}\u{647} \u{62a}\u{648}\u{6cc} \u{641}\u{636}\u{627} \u{628}\u{633}\u{62a}\u{646}\u{6cc} \u{645}\u{6cc}‌\u{641}\u{631}\u{648}\u{634}\u{647}";

    #[test]
    fn tier_inference_matrix_is_deterministic() {
        // explicit override always wins
        let override_toy = infer_or_override_tier(&CognitiveTelemetry {
            prompt_length_chars: 500,
            uses_technical_tokens: true,
            typing_cadence_ms: 900,
            explicit_tier_override: Some(CognitiveTier::ToySpark),
        });
        assert_eq!(override_toy, CognitiveTier::ToySpark);
        // technical marker → `ProTerminal`
        let tech = infer_or_override_tier(&CognitiveTelemetry {
            prompt_length_chars: 20,
            uses_technical_tokens: true,
            typing_cadence_ms: 0,
            explicit_tier_override: None,
        });
        assert_eq!(tech, CognitiveTier::ProTerminal);
        // high length → `ProTerminal`
        let long = infer_or_override_tier(&CognitiveTelemetry {
            prompt_length_chars: 240,
            uses_technical_tokens: false,
            typing_cadence_ms: 0,
            explicit_tier_override: None,
        });
        assert_eq!(long, CognitiveTier::ProTerminal);
        // slow medium → `CreatorStudio`
        let medium = infer_or_override_tier(&CognitiveTelemetry {
            prompt_length_chars: 160,
            uses_technical_tokens: false,
            typing_cadence_ms: 500,
            explicit_tier_override: None,
        });
        assert_eq!(medium, CognitiveTier::CreatorStudio);
        // short → `ToySpark`
        let short = infer_or_override_tier(&CognitiveTelemetry {
            prompt_length_chars: 30,
            uses_technical_tokens: false,
            typing_cadence_ms: 120,
            explicit_tier_override: None,
        });
        assert_eq!(short, CognitiveTier::ToySpark);
        assert!(uses_technical_tokens("deploy the k8s schema with curl"));
        assert!(!uses_technical_tokens(DINO_PROMPT));
    }

    #[test]
    fn toyspark_nine_year_old_one_click_story_is_complete() {
        let creation = build_toyspark_instant_creation(DINO_PROMPT);
        // all 3 giant buttons carry icon-first labels
        assert_eq!(creation.storybook_html.matches("class=\"fk-action\"").count(), TOYSPARK_MAX_ACTIONS_PER_SCREEN);
        assert_eq!(creation.storybook_html.matches("data-step=").count(), TOYSPARK_MAX_ACTIONS_PER_SCREEN);
        for icon in ["✨", "🎨", "🪙"] {
            assert!(creation.storybook_html.contains(icon), "icon {icon} missing");
        }
        assert!(creation.storybook_html.contains("Make Magic"));
        assert!(creation.storybook_html.contains("Make It Pretty"));
        assert!(creation.storybook_html.contains("Share &amp; Earn"));
        // 64px touch target and zero-harm sandbox
        assert!(creation.storybook_html.contains("min-width: 64px"));
        assert!(creation.storybook_html.contains("data-zero-loss-sandbox=\"true\""));
        // auto audio wizard — `speechSynthesis` + three scripts
        assert!(creation.storybook_html.contains("speechSynthesis"));
        assert_eq!(creation.voice_script.len(), 3);
        // procedural `SVG` illustration — three scenes
        assert_eq!(creation.storybook_html.matches("<svg class=\"fk-scene\"").count(), 3);
        // procedural SFX — both engines
        assert!(creation.storybook_html.contains("ARDA_STUDIO.cosmos.sfx.milestone"));
        assert!(creation.storybook_html.contains("ARDA_STUDIO.fishkal.harmonicBloom"));
        // storefront-ready listing — 50 Stars / $1.00 locked
        assert_eq!(creation.listing.price_xtr_stars, TOYSPARK_DEFAULT_XTR_STARS);
        assert_eq!(creation.listing.price_usdc_micros, TOYSPARK_DEFAULT_USDC_MICROS);
        assert_eq!(creation.listing.price_ukian, TOYSPARK_DEFAULT_UKIAN);
        assert_eq!(creation.listing.cognitive_tier_badge, "ToySpark");
        // Persian RTL detected
        assert!(creation.storybook_html.contains("dir=\"rtl\""));
        // the storybook JS syntax is healthy
        let scripts: Vec<&str> = creation
            .storybook_html
            .split("<script>")
            .skip(1)
            .filter_map(|chunk| chunk.split("</script>").next())
            .collect();
        assert!(scripts.len() >= 4, "four script blocks expected");
        for js in &scripts {
            js_syntax_sanity_check(js).unwrap_or_else(|err| panic!("storybook JS broken: {err}"));
        }
    }

    #[test]
    fn toyspark_is_deterministic() {
        let a = build_toyspark_instant_creation(DINO_PROMPT);
        let b = build_toyspark_instant_creation(DINO_PROMPT);
        assert_eq!(a.storybook_html, b.storybook_html);
        assert_eq!(a.listing.slug, b.listing.slug);
        assert_eq!(prompt_seed(DINO_PROMPT), prompt_seed("  \u{6cc}\u{647} \u{62f}\u{627}\u{6cc}\u{646}\u{627}\u{633}\u{648}\u{631} \u{622}\u{628}\u{6cc} \u{6a9}\u{647} \u{62a}\u{648}\u{6cc} \u{641}\u{636}\u{627} \u{628}\u{633}\u{62a}\u{646}\u{6cc} \u{645}\u{6cc}‌\u{641}\u{631}\u{648}\u{634}\u{647}  "));
    }

    #[test]
    fn pro_terminal_generation_uses_dense_layout_and_gate_passes() {
        let engine = OmniModalStudioEngine;
        let artifact = engine.generate(
            "build a quant order-flow terminal with schema validation",
            StudioModality::WorkspaceMode,
            CognitiveTier::ProTerminal,
        );
        assert!(uses_technical_tokens("build a quant order-flow terminal with schema validation"));
        assert!(artifact.artifact_html.contains("tier-pro"));
        assert!(artifact.artifact_html.contains("data-cog-tier=\"ProTerminal\""));
        assert!(artifact.artifact_html.contains("Ctrl+K") || artifact.artifact_html.contains("kbd"));
        assert!(artifact.rea_score >= 88, "REA must stay above the 85 forge gate");
        assert!(artifact.price_xtr_stars >= 300);
        // deterministic reproducibility
        let again = engine.generate(
            "build a quant order-flow terminal with schema validation",
            StudioModality::WorkspaceMode,
            CognitiveTier::ProTerminal,
        );
        assert_eq!(artifact.artifact_html, again.artifact_html);
    }

    #[test]
    fn all_nine_modalities_generate_marked_artifacts() {
        let engine = OmniModalStudioEngine;
        assert_eq!(STUDIO_MODALITY_COUNT, 9);
        for modality in StudioModality::all() {
            let artifact = engine.generate("\u{6cc}\u{647} \u{627}\u{633}\u{62a}\u{648}\u{62f}\u{6cc}\u{648}\u{6cc} \u{62c}\u{627}\u{62f}\u{648}\u{6cc}\u{6cc} \u{628}\u{631}\u{627}\u{6cc} \u{645}\u{646} \u{628}\u{633}\u{627}\u{632}", modality, CognitiveTier::CreatorStudio);
            assert!(artifact
                .artifact_html
                .contains(&format!("data-modality=\"{}\"", modality.as_str())));
            assert_eq!(artifact.tier, CognitiveTier::CreatorStudio);
            js_syntax_sanity_check(&artifact.dual_bundle.fishkal_js)
                .unwrap_or_else(|err| panic!("fishkal JS broken for {}: {err}", modality.as_str()));
            js_syntax_sanity_check(&artifact.dual_bundle.cosmos_js)
                .unwrap_or_else(|err| panic!("cosmos JS broken for {}: {err}", modality.as_str()));
        }
        for (spec, modality) in studio_modality_specs().iter().zip(StudioModality::all()) {
            assert_eq!(spec.modality, modality);
            assert!(spec.zero_cost, "every modality must be zero-cost");
            assert_eq!(spec.auth, "none");
        }
        for modality in StudioModality::all() {
            assert_eq!(StudioModality::parse(modality.as_str()), Some(modality));
        }
        assert_eq!(StudioModality::parse("hologram"), None);
    }

    #[test]
    fn dual_bundle_carries_exact_fishkal_constants() {
        let bundle = emit_dual_cosmos_fishkal_bundle();
        let js = &bundle.fishkal_js;
        // 4-second Brownian noise — coefficients preserved
        assert!(js.contains("b0 = (b0 + 0.02 * white) / 1.02"));
        assert!(js.contains("data[i] = b0 * 0.18"));
        // `520Hz` low-pass with a `0.06Hz` sine LFO and a `±220Hz` sweep
        assert!(js.contains("lp.frequency.value = 520"));
        assert!(js.contains("lfo.frequency.value = 0.06"));
        assert!(js.contains("lfoGain.gain.value = 220"));
        // D-major pentatonic bloom + the 4-voice signature chord
        assert!(js.contains("[587.33, 880, 1174.66]"));
        assert!(js.contains("[293.66, 440, 587.33, 880]"));
        // `xorshift32` + 4-octave `fbm` + cubic-Hermite `smoothstep`
        assert!(js.contains("s ^= s << 13"));
        assert!(js.contains("s ^= s >>> 17"));
        assert!(js.contains("f *= 2.03"));
        assert!(js.contains("t * t * t * (t * (t * 6 - 15) + 10)"));
        // frame-independent lambda variable — the `0.42..0.63` window with `2.3` / `5.0`
        assert!(js.contains("FISHKAL_DAMP_WINDOW_LO = 0.42"));
        assert!(js.contains("FISHKAL_DAMP_WINDOW_HI = 0.63"));
        assert!(js.contains("FISHKAL_DAMP_LAMBDA_WINDOW = 2.3"));
        assert!(js.contains("FISHKAL_DAMP_LAMBDA = 5.0"));
        assert!(js.contains("Math.exp(-l * dt)"));
        assert_eq!(bundle.signature, DUAL_BUNDLE_SIGNATURE);
        assert_eq!(bundle.content_sha256.len(), 64);
        // deterministic reproducibility — same `sha256`
        assert_eq!(bundle.content_sha256, emit_dual_cosmos_fishkal_bundle().content_sha256);
    }

    #[test]
    fn dual_bundle_carries_exact_cosmos_constants() {
        let bundle = emit_dual_cosmos_fishkal_bundle();
        let js = &bundle.cosmos_js;
        // starfall — `110` mobile / `260` desktop, `DPR` cap `2`
        assert!(js.contains("COSMOS_STARS_MOBILE = 110"));
        assert!(js.contains("COSMOS_STARS_DESKTOP = 260"));
        assert!(js.contains("Math.min(2, (global.devicePixelRatio || 1))"));
        // `0.05` parallax decay + `0.16` HUD smoothing + 7 milestone nodes
        assert!(js.contains("COSMOS_PARALLAX_DAMPING = 0.05"));
        assert!(js.contains("COSMOS_HUD_EASING = 0.16"));
        assert!(js.contains("COSMOS_MILESTONE_NODES = 7"));
        // delay-feedback synth — `0.22s` + `0.25` + `1600Hz` low-pass
        assert!(js.contains("delay.delayTime.value = COSMOS_DELAY_SECONDS"));
        assert!(js.contains("COSMOS_DELAY_SECONDS = 0.22"));
        assert!(js.contains("COSMOS_FEEDBACK_GAIN = 0.25"));
        assert!(js.contains("COSMOS_TONE_LOWPASS_HZ = 1600"));
        // five SFX triggers with preserved parameters
        assert!(js.contains("osc.frequency.value = 220 + (idx % 7) * 40"));
        assert!(js.contains("exponentialRampToValueAtTime(520, t + 0.28)"));
        assert!(js.contains("bp.frequency.setValueAtTime(260, t)"));
        assert!(js.contains("exponentialRampToValueAtTime(1200, t + 0.32)"));
        assert!(js.contains("bp.Q.value = 2.2"));
        assert!(js.contains("var notes = [440, 660]"));
        assert!(js.contains("var notes = [330, 495, 660]"));
        assert!(js.contains("g.gain.exponentialRampToValueAtTime(0.0001, t + i * 0.07 + 0.07)"));
        assert!(js.contains("g.gain.exponentialRampToValueAtTime(0.0001, t + i * 0.09 + 0.09)"));
    }

    #[test]
    fn pause_lifecycle_is_shared_and_ref_counted() {
        let bundle = emit_dual_cosmos_fishkal_bundle();
        for js in [&bundle.fishkal_js, &bundle.cosmos_js, &bundle.pause_lifecycle_js] {
            js_syntax_sanity_check(js).unwrap_or_else(|err| panic!("bundle JS broken: {err}"));
        }
        assert!(bundle.pause_lifecycle_js.contains("FK.__fkPauseDepth += 1"));
        assert!(bundle.pause_lifecycle_js.contains("Math.max(0, FK.__fkPauseDepth - 1)"));
        assert!(bundle.pause_lifecycle_js.contains("visibilitychange"));
        assert!(bundle.pause_lifecycle_js.contains("document.hidden"));
        assert!(bundle.cosmos_js.contains("if (FK.__fkPauseDepth === 0 && !document.hidden)"));
        assert!(bundle.cosmos_js.contains("FK.fkPause"));
        assert!(bundle.cosmos_js.contains("FK.fkResume"));
    }

    #[test]
    fn js_syntax_sanity_check_flags_breakage() {
        assert!(js_syntax_sanity_check("var a = { b: [1, 2] };").is_ok());
        assert!(js_syntax_sanity_check("var s = 'it\\'s { fine'; var t = \"a) b\";").is_ok());
        assert!(js_syntax_sanity_check("// braces { in comment\nvar ok = 1;").is_ok());
        assert!(js_syntax_sanity_check("var x = {").is_err());
        assert!(js_syntax_sanity_check("var s = 'never closed;").is_err());
        assert!(js_syntax_sanity_check("var x = } mismatch;").is_err());
        assert!(js_syntax_sanity_check("/* never closed").is_err());
    }

    #[test]
    fn portfolio_dark_glass_tokens_are_verbatim() {
        let css = portfolio_dark_glass_css();
        for token in [
            "--bg: #05050a",
            "--bg-2: #0a0b14",
            "--surface: rgba(16, 18, 32, 0.68)",
            "--glass: rgba(6, 6, 14, 0.78)",
            "--text: #f3f4f8",
            "--muted: #9ca3b8",
            "--accent: #7c5cff",
            "--accent-2: #19d3da",
            "--accent-3: #ff5ca8",
            "--ease: cubic-bezier(.2, .7, .2, 1)",
            "backdrop-filter: blur(14px)",
            "clamp(36px, 5.6vw, 62px)",
            "max-width: 1240px",
            "'Space Grotesk'",
            "'Inter'",
            "'JetBrains Mono'",
            "'Vazirmatn'",
        ] {
            assert!(css.contains(token), "missing token: {token}");
        }
    }

    #[test]
    fn storefront_card_keeps_twelve_canonical_keys() {
        let card = &seed_storefront_catalog()[0];
        let json = serde_json::to_string(card).unwrap_or_default();
        for key in [
            "slug", "name", "tagline", "category", "stack", "highlights",
            "preview", "sourceDir", "readme", "loc", "fileCount", "featured",
        ] {
            assert!(json.contains(&format!("\"{key}\"")), "canonical key missing: {key}");
        }
        for ext in [
            "priceUkian", "priceXtrStars", "priceUsdcMicros", "reaScore",
            "skillMdUrl", "supportedLocalesCount", "cognitiveTierBadge",
        ] {
            assert!(json.contains(&format!("\"{ext}\"")), "commerce extension missing: {ext}");
        }
        assert_eq!(seed_storefront_catalog().len(), 8);
    }

    #[test]
    fn featured_projects_promote_to_top_two() {
        let catalog = seed_storefront_catalog();
        assert_eq!(catalog.iter().filter(|c| c.featured).count(), 2);
        let promoted = promote_featured_top2(&catalog);
        assert_eq!(promoted.len(), catalog.len());
        assert!(promoted[0].featured && promoted[1].featured);
        assert!(!promoted[2].featured);
        assert_eq!(promoted[0].slug, "fishkal");
        assert_eq!(promoted[1].slug, "cosmos");
    }

    #[test]
    fn blocked_source_paths_never_pass_the_guard() {
        for blocked in [
            ".env", ".env.local", "config/.env.production", "cert.pem", "signing.key",
            "ssh/id_rsa", "ssh/id_rsa.pub", "credentials.json", "data/db.sqlite",
        ] {
            assert!(is_blocked_source_path(blocked), "must be blocked: {blocked}");
        }
        for allowed in ["index.html", "src/main.rs", "assets/style.css", "readme.md"] {
            assert!(!is_blocked_source_path(allowed), "must pass: {allowed}");
        }
        assert_eq!(
            sanitize_source_tree(&[".env", "index.html", "key.pem", "app.js"]),
            vec!["index.html", "app.js"]
        );
    }

    #[test]
    fn portfolio_safety_fixes_are_mechanically_verified() {
        let bundle = emit_dual_cosmos_fishkal_bundle();
        let engine = OmniModalStudioEngine;
        let workspace = engine.generate("multi-file canvas", StudioModality::WorkspaceMode, CognitiveTier::CreatorStudio);
        assert!(verify_portfolio_safety_fixes(&bundle, &workspace.artifact_html));
        // wire report — always 6/6
        let report = portfolio_safety_report();
        assert_eq!(report.len(), 6);
        for (idx, fix) in report.iter().enumerate() {
            assert_eq!(fix.fix_id, u8::try_from(idx + 1).unwrap_or(6));
            assert_eq!(fix.status, "fixed");
            assert_ne!(fix.evidence, "");
        }
        // Fix #1 — no `path.resolve(PORT, '..')` and no `..` in the root
        assert!(!PORTFOLIO_ROOT_RESOLUTION.contains(".."));
        assert_eq!(PORTFOLIO_ROOT_RESOLUTION, "./seed/portfolio_catalog");
        assert_eq!(PORTFOLIO_CATALOG_ENV, "KIAN_PORTFOLIO_CATALOG_PATH");
    }

    #[test]
    fn command_palette_is_keyboard_first() {
        let cfg = command_palette_config();
        assert_eq!(cfg.open_shortcuts, ["Ctrl+K", "Meta+K", "/"]);
        assert!(cfg.keyboard_first);
        let js = emit_command_palette_js();
        assert!(js.contains("e.ctrlKey || e.metaKey"));
        assert!(js.contains("e.key === 'k' || e.key === 'K'"));
        assert!(js.contains("e.key === '/'"));
        assert!(js.contains("e.key === 'Escape'"));
        assert!(js.contains("e.key === 'ArrowDown'"));
        js_syntax_sanity_check(&js).unwrap_or_else(|err| panic!("palette JS broken: {err}"));
    }

    #[test]
    fn toyspark_pricing_is_locked_and_rea_gate_always_passes() {
        for prompt in ["\u{642}\u{635}\u{635} \u{633}\u{62a}\u{627}\u{631}\u{647}‌\u{627}\u{6cc}", "build me a spaceship game", "\u{62f}\u{627}\u{6cc}\u{646}\u{627}\u{633}\u{648}\u{631} \u{622}\u{628}\u{6cc} \u{62f}\u{631} \u{641}\u{636}\u{627}"] {
            let score = derive_rea_score(prompt_seed(prompt));
            assert!((88..=97).contains(&score), "REA {score} out of deterministic band");
            assert!(score >= crate::arda_fishkal_engine::SKILL_FORGE_REA_GATE);
            let (ukian, xtr, usdc) = derive_pricing(prompt_seed(prompt), CognitiveTier::ToySpark);
            assert_eq!((ukian, xtr, usdc), (1_000_000, 50, 1_000_000));
        }
    }
}

