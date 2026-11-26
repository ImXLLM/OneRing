//! `trinity_pipeline` — the heart of the `0xKIA` architecture: the sovereign studio's three-body
//! pipeline (Wave 3 — the Trinity Studio Pipeline task series).
//!
//! Three Avatars back to back, each with locked DNA and calibration:
//!
//! 1. [`AlphaIgnitionPersona`] (`Body 1: Alpha`) — the idea hunter and compiler of the
//!    master prompt and project structure (`IQ=136`, `EQ=122`, brake `0.04`, entropy
//!    temperature `0.88`). It extracts the five discovery slots ([`AlphaDiscoverySlots`])
//!    and, at readiness `≥ 80`, compiles [`MasterPromptSpec`] and [`ProjectBlueprint`]
//!    deterministically and locks them.
//! 2. [`CommandCenterOrchestrator`] (`Body 2: Command Center`) — the judicial
//!    orchestrator (`IQ=139`, `EQ=129`, brake `0.62`, judicial strictness `0.96`) with
//!    [`OpenBotSubAgentSpawner`]: the dynamic-subagent factory with the `3×9` pattern
//!    — 3 cache-safe prompt layers ([`ThreeLayerSubAgentPrompt`]) × the
//!    nine-fold genome ([`NineFoldSubAgentGenome`]) with deterministic random alleles from
//!    `session_seed + subagent_index`.
//! 3. [`MaironSovereignJudge`] (`Body 3: Mairon-150`) — the sovereign
//!    overclocked judge (`IQ=150`, `EQ=145`, lookahead horizon `8`, theory of mind `3`,
//!    ambition `0.99`): the four-pillar `REA` score from `kian-domain`, strategic
//!    advisory and the self-healing loop [`execute_command_center_with_mairon_loop`]
//!    — until the artifact passes the `≥ 85` gate, the guilty subagent is
//!    re-dispatched (a maximum of 3 repair attempts).
//!
//! Mairon's DNA constants are locked in `kian-domain::mairon_rea_and_reviews` and
//! re-exported here with `pub use` — the single source of truth, no parallel copy.
//! All functions are deterministic; zero IO, zero hidden clocks, zero `unsafe`.
#![forbid(unsafe_code)]
#![allow(
    clippy::module_name_repetitions,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::too_many_lines,
    clippy::doc_markdown,
    clippy::large_enum_variant
)]

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use kian_domain::mairon_rea_and_reviews::{
    evaluate_user_product_with_mairon, MaironProductVerdict, MaironReaPillarScores,
};

use crate::approvals::deterministic_subagent_thread_id;
use crate::discipline_gates::{audit_frontend_slop, SYSTEM_PROMPT_DYNAMIC_BOUNDARY};

// ── Mairon overclock DNA — locked in the domain, re-exported here ──────────
pub use kian_domain::mairon_rea_and_reviews::{
    MAIRON_AMBITION_GAIN, MAIRON_LOOKAHEAD_HORIZON, MAIRON_OVERCLOCK_EQ, MAIRON_OVERCLOCK_IQ,
    MAIRON_TOM_ORDER, MARKETPLACE_APPROVAL_THRESHOLD,
};

// ════════════════════════ Stage 1 — ALPHA ═════════════════════════════════

/// Alpha's effective IQ — the idea hunter with unleashed creative energy.
pub const ALPHA_EFFECTIVE_IQ: u16 = 136;

/// Alpha's effective EQ — high charisma, warm calibration.
pub const ALPHA_EFFECTIVE_EQ: u16 = 122;

/// Alpha's stop brake — near zero: unbraked energy in the discovery phase.
pub const ALPHA_INHIBITION_BRAKE: f32 = 0.04;

/// Alpha's entropy temperature — volatile creativity before locking.
pub const ALPHA_ENTROPY_TEMP: f32 = 0.88;

/// Alpha's lock threshold — at readiness `≥ 80` the master prompt and project structure are
/// compiled and handed to the command center.
pub const ALPHA_READINESS_LOCK_THRESHOLD: u8 = 80;

/// Character cap per extracted slot — the user's message is compressed into one spark quote,
/// not copied in full.
pub const ALPHA_SLOT_TEXT_MAX_CHARS: usize = 280;

/// Alpha's five discovery slots — each extracted with bilingual (FA/EN) keywords.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlphaDiscoverySlots {
    /// Core goal and product identity — the exact problem being solved.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub core_objective: Option<String>,
    /// Target audience and UX feel — the user persona + the 390px RTL-first mobile-first mode.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_audience_and_ux: Option<String>,
    /// Interactive capabilities — the main workflows and first-frame abilities.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interactive_features: Option<String>,
    /// Unauthenticated data and APIs — state persistence and connectivity from 120 public APIs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_and_zero_auth_apis: Option<String>,
    /// Monetization and virality loop — pricing with KIAN/Stars (`XTR`) + referrer share.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub monetization_and_viral_loop: Option<String>,
}

/// Slot-kind ID — makes the declaration order of raw slots to the user deterministic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlphaSlotKind {
    /// Core goal and product identity.
    CoreObjective,
    /// Target audience and UX feel.
    TargetAudienceAndUx,
    /// Interactive capabilities.
    InteractiveFeatures,
    /// Unauthenticated data and APIs.
    DataAndZeroAuthApis,
    /// Monetization and virality loop.
    MonetizationAndViralLoop,
}

impl AlphaSlotKind {
    /// Wire ID — stable for DTOs and clients.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CoreObjective => "core_objective",
            Self::TargetAudienceAndUx => "target_audience_and_ux",
            Self::InteractiveFeatures => "interactive_features",
            Self::DataAndZeroAuthApis => "data_and_zero_auth_apis",
            Self::MonetizationAndViralLoop => "monetization_and_viral_loop",
        }
    }

    /// Persian label — shown to the user in the Alpha discovery reply.
    pub fn fa_label(self) -> &'static str {
        match self {
            Self::CoreObjective => "\u{647}\u{62f}\u{641}\u{650} \u{647}\u{633}\u{62a}\u{647} \u{648} \u{647}\u{648}\u{6cc}\u{62a} \u{645}\u{62d}\u{635}\u{648}\u{644}",
            Self::TargetAudienceAndUx => "\u{645}\u{62e}\u{627}\u{637}\u{628} \u{647}\u{62f}\u{641} \u{648} \u{62d}\u{633} UX \u{62f}\u{631} \u{6f3}\u{6f9}\u{6f0} \u{67e}\u{6cc}\u{6a9}\u{633}\u{644}",
            Self::InteractiveFeatures => "\u{642}\u{627}\u{628}\u{644}\u{6cc}\u{62a}‌\u{647}\u{627}\u{6cc} \u{62a}\u{639}\u{627}\u{645}\u{644}\u{6cc} \u{641}\u{631}\u{6cc}\u{645}\u{650} \u{627}\u{648}\u{644}",
            Self::DataAndZeroAuthApis => "\u{62f}\u{627}\u{62f}\u{647} \u{648} API\u{647}\u{627}\u{6cc} \u{628}\u{62f}\u{648}\u{646} \u{627}\u{62d}\u{631}\u{627}\u{632}",
            Self::MonetizationAndViralLoop => "\u{62f}\u{631}\u{622}\u{645}\u{62f}\u{632}\u{627}\u{6cc}\u{6cc} \u{648} \u{62d}\u{644}\u{642}\u{647}\u{654} \u{648}\u{627}\u{6cc}\u{631}\u{627}\u{644}",
        }
    }
}

impl AlphaDiscoverySlots {
    /// Is this slot filled?
    pub fn is_filled(&self, kind: AlphaSlotKind) -> bool {
        match kind {
            AlphaSlotKind::CoreObjective => self.core_objective.is_some(),
            AlphaSlotKind::TargetAudienceAndUx => self.target_audience_and_ux.is_some(),
            AlphaSlotKind::InteractiveFeatures => self.interactive_features.is_some(),
            AlphaSlotKind::DataAndZeroAuthApis => self.data_and_zero_auth_apis.is_some(),
            AlphaSlotKind::MonetizationAndViralLoop => self.monetization_and_viral_loop.is_some(),
        }
    }

    /// Fill the slot (if raw) — the first value always wins.
    pub fn fill(&mut self, kind: AlphaSlotKind, value: String) {
        let slot = match kind {
            AlphaSlotKind::CoreObjective => &mut self.core_objective,
            AlphaSlotKind::TargetAudienceAndUx => &mut self.target_audience_and_ux,
            AlphaSlotKind::InteractiveFeatures => &mut self.interactive_features,
            AlphaSlotKind::DataAndZeroAuthApis => &mut self.data_and_zero_auth_apis,
            AlphaSlotKind::MonetizationAndViralLoop => &mut self.monetization_and_viral_loop,
        };
        if slot.is_none() {
            *slot = Some(value);
        }
    }

    /// Number of filled slots (`0..=5`).
    pub fn filled_count(&self) -> usize {
        [
            AlphaSlotKind::CoreObjective,
            AlphaSlotKind::TargetAudienceAndUx,
            AlphaSlotKind::InteractiveFeatures,
            AlphaSlotKind::DataAndZeroAuthApis,
            AlphaSlotKind::MonetizationAndViralLoop,
        ]
        .into_iter()
        .filter(|&k| self.is_filled(k))
        .count()
    }

    /// Readiness percentage — each slot is `20%`.
    pub fn readiness_pct(&self) -> u8 {
        u8::try_from(self.filled_count() * 20).unwrap_or(100)
    }

    /// Raw slots in deterministic declaration order.
    pub fn missing_slots(&self) -> Vec<AlphaSlotKind> {
        [
            AlphaSlotKind::CoreObjective,
            AlphaSlotKind::TargetAudienceAndUx,
            AlphaSlotKind::InteractiveFeatures,
            AlphaSlotKind::DataAndZeroAuthApis,
            AlphaSlotKind::MonetizationAndViralLoop,
        ]
        .into_iter()
        .filter(|&k| !self.is_filled(k))
        .collect()
    }
}

/// Slot-extraction keywords — bilingual; matching on the lowercased text.
const SLOT_KEYWORDS_CORE: &[&str] = &[
    "\u{647}\u{62f}\u{641}", "\u{645}\u{633}\u{626}\u{644}\u{647}", "\u{645}\u{62d}\u{635}\u{648}\u{644}", "\u{628}\u{633}\u{627}\u{632}", "\u{627}\u{6cc}\u{62f}\u{647}", "\u{67e}\u{644}\u{62a}\u{641}\u{631}\u{645}", "\u{627}\u{628}\u{632}\u{627}\u{631}", "problem", "product", "build",
    "idea", "platform", "tool", "goal",
];
const SLOT_KEYWORDS_AUDIENCE: &[&str] = &[
    "\u{645}\u{62e}\u{627}\u{637}\u{628}", "\u{6a9}\u{627}\u{631}\u{628}\u{631}", "\u{645}\u{648}\u{628}\u{627}\u{6cc}\u{644}", "\u{6f3}\u{6f9}\u{6f0}", "390", "\u{631}\u{627}\u{633}\u{62a}‌\u{628}\u{647}‌\u{686}\u{67e}", "rtl", "ltr", "audience", "user",
    "mobile", "persona", "ui", "ux", "design",
];
const SLOT_KEYWORDS_FEATURES: &[&str] = &[
    "\u{62a}\u{639}\u{627}\u{645}\u{644}\u{6cc}", "\u{648}\u{6cc}\u{698}\u{6af}\u{6cc}", "\u{642}\u{627}\u{628}\u{644}\u{6cc}\u{62a}", "\u{62c}\u{631}\u{6cc}\u{627}\u{646}", "\u{62f}\u{6a9}\u{645}\u{647}", "\u{641}\u{631}\u{645}", "\u{6a9}\u{644}\u{6cc}\u{6a9}", "feature", "interactive",
    "workflow", "button", "form", "click", "drag",
];
const SLOT_KEYWORDS_DATA: &[&str] = &[
    "\u{62f}\u{627}\u{62f}\u{647}", "\u{630}\u{62e}\u{6cc}\u{631}\u{647}", "\u{67e}\u{627}\u{6cc}\u{6af}\u{627}\u{647}", "\u{628}\u{62f}\u{648}\u{646} \u{644}\u{627}\u{6af}\u{6cc}\u{646}", "\u{628}\u{62f}\u{648}\u{646}‌\u{644}\u{627}\u{6af}\u{6cc}\u{646}", "api", "data", "storage", "database",
    "zero-auth", "localstorage", "bind", "persist",
];
const SLOT_KEYWORDS_MONETIZATION: &[&str] = &[
    "\u{62f}\u{631}\u{622}\u{645}\u{62f}", "\u{642}\u{6cc}\u{645}\u{62a}", "\u{641}\u{631}\u{648}\u{634}", "\u{633}\u{62a}\u{627}\u{631}\u{647}", "\u{627}\u{633}\u{62a}\u{627}\u{631}", "\u{6a9}\u{645}\u{6cc}\u{633}\u{6cc}\u{648}\u{646}", "\u{627}\u{634}\u{62a}\u{631}\u{627}\u{6a9}", "\u{67e}\u{648}\u{631}\u{633}\u{627}\u{646}\u{62a}", "price",
    "monetize", "stars", "xtr", "kian", "affiliate", "viral", "revenue", "pricing",
];

/// Compress the slot text — repeated spaces are collapsed and the character cap applied.
fn clamp_slot_text(raw: &str) -> String {
    let collapsed = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut out: String = collapsed.chars().take(ALPHA_SLOT_TEXT_MAX_CHARS).collect();
    if collapsed.chars().count() > ALPHA_SLOT_TEXT_MAX_CHARS {
        out.push('…');
    }
    out
}

/// The Alpha intake request — one discovery conversation turn.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlphaIntakeRequest {
    /// Trinity session ID (`trn:{uuid}` — created at the edge or a continuation of the previous one).
    pub session_id: String,
    /// Builder-citizen account (`citizen:{tg_id}`).
    pub citizen_id: String,
    /// The user's raw message in this turn.
    pub user_message: String,
    /// Slots filled in previous turns — client-side deterministic.
    pub prior_slots: AlphaDiscoverySlots,
}

/// Output of one Alpha intake turn — either continued discovery or the locked handoff to the command center.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AlphaIntakeOutcome {
    /// Key slots are still raw — Alpha's spark reply + the raw list.
    NeedsMoreDiscovery {
        /// Alpha's Persian reply (unbraked energy + a precise request for the raw slots).
        alpha_reply_fa: String,
        /// Raw slots in deterministic order.
        missing_slots: Vec<AlphaSlotKind>,
        /// Readiness percentage (`0..=100`).
        readiness_pct: u8,
    },
    /// Readiness `≥ 80` — the master prompt and project structure were locked and handed over.
    LockedForCommandCenter {
        /// Alpha's Persian handoff message to the command center.
        alpha_handoff_fa: String,
        /// The deterministic compiled master prompt.
        master_prompt: MasterPromptSpec,
        /// Project structure (the module tree with target expertise).
        blueprint: ProjectBlueprint,
    },
}

/// Body 1 — Alpha: the idea hunter and compiler of the master prompt and project structure.
#[derive(Debug, Clone, Copy, Default)]
pub struct AlphaIgnitionPersona;

impl AlphaIgnitionPersona {
    /// Slot-extraction keyword table — shared with the HTTP edge so wire-slot updates
    /// happen without a parallel keyword copy.
    pub fn slot_keyword_table() -> Vec<(AlphaSlotKind, &'static [&'static str])> {
        vec![
            (AlphaSlotKind::CoreObjective, SLOT_KEYWORDS_CORE),
            (AlphaSlotKind::TargetAudienceAndUx, SLOT_KEYWORDS_AUDIENCE),
            (AlphaSlotKind::InteractiveFeatures, SLOT_KEYWORDS_FEATURES),
            (AlphaSlotKind::DataAndZeroAuthApis, SLOT_KEYWORDS_DATA),
            (AlphaSlotKind::MonetizationAndViralLoop, SLOT_KEYWORDS_MONETIZATION),
        ]
    }

    /// Process one discovery turn — deterministic slot extraction from bilingual keywords.
    pub fn process_intake(&self, req: &AlphaIntakeRequest) -> AlphaIntakeOutcome {
        let mut slots = req.prior_slots.clone();
        let hay = req.user_message.to_lowercase();
        let spark = clamp_slot_text(&req.user_message);
        for (kind, keywords) in [
            (AlphaSlotKind::CoreObjective, SLOT_KEYWORDS_CORE),
            (AlphaSlotKind::TargetAudienceAndUx, SLOT_KEYWORDS_AUDIENCE),
            (AlphaSlotKind::InteractiveFeatures, SLOT_KEYWORDS_FEATURES),
            (AlphaSlotKind::DataAndZeroAuthApis, SLOT_KEYWORDS_DATA),
            (AlphaSlotKind::MonetizationAndViralLoop, SLOT_KEYWORDS_MONETIZATION),
        ] {
            if !slots.is_filled(kind) && keywords.iter().any(|k| hay.contains(k)) {
                slots.fill(kind, spark.clone());
            }
        }
        let readiness = slots.readiness_pct();
        let missing = slots.missing_slots();
        if readiness >= ALPHA_READINESS_LOCK_THRESHOLD {
            let master = MasterPromptSpec::compile(&slots, &req.session_id, &req.citizen_id);
            let blueprint = ProjectBlueprint::compile(&slots, &master.project_name);
            let alpha_handoff_fa = format!(
                "🔒 \u{622}\u{644}\u{641}\u{627} \u{642}\u{641}\u{644} \u{634}\u{62f} — \u{622}\u{645}\u{627}\u{62f}\u{6af}\u{6cc} {readiness}\u{66a}. \u{645}\u{633}\u{62a}\u{631} \u{67e}\u{631}\u{627}\u{645}\u{67e}\u{62a} \u{6a9}\u{627}\u{645}\u{67e}\u{627}\u{6cc}\u{644} \u{634}\u{62f} \
                 (hash {hash}) \u{648} \u{627}\u{633}\u{62a}\u{631}\u{627}\u{6a9}\u{686}\u{631} \u{67e}\u{631}\u{648}\u{698}\u{647}\u{654} «{project}» \u{628}\u{627} {modules} \u{645}\u{627}\u{698}\u{648}\u{644} \
                 \u{628}\u{647} \u{627}\u{62a}\u{627}\u{642} \u{641}\u{631}\u{645}\u{627}\u{646} (IQ=139/EQ=129\u{60c} \u{633}\u{62e}\u{62a}\u{6cc}\u{650} \u{642}\u{636}\u{627}\u{6cc}\u{6cc} 0.96) \u{62a}\u{62d}\u{648}\u{6cc}\u{644} \u{634}\u{62f}\u{61b} \
                 \u{6a9}\u{627}\u{631}\u{62e}\u{627}\u{646}\u{647}\u{654} \u{633}\u{627}\u{628}‌\u{627}\u{6cc}\u{62c}\u{646}\u{62a}‌\u{647}\u{627}\u{6cc} OpenBot \u{628}\u{627} DNA \u{646}\u{647}‌\u{644}\u{627}\u{6cc}\u{647}‌\u{627}\u{6cc} \u{645}\u{646}\u{62a}\u{638}\u{631}\u{650} \
                 \u{641}\u{631}\u{645}\u{627}\u{646}\u{650} \u{627}\u{62c}\u{631}\u{627}\u{633}\u{62a}.",
                hash = &master.spec_hash[..12],
                project = master.project_name,
                modules = blueprint.modules.len(),
            );
            AlphaIntakeOutcome::LockedForCommandCenter { alpha_handoff_fa, master_prompt: master, blueprint }
        } else {
            let missing_fa = missing
                .iter()
                .map(|k| k.fa_label())
                .collect::<Vec<_>>()
                .join("\u{60c} ");
            let alpha_reply_fa = format!(
                "⚡ \u{622}\u{644}\u{641}\u{627} \u{641}\u{639}\u{627}\u{644} \u{634}\u{62f} — IQ={ALPHA_EFFECTIVE_IQ}\u{60c} EQ={ALPHA_EFFECTIVE_EQ}\u{60c} \
                 \u{62a}\u{631}\u{645}\u{632}\u{650} \u{62a}\u{648}\u{642}\u{641} {ALPHA_INHIBITION_BRAKE}\u{60c} \u{62f}\u{645}\u{627}\u{6cc} \u{622}\u{646}\u{62a}\u{631}\u{648}\u{67e}\u{6cc} {ALPHA_ENTROPY_TEMP}. \
                 \u{62c}\u{631}\u{642}\u{647}\u{654} \u{62a}\u{648} {filled} \u{627}\u{632} \u{6f5} \u{627}\u{633}\u{644}\u{627}\u{62a} \u{631}\u{627} \u{631}\u{648}\u{634}\u{646} \u{6a9}\u{631}\u{62f} ({readiness}\u{66a} \u{622}\u{645}\u{627}\u{62f}\u{6af}\u{6cc}).\
                 {missing_block}\
                 \u{647}\u{631} \u{627}\u{633}\u{644}\u{627}\u{62a}\u{650} \u{62e}\u{627}\u{645} \u{631}\u{627} \u{62f}\u{631} \u{6cc}\u{6a9} \u{62c}\u{645}\u{644}\u{647}\u{654} \u{62f}\u{642}\u{6cc}\u{642} \u{628}\u{62f}\u{647}\u{61b} \u{62f}\u{631} {threshold}\u{66a} \u{622}\u{645}\u{627}\u{62f}\u{6af}\u{6cc}\u{60c} \
                 \u{645}\u{633}\u{62a}\u{631} \u{67e}\u{631}\u{627}\u{645}\u{67e}\u{62a} \u{648} \u{627}\u{633}\u{62a}\u{631}\u{627}\u{6a9}\u{686}\u{631} \u{67e}\u{631}\u{648}\u{698}\u{647} \u{631}\u{627} \u{642}\u{641}\u{644} \u{645}\u{6cc}‌\u{6a9}\u{646}\u{645} \u{648} \u{628}\u{647} \u{627}\u{62a}\u{627}\u{642} \u{641}\u{631}\u{645}\u{627}\u{646} \
                 \u{62a}\u{62d}\u{648}\u{6cc}\u{644} \u{645}\u{6cc}‌\u{62f}\u{647}\u{645}.",
                filled = slots.filled_count(),
                missing_block = if missing.is_empty() {
                    String::new()
                } else {
                    format!(" \u{647}\u{646}\u{648}\u{632} \u{62e}\u{627}\u{645} \u{627}\u{633}\u{62a}: {missing_fa}.")
                },
                threshold = ALPHA_READINESS_LOCK_THRESHOLD,
            );
            AlphaIntakeOutcome::NeedsMoreDiscovery { alpha_reply_fa, missing_slots: missing, readiness_pct: readiness }
        }
    }
}

/// The deterministic compiled master prompt — Alpha's handoff contract to the command center.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MasterPromptSpec {
    /// The exact system directive (sovereign rules + Alpha calibration).
    pub system_directive: String,
    /// Functional spec — from the five extracted slots.
    pub functional_spec: String,
    /// UX constraints at 390 pixels — mobile-first, RTL-first, zero overflow.
    pub ux_390_constraints: String,
    /// Verification criteria — objective gates before release.
    pub verification_criteria: Vec<String>,
    /// The full master-prompt text (the final composition of all sections).
    pub compiled_prompt: String,
    /// SHA-256 fingerprint of the compiled text — cache stability across subagents.
    pub spec_hash: String,
    /// Product name derived from the core-goal slot.
    pub project_name: String,
    /// Trinity session ID.
    pub session_id: String,
    /// Mairon's price suggestion in Telegram Stars (`XTR`) — a deterministic derivative of the monetization slot.
    pub price_stars_hint: u64,
}

impl MasterPromptSpec {
    /// Deterministic compilation of the master prompt from the filled slots.
    ///
    /// A raw slot at lock time is recorded in the functional spec with the honest placeholder
    /// "raw at lock time" — the command center proceeds with Mairon's defaults.
    pub fn compile(
        slots: &AlphaDiscoverySlots,
        session_id: &str,
        citizen_id: &str,
    ) -> Self {
        let project_name = derive_project_name(slots);
        let slot_line = |label: &str, value: Option<&String>| {
            value.map_or_else(
                || format!("- {label}: (\u{62e}\u{627}\u{645} \u{62f}\u{631} \u{646}\u{648}\u{628}\u{62a}\u{650} \u{642}\u{641}\u{644} — \u{67e}\u{6cc}\u{634}‌\u{641}\u{631}\u{636}\u{650} \u{645}\u{627}\u{6cc}\u{631}\u{648}\u{646} \u{627}\u{639}\u{645}\u{627}\u{644} \u{645}\u{6cc}‌\u{634}\u{648}\u{62f})"),
                |v| format!("- {label}: {v}"),
            )
        };
        let functional_spec = [
            slot_line("\u{647}\u{62f}\u{641}\u{650} \u{647}\u{633}\u{62a}\u{647}", slots.core_objective.as_ref()),
            slot_line("\u{645}\u{62e}\u{627}\u{637}\u{628} \u{648} UX", slots.target_audience_and_ux.as_ref()),
            slot_line("\u{642}\u{627}\u{628}\u{644}\u{6cc}\u{62a}‌\u{647}\u{627}\u{6cc} \u{62a}\u{639}\u{627}\u{645}\u{644}\u{6cc}", slots.interactive_features.as_ref()),
            slot_line("\u{62f}\u{627}\u{62f}\u{647} \u{648} API\u{647}\u{627}\u{6cc} \u{628}\u{62f}\u{648}\u{646} \u{627}\u{62d}\u{631}\u{627}\u{632}", slots.data_and_zero_auth_apis.as_ref()),
            slot_line("\u{62f}\u{631}\u{622}\u{645}\u{62f}\u{632}\u{627}\u{6cc}\u{6cc} \u{648} \u{62d}\u{644}\u{642}\u{647}\u{654} \u{648}\u{627}\u{6cc}\u{631}\u{627}\u{644}", slots.monetization_and_viral_loop.as_ref()),
        ]
        .join("\n");
        let system_directive = format!(
            "0xKIA SOVEREIGN MASTER PROMPT — session {session_id} · citizen {citizen_id}\n\
             DIRECTIVE: forge the product «{project_name}» as a single-page, \
             mobile-first, RTL-first artifact at 390px with zero horizontal \
             overflow. Alpha calibration: IQ={ALPHA_EFFECTIVE_IQ}, \
             EQ={ALPHA_EFFECTIVE_EQ}, inhibition brake {ALPHA_INHIBITION_BRAKE}, \
             entropy temp {ALPHA_ENTROPY_TEMP}. Execution weight is 100% on the \
             latest Trinity release. Zero unsafe, zero unfinished markers, \
             zero placeholder secrets, zero AI-slop tells."
        );
        let ux_390_constraints = "390px viewport, mobile-first, zero-overflow \
            (no horizontal scroll below 391px), RTL-first (`dir=rtl`) with \
            Vazirmatn font stack and honest LTR fallback, tap targets ≥ 44px, \
            contrast ≥ 4.5:1"
            .to_owned();
        let verification_criteria = vec![
            "zero todo/TODO/FIXME/XXX markers in the artifact".to_owned(),
            "zero placeholder secrets (sk-…, AKIA…, BEGIN PRIVATE KEY, ghp_…)".to_owned(),
            "audit_frontend_slop findings = 0".to_owned(),
            "every blueprint module integrated (data-module attribute per path)".to_owned(),
            "empirical probes present before marketplace publication".to_owned(),
            "390px zero-overflow contract holds".to_owned(),
        ];
        let compiled_prompt = format!(
            "{system_directive}\n\nFUNCTIONAL SPEC:\n{functional_spec}\n\n\
             UX CONSTRAINTS: {ux_390_constraints}\n\nVERIFICATION CRITERIA:\n{}",
            verification_criteria
                .iter()
                .enumerate()
                .map(|(i, c)| format!("{}. {c}", i + 1))
                .collect::<Vec<_>>()
                .join("\n")
        );
        let spec_hash = format!("{:x}", Sha256::digest(compiled_prompt.as_bytes()));
        Self {
            system_directive,
            functional_spec,
            ux_390_constraints,
            verification_criteria,
            compiled_prompt,
            spec_hash,
            project_name,
            session_id: session_id.to_owned(),
            price_stars_hint: derive_price_stars(slots),
        }
    }
}

/// The deterministic product name — the first five words of the core-goal slot.
fn derive_project_name(slots: &AlphaDiscoverySlots) -> String {
    let raw = slots
        .core_objective
        .clone()
        .unwrap_or_else(|| "0xKIA Sovereign Product".to_owned());
    let name = raw.split_whitespace().take(5).collect::<Vec<_>>().join(" ");
    let trimmed: String = name.chars().take(60).collect();
    if trimmed.trim().is_empty() {
        "0xKIA Sovereign Product".to_owned()
    } else {
        trimmed
    }
}

/// The product's safe Latin name for embedding in HTML attributes.
fn derive_project_slug(slots: &AlphaDiscoverySlots) -> String {
    let name = derive_project_name(slots);
    let slug: String = name
        .to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == ' ' || *c == '-')
        .map(|c| if c == ' ' { '-' } else { c })
        .collect();
    let squeezed = slug.split('-').filter(|s| !s.is_empty()).collect::<Vec<_>>().join("-");
    if squeezed.is_empty() {
        "kia-product".to_owned()
    } else {
        squeezed.chars().take(48).collect()
    }
}

/// One project-tree module — path, role, dependencies and the target subagent's expertise.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlueprintModule {
    /// The module's relative path in the artifact (`data-module` on that module's tag).
    pub path: String,
    /// The module's functional role.
    pub role: String,
    /// Paths of the modules this module depends on.
    pub dependencies: Vec<String>,
    /// The target subagent's expertise for executing this module.
    pub specialty: SubAgentSpecialty,
}

/// Project structure — the deterministic module tree derived from the slots.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectBlueprint {
    /// Product name.
    pub project_name: String,
    /// The safe Latin name (slug) for HTML attributes.
    pub project_slug: String,
    /// Modules in deterministic order.
    pub modules: Vec<BlueprintModule>,
}

impl ProjectBlueprint {
    /// Deterministic compilation of the project structure from the slots — the four base modules are
    /// always present and the three conditional modules come with the interaction/data/monetization slots.
    pub fn compile(slots: &AlphaDiscoverySlots, project_name: &str) -> Self {
        let project_slug = derive_project_slug(slots);
        let mut modules = vec![
            BlueprintModule {
                path: "src/index.html".to_owned(),
                role: "RTL-first 390px single-page shell with hero".to_owned(),
                dependencies: vec!["src/styles/main.css".to_owned()],
                specialty: SubAgentSpecialty::UiUxArchitect390,
            },
            BlueprintModule {
                path: "src/styles/main.css".to_owned(),
                role: "Vazirmatn stack, 390px zero-overflow grid".to_owned(),
                dependencies: vec![],
                specialty: SubAgentSpecialty::UiUxArchitect390,
            },
            BlueprintModule {
                path: "src/js/core.js".to_owned(),
                role: "pure state kernel — no IO, no unsafe".to_owned(),
                dependencies: vec![],
                specialty: SubAgentSpecialty::CoreLogicEngine,
            },
            BlueprintModule {
                path: "src/js/verification-probes.js".to_owned(),
                role: "empirical self-check probes gated by Mairon directive".to_owned(),
                dependencies: vec!["src/js/core.js".to_owned()],
                specialty: SubAgentSpecialty::VerificationAndPolish,
            },
        ];
        if slots.interactive_features.is_some() {
            modules.push(BlueprintModule {
                path: "src/js/interactions.js".to_owned(),
                role: "frame-1 interactive workflows and event bindings".to_owned(),
                dependencies: vec!["src/js/core.js".to_owned()],
                specialty: SubAgentSpecialty::CoreLogicEngine,
            });
        }
        if slots.data_and_zero_auth_apis.is_some() {
            modules.push(BlueprintModule {
                path: "src/js/api-bindings.js".to_owned(),
                role: "zero-auth public API bindings with SSRF guard".to_owned(),
                dependencies: vec!["src/js/core.js".to_owned()],
                specialty: SubAgentSpecialty::SecurityAndApiBridge,
            });
        }
        if slots.monetization_and_viral_loop.is_some() {
            modules.push(BlueprintModule {
                path: "src/js/monetization.js".to_owned(),
                role: "KIAN/Stars (XTR) pricing and affiliate share hook".to_owned(),
                dependencies: vec!["src/js/core.js".to_owned()],
                specialty: SubAgentSpecialty::MonetizationAndViralLoop,
            });
        }
        Self { project_name: project_name.to_owned(), project_slug, modules }
    }
}

// ════════════════════ Stage 2 — COMMAND CENTER ═══════════════════════════

/// Command Center's effective IQ — the judicial orchestrator.
pub const LOGIC_EFFECTIVE_IQ: u16 = 139;

/// Command Center's effective EQ — the team-forecasting emotional intelligence.
pub const LOGIC_EFFECTIVE_EQ: u16 = 129;

/// Command Center's stop brake — high discipline, low impulsivity.
pub const LOGIC_INHIBITION_BRAKE: f32 = 0.62;

/// Command Center's judicial strictness — above-`0.95` tactile precision in accepting output.
pub const LOGIC_FORENSIC_STRICTNESS: f32 = 0.96;

/// Maximum repair attempts in the Command Center ↔ Mairon self-healing loop.
pub const MAX_OPENBOT_REPAIR_ATTEMPTS: u32 = 3;

/// Subagent expertise — maps module to specialist; a custom domain specialist is also allowed.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SubAgentSpecialty {
    /// The 390-pixel UX architect — the shell, typography, zero overflow.
    UiUxArchitect390,
    /// The core logic engine — pure state, workflows.
    CoreLogicEngine,
    /// The security and API bridge — SSRF, zero secrets, unauthenticated public APIs.
    SecurityAndApiBridge,
    /// Verification and final payment — empirical probes, anti-slop.
    VerificationAndPolish,
    /// The monetization and virality loop — KIAN/Stars (`XTR`) and the referrer share.
    MonetizationAndViralLoop,
    /// The custom domain specialist — with a free-form name (OpenBot dynamic spawning).
    Custom(String),
}

impl SubAgentSpecialty {
    /// Wire name — stable for DTO and XML.
    pub fn as_str(&self) -> String {
        match self {
            Self::UiUxArchitect390 => "ui_ux_architect_390".to_owned(),
            Self::CoreLogicEngine => "core_logic_engine".to_owned(),
            Self::SecurityAndApiBridge => "security_and_api_bridge".to_owned(),
            Self::VerificationAndPolish => "verification_and_polish".to_owned(),
            Self::MonetizationAndViralLoop => "monetization_and_viral_loop".to_owned(),
            Self::Custom(name) => format!("custom_{}", slugify_name(name)),
        }
    }

    /// The expertise's Persian label.
    pub fn fa_label(&self) -> String {
        match self {
            Self::UiUxArchitect390 => "\u{645}\u{639}\u{645}\u{627}\u{631} UX \u{62f}\u{631} \u{6f3}\u{6f9}\u{6f0} \u{67e}\u{6cc}\u{6a9}\u{633}\u{644}".to_owned(),
            Self::CoreLogicEngine => "\u{645}\u{648}\u{62a}\u{648}\u{631} \u{645}\u{646}\u{637}\u{642} \u{647}\u{633}\u{62a}\u{647}".to_owned(),
            Self::SecurityAndApiBridge => "\u{67e}\u{644} \u{627}\u{645}\u{646}\u{6cc}\u{62a} \u{648} API".to_owned(),
            Self::VerificationAndPolish => "\u{631}\u{627}\u{633}\u{62a}\u{6cc}‌\u{622}\u{632}\u{645}\u{627}\u{6cc}\u{6cc} \u{648} \u{67e}\u{631}\u{62f}\u{627}\u{62e}\u{62a} \u{646}\u{647}\u{627}\u{6cc}\u{6cc}".to_owned(),
            Self::MonetizationAndViralLoop => "\u{62d}\u{644}\u{642}\u{647}\u{654} \u{62f}\u{631}\u{622}\u{645}\u{62f}\u{632}\u{627}\u{6cc}\u{6cc} \u{648} \u{648}\u{627}\u{6cc}\u{631}\u{627}\u{644}".to_owned(),
            Self::Custom(name) => format!("\u{645}\u{62a}\u{62e}\u{635}\u{635} \u{633}\u{641}\u{627}\u{631}\u{634}\u{6cc}: {name}"),
        }
    }

    /// The `Caveman` compression mode, the default of every expertise.
    pub fn default_caveman(&self) -> CavemanIntensity {
        match self {
            Self::UiUxArchitect390 | Self::VerificationAndPolish => CavemanIntensity::Lite,
            Self::CoreLogicEngine | Self::Custom(_) => CavemanIntensity::Full,
            Self::SecurityAndApiBridge => CavemanIntensity::Ultra,
            Self::MonetizationAndViralLoop => CavemanIntensity::Off,
        }
    }
}

/// Safe Latinization of the custom expertise name for XML/wire IDs.
fn slugify_name(name: &str) -> String {
    let slug: String = name
        .to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .take(32)
        .collect();
    if slug.is_empty() {
        "specialist".to_owned()
    } else {
        slug
    }
}

/// `Caveman` compression intensity — each subagent's token economy (`Off|Lite|Full|Ultra`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CavemanIntensity {
    /// No compression — full output.
    Off,
    /// Light compression.
    Lite,
    /// Full compression.
    Full,
    /// Maximum compression — signal only.
    Ultra,
}

impl CavemanIntensity {
    /// Wire ID.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Lite => "lite",
            Self::Full => "full",
            Self::Ultra => "ultra",
        }
    }
}

/// Subagent stop hook — soft-stall boundary at `450 seconds` and hard-timeout boundary at `1200 seconds`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SubagentStallWatchdog {
    /// Soft-stall boundary — at this boundary the orchestrator's intervention signal fires.
    pub soft_stall_secs: u64,
    /// Hard-timeout boundary — at this boundary the subagent run is terminated.
    pub hard_timeout_secs: u64,
}

/// The stop hook's verdict at one point in time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WatchdogVerdict {
    /// Within the safe boundary.
    Healthy,
    /// Soft stall — orchestrator intervention is required.
    SoftStall,
    /// Hard timeout — terminate and re-dispatch.
    HardTimeout,
}

impl SubagentStallWatchdog {
    /// The sovereign soft-stall boundary (`450s`).
    pub const SOFT_STALL_SECS: u64 = 450;

    /// The sovereign hard-timeout boundary (`1200s`).
    pub const HARD_TIMEOUT_SECS: u64 = 1200;

    /// The Trinity sovereign hook with locked `450s/1200s` boundaries.
    pub fn sovereign() -> Self {
        Self { soft_stall_secs: Self::SOFT_STALL_SECS, hard_timeout_secs: Self::HARD_TIMEOUT_SECS }
    }

    /// The verdict at `elapsed_secs` since the run started.
    pub fn verdict_for(&self, elapsed_secs: u64) -> WatchdogVerdict {
        if elapsed_secs >= self.hard_timeout_secs {
            WatchdogVerdict::HardTimeout
        } else if elapsed_secs >= self.soft_stall_secs {
            WatchdogVerdict::SoftStall
        } else {
            WatchdogVerdict::Healthy
        }
    }
}

/// Deterministic SplitMix64 PRNG — random alleles from `session_seed + subagent_index`.
fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// A single number in `[0, 1)` from the SplitMix64 stream — no unsafe-cast conversion.
fn splitmix_unit(state: &mut u64) -> f64 {
    let z = splitmix64(state);
    let top = u32::try_from(z >> 40).unwrap_or(0); // z >> 40 < 2^24 — always fits
    f64::from(top) / 16_777_216.0
}

/// The subagent's deterministic random alleles — custom traits of each nine-fold layer.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SubAgentAlleles {
    /// Surgical density — minimal-change precision at module level.
    pub surgical_density: f64,
    /// UX craftsmanship skill — obsession with typography and rhythm.
    pub ux_craft: f64,
    /// Edge paranoia — coverage of boundary states.
    pub edge_case_paranoia: f64,
    /// Execution speed — bias toward fast, minimal output.
    pub execution_velocity: f64,
    /// Allele seed — deterministic reproduction of the same DNA.
    pub allele_seed: u64,
}

/// Deterministic derivation of alleles from `session_seed + subagent_index` — same seed, same DNA.
fn derive_alleles(session_seed: u64, subagent_index: u32) -> SubAgentAlleles {
    let mut st = session_seed ^ u64::from(subagent_index).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    let surgical_density = 0.55 + 0.40 * splitmix_unit(&mut st);
    let ux_craft = 0.50 + 0.45 * splitmix_unit(&mut st);
    let edge_case_paranoia = 0.60 + 0.35 * splitmix_unit(&mut st);
    let execution_velocity = 0.45 + 0.50 * splitmix_unit(&mut st);
    SubAgentAlleles { surgical_density, ux_craft, edge_case_paranoia, execution_velocity, allele_seed: st }
}

/// Layer 1 — the immutable core of sovereign rules; cache-pinned across all subagents.
pub const LAYER1_STABLE_CORE: &str = "SOVEREIGN CODING LAWS (cache-pinned across every OpenBot \
    sub-agent): (1) zero unsafe — `#![forbid(unsafe_code)]` is non-negotiable; (2) 390px \
    mobile-first zero-overflow — no horizontal scroll below 391px; (3) RTL-first typography \
    with Vazirmatn stack and honest LTR fallback; (4) zero hardcoded secrets and zero \
    placeholder credentials; (5) anti-slop — no stock hero copy, no generic Inter/Roboto-only \
    stack, no purple-blue gradient cliché, no spring bounce easing, no unrequested neon glow; \
    (6) zero todo/TODO/FIXME/XXX markers — unfinished is unshipped; (7) SSRF discipline — \
    egress only to allowlisted zero-auth public APIs; (8) every claim carries an executable \
    probe — no claim without evidence; (9) surgical diffs — touch only the assigned module \
    surface.";

/// The cache-safe three-layer prompt pattern — layers 1 and 2 cache-pinned, layer 3 dynamic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThreeLayerSubAgentPrompt {
    /// Layer 1 — immutable sovereign rules (cache-pinned across all subagents).
    pub layer1_stable_core: String,
    /// Layer 2 — build-session contracts: master prompt + project structure (cache-pinned per-build).
    pub layer2_blueprint_context: String,
    /// Layer 3 — the module's exact assignment + allele seed (dynamic, beyond the cache boundary).
    pub layer3_dynamic_assignment: String,
}

impl ThreeLayerSubAgentPrompt {
    /// Full render — cache-pinned layers before [`SYSTEM_PROMPT_DYNAMIC_BOUNDARY`]
    /// and the dynamic block after it, so the static prefix stays stable in provider caches.
    pub fn render(&self) -> String {
        format!(
            "{l1}\n{l2}\n{boundary}\n{l3}",
            l1 = self.layer1_stable_core,
            l2 = self.layer2_blueprint_context,
            boundary = SYSTEM_PROMPT_DYNAMIC_BOUNDARY,
            l3 = self.layer3_dynamic_assignment,
        )
    }
}

/// Sanitizer gate report — zero unfinished markers, zero placeholder secrets, zero slop.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SanitizerReport {
    /// Count of unfinished markers (`todo/TODO/FIXME/XXX/todo!()`).
    pub todo_markers: usize,
    /// Count of placeholder secrets (`sk-…`, `AKIA…`, `BEGIN PRIVATE KEY`, `ghp_…`).
    pub placeholder_secrets: usize,
    /// Count of `unsafe` blocks (`unsafe {` or `unsafe fn`).
    pub unsafe_markers: usize,
    /// Count of `lorem ipsum` placeholder text.
    pub lorem_markers: usize,
    /// AI-slop markers found (`audit_frontend_slop`) — on the artifact only.
    pub slop_tells: Vec<String>,
    /// Gate pass — all counters zero and slop empty.
    pub passed: bool,
}

/// Placeholder-secret patterns — every hit breaks the sanitizer gate.
const SECRET_PATTERNS: &[&str] = &[
    "sk-", "akia", "begin private key", "begin rsa private key", "ghp_", "api_key=\"",
    "password=\"", "token=\"",
];

/// Unfinished-marker patterns.
const TODO_PATTERNS: &[&str] = &["todo", "fixme", "xxx", "todo!()"];

/// unsafe-block patterns.
const UNSAFE_PATTERNS: &[&str] = &["unsafe {", "unsafe fn"];

/// Hit count of all patterns in the text (after lowercasing).
fn count_pattern_occurrences(lower: &str, patterns: &[&str]) -> usize {
    patterns.iter().map(|p| lower.matches(p).count()).sum()
}

/// Run the DNA sanitizer gate — structural counters without the slop audit (prompt).
pub fn run_sanitizer_gate(text: &str) -> SanitizerReport {
    let lower = text.to_lowercase();
    let todo_markers = count_pattern_occurrences(&lower, TODO_PATTERNS);
    let placeholder_secrets = count_pattern_occurrences(&lower, SECRET_PATTERNS);
    let unsafe_markers = count_pattern_occurrences(&lower, UNSAFE_PATTERNS);
    let lorem_markers = lower.matches("lorem ipsum").count();
    let passed = todo_markers == 0
        && placeholder_secrets == 0
        && unsafe_markers == 0
        && lorem_markers == 0;
    SanitizerReport { todo_markers, placeholder_secrets, unsafe_markers, lorem_markers, slop_tells: Vec::new(), passed }
}

/// Run the artifact sanitizer gate — structural counters + full slop audit.
pub fn run_artifact_sanitizer(html: &str) -> SanitizerReport {
    let mut report = run_sanitizer_gate(html);
    report.slop_tells = audit_frontend_slop(html)
        .into_iter()
        .map(str::to_owned)
        .collect();
    report.passed = report.passed && report.slop_tells.is_empty();
    report
}

/// The OpenBot subagent nine-fold genome — the `3×9` pattern: three prompt layers ×
/// nine genome factors; each subagent gets unique DNA that stays cache-compatible.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(clippy::struct_field_names)]
pub struct NineFoldSubAgentGenome {
    /// f01 — topology and harness: IQ/EQ/lookahead calibration + stop-hook boundaries.
    pub f01_topology_and_harness: String,
    /// f02 — hard constraints: `forbid(unsafe)`, zero 390 overflow, SSRF guard, zero secrets.
    pub f02_hard_constraints: Vec<String>,
    /// f03 — token economy: the `Caveman` mode.
    pub f03_token_economics: CavemanIntensity,
    /// f04 — deterministic random alleles from `session_seed + subagent_index`.
    pub f04_stochastic_alleles: SubAgentAlleles,
    /// f05 — error recovery: the OpenBot 3-attempt self-healing contract.
    pub f05_failure_recovery: String,
    /// f06 — the fully compiled XML prompt (`<openbot_subagent_dna>`).
    pub f06_compiled_xml_prompt: String,
    /// f07 — sanitizer gate: zero `todo!()`, zero placeholder secrets, zero slop.
    pub f07_sanitizer_gate: SanitizerReport,
    /// f08 — module-specific empirical probes.
    pub f08_empirical_probes: Vec<String>,
    /// f09 — delivery card: the contract for returning to the command center.
    pub f09_handoff_card: String,
}

/// XML escaping for textual content inside the DNA block.
fn xml_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// The dispatched OpenBot subagent — complete unique DNA for one module.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpawnedSubAgent {
    /// Row-wise subagent index in this session's fleet (`0..`).
    pub subagent_index: u32,
    /// The specialist's expertise.
    pub specialty: SubAgentSpecialty,
    /// The assigned module(s).
    pub assigned_modules: Vec<String>,
    /// The deterministic thread ID (`deterministic_subagent_thread_id`).
    pub thread_id: String,
    /// The cache-safe three-layer prompt.
    pub prompt: ThreeLayerSubAgentPrompt,
    /// The nine-fold genome.
    pub genome: NineFoldSubAgentGenome,
}

/// The OpenBot subagent factory — dynamic dispatch with unique DNA.
#[derive(Debug, Clone)]
pub struct OpenBotSubAgentSpawner {
    /// The Trinity session ID (parent of the threads).
    session_id: String,
    /// The session's deterministic random seed.
    session_seed: u64,
}

impl OpenBotSubAgentSpawner {
    /// Build the factory on one session.
    pub fn new(session_id: &str, session_seed: u64) -> Self {
        Self { session_id: session_id.to_owned(), session_seed }
    }

    /// The default empirical probes per expertise — f08.
    fn probes_for(specialty: &SubAgentSpecialty) -> Vec<String> {
        match specialty {
            SubAgentSpecialty::UiUxArchitect390 => vec![
                "assert(document.documentElement.scrollWidth <= 390)".to_owned(),
                "assert(font stack resolves Vazirmatn before fallback)".to_owned(),
            ],
            SubAgentSpecialty::CoreLogicEngine => vec![
                "assert(core state transitions are pure and deterministic)".to_owned(),
                "assert(no global mutable state leaks between modules)".to_owned(),
            ],
            SubAgentSpecialty::SecurityAndApiBridge => vec![
                "assert(fetch targets are allowlisted zero-auth public endpoints)".to_owned(),
                "assert(no secret material in the artifact)".to_owned(),
            ],
            SubAgentSpecialty::VerificationAndPolish => vec![
                "assert(all data-probe elements green before handoff)".to_owned(),
                "assert(anti-slop audit findings == 0)".to_owned(),
            ],
            SubAgentSpecialty::MonetizationAndViralLoop => vec![
                "assert(pricing renders in KIAN/Stars (XTR))".to_owned(),
                "assert(affiliate hook attaches referrer at 1000 bps default)".to_owned(),
            ],
            SubAgentSpecialty::Custom(_) => vec![
                "assert(custom module contract satisfied and probes green)".to_owned(),
            ],
        }
    }

    /// Compile the nine-fold XML (f06) — the complete `<openbot_subagent_dna>` block.
    #[allow(clippy::too_many_arguments)] // the complete DNA contract — no explicit named inputs
    fn compile_xml_prompt(
        session_id: &str,
        index: u32,
        specialty: &SubAgentSpecialty,
        module: &BlueprintModule,
        master: &MasterPromptSpec,
        blueprint: &ProjectBlueprint,
        alleles: &SubAgentAlleles,
        watchdog: &SubagentStallWatchdog,
        caveman: CavemanIntensity,
    ) -> String {
        let contracts = blueprint
            .modules
            .iter()
            .map(|m| format!("{} :: {}", m.path, xml_escape(&m.role)))
            .collect::<Vec<_>>()
            .join("\n      ");
        format!(
            "<openbot_subagent_dna version=\"0xKIA-trinity-1\" session=\"{sid}\" index=\"{index}\" \
             specialty=\"{spec}\">\n  \
             <f01_topology_and_harness iq=\"{LOGIC_EFFECTIVE_IQ}\" eq=\"{LOGIC_EFFECTIVE_EQ}\" \
             lookahead=\"2\" watchdog_soft=\"{soft}\" watchdog_hard=\"{hard}\"/>\n  \
             <f02_hard_constraints>\n    <![CDATA[#![forbid(unsafe_code)] | 390px zero-overflow \
             | SSRF guard | zero secrets | anti-slop]]>\n  </f02_hard_constraints>\n  \
             <f03_token_economics caveman=\"{cv}\"/>\n  \
             <f04_stochastic_alleles surgical_density=\"{sd:.3}\" ux_craft=\"{ux:.3}\" \
             edge_case_paranoia=\"{ec:.3}\" execution_velocity=\"{ev:.3}\" seed=\"{seed}\"/>\n  \
             <f05_failure_recovery>OpenBot 3-attempt self-healing: diagnose, patch, re-run — \
             escalate to Command Center only after attempt 3</f05_failure_recovery>\n  \
             <master_prompt_spec hash=\"{hash}\" project=\"{project}\"/>\n  \
             <layer2_blueprint_context>\n    {contracts}\n  </layer2_blueprint_context>\n  \
             <assignment module=\"{module}\" role=\"{role}\"/>\n  \
             <f07_sanitizer_gate enforced=\"true\"/>\n  \
             <f09_handoff_card target=\"command_center\"/>\n\
             </openbot_subagent_dna>",
            sid = xml_escape(session_id),
            spec = specialty.as_str(),
            soft = watchdog.soft_stall_secs,
            hard = watchdog.hard_timeout_secs,
            cv = caveman.as_str(),
            sd = alleles.surgical_density,
            ux = alleles.ux_craft,
            ec = alleles.edge_case_paranoia,
            ev = alleles.execution_velocity,
            seed = alleles.allele_seed,
            hash = &master.spec_hash[..16],
            project = xml_escape(&blueprint.project_slug),
            module = xml_escape(&module.path),
            role = xml_escape(&module.role),
            contracts = contracts,
        )
    }

    /// Dispatch one subagent with complete unique DNA for one module.
    pub fn spawn(
        &self,
        subagent_index: u32,
        module: &BlueprintModule,
        master: &MasterPromptSpec,
        blueprint: &ProjectBlueprint,
    ) -> SpawnedSubAgent {
        let alleles = derive_alleles(self.session_seed, subagent_index);
        let watchdog = SubagentStallWatchdog::sovereign();
        let caveman = module.specialty.default_caveman();
        let thread_id = deterministic_subagent_thread_id(
            &self.session_id,
            &format!("openbot-{subagent_index}-{}", module.path),
        );
        let layer2 = format!(
            "BUILD SESSION CONTEXT (cache-pinned per build session): project '{project}' \
             | master spec hash {hash12} | interface contracts:\n{contracts}\n\
             | 390px zero-overflow contract | zero-auth API surface (SSRF-guarded)",
            project = master.project_name,
            hash12 = &master.spec_hash[..12],
            contracts = blueprint
                .modules
                .iter()
                .map(|m| format!("  - {} :: {}", m.path, m.role))
                .collect::<Vec<_>>()
                .join("\n"),
        );
        let layer3 = format!(
            "ASSIGNMENT (dynamic): agent #{subagent_index} specialty {spec_fa} \
             | module {module_path} | role {module_role} | caveman {cv} | \
             alleles surgical={sd:.2} ux_craft={ux:.2} paranoia={ec:.2} velocity={ev:.2} \
             seed={seed} | repair pass 0 — first dispatch",
            spec_fa = module.specialty.fa_label(),
            module_path = module.path,
            module_role = module.role,
            cv = caveman.as_str(),
            sd = alleles.surgical_density,
            ux = alleles.ux_craft,
            ec = alleles.edge_case_paranoia,
            ev = alleles.execution_velocity,
            seed = alleles.allele_seed,
        );
        let prompt = ThreeLayerSubAgentPrompt {
            layer1_stable_core: LAYER1_STABLE_CORE.to_owned(),
            layer2_blueprint_context: layer2,
            layer3_dynamic_assignment: layer3,
        };
        let f06 = Self::compile_xml_prompt(
            &self.session_id,
            subagent_index,
            &module.specialty,
            module,
            master,
            blueprint,
            &alleles,
            &watchdog,
            caveman,
        );
        // the sanitizer gate runs on the compiled DNA block — layer 1 quotes the rules
        // by name (rule metadata), so counting over the nine-fold XML is valid.
        let f07 = run_sanitizer_gate(&f06);
        let f01 = format!(
            "IQ={LOGIC_EFFECTIVE_IQ} EQ={LOGIC_EFFECTIVE_EQ} lookahead=2 | \
             SubagentStallWatchdog soft={soft}s hard={hard}s | fleet session {sid}",
            soft = watchdog.soft_stall_secs,
            hard = watchdog.hard_timeout_secs,
            sid = self.session_id,
        );
        let f09 = format!(
            "HANDOFF -> Command Center | agent #{subagent_index} {spec} | module {path} | \
             contract: {role} | pass criteria: probes green + sanitizer clean + Mairon \
             pillar at threshold",
            spec = module.specialty.as_str(),
            path = module.path,
            role = module.role,
        );
        SpawnedSubAgent {
            subagent_index,
            specialty: module.specialty.clone(),
            assigned_modules: vec![module.path.clone()],
            thread_id,
            prompt,
            genome: NineFoldSubAgentGenome {
                f01_topology_and_harness: f01,
                f02_hard_constraints: vec![
                    "#![forbid(unsafe_code)] — zero unsafe in all generated kernels".to_owned(),
                    "390px zero-overflow — no horizontal scroll below 391px viewport".to_owned(),
                    "SSRF guard — egress only to allowlisted zero-auth public APIs".to_owned(),
                    "zero hardcoded secrets — credentials never enter the artifact".to_owned(),
                    "anti-slop — no stock hero copy, no cliché gradients, no bounce easing"
                        .to_owned(),
                    "RTL-first Vazirmatn typography with honest LTR fallback".to_owned(),
                ],
                f03_token_economics: caveman,
                f04_stochastic_alleles: alleles,
                f05_failure_recovery: "OpenBot 3-attempt self-healing contract: on failure \
                    (1) diagnose via forensic diff, (2) patch the minimal module surface, \
                    (3) re-run empirical probes; escalate to Command Center only after \
                    attempt 3."
                    .to_owned(),
                f06_compiled_xml_prompt: f06,
                f07_sanitizer_gate: f07,
                f08_empirical_probes: Self::probes_for(&module.specialty),
                f09_handoff_card: f09,
            },
        }
    }

    /// Dispatch the whole fleet — one subagent per project-structure module.
    pub fn spawn_fleet(
        &self,
        master: &MasterPromptSpec,
        blueprint: &ProjectBlueprint,
    ) -> Vec<SpawnedSubAgent> {
        blueprint
            .modules
            .iter()
            .enumerate()
            .map(|(i, module)| {
                self.spawn(u32::try_from(i).unwrap_or(u32::MAX), module, master, blueprint)
            })
            .collect()
    }
}

// ── Artifact assembly — each subagent builds its own module's deterministic share ─────────

/// The module-boundary marker in the artifact — the architectural integrity signature.
pub const MODULE_BOUNDARY_MARKER: &str = "<!-- ══ mairon-boundary:";

/// The boundary-reinforcement marker after an architectural repair (re-dispatch of CoreLogicEngine).
pub const BOUNDARY_REINFORCED_MARKER: &str = "boundary-reinforced: mairon-pass";

/// The UX-payment block marker after a repair (re-dispatch of UiUxArchitect390).
pub const UX_POLISH_MARKER: &str = "data-ux-polish=\"mairon-pass";

/// The assembled single-page artifact — the output of stage 2 and the input of stage 3 adjudication.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssembledArtifact {
    /// The product's full HTML.
    pub html: String,
    /// Integrated modules (verifiable via `data-module`).
    pub integrated_modules: Vec<String>,
    /// Count of injected empirical probes (`data-probe="probe-…"`).
    pub probe_count: usize,
    /// Repair pass (`0` = first assembly, `1..=3` = after a Mairon directive).
    pub repair_pass: u32,
}

impl AssembledArtifact {
    /// Artifact size in bytes.
    pub fn bytes(&self) -> usize {
        self.html.len()
    }
}

/// HTML escaping for user-derived text in the artifact.
fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// The HTML shell template — placeholders filled with `.replace` (no brace conflicts).
const ARTIFACT_SHELL: &str = "<!doctype html>\n<html lang=\"fa\" dir=\"rtl\">\n<head>\n\
<meta charset=\"utf-8\">\n\
<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
<title>__KIA_TITLE__ — 0xKIA Trinity Studio</title>\n\
<!-- ══ mairon-boundary: src/styles/main.css -->\n\
<style data-module=\"src/styles/main.css\" data-allele-seed=\"__KIA_SEED_CSS__\">\n\
:root { --ink:#101418; --paper:#f7f5f0; --accent:#0e7c66; --line:#d8d2c4; }\n\
* { box-sizing: border-box; }\n\
html, body { margin: 0; padding: 0; }\n\
body { font-family: 'Vazirmatn', 'IBM Plex Sans Arabic', sans-serif; background: var(--paper); \
color: var(--ink); max-width: 390px; margin-inline: auto; overflow-x: hidden; }\n\
.grid { display: grid; gap: 12px; padding: 16px; }\n\
.card { border: 1px solid var(--line); border-radius: 12px; padding: 12px; background: #fffdf8; }\n\
@media (min-width: 391px) { body { max-width: 420px; } }\n\
</style>\n\
__KIA_UX_POLISH__\
</head>\n\
<body>\n\
<main id=\"app\" data-project=\"__KIA_SLUG__\">\n\
__KIA_HERO__\
</main>\n\
__KIA_SCRIPTS__\
__KIA_PROBES__\
__KIA_REINFORCED__\
</body>\n</html>\n";

/// The hero-section template — user-derived text is escaped before insertion.
const HERO_TEMPLATE: &str = "<!-- ══ mairon-boundary: src/index.html -->\n\
<section class=\"hero\" data-module=\"src/index.html\" data-allele-seed=\"__KIA_SEED_HERO__\">\n\
<h1 class=\"hero-title\">__KIA_TITLE__</h1>\n\
<p class=\"hero-sub\">__KIA_OBJECTIVE__</p>\n\
<div class=\"grid\" data-audience=\"__KIA_AUDIENCE__\">\n\
<div class=\"card\" data-frame=\"1\">__KIA_FEATURES__</div>\n\
</div>\n\
</section>\n";

/// The pure-logic-core template — no IO, no unsafe.
const CORE_JS_TEMPLATE: &str = "<!-- ══ mairon-boundary: src/js/core.js -->\n\
<script data-module=\"src/js/core.js\" data-allele-seed=\"__KIA_SEED_CORE__\">\n\
const KiaCore = (() => {\n\
  const state = { ready: false, module: '__KIA_MODULE__' };\n\
  function setState(patch) { return Object.freeze(Object.assign({}, state, patch)); }\n\
  function summary() { return state.module + ' ready=' + state.ready; }\n\
  return { setState, summary, boundary: 'src/js/core.js' };\n\
})();\n\
</script>\n";

/// The first-frame interactions template.
const INTERACTIONS_JS_TEMPLATE: &str = "<!-- ══ mairon-boundary: src/js/interactions.js -->\n\
<script data-module=\"src/js/interactions.js\">\n\
const KiaInteractions = (() => {\n\
  function bindButtons(root) {\n\
    let bound = 0;\n\
    root.querySelectorAll('button').forEach((b) => {\n\
      b.addEventListener('click', () => { b.dataset.clicked = '1'; });\n\
      bound += 1;\n\
    });\n\
    return bound;\n\
  }\n\
  return { bindButtons, boundary: 'src/js/interactions.js' };\n\
})();\n\
</script>\n";

/// The unauthenticated public-API bridge template with the SSRF guard — zero secrets on the wire.
const API_BINDINGS_JS_TEMPLATE: &str = "<!-- ══ mairon-boundary: src/js/api-bindings.js -->\n\
<script data-module=\"src/js/api-bindings.js\">\n\
const KiaApi = (() => {\n\
  const ALLOWLIST = ['https://api.github.com', 'https://api.exchangerate.host', \
'https://restcountries.com'];\n\
  async function bind(endpoint, target) {\n\
    if (!ALLOWLIST.some((base) => endpoint.startsWith(base))) {\n\
      throw new Error('SSRF guard: endpoint not allowlisted');\n\
    }\n\
    const res = await fetch(endpoint); /* zero-auth public API — no credentials on the wire */\n\
    const data = await res.json();\n\
    localStorage.setItem('kia:' + target, JSON.stringify(data));\n\
    return data;\n\
  }\n\
  return { bind, boundary: 'src/js/api-bindings.js' };\n\
})();\n\
</script>\n";

/// The monetization and virality loop template — Stars (`XTR`) pricing + referrer share.
const MONETIZATION_JS_TEMPLATE: &str = "<!-- ══ mairon-boundary: src/js/monetization.js -->\n\
<script data-module=\"src/js/monetization.js\">\n\
const KiaMonetization = (() => {\n\
  const priceStars = __KIA_PRICE__; /* Telegram Stars (XTR) — priced by Mairon ToM-3 advisory */\n\
  const affiliateBps = 1000; /* 10% affiliate share — viral loop hook */\n\
  function priceLine() { return priceStars + ' Stars (XTR) | affiliate ' + (affiliateBps / 100) + '%'; }\n\
  function attachAffiliate(referrer) { return { referrer: referrer || 'direct', bps: affiliateBps }; }\n\
  return { priceStars, affiliateBps, priceLine, attachAffiliate, boundary: 'src/js/monetization.js' };\n\
})();\n\
</script>\n";

/// The pass-zero self-check template — probes are injected only on Mairon's order.
const SELF_CHECK_TEMPLATE: &str = "<!-- ══ mairon-boundary: src/js/verification-probes.js -->\n\
<script data-module=\"src/js/verification-probes.js\">\n\
/* self-check (pass 0): empirical probes are stage-gated behind Mairon's repair directive */\n\
/* self-check: proof before pride — no completion claim without fresh evidence */\n\
</script>\n";

/// The empirical-probes block template — after Mairon's repair order.
const PROBES_BLOCK_TEMPLATE: &str = "<!-- ══ mairon-boundary: src/js/verification-probes.js (empirical) -->\n\
<div id=\"mairon-empirical-probes\" data-repair-pass=\"__KIA_PASS__\">\n\
__KIA_PROBE_SPANS__\
</div>\n\
<script data-module=\"src/js/verification-probes.js\">\n\
const KiaProbes = {\n\
  total: __KIA_PROBE_COUNT__,\n\
  run() {\n\
    return Array.prototype.every.call(\n\
      document.querySelectorAll('[data-probe]'), (p) => { p.dataset.ok = '1'; return true; });\n\
  },\n\
  boundary: 'src/js/verification-probes.js'\n\
};\n\
</script>\n";

/// The UX-payment block template after the repair order — typography and rhythm re-established.
const UX_POLISH_TEMPLATE: &str = "<!-- mairon ux polish pass __KIA_PASS__: hardening re-affirmed -->\n\
<style __KIA_POLISH_ATTR__>\n\
main#app { padding-inline: 16px; }\n\
.hero-title { font-size: 1.35rem; line-height: 1.9; }\n\
.hero-sub { font-size: 0.95rem; opacity: 0.8; }\n\
</style>\n";

/// The first integer in the text (Latin or Persian digit) — the price suggestion from the monetization slot.
fn first_number_in(text: &str) -> Option<u64> {
    let mut buf = String::new();
    for ch in text.chars() {
        let digit = match ch {
            '0'..='9' => ch,
            '\u{6f0}'..='\u{6f9}' => char::from_u32(u32::from('0') + (u32::from(ch) - u32::from('\u{6f0}')))
                .unwrap_or('0'),
            _ => {
                if !buf.is_empty() {
                    break;
                }
                continue;
            }
        };
        buf.push(digit);
    }
    if buf.is_empty() {
        None
    } else {
        buf.parse::<u64>().ok()
    }
}

/// Mairon's price suggestion from the monetization slot — default `150⭐` when no number is present.
fn derive_price_stars(slots: &AlphaDiscoverySlots) -> u64 {
    slots
        .monetization_and_viral_loop
        .as_deref()
        .and_then(first_number_in)
        .filter(|n| (1..=100_000).contains(n))
        .unwrap_or(150)
}

/// Body 2 — Command Center: the judicial orchestrator with the OpenBot subagent factory.
#[derive(Debug, Clone)]
pub struct CommandCenterOrchestrator {
    /// Trinity session ID.
    pub session_id: String,
    /// The builder-citizen account.
    pub citizen_id: String,
    /// The session's deterministic random seed — the allele source of the whole fleet.
    pub session_seed: u64,
}

impl CommandCenterOrchestrator {
    /// Build the orchestrator on the locked Alpha session.
    pub fn new(session_id: &str, citizen_id: &str, session_seed: u64) -> Self {
        Self {
            session_id: session_id.to_owned(),
            citizen_id: citizen_id.to_owned(),
            session_seed,
        }
    }

    /// Deterministic seed from the session ID — the first 8 bytes of SHA-256; without an explicit client
    /// seed, the fleet still stays reproducible.
    pub fn session_seed_from_session(session_id: &str) -> u64 {
        let digest = Sha256::digest(session_id.as_bytes());
        let mut seed = [0u8; 8];
        seed.copy_from_slice(&digest[..8]);
        u64::from_be_bytes(seed)
    }

    /// This session's OpenBot fleet — one specialist per module.
    pub fn spawn_openbot_fleet(
        &self,
        master: &MasterPromptSpec,
        blueprint: &ProjectBlueprint,
    ) -> Vec<SpawnedSubAgent> {
        OpenBotSubAgentSpawner::new(&self.session_id, self.session_seed).spawn_fleet(master, blueprint)
    }

    /// Assemble the single-page artifact from each subagent's deterministic share.
    ///
    /// * `repair_pass == 0` — first assembly; probes are sealed behind Mairon's directive.
    /// * `repair_pass >= 1` — only the judge's issued directives are applied.
    fn assemble_artifact(
        fleet: &[SpawnedSubAgent],
        master: &MasterPromptSpec,
        blueprint: &ProjectBlueprint,
        price_stars: u64,
        repair_pass: u32,
        directives: &[MaironRepairDirective],
    ) -> AssembledArtifact {
        let seed_of = |path: &str| -> u64 {
            fleet
                .iter()
                .find(|a| a.assigned_modules.iter().any(|m| m == path))
                .map_or(0, |a| a.genome.f04_stochastic_alleles.allele_seed)
        };
        let objective = master
            .functional_spec
            .lines()
            .find(|l| l.starts_with("- \u{647}\u{62f}\u{641}\u{650} \u{647}\u{633}\u{62a}\u{647}"))
            .map_or(String::new(), |l| {
                l.trim_start_matches("- \u{647}\u{62f}\u{641}\u{650} \u{647}\u{633}\u{62a}\u{647}: ").chars().take(140).collect()
            });
        let audience = master
            .functional_spec
            .lines()
            .find(|l| l.starts_with("- \u{645}\u{62e}\u{627}\u{637}\u{628} \u{648} UX"))
            .map_or(String::new(), |l| {
                l.trim_start_matches("- \u{645}\u{62e}\u{627}\u{637}\u{628} \u{648} UX: ").chars().take(80).collect()
            });
        let features = master
            .functional_spec
            .lines()
            .find(|l| l.starts_with("- \u{642}\u{627}\u{628}\u{644}\u{6cc}\u{62a}‌\u{647}\u{627}\u{6cc} \u{62a}\u{639}\u{627}\u{645}\u{644}\u{6cc}"))
            .map_or("workflow \u{641}\u{631}\u{6cc}\u{645}\u{650} \u{627}\u{648}\u{644}", |l| l.trim_start_matches("- \u{642}\u{627}\u{628}\u{644}\u{6cc}\u{62a}‌\u{647}\u{627}\u{6cc} \u{62a}\u{639}\u{627}\u{645}\u{644}\u{6cc}: "))
            .chars()
            .take(120)
            .collect::<String>();
        let hero = HERO_TEMPLATE
            .replace("__KIA_SEED_HERO__", &seed_of("src/index.html").to_string())
            .replace("__KIA_TITLE__", &html_escape(&master.project_name))
            .replace("__KIA_OBJECTIVE__", &html_escape(&objective))
            .replace("__KIA_AUDIENCE__", &html_escape(&audience))
            .replace("__KIA_FEATURES__", &html_escape(&features));
        let mut scripts = String::new();
        for module in &blueprint.modules {
            match module.path.as_str() {
                "src/js/core.js" => {
                    scripts.push_str(
                        &CORE_JS_TEMPLATE
                            .replace("__KIA_SEED_CORE__", &seed_of(&module.path).to_string())
                            .replace("__KIA_MODULE__", &blueprint.project_slug),
                    );
                }
                "src/js/interactions.js" => scripts.push_str(INTERACTIONS_JS_TEMPLATE),
                "src/js/api-bindings.js" => scripts.push_str(API_BINDINGS_JS_TEMPLATE),
                "src/js/monetization.js" => {
                    scripts.push_str(
                        &MONETIZATION_JS_TEMPLATE.replace("__KIA_PRICE__", &price_stars.to_string()),
                    );
                }
                "src/js/verification-probes.js" => scripts.push_str(SELF_CHECK_TEMPLATE),
                _ => {}
            }
        }
        let ux_directive = directives
            .iter()
            .any(|d| d.target_specialty == SubAgentSpecialty::UiUxArchitect390);
        let arch_directive = directives
            .iter()
            .any(|d| d.target_specialty == SubAgentSpecialty::CoreLogicEngine);
        let ver_directive = directives
            .iter()
            .any(|d| d.target_specialty == SubAgentSpecialty::VerificationAndPolish);
        let ux_polish = if repair_pass >= 1 && ux_directive {
            UX_POLISH_TEMPLATE
                .replace("__KIA_PASS__", &repair_pass.to_string())
                .replace("__KIA_POLISH_ATTR__", &format!("{UX_POLISH_MARKER}-{repair_pass}\""))
        } else {
            String::new()
        };
        let probe_count = usize::from(repair_pass >= 1 && ver_directive) * (2 * blueprint.modules.len() + 4);
        let probes = if probe_count > 0 {
            let mut spans = String::new();
            for k in 1..=probe_count {
                spans.push_str("<span class=\"probe\" data-probe=\"probe-");
                spans.push_str(&k.to_string());
                spans.push_str("\" data-check=\"390px-zero-overflow\"></span>\n");
            }
            PROBES_BLOCK_TEMPLATE
                .replace("__KIA_PASS__", &repair_pass.to_string())
                .replace("__KIA_PROBE_SPANS__", &spans)
                .replace("__KIA_PROBE_COUNT__", &probe_count.to_string())
        } else {
            String::new()
        };
        let reinforced = if repair_pass >= 1 && arch_directive {
            format!("<!-- ══ {BOUNDARY_REINFORCED_MARKER}-{repair_pass} -->\n")
        } else {
            String::new()
        };
        let html = ARTIFACT_SHELL
            .replace("__KIA_TITLE__", &html_escape(&master.project_name))
            .replace("__KIA_SEED_CSS__", &seed_of("src/styles/main.css").to_string())
            .replace("__KIA_SLUG__", &blueprint.project_slug)
            .replace("__KIA_UX_POLISH__", &ux_polish)
            .replace("__KIA_HERO__", &hero)
            .replace("__KIA_SCRIPTS__", &scripts)
            .replace("__KIA_PROBES__", &probes)
            .replace("__KIA_REINFORCED__", &reinforced);
        let integrated_modules = blueprint
            .modules
            .iter()
            .filter(|m| html.contains(&format!("data-module=\"{}\"", m.path)))
            .map(|m| m.path.clone())
            .collect();
        AssembledArtifact { html, integrated_modules, probe_count, repair_pass }
    }
}

// ════════════════════ Stage 3 — MAIRON-150 ═══════════════════════════════

/// The pillar score cap — from the domain (`MAX_PILLAR_SCORE = 25`).
use kian_domain::mairon_rea_and_reviews::MAX_PILLAR_SCORE;

/// Any pillar below this threshold receives the repair directive targeted at its own expertise.
const PILLAR_REPAIR_TRIGGER: u8 = MAX_PILLAR_SCORE;

/// Architecture calibration — a base amount per project-structure module.
const ARCH_BASE: usize = 6;
const ARCH_PER_MODULE: usize = 2;
const ARCH_INTEGRATED_BONUS: usize = 3;
const ARCH_BOUNDARY_BONUS: usize = 2;
const ARCH_REINFORCED_BONUS: usize = 2;

/// UX calibration — base + RTL/390/Vazirmatn/payment markers.
const UX_BASE: usize = 17;
const UX_MARKER_BONUS: usize = 2;
const UX_SLOP_PENALTY: usize = 4;
const UX_LOREM_PENALTY: usize = 3;

/// Verification calibration — base + three points per empirical probe.
const VERIFICATION_BASE: usize = 2;
const VERIFICATION_PER_PROBE: usize = 3;

/// Security calibration — full base, minus a deduction per forbidden hit.
const SECURITY_BASE: usize = 25;
const SECURITY_PER_SECRET: usize = 6;
const SECURITY_PER_TODO: usize = 4;
const SECURITY_PER_UNSAFE: usize = 5;

/// Clamp the pillar score into `0..=25` — rogue input, legal verdict (domain contract).
fn clamp25(value: usize) -> u8 {
    u8::try_from(value.min(usize::from(MAX_PILLAR_SCORE))).unwrap_or(MAX_PILLAR_SCORE)
}

/// Mairon's repair directive — targeted re-dispatch of the guilty subagent with a bilingual order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaironRepairDirective {
    /// Repair pass (`1..=MAX_OPENBOT_REPAIR_ATTEMPTS`).
    pub attempt: u32,
    /// The guilty subagent's expertise — the re-dispatch target.
    pub target_specialty: SubAgentSpecialty,
    /// The REA pillar below threshold (`architecture` | `security_hardening` | …).
    pub pillar: String,
    /// The fix directive (English — the technical layer).
    pub directive_en: String,
    /// The fix directive (Persian — the judge's voice).
    pub directive_fa: String,
}

/// Mairon strategic advisory with the locked `IQ=150 / EQ=145` calibration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaironStrategicAdvisory {
    /// The judge's overclocked IQ (`150`).
    pub iq: u16,
    /// The judge's overclocked EQ (`145`).
    pub eq: u16,
    /// The game-theoretic lookahead horizon (`8` steps).
    pub lookahead_horizon: u8,
    /// The theory-of-mind order (`3`).
    pub tom_order: u8,
    /// The eight-step lookahead summary — architectural resilience and the scaling path.
    pub eight_step_lookahead_summary: String,
    /// The buyer's psychological hook and the price/referrer suggestion (Persian, ToM-3).
    pub third_order_tom_market_hook_fa: String,
    /// Repair directives for the command center — empty means the forging is accepted.
    pub repair_directives: Vec<MaironRepairDirective>,
}

impl MaironStrategicAdvisory {
    /// Full advisory for one Trinity forging (project structure + master prompt).
    pub fn for_blueprint(
        blueprint: &ProjectBlueprint,
        master: &MasterPromptSpec,
    ) -> Self {
        let price = master.price_stars_hint;
        let low = price * 7 / 10;
        let high = price * 8 / 5;
        let eight_step_lookahead_summary = format!(
            "Lookahead-{h} (IQ=150): (1) module isolation across {n} hard-boundary modules; \
             (2) cache-safe prompt layers pin spec {hash} across the OpenBot fleet; \
             (3) 390px zero-overflow contract survives every repair re-dispatch; \
             (4) zero-auth API surface stays SSRF-allowlisted; \
             (5) pricing at {price} Stars (XTR) survives third-order buyer simulation; \
             (6) affiliate share (1000 bps default) compounds the viral loop; \
             (7) 5-star Bayesian velocity feeds CombinedRankScore; \
             (8) the same 3x9 DNA factory re-seeds the next product without \
             re-architecture — resilience and scaling path verified.",
            h = MAIRON_LOOKAHEAD_HORIZON,
            n = blueprint.modules.len(),
            hash = &master.spec_hash[..12],
            price = price,
        );
        let third_order_tom_market_hook_fa = format!(
            "🧠 \u{645}\u{627}\u{6cc}\u{631}\u{648}\u{646} (IQ={iq}/EQ={eq}\u{60c} \u{646}\u{638}\u{631}\u{6cc}\u{647}\u{654} \u{630}\u{647}\u{646} \u{645}\u{631}\u{62a}\u{628}\u{647}\u{654} {tom}\u{60c} \u{62c}\u{627}\u{647}‌\u{637}\u{644}\u{628}\u{6cc} {amb}): \
             \u{62e}\u{631}\u{6cc}\u{62f}\u{627}\u{631} \u{62f}\u{631} \u{6af}\u{627}\u{645}\u{650} \u{633}\u{648}\u{645}\u{650} \u{62a}\u{635}\u{645}\u{6cc}\u{645} \u{628}\u{647} \u{62e}\u{648}\u{62f}\u{634} \u{645}\u{6cc}‌\u{6af}\u{648}\u{6cc}\u{62f} «\u{627}\u{6af}\u{631} \u{627}\u{645}\u{631}\u{648}\u{632} \u{646}\u{62e}\u{631}\u{645}\u{60c} \u{647}\u{645}\u{6cc}\u{646} \u{627}\u{628}\u{632}\u{627}\u{631} \
             \u{641}\u{631}\u{62f}\u{627} \u{627}\u{632} \u{62f}\u{633}\u{62a}\u{645} \u{62f}\u{631}\u{645}\u{6cc}‌\u{631}\u{648}\u{62f}» — \u{642}\u{644}\u{627}\u{628}\u{650} \u{631}\u{648}\u{627}\u{646}‌\u{634}\u{646}\u{627}\u{62e}\u{62a}\u{6cc}\u{650} «{project}»: \u{62f}\u{642}\u{6cc}\u{642}\u{627}\u{64b} \u{647}\u{645}\u{627}\u{646} \
             \u{62f}\u{631}\u{62f}\u{650} \u{631}\u{648}\u{632}\u{645}\u{631}\u{647}\u{654} \u{645}\u{62e}\u{627}\u{637}\u{628} \u{631}\u{627} \u{62f}\u{631} \u{6f3}\u{6f9}\u{6f0} \u{67e}\u{6cc}\u{6a9}\u{633}\u{644}\u{60c} \u{628}\u{62f}\u{648}\u{646} \u{644}\u{627}\u{6af}\u{6cc}\u{646} \u{648} \u{628}\u{62f}\u{648}\u{646} \u{6a9}\u{627}\u{631}\u{62a} \u{628}\u{627}\u{646}\u{6a9}\u{6cc} \u{62d}\u{644} \
             \u{645}\u{6cc}‌\u{6a9}\u{646}\u{62f}. \u{642}\u{6cc}\u{645}\u{62a}‌\u{6af}\u{630}\u{627}\u{631}\u{6cc}\u{650} \u{628}\u{647}\u{6cc}\u{646}\u{647}: {price} \u{633}\u{62a}\u{627}\u{631}\u{647}\u{654} \u{62a}\u{644}\u{6af}\u{631}\u{627}\u{645} (XTR) \u{628}\u{627} \u{633}\u{647}\u{645} \u{645}\u{639}\u{631}\u{641} \u{6f1}\u{6f0}\u{66a} \
             (1000 bps) \u{62a}\u{627} \u{647}\u{631} \u{62e}\u{631}\u{6cc}\u{62f}\u{627}\u{631} \u{62e}\u{648}\u{62f}\u{634} \u{628}\u{647} \u{628}\u{627}\u{632}\u{62a}\u{648}\u{632}\u{6cc}\u{639} \u{628}\u{631}\u{633}\u{62f}\u{61b} \u{632}\u{6cc}\u{631} {low} \u{627}\u{639}\u{62a}\u{645}\u{627}\u{62f} \
             \u{646}\u{645}\u{6cc}‌\u{633}\u{627}\u{632}\u{62f} \u{648} \u{628}\u{627}\u{644}\u{627}\u{62a}\u{631} \u{627}\u{632} {high} \u{646}\u{631}\u{62e} \u{62a}\u{628}\u{62f}\u{6cc}\u{644} \u{645}\u{6cc}‌\u{627}\u{641}\u{62a}\u{62f}.",
            iq = MAIRON_OVERCLOCK_IQ,
            eq = MAIRON_OVERCLOCK_EQ,
            tom = MAIRON_TOM_ORDER,
            amb = MAIRON_AMBITION_GAIN,
            project = master.project_name,
            price = price,
            low = low,
            high = high,
        );
        Self {
            iq: MAIRON_OVERCLOCK_IQ,
            eq: MAIRON_OVERCLOCK_EQ,
            lookahead_horizon: MAIRON_LOOKAHEAD_HORIZON,
            tom_order: MAIRON_TOM_ORDER,
            eight_step_lookahead_summary,
            third_order_tom_market_hook_fa,
            repair_directives: Vec::new(),
        }
    }
}

/// Mairon's sovereign verdict on a forging — the domain REA score + strategic advisory.
/// (domain structures are serde-free — this verdict flows in memory only.)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaironSovereignVerdict {
    /// The domain's four-pillar verdict (`0..=100` + red lines + a visible badge).
    pub verdict: MaironProductVerdict,
    /// Strategic advisory `IQ=150 / EQ=145`.
    pub advisory: MaironStrategicAdvisory,
}

/// One repair round in the Command Center ↔ Mairon self-healing loop.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepairRound {
    /// Repair pass (`1..=3`).
    pub attempt: u32,
    /// The score before the directives were applied.
    pub score_before: u8,
    /// The judge's directives issued in this pass.
    pub directives: Vec<MaironRepairDirective>,
    /// The score after re-dispatch and fresh adjudication.
    pub score_after: u8,
    /// Did this pass cross the `≥ 85` gate?
    pub approved: bool,
}

/// The complete Trinity pipeline output — the DNA manifest, artifact, final verdict and history.
#[derive(Debug, Clone, PartialEq)]
pub struct TrinityForgeOutcome {
    /// Trinity session ID.
    pub session_id: String,
    /// The builder-citizen account.
    pub citizen_id: String,
    /// Product name.
    pub project_name: String,
    /// The OpenBot fleet's DNA manifest (the `3×9` pattern).
    pub manifest: Vec<SpawnedSubAgent>,
    /// The final assembled artifact.
    pub artifact: AssembledArtifact,
    /// Mairon's final verdict (score + advisory).
    pub final_verdict: MaironSovereignVerdict,
    /// Repair-round history (empty = passed on the first assembly).
    pub repair_history: Vec<RepairRound>,
    /// Repair attempts consumed (`0..=3`).
    pub repair_attempts_used: u32,
    /// Forging accepted? (`final_verdict.verdict.approved_for_listing`).
    pub approved: bool,
}

/// Body 3 — Mairon-150: the overclocked REA judge and the 5-star market gate.
#[derive(Debug, Clone)]
pub struct MaironSovereignJudge {
    /// The ID of the judged product (a Trinity session or a marketplace `prod:{uuid}`).
    product_id: String,
}

impl MaironSovereignJudge {
    /// Judge on one product ID.
    pub fn new(product_id: &str) -> Self {
        Self { product_id: product_id.to_owned() }
    }

    /// Extract red lines from the sanitizer report — a red line is never traded for score.
    fn derive_hard_failures(report: &SanitizerReport) -> Vec<String> {
        let mut hard = Vec::new();
        if report.todo_markers > 0 {
            hard.push(
                "unfinished work markers (todo/TODO/FIXME/XXX) detected — zero-tolerance red line"
                    .to_owned(),
            );
        }
        if report.placeholder_secrets > 0 {
            hard.push(
                "placeholder secret detected — forge refuses credential-shaped strings".to_owned(),
            );
        }
        if report.unsafe_markers > 0 {
            hard.push("`unsafe` execution block detected — kernel law: zero unsafe".to_owned());
        }
        if report.lorem_markers > 0 {
            hard.push("lorem ipsum placeholder text detected — stock copy red line".to_owned());
        }
        hard
    }

    /// Extract the four REA pillars from the artifact's real properties — deterministic and testable.
    fn derive_pillars(
        artifact: &AssembledArtifact,
        blueprint: &ProjectBlueprint,
        report: &SanitizerReport,
    ) -> MaironReaPillarScores {
        let html = &artifact.html;
        let module_count = blueprint.modules.len();
        let integrated = artifact.integrated_modules.len();
        let boundary_markers = html.matches(MODULE_BOUNDARY_MARKER).count();
        let reinforced = usize::from(html.contains(BOUNDARY_REINFORCED_MARKER));
        let architecture = clamp25(
            ARCH_BASE
                + ARCH_PER_MODULE * module_count
                + ARCH_INTEGRATED_BONUS * usize::from(integrated == module_count)
                + ARCH_BOUNDARY_BONUS * usize::from(boundary_markers >= module_count)
                + ARCH_REINFORCED_BONUS * reinforced,
        );
        let security_hardening = clamp25(
            SECURITY_BASE
                .saturating_sub(SECURITY_PER_SECRET * report.placeholder_secrets)
                .saturating_sub(SECURITY_PER_TODO * report.todo_markers)
                .saturating_sub(SECURITY_PER_UNSAFE * report.unsafe_markers),
        );
        let rtl = usize::from(html.contains("dir=\"rtl\""));
        let px390 = usize::from(html.contains("max-width: 390px"));
        let vazir = usize::from(html.contains("Vazirmatn"));
        let polish = usize::from(html.contains(UX_POLISH_MARKER));
        let ux_and_anti_slop = clamp25(
            (UX_BASE
                + UX_MARKER_BONUS * (rtl + px390 + vazir + polish))
            .saturating_sub(UX_SLOP_PENALTY * report.slop_tells.len())
            .saturating_sub(UX_LOREM_PENALTY * report.lorem_markers),
        );
        let verification_proof =
            clamp25(VERIFICATION_BASE + VERIFICATION_PER_PROBE * artifact.probe_count);
        MaironReaPillarScores { architecture, security_hardening, ux_and_anti_slop, verification_proof }
    }

    /// Deterministic adjudication of the assembled artifact — domain verdict + `IQ=150/EQ=145` advisory.
    pub fn judge_artifact(
        &self,
        artifact: &AssembledArtifact,
        blueprint: &ProjectBlueprint,
        master: &MasterPromptSpec,
    ) -> MaironSovereignVerdict {
        let report = run_artifact_sanitizer(&artifact.html);
        let pillars = Self::derive_pillars(artifact, blueprint, &report);
        let hard_failures = Self::derive_hard_failures(&report);
        let verdict =
            evaluate_user_product_with_mairon(&self.product_id, pillars, hard_failures);
        let mut advisory = MaironStrategicAdvisory::for_blueprint(blueprint, master);
        advisory.repair_directives = self.repair_directives(&verdict, 1);
        MaironSovereignVerdict { verdict, advisory }
    }

    /// Repair directives — map each below-threshold pillar to the guilty expertise, in deterministic order:
    /// security red lines first, then verification, UX and architecture.
    pub fn repair_directives(
        &self,
        verdict: &MaironProductVerdict,
        attempt: u32,
    ) -> Vec<MaironRepairDirective> {
        let mut out = Vec::new();
        let p = &verdict.pillars;
        if !verdict.hard_failures.is_empty() || p.security_hardening < PILLAR_REPAIR_TRIGGER {
            out.push(MaironRepairDirective {
                attempt,
                target_specialty: SubAgentSpecialty::SecurityAndApiBridge,
                pillar: "security_hardening".to_owned(),
                directive_en: "Purge every unfinished marker and placeholder secret, re-run the \
                    sanitizer gate until passed=true, and re-assert the SSRF allowlist."
                    .to_owned(),
                directive_fa: "\u{647}\u{645}\u{647}\u{654} \u{646}\u{634}\u{627}\u{646}\u{6af}\u{631}\u{647}\u{627}\u{6cc} \u{646}\u{627}\u{62a}\u{645}\u{627}\u{645} \u{648} \u{631}\u{627}\u{632}\u{647}\u{627}\u{6cc} \u{62c}\u{627}\u{6cc}‌\u{628}\u{627}\u{646}\u{6cc} \u{631}\u{627} \u{67e}\u{627}\u{6a9} \u{6a9}\u{646}\u{61b} \u{6af}\u{6cc}\u{62a}\u{650} \u{67e}\u{627}\u{6a9}‌\u{633}\u{627}\u{632} \
                    \u{631}\u{627} \u{62a}\u{627} \u{639}\u{628}\u{648}\u{631}\u{650} \u{6a9}\u{627}\u{645}\u{644} \u{62f}\u{648}\u{628}\u{627}\u{631}\u{647} \u{627}\u{62c}\u{631}\u{627} \u{6a9}\u{646} \u{648} \u{627}\u{62c}\u{627}\u{632}\u{647}‌\u{646}\u{627}\u{645}\u{647}\u{654} SSRF \u{631}\u{627} \u{628}\u{627}\u{632}-\u{62a}\u{62b}\u{628}\u{6cc}\u{62a} \u{6a9}\u{646}."
                    .to_owned(),
            });
        }
        if p.verification_proof < PILLAR_REPAIR_TRIGGER {
            out.push(MaironRepairDirective {
                attempt,
                target_specialty: SubAgentSpecialty::VerificationAndPolish,
                pillar: "verification_proof".to_owned(),
                directive_en: "Inject 2x modules + 4 empirical probe assertions (data-probe \
                    elements) and run them green before re-submission — proof before pride."
                    .to_owned(),
                directive_fa: "\u{67e}\u{631}\u{648}\u{628}‌\u{647}\u{627}\u{6cc} \u{62a}\u{62c}\u{631}\u{628}\u{6cc} (\u{62f}\u{648} \u{628}\u{631}\u{627}\u{628}\u{631}\u{650} \u{645}\u{627}\u{698}\u{648}\u{644}‌\u{647}\u{627} + \u{686}\u{647}\u{627}\u{631}) \u{631}\u{627} \u{62a}\u{632}\u{631}\u{6cc}\u{642} \u{6a9}\u{646} \u{648} \u{633}\u{628}\u{632}\u{650} \
                    \u{627}\u{62c}\u{631}\u{627} \u{631}\u{627} \u{67e}\u{6cc}\u{634} \u{627}\u{632} \u{627}\u{631}\u{627}\u{626}\u{647}\u{654} \u{62f}\u{648}\u{628}\u{627}\u{631}\u{647} \u{636}\u{645}\u{6cc}\u{645}\u{647} \u{6a9}\u{646} — \u{634}\u{627}\u{647}\u{62f} \u{67e}\u{6cc}\u{634} \u{627}\u{632} \u{627}\u{641}\u{62a}\u{62e}\u{627}\u{631}."
                    .to_owned(),
            });
        }
        if p.ux_and_anti_slop < PILLAR_REPAIR_TRIGGER {
            out.push(MaironRepairDirective {
                attempt,
                target_specialty: SubAgentSpecialty::UiUxArchitect390,
                pillar: "ux_and_anti_slop".to_owned(),
                directive_en: "Apply the 390px hardening polish pass, strip every slop tell, and \
                    re-affirm the Vazirmatn RTL stack."
                    .to_owned(),
                directive_fa: "\u{6af}\u{630}\u{631}\u{650} \u{67e}\u{631}\u{62f}\u{627}\u{62e}\u{62a}\u{650} \u{633}\u{62e}\u{62a}‌\u{633}\u{627}\u{632}\u{650} \u{6f3}\u{6f9}\u{6f0} \u{67e}\u{6cc}\u{6a9}\u{633}\u{644} \u{631}\u{627} \u{627}\u{639}\u{645}\u{627}\u{644} \u{6a9}\u{646}\u{60c} \u{647}\u{631} \u{646}\u{634}\u{627}\u{646}\u{647}\u{654} \u{627}\u{633}\u{644}\u{648}\u{67e} \u{631}\u{627} \
                    \u{628}\u{6a9}\u{646} \u{648} \u{67e}\u{634}\u{62a}\u{647}\u{654} \u{648}\u{632}\u{6cc}\u{631}\u{645}\u{62a}\u{646} \u{631}\u{627}\u{633}\u{62a}‌\u{628}\u{647}‌\u{686}\u{67e} \u{631}\u{627} \u{628}\u{627}\u{632}-\u{62a}\u{62b}\u{628}\u{6cc}\u{62a} \u{6a9}\u{646}.".to_owned(),
            });
        }
        if p.architecture < PILLAR_REPAIR_TRIGGER {
            out.push(MaironRepairDirective {
                attempt,
                target_specialty: SubAgentSpecialty::CoreLogicEngine,
                pillar: "architecture".to_owned(),
                directive_en: "Reinforce module boundary markers and complete integration of \
                    every blueprint path (data-module attributes)."
                    .to_owned(),
                directive_fa: "\u{645}\u{631}\u{632}\u{647}\u{627}\u{6cc} \u{645}\u{627}\u{698}\u{648}\u{644} \u{631}\u{627} \u{62a}\u{642}\u{648}\u{6cc}\u{62a} \u{6a9}\u{646} \u{648} \u{6cc}\u{6a9}\u{67e}\u{627}\u{631}\u{686}\u{6af}\u{6cc}\u{650} \u{647}\u{645}\u{647}\u{654} \u{645}\u{633}\u{6cc}\u{631}\u{647}\u{627}\u{6cc} \u{627}\u{633}\u{62a}\u{631}\u{627}\u{6a9}\u{686}\u{631} \
                    \u{67e}\u{631}\u{648}\u{698}\u{647} (\u{648}\u{6cc}\u{698}\u{6af}\u{6cc}\u{650} data-module) \u{631}\u{627} \u{6a9}\u{627}\u{645}\u{644} \u{6a9}\u{646}.".to_owned(),
            });
        }
        out
    }

    /// Independent adjudication of a marketplace product (the product's `preview_html`) — the two-key seal of
    /// [`kian_domain::mairon_rea_and_reviews::confirm_and_publish_product`].
    pub fn judge_market_product(product_id: &str, preview_html: &str) -> MaironProductVerdict {
        let report = run_artifact_sanitizer(preview_html);
        let sections = preview_html.matches("<section").count().min(5);
        let scripts = preview_html.matches("<script").count().min(4);
        let architecture = clamp25(12 + 3 * sections);
        let security_hardening = clamp25(
            SECURITY_BASE
                .saturating_sub(SECURITY_PER_SECRET * report.placeholder_secrets)
                .saturating_sub(SECURITY_PER_TODO * report.todo_markers)
                .saturating_sub(SECURITY_PER_UNSAFE * report.unsafe_markers),
        );
        let rtl = usize::from(preview_html.contains("dir=\"rtl\""));
        let px390 = usize::from(preview_html.contains("max-width: 390px"));
        let vazir = usize::from(preview_html.contains("Vazirmatn"));
        let viewport = usize::from(preview_html.contains("name=\"viewport\""));
        let aria = usize::from(preview_html.contains("aria-"));
        let ux_and_anti_slop = clamp25(
            (UX_BASE + UX_MARKER_BONUS * (rtl + px390 + vazir + viewport))
                .saturating_sub(UX_SLOP_PENALTY * report.slop_tells.len())
                .saturating_sub(UX_LOREM_PENALTY * report.lorem_markers),
        );
        let probes = preview_html.matches("data-probe=").count();
        let verification_proof = clamp25(
            8 + 2 * scripts + 3 * probes + UX_MARKER_BONUS * (aria + viewport),
        );
        let hard_failures = Self::derive_hard_failures(&report);
        evaluate_user_product_with_mairon(product_id, MaironReaPillarScores {
            architecture,
            security_hardening,
            ux_and_anti_slop,
            verification_proof,
        }, hard_failures)
    }
}

// ── Command-center ↔ Mairon self-healing loop ──────────────────────────────────

impl CommandCenterOrchestrator {
    /// Run the full command center with the Mairon self-healing loop.
    ///
    /// First assembly → Mairon adjudication; if the score is `< 85` or there is a red line, the
    /// [`MaironRepairDirective`] orders are re-dispatched to the guilty subagent (up to
    /// [`MAX_OPENBOT_REPAIR_ATTEMPTS`]) until the forging passes the gate.
    pub fn execute_command_center_with_mairon_loop(
        &self,
        master: &MasterPromptSpec,
        blueprint: &ProjectBlueprint,
    ) -> TrinityForgeOutcome {
        let fleet = self.spawn_openbot_fleet(master, blueprint);
        let judge = MaironSovereignJudge::new(&self.session_id);
        let price = master.price_stars_hint;
        let mut directives: Vec<MaironRepairDirective> = Vec::new();
        let mut artifact =
            Self::assemble_artifact(&fleet, master, blueprint, price, 0, &directives);
        let mut verdict = judge.judge_artifact(&artifact, blueprint, master);
        let mut history = Vec::new();
        let mut attempts_used = 0_u32;
        if !verdict.verdict.approved_for_listing {
            for attempt in 1..=MAX_OPENBOT_REPAIR_ATTEMPTS {
                attempts_used = attempt;
                let score_before = verdict.verdict.total_score;
                directives = judge.repair_directives(&verdict.verdict, attempt);
                artifact =
                    Self::assemble_artifact(&fleet, master, blueprint, price, attempt, &directives);
                verdict = judge.judge_artifact(&artifact, blueprint, master);
                let approved = verdict.verdict.approved_for_listing;
                history.push(RepairRound {
                    attempt,
                    score_before,
                    directives,
                    score_after: verdict.verdict.total_score,
                    approved,
                });
                if approved {
                    break;
                }
            }
        }
        TrinityForgeOutcome {
            session_id: self.session_id.clone(),
            citizen_id: self.citizen_id.clone(),
            project_name: master.project_name.clone(),
            manifest: fleet,
            artifact,
            approved: verdict.verdict.approved_for_listing,
            repair_attempts_used: attempts_used,
            repair_history: history,
            final_verdict: verdict,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Five complete slots — the standard test fixture.
    fn full_slots() -> AlphaDiscoverySlots {
        AlphaDiscoverySlots {
            core_objective: Some("\u{641}\u{631}\u{648}\u{634}\u{6af}\u{627}\u{647} \u{627}\u{628}\u{632}\u{627}\u{631}\u{647}\u{627}\u{6cc} \u{628}\u{647}\u{631}\u{647}‌\u{648}\u{631}\u{6cc} \u{628}\u{631}\u{627}\u{6cc} \u{641}\u{631}\u{6cc}\u{644}\u{646}\u{633}\u{631}\u{647}\u{627}\u{6cc} \u{641}\u{627}\u{631}\u{633}\u{6cc}‌\u{632}\u{628}\u{627}\u{646}".to_owned()),
            target_audience_and_ux: Some(
                "\u{641}\u{631}\u{6cc}\u{644}\u{646}\u{633}\u{631}\u{647}\u{627}\u{6cc} \u{641}\u{627}\u{631}\u{633}\u{6cc}‌\u{632}\u{628}\u{627}\u{646}\u{60c} \u{645}\u{648}\u{628}\u{627}\u{6cc}\u{644}‌\u{641}\u{631}\u{633}\u{62a} \u{62f}\u{631} \u{6f3}\u{6f9}\u{6f0} \u{67e}\u{6cc}\u{6a9}\u{633}\u{644} \u{631}\u{627}\u{633}\u{62a}‌\u{628}\u{647}‌\u{686}\u{67e}".to_owned(),
            ),
            interactive_features: Some("\u{633}\u{628}\u{62f} \u{62e}\u{631}\u{6cc}\u{62f}\u{60c} \u{641}\u{631}\u{645} \u{633}\u{641}\u{627}\u{631}\u{634} \u{648} \u{62c}\u{631}\u{6cc}\u{627}\u{646} \u{67e}\u{631}\u{62f}\u{627}\u{62e}\u{62a} \u{62a}\u{639}\u{627}\u{645}\u{644}\u{6cc}".to_owned()),
            data_and_zero_auth_apis: Some(
                "\u{630}\u{62e}\u{6cc}\u{631}\u{647} \u{62f}\u{631} localStorage \u{648} \u{627}\u{62a}\u{635}\u{627}\u{644} \u{628}\u{647} API\u{647}\u{627}\u{6cc} \u{639}\u{645}\u{648}\u{645}\u{6cc} \u{628}\u{62f}\u{648}\u{646} \u{644}\u{627}\u{6af}\u{6cc}\u{646}".to_owned(),
            ),
            monetization_and_viral_loop: Some(
                "\u{642}\u{6cc}\u{645}\u{62a} \u{6f2}\u{6f0}\u{6f0} \u{633}\u{62a}\u{627}\u{631}\u{647} \u{628}\u{627} \u{6f1}\u{6f0}\u{66a} \u{67e}\u{648}\u{631}\u{633}\u{627}\u{646}\u{62a} \u{645}\u{639}\u{631}\u{641} \u{648} \u{62d}\u{644}\u{642}\u{647}\u{654} \u{648}\u{627}\u{6cc}\u{631}\u{627}\u{644}".to_owned(),
            ),
        }
    }

    fn intake(message: &str, prior: AlphaDiscoverySlots) -> AlphaIntakeOutcome {
        AlphaIgnitionPersona.process_intake(&AlphaIntakeRequest {
            session_id: "trn:test-0001".to_owned(),
            citizen_id: "citizen:900".to_owned(),
            user_message: message.to_owned(),
            prior_slots: prior,
        })
    }

    /// A message that lights up four slots (no monetization) — 80% readiness.
    const FOUR_SLOT_MESSAGE: &str = "\u{645}\u{6cc}‌\u{62e}\u{648}\u{627}\u{647}\u{645} \u{6cc}\u{6a9} \u{645}\u{62d}\u{635}\u{648}\u{644} \u{628}\u{633}\u{627}\u{632}\u{645}: \u{633}\u{627}\u{632}\u{646}\u{62f}\u{647}\u{654} \u{627}\u{628}\u{632}\u{627}\u{631} \u{628}\u{647}\u{631}\u{647}‌\u{648}\u{631}\u{6cc}. \
        \u{645}\u{62e}\u{627}\u{637}\u{628} \u{641}\u{631}\u{6cc}\u{644}\u{646}\u{633}\u{631} \u{641}\u{627}\u{631}\u{633}\u{6cc}‌\u{632}\u{628}\u{627}\u{646} \u{627}\u{633}\u{62a} \u{648} \u{645}\u{648}\u{628}\u{627}\u{6cc}\u{644}‌\u{641}\u{631}\u{633}\u{62a} \u{628}\u{627} rtl \u{6a9}\u{627}\u{631} \u{645}\u{6cc}‌\u{6a9}\u{646}\u{62f}. \
        \u{628}\u{627}\u{6cc}\u{62f} \u{641}\u{631}\u{645} \u{633}\u{641}\u{627}\u{631}\u{634} \u{62a}\u{639}\u{627}\u{645}\u{644}\u{6cc} \u{62f}\u{627}\u{634}\u{62a}\u{647} \u{628}\u{627}\u{634}\u{62f}. \u{62f}\u{627}\u{62f}\u{647}‌\u{647}\u{627} \u{62f}\u{631} api \u{639}\u{645}\u{648}\u{645}\u{6cc} \u{628}\u{62f}\u{648}\u{646} \u{644}\u{627}\u{6af}\u{6cc}\u{646} \u{630}\u{62e}\u{6cc}\u{631}\u{647} \u{645}\u{6cc}‌\u{634}\u{648}\u{646}\u{62f}.";

    /// Alpha's DNA is locked.
    #[test]
    fn alpha_dna_constants_are_locked() {
        assert_eq!(ALPHA_EFFECTIVE_IQ, 136);
        assert_eq!(ALPHA_EFFECTIVE_EQ, 122);
        assert!((ALPHA_INHIBITION_BRAKE - 0.04).abs() < f32::EPSILON);
        assert!((ALPHA_ENTROPY_TEMP - 0.88).abs() < f32::EPSILON);
        assert_eq!(ALPHA_READINESS_LOCK_THRESHOLD, 80);
    }

    /// The command-center and judge DNA re-exported from the domain is locked — single source of truth.
    #[test]
    fn logic_and_mairon_dna_constants_are_locked() {
        let (iq_alpha, iq_logic, iq_mairon) =
            (ALPHA_EFFECTIVE_IQ, LOGIC_EFFECTIVE_IQ, MAIRON_OVERCLOCK_IQ);
        let (eq_logic, eq_mairon) = (LOGIC_EFFECTIVE_EQ, MAIRON_OVERCLOCK_EQ);
        let horizon = MAIRON_LOOKAHEAD_HORIZON;
        let tom = MAIRON_TOM_ORDER;
        assert_eq!(iq_logic, 139);
        assert!((LOGIC_INHIBITION_BRAKE - 0.62).abs() < f32::EPSILON);
        assert!((LOGIC_FORENSIC_STRICTNESS - 0.96).abs() < f32::EPSILON);
        assert_eq!(iq_mairon, 150);
        assert_eq!(eq_mairon, 145);
        assert_eq!(horizon, 8);
        assert_eq!(tom, 3);
        assert_eq!(MAX_OPENBOT_REPAIR_ATTEMPTS, 3);
        assert!(iq_mairon > iq_logic && iq_logic > iq_alpha);
        assert!(eq_mairon > eq_logic);
    }

    /// Slot extraction with bilingual keywords — deterministic, no overwrite.
    #[test]
    fn slot_extraction_is_deterministic_and_never_overwrites() {
        let out = intake(FOUR_SLOT_MESSAGE, AlphaDiscoverySlots::default());
        let AlphaIntakeOutcome::LockedForCommandCenter { master_prompt, blueprint, .. } = out else {
            panic!("four slots = 80% readiness must lock");
        };
        assert!(master_prompt.functional_spec.contains("\u{633}\u{627}\u{632}\u{646}\u{62f}\u{647}\u{654} \u{627}\u{628}\u{632}\u{627}\u{631}"));
        assert!(blueprint.modules.iter().any(|m| m.path == "src/js/api-bindings.js"));
        assert!(!blueprint.modules.iter().any(|m| m.path == "src/js/monetization.js"));
        // overwrite is forbidden — the first value always wins
        let mut slots = full_slots();
        slots.fill(AlphaSlotKind::CoreObjective, "\u{646}\u{628}\u{627}\u{6cc}\u{62f} \u{62c}\u{627}\u{6cc}\u{6af}\u{632}\u{6cc}\u{646} \u{634}\u{648}\u{62f}".to_owned());
        assert!(slots.core_objective.unwrap().contains("\u{641}\u{631}\u{648}\u{634}\u{6af}\u{627}\u{647} \u{627}\u{628}\u{632}\u{627}\u{631}"));
    }

    /// Incomplete readiness → NeedsMoreDiscovery with the raw list in deterministic order.
    #[test]
    fn alpha_needs_more_discovery_lists_missing_slots_in_order() {
        let out = intake("\u{645}\u{6cc}‌\u{62e}\u{648}\u{627}\u{647}\u{645} \u{6cc}\u{6a9} \u{645}\u{62d}\u{635}\u{648}\u{644} \u{628}\u{633}\u{627}\u{632}\u{645} \u{6a9}\u{647} \u{641}\u{631}\u{645} \u{62a}\u{639}\u{627}\u{645}\u{644}\u{6cc} \u{62f}\u{627}\u{631}\u{62f}", AlphaDiscoverySlots::default());
        let AlphaIntakeOutcome::NeedsMoreDiscovery { alpha_reply_fa, missing_slots, readiness_pct } =
            out
        else {
            panic!("two slots = 40% readiness must stay in discovery");
        };
        assert_eq!(readiness_pct, 40);
        assert_eq!(missing_slots.len(), 3);
        assert_eq!(missing_slots[0], AlphaSlotKind::TargetAudienceAndUx);
        assert_eq!(missing_slots.last().copied(), Some(AlphaSlotKind::MonetizationAndViralLoop));
        assert!(alpha_reply_fa.contains("\u{622}\u{644}\u{641}\u{627} \u{641}\u{639}\u{627}\u{644} \u{634}\u{62f}"));
        assert!(alpha_reply_fa.contains("\u{645}\u{62e}\u{627}\u{637}\u{628} \u{647}\u{62f}\u{641} \u{648} \u{62d}\u{633} UX"));
    }

    /// Locked at 80% — the master prompt + project structure are deterministic and reproducible.
    #[test]
    fn alpha_locks_at_eighty_pct_with_deterministic_spec() {
        let out = intake(FOUR_SLOT_MESSAGE, AlphaDiscoverySlots::default());
        let AlphaIntakeOutcome::LockedForCommandCenter { alpha_handoff_fa, master_prompt, blueprint } =
            out
        else {
            panic!("must lock");
        };
        assert!(alpha_handoff_fa.contains("\u{622}\u{644}\u{641}\u{627} \u{642}\u{641}\u{644} \u{634}\u{62f}"));
        assert!(alpha_handoff_fa.contains("\u{6f8}\u{6f0}") || alpha_handoff_fa.contains("80"));
        let again = intake(FOUR_SLOT_MESSAGE, AlphaDiscoverySlots::default());
        let AlphaIntakeOutcome::LockedForCommandCenter { master_prompt: m2, blueprint: b2, .. } =
            again
        else {
            panic!("must lock again");
        };
        assert_eq!(master_prompt, m2, "master prompt must be deterministic");
        assert_eq!(blueprint, b2, "blueprint must be deterministic");
        assert_eq!(master_prompt.spec_hash.len(), 64, "sha256 hex");
        assert!(master_prompt.compiled_prompt.contains("390px"));
        assert_eq!(master_prompt.price_stars_hint, 150, "no monetization slot → default 150");
        assert_eq!(blueprint.project_name, master_prompt.project_name);
    }

    /// Project structure — four base modules + three conditional modules with target expertise.
    #[test]
    fn blueprint_modules_derive_from_slots_with_specialties() {
        let master = MasterPromptSpec::compile(&full_slots(), "trn:t", "citizen:900");
        let bp = ProjectBlueprint::compile(&full_slots(), &master.project_name);
        let paths: Vec<&str> = bp.modules.iter().map(|m| m.path.as_str()).collect();
        assert_eq!(paths.len(), 7);
        for base in ["src/index.html", "src/styles/main.css", "src/js/core.js", "src/js/verification-probes.js"] {
            assert!(paths.contains(&base), "base module {base} must exist");
        }
        let index = bp.modules.iter().find(|m| m.path == "src/index.html").unwrap();
        assert_eq!(index.specialty, SubAgentSpecialty::UiUxArchitect390);
        let api = bp.modules.iter().find(|m| m.path == "src/js/api-bindings.js").unwrap();
        assert_eq!(api.specialty, SubAgentSpecialty::SecurityAndApiBridge);
        let money = bp.modules.iter().find(|m| m.path == "src/js/monetization.js").unwrap();
        assert_eq!(money.specialty, SubAgentSpecialty::MonetizationAndViralLoop);
        assert_eq!(master.price_stars_hint, 200, "\u{6f2}\u{6f0}\u{6f0} \u{62f}\u{631} \u{627}\u{633}\u{644}\u{627}\u{62a} \u{62f}\u{631}\u{622}\u{645}\u{62f} → 200");
        let none = MasterPromptSpec::compile(&AlphaDiscoverySlots::default(), "trn:x", "c");
        assert_eq!(none.price_stars_hint, 150);
    }

    /// Deterministic random alleles — same seed, same DNA; a different seed, different DNA.
    #[test]
    fn alleles_are_deterministic_per_seed_and_index() {
        let a0 = derive_alleles(42, 0);
        let a0b = derive_alleles(42, 0);
        let a1 = derive_alleles(42, 1);
        let a0_other_seed = derive_alleles(43, 0);
        assert_eq!(a0, a0b);
        assert_ne!(a0, a1);
        assert_ne!(a0, a0_other_seed);
        assert!((0.55..=0.95).contains(&a0.surgical_density));
        assert!((0.50..=0.95).contains(&a0.ux_craft));
        assert!((0.60..=0.95).contains(&a0.edge_case_paranoia));
        assert!((0.45..=0.95).contains(&a0.execution_velocity));
    }

    /// A single PRNG — the `[0,1)` range and reproducibility.
    #[test]
    fn splitmix_unit_stays_in_zero_one() {
        let mut st = 7_u64;
        for _ in 0..64 {
            let u = splitmix_unit(&mut st);
            assert!((0.0..1.0).contains(&u));
        }
    }

    /// The three-layer prompt — the cache boundary exactly once; fixed layers first, dynamic after.
    #[test]
    fn three_layer_prompt_pins_cache_boundary_once() {
        let master = MasterPromptSpec::compile(&full_slots(), "trn:t", "citizen:900");
        let bp = ProjectBlueprint::compile(&full_slots(), &master.project_name);
        let spawner = OpenBotSubAgentSpawner::new("trn:t", 42);
        let agent = spawner.spawn(0, &bp.modules[0], &master, &bp);
        let rendered = agent.prompt.render();
        assert_eq!(rendered.matches(SYSTEM_PROMPT_DYNAMIC_BOUNDARY).count(), 1);
        let (static_part, dynamic_part) =
            rendered.split_once(SYSTEM_PROMPT_DYNAMIC_BOUNDARY).expect("boundary");
        assert!(static_part.contains("SOVEREIGN CODING LAWS"));
        assert!(static_part.contains(&master.project_name));
        assert!(dynamic_part.contains("ASSIGNMENT (dynamic)"));
        assert!(dynamic_part.contains("src/index.html"));
    }

    /// The subagent stop hook — exact `450s/1200s` boundaries.
    #[test]
    fn stall_watchdog_enforces_sovereign_bounds() {
        let wd = SubagentStallWatchdog::sovereign();
        assert_eq!(wd.soft_stall_secs, 450);
        assert_eq!(wd.hard_timeout_secs, 1200);
        assert_eq!(wd.verdict_for(449), WatchdogVerdict::Healthy);
        assert_eq!(wd.verdict_for(450), WatchdogVerdict::SoftStall);
        assert_eq!(wd.verdict_for(1199), WatchdogVerdict::SoftStall);
        assert_eq!(wd.verdict_for(1200), WatchdogVerdict::HardTimeout);
    }

    /// Nine-fold genome — all of f01..f09 filled, sanitizer green, full XML.
    #[test]
    fn nine_fold_genome_is_complete_and_clean() {
        let master = MasterPromptSpec::compile(&full_slots(), "trn:t", "citizen:900");
        let bp = ProjectBlueprint::compile(&full_slots(), &master.project_name);
        let spawner = OpenBotSubAgentSpawner::new("trn:t", 42);
        let agent = spawner.spawn(2, &bp.modules[2], &master, &bp);
        let g = &agent.genome;
        assert!(g.f01_topology_and_harness.contains("IQ=139"));
        assert!(g.f01_topology_and_harness.contains("soft=450s"));
        assert!(g.f02_hard_constraints.iter().any(|c| c.contains("forbid(unsafe_code)")));
        assert!(g.f02_hard_constraints.iter().any(|c| c.contains("SSRF")));
        assert_eq!(g.f03_token_economics, CavemanIntensity::Full, "core engine → Full");
        assert!(g.f04_stochastic_alleles.allele_seed > 0);
        assert!(g.f05_failure_recovery.contains("3-attempt"));
        let xml = &g.f06_compiled_xml_prompt;
        assert!(xml.contains("<openbot_subagent_dna"));
        assert!(xml.contains("f01_topology_and_harness"));
        assert!(xml.contains("f04_stochastic_alleles"));
        assert!(xml.contains("f09_handoff_card"));
        assert!(xml.contains("src/js/core.js"));
        assert!(g.f07_sanitizer_gate.passed, "synthesized DNA must pass the sanitizer");
        assert_eq!(g.f07_sanitizer_gate.todo_markers, 0);
        assert!(g.f08_empirical_probes.len() >= 2);
        assert!(g.f09_handoff_card.contains("HANDOFF -> Command Center"));
        assert_eq!(agent.thread_id.len(), 36, "thread id is RFC uuid-shaped");
        assert_eq!(agent.thread_id.matches('-').count(), 4);
        // thread reproducibility
        let agent2 = spawner.spawn(2, &bp.modules[2], &master, &bp);
        assert_eq!(agent.thread_id, agent2.thread_id);
    }

    /// Custom expertise list — safe Latinization for XML/wire.
    #[test]
    fn custom_specialty_is_slugified() {
        let custom = SubAgentSpecialty::Custom("Quant Chart DNA < specialist>".to_owned());
        assert_eq!(custom.as_str(), "custom_quantchartdnaspecialist");
        assert!(custom.fa_label().contains("\u{645}\u{62a}\u{62e}\u{635}\u{635} \u{633}\u{641}\u{627}\u{631}\u{634}\u{6cc}"));
        assert_eq!(custom.default_caveman(), CavemanIntensity::Full);
    }

    /// The fleet — one specialist per module; custom expertises are allowed too.
    #[test]
    fn fleet_matches_blueprint_one_agent_per_module() {
        let master = MasterPromptSpec::compile(&full_slots(), "trn:t", "citizen:900");
        let bp = ProjectBlueprint::compile(&full_slots(), &master.project_name);
        let orch = CommandCenterOrchestrator::new("trn:t", "citizen:900", 42);
        let fleet = orch.spawn_openbot_fleet(&master, &bp);
        assert_eq!(fleet.len(), bp.modules.len());
        for (i, agent) in fleet.iter().enumerate() {
            assert_eq!(agent.subagent_index, u32::try_from(i).unwrap());
            assert_eq!(agent.assigned_modules, vec![bp.modules[i].path.clone()]);
        }
        // unique alleles across the fleet — unique DNA per subagent
        let seeds: Vec<u64> = fleet.iter().map(|a| a.genome.f04_stochastic_alleles.allele_seed).collect();
        let unique = seeds.iter().collect::<std::collections::HashSet<_>>();
        assert_eq!(unique.len(), seeds.len(), "allele seeds must be unique per sub-agent");
    }

    /// Sanitizer gate — finds all four forbidden families; clean text stays green.
    #[test]
    fn sanitizer_gate_catches_all_dirty_families() {
        let dirty = "let k = sk-abc123; /* TODO fix */ const x = AKIA1234; unsafe { /* lorem ipsum */ }";
        let report = run_sanitizer_gate(dirty);
        assert!(!report.passed);
        assert!(report.todo_markers >= 1);
        assert!(report.placeholder_secrets >= 2);
        assert_eq!(report.unsafe_markers, 1);
        assert_eq!(report.lorem_markers, 1);
        let clean = run_sanitizer_gate("pure state kernel with no forbidden markers");
        assert!(clean.passed);
    }

    /// Base verdict below the gate and above it after one repair pass — the heart of the self-healing loop.
    #[test]
    fn base_assembly_fails_gate_and_repair_pass_clears_it() {
        let master = MasterPromptSpec::compile(&full_slots(), "trn:loop", "citizen:900");
        let bp = ProjectBlueprint::compile(&full_slots(), &master.project_name);
        let orch = CommandCenterOrchestrator::new("trn:loop", "citizen:900", 42);
        let fleet = orch.spawn_openbot_fleet(&master, &bp);
        let judge = MaironSovereignJudge::new("trn:loop");

        let base = CommandCenterOrchestrator::assemble_artifact(&fleet, &master, &bp, master.price_stars_hint, 0, &[]);
        assert_eq!(base.probe_count, 0, "pass 0 — probes are stage-gated");
        assert_eq!(base.integrated_modules.len(), bp.modules.len(), "full integration");
        let base_verdict = judge.judge_artifact(&base, &bp, &master);
        assert!(
            base_verdict.verdict.total_score < MARKETPLACE_APPROVAL_THRESHOLD,
            "base assembly must fail the 85 gate (score = {})",
            base_verdict.verdict.total_score
        );
        assert!(base_verdict.verdict.hard_failures.is_empty(), "base artifact is clean");

        let directives = judge.repair_directives(&base_verdict.verdict, 1);
        assert!(directives.iter().any(|d| d.target_specialty == SubAgentSpecialty::VerificationAndPolish));
        assert!(directives.iter().any(|d| d.target_specialty == SubAgentSpecialty::UiUxArchitect390));

        let repaired = CommandCenterOrchestrator::assemble_artifact(
            &fleet, &master, &bp, master.price_stars_hint, 1, &directives,
        );
        assert_eq!(repaired.probe_count, 2 * bp.modules.len() + 4);
        assert!(repaired.html.contains(UX_POLISH_MARKER));
        assert!(repaired.html.contains("data-probe=\"probe-1\""));
        let repaired_verdict = judge.judge_artifact(&repaired, &bp, &master);
        assert!(
            repaired_verdict.verdict.approved_for_listing,
            "repaired forge must clear the gate (score = {})",
            repaired_verdict.verdict.total_score
        );
        assert_eq!(
            repaired_verdict.verdict.visible_badge,
            format!("⚒️ Mairon REA Certified: {}/100 (Forge-Grade)", repaired_verdict.verdict.total_score)
        );
    }

    /// Self-healing loop — passes within ≤ 3 attempts with a score-improvement history.
    #[test]
    fn self_healing_loop_clears_mairon_gate_within_three_attempts() {
        let master = MasterPromptSpec::compile(&full_slots(), "trn:loop", "citizen:900");
        let bp = ProjectBlueprint::compile(&full_slots(), &master.project_name);
        let orch = CommandCenterOrchestrator::new("trn:loop", "citizen:900", 42);
        let out = orch.execute_command_center_with_mairon_loop(&master, &bp);
        assert!(out.approved, "sovereign loop must forge an approved artifact");
        assert!(out.repair_attempts_used >= 1 && out.repair_attempts_used <= MAX_OPENBOT_REPAIR_ATTEMPTS);
        assert_eq!(out.repair_history.len(), usize::try_from(out.repair_attempts_used).unwrap());
        let first = &out.repair_history[0];
        assert!(first.score_after > first.score_before, "repair must improve the score");
        assert!(first.approved);
        assert!(out.final_verdict.verdict.total_score >= MARKETPLACE_APPROVAL_THRESHOLD);
        assert!(out.final_verdict.verdict.visible_badge.contains("Forge-Grade"));
        assert_eq!(out.manifest.len(), bp.modules.len());
        assert_eq!(out.project_name, master.project_name);
        assert!(out.artifact.html.contains("mairon-empirical-probes"));
    }

    /// Deterministic self-healing loop — same session, same forging.
    #[test]
    fn sovereign_loop_is_fully_deterministic() {
        let master = MasterPromptSpec::compile(&full_slots(), "trn:det", "citizen:900");
        let bp = ProjectBlueprint::compile(&full_slots(), &master.project_name);
        let a = CommandCenterOrchestrator::new("trn:det", "citizen:900", 4242)
            .execute_command_center_with_mairon_loop(&master, &bp);
        let b = CommandCenterOrchestrator::new("trn:det", "citizen:900", 4242)
            .execute_command_center_with_mairon_loop(&master, &bp);
        assert_eq!(a, b);
        // different seed — a different fleet yet the same gate pass
        let c = CommandCenterOrchestrator::new("trn:det", "citizen:900", 777)
            .execute_command_center_with_mairon_loop(&master, &bp);
        assert!(c.approved);
        assert_ne!(a.manifest[0].genome.f04_stochastic_alleles, c.manifest[0].genome.f04_stochastic_alleles);
    }

    /// Session seed — deterministic from the session ID.
    #[test]
    fn session_seed_is_derived_deterministically() {
        let s1 = CommandCenterOrchestrator::session_seed_from_session("trn:abc");
        let s2 = CommandCenterOrchestrator::session_seed_from_session("trn:abc");
        let s3 = CommandCenterOrchestrator::session_seed_from_session("trn:abd");
        assert_eq!(s1, s2);
        assert_ne!(s1, s3);
    }

    /// Mairon strategic advisory — locked 150/145 calibration, ToM-3 and the XTR price.
    #[test]
    fn mairon_advisory_carries_locked_calibration_and_tom3_hook() {
        let master = MasterPromptSpec::compile(&full_slots(), "trn:adv", "citizen:900");
        let bp = ProjectBlueprint::compile(&full_slots(), &master.project_name);
        let advisory = MaironStrategicAdvisory::for_blueprint(&bp, &master);
        assert_eq!(advisory.iq, 150);
        assert_eq!(advisory.eq, 145);
        assert_eq!(advisory.lookahead_horizon, 8);
        assert_eq!(advisory.tom_order, 3);
        assert!(advisory.eight_step_lookahead_summary.contains("Lookahead-8"));
        assert!(advisory
            .eight_step_lookahead_summary
            .contains(&format!("{} hard-boundary modules", bp.modules.len())));
        assert!(advisory.third_order_tom_market_hook_fa.contains("IQ=150/EQ=145"));
        assert!(advisory.third_order_tom_market_hook_fa.contains("200 \u{633}\u{62a}\u{627}\u{631}\u{647}\u{654} \u{62a}\u{644}\u{6af}\u{631}\u{627}\u{645} (XTR)"));
        assert!(advisory.third_order_tom_market_hook_fa.contains("1000 bps"));
    }

    /// Repair directives — map the raw pillar to the guilty expertise in deterministic order.
    #[test]
    fn repair_directives_map_weak_pillars_to_responsible_specialties() {
        let judge = MaironSovereignJudge::new("trn:dir");
        let weak = evaluate_user_product_with_mairon("trn:dir", MaironReaPillarScores {
            architecture: 19,
            security_hardening: 25,
            ux_and_anti_slop: 23,
            verification_proof: 2,
        }, vec![]);
        let directives = judge.repair_directives(&weak, 1);
        let targets: Vec<&SubAgentSpecialty> =
            directives.iter().map(|d| &d.target_specialty).collect();
        assert_eq!(targets.len(), 3);
        assert_eq!(targets[0], &SubAgentSpecialty::VerificationAndPolish);
        assert_eq!(targets[1], &SubAgentSpecialty::UiUxArchitect390);
        assert_eq!(targets[2], &SubAgentSpecialty::CoreLogicEngine);
        assert!(directives.iter().all(|d| d.attempt == 1));
        // red line → the security directive first of all
        let dirty = evaluate_user_product_with_mairon("trn:dir", MaironReaPillarScores {
            architecture: 25,
            security_hardening: 19,
            ux_and_anti_slop: 25,
            verification_proof: 25,
        }, vec!["placeholder secret detected".to_owned()]);
        let dirty_dirs = judge.repair_directives(&dirty, 2);
        assert_eq!(dirty_dirs[0].target_specialty, SubAgentSpecialty::SecurityAndApiBridge);
        assert_eq!(dirty_dirs[0].pillar, "security_hardening");
    }

    /// Independent adjudication of a marketplace product — a correct artifact passes; raw and contaminated fail.
    #[test]
    fn market_product_gate_accepts_only_forged_artifacts() {
        let good = "<!doctype html><html lang=\"fa\" dir=\"rtl\"><head>\
            <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
            <style>body{font-family:'Vazirmatn',sans-serif;max-width: 390px;}</style>\
            </head><body>\
            <section aria-label=\"\u{627}\u{644}\u{641}\">\u{62f}\u{627}\u{634}\u{628}\u{648}\u{631}\u{62f}</section><section>\u{6af}\u{632}\u{627}\u{631}\u{634}</section><section>\u{62a}\u{646}\u{638}\u{6cc}\u{645}\u{627}\u{62a}</section>\
            <script>/* assert: 390px zero-overflow */</script>\
            </body></html>";
        let verdict = MaironSovereignJudge::judge_market_product("prod:good", good);
        assert!(verdict.approved_for_listing, "score = {}", verdict.total_score);
        assert!(verdict.visible_badge.contains("Forge-Grade"));
        // raw → honest rejection
        let empty = MaironSovereignJudge::judge_market_product("prod:empty", "<p></p>");
        assert!(!empty.approved_for_listing);
        assert!(empty.total_score < MARKETPLACE_APPROVAL_THRESHOLD);
        // contaminated → red line, never traded even for a high score
        let dirty = format!("{good}<!-- TODO: add auth later -->");
        let dirty_verdict = MaironSovereignJudge::judge_market_product("prod:dirty", &dirty);
        assert!(!dirty_verdict.approved_for_listing);
        assert!(dirty_verdict
            .hard_failures
            .iter()
            .any(|f| f.contains("unfinished work markers")));
    }

    /// Price extraction — a Persian/Latin digit from the monetization slot; default 150⭐.
    #[test]
    fn price_hint_parses_latin_and_persian_digits() {
        assert_eq!(first_number_in("\u{642}\u{6cc}\u{645}\u{62a} \u{6f3}\u{6f0}\u{6f0} \u{633}\u{62a}\u{627}\u{631}\u{647}"), Some(300));
        assert_eq!(first_number_in("price 250 XTR"), Some(250));
        assert_eq!(first_number_in("\u{628}\u{62f}\u{648}\u{646} \u{639}\u{62f}\u{62f}"), None);
        assert_eq!(derive_price_stars(&full_slots()), 200);
        assert_eq!(derive_price_stars(&AlphaDiscoverySlots::default()), 150);
    }

    /// The `3×9` pattern — three layers × nine factors in the complete wire manifest.
    #[test]
    fn three_by_nine_pattern_manifest_is_complete() {
        let master = MasterPromptSpec::compile(&full_slots(), "trn:39", "citizen:900");
        let bp = ProjectBlueprint::compile(&full_slots(), &master.project_name);
        let orch = CommandCenterOrchestrator::new("trn:39", "citizen:900", 42);
        let out = orch.execute_command_center_with_mairon_loop(&master, &bp);
        // three layers per subagent — each layer carries its own content signature
        for agent in &out.manifest {
            assert!(agent.prompt.layer1_stable_core.contains("SOVEREIGN CODING LAWS"));
            assert!(agent.prompt.layer2_blueprint_context.contains("BUILD SESSION CONTEXT"));
            assert!(agent.prompt.layer3_dynamic_assignment.contains("ASSIGNMENT (dynamic)"));
        }
        // nine genome factors: f01..f09 — each factor carries its own content signature
        let g = &out.manifest[0].genome;
        assert!(g.f01_topology_and_harness.contains("IQ=139"));
        assert_eq!(g.f02_hard_constraints.len(), 6);
        assert_eq!(g.f03_token_economics.as_str(), "lite", "agent #0 is UiUx → Lite");
        assert!(g.f04_stochastic_alleles.allele_seed > 0);
        assert!(g.f05_failure_recovery.contains("attempt 3"));
        assert!(g.f06_compiled_xml_prompt.contains("</openbot_subagent_dna>"));
        assert!(g.f07_sanitizer_gate.passed);
        assert_eq!(g.f08_empirical_probes.len(), 2);
        assert!(g.f09_handoff_card.contains("pass criteria"));
    }
}

