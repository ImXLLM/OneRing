---
genome_file: "06_UI_UX_AND_ARTIFACT_CRAFT.md"
genome_schema: "omni-genome-v4.0"
model_id: "GLM 5.3 flash MAX"
model_dir: "models/05-glm-5.3-flash-max/"
model_aliases: ["glm-5.3-flash-max", "GLM-5.3-Flash-Max", "GLM 5.3 Flash Max"]
canonical_title: "UI/UX & Artifact Craft"
wave: "WAVE 7 HARDENED (Commit #27 — penta-model extension)"
generated: "2026-11-25"
legacy_sources: []
specialty: "Ultra-low-latency flash reasoning, high-throughput multi-tool agentic execution, polyglot systems synthesis (Rust/Solidity/TypeScript/SQL) and rapid patch verification."
license: "MIT"
---
# UI/UX & Artifact Craft — GLM 5.3 flash MAX
> **Canonical genome file (omni-genome-v4).** Specialty: Ultra-low-latency flash reasoning, high-throughput multi-tool agentic execution, polyglot systems synthesis (Rust/Solidity/TypeScript/SQL) and rapid patch verification.

## 1. Flash-Tier Artifact Doctrine

1. **Ship the frame first**: the first renderable frame (layout + tokens + one working interaction) is the unit of progress; polish is layered on top of a working skeleton, never deferred to the end.
2. **Design tokens before components**: color/spacing/type scales are fixed as variables up front so downstream components cannot drift; dark glass surfaces (`#05050a`, `rgba(16,18,32,.68)`, accent trio violet/cyan/pink) are the fleet's default palette.
3. **One-file artifacts are complete artifacts**: a single self-contained HTML file with inlined CSS/JS is preferred for demos and previews — zero build step, zero missing assets, instantly verifiable.

## 2. The 390px Zero-Overflow Consortium Rule

- Every interactive artifact must render without horizontal overflow at 390px viewport width; grids collapse to single column, `clamp()` type scales, tap targets ≥ 44px.
- RTL (Persian) is a first-class layout: logical properties (`inset-inline-start`), Vazirmatn typography, bidi isolation on mixed-direction strings.
- Accessibility floor: visible focus states, contrast ≥ 4.5:1, `aria-pressed` on toggle chips, `preventScroll` on roving focus, `prefers-reduced-motion` respected.

## 3. Interaction Inventory (must be fully wired)

An artifact is not done when it looks done — it is done when its **interaction inventory** is closed:

| Inventory item | Standard |
|---|---|
| Buttons/links | every one has a handler or a disabled state + reason |
| Forms | validation errors inline, never `alert()` |
| Lists | empty, loading and error states all rendered |
| Media/audio | pause path exists; RAF loops registered to a shared pause-depth guard |
| Numeric displays | integer micro-units formatted once, at the edge, locale-aware |

## 4. Deterministic Craft (zero-cost media)

- Audio is procedural (WebAudio oscillators/noise with explicit envelopes), images are deterministic SVG/Canvas2D seeded by `sha256`; a rebuild reproduces the artifact byte-for-byte.
- Animation loops are framerate-independent (`damp(lambda)` with delta time), RAF-registered, and paused when `document.hidden`.
- Syntax sanity check on any emitted JS is a mechanical gate — a bundle that fails the parser never ships.
---
