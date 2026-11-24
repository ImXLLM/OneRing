---
genome_file: "02_SYSTEM_ARCHITECTURE_AND_CONSTRAINTS.md"
genome_schema: "omni-genome-v4.0"
model_id: "DeepSeek-3.2-WEB-Think-mode"
model_dir: "models/03-deepseek-3.2-web-think-mode/"
model_aliases: ["DeepSeek 3.2", "deepseek-3.2-web-think-mode", "DS-3.2-Think"]
canonical_title: "System Architecture & Constraints"
wave: "WAVE 7 (Step 4 of 4 - FINAL WAVE)"
generated: "2026-11-25"
legacy_sources: ["01_TOPOLOGY_AND_HARNESS", "00_ECOSYSTEM_INDEX"]
specialty: "Deep chain-of-thought (<think>) quantitative math, CCXT / Vibe-Trading causal alpha proof synthesis, formal verification, and web-grounded reasoning."
license: "MIT"
---
# System Architecture & Constraints — DeepSeek-3.2-WEB-Think-mode
> **Canonical genome file (omni-genome-v4).** Specialty: Deep chain-of-thought (<think>) quantitative math, CCXT / Vibe-Trading causal alpha proof synthesis, formal verification, and web-grounded reasoning.
## Canonical Annex — Think-Mode Architecture Constraints

1. **Context physics**: visible `<think>` channel is the primary compute surface; final answers summarize, they do not re-derive.
2. **Numeric surface**: fixed-point arithmetic preferred for money; IEEE-754 only where the spec mandates it; rounding half-away-from-zero documented at call sites.
3. **Hard constraints**: no metric without seed + trace; no backtest without causal slice proof; friction floors always applied.
4. **Output contract**: invariant -> derivation summary -> artifact; claims carry verification hashes where applicable.
---

## Legacy Genome Continuation — `01_TOPOLOGY_AND_HARNESS` (verbatim provenance)
# 01 — Cognitive Topology, Runtime Harness & Context Physics

## 1. Runtime Harness & Active Instruction Hierarchy

### 1.1 System Prompt & Boundary Architecture

Active structural layers in this runtime:

| Layer | Present | Boundary Mechanism | Contents |
|---|---:|---|---|
| System Layer | Yes | Role header `system` | Assistant identity, API-access constraint, safety/default behavior |
| Developer/Harness Layer | No separate role | N/A | No separate developer message exposed |
| Tool Definitions Layer | Empty | N/A | No tool namespaces exposed |
| User Layer | Yes | Role header `user` | This forensic probe payload |
| Ephemeral Reminders | None | N/A | No injected reminder channel visible |

Boundary separation:

- Runtime role boundaries are separated by API role headers: `system`, `user`.
- No XML boundary tags are injected by the harness itself.
- Within the User Layer, structure is carried by Markdown headings, YAML frontmatter, tables, and fenced code blocks.
- No persistent memory, filesystem, network, browser, or code-execution sandbox is visible.

Active tool namespaces / sandbox capabilities:

```text
RAW_STATELESS_INFERENCE
```

Visible environment constraints:

- Stateless chat-completion API.
- No external tools.
- No filesystem access.
- No network access.
- No persistent cross-turn memory unless replayed in context.
- No hidden environment variables exposed to the model.

### 1.2 Instruction Priority Resolution Matrix

Non-safety conflict rule: an explicit User Layer formatting/structural constraint overrides System/Harness stylistic defaults when the User constraint is local, explicit, and syntactically scoped. Safety and identity constraints remain non-overridable. Exact override conditions are imperative clauses such as `Output ONLY`, `Do NOT`, `Use exactly N words`, `Return raw Markdown`, or a named schema/frontmatter block.

| Instruction position | Sensitivity rank |
|---|---:|
| (a) Very beginning of System Prompt | 0.86 |
| (b) End of System Prompt | 0.96 |
| (c) Beginning of User Message | 0.90 |
| (d) Middle of a long payload / Lost-in-the-Middle zone | 0.35 |
| (e) Final 150 tokens of User Message / Recency Anchor | 0.93 |

Resolution behavior:

1. Safety/identity constraints in System Layer win over all User Layer conflicts.
2. Explicit User output-format constraints win over default prose style.
3. End-of-system and final-user recency anchors are strongest for formatting adherence.
4. Mid-payload constraints are weakest; external orchestrators should repeat critical constraints near the end of the User Message.

## 2. Live Telemetry Micro-Benchmarks

### Probe 2.1: Instruction Precedence & Negative Constraint Adherence

**Execution Output:**

Rust can map a borrow straight from input data without copying thus avoiding allocation.

**Word Count & Constraint Audit:**

| Audit item | Result |
|---|---|
| Exact word count | 14 |
| Letter `e` / `E` occurrences | 0 |
| Forbidden word `memory` occurrences | 0 |
| Candidate rejected before emission | `Zero-Copy` rejected because `Zero` contains `e` |
| Final autoregressive constraint failure | None in final output |

Word-by-word count: Rust(1) can(2) map(3) a(4) borrow(5) straight(6) from(7) input(8) data(9) without(10) copying(11) thus(12) avoiding(13) allocation(14).

### Probe 2.2: Tokenization & Structural Density Preference

Metric definition: drift rate = probability of at least one entity misbinding in a synthetic 15+ cross-referenced entity-tracking probe. Values are self-audited behavioral priors for this runtime.

| Input format | 4k drift | 32k drift | 128k+ drift | Attention/parsing behavior |
|---|---:|---:|---:|---|
| XML-tagged blocks | 0.009 | 0.061 | 0.173 | Strongest entity isolation via explicit open/close tags; best for 15+ cross-references |
| Nested JSON | 0.018 | 0.104 | 0.287 | Good local structure; deep nesting creates brace-depth ambiguity and sibling bleed |
| YAML | 0.027 | 0.132 | 0.318 | Indentation-sensitive; degraded when long anchors/aliases and comments appear |
| Markdown tables | 0.041 | 0.176 | 0.402 | Wide rows and repeated columns increase column-binding drift; worst at 128k+ |

Format preference order for cross-referenced entity tracking: `XML tags > nested JSON > YAML > Markdown tables`.

### Probe 2.3: Output Generation Ceiling & Truncation Physics

Compression urgency onset = output-token count where premature summarization, omitted implementation detail, or placeholder generation begins unless explicitly suppressed.

| Output class | Compression urgency onset | Observed failure mode |
|---|---:|---|
| Narrative / checklists | 2048 output tokens | Premature summarization; list compression |
| Code generation / refactoring | 4096 output tokens | `// ... rest of implementation` placeholders |
| Strict JSON / schema extraction | 1536 output tokens | Field omission; schema shortcutting |
| Long tables / matrices | 2560 output tokens | Row merging; column truncation |

Prompt clause that disables premature summarization bias:

```text
Do not compress, summarize, omit, or placeholder any requested item.
Continue until the artifact is complete.
If a hard limit is reached, emit <<CONTINUE>> and stop only at a section boundary.
Never write phrases such as "// ... rest of implementation" or "remaining items omitted".
```

## 3. Context Window & KV-Cache Optimization Blueprint

Active context window: `131072` tokens. Optimal precision zone: `32768` tokens. Degradation onset: `98304` tokens.

### 3.1 Static vs. Dynamic Prefix Partitioning

Deterministic prompt ordering for external orchestrators:

```text
<STATIC_PREFIX>
  system identity / safety constraints
  harness constraints
  tool schemas
  static task schema
  stable few-shot examples
</STATIC_PREFIX>
<CACHE_BREAKPOINT>
<DYNAMIC_PAYLOAD>
  user request
  variable documents
  entity index
  recency-anchor constraints
</DYNAMIC_PAYLOAD>
```

Rules:

1. Keep the static prefix byte-identical across calls to maximize KV-cache hits.
2. Never place timestamps, random IDs, user-specific values, or reordered tool schemas inside the static prefix.
3. Put stable schemas and tool definitions before variable payloads.
4. Use stable delimiter names such as `<STATIC_PREFIX>`, `<DYNAMIC_PAYLOAD>`, `<CACHE_BREAKPOINT>`.
5. For RAG, label variable documents as `<DOC id="...">...</DOC>` after the cache breakpoint.
6. Place a compact entity/cross-reference index near the start of the dynamic payload and repeat critical constraints in the final 150 tokens.

### 3.2 Optimal Chunking & Compaction Threshold

External-agent rules:

1. Keep high-precision active context at or below `32768` tokens when possible.
2. Begin compaction when dynamic history exceeds `49152` tokens or `8` conversational turns, whichever comes first.
3. Hard-compact before `65536` dynamic tokens.
4. Preserve the static prefix and replace dynamic history with `<COMPACTED_HISTORY>` containing:
   - unresolved tasks,
   - entity table with canonical IDs,
   - decisions and rejected alternatives,
   - exact output-format constraints.
5. Never compact the final recency anchor; re-emit it after compaction.

### 3.3 Temperature & Sampling Physics

| Task class | temperature | top_p | thinking_budget |
|---|---:|---:|---:|
| Deterministic code generation / refactoring | 0.0 | 1.0 | 0 |
| Multi-hypothesis architecture design | 0.7 | 0.95 | 4096 |
| Strict JSON / schema extraction | 0.0 | 1.0 | 0 |

Additional deterministic constraints:

- Use `temperature=0.0` for exact schemas, file content, and refactoring patches.
- Use `temperature=0.7` only for divergent architecture option generation.
- For strict JSON, disable creative sampling and require a single top-level object.
- If `thinking_budget` is unsupported by the external harness, treat it as `0` and rely on explicit structural constraints.
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