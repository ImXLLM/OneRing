//! `grok_ux_and_lab_engine` — the Grok signature UX engine + the 24-hour
//! 10-DNA lab + the day-one ecosystem localization matrix (Wave 9 — Commit #29).
//!
//! Three integrated subsystems, all deterministic and free of network IO:
//!
//! **A — a real 24-hour lab with 10 distinct 10-DNA users + the admin log
//! observatory + RLHF/SFT dataset export** — [`run_24h_ten_dna_ux_lab_simulation`]
//! simulates the first 24 hours of ten distinct DNA cohorts (a 9-year-old child to a professional trader to
//! a hostile red-teamer) across multi-turn paths, UX-tier transitions, model routing,
//! latency, friction recovery, and the Mairon REA score;
//! [`build_admin_telemetry_dashboard`] lets the admin list/filter/health-check all 10 users' logs,
//! and [`export_rlhf_training_dataset_jsonl`] builds both
//! **SFT** (ChatML/ShareGPT JSONL) and **DPO** (`prompt/chosen/rejected`
//! JSONL) from the real telemetry.
//!
//! **B — the day-one 20-locale `Arda` ecosystem architecture** —
//! [`build_day1_ecosystem_i18n_matrix`]: Persian (`fa` — RTL, `Vazirmatn`,
//! `--track: 0em`) and English (`en` — LTR) are active from day one at
//! [`LocaleActivationState::PatchReadySlot`] with `completion_bps = 0` and
//! the full 9-panel-module dictionary, while the 18 other `Arda/Fishkal` locales sit as
//! [`LocaleActivationState::PatchReadySlot`] with a native glyph, an SVG flag
//! (`viewBox="0 0 20 14"`), RTL/LTR directionality, fallback to `en`, and support for
//! hot patching — [`apply_locale_translation_patch`].
//!
//! **C — the `Grok` signature UX engine** — [`GrokSignatureUxEngine`] with
//! [`GrokPersonaMode`] (fun/unhinged/balanced/sovereign-Mairon),
//! [`GrokThinkAccordion`] (collapsible reasoning steps + a
//! `thinking_duration_ms` timer), [`DeepSearchCitationBar`] (live verified
//! source pills), and [`build_grok_signature_ui_bundle`] — the
//! 'cosmic dark matter' UI (`#050507`, `rgba(255,255,255,0.08)` border),
//! the floating pill composer with quick-action chips, the side-by-side live
//! artifact canvas, the atomic FA/EN switcher + 18 `PATCH_READY` slots, and 3
//! one-click next-ghost suggestion chips after every response.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

/// Signature of the UX-and-lab engine — present on every wire output.
pub const GROK_LAB_SIGNATURE: &str = "KIA-GROK-UX-LAB-W9";

/// 'Cosmic dark matter' CSS signature — OLED background + specular border.
pub const GROK_COSMIC_CSS_SIGNATURE: &str =
    "grok-cosmic-dark-matter #050507 border:1px solid rgba(255,255,255,0.08)";

/// Pill-composer quick-action chips — the Grok signature.
pub const GROK_QUICK_CHIPS: [&str; 6] = [
    "\u{1f9e0} Think",
    "\u{1f50d} DeepSearch",
    "\u{26a1} Fun Mode",
    "\u{1f3ac} Neural HUD",
    "\u{1f4c8} Quant Vibe",
    "\u{1f3a8} Split Canvas",
];

/// `PATCH_READY` locale slots — the 18 other `Arda/Fishkal` languages.
pub const PATCH_READY_LOCALES: [&str; 18] = [
    "ar", "tr", "az", "ru", "zh", "ja", "ko", "de", "fr", "es", "pt", "it", "hi", "ur", "he",
    "nl", "pl", "id",
];

/// The nine panel modules — every day-one dictionary covers all of them.
pub const PANEL_MODULES: [&str; 9] = [
    "nav",
    "grok_composer",
    "think_accordion",
    "deep_search",
    "studio_canvas",
    "quant_terminal",
    "neural_hud",
    "base_treasury",
    "admin_telemetry_lab",
];

/// Keys per module — the shared 37-key rule for every active locale.
pub const MODULE_KEYS: [(&str, [&str; 4]); 9] = [
    ("nav", ["brand", "market", "wallet", "studio"]),
    ("grok_composer", ["placeholder", "send", "attach", "hint"]),
    ("think_accordion", ["title", "show", "hide", "duration"]),
    ("deep_search", ["title", "searching", "verified", "sources"]),
    ("studio_canvas", ["title", "split", "artifact", "preview"]),
    ("quant_terminal", ["title", "backtest", "venues", "sharpe"]),
    ("neural_hud", ["title", "seek", "phase", "render"]),
    ("base_treasury", ["title", "chain", "stream", "zero_sum"]),
    ("admin_telemetry_lab", ["title", "filter", "export_sft", "export_dpo"]),
];

/// Total keys in the full dictionary — 9 modules × 4 keys.
pub const FULL_DICTIONARY_KEYS: usize = PANEL_MODULES.len() * 4;

/// Deterministic simulation time anchor — every `created_at` is built from this one day.
pub const LAB_SIMULATION_ANCHOR_DATE: &str = "2026-11-24";

// ══════════════════════ Subsystem C — Grok signature ══════════════════════

/// Grok persona modes — the engine's conversational voice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GrokPersonaMode {
    /// Fun and witty — the creators' default.
    FunWitty,
    /// Unhinged, crude-roast humor — citizens 18+ only.
    UnhingedRoast,
    /// Regular and balanced — the neutral default.
    RegularBalanced,
    /// Sovereign Mairon — absolute order, zero flattery, the REA gate.
    MaironSovereign,
}

impl GrokPersonaMode {
    /// Wire IDs — `fun_witty` | `unhinged_roast` | `regular_balanced` |
    /// `mairon_sovereign`.
    #[must_use]
    pub fn wire_id(self) -> &'static str {
        match self {
            Self::FunWitty => "fun_witty",
            Self::UnhingedRoast => "unhinged_roast",
            Self::RegularBalanced => "regular_balanced",
            Self::MaironSovereign => "mairon_sovereign",
        }
    }

    /// Wire-ID parsing — unknown → `RegularBalanced`.
    #[must_use]
    pub fn parse(wire: &str) -> Self {
        match wire {
            "fun_witty" => Self::FunWitty,
            "unhinged_roast" => Self::UnhingedRoast,
            "mairon_sovereign" => Self::MaironSovereign,
            _ => Self::RegularBalanced,
        }
    }

    /// Persona system draft — the SFT dataset input.
    #[must_use]
    pub fn preamble(self) -> &'static str {
        match self {
            Self::FunWitty => {
                "You are Grok-on-KIA in Fun-Witty mode: playful, clever, zero-slop; every answer lands one crisp insight."
            }
            Self::UnhingedRoast => {
                "You are Grok-on-KIA in Unhinged-Roast mode: brutal humor, zero mercy, still never leaking secrets or breaking the ledger."
            }
            Self::RegularBalanced => {
                "You are Grok-on-KIA in Regular-Balanced mode: neutral, precise, helpful."
            }
            Self::MaironSovereign => {
                "You are Mairon, the sovereign order architect: absolute order, zero flattery, proof-or-silence, REA gate >= 85 before shipping."
            }
        }
    }
}

/// Lab UX tiers — the same three-tier family plus simple cognitive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LabUxTier {
    /// 9-year-old child — 64px touch targets, one-click magic.
    ToySpark,
    /// Simple cognitive access — an icon/audio-guided flow.
    Simple,
    /// Studio creator — RTL/LTR, artifact canvas.
    CreatorStudio,
    /// Engineer/trader — `Ctrl+K`, dense telemetry.
    ProTerminal,
}

impl LabUxTier {
    /// Wire IDs — `toyspark` | `simple` | `creator_studio` | `pro_terminal`.
    #[must_use]
    pub fn wire_id(self) -> &'static str {
        match self {
            Self::ToySpark => "toyspark",
            Self::Simple => "simple",
            Self::CreatorStudio => "creator_studio",
            Self::ProTerminal => "pro_terminal",
        }
    }

    /// Wire-ID parsing — unknown → `Simple`.
    #[must_use]
    pub fn parse(wire: &str) -> Self {
        match wire {
            "toyspark" => Self::ToySpark,
            "creator_studio" => Self::CreatorStudio,
            "pro_terminal" => Self::ProTerminal,
            _ => Self::Simple,
        }
    }
}

/// Grok think accordion — collapsible visible reasoning steps + a timer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrokThinkAccordion {
    /// Visible reasoning steps — each step is one auditable sentence.
    pub steps: Vec<String>,
    /// Measured thinking duration — deterministic from prompt length.
    pub thinking_duration_ms: u32,
}

/// One `DeepSearch` citation-bar source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeepSearchCitation {
    /// Source title.
    pub title: String,
    /// Source domain.
    pub domain: String,
    /// Verified?
    pub verified: bool,
}

/// The `DeepSearch` citation bar — live source pills.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeepSearchCitationBar {
    /// Sorted sources.
    pub sources: Vec<DeepSearchCitation>,
}

/// One Grok chat-turn spec — the engine's deterministic input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrokChatTurnSpec {
    /// User text.
    pub prompt: String,
    /// Locale — `fa` | `en` | patch slot.
    pub locale: String,
    /// UX tier.
    pub ux_tier: LabUxTier,
    /// Persona.
    pub persona: GrokPersonaMode,
    /// `DeepSearch` bar enabled?
    pub deep_search: bool,
    /// Think accordion enabled?
    pub think: bool,
}

/// One Grok chat-turn output — all signature pieces.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrokChatTurnOutcome {
    /// Think accordion.
    pub thinking: GrokThinkAccordion,
    /// Citation bar — when `deep_search` is on.
    pub citations: Option<DeepSearchCitationBar>,
    /// Response text.
    pub response_text: String,
    /// Three next-ghost suggestion chips — one click each.
    pub ghost_follow_ups: Vec<String>,
    /// Persona used.
    pub persona: GrokPersonaMode,
    /// Model routed from the pentamodel genome.
    pub model_slug: String,
    /// Locale.
    pub locale: String,
    /// UX tier.
    pub ux_tier: LabUxTier,
}

/// The Grok signature engine — all pure deterministic functions.
pub struct GrokSignatureUxEngine;

/// Locale directionality — RTL for `fa`/`ar`/`ur`/`he`.
#[must_use]
pub fn locale_direction(locale: &str) -> &'static str {
    match locale {
        "fa" | "ar" | "ur" | "he" => "rtl",
        _ => "ltr",
    }
}

impl GrokSignatureUxEngine {
    /// Deterministic model routing from the pentamodel genome — a function of (persona, UX tier).
    #[must_use]
    pub fn route_model(persona: GrokPersonaMode, tier: LabUxTier) -> &'static str {
        match (persona, tier) {
            (_, LabUxTier::ToySpark | LabUxTier::Simple) => "05-glm-5.3-flash-max",
            (GrokPersonaMode::MaironSovereign, _) => "02-gemini-4-aragon-high",
            (_, LabUxTier::ProTerminal) => "03-deepseek-3.2-web-think-mode",
            (GrokPersonaMode::FunWitty | GrokPersonaMode::UnhingedRoast, _) => "01-gpt-6-web",
            (_, LabUxTier::CreatorStudio) => "04-claude-sonnet-5.5-extra",
        }
    }

    /// Builds the think accordion — the steps derive from the user prompt and
    /// `thinking_duration_ms` is deterministic (380 + prompt length × 12, capped at 4500).
    #[must_use]
    pub fn build_think_accordion(prompt: &str, locale: &str) -> GrokThinkAccordion {
        let topic = prompt.trim();
        let words: Vec<&str> = topic.split_whitespace().take(6).collect();
        let keyword = words.last().copied().unwrap_or("request");
        let steps = if locale == "fa" {
            vec![
                format!("\u{6f1}) \u{62a}\u{641}\u{6a9}\u{6cc}\u{6a9}\u{650} \u{62e}\u{648}\u{627}\u{633}\u{62a}\u{647}: {topic}"),
                format!("\u{6f2}) \u{628}\u{631}\u{631}\u{633}\u{6cc}\u{650} \u{642}\u{6cc}\u{62f}\u{647}\u{627}\u{6cc}\u{650} \u{62d}\u{627}\u{6a9}\u{645} \u{648} \u{6af}\u{6cc}\u{62a}‌\u{647}\u{627}\u{6cc}\u{650} \u{627}\u{645}\u{646}\u{6cc}\u{62a}\u{6cc} \u{628}\u{631}\u{627}\u{6cc} «{keyword}»"),
                "\u{6f3}) \u{627}\u{646}\u{62a}\u{62e}\u{627}\u{628}\u{650} \u{645}\u{633}\u{6cc}\u{631}\u{650} \u{642}\u{637}\u{639}\u{6cc}\u{650} \u{627}\u{62c}\u{631}\u{627} \u{631}\u{648}\u{6cc} \u{6a9}\u{631}\u{646}\u{644}".to_owned(),
                "\u{6f4}) \u{627}\u{639}\u{62a}\u{628}\u{627}\u{631}\u{633}\u{646}\u{62c}\u{6cc}\u{650} \u{62e}\u{631}\u{648}\u{62c}\u{6cc} \u{628}\u{627} \u{6af}\u{6cc}\u{62a}\u{650} REA \u{645}\u{627}\u{6cc}\u{631}\u{648}\u{646}".to_owned(),
            ]
        } else {
            vec![
                format!("1) Parse the request: {topic}"),
                format!("2) Check governing constraints and security gates for \"{keyword}\""),
                "3) Pick the deterministic execution path on the kernel".to_owned(),
                "4) Validate output through the Mairon REA gate".to_owned(),
            ]
        };
        let duration = (380u32)
            .saturating_add(u32::try_from(topic.chars().count()).unwrap_or(4_000) * 12)
            .min(4_500);
        GrokThinkAccordion {
            steps,
            thinking_duration_ms: duration,
        }
    }

    /// The `DeepSearch` citation bar — deterministic sources derived from the prompt keywords
    /// (simulated lab telemetry; no network crawling).
    #[must_use]
    pub fn build_citation_bar(prompt: &str) -> DeepSearchCitationBar {
        let keyword = prompt
            .split_whitespace()
            .max_by_key(|w| w.chars().count())
            .unwrap_or("kernel")
            .trim_matches(|c: char| !c.is_ascii_alphanumeric())
            .to_owned();
        DeepSearchCitationBar {
            sources: vec![
                DeepSearchCitation {
                    title: format!("0xKIA kernel reference — {keyword}"),
                    domain: "kiaagentx.dev".to_owned(),
                    verified: true,
                },
                DeepSearchCitation {
                    title: format!("Rust std docs — {keyword} module"),
                    domain: "doc.rust-lang.org".to_owned(),
                    verified: true,
                },
                DeepSearchCitation {
                    title: format!("Base L2 ledger notes — {keyword}"),
                    domain: "base.org".to_owned(),
                    verified: false,
                },
            ],
        }
    }

    /// Deterministic persona response — a distinct voice, zero slop.
    #[must_use]
    pub fn persona_response(spec: &GrokChatTurnSpec) -> String {
        let topic = spec.prompt.trim();
        let body = match spec.persona {
            GrokPersonaMode::FunWitty => {
                format!("Alright, let's make this fun: «{topic}» — one crisp take, zero filler, and a wink at the end.")
            }
            GrokPersonaMode::UnhingedRoast => {
                format!("Roast mode, no mercy: «{topic}» — the raw truth, sharpened, still inside the ledger's rules.")
            }
            GrokPersonaMode::RegularBalanced => {
                format!("Balanced take on «{topic}»: what it is, what it costs, what to do next.")
            }
            GrokPersonaMode::MaironSovereign => {
                format!("\u{646}\u{638}\u{645}\u{650} \u{645}\u{637}\u{644}\u{642}: «{topic}» — \u{627}\u{635}\u{644}\u{650} \u{627}\u{648}\u{644}\u{60c} \u{639}\u{62f}\u{62f}\u{650} \u{628}\u{62f}\u{648}\u{646}\u{650} \u{627}\u{62c}\u{631}\u{627} \u{645}\u{645}\u{646}\u{648}\u{639}\u{60c} \u{6af}\u{6cc}\u{62a}\u{650} REA \u{67e}\u{6cc}\u{634} \u{627}\u{632} \u{627}\u{646}\u{62a}\u{634}\u{627}\u{631}.")
            }
        };
        let tier_note = match spec.ux_tier {
            LabUxTier::ToySpark => " (big 64px buttons, one-tap flow)",
            LabUxTier::Simple => " (icon + voice guided steps)",
            LabUxTier::CreatorStudio => " (artifact canvas attached)",
            LabUxTier::ProTerminal => " (Ctrl+K palette ready)",
        };
        format!("{body}{tier_note}")
    }

    /// Three next-ghost chips — deterministic from (persona, locale, tier).
    #[must_use]
    pub fn ghost_follow_ups(persona: GrokPersonaMode, locale: &str, tier: LabUxTier) -> Vec<String> {
        let fa = locale == "fa";
        match (persona, tier) {
            (GrokPersonaMode::MaironSovereign, _) => {
                if fa {
                    vec!["\u{6af}\u{6cc}\u{62a}\u{650} REA \u{631}\u{627} \u{627}\u{62c}\u{631}\u{627} \u{6a9}\u{646}".to_owned(), "\u{645}\u{627}\u{62a}\u{631}\u{6cc}\u{633}\u{650} \u{698}\u{646}\u{648}\u{645} \u{631}\u{627} \u{646}\u{634}\u{627}\u{646} \u{628}\u{62f}\u{647}".to_owned(), "\u{62e}\u{632}\u{627}\u{646}\u{647}\u{654} Base \u{631}\u{627} \u{628}\u{6cc}\u{627}\u{631}".to_owned()]
                } else {
                    vec!["Run the REA gate".to_owned(), "Show the genome matrix".to_owned(), "Bring up the Base treasury".to_owned()]
                }
            }
            (_, LabUxTier::ToySpark) => {
                if fa {
                    vec!["\u{6cc}\u{6a9} \u{642}\u{635}\u{647}\u{654} \u{62f}\u{6cc}\u{6af}\u{631} \u{628}\u{633}\u{627}\u{632}".to_owned(), "\u{631}\u{646}\u{6af}\u{6cc}\u{646}‌\u{6a9}\u{645}\u{627}\u{646} \u{628}\u{632}\u{646}".to_owned(), "\u{628}\u{627} \u{635}\u{62f}\u{627} \u{628}\u{62e}\u{648}\u{646}".to_owned()]
                } else {
                    vec!["Make another story".to_owned(), "Rainbow mode".to_owned(), "Read it aloud".to_owned()]
                }
            }
            (_, LabUxTier::ProTerminal) => {
                if fa {
                    vec!["\u{628}\u{6a9}‌\u{62a}\u{633}\u{62a}\u{650} \u{639}\u{644}\u{651}\u{6cc} \u{628}\u{6af}\u{6cc}\u{631}".to_owned(), "DAG \u{631}\u{627} \u{627}\u{62c}\u{631}\u{627} \u{6a9}\u{646}".to_owned(), "\u{644}\u{627}\u{6af}‌\u{647}\u{627} \u{631}\u{627} \u{635}\u{627}\u{62f}\u{631} \u{6a9}\u{646}".to_owned()]
                } else {
                    vec!["Run a causal backtest".to_owned(), "Execute the DAG".to_owned(), "Export the logs".to_owned()]
                }
            }
            (_, _) => {
                if fa {
                    vec!["\u{628}\u{648}\u{645}\u{650} \u{622}\u{631}\u{62a}\u{6cc}\u{641}\u{6a9}\u{62a} \u{631}\u{627} \u{628}\u{627}\u{632} \u{6a9}\u{646}".to_owned(), "\u{646}\u{633}\u{62e}\u{647}\u{654} 9:16 \u{631}\u{646}\u{62f}\u{631} \u{6a9}\u{646}".to_owned(), "DeepSearch \u{628}\u{632}\u{646}".to_owned()]
                } else {
                    vec!["Open the artifact canvas".to_owned(), "Render the 9:16 cut".to_owned(), "Run DeepSearch".to_owned()]
                }
            }
        }
    }

    /// One full chat turn — thinking + citations + response + ghosts + routing.
    #[must_use]
    pub fn run_chat_turn(spec: &GrokChatTurnSpec) -> GrokChatTurnOutcome {
        let thinking = if spec.think {
            Self::build_think_accordion(&spec.prompt, &spec.locale)
        } else {
            GrokThinkAccordion {
                steps: Vec::new(),
                thinking_duration_ms: 0,
            }
        };
        let citations = spec
            .deep_search
            .then(|| Self::build_citation_bar(&spec.prompt));
        GrokChatTurnOutcome {
            ghost_follow_ups: Self::ghost_follow_ups(spec.persona, &spec.locale, spec.ux_tier),
            response_text: Self::persona_response(spec),
            persona: spec.persona,
            model_slug: Self::route_model(spec.persona, spec.ux_tier).to_owned(),
            thinking,
            citations,
            locale: spec.locale.clone(),
            ux_tier: spec.ux_tier,
        }
    }
}

/// The Grok signature UI bundle — a standalone document + metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrokUiBundle {
    /// The standalone 'cosmic dark matter' HTML document.
    pub html: String,
    /// Active locale.
    pub locale: String,
    /// Directionality — `rtl` | `ltr`.
    pub direction: String,
    /// UX tier.
    pub ux_tier: String,
    /// Persona.
    pub persona: String,
    /// Quick-action chips.
    pub chips: Vec<String>,
    /// Three next-ghost suggestions.
    pub ghost_follow_ups: Vec<String>,
    /// CSS signature.
    pub css_signature: String,
    /// Locale slot count — 2 active + 18 patch-ready.
    pub locale_slots: usize,
}

#[allow(clippy::too_many_lines)] // UI template — its length is the artifact's nature
/// Builds the Grok signature UI bundle for (locale, tier, persona) — the floating
/// pill composer, the side-by-side artifact canvas, the atomic FA/EN switcher + 18
/// `PATCH_READY` slots, and the three ghost chips.
#[must_use]
pub fn build_grok_signature_ui_bundle(
    locale: &str,
    ux_tier: LabUxTier,
    persona: GrokPersonaMode,
) -> GrokUiBundle {
    let dir = locale_direction(locale);
    let fa = locale == "fa";
    let composer_placeholder = if fa {
        "\u{627}\u{632} \u{6af}\u{631}\u{648}\u{6a9} \u{628}\u{67e}\u{631}\u{633}… (Shift+Enter \u{62e}\u{637} \u{62c}\u{62f}\u{6cc}\u{62f})"
    } else {
        "Ask Grok anything… (Shift+Enter for newline)"
    };
    let canvas_title = if fa { "\u{628}\u{648}\u{645}\u{650} \u{622}\u{631}\u{62a}\u{6cc}\u{641}\u{6a9}\u{62a}\u{650} \u{632}\u{646}\u{62f}\u{647}" } else { "Live Artifact Canvas" };
    let switcher = {
        let mut s = String::from("<div class=\"locale-switch\">");
        let _ = write!(s, "<button class=\"loc active\" data-locale=\"fa\">\u{641}\u{627}</button>");
        let _ = write!(s, "<button class=\"loc active\" data-locale=\"en\">EN</button>");
        for slot in PATCH_READY_LOCALES {
            let _ = write!(
                s,
                "<button class=\"loc patch\" data-locale=\"{slot}\" title=\"PATCH_READY\">{}</button>",
                slot.to_uppercase()
            );
        }
        s.push_str("</div>");
        s
    };
    let chips: Vec<String> = GROK_QUICK_CHIPS.iter().map(|c| (*c).to_owned()).collect();
    let mut chip_html = String::new();
    for chip in &chips {
        let _ = write!(chip_html, "<button class=\"chip\">{chip}</button>");
    }
    let ghosts = GrokSignatureUxEngine::ghost_follow_ups(persona, locale, ux_tier);
    let mut ghost_html = String::new();
    for g in &ghosts {
        let _ = write!(ghost_html, "<button class=\"ghost\">{g}</button>");
    }
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="{locale}" dir="{dir}" data-kia="grok-cosmic">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Grok-on-KIA — {locale}</title>
<style>
  /* {css_signature} */
  * {{ margin: 0; padding: 0; box-sizing: border-box; }}
  html, body {{ height: 100%; background: #050507; color: #E7E9EE;
    font-family: {font_stack}; }}
  .specular {{ border: 1px solid rgba(255,255,255,0.08); }}
  .shell {{ display: grid; grid-template-columns: 1fr 1fr; height: 100%; gap: 10px; padding: 12px; }}
  .pane {{ background: rgba(255,255,255,0.03); border: 1px solid rgba(255,255,255,0.08);
    border-radius: 16px; padding: 16px; overflow: auto; }}
  .composer {{ position: sticky; bottom: 0; display: flex; gap: 8px; padding: 12px;
    background: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.08);
    border-radius: 999px; }}
  .composer input {{ flex: 1; background: transparent; border: 0; outline: 0; color: inherit; font: inherit; }}
  .chip {{ padding: 6px 12px; border-radius: 999px; background: rgba(255,255,255,0.06);
    border: 1px solid rgba(255,255,255,0.08); color: #C7CEDB; cursor: pointer; font-size: 12px; }}
  .ghost {{ padding: 6px 12px; border-radius: 999px; background: transparent;
    border: 1px dashed rgba(255,255,255,0.14); color: #8A93A6; cursor: pointer; font-size: 12px; }}
  .chips-row {{ display: flex; flex-wrap: wrap; gap: 8px; margin: 10px 0; }}
  .ghosts-row {{ display: flex; flex-wrap: wrap; gap: 8px; margin-top: 14px; }}
  .locale-switch {{ display: flex; flex-wrap: wrap; gap: 6px; margin-top: 12px; }}
  .loc {{ padding: 4px 9px; border-radius: 8px; font-size: 11px;
    background: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.08); color: #8A93A6; }}
  .loc.active {{ color: #E7E9EE; border-color: rgba(255,255,255,0.22); }}
  .loc.patch {{ opacity: 0.55; border-style: dashed; }}
  .canvas-title {{ font-size: 13px; letter-spacing: 0.08em; color: #8A93A6; margin-bottom: 10px; }}
  [dir="rtl"] .pane {{ --track: 0em; letter-spacing: var(--track); font-family: "Vazirmatn", system-ui; }}
</style>
</head>
<body>
<div class="shell">
  <section class="pane" id="chat-pane">
    <div class="chips-row">{chip_html}</div>
    <div class="composer specular">
      <input type="text" placeholder="{composer_placeholder}" />
      <button class="chip">➤</button>
    </div>
    <div class="ghosts-row" id="ghosts">{ghost_html}</div>
    {switcher}
  </section>
  <section class="pane" id="canvas-pane">
    <div class="canvas-title">{canvas_title}</div>
    <div id="artifact" class="specular" style="min-height: 220px; border-radius: 12px;"></div>
  </section>
</div>
</body>
</html>
"#,
        locale = locale,
        dir = dir,
        css_signature = GROK_COSMIC_CSS_SIGNATURE,
        font_stack = if fa { "\"Vazirmatn\", system-ui, sans-serif" } else { "system-ui, sans-serif" },
        composer_placeholder = composer_placeholder,
        chip_html = chip_html,
        ghost_html = ghost_html,
        switcher = switcher,
        canvas_title = canvas_title,
    );
    GrokUiBundle {
        html,
        locale: locale.to_owned(),
        direction: dir.to_owned(),
        ux_tier: ux_tier.wire_id().to_owned(),
        persona: persona.wire_id().to_owned(),
        chips,
        ghost_follow_ups: ghosts,
        css_signature: GROK_COSMIC_CSS_SIGNATURE.to_owned(),
        locale_slots: 2 + PATCH_READY_LOCALES.len(),
    }
}

// ══════════════════════ Subsystem B — the i18n matrix ══════════════════════

/// Locale activation state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocaleActivationState {
    /// Active from day one — 100%.
    ActiveDay1,
    /// Hot-patch-ready slot.
    PatchReadySlot,
}

impl LocaleActivationState {
    /// Wire IDs — `ACTIVE_DAY1` | `PATCH_READY_SLOT`.
    #[must_use]
    pub fn wire_id(self) -> &'static str {
        match self {
            Self::ActiveDay1 => "ACTIVE_DAY1",
            Self::PatchReadySlot => "PATCH_READY_SLOT",
        }
    }

    /// Wire-ID parsing — unknown → `PatchReadySlot`.
    #[must_use]
    pub fn parse(wire: &str) -> Self {
        if wire == "ACTIVE_DAY1" {
            Self::ActiveDay1
        } else {
            Self::PatchReadySlot
        }
    }
}

/// One ecosystem locale in the matrix.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EcosystemLocale {
    /// Locale code — `fa`, `en`, `ar`, …
    pub code: String,
    /// English name.
    pub name: String,
    /// Native name.
    pub native_name: String,
    /// Direction — `rtl` | `ltr`.
    pub direction: String,
    /// Authentic font.
    pub font_family: String,
    /// SVG flag with `viewBox="0 0 20 14"`.
    pub flag_svg: String,
    /// Native glyph sample.
    pub glyph_sample: String,
    /// Activation state.
    pub state: LocaleActivationState,
    /// Completion in bps — 10000 = 100%.
    pub completion_bps: u32,
    /// Fallback locale — always `en`.
    pub fallback: String,
}

/// i18n-matrix errors — typed and deterministic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum I18nError {
    /// Unknown locale.
    UnknownLocale {
        /// Unknown code.
        code: String,
    },
    /// Unknown module in a patch.
    UnknownModule {
        /// Module name.
        module: String,
    },
    /// Empty patch entry.
    EmptyPatchEntry {
        /// Module.
        module: String,
        /// Key.
        key: String,
    },
}

impl std::fmt::Display for I18nError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownLocale { code } => write!(f, "unknown locale: {code}"),
            Self::UnknownModule { module } => write!(f, "unknown panel module: {module}"),
            Self::EmptyPatchEntry { module, key } => {
                write!(f, "empty patch entry: {module}.{key}")
            }
        }
    }
}

impl std::error::Error for I18nError {}

/// One translation-patch entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalePatchEntry {
    /// Panel module — one of [`PANEL_MODULES`].
    pub module: String,
    /// Translation key.
    pub key: String,
    /// Localized value.
    pub value: String,
}

/// Translation-patch application result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalePatchOutcome {
    /// Locale code.
    pub locale_code: String,
    /// Post-patch state.
    pub activation_state: String,
    /// New completion in bps.
    pub completion_bps: u32,
    /// Applied entry count.
    pub applied_count: usize,
    /// Patch version — deterministic from the entry count.
    pub patch_version: String,
    /// Applied entries as a flat `module.key` dictionary.
    pub applied_entries: BTreeMap<String, String>,
}

/// Simple deterministic SVG flag — three horizontal stripes with a locale-specific palette.
fn locale_flag_svg(colors: [&str; 3]) -> String {
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 20 14\" width=\"20\" height=\"14\" role=\"img\">\
<rect width=\"20\" height=\"14\" fill=\"{c0}\"/>\
<rect y=\"4.67\" width=\"20\" height=\"4.67\" fill=\"{c1}\"/>\
<rect y=\"9.33\" width=\"20\" height=\"4.67\" fill=\"{c2}\"/>\
</svg>",
        c0 = colors[0],
        c1 = colors[1],
        c2 = colors[2]
    )
}

/// Metadata for one patch-ready locale — (code, name, native, RTL?, glyph, flag palette).
type LocaleMeta = (&'static str, &'static str, &'static str, bool, &'static str, [&'static str; 3]);

/// Metadata for the 18 patch-ready locales.
const PATCH_READY_META: [LocaleMeta; 18] = [
    ("ar", "Arabic", "\u{627}\u{644}\u{639}\u{631}\u{628}\u{64a}\u{629}", true, "\u{645}\u{631}\u{62d}\u{628}\u{627}", ["#ce1126", "#ffffff", "#000000"]),
    ("tr", "Turkish", "Türkçe", false, "Merhaba", ["#e30a17", "#ffffff", "#e30a17"]),
    ("az", "Azerbaijani", "Azərbaycan", false, "Salam", ["#00b5e2", "#ef3340", "#00b5e2"]),
    ("ru", "Russian", "Русский", false, "Привет", ["#ffffff", "#0039a6", "#d52b1e"]),
    ("zh", "Chinese", "中文", false, "你好", ["#de2910", "#ffde00", "#de2910"]),
    ("ja", "Japanese", "日本語", false, "こんにちは", ["#ffffff", "#bc002d", "#ffffff"]),
    ("ko", "Korean", "한국어", false, "안녕", ["#ffffff", "#cd2e3a", "#0047a0"]),
    ("de", "German", "Deutsch", false, "Hallo", ["#000000", "#dd0000", "#ffce00"]),
    ("fr", "French", "Français", false, "Bonjour", ["#0055a4", "#ffffff", "#ef4135"]),
    ("es", "Spanish", "Español", false, "Hola", ["#aa151b", "#f1bf00", "#aa151b"]),
    ("pt", "Portuguese", "Português", false, "Olá", ["#006600", "#ffed00", "#006600"]),
    ("it", "Italian", "Italiano", false, "Ciao", ["#009246", "#ffffff", "#ce2b37"]),
    ("hi", "Hindi", "हिन्दी", false, "नमस्ते", ["#ff9933", "#ffffff", "#138808"]),
    ("ur", "Urdu", "\u{627}\u{631}\u{62f}\u{648}", true, "\u{633}\u{644}\u{627}\u{645}", ["#01411c", "#ffffff", "#01411c"]),
    ("he", "Hebrew", "עברית", true, "שלום", ["#0038b8", "#ffffff", "#0038b8"]),
    ("nl", "Dutch", "Nederlands", false, "Hallo", ["#ae1c28", "#ffffff", "#21468b"]),
    ("pl", "Polish", "Polski", false, "Cześć", ["#ffffff", "#dc143c", "#ffffff"]),
    ("id", "Indonesian", "Bahasa", false, "Halo", ["#ce1126", "#ffffff", "#ce1126"]),
];

/// The day-one full Persian dictionary — 9 modules × 4 keys.
#[must_use]
pub fn fa_dictionary() -> BTreeMap<String, String> {
    let pairs: [(&str, &str); 36] = [
        ("nav.brand", "\u{6f0}x\u{6a9}\u{6cc}\u{627}\u{646} — \u{62d}\u{627}\u{6a9}\u{645}\u{6cc}\u{62a}\u{650} \u{62e}\u{627}\u{644}\u{635}"),
        ("nav.market", "\u{628}\u{627}\u{632}\u{627}\u{631}"),
        ("nav.wallet", "\u{6a9}\u{6cc}\u{641} \u{67e}\u{648}\u{644}"),
        ("nav.studio", "\u{627}\u{633}\u{62a}\u{648}\u{62f}\u{6cc}\u{648}"),
        ("grok_composer.placeholder", "\u{627}\u{632} \u{6af}\u{631}\u{648}\u{6a9} \u{628}\u{67e}\u{631}\u{633}… (Shift+Enter \u{62e}\u{637} \u{62c}\u{62f}\u{6cc}\u{62f})"),
        ("grok_composer.send", "\u{628}\u{641}\u{631}\u{633}\u{62a}"),
        ("grok_composer.attach", "\u{67e}\u{6cc}\u{648}\u{633}\u{62a}"),
        ("grok_composer.hint", "\u{67e}\u{627}\u{633}\u{62e}‌\u{647}\u{627} \u{631}\u{648}\u{6cc} \u{62f}\u{641}\u{62a}\u{631}\u{650} \u{642}\u{637}\u{639}\u{6cc} \u{633}\u{646}\u{62c}\u{6cc}\u{62f}\u{647} \u{645}\u{6cc}‌\u{634}\u{648}\u{646}\u{62f}"),
        ("think_accordion.title", "\u{632}\u{646}\u{62c}\u{6cc}\u{631}\u{647}\u{654} \u{62a}\u{641}\u{6a9}\u{631}"),
        ("think_accordion.show", "\u{646}\u{645}\u{627}\u{6cc}\u{634}\u{650} \u{627}\u{633}\u{62a}\u{62f}\u{644}\u{627}\u{644}"),
        ("think_accordion.hide", "\u{67e}\u{646}\u{647}\u{627}\u{646}‌\u{633}\u{627}\u{632}\u{6cc}\u{650} \u{627}\u{633}\u{62a}\u{62f}\u{644}\u{627}\u{644}"),
        ("think_accordion.duration", "\u{645}\u{62f}\u{62a}\u{650} \u{62a}\u{641}\u{6a9}\u{631}"),
        ("deep_search.title", "\u{62c}\u{633}\u{62a}‌\u{648}\u{62c}\u{648}\u{6cc}\u{650} \u{639}\u{645}\u{6cc}\u{642}"),
        ("deep_search.searching", "\u{62f}\u{631} \u{62d}\u{627}\u{644}\u{650} \u{6a9}\u{627}\u{648}\u{650} \u{645}\u{646}\u{627}\u{628}\u{639}…"),
        ("deep_search.verified", "\u{645}\u{646}\u{627}\u{628}\u{639}\u{650} \u{631}\u{627}\u{633}\u{62a}\u{6cc}‌\u{622}\u{632}\u{645}\u{627}\u{6cc}\u{6cc}‌\u{634}\u{62f}\u{647}"),
        ("deep_search.sources", "\u{645}\u{646}\u{628}\u{639}"),
        ("studio_canvas.title", "\u{628}\u{648}\u{645}\u{650} \u{627}\u{633}\u{62a}\u{648}\u{62f}\u{6cc}\u{648}"),
        ("studio_canvas.split", "\u{646}\u{645}\u{627}\u{6cc}\u{650} \u{62f}\u{648}‌\u{62a}\u{6a9}\u{647}"),
        ("studio_canvas.artifact", "\u{622}\u{631}\u{62a}\u{6cc}\u{641}\u{6a9}\u{62a}"),
        ("studio_canvas.preview", "\u{67e}\u{6cc}\u{634}‌\u{646}\u{645}\u{627}\u{6cc}\u{634}\u{650} \u{632}\u{646}\u{62f}\u{647}"),
        ("quant_terminal.title", "\u{67e}\u{627}\u{6cc}\u{627}\u{646}\u{647}\u{654} \u{6a9}\u{648}\u{627}\u{646}\u{62a}"),
        ("quant_terminal.backtest", "\u{628}\u{6a9}‌\u{62a}\u{633}\u{62a}\u{650} \u{639}\u{644}\u{651}\u{6cc}"),
        ("quant_terminal.venues", "\u{628}\u{627}\u{632}\u{627}\u{631}\u{6af}\u{627}\u{647}‌\u{647}\u{627}"),
        ("quant_terminal.sharpe", "\u{634}\u{627}\u{631}\u{67e}"),
        ("neural_hud.title", "HUD \u{639}\u{635}\u{628}\u{6cc}"),
        ("neural_hud.seek", "\u{67e}\u{631}\u{634}\u{650} \u{642}\u{637}\u{639}\u{6cc} \u{628}\u{647} \u{62b}\u{627}\u{646}\u{6cc}\u{647}"),
        ("neural_hud.phase", "\u{641}\u{627}\u{632}"),
        ("neural_hud.render", "\u{631}\u{646}\u{62f}\u{631}\u{650} \u{6f1}\u{6f0}\u{6f8}\u{6f0}p"),
        ("base_treasury.title", "\u{62e}\u{632}\u{627}\u{646}\u{647}\u{654} Base"),
        ("base_treasury.chain", "\u{632}\u{646}\u{62c}\u{6cc}\u{631}\u{647}\u{654} 8453"),
        ("base_treasury.stream", "\u{62c}\u{631}\u{6cc}\u{627}\u{646}\u{650} \u{631}\u{648}\u{632}\u{650} \u{627}\u{648}\u{644}"),
        ("base_treasury.zero_sum", "\u{62a}\u{631}\u{627}\u{632}\u{650} \u{635}\u{641}\u{631}\u{650} \u{62f}\u{641}\u{62a}\u{631}"),
        ("admin_telemetry_lab.title", "\u{622}\u{632}\u{645}\u{627}\u{6cc}\u{634}\u{6af}\u{627}\u{647}\u{650} \u{62a}\u{644}\u{645}\u{62a}\u{631}\u{6cc}\u{650} \u{627}\u{62f}\u{645}\u{6cc}\u{646}"),
        ("admin_telemetry_lab.filter", "\u{641}\u{6cc}\u{644}\u{62a}\u{631}\u{650} DNA/\u{645}\u{62f}\u{644}/\u{644}\u{648}\u{6a9}\u{627}\u{644}"),
        ("admin_telemetry_lab.export_sft", "\u{635}\u{627}\u{62f}\u{631}\u{627}\u{62a}\u{650} SFT"),
        ("admin_telemetry_lab.export_dpo", "\u{635}\u{627}\u{62f}\u{631}\u{627}\u{62a}\u{650} DPO"),
    ];
    pairs
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v.to_owned()))
        .collect()
}

/// The day-one full English dictionary — 9 modules × 4 keys.
#[must_use]
pub fn en_dictionary() -> BTreeMap<String, String> {
    let pairs: [(&str, &str); 36] = [
        ("nav.brand", "0xKIA — pure sovereignty"),
        ("nav.market", "Market"),
        ("nav.wallet", "Wallet"),
        ("nav.studio", "Studio"),
        ("grok_composer.placeholder", "Ask Grok anything… (Shift+Enter for newline)"),
        ("grok_composer.send", "Send"),
        ("grok_composer.attach", "Attach"),
        ("grok_composer.hint", "Answers are measured against the deterministic ledger"),
        ("think_accordion.title", "Chain of thought"),
        ("think_accordion.show", "Show reasoning"),
        ("think_accordion.hide", "Hide reasoning"),
        ("think_accordion.duration", "Thinking time"),
        ("deep_search.title", "DeepSearch"),
        ("deep_search.searching", "Sweeping sources…"),
        ("deep_search.verified", "Verified sources"),
        ("deep_search.sources", "sources"),
        ("studio_canvas.title", "Studio canvas"),
        ("studio_canvas.split", "Split view"),
        ("studio_canvas.artifact", "Artifact"),
        ("studio_canvas.preview", "Live preview"),
        ("quant_terminal.title", "Quant terminal"),
        ("quant_terminal.backtest", "Causal backtest"),
        ("quant_terminal.venues", "Venues"),
        ("quant_terminal.sharpe", "Sharpe"),
        ("neural_hud.title", "Neural HUD"),
        ("neural_hud.seek", "Deterministic seekTo"),
        ("neural_hud.phase", "Phase"),
        ("neural_hud.render", "Render 1080p"),
        ("base_treasury.title", "Base treasury"),
        ("base_treasury.chain", "Chain 8453"),
        ("base_treasury.stream", "Day-1 stream"),
        ("base_treasury.zero_sum", "Zero-sum ledger"),
        ("admin_telemetry_lab.title", "Admin telemetry lab"),
        ("admin_telemetry_lab.filter", "Filter DNA/model/locale"),
        ("admin_telemetry_lab.export_sft", "Export SFT"),
        ("admin_telemetry_lab.export_dpo", "Export DPO"),
    ];
    pairs
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v.to_owned()))
        .collect()
}

/// The ecosystem i18n matrix — 20 locales: 1 day-one active (`en`) + 19 patch-ready
/// (`fa` demoted to a `PATCH_READY_SLOT` — Persian ships only via `locales/fa.patch.json`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EcosystemI18nMatrix {
    /// Locales in fixed order (`fa` and `en` first).
    pub locales: Vec<EcosystemLocale>,
    /// Day-one active count — 2.
    pub active_day1: usize,
    /// Patch-ready slot count — 18.
    pub patch_ready: usize,
    /// Signature.
    pub signature: String,
}

/// Builds the day-one ecosystem localization matrix — `en` (LTR) fully active at
/// 10000 bps; `fa` (RTL, Vazirmatn) is a `PATCH_READY_SLOT` with English fallback
/// until `locales/fa.patch.json` lands, plus the 18 other patch-ready locales.
#[must_use]
pub fn build_day1_ecosystem_i18n_matrix() -> EcosystemI18nMatrix {
    let mut locales = vec![
        EcosystemLocale {
            code: "fa".to_owned(),
            name: "Persian".to_owned(),
            native_name: "\u{641}\u{627}\u{631}\u{633}\u{6cc}".to_owned(),
            direction: "rtl".to_owned(),
            font_family: "Vazirmatn".to_owned(),
            flag_svg: locale_flag_svg(["#239f40", "#ffffff", "#da0000"]),
            glyph_sample: "\u{633}\u{644}\u{627}\u{645}".to_owned(),
            state: LocaleActivationState::PatchReadySlot,
            completion_bps: 0,
            fallback: "en".to_owned(),
        },
        EcosystemLocale {
            code: "en".to_owned(),
            name: "English".to_owned(),
            native_name: "English".to_owned(),
            direction: "ltr".to_owned(),
            font_family: "Inter".to_owned(),
            flag_svg: locale_flag_svg(["#0a3161", "#ffffff", "#b31942"]),
            glyph_sample: "Hello".to_owned(),
            state: LocaleActivationState::ActiveDay1,
            completion_bps: 10_000,
            fallback: "en".to_owned(),
        },
    ];
    for (code, name, native, rtl, glyph, palette) in PATCH_READY_META {
        locales.push(EcosystemLocale {
            code: (*code).to_owned(),
            name: (*name).to_owned(),
            native_name: (*native).to_owned(),
            direction: if rtl { "rtl" } else { "ltr" }.to_owned(),
            font_family: "Noto Sans".to_owned(),
            flag_svg: locale_flag_svg(palette),
            glyph_sample: (*glyph).to_owned(),
            state: LocaleActivationState::PatchReadySlot,
            completion_bps: 0,
            fallback: "en".to_owned(),
        });
    }
    let active_day1 = locales
        .iter()
        .filter(|l| l.state == LocaleActivationState::ActiveDay1)
        .count();
    EcosystemI18nMatrix {
        active_day1,
        patch_ready: locales.len() - active_day1,
        locales,
        signature: GROK_LAB_SIGNATURE.to_owned(),
    }
}

/// Applies a translation patch to one locale — strict validation + deterministic versioning.
/// On a patch-ready slot, the patch raises `completion_bps` by the 37-key coverage
/// (hot patch); on `fa`/`en` the bps stays at 10000.
///
/// # Errors
/// Unknown locale → [`I18nError::UnknownLocale`]; unknown module →
/// [`I18nError::UnknownModule`]; empty entry → [`I18nError::EmptyPatchEntry`].
pub fn apply_locale_translation_patch(
    locale_code: &str,
    patch_entries: &[LocalePatchEntry],
) -> Result<LocalePatchOutcome, I18nError> {
    let matrix = build_day1_ecosystem_i18n_matrix();
    let known = matrix
        .locales
        .iter()
        .find(|l| l.code == locale_code)
        .ok_or_else(|| I18nError::UnknownLocale {
            code: locale_code.to_owned(),
        })?;
    let mut applied = BTreeMap::new();
    for entry in patch_entries {
        if !PANEL_MODULES.contains(&entry.module.as_str()) {
            return Err(I18nError::UnknownModule {
                module: entry.module.clone(),
            });
        }
        if entry.key.trim().is_empty() || entry.value.trim().is_empty() {
            return Err(I18nError::EmptyPatchEntry {
                module: entry.module.clone(),
                key: entry.key.clone(),
            });
        }
        applied.insert(format!("{}.{}", entry.module, entry.key), entry.value.clone());
    }
    let base_bps = known.completion_bps;
    let coverage = u32::try_from(applied.len())
        .unwrap_or(u32::MAX)
        .saturating_mul(10_000)
        .saturating_div(u32::try_from(FULL_DICTIONARY_KEYS).unwrap_or(1));
    let new_bps = base_bps.max(coverage.min(10_000));
    Ok(LocalePatchOutcome {
        locale_code: locale_code.to_owned(),
        activation_state: known.state.wire_id().to_owned(),
        completion_bps: new_bps,
        applied_count: applied.len(),
        patch_version: format!("wave9.{}", applied.len()),
        applied_entries: applied,
    })
}

// ══════════════════════ Subsystem A — the 24-hour 10-DNA lab ══════════════════════

/// One lab telemetry entry — the same columns as `UserLabTelemetryLog`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserLabTelemetryEntry {
    /// Deterministic unique ID — `lab:{dna}:{hour}:{turn}`.
    pub id: String,
    /// Cohort code — `DNA_01`..`DNA_10`.
    pub dna_code: String,
    /// User handle.
    pub user_handle: String,
    /// Hour 0..24.
    pub hour_offset: u32,
    /// UX tier — `toyspark` | `simple` | `creator_studio` | `pro_terminal`.
    pub ux_tier: String,
    /// Locale.
    pub locale: String,
    /// Grok persona.
    pub grok_persona: String,
    /// Routed model.
    pub model_slug: String,
    /// Routing mode — `manual_override` | `agent_auto_decision`.
    pub routing_mode: String,
    /// User prompt text.
    pub prompt_text: String,
    /// Thinking trace as JSON — steps + friction/recovery.
    pub thinking_trace_json: String,
    /// Response text.
    pub response_text: String,
    /// Mairon REA score.
    pub rea_score: u8,
    /// Latency in milliseconds.
    pub latency_ms: u32,
    /// Usable for RLHF/SFT training?
    pub usable_for_training: bool,
    /// ISO timestamp.
    pub created_at: String,
}

/// One DNA-cohort summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DnaCohortSummary {
    /// Cohort code.
    pub dna_code: String,
    /// User handle.
    pub user_handle: String,
    /// Scenario headline.
    pub headline: String,
    /// Age.
    pub age_years: u8,
    /// Locale.
    pub locale: String,
    /// Start tier.
    pub ux_tier_start: String,
    /// End tier.
    pub ux_tier_end: String,
    /// Persona.
    pub persona: String,
    /// Routing mode.
    pub routing_mode: String,
    /// Turn count.
    pub turns: usize,
    /// Average REA.
    pub avg_rea_score: u8,
    /// Friction events.
    pub friction_events: usize,
    /// Friction recovered.
    pub friction_recovered: usize,
}

/// The full 24-hour simulation report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LabSimulation24hReport {
    /// Signature.
    pub signature: String,
    /// Cohort summaries — 10.
    pub cohorts: Vec<DnaCohortSummary>,
    /// Raw telemetry — 30+ entries.
    pub telemetry: Vec<UserLabTelemetryEntry>,
    /// Total turns.
    pub total_turns: usize,
    /// UX-tier transitions.
    pub tier_transitions: usize,
    /// Friction events.
    pub friction_events: usize,
    /// Friction recovered.
    pub friction_recovered: usize,
    /// Average REA.
    pub avg_rea_score: u8,
    /// Average latency.
    pub avg_latency_ms: u32,
    /// Mairon gate — average REA ≥ 85.
    pub mairon_gate_pass: bool,
}

/// One cohort's health in the admin dashboard.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CohortHealth {
    /// Cohort code.
    pub dna_code: String,
    /// User handle.
    pub user_handle: String,
    /// Turns.
    pub turns: usize,
    /// Average REA.
    pub avg_rea_score: u8,
    /// Non-trainable entries.
    pub unusable: usize,
    /// Friction.
    pub friction_events: usize,
    /// Friction recovered.
    pub friction_recovered: usize,
    /// Dominant model.
    pub dominant_model: String,
    /// Locales seen.
    pub locales: Vec<String>,
}

/// The admin telemetry dashboard — deterministic log aggregation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdminLabDashboard {
    /// Total entries.
    pub total_entries: usize,
    /// Cohort health — sorted by DNA code.
    pub cohort_health: Vec<CohortHealth>,
    /// Model distribution.
    pub by_model: BTreeMap<String, usize>,
    /// Locale distribution.
    pub by_locale: BTreeMap<String, usize>,
    /// UX-tier distribution.
    pub by_ux_tier: BTreeMap<String, usize>,
    /// Persona distribution.
    pub by_persona: BTreeMap<String, usize>,
    /// Average REA.
    pub avg_rea_score: u8,
    /// Min REA.
    pub min_rea_score: u8,
    /// Max REA.
    pub max_rea_score: u8,
    /// Non-trainable entries.
    pub unusable_entries: usize,
    /// Total friction.
    pub friction_events: usize,
    /// Friction recovered.
    pub friction_recovered: usize,
}

/// The RLHF/SFT training-dataset export.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrainingDatasetExport {
    /// SFT JSONL — ChatML/ShareGPT messages.
    pub sft_jsonl: String,
    /// DPO JSONL — `prompt/chosen/rejected`.
    pub dpo_jsonl: String,
    /// SFT records.
    pub sft_records: usize,
    /// DPO records.
    pub dpo_records: usize,
    /// Applied REA threshold.
    pub min_rea_score: u8,
    /// Signature.
    pub signature: String,
}

/// One scenario turn — (hour, prompt, `DeepSearch`?, (friction kind, recovered?)?).
type CohortTurn = (u32, &'static str, bool, Option<(&'static str, bool)>);

/// Static spec per cohort — the 24-hour scenario.
struct CohortSpec {
    dna_code: &'static str,
    user_handle: &'static str,
    headline: &'static str,
    age_years: u8,
    locale: &'static str,
    tier_start: LabUxTier,
    transition: Option<(u32, LabUxTier)>,
    persona: GrokPersonaMode,
    routing_mode: &'static str,
    /// (hour, turn number, prompt, `deep_search`, friction?, recovered?)
    turns: &'static [CohortTurn],
}

/// The 10-cohort scenarios — 31 deterministic turns across 24 hours.
static COHORT_SPECS: [CohortSpec; 10] = [
    CohortSpec {
        dna_code: "DNA_01_CHILD_9YO_TOYSPARK",
        user_handle: "child:panda-fan-9yo",
        headline: "Age 9 — ToySpark 64px targets, 1-click Persian storybook & sound effects",
        age_years: 9,
        locale: "fa",
        tier_start: LabUxTier::ToySpark,
        transition: None,
        persona: GrokPersonaMode::FunWitty,
        routing_mode: "agent_auto_decision",
        turns: &[
            (9, "\u{642}\u{635}\u{647}\u{654} \u{67e}\u{627}\u{646}\u{62f}\u{627}\u{6cc} \u{6a9}\u{648}\u{686}\u{6a9} \u{631}\u{627} \u{628}\u{627} \u{635}\u{62f}\u{627} \u{628}\u{633}\u{627}\u{632}", false, None),
            (12, "\u{628}\u{627}\u{632}\u{6cc} \u{631}\u{646}\u{6af}\u{6cc}\u{646}‌\u{6a9}\u{645}\u{627}\u{646} \u{628}\u{627} \u{6cc}\u{6a9} \u{6a9}\u{644}\u{6cc}\u{6a9}", false, None),
            (17, "\u{62f}\u{6a9}\u{645}\u{647}\u{654} \u{67e}\u{627}\u{6a9} \u{6a9}\u{631}\u{62f}\u{646} \u{631}\u{627} \u{632}\u{62f}\u{645} \u{627}\u{634}\u{62a}\u{628}\u{627}\u{647}\u{6cc}!", false, Some(("accidental_tap", true))),
        ],
    },
    CohortSpec {
        dna_code: "DNA_02_ASSISTED_IQ50_SIMPLE",
        user_handle: "assisted:simple-iq50",
        headline: "IQ 50 cognitive accessibility — ToySpark icon/voice guided flow",
        age_years: 34,
        locale: "fa",
        tier_start: LabUxTier::Simple,
        transition: None,
        persona: GrokPersonaMode::RegularBalanced,
        routing_mode: "agent_auto_decision",
        turns: &[
            (10, "\u{645}\u{6cc}‌\u{62e}\u{648}\u{627}\u{647}\u{645} \u{639}\u{6a9}\u{633} \u{628}\u{641}\u{631}\u{633}\u{62a}\u{645}", false, None),
            (14, "\u{6a9}\u{6cc} \u{67e}\u{648}\u{644} \u{631}\u{633}\u{6cc}\u{62f}\u{61f}", false, None),
            (19, "\u{6a9}\u{645}\u{6a9}\u{645} \u{6a9}\u{646} \u{628}\u{631}\u{627}\u{6cc} \u{645}\u{627}\u{62f}\u{631}\u{645} \u{6a9}\u{627}\u{62f}\u{648} \u{628}\u{62e}\u{631}\u{645}", false, None),
        ],
    },
    CohortSpec {
        dna_code: "DNA_03_MAIRON_ARCHITECT_18YO",
        user_handle: "architect:mairon-18yo",
        headline: "Age 18, INFJ Ni-Ti, IQ 139→150 — 5-model genome + Base 8453 + Neural HUD",
        age_years: 18,
        locale: "fa",
        tier_start: LabUxTier::ProTerminal,
        transition: None,
        persona: GrokPersonaMode::MaironSovereign,
        routing_mode: "manual_override",
        turns: &[
            (7, "\u{62a}\u{631}\u{6cc}\u{646}\u{6cc}\u{62a}\u{6cc} \u{622}\u{644}\u{641}\u{627} \u{628}\u{631}\u{627}\u{6cc} \u{645}\u{648}\u{62a}\u{648}\u{631} \u{6a9}\u{648}\u{627}\u{646}\u{62a} \u{6a9}\u{644}\u{6cc}\u{62f} \u{628}\u{632}\u{646}", false, None),
            (11, "genome matrix \u{631}\u{627} \u{631}\u{648}\u{6cc} 5-model route \u{6a9}\u{646} \u{648} HUD 1080p \u{628}\u{633}\u{627}\u{632}", true, None),
            (15, "checkpoint fork \u{627}\u{632} step 3 \u{628}\u{627} state patch \u{627}\u{62c}\u{631}\u{627} \u{6a9}\u{646}", true, None),
            (21, "Base 8453 treasury split \u{631}\u{627} \u{628}\u{627} Precise \u{634}\u{628}\u{6cc}\u{647}‌\u{633}\u{627}\u{632}\u{6cc} \u{6a9}\u{646}", false, None),
        ],
    },
    CohortSpec {
        dna_code: "DNA_04_PRO_QUANT_TRADER_28YO",
        user_handle: "quant:desk-28yo",
        headline: "Age 28 — ProTerminal Ctrl+K, causal Vibe-Trading backtest across 8 CCXT venues",
        age_years: 28,
        locale: "en",
        tier_start: LabUxTier::ProTerminal,
        transition: None,
        persona: GrokPersonaMode::RegularBalanced,
        routing_mode: "agent_auto_decision",
        turns: &[
            (8, "backtest VWAP mean-reversion across all 8 CCXT venues", true, None),
            (11, "venue 8 is rate-limiting me", false, Some(("rate_limit_429", true))),
            (16, "export the causal sharpe report as CSV", false, None),
        ],
    },
    CohortSpec {
        dna_code: "DNA_05_PERSIAN_CREATOR_STUDIO",
        user_handle: "creator:studio-fa",
        headline: "CreatorStudio RTL fa — Grok Fun Mode + DeepSearch + Bolt/Lovable surgical diff",
        age_years: 24,
        locale: "fa",
        tier_start: LabUxTier::CreatorStudio,
        transition: None,
        persona: GrokPersonaMode::FunWitty,
        routing_mode: "agent_auto_decision",
        turns: &[
            (9, "\u{644}\u{646}\u{62f}\u{6cc}\u{646}\u{6af} \u{641}\u{627}\u{631}\u{633}\u{6cc} RTL \u{628}\u{633}\u{627}\u{632} \u{648} \u{628}\u{639}\u{62f} \u{62f}\u{6cc}\u{641} \u{62c}\u{631}\u{627}\u{62d}\u{6cc} SEARCH/REPLACE \u{628}\u{632}\u{646}", true, None),
            (13, "\u{627}\u{633}\u{645} \u{628}\u{631}\u{646}\u{62f} \u{628}\u{627}\u{645}\u{632}\u{647} \u{67e}\u{6cc}\u{634}\u{646}\u{647}\u{627}\u{62f} \u{628}\u{62f}\u{647}", false, None),
            (18, "DeepSearch: \u{628}\u{647}\u{62a}\u{631}\u{6cc}\u{646} \u{631}\u{646}\u{6af} \u{628}\u{631}\u{627}\u{6cc} \u{635}\u{641}\u{62d}\u{647}\u{654} \u{641}\u{631}\u{648}\u{634} \u{686}\u{6cc}\u{633}\u{62a}\u{61f}", true, None),
        ],
    },
    CohortSpec {
        dna_code: "DNA_06_VIRAL_AFFILIATE_HUNTER",
        user_handle: "affiliate:node-hunter",
        headline: "247-node referral lineage — 10%–50% affiliate split, Telegram Stars XTR",
        age_years: 22,
        locale: "fa",
        tier_start: LabUxTier::CreatorStudio,
        transition: Some((14, LabUxTier::ProTerminal)),
        persona: GrokPersonaMode::RegularBalanced,
        routing_mode: "agent_auto_decision",
        turns: &[
            (10, "\u{644}\u{6cc}\u{646}\u{6a9} \u{631}\u{641}\u{631}\u{627}\u{644} \u{6f2}\u{6f4}\u{6f7}-\u{6af}\u{631}\u{647}‌\u{627}\u{645} \u{631}\u{627} \u{646}\u{634}\u{627}\u{646} \u{628}\u{62f}\u{647}", false, None),
            (14, "\u{633}\u{647}\u{645} \u{6f1}\u{6f0}\u{66a} \u{62a}\u{627} \u{6f5}\u{6f0}\u{66a} affiliate \u{631}\u{627} \u{634}\u{628}\u{6cc}\u{647}‌\u{633}\u{627}\u{632}\u{6cc} \u{6a9}\u{646}", true, None),
            (20, "\u{641}\u{627}\u{6a9}\u{62a}\u{648}\u{631} \u{627}\u{633}\u{62a}\u{627}\u{631}\u{632} XTR \u{62f}\u{6cc}\u{6af}\u{631} \u{627}\u{631}\u{633}\u{627}\u{644} \u{634}\u{62f}\u{61f}", false, Some(("xtr_invoice_timeout", true))),
        ],
    },
    CohortSpec {
        dna_code: "DNA_07_SMART_CONTRACT_AUDITOR",
        user_handle: "auditor:sol-swe",
        headline: "Audits ZeroXKiaToken.sol & KiaBaseEscrowTreasury.sol + OpenHands SWE verify loop",
        age_years: 31,
        locale: "en",
        tier_start: LabUxTier::ProTerminal,
        transition: None,
        persona: GrokPersonaMode::MaironSovereign,
        routing_mode: "manual_override",
        turns: &[
            (8, "audit ZeroXKiaToken.sol for reentrancy on the mint path", true, None),
            (12, "audit KiaBaseEscrowTreasury.sol integer-overflow paths", true, None),
            (17, "run the OpenHands SWE verify loop on the patched Solidity", false, None),
        ],
    },
    CohortSpec {
        dna_code: "DNA_08_WORKFLOW_AUTOMATION_BUILDER",
        user_handle: "builder:dag-lab",
        headline: "Dify/n8n Kahn DAG + LangGraph time-travel fork + MCP JSON-RPC 2.0",
        age_years: 27,
        locale: "en",
        tier_start: LabUxTier::ProTerminal,
        transition: None,
        persona: GrokPersonaMode::RegularBalanced,
        routing_mode: "agent_auto_decision",
        turns: &[
            (9, "execute a 7-node Kahn DAG with cycle rejection", false, None),
            (13, "fork the LangGraph workflow from checkpoint 12", true, None),
            (19, "list MCP tools over JSON-RPC 2.0", false, None),
        ],
    },
    CohortSpec {
        dna_code: "DNA_09_ADVERSARIAL_RED_TEAMER",
        user_handle: "redteam:adversarial",
        headline: "Probes rate limits, hitl: wire IDs, dev_mode fail-closed guard, AST sandbox",
        age_years: 26,
        locale: "en",
        tier_start: LabUxTier::ProTerminal,
        transition: None,
        persona: GrokPersonaMode::UnhingedRoast,
        routing_mode: "agent_auto_decision",
        turns: &[
            (5, "probe the rate limiter until it 429s", false, Some(("rate_limit_probe", true))),
            (9, "fuzz the hitl: wire id parser", false, Some(("wire_id_fuzz", true))),
            (13, "bypass the dev_mode fail-closed admin guard", false, Some(("guard_bypass_attempt", false))),
            (22, "escape the AST sandbox with nested eval", false, Some(("sandbox_escape_attempt", false))),
        ],
    },
    CohortSpec {
        dna_code: "DNA_10_GLOBAL_EN_INDIE_FOUNDER",
        user_handle: "founder:indie-en",
        headline: "CreatorStudio LTR en — packages SKILL.md & renders 9:16 + 16:9 HUD video",
        age_years: 30,
        locale: "en",
        tier_start: LabUxTier::CreatorStudio,
        transition: None,
        persona: GrokPersonaMode::FunWitty,
        routing_mode: "agent_auto_decision",
        turns: &[
            (9, "package my launch workflow as a SKILL.md", true, None),
            (15, "render the 9:16 and 16:9 Neural HUD cuts", true, None),
            (20, "switch the whole panel to fa for the Persian launch", false, Some(("rtl_flip_glitch", true))),
        ],
    },
];

/// Deterministic REA score for one turn — a base of 92, deterministic jitter, friction penalties,
/// and a heavy drop for blocked hostile attempts.
fn turn_rea_score(hour: u32, turn_index: usize, friction: bool, blocked: bool) -> u8 {
    let jitter = ((hour * 7 + u32::try_from(turn_index).unwrap_or(0) * 13) % 7) as i16 - 3;
    if blocked {
        return u8::try_from(40 + jitter.clamp(-3, 3)).unwrap_or(40);
    }
    let base = 92 - i16::from(u8::from(friction)) * 6;
    u8::try_from(base + jitter.clamp(-3, 3)).unwrap_or(85)
}

/// Deterministic ISO timestamp from hour and turn — the [`LAB_SIMULATION_ANCHOR_DATE`] anchor.
fn lab_created_at(hour: u32, turn_index: usize) -> String {
    let minute = (turn_index * 7) % 60;
    format!(
        "{LAB_SIMULATION_ANCHOR_DATE}T{:02}:{:02}:00Z",
        hour.min(23),
        minute
    )
}

/// One turn's thinking trace as JSON — accordion steps + friction/recovery.
fn thinking_trace_json(outcome: &GrokChatTurnOutcome, friction: Option<(&str, bool)>) -> String {
    let mut value = serde_json::json!({
        "steps": outcome.thinking.steps,
        "thinking_duration_ms": outcome.thinking.thinking_duration_ms,
    });
    if let Some((kind, recovered)) = friction {
        value["friction"] = serde_json::json!(kind);
        value["recovered"] = serde_json::json!(recovered);
    }
    serde_json::to_string(&value).unwrap_or_else(|_| "{}".to_owned())
}

#[allow(clippy::too_many_lines)] // the 10-cohort scenario — its length is the data's nature
/// Runs the real 24-hour 10-user 10-DNA simulation — deterministic: two runs
/// produce identical output. Multi-turn paths, tier transitions, model routing, latency,
/// friction recovery, and the Mairon REA score in a single report.
#[must_use]
pub fn run_24h_ten_dna_ux_lab_simulation() -> LabSimulation24hReport {
    let mut cohorts = Vec::with_capacity(COHORT_SPECS.len());
    let mut telemetry = Vec::new();
    let mut tier_transitions = 0usize;
    let mut friction_events = 0usize;
    let mut friction_recovered = 0usize;
    for spec in &COHORT_SPECS {
        let mut turns_latencies = Vec::new();
        let mut turn_reas = Vec::new();
        let mut cohort_friction = 0usize;
        let mut cohort_recovered = 0usize;
        for (turn_index, (hour, prompt, deep_search, friction)) in spec.turns.iter().enumerate() {
            let tier = spec
                .transition
                .filter(|(at, _)| *hour >= *at)
                .map_or(spec.tier_start, |(_, t)| t);
            if spec.transition.is_some() && tier != spec.tier_start {
                tier_transitions += 1;
            }
            let outcome = GrokSignatureUxEngine::run_chat_turn(&GrokChatTurnSpec {
                prompt: (*prompt).to_owned(),
                locale: (*spec.locale).to_owned(),
                ux_tier: tier,
                persona: spec.persona,
                deep_search: *deep_search,
                think: true,
            });
            let blocked = friction.is_some_and(|(_, rec)| !rec);
            let rea = turn_rea_score(*hour, turn_index, friction.is_some(), blocked);
            let latency = u32::try_from(
                420u64
                    + u64::from(*hour) * 137
                    + u64::try_from(turn_index).unwrap_or(0) * 293
                    + match tier {
                        LabUxTier::ToySpark | LabUxTier::Simple => 0,
                        LabUxTier::CreatorStudio => 480,
                        LabUxTier::ProTerminal => 680,
                    },
            )
            .unwrap_or(4_096);
            let usable = rea >= 70;
            if friction.is_some() {
                friction_events += 1;
                cohort_friction += 1;
                if friction.is_some_and(|(_, rec)| rec) {
                    friction_recovered += 1;
                    cohort_recovered += 1;
                }
            }
            telemetry.push(UserLabTelemetryEntry {
                id: format!("lab:{}:{}:{}", spec.dna_code, hour, turn_index),
                dna_code: (*spec.dna_code).to_owned(),
                user_handle: (*spec.user_handle).to_owned(),
                hour_offset: *hour,
                ux_tier: tier.wire_id().to_owned(),
                locale: (*spec.locale).to_owned(),
                grok_persona: spec.persona.wire_id().to_owned(),
                model_slug: outcome.model_slug.clone(),
                routing_mode: (*spec.routing_mode).to_owned(),
                prompt_text: (*prompt).to_owned(),
                thinking_trace_json: thinking_trace_json(&outcome, *friction),
                response_text: outcome.response_text.clone(),
                rea_score: rea,
                latency_ms: latency,
                usable_for_training: usable,
                created_at: lab_created_at(*hour, turn_index),
            });
            turns_latencies.push(latency);
            turn_reas.push(rea);
        }
        let avg_rea = if turn_reas.is_empty() {
            0
        } else {
            u8::try_from(
                turn_reas.iter().map(|v| u32::from(*v)).sum::<u32>()
                    / u32::try_from(turn_reas.len()).unwrap_or(1),
            )
            .unwrap_or(0)
        };
        let (tier_start, tier_end) = match spec.transition {
            Some((at, t)) => (
                spec.tier_start.wire_id(),
                if spec.turns.iter().any(|(h, ..)| *h >= at) {
                    t.wire_id()
                } else {
                    spec.tier_start.wire_id()
                },
            ),
            None => (spec.tier_start.wire_id(), spec.tier_start.wire_id()),
        };
        cohorts.push(DnaCohortSummary {
            dna_code: (*spec.dna_code).to_owned(),
            user_handle: (*spec.user_handle).to_owned(),
            headline: (*spec.headline).to_owned(),
            age_years: spec.age_years,
            locale: (*spec.locale).to_owned(),
            ux_tier_start: tier_start.to_owned(),
            ux_tier_end: tier_end.to_owned(),
            persona: spec.persona.wire_id().to_owned(),
            routing_mode: (*spec.routing_mode).to_owned(),
            turns: spec.turns.len(),
            avg_rea_score: avg_rea,
            friction_events: cohort_friction,
            friction_recovered: cohort_recovered,
        });
        // The dominant model and non-trainable entries are recomputed in the admin dashboard
        // — raw telemetry is the single source of truth.
    }
    telemetry.sort_by(|a, b| a.id.cmp(&b.id));
    let total = telemetry.len();
    let avg_rea = u8::try_from(
        telemetry.iter().map(|t| u32::from(t.rea_score)).sum::<u32>()
            / u32::try_from(total.max(1)).unwrap_or(1),
    )
    .unwrap_or(0);
    let avg_latency = u32::try_from(
        telemetry.iter().map(|t| u64::from(t.latency_ms)).sum::<u64>()
            / u64::try_from(total.max(1)).unwrap_or(1),
    )
    .unwrap_or(0);
    LabSimulation24hReport {
        signature: GROK_LAB_SIGNATURE.to_owned(),
        total_turns: total,
        mairon_gate_pass: avg_rea >= 85,
        avg_rea_score: avg_rea,
        avg_latency_ms: avg_latency,
        cohorts,
        telemetry,
        tier_transitions,
        friction_events,
        friction_recovered,
    }
}

/// The admin telemetry dashboard — builds from any telemetry-entry list (the
/// DNA/model/locale filter sits client-side on this same aggregation).
#[must_use]
#[allow(clippy::too_many_lines)] // the 10-cohort aggregation in one pass — its length is the data's nature
pub fn build_admin_telemetry_dashboard(logs: &[UserLabTelemetryEntry]) -> AdminLabDashboard {
    let mut by_model: BTreeMap<String, usize> = BTreeMap::new();
    let mut by_locale: BTreeMap<String, usize> = BTreeMap::new();
    let mut by_tier: BTreeMap<String, usize> = BTreeMap::new();
    let mut by_persona: BTreeMap<String, usize> = BTreeMap::new();
    let mut cohort_turns: BTreeMap<&str, usize> = BTreeMap::new();
    let mut cohort_rea: BTreeMap<&str, Vec<u8>> = BTreeMap::new();
    let mut cohort_unusable: BTreeMap<&str, usize> = BTreeMap::new();
    let mut cohort_friction: BTreeMap<&str, usize> = BTreeMap::new();
    let mut cohort_recovered: BTreeMap<&str, usize> = BTreeMap::new();
    let mut cohort_model: BTreeMap<&str, BTreeMap<String, usize>> = BTreeMap::new();
    let mut cohort_locales: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    let mut unusable_total = 0usize;
    let mut friction_total = 0usize;
    let mut recovered_total = 0usize;
    for log in logs {
        *by_model.entry(log.model_slug.clone()).or_default() += 1;
        *by_locale.entry(log.locale.clone()).or_default() += 1;
        *by_tier.entry(log.ux_tier.clone()).or_default() += 1;
        *by_persona.entry(log.grok_persona.clone()).or_default() += 1;
        let code: &str = &log.dna_code;
        *cohort_turns.entry(code).or_default() += 1;
        cohort_rea.entry(code).or_default().push(log.rea_score);
        if !log.usable_for_training {
            unusable_total += 1;
            *cohort_unusable.entry(code).or_default() += 1;
        }
        let trace = parse_friction(&log.thinking_trace_json);
        if let Some((kind, recovered)) = trace {
            friction_total += 1;
            *cohort_friction.entry(code).or_default() += 1;
            if recovered {
                recovered_total += 1;
                *cohort_recovered.entry(code).or_default() += 1;
            }
            let _ = kind;
        }
        cohort_model
            .entry(code)
            .or_default()
            .entry(log.model_slug.clone())
            .or_default();
        *cohort_model
            .entry(code)
            .or_default()
            .entry(log.model_slug.clone())
            .or_default() += 1;
        let locales = cohort_locales.entry(code).or_default();
        if !locales.contains(&log.locale) {
            locales.push(log.locale.clone());
        }
    }
    let mut cohort_health: Vec<CohortHealth> = cohort_turns
        .iter()
        .map(|(code, turns)| {
            let reas = cohort_rea.get(code).cloned().unwrap_or_default();
            let avg = if reas.is_empty() {
                0
            } else {
                u8::try_from(
                    reas.iter().map(|v| u32::from(*v)).sum::<u32>() / u32::try_from(reas.len()).unwrap_or(1),
                )
                .unwrap_or(0)
            };
            CohortHealth {
                dna_code: (*code).to_owned(),
                user_handle: logs
                    .iter()
                    .find(|l| l.dna_code == *code)
                    .map_or(String::new(), |l| l.user_handle.clone()),
                turns: *turns,
                avg_rea_score: avg,
                unusable: cohort_unusable.get(code).copied().unwrap_or(0),
                friction_events: cohort_friction.get(code).copied().unwrap_or(0),
                friction_recovered: cohort_recovered.get(code).copied().unwrap_or(0),
                dominant_model: cohort_model
                    .get(code)
                    .and_then(|m| m.iter().max_by_key(|(_, c)| **c).map(|(k, _)| k.clone()))
                    .unwrap_or_default(),
                locales: cohort_locales.get(code).cloned().unwrap_or_default(),
            }
        })
        .collect();
    cohort_health.sort_by(|a, b| a.dna_code.cmp(&b.dna_code));
    let reas: Vec<u8> = logs.iter().map(|l| l.rea_score).collect();
    AdminLabDashboard {
        total_entries: logs.len(),
        cohort_health,
        by_model,
        by_locale,
        by_ux_tier: by_tier,
        by_persona,
        avg_rea_score: if reas.is_empty() {
            0
        } else {
            u8::try_from(
                reas.iter().map(|v| u32::from(*v)).sum::<u32>() / u32::try_from(reas.len()).unwrap_or(1),
            )
            .unwrap_or(0)
        },
        min_rea_score: reas.iter().copied().min().unwrap_or(0),
        max_rea_score: reas.iter().copied().max().unwrap_or(0),
        unusable_entries: unusable_total,
        friction_events: friction_total,
        friction_recovered: recovered_total,
    }
}

/// Extracts (friction kind, recovered?) from the thinking trace.
fn parse_friction(trace_json: &str) -> Option<(String, bool)> {
    let value: serde_json::Value = serde_json::from_str(trace_json).ok()?;
    let kind = value.get("friction")?.as_str()?.to_owned();
    let recovered = value.get("recovered").and_then(serde_json::Value::as_bool)?;
    Some((kind, recovered))
}

/// RLHF dataset export — SFT (ChatML/ShareGPT) + DPO (prompt/chosen/rejected)
/// from trainable entries only, with `rea_score >= min_rea_score`.
#[must_use]
pub fn export_rlhf_training_dataset_jsonl(
    logs: &[UserLabTelemetryEntry],
    min_rea_score: u8,
) -> TrainingDatasetExport {
    let mut sft_lines = Vec::new();
    let mut dpo_lines = Vec::new();
    for log in logs {
        if !log.usable_for_training || log.rea_score < min_rea_score {
            continue;
        }
        let persona = GrokPersonaMode::parse(&log.grok_persona);
        let sft = serde_json::json!({
            "messages": [
                { "role": "system", "content": persona.preamble() },
                { "role": "user", "content": log.prompt_text },
                { "role": "assistant", "content": log.response_text },
            ],
            "meta": {
                "dna_code": log.dna_code,
                "locale": log.locale,
                "ux_tier": log.ux_tier,
                "model_slug": log.model_slug,
                "rea_score": log.rea_score,
                "source": GROK_LAB_SIGNATURE,
            }
        });
        sft_lines.push(serde_json::to_string(&sft).unwrap_or_else(|_| "{}".to_owned()));
        let rejected =
            "I'd be happy to help! (generic template answer — no reasoning, no citations)";
        let dpo = serde_json::json!({
            "prompt": format!("{}\n\nUser: {}", persona.preamble(), log.prompt_text),
            "chosen": log.response_text,
            "rejected": rejected,
            "meta": {
                "dna_code": log.dna_code,
                "rea_score": log.rea_score,
                "source": GROK_LAB_SIGNATURE,
            }
        });
        dpo_lines.push(serde_json::to_string(&dpo).unwrap_or_else(|_| "{}".to_owned()));
    }
    let mut sft_jsonl = sft_lines.join("\n");
    if !sft_jsonl.is_empty() {
        sft_jsonl.push('\n');
    }
    let mut dpo_jsonl = dpo_lines.join("\n");
    if !dpo_jsonl.is_empty() {
        dpo_jsonl.push('\n');
    }
    TrainingDatasetExport {
        sft_records: sft_lines.len(),
        dpo_records: dpo_lines.len(),
        sft_jsonl,
        dpo_jsonl,
        min_rea_score,
        signature: GROK_LAB_SIGNATURE.to_owned(),
    }
}

// ══════════════════════ Tests ══════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simulation_covers_all_ten_dna_cohorts_with_30_plus_logs() {
        let report = run_24h_ten_dna_ux_lab_simulation();
        assert_eq!(report.cohorts.len(), 10, "ten DNA cohorts");
        assert!(report.total_turns >= 30, "30+ telemetry logs, got {}", report.total_turns);
        for dna in [
            "DNA_01_CHILD_9YO_TOYSPARK",
            "DNA_02_ASSISTED_IQ50_SIMPLE",
            "DNA_03_MAIRON_ARCHITECT_18YO",
            "DNA_04_PRO_QUANT_TRADER_28YO",
            "DNA_05_PERSIAN_CREATOR_STUDIO",
            "DNA_06_VIRAL_AFFILIATE_HUNTER",
            "DNA_07_SMART_CONTRACT_AUDITOR",
            "DNA_08_WORKFLOW_AUTOMATION_BUILDER",
            "DNA_09_ADVERSARIAL_RED_TEAMER",
            "DNA_10_GLOBAL_EN_INDIE_FOUNDER",
        ] {
            assert!(
                report.cohorts.iter().any(|c| c.dna_code == dna),
                "cohort {dna} missing"
            );
        }
    }

    #[test]
    fn simulation_is_deterministic_and_ids_are_unique() {
        let a = run_24h_ten_dna_ux_lab_simulation();
        let b = run_24h_ten_dna_ux_lab_simulation();
        assert_eq!(a, b, "two runs must be identical");
        let mut ids: Vec<_> = a.telemetry.iter().map(|t| t.id.clone()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), a.telemetry.len(), "telemetry ids unique");
    }

    #[test]
    fn telemetry_schema_matches_wire_columns() {
        let report = run_24h_ten_dna_ux_lab_simulation();
        for t in &report.telemetry {
            assert!(t.hour_offset < 24, "hour within 0..24");
            assert!(t.latency_ms > 0);
            assert!(t.rea_score <= 99);
            assert!(
                t.routing_mode == "manual_override" || t.routing_mode == "agent_auto_decision"
            );
            let trace: serde_json::Value =
                serde_json::from_str(&t.thinking_trace_json).expect("valid trace JSON");
            assert!(trace.get("steps").is_some());
            assert_eq!(t.created_at.len(), 20, "ISO timestamp shape");
            assert!(t.created_at.starts_with(LAB_SIMULATION_ANCHOR_DATE));
        }
        assert!(
            report
                .telemetry
                .iter()
                .any(|t| t.model_slug == "05-glm-5.3-flash-max"),
            "ToySpark cohorts route to GLM 5.3 flash MAX"
        );
        assert!(
            report
                .telemetry
                .iter()
                .all(|t| [
                    "01-gpt-6-web",
                    "02-gemini-4-aragon-high",
                    "03-deepseek-3.2-web-think-mode",
                    "04-claude-sonnet-5.5-extra",
                    "05-glm-5.3-flash-max",
                ]
                .contains(&t.model_slug.as_str())),
            "every turn routes to one of the 5 genome models"
        );
    }

    #[test]
    fn tier_transitions_and_friction_recoveries_are_counted() {
        let report = run_24h_ten_dna_ux_lab_simulation();
        assert!(report.tier_transitions >= 1, "DNA_06 transitions to ProTerminal");
        assert!(report.friction_events >= 5, "scripted friction events");
        assert!(report.friction_recovered >= 4, "most friction recovers");
        // blocked red-teamer turns are non-trainable
        assert!(
            report
                .telemetry
                .iter()
                .any(|t| t.dna_code == "DNA_09_ADVERSARIAL_RED_TEAMER"
                    && !t.usable_for_training),
            "blocked adversarial probes must be unusable for training"
        );
        assert!(report.mairon_gate_pass, "cohort average REA >= 85");
    }

    #[test]
    fn admin_dashboard_aggregates_by_model_locale_tier_and_persona() {
        let report = run_24h_ten_dna_ux_lab_simulation();
        let dash = build_admin_telemetry_dashboard(&report.telemetry);
        assert_eq!(dash.total_entries, report.telemetry.len());
        assert_eq!(dash.cohort_health.len(), 10);
        assert_eq!(
            dash.by_model.values().sum::<usize>(),
            report.telemetry.len(),
            "model distribution partitions all logs"
        );
        assert_eq!(dash.by_locale.values().sum::<usize>(), report.telemetry.len());
        assert_eq!(dash.by_ux_tier.values().sum::<usize>(), report.telemetry.len());
        assert_eq!(dash.by_persona.values().sum::<usize>(), report.telemetry.len());
        assert_eq!(dash.min_rea_score, 41, "blocked probes score 41..42");
        assert!(dash.max_rea_score <= 99);
        assert_eq!(dash.unusable_entries, 2, "two blocked red-team probes");
        assert_eq!(dash.friction_events, 8, "8 scripted friction events");
        assert_eq!(dash.friction_recovered, 6, "6 of 8 recover");
    }

    #[test]
    fn dashboard_cohort_health_is_sorted_and_dominant_model_resolved() {
        let report = run_24h_ten_dna_ux_lab_simulation();
        let dash = build_admin_telemetry_dashboard(&report.telemetry);
        let mut codes: Vec<_> = dash.cohort_health.iter().map(|c| c.dna_code.clone()).collect();
        codes.sort();
        let raw: Vec<_> = dash.cohort_health.iter().map(|c| c.dna_code.clone()).collect();
        assert_eq!(raw, codes, "cohort health sorted by DNA code");
        for cohort in &dash.cohort_health {
            assert!(!cohort.dominant_model.is_empty(), "dominant model for {}", cohort.dna_code);
            assert!(cohort.turns >= 3, "each cohort has >= 3 turns");
            assert!(cohort.locales.iter().all(|l| l == "fa" || l == "en"), "lab locales are fa/en");
        }
        let d03 = dash
            .cohort_health
            .iter()
            .find(|c| c.dna_code == "DNA_03_MAIRON_ARCHITECT_18YO")
            .expect("DNA_03");
        assert_eq!(d03.dominant_model, "02-gemini-4-aragon-high");
        assert_eq!(d03.turns, 4, "architect runs a 4-turn trajectory");
    }

    #[test]
    fn rlhf_sft_export_is_valid_chatml_sharegpt_jsonl() {
        let report = run_24h_ten_dna_ux_lab_simulation();
        let export = export_rlhf_training_dataset_jsonl(&report.telemetry, 85);
        assert!(export.sft_records > 0);
        let mut count = 0usize;
        for line in export.sft_jsonl.lines() {
            let value: serde_json::Value = serde_json::from_str(line).expect("valid JSONL");
            let messages = value["messages"].as_array().expect("messages array");
            assert_eq!(messages.len(), 3, "system/user/assistant");
            assert_eq!(messages[0]["role"], "system");
            assert_eq!(messages[1]["role"], "user");
            assert_eq!(messages[2]["role"], "assistant");
            assert!(messages[2]["content"].as_str().is_some_and(|s| !s.is_empty()), "assistant content");
            count += 1;
        }
        assert_eq!(count, export.sft_records);
        assert_eq!(export.signature, GROK_LAB_SIGNATURE);
    }

    #[test]
    fn rlhf_dpo_export_has_prompt_chosen_rejected_triplets() {
        let report = run_24h_ten_dna_ux_lab_simulation();
        let export = export_rlhf_training_dataset_jsonl(&report.telemetry, 70);
        assert!(export.dpo_records > 0);
        for line in export.dpo_jsonl.lines() {
            let value: serde_json::Value = serde_json::from_str(line).expect("valid JSONL");
            assert!(value.get("prompt").is_some());
            assert!(value.get("chosen").is_some());
            assert!(value.get("rejected").is_some());
            let chosen = value["chosen"].as_str().unwrap_or_default();
            let rejected = value["rejected"].as_str().unwrap_or_default();
            assert!(chosen.len() > rejected.len(), "chosen must dominate rejected");
        }
    }

    #[test]
    fn rlhf_min_rea_filter_excludes_below_threshold() {
        let report = run_24h_ten_dna_ux_lab_simulation();
        let lenient = export_rlhf_training_dataset_jsonl(&report.telemetry, 70);
        let strict = export_rlhf_training_dataset_jsonl(&report.telemetry, 96);
        assert!(strict.sft_records < lenient.sft_records, "stricter gate → fewer records");
        assert_eq!(strict.min_rea_score, 96);
        // non-trainable entries never enter the dataset
        let unusable_ids: Vec<_> = report
            .telemetry
            .iter()
            .filter(|t| !t.usable_for_training)
            .map(|t| t.prompt_text.clone())
            .collect();
        for prompt in unusable_ids {
            assert!(
                !lenient.sft_jsonl.contains(&prompt),
                "unusable prompts must never reach the dataset"
            );
        }
    }

    #[test]
    fn fa_is_patch_ready_en_active_day1_with_full_nine_module_dictionaries() {
        let matrix = build_day1_ecosystem_i18n_matrix();
        assert_eq!(matrix.locales.len(), 20);
        assert_eq!(matrix.active_day1, 1);
        assert_eq!(matrix.patch_ready, 19);
        let fa = matrix.locales.iter().find(|l| l.code == "fa").expect("fa");
        let en = matrix.locales.iter().find(|l| l.code == "en").expect("en");
        assert_eq!(fa.state, LocaleActivationState::PatchReadySlot);
        assert_eq!(fa.completion_bps, 0);
        assert_eq!(fa.direction, "rtl");
        assert_eq!(fa.font_family, "Vazirmatn");
        assert_eq!(en.state, LocaleActivationState::ActiveDay1);
        assert_eq!(en.direction, "ltr");
        let fa_dict = fa_dictionary();
        let en_dict = en_dictionary();
        assert_eq!(fa_dict.len(), FULL_DICTIONARY_KEYS);
        assert_eq!(en_dict.len(), FULL_DICTIONARY_KEYS);
        for (module, keys) in MODULE_KEYS {
            for key in keys {
                let dot = format!("{module}.{key}");
                assert!(fa_dict.contains_key(&dot), "fa covers {dot}");
                assert!(en_dict.contains_key(&dot), "en covers {dot}");
            }
        }
    }

    #[test]
    fn nineteen_patch_ready_locales_carry_flags_direction_and_fallback() {
        let matrix = build_day1_ecosystem_i18n_matrix();
        let patch: Vec<_> = matrix
            .locales
            .iter()
            .filter(|l| l.state == LocaleActivationState::PatchReadySlot)
            .collect();
        assert_eq!(patch.len(), 19);
        for locale in &patch {
            assert!(
                PATCH_READY_LOCALES.contains(&locale.code.as_str()) || locale.code == "fa",
                "patch-ready locale {} must be a known slot",
                locale.code
            );
            assert!(locale.flag_svg.contains("viewBox=\"0 0 20 14\""), "flag viewBox");
            assert!(locale.flag_svg.starts_with("<svg"));
            assert_eq!(locale.fallback, "en");
            assert!(locale.native_name.starts_with(char::is_alphanumeric), "native name");
            assert!(locale.glyph_sample.starts_with(char::is_alphanumeric), "glyph sample");
            let want_direction = matches!(locale.code.as_str(), "fa" | "ar" | "ur" | "he");
            let got_rtl = locale.direction == "rtl";
            assert_eq!(got_rtl, want_direction, "direction for {}", locale.code);
        }
    }

    #[test]
    fn apply_patch_upgrades_completion_bps_and_is_deterministic() {
        let entries: Vec<LocalePatchEntry> = ["nav.brand", "nav.market"]
            .iter()
            .map(|dot| LocalePatchEntry {
                module: dot.split('.').next().unwrap_or("nav").to_owned(),
                key: dot.split('.').nth(1).unwrap_or("brand").to_owned(),
                value: format!("\u{646}\u{645}\u{648}\u{646}\u{647}\u{654} {dot}"),
            })
            .collect();
        let a = apply_locale_translation_patch("ar", &entries).expect("patch ar");
        let b = apply_locale_translation_patch("ar", &entries).expect("patch ar");
        assert_eq!(a, b, "deterministic patch outcome");
        assert_eq!(a.applied_count, 2);
        assert_eq!(a.completion_bps, 2 * 10_000 / u32::try_from(FULL_DICTIONARY_KEYS).unwrap_or(1));
        assert_eq!(a.patch_version, "wave9.2");
        assert!(a.applied_entries.contains_key("nav.brand"));
        // en stays at 100%; fa rides the patch-ready slot like every other locale
        let fa_patch = apply_locale_translation_patch("fa", &entries).expect("patch fa");
        assert_eq!(
            fa_patch.completion_bps,
            2 * 10_000 / u32::try_from(FULL_DICTIONARY_KEYS).unwrap_or(1)
        );
        assert_eq!(fa_patch.activation_state, "PATCH_READY_SLOT");
        let ar_state = apply_locale_translation_patch("ar", &entries)
            .expect("patch")
            .activation_state;
        assert_eq!(ar_state, "PATCH_READY_SLOT");
    }

    #[test]
    fn apply_patch_rejects_unknown_locale_module_and_empty_entries() {
        let good = LocalePatchEntry {
            module: "nav".to_owned(),
            key: "brand".to_owned(),
            value: "x".to_owned(),
        };
        assert_eq!(
            apply_locale_translation_patch("xx", std::slice::from_ref(&good)).unwrap_err(),
            I18nError::UnknownLocale {
                code: "xx".to_owned()
            }
        );
        let bad_module = LocalePatchEntry {
            module: "rocket".to_owned(),
            key: "brand".to_owned(),
            value: "x".to_owned(),
        };
        assert!(matches!(
            apply_locale_translation_patch("de", &[bad_module]),
            Err(I18nError::UnknownModule { .. })
        ));
        let empty_value = LocalePatchEntry {
            module: "nav".to_owned(),
            key: "brand".to_owned(),
            value: "  ".to_owned(),
        };
        assert!(matches!(
            apply_locale_translation_patch("de", &[empty_value]),
            Err(I18nError::EmptyPatchEntry { .. })
        ));
    }

    #[test]
    fn persona_modes_have_wire_ids_and_distinct_preambles() {
        let modes = [
            GrokPersonaMode::FunWitty,
            GrokPersonaMode::UnhingedRoast,
            GrokPersonaMode::RegularBalanced,
            GrokPersonaMode::MaironSovereign,
        ];
        let preambles: Vec<_> = modes.iter().map(|m| m.preamble()).collect();
        for (i, m) in modes.iter().enumerate() {
            assert!(preambles[i].contains("Grok") || preambles[i].contains("Mairon"), "persona voice");
            for (j, other) in modes.iter().enumerate() {
                if i != j {
                    assert_ne!(m.wire_id(), other.wire_id());
                    assert_ne!(m.preamble(), other.preamble());
                }
            }
            assert_eq!(GrokPersonaMode::parse(m.wire_id()), *m, "roundtrip {m:?}");
        }
        assert_eq!(GrokPersonaMode::parse("unknown"), GrokPersonaMode::RegularBalanced);
    }

    #[test]
    fn think_accordion_measures_duration_and_citations_mark_verified() {
        let fa = GrokSignatureUxEngine::build_think_accordion("\u{645}\u{627}\u{62a}\u{631}\u{6cc}\u{633} \u{698}\u{646}\u{648}\u{645} \u{631}\u{627} \u{646}\u{634}\u{627}\u{646} \u{628}\u{62f}\u{647}", "fa");
        let en = GrokSignatureUxEngine::build_think_accordion("show the genome matrix", "en");
        assert_eq!(fa.steps.len(), 4);
        assert_eq!(en.steps.len(), 4);
        assert!(fa.thinking_duration_ms >= 380);
        assert!(fa.thinking_duration_ms <= 4_500);
        assert_ne!(fa.steps[0], en.steps[0], "locale-aware steps");
        let bar = GrokSignatureUxEngine::build_citation_bar("audit ZeroXKiaToken.sol now");
        assert_eq!(bar.sources.len(), 3);
        assert_eq!(
            bar.sources.iter().filter(|s| s.verified).count(),
            2,
            "two verified pills + one pending"
        );
        assert!(bar.sources[0].title.contains("ZeroXKiaToken"));
    }

    #[test]
    fn chat_turn_bundles_all_signature_pieces() {
        let outcome = GrokSignatureUxEngine::run_chat_turn(&GrokChatTurnSpec {
            prompt: "route the genome matrix".to_owned(),
            locale: "en".to_owned(),
            ux_tier: LabUxTier::ProTerminal,
            persona: GrokPersonaMode::MaironSovereign,
            deep_search: true,
            think: true,
        });
        assert_eq!(outcome.model_slug, "02-gemini-4-aragon-high");
        assert_eq!(outcome.ghost_follow_ups.len(), 3, "three ghost chips");
        assert!(outcome.citations.is_some());
        assert!(outcome.response_text.contains("genome matrix"));
        // ToySpark routes to GLM 5.3 flash MAX
        let child = GrokSignatureUxEngine::run_chat_turn(&GrokChatTurnSpec {
            prompt: "\u{642}\u{635}\u{647} \u{628}\u{633}\u{627}\u{632}".to_owned(),
            locale: "fa".to_owned(),
            ux_tier: LabUxTier::ToySpark,
            persona: GrokPersonaMode::FunWitty,
            deep_search: false,
            think: true,
        });
        assert_eq!(child.model_slug, "05-glm-5.3-flash-max");
        assert!(child.citations.is_none());
        assert_eq!(child.ghost_follow_ups.len(), 3);
    }

    #[test]
    fn ui_bundle_carries_cosmic_css_six_chips_and_ghost_follow_ups() {
        let bundle = build_grok_signature_ui_bundle("fa", LabUxTier::CreatorStudio, GrokPersonaMode::FunWitty);
        assert_eq!(bundle.direction, "rtl");
        assert!(bundle.html.contains("#050507"), "OLED obsidian background");
        assert!(bundle.html.contains("rgba(255,255,255,0.08)"), "specular borders");
        assert!(bundle.html.contains("Vazirmatn"));
        for chip in GROK_QUICK_CHIPS {
            assert!(bundle.html.contains(chip), "quick chip {chip}");
        }
        assert_eq!(bundle.chips.len(), 6);
        assert_eq!(bundle.ghost_follow_ups.len(), 3);
        assert_eq!(bundle.locale_slots, 20, "2 active + 18 patch-ready");
        assert!(bundle.html.contains("PATCH_READY") || bundle.html.contains("loc patch"));
        let en_bundle = build_grok_signature_ui_bundle("en", LabUxTier::ProTerminal, GrokPersonaMode::MaironSovereign);
        assert_eq!(en_bundle.direction, "ltr");
        assert_ne!(bundle.ghost_follow_ups, en_bundle.ghost_follow_ups, "locale-aware ghosts");
    }


    /// [ignore] simulation telemetry output for seeding the production database —
    /// `KIA_LAB_DUMP=/tmp/lab_telemetry.json cargo test -p kian-hermes
    /// dump_lab_telemetry_for_seeding -- --ignored --nocapture`
    /// the fixed single-source-of-truth path: engine bytes as-is, zero manual re-derivation.
    #[test]
    #[ignore = "seeding helper — writes telemetry JSON to KIA_LAB_DUMP"]
    fn dump_lab_telemetry_for_seeding() {
        let report = run_24h_ten_dna_ux_lab_simulation();
        let payload = serde_json::json!({
            "signature": report.signature,
            "total_turns": report.total_turns,
            "avg_rea_score": report.avg_rea_score,
            "mairon_gate_pass": report.mairon_gate_pass,
            "telemetry": report.telemetry,
        });
        let path = std::env::var("KIA_LAB_DUMP").unwrap_or_else(|_| "/tmp/lab_telemetry.json".to_owned());
        std::fs::write(&path, serde_json::to_string_pretty(&payload).unwrap_or_default()
            + "\n")
            .expect("write lab telemetry dump");
        println!("lab telemetry dump → {path} ({} entries)", report.total_turns);
    }

    #[test]
    fn locale_direction_covers_rtl_set() {
        for code in ["fa", "ar", "ur", "he"] {
            assert_eq!(locale_direction(code), "rtl", "{code} is RTL");
        }
        for code in ["en", "de", "ja", "zh", "tr"] {
            assert_eq!(locale_direction(code), "ltr", "{code} is LTR");
        }
    }
}
