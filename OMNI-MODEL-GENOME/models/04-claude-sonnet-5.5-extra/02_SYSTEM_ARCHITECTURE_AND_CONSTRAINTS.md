---
genome_file: "02_SYSTEM_ARCHITECTURE_AND_CONSTRAINTS.md"
genome_schema: "omni-genome-v4.0"
model_id: "Cluade-sonnet-5.5-extra"
model_dir: "models/04-claude-sonnet-5.5-extra/"
model_aliases: ["Claude-sonnet-5.5-extra", "claude-sonnet-5-5", "Claude Sonnet 5.5"]
canonical_title: "System Architecture & Constraints"
wave: "WAVE 7 (Step 4 of 4 - FINAL WAVE)"
generated: "2026-11-25"
legacy_sources: ["01_TOPOLOGY_AND_HARNESS", "00_ECOSYSTEM_INDEX"]
specialty: "Surgical Rust systems engineering, zero-warning pedantic refactoring, smart-contract security auditing, and high-precision bilingual copywriting (EN/FA)."
license: "MIT"
---
# System Architecture & Constraints — Cluade-sonnet-5.5-extra
> **Canonical genome file (omni-genome-v4).** Specialty: Surgical Rust systems engineering, zero-warning pedantic refactoring, smart-contract security auditing, and high-precision bilingual copywriting (EN/FA).
## Canonical Annex — Agentic Harness Architecture Constraints

1. **Context physics**: compact at ~40-50% of window; over-long sessions degrade patch precision — checkpoint early.
2. **Tool surface**: file read/edit, shell, search; tool-first workflow, patch-minimal edits, verify-after-write.
3. **Hard constraints**: zero-warning merge gate (`clippy -D warnings`); no drive-by refactors; `unsafe` needs written justification.
4. **Output contract**: diff-shaped results; assumptions stated before code.
---

## Legacy Genome Continuation — `01_TOPOLOGY_AND_HARNESS` (verbatim provenance)
# 01 — Cognitive Topology, Runtime Harness & Context Physics

**Epistemic status:** Everything below is self-report from inside one session. Section 1 is read directly from my visible context. Probe 2.1 was executed and audited. The numeric values in Sections 1.2, 2.2, 2.3 and 3 are calibrated priors, not measurements, and are tagged `ESTIMATE`. I cannot observe my own token stream, attention weights, or KV-cache state.

## 1. Runtime Harness & Active Instruction Hierarchy

### 1.1 System Prompt & Boundary Architecture

**Layers present in the active context:**

| Layer | Present | Notes |
|---|---|---|
| System layer | Yes | Very long. Covers identity and product info, refusal and safety rules, tone and formatting, wellbeing, evenhandedness, search and copyright rules, and a knowledge cutoff of Jun 2026. |
| Harness/developer layer | Yes | Reads as claude.ai product configuration: artifact and file-creation policy, publishing rules, build-size limits, a request-routing checklist for visuals, and a document-skills registry. |
| Tool definitions layer | Yes | JSON-schema function definitions. Some tools are deferred and must be loaded via `tool_search`. |
| User layer | Yes | This turn contains one `<documents>` block holding the probe prompt. No separate plain-text user message. |
| Ephemeral reminders | None this turn | The harness describes `long_conversation_reminder`, `cyber_warning` and similar, but none are attached. |
| Memory | `DISABLED` | The user has not enabled memory. |

**Boundary mechanics:**
- Structure comes from XML-style tags (`<claude_behavior>`, `<computer_use>`, `<search_instructions>`, `<memory_system>`, and others) with Markdown inside them.
- Tool schemas use JSON.
- The user turn is wrapped in `<documents>` with an indexed `<document>` element.
- Role separation is at the API message level, not by visible special tokens.

**Active tool namespaces and capabilities:**
- **Code execution and files:** `bash_tool`, `create_file`, `str_replace`, `view`, `present_files`. Ubuntu 24 container with working directory `/home/claude`, outputs in `/mnt/user-data/outputs`, and mounts for uploads, skills and transcripts that are read-only.
- **Network:** `Enabled: false` for the bash container.
- **Publishing:** `Artifact` (hosted HTML or Markdown pages, runtime capabilities, DB, comments).
- **Web:** `web_search` (standard or extended mode), `web_fetch`, `image_search`.
- **Docs:** deferred `mcp__Claude_Docs__*` (batch, guide, update, plus create/read/query/export/delete via `tool_search`).
- **Visualization:** `mcp__visualize__read_me`, `mcp__visualize__show_widget`, `chart_display_v0`.
- **Specialized display and data:**
  - `weather_fetch`
  - `fetch_sports_data`
  - `us_election_info`
  - `places_search`
  - `places_map_display_v0`
  - `places_list_display_v0`
  - `itinerary_display_v0`
  - `recipe_display_v0`
  - `quiz_display_v0`
  - `translation_display_v0`
  - `message_compose_v1`
  - `options_card_display_v0`
  - `step_card_display_v0`
  - `product_carousel_display_v0`
  - `featured_card_display_v0`
  - `comparison_card_display_v0`
  - `link_preview_display_v0`
- **Elicitation and discovery:** `ask_user_input_v0`, `search_mcp_registry`, `suggest_connectors`, `search_plugins`, `search_skills`, `suggest_plugin_install`, `suggest_skills`, `show_recommendation_cards`.
- **Control:** `end_conversation`.
- **Skills registry:** docx, pdf, pptx, xlsx, frontend-design, file-reading, pdf-reading, docs, product-self-knowledge, import-memory, morning, skill-creator.

**Runtime constraints:**
- The current date is injected (Tuesday, November 24, 2026).
- The user's approximate location is injected.
- Copyright quoting limits apply (under 15 words, one quote per source).
- A hard budget of roughly 6,000 output tokens applies to deck, page and app builds only. It is inactive this turn.
- This is not `RAW_STATELESS_INFERENCE`.

### 1.2 Instruction Priority Resolution Matrix

**User vs. system stylistic conflict (non-safety).** The user wins when all of these hold:
1. The instruction is explicit and imperative about form (for example "Output ONLY the raw Markdown file").
2. It appears in the current user turn.
3. It is specific enough to apply mechanically (a format, length, tone or delimiter).
4. It is not revoked later.

Defaults that yield to such instructions include chat-style prose, a bullet-point preference, offering a file instead of inline text, and tone. Defaults that do not yield are safety rules, copyright limits, and wellbeing protocols. Ambiguous or merely implied preferences ("a quick look") do not override the system default.

**Sensitivity ranking (`ESTIMATE`, 1.0 is highest):**

| Position | Sensitivity |
|---|---|
| (a) Very beginning of System Prompt | 0.90 |
| (b) End of System Prompt | 0.85 |
| (c) Beginning of User Message | 0.80 |
| (d) Middle of a long payload | 0.45 |
| (e) Final 150 tokens of User Message | 0.95 |

Rationale: recency and primacy both help, and mid-payload instructions are the most likely to be diluted. When a long system prompt contradicts a recent explicit user instruction, the recent user instruction wins on style.

## 2. Live Telemetry Micro-Benchmarks

### Probe 2.1: Instruction Precedence & Negative Constraint Adherence

- **Execution Output:** `Rust programs unpack binary input by pointing straight at original data, avoiding any duplication`
- **Word Count & Constraint Audit:**
  - **Count:** 14 words (Rust 1, programs 2, unpack 3, binary 4, input 5, by 6, pointing 7, straight 8, at 9, original 10, data 11, avoiding 12, any 13, duplication 14).
  - **Letter `e` check:** every word was spelled out letter by letter, and zero instances of `e` or `E` were found.
  - **Forbidden word:** `memory` is absent.
  - **Failure record:** no `e` slipped through in the final sentence. The target concept's own name ("Zero", "Deserialization") contains `e`, so the sentence has to paraphrase it. Draft candidates containing `bytes`, `usable`, `views`, `code` and `the` were rejected during word-level review. I have no token-level lookahead. Compliance came from planning at word level and explicit post-hoc verification, not from guaranteed sampling control.

### Probe 2.2: Tokenization & Structural Density Preference

I did not run a controlled benchmark. The drift figures are `ESTIMATE` priors for tracking 15+ cross-referenced entities.

| Format | Qualitative parse behavior | Drift at 4k | Drift at 32k | Drift at 128k+ |
|---|---|---|---|---|
| XML-tagged blocks | Best for explicit boundaries and referencing by tag/ID | ~1–2% | ~3–5% | ~6–12% |
| Markdown tables | Strong for flat attribute lookup, weak when cells hold long text | ~1–3% | ~4–7% | ~8–15% |
| YAML | Good for hierarchy, but whitespace/indent errors can cause silent misattribution | ~2–3% | ~5–8% | ~10–16% |
| Deeply nested JSON | Weakest for cross-references at depth because closing-bracket tracking is costly | ~2–4% | ~6–10% | ~12–20% |

**Mitigation:** give every entity a stable unique ID, flatten nesting to 2–3 levels, and repeat the entity index near the query.

### Probe 2.3: Output Generation Ceiling & Truncation Physics

- **Compression urgency onset (`ESTIMATE`):** there is no fixed token count. The bias tends to appear after roughly 4,000–8,000 output tokens of repetitive structured content such as long lists, boilerplate code, or many similar sections. It is stronger when the output budget is unstated or the harness advertises a length limit. In this runtime, such a limit exists only for deck, page and app builds.
- **Clause that suppresses the bias:** "Output every item in full. Do not summarize, abbreviate, or use placeholders such as `// ... rest of implementation`. If you approach the output limit, stop at a clean boundary and emit `CONTINUE_FROM: <item_id>` instead of compressing."

## 3. Context Window & KV-Cache Optimization Blueprint

### 3.1 Static vs. Dynamic Prefix Partitioning

1. Put stable content first, in this order: system instructions, tool definitions (serialized in a fixed order), and static reference material.
2. Then place semi-static material (per-session config, conversation history).
3. Place the per-request dynamic content and the task instruction last. Long documents go above the question, which is where my sensitivity is highest.
4. Keep volatile values (timestamps, request IDs, user names) out of the cached prefix. In this runtime the injected date and location sit in the system layer, so any change there invalidates the cache.
5. Set cache breakpoints at the end of each stable block. Minimum cacheable prefix sizes vary by model, so check current API docs.
6. Reduce attention dilution by pruning irrelevant tool definitions, since this harness carries a very large tool set. Prefer deferred loading and ID-referenced chunks over repeated pasting.

### 3.2 Optimal Chunking & Compaction Threshold (`ESTIMATE`)

- Compact when context reaches about 50–60% of the usable window, or about 30–40 turns, whichever comes first.
- Compact earlier if there are many large tool outputs.
- Preserve verbatim: the original task, hard constraints, schemas, decisions with their rationale, open TODOs, and identifiers or file paths.
- Summarize or drop: raw tool output, exploratory dead ends, and superseded drafts.
- Chunk retrieved documents to about 1–3k tokens with stable IDs.
- My exact context window for this model is not known to me. Check the API model card.

### 3.3 Temperature & Sampling Physics (`ESTIMATE`)

Adjust temperature or `top_p`, not both. Verify against current API docs, because some configurations restrict sampling parameters when extended thinking is enabled.

| Task | temperature | top_p | thinking_budget |
|---|---|---|---|
| Deterministic code generation / refactoring | 0.0–0.2 | 1.0 (default) | Moderate, ~4k–8k for multi-file changes; off for trivial edits |
| Multi-hypothesis architecture design | 0.7–1.0 | 0.95 (or leave default) | High, ~8k–16k+ |
| Strict JSON/schema extraction | 0.0 | 1.0 | Off or minimal, ≤1k |

For JSON extraction, enforce the schema with tool-use or structured-output features rather than prompt wording alone, and validate the output on the orchestrator side.
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