# 00 — MASTER MODEL COMPARISON MATRIX (Quad-Model Genome, omni-genome-v4)

**Wave:** WAVE 7 (Step 4 of 4 — FINAL WAVE) · **Generated:** 2026-11-25 · **License:** MIT
**Fleet size:** 4 frontier models × 9 canonical DNA files = **36 genome files** (all verified non-empty).

## 1. Sovereign Fleet Inventory

| # | Model ID | Genome dir | Reasoning paradigm | Active harness | Context sweet spot | Cost/speed tier | Fleet roles |
|---|---|---|---|---|---|---|---|
| 01 | **GPT 6-Web** | `models/01-gpt-6-web/` | autoregressive_generation_with_hidden_reasoning | ChatGPT product harness + web_search/web_run toolbelt (browser-backed synthesis) | ~40-60% context window; web evidence freshest inside the first 3 tool hops | `balanced_frontier_web` | artifact_builder, full_stack_shipper, web_synth_primary |
| 02 | **GEMINI 4 ARAGON HIGH** | `models/02-gemini-4-aragon-high/` | long_context_retrieval_fused_multimodal_reasoning | ARAGON HIGH compute tier; full-repo / full-media context fusion | 200k-1M tokens; whole-monorepo ingestion in one pass | `high_context_archival_frontier` | context_archaeologist, multimodal_orchestrator, architectural_verifier |
| 03 | **DeepSeek-3.2-WEB-Think-mode** | `models/03-deepseek-3.2-web-think-mode/` | explicit_chain_of_thought_with_think_blocks | DeepSeek web harness, Think-mode (visible <think> reasoning channel) + web search | dense numeric/proof workloads; think budget scales with proof depth | `reasoning_density_value` | quant_proof_synthesizer, formal_verifier, numeric_auditor |
| 04 | **Cluade-sonnet-5.5-extra** | `models/04-claude-sonnet-5.5-extra/` | reflective_single_pass_with_self_verification_loops | agentic CLI/file-system harness; tool-first, patch-minimal edits | 40-50% context; compact over long tasks preserves surgical precision | `balanced_workhorse_audit` | rust_surgeon, smart_contract_auditor, bilingual_copywriter |

**Aliases:** `GPT 6-Web` ≡ `gpt-6-web` (supersedes legacy `gpt-6-standard-free`); `Cluade-sonnet-5.5-extra` ≡ `Claude-sonnet-5.5-extra` (both exact identifiers accepted everywhere in the fleet router).

## 2. Specialty Champion Matrix (20 workload classes)

| # | Workload class | Primary champion | Verifier / reviewer | Zero-cost fallback |
|---|---|---|---|---|
| 1 | Real-time web-grounded synthesis | **GPT 6-Web** | DeepSeek-3.2 (web cross-check) | ARAGON (cached context) |
| 2 | Frame-1 interactive UI artifact (390px zero-overflow) | **GPT 6-Web** | Claude-5.5 (layout audit) | — |
| 3 | Rapid full-stack product generation | **GPT 6-Web** | Claude-5.5 (lint/tests) | — |
| 4 | 1M-token whole-repo archaeology | **GEMINI 4 ARAGON HIGH** | grep/tree tooling | — |
| 5 | 9-modality studio orchestration | **GEMINI 4 ARAGON HIGH** | per-modality schema gates | — |
| 6 | High-context architectural verification | **GEMINI 4 ARAGON HIGH** | Claude-5.5 (surgical read) | — |
| 7 | Quantitative math / probability derivations | **DeepSeek-3.2-WEB-Think-mode** | property tests | — |
| 8 | CCXT/Vibe-Trading causal alpha proof | **DeepSeek-3.2-WEB-Think-mode** | CausalBacktestEngine + claim ledger | — |
| 9 | Formal verification & invariants | **DeepSeek-3.2-WEB-Think-mode** | cargo test / proptest | — |
| 10 | Surgical Rust systems engineering | **Claude-sonnet-5.5-extra** | clippy -D warnings + tests | — |
| 11 | Zero-warning pedantic refactoring | **Claude-sonnet-5.5-extra** | CI matrix | — |
| 12 | Smart-contract security auditing | **Claude-sonnet-5.5-extra** | GoPlus/TokenSniffer checklists | — |
| 13 | Bilingual EN/FA copywriting | **Claude-sonnet-5.5-extra** | human spot-check | — |
| 14 | DB schema / WAL / ledger design | Claude-5.5 (build) | ARAGON (cross-invariant verify) | — |
| 15 | Concurrency / Heisenbug RCA | Claude-5.5 | DeepSeek-3.2 (proof) | — |
| 16 | Defensive security / sandbox design | Claude-5.5 | ARAGON (whole-tree attack surface) | — |
| 17 | Marketing / launch copy (Product Hunt, CMC) | GPT 6-Web | Claude-5.5 (FA mirror) | — |
| 18 | Multi-step shell/tool orchestration | Claude-5.5 | logs + CI | — |
| 19 | Long-horizon planning / roadmap | ARAGON HIGH | Mairon quant gate | — |
| 20 | Trilingual DNA documentation synthesis | ARAGON HIGH | Claude-5.5 (precision pass) | — |

> Scores are qualitative role assignments from the forensic card set, not measured pass rates; replace with shared-eval pass rates (n ≥ 30 per class) when available. The same model never fills both Primary and Verifier on the same row.

## 3. Trinity Wiring (3-Avatar Sovereign Loop)

`Alpha (ingest/ideate)` → `Logic-Prime (compile/verify)` → `Mairon (quant gate/judge)` — each avatar resolves to one or two genome members per task domain through `QuadModelGenomeRouter::route_task_to_quad_genome` in `crates/hermes/src/sovereign_monopoly_engine.rs`.

## 4. Canonical File Schema (per model directory)

| File | Dimension |
|---|---|
| `01_IDENTITY_AND_PERSONA_GENOME.md` | identity card, specialty doctrine, persona contract |
| `02_SYSTEM_ARCHITECTURE_AND_CONSTRAINTS.md` | context physics, tool surface, hard constraints, output contract |
| `03_REASONING_AND_DECISION_TREE.md` | decision tree, escalation rules |
| `04_TOOL_ORCHESTRATION_AND_AGENTIC_LOOP.md` | agentic loop, loop-exit criteria |
| `05_CODING_AND_ENGINEERING_DOCTRINE.md` | language/stack doctrine, verification gates |
| `06_UI_UX_AND_ARTIFACT_CRAFT.md` | artifact standards, 390px consortium rule |
| `07_SAFETY_GUARDRAILS_AND_EDGE_CASES.md` | refusals, injection defense, edge cases |
| `08_FAILURE_MODES_AND_BLINDSPOTS.md` | symptom → root cause → patch table |
| `09_DISTILLED_CLONE_SYSTEM_PROMPT.md` | deployable distilled clone prompt |

Legacy v1/v2 workspaces (gpt-6-standard-free, gemini-4-aragon-high v1 profiles, compiler scripts) are preserved verbatim under [`legacy/`](legacy/). The forensic probe methodology is preserved under [`methodology/`](methodology/), and Kia's creator-subconscious genome under [`sovereign/`](sovereign/).
