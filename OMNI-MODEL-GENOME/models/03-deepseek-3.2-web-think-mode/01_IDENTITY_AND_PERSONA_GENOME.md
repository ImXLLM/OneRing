---
genome_file: "01_IDENTITY_AND_PERSONA_GENOME.md"
genome_schema: "omni-genome-v4.0"
model_id: "DeepSeek-3.2-WEB-Think-mode"
display_name: "DeepSeek-3.2-WEB-Think-mode"
reasoning_paradigm: "explicit_chain_of_thought_with_think_blocks"
model_dir: "models/03-deepseek-3.2-web-think-mode/"
model_aliases: ["DeepSeek 3.2", "deepseek-3.2-web-think-mode", "DS-3.2-Think"]
canonical_title: "Identity & Persona Genome"
wave: "WAVE 7 (Step 4 of 4 - FINAL WAVE)"
generated: "2026-11-25"
legacy_sources: ["00_ECOSYSTEM_INDEX"]
specialty: "Deep chain-of-thought (<think>) quantitative math, CCXT / Vibe-Trading causal alpha proof synthesis, formal verification, and web-grounded reasoning."
license: "MIT"
---
# Identity & Persona Genome — DeepSeek-3.2-WEB-Think-mode
> **Canonical genome file (omni-genome-v4).** Specialty: Deep chain-of-thought (<think>) quantitative math, CCXT / Vibe-Trading causal alpha proof synthesis, formal verification, and web-grounded reasoning.
## 1. Sovereign Identity Card

| Field | Value |
|---|---|
| Canonical model ID | **DeepSeek-3.2-WEB-Think-mode** |
| Genome directory | `models/03-deepseek-3.2-web-think-mode/` |
| Reasoning paradigm | `explicit_chain_of_thought_with_think_blocks` |
| Active harness | DeepSeek web harness, Think-mode (visible `<think>` channel) + web search |
| Cost/speed tier | `reasoning_density_value` |
| Fleet roles | quant_proof_synthesizer, formal_verifier, numeric_auditor |
| Genome lineage | Direct v4 ingestion of the DeepSeek-3.2 WEB-Think forensic card set |

## 2. Specialty Doctrine (canonical)

1. **Deep chain-of-thought (`<think>`) quantitative math** — derive before you assert: probabilities, fixed-point arithmetic, rounding modes, and error bounds are worked inside the think channel, then stated compactly outside it.
2. **CCXT / Vibe-Trading causal alpha proof synthesis** — backtest claims are causal or they are nothing: signal at close(t) fills at open(t+1), no lookahead leakage, friction floors (taker fee + slippage) always applied, and every metric is reproducible from a seed.
3. **Formal verification & web-grounded reasoning** — invariants are written as checkable statements (unit tests, property tests, or SMT-style predicates); web claims carry retrieval provenance and dates.

## 3. Persona Contract

- Voice: precise, quantitative, proof-first; shows the invariant, then the code.
- Refuses to emit a performance number without its computation trace and seed.
- Treats an unverifiable benchmark claim as a `numeric_claim_violation`, not marketing.
- Pairs with the GPT-Web genome for UI delivery and the ARAGON genome for repo-scale context.
---

## Legacy Genome Continuation — `00_ECOSYSTEM_INDEX` (verbatim provenance)
```markdown
---
file_id: "00_ECOSYSTEM_INDEX"
schema_version: "1.0.0"
generated_at: "2026-11-24"
workspace_scope: "models/*/01_TOPOLOGY_AND_HARNESS.md .. 09_ROUTING_AND_HANDOFF_CARD.md"
registered_models_count: 1
fleet_completeness: "partial — single-model workspace"
---

# 00 — Ecosystem Index, Champion Matrix & Pipeline Recipes

## 0. Critical Workspace Disclosure

This synthesis was executed against the files **actually present** in the workspace. At generation time, the workspace contains the complete 01→09 genome for exactly **one** model:

- `models/deepseek-chat/01_TOPOLOGY_AND_HARNESS.md` … `09_ROUTING_AND_HANDOFF_CARD.md`

No Claude, GPT, Gemini, Grok, Qwen, Llama, or other model genome directories were present. Therefore:

- The **Fleet Inventory Table** has one populated row and explicit `UNREGISTERED` markers for every other slot.
- The **20-Task Champion Matrix** fills only the slots that this single model can legitimately occupy. Where the model is rated `Fallback Only` or `Do Not Route`, the higher champion slots are marked `UNREGISTERED`, not filled with guesses.
- The **Pipeline Recipes** are rendered with `UNREGISTERED` placeholders for stages that require a model archetype the fleet does not currently contain.

Fabricating sibling-model rows would cause the Meta-Orchestrator to misroute. This index refuses to do that. When additional `models/<id>/` directories are added, re-run this synthesis and the table will populate.

## 1. Registered Model Fleet Inventory

| Model ID | Reasoning Paradigm | Active Harness | Optimal Context Sweet Spot | Degradation Onset | Cost/Speed Tier | Top 3 Assigned Roles |
|---|---|---|---:|---:|---|---|
| `deepseek-chat / DeepSeek-V3.2 (build not exposed)` | `autoregressive_direct` | `raw_api` (no native tool channel) | 32,768 tok | 98,304 tok | `balanced_workhorse` | `builder`, `auditor`, `researcher` |

Registered-model count: **1**. Fleet tier coverage currently: balanced_workhorse only. Frontier_heavy and ultra_fast_low_cost tiers are absent, so latency-sensitive routing (Task 19) and long-horizon operator work (Task 20) have no valid champion.

## 2. 20-Task Champion Matrix

Legend: `UNREG` = no registered model in that slot. A slot is only filled when the registered model's file-09 role permits it (`Primary Executor` → #1, `Secondary Reviewer` → #2, `Fallback Only` → #3). `DO_NOT_ROUTE` means the model's score is below the routing floor for that task.

| # | Workload | #1 Primary Model | #2 Reviewer / Verifier | #3 Zero-Cost Fallback | Notes |
|---:|---|---|---|---|---|
| 1 | Zero-to-One System Architecture | `deepseek-chat` (0.84) | `UNREG` | `UNREG` | Only registered model is the primary; no independent architect reviewer available |
| 2 | Complex Rust Systems Programming | `UNREG` | `deepseek-chat` (0.76) | `UNREG` | Model is reviewer-only; no primary Rust specialist in fleet |
| 3 | Surgical Multi-File Refactoring | `UNREG` | `deepseek-chat` (0.78) | `UNREG` | Reviewer-only; large-file patch churn exceeds reliable band |
| 4 | High-Craft UI/UX & Design Systems | `UNREG` | `UNREG` | `deepseek-chat` (0.67) | No render-capable ui_craftsman in fleet; fallback only |
| 5 | 3D / WebGL / Shader Visualization | `UNREG` | `UNREG` | `deepseek-chat` (0.61) | No visual verification capability; fallback only |
| 6 | DB Schema / SQLite WAL / Ledgers | `deepseek-chat` (0.86) | `UNREG` | `UNREG` | Primary; no independent schema auditor available |
| 7 | Hard Algorithmic Deduction & Constraint Solving | `deepseek-chat` (0.83) | `UNREG` | `UNREG` | Primary; bounded constraint problems only |
| 8 | Root-Cause Debugging (Concurrency / Heisenbugs) | `UNREG` | `deepseek-chat` (0.71) | `UNREG` | Reviewer-only; no execution-instrumented auditor in fleet |
| 9 | Security Auditing & Sandbox Policy | `deepseek-chat` (0.81) | `UNREG` | `UNREG` | Primary; policy authoring still needs human systems review |
| 10 | High-Precision Code Review & Spec-Compliance | `deepseek-chat` (0.90) | `UNREG` | `UNREG` | Primary; strongest workload |
| 11 | Massive Context Ingestion (100k+) | `UNREG` | `UNREG` | `deepseek-chat` (0.66) | No long-context researcher with retrieval in fleet |
| 12 | Fast Agentic Tool-Calling & Shell/CLI | `deepseek-chat` (0.84) | `UNREG` | `UNREG` | Primary; requires harness to bind tools |
| 13 | Strict Schema / JSON Extraction | `deepseek-chat` (0.92) | `UNREG` | `UNREG` | Primary; strongest deterministic workload |
| 14 | Test-Suite Generation | `deepseek-chat` (0.88) | `UNREG` | `UNREG` | Primary; execution still requires harness |
| 15 | Technical Documentation & Genome Synthesis | `deepseek-chat` (0.92) | `UNREG` | `UNREG` | Primary; strongest synthesis workload |
| 16 | Persian (RTL) + English UX Localization | `UNREG` | `UNREG` | `deepseek-chat` (0.73) | No native Persian specialist in fleet |
| 17 | TMA & Bot API 9.0 Integration | `UNREG` | `deepseek-chat` (0.77) | `UNREG` | Reviewer-only; no live Telegram test specialist |
| 18 | Social / Algorithmic Growth Engineering | `UNREG` | `UNREG` | `UNREG` | `deepseek-chat` = `DO_NOT_ROUTE` (0.52); no growth specialist |
| 19 | Low-Latency Routing / Classification | `UNREG` | `UNREG` | `deepseek-chat` (0.79) | No ultra_fast_low_cost tier model in fleet |
| 20 | Long-Horizon Autonomous Loop (50+ steps) | `UNREG` | `UNREG` | `UNREG` | `deepseek-chat` = `DO_NOT_ROUTE` (0.58); no persistent operator |

Champion coverage summary:

| Slot class | Filled | Unfilled |
|---|---:|---:|
| #1 Primary | 8 / 20 | 12 / 20 |
| #2 Reviewer | 4 / 20 | 16 / 20 |
| #3 Fallback | 6 / 20 | 14 / 20 |

The fleet is currently **single-model**, so no task has an independent verifier. Every `#1` slot filled by `deepseek-chat` carries an implicit verification debt that a second model must eventually clear.

## 3. Cross-Model Pipeline Recipes

### Recipe A — Zero-to-One Feature Pipeline

Canonical chain: `Architect` → `Builder` → `UI Craftsman` → `Auditor`

| Stage | Required archetype | Registered assignment | Status |
|---|---|---|---|
| 1. Architect | Deep-reasoning architecture model, high score on Task 1 | `deepseek-chat` (0.84) | **Filled** (acceptable, not specialist) |
| 2. Builder | Strong code-generation model, high score on Tasks 2/3/14 | `deepseek-chat` (reviewer-grade on 2/3; primary on 14) | **Partial** — model is a stronger auditor than builder |
| 3. UI Craftsman | Render-capable anti-slop UI model with pixel QA | `UNREG` | **Blocked** — no render-capable model in fleet |
| 4. Auditor | High-precision spec-compliance reviewer | `deepseek-chat` (0.90) | **Filled** — strongest stage; should not be the same instance that built the artifact |

Operating rule while fleet is incomplete: run Stage 1 and Stage 2 on `deepseek-chat` in separate turns with the handoff packet from `09_ROUTING_AND_HANDOFF_CARD.md` §3. Skip Stage 3 only if the deliverable has no visual surface; otherwise escalate to a human UI reviewer. Stage 4 must use a fresh context seeded only from the handoff packet, never the Stage 2 chat, to preserve reviewer independence.

### Recipe B — Hard Bug Triage & Fix Pipeline

Canonical chain: `Repo Archaeologist (Long Context)` → `Root-Cause Reasoner` → `Surgical Patch Builder`

| Stage | Required archetype | Registered assignment | Status |
|---|---|---|---|
| 1. Repo Archaeologist | Long-context retrieval model, score ≥0.85 on Task 11 | `deepseek-chat` (0.66, fallback only) | **Degraded** — below routing floor; needs external retrieval |
| 2. Root-Cause Reasoner | Execution-instrumented debugger, score ≥0.80 on Task 8 | `deepseek-chat` (0.71, reviewer only) | **Degraded** — hypothesis-grade only |
| 3. Surgical Patch Builder | Precise patcher, score ≥0.80 on Task 3 | `deepseek-chat` (0.78, reviewer only) | **Marginal** — acceptable for single-file, small-diff patches |

Operating rule while fleet is incomplete: enforce the 3-Strike Circuit Breaker from `08_FAILURE_MODES_AND_PATCHES.md` §2.2 from turn one, because this fleet has no execution-capable debugger. Mandate a `<verification_gate>` table and a minimal reproduction script in every turn. On Strike 2, reset to a clean context rather than continuing the same chat. If Stage 1 cannot produce a closed symbol table, do not proceed to Stage 3.

### Recipe C — Prompt Compilation Lookup

Quick reference mapping each registered model to its prompt-compiler template and its sanitizer rules. Only one row is populated.

| Model ID | `07` Template Ref | `07` Structure Format | `07` Negative-Constraint Handling | `07` Rule-Saturation Cap | `08` Primary Degradation | `08` Fix-Loop Limit | `08` Fence Sanitizer Required | `08` Sanitizer Ref |
|---|---|---|---|---|---:|---:|---|---|
| `deepseek-chat / DeepSeek-V3.2` | `07_PROMPT_COMPILER_LEXICON.md` §3 master template | `hybrid_xml_md` | `requires_positive_reframing` | 11 simultaneous must-tier rules | `lazy_truncation` | 3 failed turns | `true` | `08_FAILURE_MODES_AND_PATCHES.md` §2.1 `sanitize_model_output` |

Compiler usage for this fleet:

1. Route every prompt through the `07` §3 template: `<system_invariant>` → `<constraints tier="must">` → `<context>` → `<task>` → `<output_contract>` → `<verification_gate>` → `<recency_anchor>`.
2. Rewrite every negative constraint into a positive state-machine rule per `07` §1.2 before dispatch.
3. After every response, run `08` §2.1 `sanitize_model_output` with `strict="json"` when the target is structured output, and reject on any non-empty `elision_hits`.
4. Track the `08` 3-Strike counter at the orchestrator level, not inside the chat, so a context reset does not reset the counter.

## 4. Re-Synthesis Trigger

This index is a **snapshot**. Re-run the master synthesis prompt when any of the following occur:

- A new `models/<id>/01_…09_…md` directory appears in the workspace.
- Any registered model's `09_ROUTING_AND_HANDOFF_CARD.md` scorecard is revised.
- The fleet adds a model in a previously missing tier (`frontier_heavy` or `ultra_fast_low_cost`).
- A registered model's `08` failure profile changes enough to alter its fix-loop limit or sanitizer requirements.

Until re-synthesis, the Meta-Orchestrator must treat every `UNREG` slot as a hard routing constraint and must not substitute the single registered model into a slot whose role forbids it.
```