//! `neural_hud_studio_engine` — neural HUD video studio (Wave 9 — Commit #29).
//!
//! Deterministic port of `TM Neural HUD Video Studio` onto the Kian sovereign kernel — curing
//! all 15 upstream defects (`UPSTREAM_DEFECTS_CURED`) with three testable outputs:
//!
//! 1. **Deterministic 3D HTML5 `window.seekTo(t)` generator** — [`build_neural_hud_html_bundle`]:
//!    a two-shell spherical Fibonacci mesh with 180 nodes (`SPHERE_RADIUS = 160`,
//!    `FOV = 340`, `camera_z = 420`, `rotY = t * 0.65`,
//!    `rotX = sin(t * 0.4) * 0.4`), edges sorted by Z depth, four
//!    HUD callouts (`TRINITY_MAIRON: IQ_150`, `MCP_GATEWAY: SYNC`,
//!    `BASE_8453: VERIFIED`, `CCXT_QUANT: CAUSAL`), four explicit
//!    timeline phases, cached geometry (no `getBoundingClientRect` per frame —
//!    the layout-thrashing cure), safe JSON injection (anti-XSS), and live
//!    `WebAudio` synth (sub-bass `55Hz A1 + 82.41Hz E2 + 110Hz A2` + radar ping
//!    `1046.5Hz C6 / 1567.98Hz G6`) for an audible browser preview.
//! 2. **Algorithmic `lavfi` filtergraph + master limiter** —
//!    [`build_ffmpeg_hud_audio_command`] (`amix=inputs=4` + fade-in/out filters +
//!    [`MASTER_LIMITER`] to eliminate clipping) and [`build_ffmpeg_mjpeg_pipe_command`]
//!    (720p and true `FullHD` 1080p in `9:16` and `16:9` at 24 and 60fps with
//!    `-movflags +faststart` for streaming).
//! 3. **Catalog of 8 telemetry presets + dynamic synthesis** — [`hud_preset_catalog`]
//!    (4 upstream presets: `mwie`, `sage-audit`, `geo-scope`,
//!    `tm-second-brain` + 4 Kian sovereign presets: `kia-monopoly-core`,
//!    `mairon-10gate-shield`, `quant-vibe-terminal`, `trinity-5model-genome`)
//!    and [`synthesize_custom_repo_hud_preset`] to turn any repo of choice into
//!    a deterministic HUD preset.
//!
//! Zero unsafe, zero network IO, unique and deterministic geometry — every generated bundle is
//! auditable via SHA-256.

use std::fmt::Write as _;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Neural HUD studio signature — present on every wire bundle.
pub const NEURAL_HUD_SIGNATURE: &str = "MAIRON-NEURAL-HUD";

/// Outer shell radius of the spherical mesh — constant ported from upstream.
pub const SPHERE_RADIUS: f64 = 160.0;

/// Focal length of the perspective camera.
pub const FOV: f64 = 340.0;

/// Camera Z position — all nodes scale by `FOV / (FOV + z + CAMERA_Z)`.
pub const CAMERA_Z: f64 = 420.0;

/// Node count per shell — two shells × 90 = 180 nodes.
pub const SHELL_NODE_COUNT: usize = 90;

/// Inner shell radius relative to the outer.
pub const INNER_SHELL_RADIUS_SCALE: f64 = 0.62;

/// Y rotation rate per second — `rotY = t * 0.65`.
pub const ROT_Y_RATE: f64 = 0.65;

/// X rotation rate — `rotX = sin(t * 0.4) * 0.4`.
pub const ROT_X_RATE: f64 = 0.4;

/// X rotation oscillation amplitude.
pub const ROT_X_AMPLITUDE: f64 = 0.4;

/// Default video duration in seconds.
pub const DEFAULT_DURATION_SEC: u32 = 60;

/// Maximum allowed duration (CLI and API safety cap).
pub const MAX_DURATION_SEC: u32 = 180;

/// Audio fade-in filter — `afade=t=in:ss=0:d=0.6`.
pub const FADE_IN_SEC: f64 = 0.6;

/// Audio fade-out filter — `afade=t=out:st={fade_out}:d=1.2`.
pub const FADE_OUT_SEC: f64 = 1.2;

/// Master limiter — appended after `amix` + fade filters to eliminate clipping.
pub const MASTER_LIMITER: &str = "alimiter=limit=0.95:attack=5:release=50";

/// Live synth sub-bass — `A1`, `E2`, and `A2`.
pub const SUB_BASS_HZ: [f64; 3] = [55.0, 82.41, 110.0];

/// Live synth radar ping — `C6` and `G6` (alternating every 4 seconds).
pub const RADAR_PING_HZ: [f64; 2] = [1046.5, 1567.98];

/// Fixed HUD callouts — the mesh's four named nodes.
pub const HUD_CALLOUTS: [(&str, &str); 4] = [
    ("TRINITY_MAIRON", "IQ_150"),
    ("MCP_GATEWAY", "SYNC"),
    ("BASE_8453", "VERIFIED"),
    ("CCXT_QUANT", "CAUSAL"),
];

/// Registry of the 15 cured upstream defects — each entry: (identifier, defect, cure applied).
pub const UPSTREAM_DEFECTS_CURED: [(&str, &str, &str); 15] = [
    (
        "D01",
        "per-frame getBoundingClientRect layout thrashing",
        "geometry is computed once in Rust and cached in window.__KIA_HUD_GEO — the render loop never touches layout",
    ),
    (
        "D02",
        "non-deterministic playback (rAF clock drift breaks reproducible renders)",
        "window.seekTo(t) pins the animation clock to an exact deterministic second",
    ),
    (
        "D03",
        "XSS via naive JSON.stringify injection of preset titles",
        "json_for_script escapes < > & and U+2028/U+2029 before embedding",
    ),
    (
        "D04",
        "silent browser preview (video had sound, HTML preview did not)",
        "in-browser WebAudio live synth (55/82.41/110 Hz sub-bass + C6/G6 radar pings)",
    ),
    (
        "D05",
        "clipping on the mixed master bus",
        "alimiter=limit=0.95:attack=5:release=50 appended after amix + fades",
    ),
    (
        "D06",
        "hardwired 720p output only",
        "true FullHD 1080p (1080x1920 / 1920x1080) alongside 720p",
    ),
    (
        "D07",
        "missing faststart — MP4 moov atom at file tail breaks streaming",
        "-movflags +faststart on every mux",
    ),
    (
        "D08",
        "fixed 24fps (judder on fast camera moves)",
        "24fps and 60fps render profiles",
    ),
    (
        "D09",
        "single-shell mesh (flat, depth-less look)",
        "dual-shell 180-node Fibonacci mesh (90 outer + 90 inner at 0.62R)",
    ),
    (
        "D10",
        "edge Z-fighting (edges drawn unsorted)",
        "edges painter-sorted by mean depth every frame",
    ),
    (
        "D11",
        "dead callouts detached from the mesh",
        "callouts are bound to fixed mesh node indices and re-projected per frame",
    ),
    (
        "D12",
        "one monotone timeline with no narrative beats",
        "4 explicit phases (NEURAL IGNITION → SEEK_TO PIPE → GENOME & ARDA → MAIRON SEALED)",
    ),
    (
        "D13",
        "hardcoded one-off copy (no preset system)",
        "8-preset telemetry catalog with slugs, accents and badges",
    ),
    (
        "D14",
        "no repo-to-HUD automation (every new repo was a manual edit)",
        "synthesize_custom_repo_hud_preset derives a deterministic preset from any repo name",
    ),
    (
        "D15",
        "audio tail clipped because fade-out started at the last sample",
        "afade=t=out:st=(duration-1.2):d=1.2 computed from the real duration",
    ),
];

// ══════════════════════ Errors ══════════════════════

/// HUD generator errors — typed and deterministic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HudError {
    /// Repo name for preset synthesis is empty.
    EmptyRepoName,
    /// The requested preset is not in the catalog.
    UnknownPreset {
        /// Unknown preset identifier.
        slug: String,
    },
    /// Duration outside the 1..=180 second range.
    DurationOutOfRange {
        /// Invalid duration.
        duration_sec: u32,
    },
    /// Frame invariant violation — dimensions do not match the aspect/resolution.
    InvalidFrameGeometry {
        /// Invalid width.
        width: u32,
        /// Invalid height.
        height: u32,
        /// Requested aspect.
        aspect: &'static str,
        /// Requested resolution.
        resolution: &'static str,
    },
    /// Invalid accent color (must be `#RGB` or `#RRGGBB` hex).
    InvalidAccentHex {
        /// Invalid value.
        value: String,
    },
}

impl std::fmt::Display for HudError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyRepoName => write!(f, "repo name is empty — cannot synthesize a HUD preset"),
            Self::UnknownPreset { slug } => write!(f, "unknown HUD preset slug: {slug}"),
            Self::DurationOutOfRange { duration_sec } => {
                write!(f, "duration {duration_sec}s is outside 1..={MAX_DURATION_SEC}")
            }
            Self::InvalidFrameGeometry {
                width,
                height,
                aspect,
                resolution,
            } => write!(
                f,
                "frame {width}x{height} does not match {aspect} at {resolution}"
            ),
            Self::InvalidAccentHex { value } => write!(f, "invalid accent hex color: {value}"),
        }
    }
}

impl std::error::Error for HudError {}

// ══════════════════════ Aspect / Resolution / Framerate ══════════════════════

/// Video aspect ratio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HudAspect {
    /// Vertical-video portrait — `9:16`.
    NineBySixteen,
    /// Classic horizontal — `16:9`.
    SixteenByNine,
}

impl HudAspect {
    /// Wire identifier — `9:16` | `16:9`.
    #[must_use]
    pub fn wire_id(self) -> &'static str {
        match self {
            Self::NineBySixteen => "9:16",
            Self::SixteenByNine => "16:9",
        }
    }
}

/// Output resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HudResolution {
    /// Stream-ready HD — 720p.
    Hd720,
    /// True `FullHD` — 1080p (D06 cure).
    FullHd1080,
}

impl HudResolution {
    /// Wire identifier — `720p` | `1080p`.
    #[must_use]
    pub fn wire_id(self) -> &'static str {
        match self {
            Self::Hd720 => "720p",
            Self::FullHd1080 => "1080p",
        }
    }
}

/// Render framerate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HudFps {
    /// Cinematic — 24.
    TwentyFour,
    /// Smooth high-fps — 60 (D08 cure).
    Sixty,
}

impl HudFps {
    /// Numeric value.
    #[must_use]
    pub fn as_u32(self) -> u32 {
        match self {
            Self::TwentyFour => 24,
            Self::Sixty => 60,
        }
    }

    /// Wire identifier — `24` | `60`.
    #[must_use]
    pub fn wire_id(self) -> &'static str {
        match self {
            Self::TwentyFour => "24",
            Self::Sixty => "60",
        }
    }
}

/// Complete video render spec — no global state; all functions are pure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HudRenderSpec {
    /// Aspect ratio.
    pub aspect: HudAspect,
    /// Resolution.
    pub resolution: HudResolution,
    /// Framerate.
    pub fps: HudFps,
    /// Duration in seconds.
    pub duration_sec: u32,
}

impl Default for HudRenderSpec {
    fn default() -> Self {
        Self {
            aspect: HudAspect::NineBySixteen,
            resolution: HudResolution::FullHd1080,
            fps: HudFps::TwentyFour,
            duration_sec: DEFAULT_DURATION_SEC,
        }
    }
}

impl HudRenderSpec {
    /// Expected frame dimensions for this spec — `width` and `height`.
    ///
    /// (9:16 → 720×1280 or 1080×1920; 16:9 → 1280×720 or 1920×1080)
    #[must_use]
    pub fn frame_size(&self) -> (u32, u32) {
        match (self.resolution, self.aspect) {
            (HudResolution::Hd720, HudAspect::NineBySixteen) => (720, 1280),
            (HudResolution::Hd720, HudAspect::SixteenByNine) => (1280, 720),
            (HudResolution::FullHd1080, HudAspect::NineBySixteen) => (1080, 1920),
            (HudResolution::FullHd1080, HudAspect::SixteenByNine) => (1920, 1080),
        }
    }

    /// Fade-out start — `duration − 1.2` (D15 cure).
    #[must_use]
    pub fn fade_out_start(&self) -> f64 {
        f64::from(self.duration_sec) - FADE_OUT_SEC
    }
}

// ══════════════════════ Deterministic geometry ══════════════════════

/// Mesh node — unitless 3D coordinates in model space (before rotation).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HudNodeGeometry {
    /// 0-based index across all 180 nodes.
    pub index: u32,
    /// Shell — `0` outer, `1` inner.
    pub shell: u32,
    /// Model X.
    pub x: f64,
    /// Model Y.
    pub y: f64,
    /// Model Z.
    pub z: f64,
    /// Callout label — only for the 4 named nodes.
    pub label: Option<String>,
    /// Callout value — only for the 4 named nodes.
    pub value: Option<String>,
}

/// Mesh edge — a pair of node indices.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HudEdgeGeometry {
    /// First endpoint.
    pub a: u32,
    /// Second endpoint.
    pub b: u32,
}

/// Generates `n` Fibonacci points on a sphere of radius `radius` — deterministic, no RNG.
#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)] // Small integer-valued geometry — f64 precision is sufficient
#[must_use]
pub fn fibonacci_shell(n: usize, radius: f64) -> Vec<(f64, f64, f64)> {
    // Fibonacci golden angle — uniform spherical spacing (computed inline — the sqrt is not constant)
    let golden_angle = std::f64::consts::PI * (3.0 - 5.0_f64.sqrt());
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let y = 1.0 - (2.0 * (f64::from(i as u32) + 0.5)) / f64::from(n as u32);
        let ring = (1.0 - y * y).max(0.0).sqrt();
        let theta = golden_angle * f64::from(i as u32);
        out.push((theta.cos() * ring * radius, y * radius, theta.sin() * ring * radius));
    }
    out
}

/// Pin the callouts to fixed mesh indices (D11 cure).
const CALLOUT_BINDINGS: [(u32, u32); 4] = [
    (0, 0),     // TRINITY_MAIRON — the first node of the outer shell
    (0, 45),    // MCP_GATEWAY — outer shell
    (1, 30),    // BASE_8453 — inner shell
    (1, 60),    // CCXT_QUANT — inner shell
];

/// The complete two-shell 180-node mesh + connected callouts — always this exact output.
#[allow(clippy::cast_precision_loss)]
#[must_use]
pub fn hud_mesh_nodes() -> Vec<HudNodeGeometry> {
    let outer = fibonacci_shell(SHELL_NODE_COUNT, SPHERE_RADIUS);
    let inner = fibonacci_shell(SHELL_NODE_COUNT, SPHERE_RADIUS * INNER_SHELL_RADIUS_SCALE);
    let mut nodes = Vec::with_capacity(SHELL_NODE_COUNT * 2);
    for (shell, shell_points) in [(0u32, &outer), (1u32, &inner)] {
        for (x, y, z) in shell_points {
            nodes.push(HudNodeGeometry {
                index: u32::try_from(nodes.len()).unwrap_or_default(),
                shell,
                x: *x,
                y: *y,
                z: *z,
                label: None,
                value: None,
            });
        }
    }
    let shell_count_u32 = u32::try_from(SHELL_NODE_COUNT).unwrap_or(0);
    for (shell, local) in CALLOUT_BINDINGS {
        let idx = shell * shell_count_u32 + local;
        if let (Some(callout), Some(node)) = (
            HUD_CALLOUTS.get(
                CALLOUT_BINDINGS
                    .iter()
                    .position(|b| b.0 == shell && b.1 == local)
                    .unwrap_or_default(),
            ),
            nodes.get_mut(idx as usize),
        ) {
            node.label = Some(callout.0.to_owned());
            node.value = Some(callout.1.to_owned());
        }
    }
    nodes
}

/// Mesh edges — for each node, 3 nearest same-shell neighbors + 8 deterministic
/// inter-shell bridges. Pairs are deduped (a < b). Deterministic and RNG-free.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_precision_loss)]
#[must_use]
pub fn hud_mesh_edges(nodes: &[HudNodeGeometry]) -> Vec<HudEdgeGeometry> {
    let mut seen = std::collections::BTreeSet::new();
    let mut edges = Vec::new();
    let link = |a: u32, b: u32, seen: &mut std::collections::BTreeSet<(u32, u32)>, edges: &mut Vec<HudEdgeGeometry>| {
        let (lo, hi) = if a < b { (a, b) } else { (b, a) };
        if lo != hi && seen.insert((lo, hi)) {
            edges.push(HudEdgeGeometry { a: lo, b: hi });
        }
    };
    for shell in 0..2u32 {
        let base = shell * SHELL_NODE_COUNT as u32;
        for i in 0..SHELL_NODE_COUNT {
            let a = &nodes[(base + i as u32) as usize];
            // Three nearest same-shell neighbors — Euclidean distance in model space
            let mut best: Vec<(f64, u32)> = Vec::with_capacity(SHELL_NODE_COUNT);
            for j in 0..SHELL_NODE_COUNT {
                if i == j {
                    continue;
                }
                let b = &nodes[(base + j as u32) as usize];
                let dx = a.x - b.x;
                let dy = a.y - b.y;
                let dz = a.z - b.z;
                best.push((dx * dx + dy * dy + dz * dz, base + j as u32));
            }
            best.sort_by(|l, r| l.0.partial_cmp(&r.0).unwrap_or(std::cmp::Ordering::Equal));
            for (_, neighbor) in best.iter().take(3) {
                link(a.index, *neighbor, &mut seen, &mut edges);
            }
        }
    }
    // Inter-shell bridges — every 15th outer node to its nearest inner node
    for i in (0..SHELL_NODE_COUNT).step_by(15) {
        let a = &nodes[i];
        let mut best: Option<(f64, u32)> = None;
        for j in 0..SHELL_NODE_COUNT {
            let b = &nodes[SHELL_NODE_COUNT + j];
            let dx = a.x - b.x;
            let dy = a.y - b.y;
            let dz = a.z - b.z;
            let d = dx * dx + dy * dy + dz * dz;
            if best.is_none_or(|(bd, _)| d < bd) {
                best = Some((d, b.index));
            }
        }
        if let Some((_, b)) = best {
            link(a.index, b, &mut seen, &mut edges);
        }
    }
    edges
}

// ══════════════════════ Timeline phases ══════════════════════

/// One HUD timeline phase.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HudPhase {
    /// 1-based phase number.
    pub index: u32,
    /// Explicit title — e.g. `PHASE 01 // NEURAL IGNITION`.
    pub title: String,
    /// Start in seconds.
    pub start_sec: f64,
    /// End in seconds.
    pub end_sec: f64,
}

/// Fixed titles of the four phases.
pub const PHASE_TITLES: [&str; 4] = [
    "PHASE 01 // NEURAL IGNITION",
    "PHASE 02 // DETERMINISTIC SEEK_TO PIPE",
    "PHASE 03 // 5-MODEL GENOME & ARDA HUB",
    "PHASE 04 // MAIRON 100/100 SEALED",
];

/// Four explicit timeline phases for duration `duration_sec` — boundaries are proportional to
/// 14/29/44/60 of 60 seconds and always cover the full span.
#[allow(clippy::cast_precision_loss)]
#[must_use]
pub fn hud_timeline_phases(duration_sec: u32) -> Vec<HudPhase> {
    let total = f64::from(duration_sec);
    let marks = [0.0, 14.0 / 60.0, 29.0 / 60.0, 44.0 / 60.0, 1.0];
    let bounds: Vec<f64> = marks
        .iter()
        .map(|m| (total * m * 100.0).round() / 100.0)
        .collect();
    PHASE_TITLES
        .iter()
        .enumerate()
        .map(|(i, title)| HudPhase {
            index: u32::try_from(i + 1).unwrap_or(1),
            title: (*title).to_owned(),
            start_sec: bounds[i],
            end_sec: bounds[i + 1],
        })
        .collect()
}

// ══════════════════════ Presets ══════════════════════

/// Preset origin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HudPresetSource {
    /// The four TM upstream presets.
    Upstream,
    /// The four Kian sovereign presets.
    KiaSovereign,
    /// Preset synthesized from a repo name.
    Synthesized,
}

impl HudPresetSource {
    /// Wire identifier — `upstream` | `kia_sovereign` | `synthesized`.
    #[must_use]
    pub fn wire_id(self) -> &'static str {
        match self {
            Self::Upstream => "upstream",
            Self::KiaSovereign => "kia_sovereign",
            Self::Synthesized => "synthesized",
        }
    }
}

/// One HUD telemetry preset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HudPreset {
    /// Unique identifier — `mwie`, `kia-monopoly-core`, …
    pub slug: String,
    /// Display title.
    pub title: String,
    /// Tagline under the title.
    pub tagline: String,
    /// Accent color as hex — `#RRGGBB`.
    pub accent_hex: String,
    /// Top-left corner badge.
    pub badge: String,
    /// Origin.
    pub source: HudPresetSource,
    /// Rotating node labels (at most 4).
    pub node_labels: Vec<String>,
}

/// Catalog of 8 telemetry presets — 4 upstream + 4 Kian sovereign (fixed order).
#[must_use]
pub fn hud_preset_catalog() -> Vec<HudPreset> {
    vec![
        HudPreset {
            slug: "mwie".into(),
            title: "MWIE Neural Cortex".into(),
            tagline: "Deterministic brain telemetry over a dual-shell mesh".into(),
            accent_hex: "#7C4DFF".into(),
            badge: "MWIE".into(),
            source: HudPresetSource::Upstream,
            node_labels: vec!["CORTEX".into(), "SYNAPSE".into(), "PULSE".into()],
        },
        HudPreset {
            slug: "sage-audit".into(),
            title: "SAGE Audit Mesh".into(),
            tagline: "Forensic audit telemetry with sealed gates".into(),
            accent_hex: "#00E5A0".into(),
            badge: "SAGE".into(),
            source: HudPresetSource::Upstream,
            node_labels: vec!["AUDIT".into(), "GATE".into(), "PROOF".into()],
        },
        HudPreset {
            slug: "geo-scope".into(),
            title: "GEO Scope Planetary".into(),
            tagline: "Planetary scope sweep across the mesh".into(),
            accent_hex: "#38BDF8".into(),
            badge: "GEO".into(),
            source: HudPresetSource::Upstream,
            node_labels: vec!["SCAN".into(), "ORBIT".into(), "SWEEP".into()],
        },
        HudPreset {
            slug: "tm-second-brain".into(),
            title: "TM Second Brain".into(),
            tagline: "Second-brain memory layers on a rotating sphere".into(),
            accent_hex: "#F471B5".into(),
            badge: "TM".into(),
            source: HudPresetSource::Upstream,
            node_labels: vec!["MEMORY".into(), "RECALL".into(), "LINK".into()],
        },
        HudPreset {
            slug: "kia-monopoly-core".into(),
            title: "KIA Monopoly Core".into(),
            tagline: "Sovereign monopoly core — 5,870,020 KIAN genesis supply".into(),
            accent_hex: "#FACC15".into(),
            badge: "0xKIA SOVEREIGN".into(),
            source: HudPresetSource::KiaSovereign,
            node_labels: vec!["MONOPOLY".into(), "GENESIS".into(), "SOVEREIGN".into()],
        },
        HudPreset {
            slug: "mairon-10gate-shield".into(),
            title: "Mairon 10-Gate Shield".into(),
            tagline: "Ten forensic gates between your code and the ledger".into(),
            accent_hex: "#EF4444".into(),
            badge: "MAIRON".into(),
            source: HudPresetSource::KiaSovereign,
            node_labels: vec!["GATE".into(), "SHIELD".into(), "REA".into()],
        },
        HudPreset {
            slug: "quant-vibe-terminal".into(),
            title: "Quant Vibe Terminal".into(),
            tagline: "Causal Vibe-Trading backtests across 8 CCXT venues".into(),
            accent_hex: "#22D3EE".into(),
            badge: "QUANT".into(),
            source: HudPresetSource::KiaSovereign,
            node_labels: vec!["CAUSAL".into(), "SHARPE".into(), "BACKTEST".into()],
        },
        HudPreset {
            slug: "trinity-5model-genome".into(),
            title: "Trinity 5-Model Genome".into(),
            tagline: "Penta-model genome routing across the Trinity pipeline".into(),
            accent_hex: "#A78BFA".into(),
            badge: "TRINITY".into(),
            source: HudPresetSource::KiaSovereign,
            node_labels: vec!["GENOME".into(), "PENTA".into(), "ROUTE".into()],
        },
    ]
}

/// Finds a preset in the catalog by identifier.
///
/// # Errors
/// Unknown preset → [`HudError::UnknownPreset`].
pub fn find_hud_preset(slug: &str) -> Result<HudPreset, HudError> {
    hud_preset_catalog()
        .into_iter()
        .find(|p| p.slug == slug)
        .ok_or_else(|| HudError::UnknownPreset {
            slug: slug.to_owned(),
        })
}

/// Accepts a valid accent color or derives one from the repo name's SHA-256.
fn resolve_accent(repo_name: &str, accent: Option<&str>) -> Result<String, HudError> {
    let Some(raw) = accent.map(str::trim).filter(|v| !v.is_empty()) else {
        let digest = Sha256::digest(repo_name.as_bytes());
        let hex = format!("{digest:x}");
        return Ok(format!("#{}", hex.chars().take(6).collect::<String>()).to_uppercase());
    };
    let body = raw.strip_prefix('#').unwrap_or(raw);
    let valid = (body.len() == 3 || body.len() == 6)
        && body.chars().all(|c| c.is_ascii_hexdigit());
    if !valid {
        return Err(HudError::InvalidAccentHex {
            value: raw.to_owned(),
        });
    }
    let hex = if body.len() == 3 {
        body.chars()
            .flat_map(|c| [c, c])
            .collect::<String>()
            .to_uppercase()
    } else {
        body.to_uppercase()
    };
    Ok(format!("#{hex}"))
}

/// Derives node labels from repo text — letters/digits, at most 8 characters,
/// at most 4 labels, deterministic.
fn derive_node_labels(text: &str) -> Vec<String> {
    text.split(|c: char| c.is_whitespace() || matches!(c, ',' | '-' | '/' | '|' | '.'))
        .filter(|w| w.chars().any(char::is_alphanumeric))
        .map(|w| {
            w.chars()
                .filter(char::is_ascii_alphanumeric)
                .take(8)
                .collect::<String>()
                .to_ascii_uppercase()
        })
        .filter(|w| !w.is_empty())
        .take(4)
        .collect()
}

/// Synthesizes a HUD preset from any repo name — deterministic: same input, same output
/// (an unstated color is derived from the repo name's SHA-256; D14 cure).
///
/// # Errors
/// Empty name → [`HudError::EmptyRepoName`]; invalid color →
/// [`HudError::InvalidAccentHex`].
pub fn synthesize_custom_repo_hud_preset(
    repo_name: &str,
    tagline: &str,
    accent_hex: Option<&str>,
) -> Result<HudPreset, HudError> {
    let name = repo_name.trim();
    if name.is_empty() {
        return Err(HudError::EmptyRepoName);
    }
    let slug = {
        let mut s = String::with_capacity(name.len());
        let mut last_dash = false;
        for c in name.chars() {
            if c.is_ascii_alphanumeric() {
                s.push(c.to_ascii_lowercase());
                last_dash = false;
            } else if !last_dash {
                s.push('-');
                last_dash = true;
            }
        }
        let trimmed = s.trim_matches('-').to_owned();
        if trimmed.is_empty() {
            "custom-repo".to_owned()
        } else {
            trimmed
        }
    };
    Ok(HudPreset {
        slug,
        title: name.to_owned(),
        tagline: if tagline.trim().is_empty() {
            "Custom repo neural telemetry".to_owned()
        } else {
            tagline.trim().to_owned()
        },
        accent_hex: resolve_accent(name, accent_hex)?,
        badge: name.to_ascii_uppercase(),
        source: HudPresetSource::Synthesized,
        node_labels: {
            let derived = derive_node_labels(tagline);
            if derived.is_empty() {
                vec!["REPO".to_owned(), "NEURAL".to_owned(), "HUD".to_owned()]
            } else {
                derived
            }
        },
    })
}

// ══════════════════════ Safe JSON injection ══════════════════════

/// HTML escaping for the template's display strings — the studio's own rule
/// (`&`, `<`, `>`, and `"`) — together with safe JSON injection, the second anti-XSS layer.
#[must_use]
pub fn html_attr_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// JSON safe inside `<script>` — D03 cure: `<`, `>`, `&`, and U+2028/U+2029
/// are converted to `\uXXXX` escapes so no fresh `</script>` or HTML can be born.
#[must_use]
pub fn json_for_script(value: &serde_json::Value) -> String {
    let raw = serde_json::to_string(value).unwrap_or_else(|_| "null".to_owned());
    raw.replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('&', "\\u0026")
        .replace('\u{2028}', "\\u2028")
        .replace('\u{2029}', "\\u2029")
}

// ══════════════════════ HTML bundle ══════════════════════

/// The generated HTML5 bundle — all figures needed for audit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NeuralHudHtmlBundle {
    /// Self-contained HTML document — one file, zero external dependencies.
    pub html: String,
    /// Preset identifier.
    pub preset_slug: String,
    /// Frame width.
    pub frame_width: u32,
    /// Frame height.
    pub frame_height: u32,
    /// Framerate.
    pub fps: u32,
    /// Duration in seconds.
    pub duration_sec: u32,
    /// Node count — always 180.
    pub node_count: usize,
    /// Edge count.
    pub edge_count: usize,
    /// Callout count — always 4.
    pub callout_count: usize,
    /// Phase count — always 4.
    pub phase_count: usize,
    /// SHA-256 of the HTML document.
    pub html_sha256: String,
    /// Live `WebAudio` synth is enabled.
    pub webaudio_enabled: bool,
    /// The deterministic `window.seekTo(t)` is present.
    pub seek_to_deterministic: bool,
}

const HUD_HTML_TEMPLATE: &str = r##"<!DOCTYPE html>
<html lang="en" data-kia="neural-hud">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>__TITLE__</title>
<style>
  :root { --accent: __ACCENT__; --bg: #050507; --panel: rgba(255,255,255,0.06); --line: rgba(255,255,255,0.08); }
  * { margin: 0; padding: 0; box-sizing: border-box; }
  html, body { width: 100%; height: 100%; background: var(--bg); overflow: hidden; }
  body { font-family: "JetBrains Mono", "SF Mono", Consolas, monospace; color: #E7E9EE; }
  #stage { position: absolute; inset: 0; width: 100%; height: 100%; }
  .hud-chip { position: absolute; padding: 8px 14px; background: var(--panel); border: 1px solid var(--line); border-radius: 10px; backdrop-filter: blur(6px); font-size: 12px; letter-spacing: 0.08em; }
  #badge { top: 18px; left: 18px; color: var(--accent); font-weight: 700; }
  #phase { top: 18px; right: 18px; }
  #phase b { color: var(--accent); }
  #callouts { position: absolute; inset: 0; pointer-events: none; }
  .callout { position: absolute; transform: translate(-50%, -130%); padding: 6px 10px; background: rgba(5,5,7,0.72); border: 1px solid var(--line); border-left: 3px solid var(--accent); border-radius: 8px; font-size: 11px; white-space: nowrap; }
  .callout .v { color: var(--accent); font-weight: 700; }
  #timecode { bottom: 18px; left: 18px; font-variant-numeric: tabular-nums; }
  #tagline { bottom: 18px; right: 18px; color: #8A93A6; max-width: 46%; text-align: right; }
  #audioHint { bottom: 56px; left: 18px; color: #6B7librar180; color: #6B7280; font-size: 10px; }
  .edge-glow { stroke: var(--accent); }
</style>
</head>
<body>
<canvas id="stage" width="__W__" height="__H__"></canvas>
<div id="callouts"></div>
<div class="hud-chip" id="badge">__BADGE__</div>
<div class="hud-chip" id="phase">PHASE <b id="phaseIdx">01</b> <span id="phaseTitle">__PHASE0__</span></div>
<div class="hud-chip" id="timecode">00:00 / __DURATION__</div>
<div class="hud-chip" id="tagline">__TAGLINE__</div>
<div id="audioHint">tap / key = enable live WebAudio synth</div>
<script>
"use strict";
// KIA Neural HUD — deterministic geometry cache (D01: zero per-frame layout reads)
window.__KIA_HUD_GEO = __GEO__;
window.__KIA_HUD_PHASES = __PHASES__;
window.__KIA_HUD_CONFIG = __CONFIG__;
window.__KIA_HUD_SIGNATURE = "__SIGNATURE__";
(function () {
  var GEO = window.__KIA_HUD_GEO, PHASES = window.__KIA_HUD_PHASES, CFG = window.__KIA_HUD_CONFIG;
  var canvas = document.getElementById("stage");
  canvas.width = CFG.width; canvas.height = CFG.height;
  var ctx = canvas.getContext("2d");
  var nodes = GEO.nodes, edges = GEO.edges;
  var state = { t: 0, playing: true, last: null, audio: null, pingIdx: 0, lastPingAt: -1 };

  function project(n, t) {
    var rotY = t * CFG.rotYRate, rotX = Math.sin(t * CFG.rotXRate) * CFG.rotXAmp;
    var cosY = Math.cos(rotY), sinY = Math.sin(rotY), cosX = Math.cos(rotX), sinX = Math.sin(rotX);
    var x1 = n.x * cosY - n.z * sinY;
    var z1 = n.x * sinY + n.z * cosY;
    var y2 = n.y * cosX - z1 * sinX;
    var z2 = n.y * sinX + z1 * cosX;
    var scale = CFG.fov / (CFG.fov + z2 + CFG.cameraZ);
    return { sx: CFG.cx + x1 * scale, sy: CFG.cy + y2 * scale, scale: scale, z: z2 };
  }

  // D10 — painter's sort by mean depth
  function render(t) {
    var w = CFG.width, h = CFG.height;
    ctx.fillStyle = "#050507";
    ctx.fillRect(0, 0, w, h);
    var proj = nodes.map(function (n) { return project(n, t); });
    var sortedEdges = edges.map(function (e) {
      var a = proj[e.a], b = proj[e.b];
      return { e: e, depth: (a.z + b.z) / 2, a: a, b: b };
    }).sort(function (l, r) { return l.depth - r.depth; });
    ctx.lineWidth = 1;
    for (var i = 0; i < sortedEdges.length; i++) {
      var s = sortedEdges[i];
      var alpha = 0.10 + 0.16 * (1 - (s.depth + CFG.sphereRadius) / (2 * CFG.sphereRadius));
      ctx.strokeStyle = "rgba(255,255,255," + alpha.toFixed(3) + ")";
      ctx.beginPath();
      ctx.moveTo(s.a.sx, s.a.sy);
      ctx.lineTo(s.b.sx, s.b.sy);
      ctx.stroke();
    }
    var sortedNodes = nodes.map(function (n, i) { return { n: n, p: proj[i] }; })
      .sort(function (l, r) { return l.p.z - r.p.z; });
    for (var k = 0; k < sortedNodes.length; k++) {
      var it = sortedNodes[k];
      var r = Math.max(1.2, it.n.label ? 5.5 * it.p.scale : 2.6 * it.p.scale);
      ctx.beginPath();
      ctx.arc(it.p.sx, it.p.sy, r, 0, 6.28318);
      ctx.fillStyle = it.n.label ? CFG.accent : "rgba(231,233,238,0.85)";
      ctx.fill();
    }
    // D11 — callouts re-projected per frame and anchored to their DOM cards
    for (var c = 0; c < CFG.calloutNodes.length; c++) {
      var idx = CFG.calloutNodes[c];
      var p = proj[idx];
      var card = document.getElementById("callout-" + c);
      if (card) { card.style.left = p.sx + "px"; card.style.top = p.sy + "px"; }
    }
    // D12 — explicit phase chip
    var ph = PHASES[PHASES.length - 1];
    for (var q = 0; q < PHASES.length; q++) {
      if (t >= PHASES[q].start_sec && t < PHASES[q].end_sec) { ph = PHASES[q]; break; }
    }
    document.getElementById("phaseIdx").textContent = ("0" + ph.index).slice(-2);
    document.getElementById("phaseTitle").textContent = ph.title.replace(/^PHASE \d+ \/\/ /, "");
    var m = Math.floor(t / 60), s = Math.floor(t % 60);
    document.getElementById("timecode").textContent =
      ("0" + m).slice(-2) + ":" + ("0" + s).slice(-2) + " / " + ("0" + Math.floor(CFG.duration / 60)).slice(-2) + ":" + ("0" + Math.floor(CFG.duration % 60)).slice(-2);
  }

  // D02 — deterministic seek: pins the clock to an exact second
  window.seekTo = function (t) {
    var clamped = Math.max(0, Math.min(CFG.duration, Number(t) || 0));
    state.t = clamped;
    state.playing = false;
    render(state.t);
    return state.t;
  };
  window.__KIA_HUD_SEEK_TO = window.seekTo;

  // D04 — in-browser live WebAudio synth: sub-bass bed + radar pings
  function initAudio() {
    if (state.audio) { return; }
    var AC = window.AudioContext || window.webkitAudioContext;
    if (!AC) { return; }
    var ac = new AC();
    var master = ac.createGain();
    master.gain.value = 0.34;
    var comp = ac.createDynamicsCompressor();
    master.connect(comp); comp.connect(ac.destination);
    var bed = CFG.subBassHz;
    for (var i = 0; i < bed.length; i++) {
      var osc = ac.createOscillator(), g = ac.createGain();
      osc.type = "sine"; osc.frequency.value = bed[i];
      g.gain.value = [0.20, 0.13, 0.10][i] || 0.1;
      osc.connect(g); g.connect(master); osc.start();
    }
    state.audio = { ac: ac, master: master };
    window.__KIA_HUD_AUDIO = true;
  }
  function radarPing(t) {
    if (!state.audio) { return; }
    var slot = Math.floor(t / 4);
    if (slot === state.lastPingAt) { return; }
    state.lastPingAt = slot;
    var hz = CFG.radarPingHz[slot % CFG.radarPingHz.length];
    var ac = state.audio.ac;
    var osc = ac.createOscillator(), g = ac.createGain();
    osc.type = "sine"; osc.frequency.value = hz;
    g.gain.setValueAtTime(0.16, ac.currentTime);
    g.gain.exponentialRampToValueAtTime(0.0001, ac.currentTime + 0.6);
    osc.connect(g); g.connect(state.audio.master);
    osc.start(); osc.stop(ac.currentTime + 0.65);
  }
  window.addEventListener("pointerdown", initAudio);
  window.addEventListener("keydown", initAudio);

  function loop(now) {
    if (state.playing) {
      if (state.last === null) { state.last = now; }
      state.t = Math.min(CFG.duration, state.t + (now - state.last) / 1000);
      state.last = now;
      render(state.t);
      radarPing(state.t);
      if (state.t >= CFG.duration) { state.playing = false; }
    }
    window.requestAnimationFrame(loop);
  }
  // DOM callout cards (positioned purely from cached projection — no layout reads)
  var host = document.getElementById("callouts");
  for (var c = 0; c < CFG.calloutNodes.length; c++) {
    var card = document.createElement("div");
    card.className = "callout"; card.id = "callout-" + c;
    var node = nodes[CFG.calloutNodes[c]];
    card.innerHTML = "<b>" + node.label + "</b>: <span class=\"v\">" + node.value + "</span>";
    host.appendChild(card);
  }
  render(state.t);
  window.requestAnimationFrame(loop);
})();
</script>
</body>
</html>
"##;

/// HUD 3D HTML5 bundle generator — cached geometry, safe JSON injection,
/// four phases, connected callouts, and live `WebAudio`. Deterministic: two calls with the same
/// input produce byte-for-byte identical HTML.
///
/// # Errors
/// Duration outside 1..=180 → [`HudError::DurationOutOfRange`].
#[allow(clippy::too_many_lines)] // HTML template — its length is part of the artifact's nature
pub fn build_neural_hud_html_bundle(
    preset: &HudPreset,
    spec: &HudRenderSpec,
) -> Result<NeuralHudHtmlBundle, HudError> {
    if spec.duration_sec == 0 || spec.duration_sec > MAX_DURATION_SEC {
        return Err(HudError::DurationOutOfRange {
            duration_sec: spec.duration_sec,
        });
    }
    let (width, height) = spec.frame_size();
    let nodes = hud_mesh_nodes();
    let edges = hud_mesh_edges(&nodes);
    let phases = hud_timeline_phases(spec.duration_sec);

    let geo = serde_json::json!({
        "nodes": nodes.iter().map(|n| serde_json::json!({
            "i": n.index, "s": n.shell, "x": n.x, "y": n.y, "z": n.z,
            "l": n.label, "v": n.value,
        })).collect::<Vec<_>>(),
        "edges": edges.iter().map(|e| serde_json::json!([e.a, e.b])).collect::<Vec<_>>(),
    });
    let phases_json = serde_json::json!(phases.iter().map(|p| serde_json::json!({
        "index": p.index, "title": p.title,
        "start_sec": p.start_sec, "end_sec": p.end_sec,
    })).collect::<Vec<_>>());
    let callout_nodes: Vec<u32> = nodes
        .iter()
        .filter(|n| n.label.is_some())
        .map(|n| n.index)
        .collect();
    let config = serde_json::json!({
        "width": width, "height": height,
        "fov": FOV, "cameraZ": CAMERA_Z, "sphereRadius": SPHERE_RADIUS,
        "rotYRate": ROT_Y_RATE, "rotXRate": ROT_X_RATE, "rotXAmp": ROT_X_AMPLITUDE,
        "fps": spec.fps.as_u32(), "duration": spec.duration_sec,
        "cx": width / 2, "cy": height / 2,
        "accent": preset.accent_hex,
        "calloutNodes": callout_nodes,
        "subBassHz": SUB_BASS_HZ, "radarPingHz": RADAR_PING_HZ,
    });
    let first_phase = phases
        .first()
        .map_or_else(|| PHASE_TITLES[0].to_owned(), |p| p.title.clone());
    let minutes = spec.duration_sec / 60;
    let seconds = spec.duration_sec % 60;
    let html = HUD_HTML_TEMPLATE
        .replace("__TITLE__", &html_attr_escape(&preset.title))
        .replace("__ACCENT__", &preset.accent_hex)
        .replace("__W__", &width.to_string())
        .replace("__H__", &height.to_string())
        .replace("__BADGE__", &html_attr_escape(&preset.badge))
        .replace("__PHASE0__", &html_attr_escape(&first_phase))
        .replace("__TAGLINE__", &html_attr_escape(&preset.tagline))
        .replace("__DURATION__", &format!("{minutes:02}:{seconds:02}"))
        .replace("__GEO__", &json_for_script(&geo))
        .replace("__PHASES__", &json_for_script(&phases_json))
        .replace("__CONFIG__", &json_for_script(&config))
        .replace("__SIGNATURE__", NEURAL_HUD_SIGNATURE);
    Ok(NeuralHudHtmlBundle {
        html_sha256: {
            let digest = Sha256::digest(html.as_bytes());
            let mut hex = String::with_capacity(64);
            for byte in digest {
                let _ = write!(hex, "{byte:02x}");
            }
            hex
        },
        html,
        preset_slug: preset.slug.clone(),
        frame_width: width,
        frame_height: height,
        fps: spec.fps.as_u32(),
        duration_sec: spec.duration_sec,
        node_count: nodes.len(),
        edge_count: edges.len(),
        callout_count: callout_nodes.len(),
        phase_count: phases.len(),
        webaudio_enabled: true,
        seek_to_deterministic: true,
    })
}

// ══════════════════════ FFmpeg commands ══════════════════════

/// Complete audio command — ready-to-run argv + an inspectable filtergraph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HudAudioCommand {
    /// Deterministic argv — element zero is `ffmpeg`.
    pub argv: Vec<String>,
    /// `lavfi` filtergraph — contains `amix=inputs=4` + fade filters + [`MASTER_LIMITER`].
    pub filter_complex: String,
    /// Fade-out start.
    pub fade_out_start: f64,
}

/// Audio filtergraph — 3 sub-basses + the C6 radar carrier (gated every 8 seconds with a
/// deterministic `volume`) → `amix=inputs=4:duration=first` → fade-in/out →
/// [`MASTER_LIMITER`] (D05/D15 cures).
fn hud_audio_filter_complex(duration_sec: u32) -> String {
    let d = duration_sec;
    let fade_out = ((f64::from(d) - FADE_OUT_SEC) * 100.0).round() / 100.0;
    format!(
        "sine=frequency=55:duration={d}[b0];\
         sine=frequency=82.41:duration={d}[b1];\
         sine=frequency=110:duration={d}[b2];\
         sine=frequency=1046.5:duration={d},\
volume='if(lt(mod(t\\,8)\\,0.35)\\,0.22\\,0)':eval=frame[p0];\
         [b0][b1][b2][p0]amix=inputs=4:duration=first,\
afade=t=in:ss=0:d={FADE_IN_SEC},\
afade=t=out:st={fade_out}:d={FADE_OUT_SEC},\
{MASTER_LIMITER}[aout]"
    )
}

/// Audio `FFmpeg` command for the given duration — shell-free argv (direct execution).
///
/// # Errors
/// Duration outside 1..=180 → [`HudError::DurationOutOfRange`].
pub fn build_ffmpeg_hud_audio_command(duration_sec: u32) -> Result<HudAudioCommand, HudError> {
    if duration_sec == 0 || duration_sec > MAX_DURATION_SEC {
        return Err(HudError::DurationOutOfRange { duration_sec });
    }
    let filter = hud_audio_filter_complex(duration_sec);
    Ok(HudAudioCommand {
        argv: vec![
            "ffmpeg".to_owned(),
            "-y".to_owned(),
            "-hide_banner".to_owned(),
            "-loglevel".to_owned(),
            "error".to_owned(),
            "-filter_complex".to_owned(),
            filter.clone(),
            "-map".to_owned(),
            "[aout]".to_owned(),
            "-ar".to_owned(),
            "48000".to_owned(),
            "-ac".to_owned(),
            "2".to_owned(),
            "-c:a".to_owned(),
            "pcm_s16le".to_owned(),
            "hud_audio.wav".to_owned(),
        ],
        filter_complex: filter,
        fade_out_start: ((f64::from(duration_sec) - FADE_OUT_SEC) * 100.0).round() / 100.0,
    })
}

/// Video `FFmpeg` command from the `MJPEG` pipe — correct resolution/aspect/framerate,
/// audio mux from `hud_audio.wav`, and `-movflags +faststart` (D06/D07/D08 cures).
///
/// # Errors
/// Invalid duration or frame invariant violation → [`HudError`].
pub fn build_ffmpeg_mjpeg_pipe_command(
    spec: &HudRenderSpec,
    audio_input: Option<&str>,
) -> Result<Vec<String>, HudError> {
    if spec.duration_sec == 0 || spec.duration_sec > MAX_DURATION_SEC {
        return Err(HudError::DurationOutOfRange {
            duration_sec: spec.duration_sec,
        });
    }
    let (width, height) = spec.frame_size();
    let (want_w, want_h) = match (spec.resolution, spec.aspect) {
        (HudResolution::Hd720, HudAspect::NineBySixteen) => (720, 1280),
        (HudResolution::Hd720, HudAspect::SixteenByNine) => (1280, 720),
        (HudResolution::FullHd1080, HudAspect::NineBySixteen) => (1080, 1920),
        (HudResolution::FullHd1080, HudAspect::SixteenByNine) => (1920, 1080),
    };
    if (width, height) != (want_w, want_h) {
        return Err(HudError::InvalidFrameGeometry {
            width,
            height,
            aspect: spec.aspect.wire_id(),
            resolution: spec.resolution.wire_id(),
        });
    }
    let mut argv = vec![
        "ffmpeg".to_owned(),
        "-y".to_owned(),
        "-hide_banner".to_owned(),
        "-loglevel".to_owned(),
        "error".to_owned(),
        "-f".to_owned(),
        "image2pipe".to_owned(),
        "-framerate".to_owned(),
        spec.fps.wire_id().to_owned(),
        "-i".to_owned(),
        "-".to_owned(),
    ];
    if let Some(audio) = audio_input {
        argv.extend(["-i".to_owned(), audio.to_owned()]);
        argv.extend(["-map".to_owned(), "0:v:0".to_owned(), "-map".to_owned(), "1:a:0".to_owned()]);
    }
    argv.extend([
        "-c:v".to_owned(),
        "libx264".to_owned(),
        "-preset".to_owned(),
        "veryfast".to_owned(),
        "-crf".to_owned(),
        "18".to_owned(),
        "-pix_fmt".to_owned(),
        "yuv420p".to_owned(),
        "-s".to_owned(),
        format!("{width}x{height}"),
        "-r".to_owned(),
        spec.fps.wire_id().to_owned(),
    ]);
    if audio_input.is_some() {
        argv.extend(["-c:a".to_owned(), "aac".to_owned(), "-b:a".to_owned(), "192k".to_owned(), "-shortest".to_owned()]);
    }
    argv.extend([
        "-movflags".to_owned(),
        "+faststart".to_owned(),
        "neural_hud.mp4".to_owned(),
    ]);
    Ok(argv)
}

// ══════════════════════ Tests ══════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn preset(slug: &str) -> HudPreset {
        find_hud_preset(slug).expect("preset exists")
    }

    #[test]
    fn geometry_is_deterministic_and_on_sphere() {
        let a = hud_mesh_nodes();
        let b = hud_mesh_nodes();
        assert_eq!(a, b, "dual-shell mesh must be byte-identical across calls");
        for (i, n) in a.iter().enumerate() {
            assert_eq!(n.index as usize, i, "indices must be dense");
            let r = (n.x * n.x + n.y * n.y + n.z * n.z).sqrt();
            let want = if n.shell == 0 {
                SPHERE_RADIUS
            } else {
                SPHERE_RADIUS * INNER_SHELL_RADIUS_SCALE
            };
            assert!(
                (r - want).abs() < 1e-6,
                "node {i} radius {r} must sit on shell {} radius {want}",
                n.shell
            );
        }
    }

    #[test]
    fn dual_shell_has_exactly_180_nodes_and_callouts_bound() {
        let nodes = hud_mesh_nodes();
        assert_eq!(nodes.len(), 180, "90 outer + 90 inner");
        assert_eq!(nodes.iter().filter(|n| n.shell == 0).count(), 90);
        assert_eq!(nodes.iter().filter(|n| n.shell == 1).count(), 90);
        let labeled: Vec<_> = nodes.iter().filter(|n| n.label.is_some()).collect();
        assert_eq!(labeled.len(), 4, "four callouts");
        assert_eq!(labeled[0].label.as_deref(), Some("TRINITY_MAIRON"));
        assert_eq!(labeled[0].value.as_deref(), Some("IQ_150"));
        assert_eq!(labeled[1].label.as_deref(), Some("MCP_GATEWAY"));
        assert_eq!(labeled[2].label.as_deref(), Some("BASE_8453"));
        assert_eq!(labeled[3].label.as_deref(), Some("CCXT_QUANT"));
    }

    #[test]
    fn edges_are_deduped_dense_and_connected() {
        let nodes = hud_mesh_nodes();
        let edges = hud_mesh_edges(&nodes);
        assert!(edges.len() >= 180, "mesh edges must be dense");
        for e in &edges {
            assert!(e.a < e.b, "edges stored with a<b");
            assert!(e.a < 180 && e.b < 180);
        }
        let unique: std::collections::BTreeSet<(u32, u32)> =
            edges.iter().map(|e| (e.a, e.b)).collect();
        assert_eq!(unique.len(), edges.len(), "no duplicate edges");
        // Every node has at least one edge — a connected mesh
        let mut covered = std::collections::BTreeSet::new();
        for e in &edges {
            covered.insert(e.a);
            covered.insert(e.b);
        }
        assert_eq!(covered.len(), 180, "every node carries at least one edge");
    }

    #[test]
    fn timeline_has_four_explicit_phases_covering_duration() {
        for duration in [30u32, 60, 90] {
            let phases = hud_timeline_phases(duration);
            assert_eq!(phases.len(), 4);
            assert_eq!(phases[0].title, "PHASE 01 // NEURAL IGNITION");
            assert_eq!(phases[1].title, "PHASE 02 // DETERMINISTIC SEEK_TO PIPE");
            assert_eq!(phases[2].title, "PHASE 03 // 5-MODEL GENOME & ARDA HUB");
            assert_eq!(phases[3].title, "PHASE 04 // MAIRON 100/100 SEALED");
            assert!((phases[0].start_sec - 0.0).abs() < f64::EPSILON);
            for pair in phases.windows(2) {
                assert!((pair[0].end_sec - pair[1].start_sec).abs() < 1e-9, "contiguous");
            }
            assert!((phases[3].end_sec - f64::from(duration)).abs() < 1e-9, "covers end");
        }
    }

    #[test]
    fn bundle_is_byte_deterministic_for_identical_inputs() {
        let spec = HudRenderSpec::default();
        let a = build_neural_hud_html_bundle(&preset("kia-monopoly-core"), &spec).expect("bundle");
        let b = build_neural_hud_html_bundle(&preset("kia-monopoly-core"), &spec).expect("bundle");
        assert_eq!(a, b, "same inputs → same bytes");
        assert_eq!(a.html_sha256.len(), 64);
    }

    #[test]
    fn html_embeds_cached_geometry_without_layout_reads() {
        let bundle =
            build_neural_hud_html_bundle(&preset("trinity-5model-genome"), &HudRenderSpec::default())
                .expect("bundle");
        assert!(bundle.html.contains("window.__KIA_HUD_GEO"), "cached geometry");
        assert!(bundle.html.contains("window.seekTo"), "deterministic seek");
        assert!(
            !bundle.html.contains("getBoundingClientRect"),
            "D01: render loop must never read layout"
        );
        assert!(bundle.html.contains("__KIA_HUD_SIGNATURE") || bundle.html.contains(NEURAL_HUD_SIGNATURE));
    }

    #[test]
    fn xss_payload_is_escaped_before_injection() {
        let malicious = synthesize_custom_repo_hud_preset(
            "</script><script>alert(1)</script>",
            "ignition & <boom>",
            None,
        )
        .expect("synthesis tolerates hostile text");
        let bundle =
            build_neural_hud_html_bundle(&malicious, &HudRenderSpec::default()).expect("bundle");
        assert!(
            !bundle.html.contains("</script><script>alert(1)</script>"),
            "raw payload must never survive"
        );
        assert!(!bundle.html.contains("& <boom>"), "raw ampersand/angle text escaped");
        assert!(bundle.html.contains("&lt;/script&gt;"), "escaped form present in title/badge");
    }

    #[test]
    fn webaudio_frequencies_are_embedded() {
        let bundle = build_neural_hud_html_bundle(&preset("mwie"), &HudRenderSpec::default())
            .expect("bundle");
        assert!(bundle.webaudio_enabled);
        for hz in ["55", "82.41", "110", "1046.5", "1567.98"] {
            assert!(bundle.html.contains(hz), "frequency {hz} must appear in synth config");
        }
        assert!(bundle.html.contains("AudioContext"), "live synth path present");
    }

    #[test]
    fn audio_command_appends_limiter_after_fades() {
        let cmd = build_ffmpeg_hud_audio_command(60).expect("audio cmd");
        let needle = "amix=inputs=4:duration=first,afade=t=in:ss=0:d=0.6,afade=t=out:st=58.8:d=1.2";
        let pos_amix = cmd
            .filter_complex
            .find(needle)
            .expect("amix + fades chain present");
        let pos_limiter = cmd
            .filter_complex
            .find(MASTER_LIMITER)
            .expect("limiter present");
        assert!(
            pos_limiter > pos_amix,
            "D05: limiter must be appended AFTER amix+afade"
        );
        assert!(cmd.filter_complex.ends_with(&format!("{MASTER_LIMITER}[aout]")));
        assert!((cmd.fade_out_start - 58.8).abs() < 1e-9);
    }

    #[test]
    fn audio_command_uses_four_mixed_inputs_and_covers_frequencies() {
        let cmd = build_ffmpeg_hud_audio_command(90).expect("audio cmd");
        assert!(cmd.filter_complex.contains("amix=inputs=4"));
        for hz in ["frequency=55", "frequency=82.41", "frequency=110", "frequency=1046.5"] {
            assert!(cmd.filter_complex.contains(hz), "sub-bass/radar source {hz}");
        }
        assert_eq!(cmd.argv.first().map(String::as_str), Some("ffmpeg"));
        let err = build_ffmpeg_hud_audio_command(0).unwrap_err();
        assert_eq!(err, HudError::DurationOutOfRange { duration_sec: 0 });
    }

    #[test]
    fn video_command_720p_vertical_24fps() {
        let spec = HudRenderSpec {
            aspect: HudAspect::NineBySixteen,
            resolution: HudResolution::Hd720,
            fps: HudFps::TwentyFour,
            duration_sec: 60,
        };
        let argv = build_ffmpeg_mjpeg_pipe_command(&spec, Some("hud_audio.wav")).expect("cmd");
        assert!(argv.contains(&"720x1280".to_owned()));
        assert!(argv.contains(&"24".to_owned()));
        let fast = argv.iter().position(|a| a == "-movflags").expect("faststart");
        assert_eq!(argv[fast + 1], "+faststart", "D07: moov atom front-loaded");
        assert!(argv.contains(&"1:a:0".to_owned()), "audio muxed from wav");
    }

    #[test]
    fn video_command_fullhd_1080p_both_aspects_and_60fps() {
        for (aspect, want) in [
            (HudAspect::NineBySixteen, "1080x1920"),
            (HudAspect::SixteenByNine, "1920x1080"),
        ] {
            let spec = HudRenderSpec {
                aspect,
                resolution: HudResolution::FullHd1080,
                fps: HudFps::Sixty,
                duration_sec: 30,
            };
            let argv =
                build_ffmpeg_mjpeg_pipe_command(&spec, Some("hud_audio.wav")).expect("cmd");
            assert!(argv.contains(&want.to_owned()), "true FullHD {want}");
            assert!(argv.contains(&"60".to_owned()));
            assert!(argv.contains(&"libx264".to_owned()));
        }
    }

    #[test]
    fn video_command_rejects_invalid_duration() {
        let spec = HudRenderSpec {
            duration_sec: 500,
            ..HudRenderSpec::default()
        };
        let err = build_ffmpeg_mjpeg_pipe_command(&spec, None).unwrap_err();
        assert_eq!(err, HudError::DurationOutOfRange { duration_sec: 500 });
    }

    #[test]
    fn frame_size_invariants_hold_for_all_four_combos() {
        let combos = [
            (HudResolution::Hd720, HudAspect::NineBySixteen, (720u32, 1280u32)),
            (HudResolution::Hd720, HudAspect::SixteenByNine, (1280, 720)),
            (HudResolution::FullHd1080, HudAspect::NineBySixteen, (1080, 1920)),
            (HudResolution::FullHd1080, HudAspect::SixteenByNine, (1920, 1080)),
        ];
        for (res, aspect, want) in combos {
            let spec = HudRenderSpec {
                aspect,
                resolution: res,
                fps: HudFps::TwentyFour,
                duration_sec: DEFAULT_DURATION_SEC,
            };
            assert_eq!(spec.frame_size(), want);
            assert!(want.0 * want.1 > 0);
        }
    }

    #[test]
    fn preset_catalog_has_eight_unique_slugs_four_plus_four() {
        let catalog = hud_preset_catalog();
        assert_eq!(catalog.len(), 8);
        let mut slugs: Vec<_> = catalog.iter().map(|p| p.slug.clone()).collect();
        slugs.sort();
        slugs.dedup();
        assert_eq!(slugs.len(), 8, "slugs unique");
        for upstream in ["mwie", "sage-audit", "geo-scope", "tm-second-brain"] {
            let p = find_hud_preset(upstream).expect("upstream preset");
            assert_eq!(p.source, HudPresetSource::Upstream);
        }
        for sovereign in [
            "kia-monopoly-core",
            "mairon-10gate-shield",
            "quant-vibe-terminal",
            "trinity-5model-genome",
        ] {
            let p = find_hud_preset(sovereign).expect("sovereign preset");
            assert_eq!(p.source, HudPresetSource::KiaSovereign);
        }
        assert!(find_hud_preset("nope").is_err());
    }

    #[test]
    fn synthesized_preset_is_deterministic_sanitized_and_accent_aware() {
        let a = synthesize_custom_repo_hud_preset("My_Repo 42!", "causal quant telemetry", None)
            .expect("ok");
        let b = synthesize_custom_repo_hud_preset("My_Repo 42!", "causal quant telemetry", None)
            .expect("ok");
        assert_eq!(a, b, "same repo → same preset");
        assert_eq!(a.slug, "my-repo-42");
        assert_eq!(a.source, HudPresetSource::Synthesized);
        assert_eq!(a.accent_hex.len(), 7, "derived #RRGGBB");
        assert!(a.accent_hex.starts_with('#'));
        assert!(!a.node_labels.is_empty() && a.node_labels.len() <= 4);
        // A valid explicit color is kept; an invalid one is rejected
        let c =
            synthesize_custom_repo_hud_preset("repo-x", "", Some("#0Af")).expect("ok");
        assert_eq!(c.accent_hex, "#00AAFF", "3-digit hex expands");
        assert_eq!(c.tagline, "Custom repo neural telemetry", "default tagline");
        assert_eq!(
            synthesize_custom_repo_hud_preset("repo-x", "", Some("zzz")).unwrap_err(),
            HudError::InvalidAccentHex {
                value: "zzz".to_owned()
            }
        );
        assert_eq!(
            synthesize_custom_repo_hud_preset("   ", "", None).unwrap_err(),
            HudError::EmptyRepoName
        );
    }

    #[test]
    fn defect_ledger_lists_fifteen_unique_cures() {
        assert_eq!(UPSTREAM_DEFECTS_CURED.len(), 15, "all 15 upstream defects");
        let mut ids: Vec<_> = UPSTREAM_DEFECTS_CURED.iter().map(|d| d.0).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), 15, "defect ids unique");
        for (_, defect, cure) in UPSTREAM_DEFECTS_CURED {
            assert!(!defect.is_empty() && !cure.is_empty(), "{defect}");
        }
    }

    #[test]
    fn json_for_script_never_emits_raw_angle_brackets() {
        let value = serde_json::json!({ "title": "</script>&<b>x</b>", "note": "a\u{2028}b" });
        let encoded = json_for_script(&value);
        assert!(!encoded.contains("</script>"));
        assert!(!encoded.contains('&'));
        assert!(!encoded.contains('\u{2028}'));
        let back: serde_json::Value = serde_json::from_str(&encoded).expect("still valid JSON");
        assert_eq!(back["title"], "</script>&<b>x</b>", "payload survives JSON-wise");
    }
}
