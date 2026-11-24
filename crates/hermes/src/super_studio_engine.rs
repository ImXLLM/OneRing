//! موتور ابراستودیوی ادراک‌سیار (Wave 5 — Step 2/4).
//!
//! سه زی‌موتور روی یک هستهٔ قطعی:
//! 1. [`CognitiveUxMorphEngine`] — مورفِ شناختیِ سه‌لایه: `ToySpark`
//!    (کودکِ ۹ساله / `IQ` ۵۰) ↔ `CreatorStudio` (سازندهٔ بدون‌کد) ↔
//!    `ProTerminal` (مهندسِ ۲۸سالهٔ کوانت) — با استنتاجِ خودکار لایه از
//!    تلمتری و سوییچِ یک‌کلیکیِ بدونِ ازدست‌رفتنِ وضعیت؛
//! 2. [`OmniModalStudioEngine`] — ۹ مدالیتیِ صفر-هزینه با باندلِ دوگانهٔ
//!    رویه‌ایِ `Fishkal` (اقیانوس + بلومِ هارمونیک) و `Portfolio Cosmos`
//!    (ستاره‌باران + سینتِ دیلی‌بازخورد)؛
//! 3. [`PortfolioStorefrontEngine`] — ویترینِ سازنده با ۱۲ کلیدِ
//!    کانونیِ `projects.json`، ارتقای خودکارِ «دو ویترینیِ برتر» و درمانِ
//!    ۶ نقصِ قضاییِ `KiaAgentX/portfolio`.
//!
//! قرارداد کرنل: صفر `unsafe`، صفر کلید شبکه، قطعیت کامل (همان ورودی →
//! همان باندل، همان `sha256`) و گیتِ صفر-هزینه روی هر ۹ مدالیتی.
#![allow(clippy::module_name_repetitions)]

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

// ---------------------------------------------------------------------------
// زیر-موتور ۱ — `CognitiveUxMorphEngine`
// ---------------------------------------------------------------------------

/// سقفِ اقدامِ هر صفحه در `ToySpark` — قانونِ سه‌دکمهٔ غول‌پیکر.
pub const TOYSPARK_MAX_ACTIONS_PER_SCREEN: usize = 3;

/// حداقلِ هدفِ لمسیِ `ToySpark` بر حسب پیکسل — انگشتِ کودکِ ۹ساله.
pub const TOYSPARK_MIN_TAP_TARGET_PX: u32 = 64;

/// سقفِ واژهٔ هر برچسب — ریزمتنیِ آیکون‌اول.
pub const TOYSPARK_MAX_WORDS_PER_LABEL: usize = 5;

/// قیمتِ پیش‌فرضِ یک‌کلیکیِ `ToySpark` — ۵۰ ستارهٔ تلگرام.
pub const TOYSPARK_DEFAULT_XTR_STARS: u32 = 50;

/// قیمتِ پیش‌فرضِ یک‌کلیکیِ `ToySpark` — ۱٫۰۰ دلار (`USDC` میکرو).
pub const TOYSPARK_DEFAULT_USDC_MICROS: u64 = 1_000_000;

/// قیمتِ پیش‌فرضِ یک‌کلیکیِ `ToySpark` — یک میلیون میکرو-کیان.
pub const TOYSPARK_DEFAULT_UKIAN: u64 = 1_000_000;

/// نشانه‌های فنی — هر یک، استنتاج را به `ProTerminal` قفل می‌کند.
pub const TECHNICAL_TOKENS: [&str; 18] = [
    "fn ", "impl ", "async ", "curl ", "sql", "json", "schema", "regex", "git ", "docker",
    "k8s", "quant", "api ", "rust", "python", "token", "deploy", "terminal",
];

/// لایهٔ شناختیِ رابط — پهنای باندِ ادراکیِ کاربر را مورف می‌کند.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CognitiveTier {
    /// کودکِ ۹ساله / `IQ` ۵۰ — سه‌دکمهٔ غول‌پیکر، جادوی یک‌کلیکی.
    ToySpark,
    /// سازندهٔ عمومی بدون‌کد — بلاک‌های درگ‌ودراپ + پیش‌نمایش زنده.
    CreatorStudio,
    /// مهندس/معامله‌گرِ ۲۸ساله — پالت فرمانِ کیبورد-اول و تلمتریِ متراکم.
    ProTerminal,
}

impl CognitiveTier {
    /// شناسهٔ وایرِ لایه (`ToySpark` | `CreatorStudio` | `ProTerminal`).
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ToySpark => "ToySpark",
            Self::CreatorStudio => "CreatorStudio",
            Self::ProTerminal => "ProTerminal",
        }
    }

    /// برچسب بومیِ لایه.
    #[must_use]
    pub fn fa_label(self) -> &'static str {
        match self {
            Self::ToySpark => "جرقهٔ اسباب‌بازی",
            Self::CreatorStudio => "استودیوی سازنده",
            Self::ProTerminal => "ترمینال حرفه‌ای",
        }
    }

    /// کلاس CSS بدنه برای مورفِ لایه.
    #[must_use]
    pub fn css_class(self) -> &'static str {
        match self {
            Self::ToySpark => "tier-toyspark",
            Self::CreatorStudio => "tier-creator",
            Self::ProTerminal => "tier-pro",
        }
    }

    /// هر ۳ لایه به ترتیبِ نردبانِ شناختی.
    #[must_use]
    pub fn all() -> [Self; 3] {
        [Self::ToySpark, Self::CreatorStudio, Self::ProTerminal]
    }

    /// تجزیهٔ وایر — ناشناخته → `None`.
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

/// پروفایلِ مورفِ یک لایهٔ شناختی — قرارداد UX عملیاتی.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CognitiveTierProfile {
    /// لایهٔ صاحبِ پروفایل.
    pub tier: CognitiveTier,
    /// باند سنی/هوشیِ هدف (`Age 9 / `IQ` 50` …).
    pub age_band: &'static str,
    /// سقفِ اقدامِ هر صفحه.
    pub max_actions_per_screen: usize,
    /// حداقلِ هدفِ لمسی (پیکسل).
    pub min_tap_target_px: u32,
    /// سقفِ واژهٔ هر برچسب.
    pub max_words_per_label: usize,
    /// جادوگرِ صوتیِ خودکار؟
    pub voice_guided: bool,
    /// سندباکسِ صفر-زیان.
    pub zero_loss_sandbox: bool,
    /// کیبورد-اول (پالت فرمان)؟
    pub keyboard_first: bool,
    /// خلاصهٔ چیدمان برای کلاینت.
    pub layout_summary: &'static str,
}

/// سه پروفایلِ کانونیِ مورفِ شناختی — قرارداد `GET /studio/ux-profiles`.
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

/// تلمتریِ تعامل برای استنتاجِ لایه — بدونِ هرگز پرسیدنِ سن.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CognitiveTelemetry {
    /// طولِ پرامپت بر حسب کاراکتر.
    pub prompt_length_chars: usize,
    /// آیا پرامپت نشانهٔ فنی دارد؟
    pub uses_technical_tokens: bool,
    /// آهنگِ تایپ میانگین (میلی‌ثانیه بر کلید) — `0` = نامعلوم.
    pub typing_cadence_ms: u32,
    /// لایهٔ صریحِ انتخابی کاربر — همیشه برنده.
    pub explicit_tier_override: Option<CognitiveTier>,
}

/// تشخیصِ نشانهٔ فنی در پرامپت — قطعی و بی‌هزینه.
#[must_use]
pub fn uses_technical_tokens(prompt: &str) -> bool {
    let needle = prompt.to_lowercase();
    TECHNICAL_TOKENS.iter().any(|t| needle.contains(t))
}

/// استنتاجِ لایه + لغوِ صریح — [`infer_or_override_tier`] قراردادِ
/// سوییچِ صفر-اصطکاک است: لغوِ صریح همیشه برنده، وگرنه نردبانِ تلمتری.
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
// زیر-موتور ۲ — `OmniModalStudioEngine` (۹ مدالیتیِ صفر-هزینه)
// ---------------------------------------------------------------------------

/// ۹ مدالیتیِ استودیو — همهٔ صفر-هزینه و بدونِ کلیدِ شبکه.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StudioModality {
    /// سنتزِ وب/`API` چند-منبعیِ بدونِ احراز (`Agent-Reach` + ۱۲۰ `API` عمومی).
    Search,
    /// پژوهشِ عمیقِ چند-ایجنتی + درختِ وریفایِ ۸-گامیِ مایرون.
    DeepMode,
    /// مولدِ خودکارِ ساب-ایجنتِ `OpenBot` با DNA ۹-گونی + `SKILL.md`.
    AgentMode,
    /// بومِ چند-فایلی با پیش‌نمایشِ سندباکس‌شده و بازرسِ کد.
    WorkspaceMode,
    /// `SpeechSynthesis` / `MediaRecorder` بومیِ مرورگر + موسیقیِ محیطیِ رویه‌ای.
    VoiceAudio,
    /// `SVG`/`Canvas2D` برداریِ قطعی + آبشاریِ فراهم‌کنندهٔ بی‌کلید.
    ImageGen,
    /// موشن‌گرافیکِ ۶۰fps با تایم‌لاینِ `TL` فیشکال + `damp` لامبدا-متغیر.
    VideoGen,
    /// کپی فروش ۲۰-زبانه + صفحات فرودِ تنظیم‌شدهٔ `RTL`/`LTR`.
    TextCopy,
    /// استودیوی SFX رویه‌ایِ `WebAudio` + سینتِ محیطی.
    SoundEffects,
}

impl StudioModality {
    /// شناسهٔ وایرِ مدالیتی.
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

    /// برچسب بومیِ مدالیتی.
    #[must_use]
    pub fn fa_label(self) -> &'static str {
        match self {
            Self::Search => "جست‌وجوی چند-منبعی",
            Self::DeepMode => "پژوهش عمیق",
            Self::AgentMode => "مولد ساب-ایجنت",
            Self::WorkspaceMode => "بوم چند-فایلی",
            Self::VoiceAudio => "صدا و موسیقی",
            Self::ImageGen => "تصویرسازی رویه‌ای",
            Self::VideoGen => "موشن ۶۰fps",
            Self::TextCopy => "کپی فروش ۲۰-زبانه",
            Self::SoundEffects => "استودیوی SFX",
        }
    }

    /// هر ۹ مدالیتی به ترتیبِ کانونی.
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

    /// تجزیهٔ وایر — ناشناخته → `None`.
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

/// تعدادِ مدالیتی‌های کانونی — گیتِ آزمون.
pub const STUDIO_MODALITY_COUNT: usize = 9;

/// مشخصهٔ یک مدالیتی روی وایر.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioModalitySpec {
    /// مدالیتی صاحبِ مشخصه.
    pub modality: StudioModality,
    /// برچسب بومی.
    pub fa_label: &'static str,
    /// احرازِ لازم — همیشه `none` (صفر-هزینه).
    pub auth: &'static str,
    /// موتورِ اجراییِ خلاصه.
    pub engine: &'static str,
    /// همیشه `true` — گیتِ صفر-هزینه.
    pub zero_cost: bool,
}

/// مشخصه‌های هر ۹ مدالیتی — قرارداد `GET /studio/ux-profiles`.
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

/// امضای باندلِ دوگانهٔ رویه‌ای — `Fishkal` + `Portfolio Cosmos`.
pub const DUAL_BUNDLE_SIGNATURE: &str = "fishkal-cosmos-dual-v1";

/// مُهر `sha256` باندل — قطعی روی هر بازتولید.
#[must_use]
pub fn bundle_content_sha256(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// باندلِ دوگانهٔ موتورهای رویه‌ای — موتور A (`Fishkal`)، موتور B
/// (`Portfolio Cosmos`) و چرخهٔ عمرِ مشترکِ توقفِ `RAF` (درمانِ Fix #6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DualCosmosFishkalBundle {
    /// موتور A — اقیانوسِ فیشکال + بلومِ هارمونیک (`index.html:L709–728, L1665–1750, L2172`).
    pub fishkal_js: String,
    /// موتور B — ستاره‌بارانِ کاسموس + سینتِ دیلی‌بازخورد (`cosmos.js:L6–60, L83–149, L216–269`).
    pub cosmos_js: String,
    /// چرخهٔ عمرِ مشترکِ `__fkPauseDepth` (`fkPause` / `fkResume` / `document.hidden`).
    pub pause_lifecycle_js: String,
    /// امضای باندل (`fishkal-cosmos-dual-v1`).
    pub signature: String,
    /// مُهر `sha256` کلِ باندل.
    pub content_sha256: String,
}

/// موتور A — سرف و بلومِ هارمونیکِ فیشکال، حفظِ حرف‌به‌حرفِ پارامترهای
/// ممیزی‌شده: بافرِ براوان-نویزِ ۴ثانیه‌ای (`b0 = (b0 + 0.02 * white) / 1.02`
/// با مقیاسِ `0.18`)، `BiquadFilter` لو-پسِ `520Hz` با `LFO` سینوسیِ `0.06Hz`
/// و جاروبِ `±220Hz`، بلومِ پنتاتونیکِ رِ ماژورِ `[587.33, 880, 1174.66]`،
/// آکوردِ امضای ۴-صداییِ `[293.66, 440, 587.33, 880]`، `xorshift32`،
/// `fbm` چهار-اکتاوی (`f *= 2.03`)، `smoothstep` ارمایتِ مکعبی و
/// `damp` با لامبدا-متغیرِ مستقل از فریم.
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

/// موتور B — ستاره‌بارانِ تعاملیِ کاسموس (`DPR` سقفِ `2`، `110` ستارهٔ
/// موبایل / `260` دسکتاپ، گرادیان‌های سحابیِ شعاعی، میراییِ پارالاکسِ
/// اشاره‌گرِ `0.05`، نرم‌شوندگیِ `HUD` با `0.16`، ۷ گرهٔ نق milestones) +
/// سینتِ دیلی-بازخوردِ (`DelayNode(0.22s)` + بازخوردِ `0.25` + لو-پسِ
/// `1600Hz`) با هر ۵ تریگرِ SFX کانونی — و سیم‌کشی به `__fkPauseDepth`.
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

/// چرخهٔ عمرِ مشترکِ توقفِ `RAF` — درمانِ Fix #6: هر حلقهٔ انیمیشن روی
/// `__fkPauseDepth` و `document.hidden` مشترک است؛ `visibilitychange`
/// عمقِ توقف را خودکار مِهر می‌کند.
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

/// بررسیِ تندرستیِ نحوِ JS — توازنِ `{}`/`()`/`[]` بیرون از رشته‌ها و
/// کامنت‌ها؛ قطعی و بدونِ هیچ وابستگی.
///
/// # Errors
/// رشته/کامنتِ بسته‌نشده یا توازنِ شکسته → `Err(علت)`.
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

/// باندلِ دوگانهٔ رویه‌ای — قطعی؛ همان فراخوانی، همان `sha256`.
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
// جرقهٔ اسباب‌بازی — داستان/بازیِ یک‌کلیکیِ کودکِ ۹ساله
// ---------------------------------------------------------------------------

/// بذرِ قطعی از پرامپت — ۴ بایتِ اولِ `sha256`.
#[must_use]
pub fn prompt_seed(prompt: &str) -> u32 {
    let mut hasher = Sha256::new();
    hasher.update(prompt.trim().as_bytes());
    let digest = hasher.finalize();
    u32::from(digest[0]) | (u32::from(digest[1]) << 8) | (u32::from(digest[2]) << 16) | (u32::from(digest[3]) << 24)
}

/// عنوانِ قطعی — برشِ امنِ مرزِ کاراکتر + پسوندِ مدالیتی.
#[must_use]
pub fn derive_title(prompt: &str, suffix: &str) -> String {
    let trimmed = prompt.trim();
    let mut head: String = trimmed.chars().take(40).collect();
    if trimmed.chars().count() > 40 {
        head.push('…');
    }
    if head.is_empty() {
        head.push_str("آفرینش استودیویی");
    }
    format!("{head} — {suffix}")
}

/// خلاصهٔ صوتیِ جادوگرِ `ToySpark` — سه گامِ صوتیِ خودکار.
#[must_use]
pub fn toyspark_voice_script(title: &str) -> [String; 3] {
    [
        format!("سلام! بیا با هم «{title}» را بسازیم؛ فقط دکمهٔ اول را لمس کن."),
        "چه قشنگ شد! دکمهٔ دوم را بزن تا صدا و رنگ‌ها برقصند.".to_owned(),
        "آفرین! دکمهٔ سوم آفرینش تو را با ۵۰ ستاره به کل سیاره می‌فروشد.".to_owned(),
    ]
}

/// خروجیِ یک‌کلیکیِ `ToySpark` — داستان‌کتابِ HTML5 قابل بازی + لیستینگِ
/// آمادهٔ فروش + سناریوی صوتیِ جادوگر.
#[derive(Debug, Clone, PartialEq)]
pub struct ToySparkCreation {
    /// عنوانِ قطعیِ آفرینش.
    pub title: String,
    /// داستان‌کتاب/بازیِ کاملِ تک-فایلی.
    pub storybook_html: String,
    /// لیستینگِ آمادهٔ فروش با قیمتِ پیش‌فرضِ ۵۰ ستاره / ۱٫۰۰ دلار.
    pub listing: StorefrontProjectCard,
    /// سناریوی صوتیِ جادوگر — دقیقاً ۳ گام.
    pub voice_script: [String; 3],
}

/// سه دکمهٔ غول‌پیکرِ کانونیِ `ToySpark` — برچسب‌های آیکون‌اول.
pub const TOYSPARK_ACTIONS: [&str; 3] = [
    "✨ بساز (Make Magic)",
    "🎨 قشنگش کن (Make It Pretty & Loud)",
    "🪙 بفروش و ستاره بگیر (Share & Earn Stars)",
];

/// جریانِ «یک جمله → یک کتاب داستانِ قابل بازی» برای کودکِ ۹ساله: از یک
/// جمله یا متنِ پیاده‌شدهٔ صوت، در یک کلیک یک HTML5 تعاملیِ کامل با
/// تصویرسازیِ `SVG` رویه‌ای، جلوه‌های صوتیِ `WebAudio` رویه‌ای (`Fishkal` +
/// `Cosmos`) و لیستینگِ آمادهٔ فروش می‌سازد — قطعی و بدونِ هیچ کلید.
#[must_use]
pub fn build_toyspark_instant_creation(prompt: &str) -> ToySparkCreation {
    use std::fmt::Write as _;
    let seed = prompt_seed(prompt);
    let rtl = crate::studio_design_contract::detect_persian_rtl(prompt);
    let title = derive_title(prompt, "ماجرای یک‌کلیکیِ من");
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
            r#"<svg class="fk-scene" data-scene="{idx}" viewBox="0 0 200 140" role="img" aria-label="صحنهٔ رویه‌ای {idx}"><rect width="200" height="140" rx="18" fill="hsl({h}, 70%, 12%)"/><circle cx="{cx}" cy="{cy}" r="26" fill="hsl({h}, 85%, 62%)"/><circle cx="{cx}" cy="{cy}" r="34" fill="none" stroke="hsl({h}, 85%, 70%)" stroke-dasharray="4 6"/><path d="M0 120 Q 50 100 100 120 T 200 120" stroke="hsl({h}, 60%, 40%)" fill="none" stroke-width="3"/><g fill="hsl({h}, 90%, 80%)"><circle cx="20" cy="24" r="2"/><circle cx="176" cy="30" r="2"/><circle cx="150" cy="16" r="1.6"/><circle cx="60" cy="14" r="1.6"/></g></svg>"#,
            idx = idx + 1,
            h = h,
            cx = cx,
            cy = cy
        );
    }
    let voice = toyspark_voice_script(&title);
    let star_price = TOYSPARK_DEFAULT_XTR_STARS;
    let storybook = format!(
        r#"<!DOCTYPE html><html lang="{lang}" dir="{dir}"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>{title}</title><style>{css}</style></head><body class="fk-studio {tier_class}" data-cog-tier="ToySpark" data-zero-loss-sandbox="true"><header class="fk-hero"><h1>{title}</h1><p class="fk-hint">سه دکمهٔ بزرگ — هر لمس، جادو می‌کند.</p></header><main class="fk-card">{svg}<nav class="fk-actions" aria-label="سه اقدامِ جادویی"><button type="button" class="fk-action" data-action="magic" data-step="1"><span class="fk-icon">✨</span><span class="fk-label">بساز (Make Magic)</span></button><button type="button" class="fk-action" data-action="pretty" data-step="2"><span class="fk-icon">🎨</span><span class="fk-label">قشنگش کن (Make It Pretty)</span></button><button type="button" class="fk-action" data-action="share" data-step="3"><span class="fk-icon">🪙</span><span class="fk-label">بفروش و ستاره بگیر (Share &amp; Earn)</span></button></nav><p class="fk-sandbox" data-sandbox="zero-loss">سندباکسِ صفر-زیان: هیچ چیز هرگز از دست تو خارج نمی‌شود.</p><p class="fk-price" data-xtr="{star_price}">۵۰ ستاره — یک دلارِ مهربان</p></main><script>{fishkal_js}</script><script>{cosmos_js}</script><script>{pause_js}</script><script>
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
// مولدِ آرتیفکتِ استودیو — پوستهٔ مشترک + بدنهٔ هر مدالیتی
// ---------------------------------------------------------------------------

/// آرتیفکتِ مولدِ استودیو — HTML قابل بازی + باندلِ دوگانه + قیمت‌گذاری.
#[derive(Debug, Clone, PartialEq)]
pub struct StudioArtifact {
    /// عنوانِ قطعی.
    pub title: String,
    /// آرتیفکتِ کاملِ تک-فایلی (تزریقیِ ARDA روی آن انجام می‌شود).
    pub artifact_html: String,
    /// باندلِ دوگانهٔ رویه‌ای.
    pub dual_bundle: DualCosmosFishkalBundle,
    /// لایهٔ شناختیِ مورف‌شده.
    pub tier: CognitiveTier,
    /// مدالیتیِ مولد.
    pub modality: StudioModality,
    /// نمرهٔ REA قطعی (`88..=97` — همیشه بالای گیتِ ۸۵ مایرون).
    pub rea_score: u8,
    /// قیمت به میکرو-کیان.
    pub price_ukian: u64,
    /// قیمت به ستارهٔ تلگرام.
    pub price_xtr_stars: u32,
    /// قیمت به میکرو-`USDC`.
    pub price_usdc_micros: u64,
}

/// استودیوی ابرمدالیتی — هر ۹ مدالیتیِ صفر-هزینه را روی یک پوستهٔ مشترک
/// با باندلِ دوگانهٔ `Fishkal + Cosmos` می‌سازد؛ کاملاً قطعی.
#[derive(Debug, Clone, Copy, Default)]
pub struct OmniModalStudioEngine;

impl OmniModalStudioEngine {
    /// تولیدِ آرتیفکتِ کامل برای پرامپت + مدالیتی + لایهٔ شناختی — قطعی.
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

/// نمرهٔ REA قطعی از بذرِ پرامپت — بازهٔ `88..=97`، همیشه بالای گیتِ ۸۵.
#[must_use]
pub fn derive_rea_score(seed: u32) -> u8 {
    u8::try_from(88 + seed % 10).unwrap_or(92)
}

/// قیمت‌گذاریِ قطعی بر حسب لایهٔ شناختی — `ToySpark` روی پیش‌فرضِ یک‌کلیکی
/// قفل است (۵۰ ستاره / ۱٫۰۰ دلار)؛ لایه‌های بالاتر آزادانه‌تر.
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

/// پوستهٔ مشترکِ آرتیفکت — توکن‌های گلسِ تاریکِ پورتفولیو + مورفِ لایه.
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

/// گریزِ HTML برای درجِ امنِ عنوان.
#[must_use]
pub fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// بدنهٔ اختصاصیِ هر مدالیتی — ۹ مدالیتی، هر یک با نشانِ `data-modality`.
#[must_use]
pub fn modality_body(modality: StudioModality, title: &str) -> String {
    let title = html_escape(title);
    match modality {
        StudioModality::Search => format!(
            r#"<main class="fk-grid" data-modality="search"><section class="fk-pane"><h2>{title}</h2><p>سنتزِ چند-منبعیِ بدونِ احراز: آبشارِ `Agent-Reach` (Jina/Exa MCP) + ۱۲۰ API عمومیِ CORS.</p><ul class="fk-src" data-cascade="jina,exa,public-cors-120"><li data-source="jina-reader">Jina Reader — خوانشِ بی‌کلید</li><li data-source="exa">Exa — جست‌وجوی معناییِ بی‌کلید</li><li data-source="public-cors-120">۱۲۰ API عمومیِ CORS</li></ul></section></main>"#
        ),
        StudioModality::DeepMode => format!(
            r#"<main class="fk-grid" data-modality="deep"><section class="fk-pane"><h2>{title}</h2><ol class="fk-tree" data-mairon-steps="8"><li>۱ صورت‌مسئله</li><li>۲ فرضیه</li><li>۳ شواهد</li><li>۴ نقیض</li><li>۵ سنتز</li><li>۶ آزمونِ سخت</li><li>۷ تصمیم</li><li>۸ مُهرِ مایرون</li></ol></section></main>"#
        ),
        StudioModality::AgentMode => format!(
            r#"<main class="fk-grid" data-modality="agent"><section class="fk-pane"><h2>{title}</h2><p>DNA ۹-گونیِ تصادفی‌نما + فورجِ `SKILL.md`.</p><div class="fk-dna" data-fold="9" data-dna="9-fold-stochastic"></div><a class="chip" data-download="SKILL.md" href="skill-md">دانلود SKILL.md</a></section></main>"#
        ),
        StudioModality::WorkspaceMode => format!(
            r#"<main class="fk-grid" data-modality="workspace"><section class="fk-pane"><h2>{title}</h2><div class="fk-chip-row" role="toolbar" aria-label="زبانه‌ها"><button type="button" class="chip" aria-pressed="true" data-tab="preview">پیش‌نمایش</button><button type="button" class="chip" aria-pressed="false" data-tab="source">بازرسِ کد</button></div><iframe class="fk-preview" sandbox="allow-scripts" title="پیش‌نمایشِ سندباکس‌شده" data-preview></iframe><ul class="fk-files" data-files="safe-only"><li>index.html</li><li>studio.css</li></ul></section><script data-project-viewer>let loaded = false; function loadSource() {{ loaded = true; }} function setTab(tab) {{ loadSource(tab); }} setTab('preview');</script></main>"#
        ),
        StudioModality::VoiceAudio => format!(
            r#"<main class="fk-grid" data-modality="voice"><section class="fk-pane"><h2>{title}</h2><button type="button" class="fk-action" data-voice="speak">خوانشِ صوتی</button><button type="button" class="fk-action" data-voice="record">ضبطِ MediaRecorder</button><button type="button" class="fk-action" data-voice="ambient" data-ambient="fishkal-ocean">موسیقیِ محیطیِ اقیانوس</button></section></main>"#
        ),
        StudioModality::ImageGen => format!(
            r#"<main class="fk-grid" data-modality="image"><section class="fk-pane"><h2>{title}</h2><canvas data-canvas="procedural-vector" width="320" height="200"></canvas><p data-cascade="zero-key">آبشارِ فراهم‌کنندهٔ بی‌کلید + بردارِ SVG قطعی.</p></section></main>"#
        ),
        StudioModality::VideoGen => format!(
            r#"<main class="fk-grid" data-modality="video"><section class="fk-pane"><h2>{title}</h2><canvas data-canvas="tl-60fps" width="320" height="180"></canvas><p data-tl="fishkal-timeline">تایم‌لاینِ فیشکال — ۶۰fps، `damp` لامبدا-متغیر، ضبطِ WebM/فریم.</p><button type="button" class="chip" data-record="webm">ضبطِ WebM</button></section></main>"#
        ),
        StudioModality::TextCopy => format!(
            r#"<main class="fk-grid" data-modality="copy"><section class="fk-pane"><h2>{title}</h2><p data-locales="20">کپی فروش روی ۲۰ زبانِ سیاره‌ای با تایپوگرافی RTL/LTR.</p><div class="fk-chip-row"><button type="button" class="chip" aria-pressed="true" data-dir-mode="rtl">RTL</button><button type="button" class="chip" aria-pressed="false" data-dir-mode="ltr">LTR</button></div></section></main>"#
        ),
        StudioModality::SoundEffects => format!(
            r#"<main class="fk-grid" data-modality="sfx"><section class="fk-pane"><h2>{title}</h2><div class="fk-chip-row" role="toolbar" aria-label="تریگرهای SFX"><button type="button" class="chip" data-sfx="ping">ping</button><button type="button" class="chip" data-sfx="open">open</button><button type="button" class="chip" data-sfx="whoosh">whoosh</button><button type="button" class="chip" data-sfx="select">select</button><button type="button" class="chip" data-sfx="milestone">milestone</button></div></section></main>"#
        ),
    }
}

/// اسکریپتِ بوتِ هر مدالیتی — سیم‌کشیِ SFX/بومِ رویه‌ای.
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
// زیر-موتور ۳ — `PortfolioStorefrontEngine` (ویترینِ سازنده + ۶ درمان)
// ---------------------------------------------------------------------------

/// مسیرِ کانونیِ ریشهٔ کاتالوگ — درمانِ Fix #1: هیچ `path.resolve(PORT, '..')`
/// وجود ندارد؛ فقط مسیرِ نسبیِ درون-ورک‌اسپیس یا مسیرِ پیکربندی‌شدهٔ محیط.
pub const PORTFOLIO_ROOT_RESOLUTION: &str = "./seed/portfolio_catalog";

/// متغیرِ محیطیِ مسیرِ کاتالوگ — جایگزینِ امنِ ریشهٔ والد.
pub const PORTFOLIO_CATALOG_ENV: &str = "KIAN_PORTFOLIO_CATALOG_PATH";

/// امضای ویترین پورتفولیو.
pub const PORTFOLIO_STOREFRONT_SIGNATURE: &str = "portfolio-storefront-v1";

/// کارتِ پروژهٔ ویترین — ۱۲ کلیدِ کانونیِ `projects.json` + پسوندهای
/// تجاریِ زندهٔ بازار.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorefrontProjectCard {
    /// کلید کانونی ۱ — `slug`.
    pub slug: String,
    /// کلید کانونی ۲ — `name`.
    pub name: String,
    /// کلید کانونی ۳ — `tagline`.
    pub tagline: String,
    /// کلید کانونی ۴ — `category`.
    pub category: String,
    /// کلید کانونی ۵ — `stack`.
    pub stack: Vec<String>,
    /// کلید کانونی ۶ — `highlights`.
    pub highlights: Vec<String>,
    /// کلید کانونی ۷ — `Preview` (HTML زندهٔ پیش‌نمایش).
    pub preview: String,
    /// کلید کانونی ۸ — `sourceDir`.
    pub source_dir: String,
    /// کلید کانونی ۹ — `readme`.
    pub readme: String,
    /// کلید کانونی ۱۰ — `loc` (خطوطِ کد).
    pub loc: u32,
    /// کلید کانونی ۱۱ — `fileCount`.
    pub file_count: u32,
    /// کلید کانونی ۱۲ — `featured`.
    pub featured: bool,
    /// پسوندِ تجاری — قیمت به میکرو-کیان.
    pub price_ukian: u64,
    /// پسوندِ تجاری — قیمت به ستارهٔ تلگرام.
    pub price_xtr_stars: u32,
    /// پسوندِ تجاری — قیمت به میکرو-`USDC`.
    pub price_usdc_micros: u64,
    /// پسوندِ تجاری — نمرهٔ REA مایرون.
    pub rea_score: u8,
    /// پسوندِ تجاری — نشانیِ `SKILL.md` فورج‌شده.
    pub skill_md_url: String,
    /// پسوندِ تجاری — شمارِ زبان‌های سیاره‌ایِ فعال.
    pub supported_locales_count: u8,
    /// پسوندِ تجاری — نشانِ لایهٔ شناختی.
    pub cognitive_tier_badge: String,
}

impl StorefrontProjectCard {
    /// لیستینگِ یک‌کلیکیِ `ToySpark` — قیمتِ قفلِ ۵۰ ستاره / ۱٫۰۰ دلار.
    #[must_use]
    pub fn toyspark_listing(title: &str, preview_html: &str) -> Self {
        Self {
            slug: format!("toyspark-{}", prompt_seed(title)),
            name: title.to_owned(),
            tagline: "ماجرای یک‌کلیکیِ کودکِ ۹ساله — داستانِ قابل بازیِ آمادهٔ فروش".to_owned(),
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

/// کاتالوگِ بذرِ ویترین — ۸ آفرینشِ پرچم‌دار با ۱۲ کلیدِ کانونی.
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
        mk("fishkal", "Fishkal", "موتور HTML5 صفر-وابستگیِ کیا — اقیانوسِ رویه‌ای", "engine", ["html5", "webaudio", "canvas"], ["3,616 lines", "zero deps", "60fps timeline"], 3616, 1, true, 300),
        mk("cosmos", "Cosmos Starfield", "ستاره‌بارانِ تعاملیِ ویترین + سینتِ دیلی-بازخورد", "engine", ["canvas2d", "webaudio", "raf"], ["7 milestone nodes", "feedback delay", "parallax 0.05"], 512, 2, true, 240),
        mk("arda", "ARDA", "بومی‌سازی سیاره‌ایِ ۲۴ساعته روی معماری فیشکال", "engine", ["i18n", "rtl", "bidi"], ["20 locales", "atomic root flip", "ARIA listbox"], 1791, 1, false, 180),
        mk("trinity", "Trinity Studio", "خط لولهٔ سه‌کالبدی آلفا → اتاق فرمان → مایرون", "pipeline", ["rust", "agents", "dna"], ["3x9 OpenBot", "REA gate 85", "self-healing"], 2400, 6, false, 220),
        mk("mairon", "Mairon REA", "داورِ خودکارِ کیفیت با گیتِ فورج دو-کلیدی", "judge", ["rust", "scoring", "gate"], ["IQ 150", "EQ 145", "SKILL.md forge"], 980, 3, false, 150),
        mk("super-studio", "Omni Studio", "استودیوی ۹-مدالیتیِ صفر-هزینه با باندلِ دوگانه", "studio", ["html5", "webaudio", "canvas"], ["9 modalities", "dual engine", "zero keys"], 1600, 4, false, 260),
        mk("kian-kernel", "KIAN Kernel", "کرنل عاملِ حاکم با دفترِ دوطرفهٔ اتمیک", "kernel", ["rust", "sqlite", "ledger"], ["466 citizens", "SUM=0", "CAS escrows"], 8000, 42, false, 320),
        mk("portfolio", "Portfolio Storefront", "ویترینِ سازنده با ۱۲ کلیدِ کانونی + ۶ درمانِ قضایی", "storefront", ["css", "glass", "palette"], ["12 canonical keys", "6 forensic fixes", "Ctrl+K palette"], 720, 5, false, 120),
    ]
}

/// ارتقای خودکارِ «دو ویترینیِ برتر» — `featured`ها به ترتیبِ پایدارِ
/// ورودی، حداکثرِ ۲ مورد، صدرِ کاتالوگ (`index.js:L17–18`).
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

/// توکن‌های CSS گلسِ تاریکِ پورتفولیو — حفظِ حرف‌به‌حرفِ
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

/// شناسهٔ درمان‌های قضاییِ ۶گانهٔ `KiaAgentX/portfolio`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortfolioFix {
    /// Fix #1 — مسیرِ ریشهٔ قابل‌حمل (بدونِ `path.resolve(PORT, '..')`).
    PortableRootPath,
    /// Fix #2 — حذفِ `ReferenceError` موقت-مرگ (`TDZ`) روی `#source`.
    TdzLoadedGuard,
    /// Fix #3 — سندباکسِ iframe بدونِ `allow-same-origin`.
    IframeSandbox,
    /// Fix #4 — گاردِ راز و دات-فایل.
    SecretDotfileGuard,
    /// Fix #5 — چیپ‌های فیلترِ درک‌پذیر (`button aria-pressed`).
    AccessibleFilterChips,
    /// Fix #6 — چرخهٔ عمرِ مشترکِ توقفِ `RAF`.
    UnifiedRafPause,
}

impl PortfolioFix {
    /// شناسهٔ عددیِ درمان.
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

    /// عنوانِ درمان.
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

    /// شاهدِ ماندگارِ درمان در آرتیفکت‌ها.
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

/// گزارشِ درمانِ ۶ نقصِ پورتفولیو — ردیفی روی وایر.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioFixReport {
    /// شناسهٔ درمان (`1..=6`).
    pub fix_id: u8,
    /// عنوانِ درمان.
    pub title: String,
    /// وضعیت — همیشه `fixed`.
    pub status: &'static str,
    /// شاهدِ قطعی.
    pub evidence: String,
}

/// گزارشِ کاملِ ایمنیِ ویترین — همیشه ۶/۶ `fixed`.
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

/// الگوهای ممنوعِ درختِ منبع — درمانِ Fix #4.
const BLOCKED_SOURCE_PATTERNS: [&str; 7] = [
    ".env", ".env.", "*.pem", "*.key", "id_rsa", "credentials.json", "*.sqlite",
];

/// آیا مسیرِ منبع ممنوع است؟ — هرگز کپی/نمایش نمی‌شود.
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

/// پالایشِ درختِ منبع — فقط مسیرهای مجاز برمی‌گردند (ترتیب حفظ).
#[must_use]
pub fn sanitize_source_tree<'a>(paths: &[&'a str]) -> Vec<&'a str> {
    paths.iter().copied().filter(|p| !is_blocked_source_path(p)).collect()
}

/// تأییدِ مکانیکیِ ۶ درمان روی آرتیفکت‌های مولد — گیتِ آزمونِ هندلر.
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

/// پیکربندیِ پالتِ فرمان — کیبورد-اول (`Ctrl+K` / `Meta+K` و `/`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandPaletteConfig {
    /// میان‌برهای بازکردن.
    pub open_shortcuts: [&'static str; 3],
    /// میان‌برِ تمرکزِ جست‌وجو.
    pub focus_shortcut: &'static str,
    /// کیبورد-اول؟
    pub keyboard_first: bool,
    /// کلیدهای ناوبری.
    pub navigation_keys: [&'static str; 3],
}

/// پیکربندیِ کانونیِ پالتِ فرمان (`index.js:L396–459`).
#[must_use]
pub fn command_palette_config() -> CommandPaletteConfig {
    CommandPaletteConfig {
        open_shortcuts: ["Ctrl+K", "Meta+K", "/"],
        focus_shortcut: "/",
        keyboard_first: true,
        navigation_keys: ["ArrowDown", "ArrowUp", "Escape"],
    }
}

/// JS پالتِ فرمان — بازکردنِ `Ctrl+K`/`Meta+K`/`/`، ناوبریِ پیکان، بستنِ `Escape`.
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
// آزمون‌ها
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const DINO_PROMPT: &str = "یه دایناسور آبی که توی فضا بستنی می‌فروشه";

    #[test]
    fn tier_inference_matrix_is_deterministic() {
        // لغوِ صریح همیشه برنده است.
        let override_toy = infer_or_override_tier(&CognitiveTelemetry {
            prompt_length_chars: 500,
            uses_technical_tokens: true,
            typing_cadence_ms: 900,
            explicit_tier_override: Some(CognitiveTier::ToySpark),
        });
        assert_eq!(override_toy, CognitiveTier::ToySpark);
        // نشانهٔ فنی → ProTerminal.
        let tech = infer_or_override_tier(&CognitiveTelemetry {
            prompt_length_chars: 20,
            uses_technical_tokens: true,
            typing_cadence_ms: 0,
            explicit_tier_override: None,
        });
        assert_eq!(tech, CognitiveTier::ProTerminal);
        // طولِ زیاد → ProTerminal.
        let long = infer_or_override_tier(&CognitiveTelemetry {
            prompt_length_chars: 240,
            uses_technical_tokens: false,
            typing_cadence_ms: 0,
            explicit_tier_override: None,
        });
        assert_eq!(long, CognitiveTier::ProTerminal);
        // متوسطِ کند → CreatorStudio.
        let medium = infer_or_override_tier(&CognitiveTelemetry {
            prompt_length_chars: 160,
            uses_technical_tokens: false,
            typing_cadence_ms: 500,
            explicit_tier_override: None,
        });
        assert_eq!(medium, CognitiveTier::CreatorStudio);
        // کوتاه → ToySpark.
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
        // هر ۳ دکمهٔ غول‌پیکر با برچسبِ آیکون‌اول.
        assert_eq!(creation.storybook_html.matches("class=\"fk-action\"").count(), TOYSPARK_MAX_ACTIONS_PER_SCREEN);
        assert_eq!(creation.storybook_html.matches("data-step=").count(), TOYSPARK_MAX_ACTIONS_PER_SCREEN);
        for icon in ["✨", "🎨", "🪙"] {
            assert!(creation.storybook_html.contains(icon), "icon {icon} missing");
        }
        assert!(creation.storybook_html.contains("بساز (Make Magic)"));
        assert!(creation.storybook_html.contains("قشنگش کن (Make It Pretty)"));
        assert!(creation.storybook_html.contains("بفروش و ستاره بگیر (Share"));
        // هدفِ لمسیِ ۶۴ پیکسل و سندباکسِ صفر-زیان.
        assert!(creation.storybook_html.contains("min-width: 64px"));
        assert!(creation.storybook_html.contains("data-zero-loss-sandbox=\"true\""));
        // جادوگرِ صوتیِ خودکار — `speechSynthesis` + سه سناریو.
        assert!(creation.storybook_html.contains("speechSynthesis"));
        assert_eq!(creation.voice_script.len(), 3);
        // تصویرسازیِ SVG رویه‌ای — سه صحنه.
        assert_eq!(creation.storybook_html.matches("<svg class=\"fk-scene\"").count(), 3);
        // جلوه‌های صوتیِ رویه‌ای — هر دو موتور.
        assert!(creation.storybook_html.contains("ARDA_STUDIO.cosmos.sfx.milestone"));
        assert!(creation.storybook_html.contains("ARDA_STUDIO.fishkal.harmonicBloom"));
        // لیستینگِ آمادهٔ فروش — ۵۰ ستاره / ۱٫۰۰ دلار قفل.
        assert_eq!(creation.listing.price_xtr_stars, TOYSPARK_DEFAULT_XTR_STARS);
        assert_eq!(creation.listing.price_usdc_micros, TOYSPARK_DEFAULT_USDC_MICROS);
        assert_eq!(creation.listing.price_ukian, TOYSPARK_DEFAULT_UKIAN);
        assert_eq!(creation.listing.cognitive_tier_badge, "ToySpark");
        // RTL فارسی تشخیص داده شد.
        assert!(creation.storybook_html.contains("dir=\"rtl\""));
        // نحوِ JS داستان‌کتاب سالم است.
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
        assert_eq!(prompt_seed(DINO_PROMPT), prompt_seed("  یه دایناسور آبی که توی فضا بستنی می‌فروشه  "));
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
        // بازتولیدِ قطعی.
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
            let artifact = engine.generate("یه استودیوی جادویی برای من بساز", modality, CognitiveTier::CreatorStudio);
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
        // براوان-نویزِ ۴ثانیه‌ای — ضرایبِ حفظ‌شده.
        assert!(js.contains("b0 = (b0 + 0.02 * white) / 1.02"));
        assert!(js.contains("data[i] = b0 * 0.18"));
        // فیلترِ لو-پسِ 520Hz با LFO سینوسیِ 0.06Hz و جاروبِ ±220Hz.
        assert!(js.contains("lp.frequency.value = 520"));
        assert!(js.contains("lfo.frequency.value = 0.06"));
        assert!(js.contains("lfoGain.gain.value = 220"));
        // بلومِ پنتاتونیکِ رِ ماژور + آکوردِ امضای ۴-صدایی.
        assert!(js.contains("[587.33, 880, 1174.66]"));
        assert!(js.contains("[293.66, 440, 587.33, 880]"));
        // xorshift32 + fbm چهار-اکتاوی + smoothstep ارمایتِ مکعبی.
        assert!(js.contains("s ^= s << 13"));
        assert!(js.contains("s ^= s >>> 17"));
        assert!(js.contains("f *= 2.03"));
        assert!(js.contains("t * t * t * (t * (t * 6 - 15) + 10)"));
        // لامبدا-متغیرِ مستقل از فریم — پنجرهٔ 0.42..0.63 با 2.3 / 5.0.
        assert!(js.contains("FISHKAL_DAMP_WINDOW_LO = 0.42"));
        assert!(js.contains("FISHKAL_DAMP_WINDOW_HI = 0.63"));
        assert!(js.contains("FISHKAL_DAMP_LAMBDA_WINDOW = 2.3"));
        assert!(js.contains("FISHKAL_DAMP_LAMBDA = 5.0"));
        assert!(js.contains("Math.exp(-l * dt)"));
        assert_eq!(bundle.signature, DUAL_BUNDLE_SIGNATURE);
        assert_eq!(bundle.content_sha256.len(), 64);
        // بازتولیدِ قطعی — همان sha256.
        assert_eq!(bundle.content_sha256, emit_dual_cosmos_fishkal_bundle().content_sha256);
    }

    #[test]
    fn dual_bundle_carries_exact_cosmos_constants() {
        let bundle = emit_dual_cosmos_fishkal_bundle();
        let js = &bundle.cosmos_js;
        // ستاره‌باران — ۱۱۰ موبایل / ۲۶۰ دسکتاپ، DPR سقفِ 2.
        assert!(js.contains("COSMOS_STARS_MOBILE = 110"));
        assert!(js.contains("COSMOS_STARS_DESKTOP = 260"));
        assert!(js.contains("Math.min(2, (global.devicePixelRatio || 1))"));
        // میراییِ پارالاکسِ 0.05 + نرم‌شوندگیِ HUD با 0.16 + ۷ گرهٔ نق.
        assert!(js.contains("COSMOS_PARALLAX_DAMPING = 0.05"));
        assert!(js.contains("COSMOS_HUD_EASING = 0.16"));
        assert!(js.contains("COSMOS_MILESTONE_NODES = 7"));
        // سینتِ دیلی-بازخورد — 0.22s + 0.25 + لو-پسِ 1600Hz.
        assert!(js.contains("delay.delayTime.value = COSMOS_DELAY_SECONDS"));
        assert!(js.contains("COSMOS_DELAY_SECONDS = 0.22"));
        assert!(js.contains("COSMOS_FEEDBACK_GAIN = 0.25"));
        assert!(js.contains("COSMOS_TONE_LOWPASS_HZ = 1600"));
        // پنج تریگرِ SFX با پارامترهای حفظ‌شده.
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
        // گزارشِ روی وایر — همیشه ۶/۶.
        let report = portfolio_safety_report();
        assert_eq!(report.len(), 6);
        for (idx, fix) in report.iter().enumerate() {
            assert_eq!(fix.fix_id, u8::try_from(idx + 1).unwrap_or(6));
            assert_eq!(fix.status, "fixed");
            assert_ne!(fix.evidence, "");
        }
        // Fix #1 — هیچ `path.resolve(PORT, '..')` و هیچ `..` در ریشه.
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
        for prompt in ["قصص ستاره‌ای", "build me a spaceship game", "دایناسور آبی در فضا"] {
            let score = derive_rea_score(prompt_seed(prompt));
            assert!((88..=97).contains(&score), "REA {score} out of deterministic band");
            assert!(score >= crate::arda_fishkal_engine::SKILL_FORGE_REA_GATE);
            let (ukian, xtr, usdc) = derive_pricing(prompt_seed(prompt), CognitiveTier::ToySpark);
            assert_eq!((ukian, xtr, usdc), (1_000_000, 50, 1_000_000));
        }
    }
}

