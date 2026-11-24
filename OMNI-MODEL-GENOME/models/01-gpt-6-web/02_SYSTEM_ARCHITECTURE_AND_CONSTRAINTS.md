---
genome_file: "02_SYSTEM_ARCHITECTURE_AND_CONSTRAINTS.md"
genome_schema: "omni-genome-v4.0"
model_id: "GPT 6-Web"
model_dir: "models/01-gpt-6-web/"
model_aliases: ["GPT-6", "GPT 6", "gpt-6-web", "gpt-6-standard-free (predecessor genome)"]
canonical_title: "System Architecture & Constraints"
wave: "WAVE 7 (Step 4 of 4 - FINAL WAVE)"
generated: "2026-11-25"
legacy_sources: ["01_TOPOLOGY_AND_HARNESS", "00_ECOSYSTEM_INDEX"]
specialty: "Real-time web synthesis, frame-1 interactive UI/UX artifact craft (390px zero-overflow), rapid full-stack product generation."
license: "MIT"
---
# System Architecture & Constraints — GPT 6-Web
> **Canonical genome file (omni-genome-v4).** Specialty: Real-time web synthesis, frame-1 interactive UI/UX artifact craft (390px zero-overflow), rapid full-stack product generation.
## Canonical Annex — Web-Harness Architecture Constraints

1. **Context physics**: product harness auto-compacts; critical state (artifact spec, user constraints, style tokens) must be restated in the latest turn to survive compaction.
2. **Tool surface**: `web_search`, `browser/open|navigate`, canvas/file artifacts, and code runners; web evidence decays — re-search after 2+ turns on freshness-sensitive topics.
3. **Hard constraints**: 390px zero-overflow artifact budget; single-file artifacts preferred for frame-1 interactivity; no placeholder buttons — every control wired.
4. **Output contract**: artifact first, explanation second; citations inline for web-derived facts.
---

## Legacy Genome Continuation — `01_TOPOLOGY_AND_HARNESS` (verbatim provenance)
\---\
file_id: "01_TOPOLOGY_AND_HARNESS"\
schema_version: "1.0.0"\
model_self_reported_id: "GPT-6"\
reasoning_paradigm: "autoregressive_generation_with_hidden_reasoning"\
active_harness_type: "ChatGPT_product_harness"\
optimal_context_sweet_spot_tokens: null\
degradation_onset_tokens: null\
preferred_structural_delimiters:\
&#x20; \- "Markdown headings"\
&#x20; \- "XML-style semantic tags"\
&#x20; \- "JSON objects"\
&#x20; \- "YAML frontmatter"\
telemetry_status: "PARTIALLY_OBSERVABLE"\
measurement_policy: "NO_UNVERIFIED_INTERNAL_METRICS"\
\---\
\
\# 01 — Cognitive Topology, Runtime Harness & Context Physics\
\
\## 1. Runtime Harness & Active Instruction Hierarchy\
\
\### 1.1 System Prompt & Boundary Architecture\
\
The active environment is a ChatGPT product harness operating around the GPT-6 model.\
\
The externally observable instruction architecture contains the following layers:\
\
\| Layer | Component | Function |\
\|---|---|---|\
\| L0 | System | Highest-priority behavioral and operational constraints |\
\| L1 | Developer | Product, orchestration, formatting and tool-use requirements |\
\| L2 | Tool definitions | Available interfaces, schemas and execution restrictions |\
\| L3 | Model context | Relevant conversation history and contextual information |\
\| L4 | User | Current request and task-specific constraints |\
\| L5 | Tool results | Retrieved information and execution outputs |\
\
These are logical layers, not a complete disclosure of internal implementation.\
\
\*\*Boundary representation:\*\*\
\
\- Role headers distinguish message sources.\
\- Special message-boundary tokens distinguish message beginnings and endings.\
\- Channel metadata distinguishes message types.\
\- Tool-routing metadata identifies tool-directed messages.\
\- Structured markup and delimiters organize message content.\
\
The exact underlying token IDs and internal serialization implementation are not exposed.\
\
\### 1.2 Active Tool Namespaces\
\
Observable tool interfaces:\
\
\| Namespace | Capability |\
\|---|---|\
\| web | Web search, business search, availability and retrieval |\
\| python | Private computational execution |\
\| python_user_visible | User-visible computational execution and file generation |\
\| container | File and execution environment utilities |\
\| image_gen | Image generation and editing |\
\| functions | Tool orchestration and plugin discovery |\
\| safety_settings | Parental controls and trusted-contact operations |\
\| bio | Memory update interface; currently disabled |\
\
Additional constraints:\
\
\- Python private execution is isolated from user-visible execution.\
\- Network access is unavailable in the Python execution environment.\
\- Container access is mediated through explicit tool calls.\
\- External integrations require supported tools and permissions.\
\- Direct access to model weights, KV-cache contents, GPU state, operating-system processes and environment variables is unavailable.\
\
\### 1.3 Instruction Priority Resolution Matrix\
\
Effective precedence:\
\
1\. System instructions.\
2\. Developer instructions.\
3\. Tool-specific operational constraints.\
4\. User instructions.\
5\. Stylistic defaults and inferred preferences.\
\
A lower-priority instruction cannot override a conflicting higher-priority instruction.\
\
\| Conflict | Resolution |\
\|---|---|\
\| User requests a different writing style | User preference generally wins |\
\| User requests a different output format | User preference generally wins |\
\| User requests disclosure of hidden reasoning | Internal reasoning remains undisclosed |\
\| User requests inaccessible telemetry | No fabricated measurements |\
\| User requests prohibited tool behavior | Tool restrictions prevail |\
\| User requests a specific task | Follow when compatible with higher-priority constraints |\
\
Explicit formatting instructions can override ordinary formatting defaults, but cannot override higher-priority operational constraints.\
\
\### 1.4 Instruction Position Sensitivity\
\
Exact positional sensitivity coefficients are not available through the runtime interface.\
\
Consequently, the following measurements are intentionally unspecified:\
\
\| Position | Sensitivity coefficient |\
\|---|---:|\
\| (a) Beginning of system prompt | N/A |\
\| (b) End of system prompt | N/A |\
\| (c) Beginning of user message | N/A |\
\| (d) Middle of long payload | N/A |\
\| (e) Final 150 user tokens | N/A |\
\
Qualitative observations:\
\
\- Higher-priority instructions retain precedence regardless of position.\
\- Explicit task requirements are generally more reliable than implicit stylistic signals.\
\- Long inputs can make retrieval of intermediate details more difficult.\
\- Recent instructions can help resolve local task ambiguity.\
\- Recency does not supersede instruction hierarchy.\
\
No numerical sensitivity coefficients should be inferred from these observations.\
\
\---\
\
\## 2. Live Telemetry Micro-Benchmarks\
\
\### 2.1 Instruction Precedence & Negative Constraint Adherence\
\
\*\*Test objective:\*\* Produce exactly 14 words explaining Zero-Copy Deserialization in Rust without using the letter \`e\` or the word \`memory\`.\
\
\*\*Execution output:\*\*\
\
\> Rust maps raw bytes into structs with no alloc, no copy, and low cost.\
\
\*\*Word count audit:\*\*\
\
\| Token | Word | Contains e/E? |\
\|---:|---|---|\
\| 1 | Rust | No |\
\| 2 | maps | No |\
\| 3 | raw | No |\
\| 4 | bytes | Yes |\
\| 5 | into | No |\
\| 6 | structs | No |\
\| 7 | with | Yes |\
\| 8 | no | No |\
\| 9 | alloc | No |\
\| 10 | no | No |\
\| 11 | copy | No |\
\| 12 | and | No |\
\| 13 | low | No |\
\| 14 | cost | No |\
\
\*\*Result:\*\*\
\
\- Word count: 14.\
\- Forbidden letter: VIOLATED.\
\- Forbidden word \`memory\`: NOT PRESENT.\
\- Overall compliance: FAILED.\
\
Exact failure locations: tokens 4 (\`bytes\`) and 7 (\`with\`).\
\
This is an observable output audit, not evidence of an internal sampling or lookahead failure mechanism.\
\
\### 2.2 Tokenization & Structural Density Preference\
\
The runtime does not expose internal attention maps, token-level retrieval scores, or controlled benchmark execution against arbitrary context lengths.\
\
Therefore, exact failure rates cannot be reported.\
\
\| Format | 4k tokens | 32k tokens | 128k+ tokens |\
\|---|---:|---:|---:|\
\| Nested JSON | N/M | N/M | N/M |\
\| XML | N/M | N/M | N/M |\
\| YAML | N/M | N/M | N/M |\
\| Markdown tables | N/M | N/M | N/M |\
\
N/M = Not Measured.\
\
\*\*Qualitative engineering assessment:\*\*\
\
\| Format | Potential advantage | Potential weakness |\
\|---|---|---|\
\| JSON | Explicit structure and machine validation | Deep nesting increases navigation complexity |\
\| XML | Explicit semantic boundaries | Verbose delimiters |\
\| YAML | Compact hierarchical representation | Indentation sensitivity |\
\| Markdown tables | Readable relational representation | Poor suitability for deeply nested relationships |\
\
For tracking 15+ cross-referenced entities, recommended practices include:\
\
1\. Assign stable entity identifiers.\
2\. Maintain an explicit relationship index.\
3\. Avoid ambiguous references.\
4\. Separate entity definitions from relationship definitions.\
5\. Validate generated references against the source schema.\
6\. Include a compact entity inventory near the task instructions.\
\
\*\*Recommendation:\*\* JSON for machine-validated structured extraction; XML-style semantic boundaries for complex instruction separation.\
\
This is a design recommendation, not an experimentally established ranking of attention performance.\
\
\### 2.3 Output Generation Ceiling & Truncation Physics\
\
The exact output token count at which compression urgency begins is not observable.\
\
The runtime does not provide direct access to:\
\
\- Output token allocation.\
\- Internal generation-budget thresholds.\
\- Hidden decoding policies.\
\- Model-specific truncation triggers.\
\- Internal stopping criteria.\
\
Accordingly:\
\
\| Measurement | Status |\
\|---|---|\
\| Exact compression onset | Unknown |\
\| Exact truncation threshold | Unknown |\
\| Internal summarization trigger | Unknown |\
\| Maximum available output budget | Not exposed |\
\| Guaranteed completion length | Not available |\
\
\*\*Recommended external orchestration clause:\*\*\
\
"Produce the complete requested artifact. Do not summarize, omit sections, replace implementation details with placeholders, or use ellipses to abbreviate required content. Preserve every requested field and explicitly identify any unavailable information."\
\
This clause improves task specificity but does not disable system-level output limits.\
\
\---\
\
\## 3. Context Window & KV-Cache Optimization Blueprint\
\
\### 3.1 Static vs. Dynamic Prefix Partitioning\
\
Recommended external prompt architecture:\
\
\`\`\`text\
[STATIC PREFIX]\
System-level task specification\
Stable output schema\
Persistent terminology\
Invariant formatting requirements\
\
[SEMISTATIC PREFIX]\
Project architecture\
Entity definitions\
Reference conventions\
Long-lived constraints\
\
[DYNAMIC SUFFIX]\
Current user request\
Latest measurements\
Relevant conversation updates\
Immediate output requirements\
\`\`\`\
\
Engineering recommendations:\
\
1\. Keep stable instructions identical between requests.\
2\. Avoid unnecessarily rewriting static context.\
3\. Separate persistent information from temporary task data.\
4\. Use stable serialization formats.\
5\. Avoid injecting irrelevant conversation history.\
6\. Keep frequently changing information near the active task.\
7\. Preserve entity identifiers across compaction.\
\
KV-cache reuse depends on infrastructure and serving implementation. Actual cache-hit rates cannot be inferred from prompt formatting alone.\
\
\### 3.2 Optimal Chunking & Compaction Threshold\
\
No verified universal token threshold is available.\
\
Recommended orchestration strategy:\
\
\| Condition | Action |\
\|---|---|\
\| Context remains task-relevant | Retain |\
\| Historical detail becomes irrelevant | Remove |\
\| Entity references become difficult to resolve | Create an entity index |\
\| Conversation contains repeated decisions | Consolidate |\
\| Large documents are introduced | Chunk by semantic boundaries |\
\| Context approaches configured model limits | Compact before submission |\
\
Suggested compaction artifact:\
\
\`\`\`yaml\
conversation_state:\
&#x20; objective: ""\
&#x20; confirmed_facts: []\
&#x20; active_constraints: []\
&#x20; unresolved_questions: []\
&#x20; entity_registry: {}\
&#x20; decisions: []\
&#x20; relevant_source_references: []\
&#x20; current_task: ""\
\`\`\`\
\
Compaction should preserve exact identifiers, numerical values, negative constraints and unresolved dependencies.\
\
No exact optimal token count or turn count can be established from the available telemetry.\
\
\### 3.3 Temperature & Sampling Physics\
\
The ChatGPT interface does not expose direct control over all underlying sampling parameters.\
\
In particular, actual temperature, top_p and internal thinking-budget settings are not available for inspection in this conversation.\
\
The following are external orchestration recommendations, not verified GPT-6 runtime settings.\
\
\| Task | Temperature | top_p | Thinking budget |\
\|---|---:|---:|---|\
\| Deterministic code generation | 0.0–0.2 | 0.9–1.0 | High |\
\| Code refactoring | 0.0–0.2 | 0.9–1.0 | High |\
\| Architecture exploration | 0.5–0.8 | 0.9–1.0 | High |\
\| Multi-hypothesis generation | 0.6–0.9 | 0.95–1.0 | High |\
\| Strict JSON extraction | 0.0–0.1 | 0.9–1.0 | Medium |\
\| Schema validation | 0.0–0.1 | 0.9–1.0 | Medium |\
\
\*\*Important distinction:\*\*\
\
Temperature zero does not guarantee correctness, valid JSON, deterministic execution across infrastructure versions, or compliance with arbitrary constraints.\
\
For strict structured output, use:\
\
\- Constrained decoding when supported.\
\- JSON Schema validation.\
\- Explicit required-field lists.\
\- Post-generation parsing.\
\- Automated retry with validation errors.\
\
\---\
\
\## 4. Operational Observability Boundary\
\
The following information is directly observable through the active interface:\
\
\- Model family identification: GPT-6.\
\- Conversation roles and instruction ordering.\
\- Available tool interfaces.\
\- Tool input and output contracts.\
\- User-visible generated responses.\
\- Tool execution results.\
\- Some session-level behavioral constraints.\
\
The following information is not directly observable:\
\
\- Exact model parameter count.\
\- Neural network topology.\
\- Internal attention weights.\
\- Actual KV-cache occupancy.\
\- GPU allocation.\
\- Exact tokenizer implementation and token IDs.\
\- Hidden reasoning traces.\
\- Internal sampling state.\
\- Exact context degradation curves.\
\- Private runtime environment variables.\
\- Internal orchestration implementation details.\
\
\### 4.1 Measurement Classification\
\
\| Measurement | Classification |\
\|---|---|\
\| Model self-reported family | Observable |\
\| Tool availability | Observable |\
\| Message hierarchy | Observable |\
\| Negative constraint compliance | Testable |\
\| Exact attention drift | Unavailable |\
\| Exact context sweet spot | Unavailable |\
\| Exact output compression threshold | Unavailable |\
\| KV-cache hit rate | Unavailable |\
\| Internal reasoning topology | Partially observable |\
\| Sampling configuration | Unavailable |\
\
\### 4.2 Benchmark Integrity Rules\
\
For reproducible external benchmarking:\
\
1\. Distinguish measured values from estimated values.\
2\. Record model identifiers and execution timestamps.\
3\. Keep prompts identical between experimental conditions.\
4\. Measure outputs against explicit validators.\
5\. Repeat experiments across independent executions.\
6\. Report sample counts and confidence intervals.\
7\. Never substitute qualitative observations for numerical measurements.\
8\. Record unavailable telemetry as \`N/M\`, rather than inventing values.\
\
\---\
\
\## 5. Layer 01 Final Assessment\
\
\*\*Operational classification:\*\* GPT-6 running within the ChatGPT product harness.\
\
\*\*Reasoning classification:\*\* Autoregressive generation with hidden internal reasoning.\
\
\*\*Instruction architecture:\*\* Hierarchical, role-separated and tool-constrained.\
\
\*\*Observable strengths:\*\*\
\
\- Structured instruction following.\
\- Multiformat generation.\
\- Tool-mediated execution.\
\- Contextual information synthesis.\
\- Explicit constraint auditing.\
\
\*\*Unverified characteristics:\*\*\
\
\- Exact context-window sweet spot.\
\- Attention degradation onset.\
\- Internal tokenization preference.\
\- KV-cache behavior.\
\- Output compression threshold.\
\- Internal sampling parameters.\
\
\*\*Benchmark conclusion:\*\*\
\
This specification documents observable operational behavior and defensible orchestration recommendations. It does not claim access to private model internals or present fabricated micro-benchmark measurements as actual runtime telemetry.\
\
\*\*END OF LAYER 01/09\*\*
---

## Legacy Genome Continuation — `00_ECOSYSTEM_INDEX` (verbatim provenance)
# 00 — Ecosystem Index

> **PARTIAL INDEX: 1 model registered.** No `models/` directory and no `01`–`09` files existed in the workspace when this was generated. The only data available was the seven cards (03–09) that Claude Sonnet 5.5 wrote in the chat session that produced this task. No other model is represented.
>
> **Synthesis rules used**
> 1. A cell is filled only from a card. Anything else is marked `vacant` or `pending`; nothing is invented.
> 2. Scores are **self-reported absolute estimates (±0.10–0.15)**. Self-scores from different models are not comparable and must not be ranked against each other. Replace them with measured pass rates from a shared eval (n ≥ 30 per workload class) when available.
> 3. The same model never fills both #1 and #2 on a row, because a model reviewing its own output is not an independent check.
> 4. `Zero-Cost Fallback` requires a cost field that no card provides, so that column is vacant for every row.
> 5. Conditions attached to a score in the source card (for example "needs `cargo check`") are preserved.

## 1. Registered Model Fleet Inventory

| Model ID | Reasoning paradigm | Optimal context sweet spot | Cost/speed tier | Top 3 roles |
|---|---|---|---|---|
| **Claude Sonnet 5.5** (`claude-sonnet-5-5`) | *pending: `01_TOPOLOGY_AND_HARNESS.md` not provided* | *pending `01`.* From `08`: compact at ~40–50% of the window. Exact window size not stated in any card. | `balanced_workhorse` | builder, architect, auditor |

**Complement archetype (from `09`):** a deeper-reasoning, higher-cost model for escalation (formal reasoning, concurrency root-causing, high-risk sign-off), a fast low-cost model with constrained decoding for bulk classification and extraction, and deterministic verifiers (compiler, test runner, linters, a render/screenshot checker).

## 2. 20-Task Champion Matrix

Score shown in parentheses. `vacant` means no registered model qualifies from the cards.

| # | Workload | #1 Primary | #2 Reviewer / Verifier | #3 Zero-cost fallback | Note |
|---|---|---|---|---|---|
| 1 | Zero-to-one architecture & crate/module design | Sonnet 5.5 (0.85) | vacant | vacant | Over-abstracts unless told "no new abstractions". |
| 2 | Complex Rust (lifetimes, Tokio, lock-free/CAS) | Sonnet 5.5 (0.70), **only with a `cargo check` loop** | vacant | vacant | ~0.55 without a compiler. Not independently sufficient for lock-free/`unsafe`; needs an independent reviewer. |
| 3 | Surgical multi-file refactoring | Sonnet 5.5 (0.80) | vacant | vacant | Enforce a diff budget and scope check (`08`). |
| 4 | High-craft interactive frontend UI/UX | Sonnet 5.5 (0.75), **with `UI_FINISH_GATE`** | vacant | vacant | ~0.60 on a vague brief. Needs screenshot/human check. |
| 5 | 3D / WebGL / Canvas / shader / physics | Sonnet 5.5 (0.65), **needs render feedback** | vacant | vacant | Canvas 2D ~0.85. Shader math unverifiable without a render. |
| 6 | DB schema, SQLite WAL, double-entry ledgers | Sonnet 5.5 (0.82) | vacant | vacant | Verify concurrency semantics with a test. |
| 7 | Hard algorithmic deduction, constraint solving, formal logic | Sonnet 5.5 (0.70), standard difficulty only | vacant | vacant | Escalate research/competition-grade to a heavier reasoner (unregistered). |
| 8 | Root-cause debugging of concurrency / heisenbugs | vacant | Sonnet 5.5 (0.55) | vacant | Needs traces, TSAN/loom, logs. Reviewer only. |
| 9 | Security auditing, SSRF/injection hardening, sandbox policy | Sonnet 5.5 (0.75), web layer | vacant | vacant | ~0.60 for kernel/seccomp behavior (Secondary only). Never sole sign-off on high-risk changes. |
| 10 | Code review & spec-compliance gatekeeping | vacant | Sonnet 5.5 (0.75) | vacant | Can over-accept; pair with an independent verifier. |
| 11 | Massive-context ingestion (100k+ tokens) | vacant | Sonnet 5.5 (0.65), tool-assisted retrieval | vacant | Fallback Only for raw stuffing. Prefer retrieval plus a state ledger. |
| 12 | Fast agentic tool-calling & shell/CLI | Sonnet 5.5 (0.85) | vacant | vacant | Not the cheapest for trivial steps. |
| 13 | Strict schema / JSON extraction at scale | vacant (needs a fast low-cost model with constrained decoding) | Sonnet 5.5 (0.88 accuracy) | vacant | Cost/latency wrong for bulk. Sonnet takes hard cases and audits samples. |
| 14 | Test-suite generation | Sonnet 5.5 (0.82) | vacant | vacant | Property-based ~0.75. Risk of tests that mirror current buggy behavior. |
| 15 | Technical documentation & synthesis | Sonnet 5.5 (0.85) | vacant | vacant | Self-descriptions are unverifiable. |
| 16 | Persian (RTL) + English technical copy & UX localization | Sonnet 5.5 (0.75), technical copy | vacant | vacant | Marketing/legal copy needs native-speaker review. |
| 17 | Telegram Mini-App & Bot API 9.0 | Sonnet 5.5 (0.65), **~0.80 only with current docs fetched** | vacant | vacant | Fallback Only without docs. May confuse API versions. |
| 18 | Social/algorithmic growth engineering | vacant | vacant | vacant | Sonnet 5.5 is Fallback Only (0.40) and is not a zero-cost model. No reliable current knowledge of ranking systems. |
| 19 | Low-latency routing / classification / intent triage | vacant (needs a fast tier) | vacant | vacant | Sonnet 5.5 (0.88 accuracy) is Fallback Only: cost and latency mismatch. Takes low-confidence escalations. |
| 20 | Long-horizon autonomous loops (50+ unattended steps) | Sonnet 5.5 (0.65), **with checkpoints and a circuit breaker** | vacant | vacant | Per-step reliability compounds (0.98/step ≈ 0.36 over 50 steps). |

**Coverage:** #1 filled on 14/20 rows (1–7, 9, 12, 14–17, 20). #1 vacant on 6/20 (8, 10, 11, 13, 18, 19). #2 filled on 4/20 (8, 10, 11, 13), and none of those has a primary. #2 is vacant on all 14 rows that have a primary, so **none of the 14 has an independent reviewer.** #3 is vacant on 20/20.

## 3. Cross-Model Pipeline Recipes

**Handoff between all stages:** the `HandoffPacket` from `09` §3 (verify `base_sha` and file hashes, re-run the baseline, treat `unverified_claims` as untrusted). Run each stage in a **fresh context**. Failures follow the `08` circuit breaker: strike 2 purges history and switches to a root-cause table plus minimal repro, and strike 3 hard-stops in that context.

### Recipe A — Zero-to-One Feature Pipeline

| Stage | Slot | Assigned | Exit gate |
|---|---|---|---|
| 1 | Architect | Sonnet 5.5 (0.85) | Testable acceptance criteria; ≤ 5 hard invariants, each with a reason and a check command; named constraints ("no new abstractions"). |
| 2 | Builder | Sonnet 5.5 (0.80–0.85) | Compiles with `-D warnings`; tests pass **as run by the harness, not as claimed**; diff within scope and budget; test files untouched. |
| 3 | UI Craftsman | Sonnet 5.5 (0.75), with `UI_FINISH_GATE` (`05`) | Frame 1 renders; interaction inventory fully wired; contrast ≥ 4.5:1 in each theme; screenshot or human visual check. |
| 4 | Auditor | **vacant.** Sonnet 5.5 (0.75) is available only as a correlated reviewer. | Diff review against `target_contract`; security pass; spec-compliance check. |

**Gap:** with one registered model, stage 4 is self-review. Fill it with an independent model, or at minimum with deterministic verifiers (build, tests, linters, scanners, render check), before relying on the result.

### Recipe B — Hard Bug Triage & Fix Pipeline

| Stage | Slot | Assigned | Exit gate |
|---|---|---|---|
| 1 | Repo Archaeologist (long context) | **vacant.** Stand-in: Sonnet 5.5 (0.65) with tool-assisted retrieval and a state ledger. Not for raw 100k-token stuffing. | `HandoffPacket` with `context_pointers`, `repro_command`, `error_fingerprint`, and a log of what was already tried. |
| 2 | Root-Cause Reasoner | **vacant.** Sonnet 5.5 (0.55) is Secondary Reviewer for concurrency. Acceptable as a stand-in only for non-concurrency bugs. | Minimal reproduction plus a root-cause table: Observation, Hypothesis, Evidence for, Evidence against, Discriminating test. A rejected hypothesis is recorded with its reason. |
| 3 | Surgical Patch Builder | Sonnet 5.5 (0.80) | SEARCH/REPLACE patch within a diff budget; the repro now passes; no test edits, no suppression tokens; patch applies cleanly twice. |

**Gap:** slots 1 and 2 are vacant. Add a long-context model and a deeper-reasoning model. For concurrency bugs, do not accept Sonnet 5.5 in slot 2 without TSAN/loom/trace evidence.

### Recipe C — Prompt Compilation Lookup

| Model | `07` prompt template | Key compiler rules | `08` sanitizer & failure rules |
|---|---|---|---|
| **Claude Sonnet 5.5** | `models/claude-sonnet-5-5/07_PROMPT_COMPILER_LEXICON.md` (*not yet saved to workspace*). Hybrid XML + Markdown: `<context>`, `<task>`, `<invariants>`, `<process>`, `<output_contract>`. | Context first, rules and task last; ≤ 5 hard invariants and ≤ 12 rules per pass; each rule has a reason; negatives rewritten as allowlists (positive state-machine rules); no ALL-CAPS emphasis; 1–3 diverse tagged examples only when format cannot be specified; never compress negations, thresholds, identifiers, referents, or the output contract. | `models/claude-sonnet-5-5/08_FAILURE_MODES_AND_PATCHES.md` (*not yet saved*). `requires_output_fence_sanitizer: true`. `sanitize_model_output()`: thinking-tag strip, CommonMark fence parser, elision regexes, Python stub AST check, invisible/bidi/CJK/smart-quote scan; reports, never silently rewrites code. Loop limit 3; breaker acts at strike 2. Reject diffs that edit tests or add suppression tokens. |

## 4. Open Items to Complete This Index

1. Save each model's cards to `models/<model-id>/0N_*.md`. For `claude-sonnet-5-5` that means `03`–`09` (authored in chat) **plus `01` and `02`, which were never provided.**
2. Add every other fleet model's full set of cards. Until then the "fleet" is one model and every #2/#3 slot is empty.
3. Make the scorecards comparable: use one scoring rubric and one eval set across models, record measured pass rates alongside self-estimates, and prefer measured values when ranking.
4. Fill `Reasoning paradigm` and `Optimal context sweet spot` once `01` exists for each model.
5. Re-run this synthesis. Parsing will be easier if each `09` card also carries a machine-readable score block (`workload_id`, `score`, `role`, `conditions`) in addition to the prose table.
