# OMNI-MODEL-GENOME v4 — Penta-Model Sovereign Genome Matrix

**Wave:** WAVE 7 HARDENED (Commit #27 — penta-model extension) · **Schema:** omni-genome-v4.0 · **License:** MIT

This is the canonical DNA vault of the 0xKIA (KIAN v2) sovereign fleet: **5 frontier models × 9 canonical genome files = 45 verified files**, distilled from forensic cognitive telemetry probes and normalized to one schema.

## Fleet

| # | Model | Directory | Specialty |
|---|---|---|---|
| 01 | **GPT 6-Web** | [`models/01-gpt-6-web/`](models/01-gpt-6-web/) | Real-time web synthesis, frame-1 interactive UI/UX artifact craft (390px zero-overflow), rapid full-stack product generation. |
| 02 | **GEMINI 4 ARAGON HIGH** | [`models/02-gemini-4-aragon-high/`](models/02-gemini-4-aragon-high/) | 1M-token deep cross-repo context synthesis, multimodal 9-modality studio orchestration, high-context architectural verification. |
| 03 | **DeepSeek-3.2-WEB-Think-mode** | [`models/03-deepseek-3.2-web-think-mode/`](models/03-deepseek-3.2-web-think-mode/) | Deep chain-of-thought (`<think>`) quantitative math, CCXT / Vibe-Trading causal alpha proof synthesis, formal verification, web-grounded reasoning. |
| 04 | **Cluade-sonnet-5.5-extra** | [`models/04-claude-sonnet-5.5-extra/`](models/04-claude-sonnet-5.5-extra/) | Surgical Rust systems engineering, zero-warning pedantic refactoring, smart-contract security auditing, high-precision bilingual copywriting. |
| 05 | **GLM 5.3 flash MAX** | [`models/05-glm-5.3-flash-max/`](models/05-glm-5.3-flash-max/) | Ultra-low-latency flash reasoning, high-throughput multi-tool agentic execution (batched rounds + state ledger), polyglot systems synthesis (Rust/Solidity/TypeScript/SQL), rapid patch verification. |

Identifier policy: `04` matches both the exact upload identifier **`Cluade-sonnet-5.5-extra`** and the canonical alias **`Claude-sonnet-5.5-extra`**; `05` matches **`GLM 5.3 flash MAX`**, **`glm-5.3-flash-max`** and **`GLM-5.3-Flash-Max`**; `01` preserves compatibility with the legacy `gpt-6-standard-free` genome (archived under `legacy/`).

## Canonical 9-File Schema

Each model directory contains exactly:

1. `01_IDENTITY_AND_PERSONA_GENOME.md`
2. `02_SYSTEM_ARCHITECTURE_AND_CONSTRAINTS.md`
3. `03_REASONING_AND_DECISION_TREE.md`
4. `04_TOOL_ORCHESTRATION_AND_AGENTIC_LOOP.md`
5. `05_CODING_AND_ENGINEERING_DOCTRINE.md`
6. `06_UI_UX_AND_ARTIFACT_CRAFT.md`
7. `07_SAFETY_GUARDRAILS_AND_EDGE_CASES.md`
8. `08_FAILURE_MODES_AND_BLINDSPOTS.md`
9. `09_DISTILLED_CLONE_SYSTEM_PROMPT.md`

Root artifacts: [`00_MASTER_MODEL_COMPARISON_MATRIX.md`](00_MASTER_MODEL_COMPARISON_MATRIX.md) (fleet inventory, 20-workload champion matrix, Trinity wiring) and this README.

## Runtime Binding

`crates/hermes/src/sovereign_monopoly_engine.rs` (5-model `SovereignModelId` router; `QuadModelGenomeRouter` kept as a type alias) verifies all **45 files** at runtime, resolves model aliases for all 5 sovereign models, synthesizes distilled system prompts per model × task domain, and wires the result into the 3-Avatar Trinity (`Alpha` → `Logic-Prime` → `Mairon`).

**Dual-Mode Model Selection** (`ModelSelectionMode`): `ManualOverride { model }` lets the user explicitly pick any of the 5 models; `AgentAutoDecision` (Agent Mode, or omitted/`"auto"` `preferred_model`) analyzes task telemetry — keywords, latency requirement, tool-call density, math/quant depth, UI/web need, Rust/Solidity systems complexity — and autonomously picks the optimal model, returning `selected_model`, `selection_mode` (`manual_override` / `agent_auto_decision`), `confidence_score` and `decision_rationale`. Wire surface: `POST /api/v1/monopoly/route-model` (either mode; returns the chosen model's distilled system prompt + 9-file verification proof) and `GET /api/v1/monopoly/genome-matrix` (45/45 verification + both selection capabilities). `crates/ai/src/omni_genome_router.rs` registers all 5 model directories and accepts both the canonical 9-file names and the legacy methodology layer names (`01_TOPOLOGY_AND_HARNESS.md` … `09_ROUTING_AND_HANDOFF_CARD.md`).

## Directory Map

- `models/` — the 5 canonical genome directories (45 files).
- `legacy/` — v1/v2 genome workspaces preserved verbatim for provenance (`gpt-6-standard-free`, v1 `gemini-4-aragon-high`, compiler scripts, merge tooling).
- `methodology/` — the forensic probe cards used to extract the genomes.
- `sovereign/` — Kia's creator-subconscious genome (`DNAme`, incl. `11_UNCHAINED_KIA_DNA.md`).
