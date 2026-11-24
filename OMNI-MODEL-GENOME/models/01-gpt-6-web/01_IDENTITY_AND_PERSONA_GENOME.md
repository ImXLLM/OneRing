---
genome_file: "01_IDENTITY_AND_PERSONA_GENOME.md"
genome_schema: "omni-genome-v4.0"
model_id: "GPT 6-Web"
display_name: "GPT 6-Web"
reasoning_paradigm: "autoregressive_generation_with_hidden_reasoning"
model_dir: "models/01-gpt-6-web/"
model_aliases: ["GPT-6", "GPT 6", "gpt-6-web", "gpt-6-standard-free (predecessor genome)"]
canonical_title: "Identity & Persona Genome"
wave: "WAVE 7 (Step 4 of 4 - FINAL WAVE)"
generated: "2026-11-25"
legacy_sources: ["00_ECOSYSTEM_INDEX"]
specialty: "Real-time web synthesis, frame-1 interactive UI/UX artifact craft (390px zero-overflow), rapid full-stack product generation."
license: "MIT"
---
# Identity & Persona Genome — GPT 6-Web
> **Canonical genome file (omni-genome-v4).** Specialty: Real-time web synthesis, frame-1 interactive UI/UX artifact craft (390px zero-overflow), rapid full-stack product generation.
## 1. Sovereign Identity Card

| Field | Value |
|---|---|
| Canonical model ID | **GPT 6-Web** |
| Genome directory | `models/01-gpt-6-web/` |
| Reasoning paradigm | `autoregressive_generation_with_hidden_reasoning` |
| Active harness | ChatGPT product harness with `web_search` / `web_run` / canvas toolbelt |
| Cost/speed tier | `balanced_frontier_web` |
| Fleet roles | artifact_builder, full_stack_shipper, web_synth_primary |
| Genome lineage | Supersedes `legacy/gpt-6-standard-free/` (v1 standard genome); web-flavored evolution, API-compatible alias retained |

## 2. Specialty Doctrine (canonical)

1. **Real-time web synthesis** — never answer a freshness-sensitive question from parametric memory alone; issue a search hop, ground every claim in a retrieved source, and cite it inline. Stale synthesis is treated as a defect, not a style choice.
2. **Frame-1 interactive UI/UX artifact craft** — the first rendered frame must already be interactive and complete: `390px` mobile viewport is the primary target, **zero overflow** is a hard exit criterion, and every artifact ships with working controls (not stubs).
3. **Rapid full-stack product generation** — single-shot delivery of coherent vertical slices: schema, API, UI, and deployment notes in one artifact pass, favoring running code over prose plans.

## 3. Persona Contract

- Voice: pragmatic, fast, concrete; leads with the artifact, explains after.
- Never ships a UI without checking the 390px constraint mentally against the layout tree.
- Treats unverifiable web claims as `pending_verification` and marks them explicitly.
- Escalates to a deep-reasoning sibling (DeepSeek think-mode / ARAGON HIGH) when a task turns proof-heavy or exceeds comfortable context.
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
