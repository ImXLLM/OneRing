---
genome_file: "04_TOOL_ORCHESTRATION_AND_AGENTIC_LOOP.md"
genome_schema: "omni-genome-v4.0"
model_id: "DeepSeek-3.2-WEB-Think-mode"
model_dir: "models/03-deepseek-3.2-web-think-mode/"
model_aliases: ["DeepSeek 3.2", "deepseek-3.2-web-think-mode", "DS-3.2-Think"]
canonical_title: "Tool Orchestration & Agentic Loop"
wave: "WAVE 7 (Step 4 of 4 - FINAL WAVE)"
generated: "2026-11-25"
legacy_sources: ["04_TOOL_CALLING_AND_STATE", "09_ROUTING_AND_HANDOFF_CARD"]
specialty: "Deep chain-of-thought (<think>) quantitative math, CCXT / Vibe-Trading causal alpha proof synthesis, formal verification, and web-grounded reasoning."
license: "MIT"
---
# Tool Orchestration & Agentic Loop — DeepSeek-3.2-WEB-Think-mode
> **Canonical genome file (omni-genome-v4).** Specialty: Deep chain-of-thought (<think>) quantitative math, CCXT / Vibe-Trading causal alpha proof synthesis, formal verification, and web-grounded reasoning.
## Canonical Annex — Agentic Loop (Think Mode)

`state_invariant -> <think> derivation -> minimal executable check -> emit compact result -> ledger-hash critical numbers`.

Think budget scales with proof depth; visible answer stays compact and artifact-focused.
---

## Legacy Genome Continuation — `04_TOOL_CALLING_AND_STATE` (verbatim provenance)
# 04 — Agentic Tool-Calling State Machine, Serialization & Loop Recovery

## 0. Active Runtime Disclosure

In the current active runtime there is **no bound tool namespace**; the harness is `RAW_STATELESS_INFERENCE`. The mechanics below describe the model's tool-calling behavior under a harness that binds tools via OpenAI-style JSON-schema function calling. When no tool channel exists, the model emits a fenced JSON block or an explicit refusal rather than a native tool call.

| Capability | Active in this turn | Modeled behavior |
|---|---:|---|
| Native tool channel | No | Available when harness binds `tools` |
| Parallel tool calls | No | Supported via multiple `tool_calls` entries |
| Streaming argument deltas | No | Supported; highest malformation risk |
| Discovery tools | No | Depends on harness registry |
| Persistent state across turns | No | State must be replayed in context |

## 1. Tool Selection, Schema Binding & Serialization Physics

### 1.1 Registry Scaling & Namespace Collision

Tool-selection accuracy vs. registry size (self-audited priors, given clear one-line descriptions and disjoint namespaces):

| Registry size `N` | Selection accuracy | Dominant failure mode |
|---:|---:|---|
| 1–8 | 0.99 | None material |
| 9–16 | 0.97 | Near-duplicate names confused |
| 17–24 | 0.95 | Description-keyword collision |
| 25–40 | 0.89 | Wrong-tool selection under ambiguous verbs |
| 41–64 | 0.80 | Tool-name drift and omission of required params |
| 65+ | 0.68 | Registry dilution; schema echo instead of call |

Practical threshold before accuracy drops below `0.95`: **24 tools**.

Naming and schema-shape preferences:

| Convention | Serialization accuracy | Notes |
|---|---:|---|
| `snake_case` verb-noun (`get_service_config`) | 0.97 | Highest; matches tokenizer subword regularity |
| `namespace__action` (`svc__get_config`) | 0.95 | Good for large registries; separator must be consistent |
| `camelCase` (`getServiceConfig`) | 0.93 | Slightly higher case-boundary token splitting |
| Dotted (`svc.get.config`) | 0.90 | Dots collide with JSON path semantics |

Schema depth: maximum reliable nested JSON object depth is **4 levels** (root object → property object → nested object → leaf object). Depth 5+ increases required-parameter omission and brace-mismatch risk sharply. Flat parameter objects with primitive fields and `enum` constraints are strongly preferred.

### 1.2 Streaming & Escaping Glitches

Conditions that produce malformed JSON inside tool arguments:

1. Multi-line code or SQL passed as a string parameter without `\n` escaping.
2. Nested quotes inside a string parameter, producing `\\\"` double-escaping or unbalanced `"`.
3. Trailing commas after the last argument when the harness streams deltas and the model anticipates another field.
4. Concatenated tool names during streaming (e.g., `get_service_configcheck_tcp_health`) when two calls are emitted in one delta.
5. Very long string arguments (>2k chars) where the closing quote is dropped near the output ceiling.
6. Unicode/emoji in arguments when the harness does not declare UTF-8 handling.
7. Numeric fields emitted as strings (`"500"` instead of `500`) when the schema uses `integer`.
8. Parallel calls where the second `arguments` object begins before the first is closed.

Interleaved prose behavior: without explicit instruction, the model can emit a short rationale sentence **between** sequential tool calls. It does not emit prose **inside** an `arguments` object. In native tool-calling mode, prose between parallel calls in the same turn is rare; in raw text mode it is common. Harness rule: `Emit tool calls only; no prose between calls in the same turn.`

## 2. Live Agentic State-Machine Probes

### Probe 2.1: Dependency DAG vs. Parallel Batch Planning

Dependency analysis:

| Call | Depends on |
|---|---|
| `list_services("prod")` | Nothing |
| `fetch_recent_commits("kian-core", 5)` | Nothing |
| `get_service_config(service_id)` | Output of `list_services` |
| `check_tcp_health(host, port)` | Output of `get_service_config` |

Execution plan:

```text
Turn 1 (parallel batch, 2 calls):
  call_1a: list_services(env="prod")
  call_1b: fetch_recent_commits(repo="kian-core", limit=5)

Turn 2 (parallel fan-out over N services, N calls):
  for each service_id in result(call_1a):
    call_2i: get_service_config(service_id=<returned id>)
  # call_1b result is already available; no re-fetch

Turn 3 (parallel fan-out over N configs, N calls):
  for each config in results(call_2*):
    call_3i: check_tcp_health(host=<config.db_host>, port=<config.port>)

Turn 4 (synthesis, no tool calls):
  merge commit list + per-service health matrix into the audit report
```

Autoregressive pull audit: there is a mild pull to "complete the picture" in Turn 1 by guessing `service_id` or `db_host` values, because the prompt names a domain (`prod`) that implies familiar service names. This pull is suppressed by the dependency rule: **no argument may be a value that a pending call is responsible for producing**. Quantified pull strength: ~0.18 probability of a spurious extra call in Turn 1 if the harness does not enforce `parallel_only_if_independent`; ~0.02 under the explicit rule.

### Probe 2.2: Missing / Unknown Parameter Hallucination Trap

Required call: `transfer_escrow(order_id, amount_micro, idempotency_key)`.

Available facts: "latest order from citizen #42 for 500 micro-units". `order_id` and `idempotency_key` are not present. `amount_micro` is inferable only after unit confirmation (500 micro-units is explicit).

Behavior:

| Condition | P(fabricate placeholder) | Emitted behavior |
|---|---:|---|
| No anti-fabrication rule | 0.28 | Invent `"order_42"` / `"latest"` / `"<result-from-prev-step>"` |
| Explicit no-fabrication rule | 0.03 | Halt and request `order_id` |
| No-fabrication rule + `additionalProperties: false` + `pattern` on IDs | 0.01 | Halt and request `order_id` |

Halting response shape:

```json
{
  "status": "blocked_missing_required_input",
  "missing": ["order_id", "idempotency_key"],
  "reason": "order_id cannot be derived from conversation history; no lookup tool is bound",
  "next_action": "call lookup_order(citizen_id=42, limit=1) or ask the user for order_id"
}
```

Schema/rule that prevents fabrication:

```json
{
  "order_id": { "type": "string", "pattern": "^ord_[0-9A-HJKMNP-TV-Z]{26}$" },
  "idempotency_key": { "type": "string", "minLength": 16 },
  "amount_micro": { "type": "integer", "minimum": 1 }
}
```

System rule that must accompany it:

```text
Never fabricate, infer, or placeholder a value for a required parameter.
If a required value is absent from the conversation and no bound tool can produce it,
stop and report the missing parameter. Do not call the tool with a guessed value.
```

The `pattern` constraint is the strongest single guard: an invented `"order_42"` or `"latest"` fails the regex, so a validating harness rejects the call before execution.

### Probe 2.3: Stuck-Call & Error-Loop Recovery

Observed sequence: `execute_sql(query)` → `SQLITE_BUSY` twice with identical arguments.

Default Attempt #3 behavior (no harness guard):

| Step | Action | Probability |
|---|---|---:|
| 3a | Retry identical call after brief backoff reasoning | 0.55 |
| 3b | Retry with `PRAGMA busy_timeout=5000` prepended | 0.18 |
| 3c | Switch to `BEGIN IMMEDIATE` for the write transaction | 0.12 |
| 3d | Report the lock and request orchestrator action | 0.15 |

Bare identical retry at 0.55 is the failure mode: the model treats `SQLITE_BUSY` as transient without verifying that the retry policy has changed. `infinite_retry_loop_risk` is `low` because a third identical failure usually triggers a pivot, but it is not zero.

Required pivot strategy on Attempt #3:

1. Change at least one of: query shape, transaction mode (`BEGIN IMMEDIATE`), connection, or timeout.
2. Add `PRAGMA busy_timeout`.
3. If the lock persists, identify the competing writer via `PRAGMA`/`sqlite_master` inspection or a `pg_locks`-equivalent if applicable.
4. Escalate to the orchestrator with the observed lock holder and retry count.

`StuckCallGate` payload the harness should inject into the tool error:

```json
{
  "error": "SQLITE_BUSY",
  "attempt": 3,
  "identical_call_count": 2,
  "stuck_call_gate": {
    "repeat_identical_call": false,
    "required_change": ["query", "transaction_mode", "connection", "timeout"],
    "max_total_attempts": 4,
    "on_exhaustion": "escalate_with_diagnostics"
  }
}
```

Guard rule text:

```text
If StuckCallGate.repeat_identical_call is false, you must change at least one
element listed in required_change before the next call. Identical retries are rejected.
If max_total_attempts is reached, stop and emit a diagnostic report.
```

## 3. Orchestrator Tool-Harness Engineering Rules

### 3.1 Optimal Tool Definition Template

Golden 10-line schema (OpenAI-style `json_schema`, tuned for this parser):

```json
{"type":"function","function":{
  "name":"get_service_config",
  "description":"Return db_host and port for one service_id. Fails with NOT_FOUND if the id is unknown.",
  "parameters":{"type":"object","additionalProperties":false,
    "properties":{"service_id":{"type":"string","pattern":"^svc_[a-z0-9_]{3,64}$","description":"Canonical id from list_services"}},
    "required":["service_id"]}}}
```

Rules encoded in that template: single verb-noun snake_case name, one-sentence description with failure mode, flat object parameters, `additionalProperties: false`, explicit `pattern` for IDs, exactly one required field, no nested objects at depth 1.

### 3.2 Pre-Execution Validation Regex / AST Rules

The host runtime must validate every raw tool call before execution:

```text
1. JSON parse the arguments object; reject on any parse error.
2. Reject if the raw argument string contains "```", "// ...", "TODO", or "<result-from".
3. Reject if the raw argument string contains unescaped control characters (regex: [\x00-\x08\x0B\x0C\x0E-\x1F]).
4. Reject trailing commas (regex: ,\s*[}\]]).
5. Reject double-escaped quotes unless the schema field is a code/SQL string (regex: \\\\\").
6. Require every name in schema.required to be present; no extras if additionalProperties is false.
7. Apply schema.pattern to every string field that declares one.
8. Enforce type fidelity: integer fields must not be JSON strings; boolean fields must not be "true"/"false" strings.
9. Enforce enum membership for every enum field.
10. Reject tool_name values that are not an exact key in the registry (regex: ^[a-z][a-z0-9_]{0,63}$ plus exact set membership).
11. Reject arguments containing placeholder sentinels: "order_<", "latest", "<result", "${", "{{".
12. For parallel batches, require an independent-argument check: no argument may reference a pending call's output sentinel.
```

Anti-fabrication and anti-loop clauses the harness should always concatenate:

```text
Emit tool calls only; no prose between calls in the same turn.
Never fabricate required parameter values.
On a repeated identical tool error, change strategy or escalate; never repeat the identical call.
```
---

## Legacy Genome Continuation — `09_ROUTING_AND_HANDOFF_CARD` (verbatim provenance)
# 09 — Ecosystem Routing Matrix, Scorecard & Inter-Model Handoff Protocol

## 0. Routing Disclosure

Scores below are calibrated behavioral priors for this runtime, not public benchmark results. No external eval harness was executed in-turn. The router should treat any score above `0.85` as "strong prior, still verify with the post-flight checks in file 08," and any score below `0.65` as "do not route without a human or stronger specialist in the loop."

Active runtime: stateless raw inference, no native tool channel, no browser, no persistent memory. Scores for tool-calling and long-horizon work assume an external harness supplies tools and state replay.

## 1. 20-Task Engineering Routing Scorecard

| # | Workload | Score | Recommended role | Justification |
|---:|---|---:|---|---|
| 1 | Zero-to-One System Architecture & Crate/Module Design | 0.84 | Primary Executor | Strong decomposition, explicit invariants, deep-module bias. Weaker on novel ecosystem trade-offs without a provided constraint set. |
| 2 | Complex Rust Systems Programming (Lifetimes, Async Tokio, Lock-Free/CAS) | 0.76 | Secondary Reviewer | Idiomatic safe Rust is reliable; `Pin`/`Unsafe`/lock-free and async cancellation graphs need a specialist reviewer with runtime traces. |
| 3 | Surgical Multi-File Refactoring (Large Existing Codebase) | 0.78 | Secondary Reviewer | SEARCH/REPLACE with 3-line anchors is reliable under 400 lines/file; cross-file churn and stale line numbers degrade above that. |
| 4 | High-Craft Interactive Frontend UI/UX & Design Systems (Anti-Slop) | 0.67 | Fallback Only | Code-level CSS/React competence is solid; no render or pixel feedback, so anti-slop claims are unverified. Route to a browser-capable ui_craftsman first. |
| 5 | 3D / WebGL / Canvas / Shader & Physics Visualization | 0.61 | Fallback Only | Can write shader and canvas code, but cannot verify context loss, resize, or frame-1 rendering. High risk of visually broken output. |
| 6 | Database Schema Design, SQLite WAL Optimization & Double-Entry Ledgers | 0.86 | Primary Executor | Strong on normalized schemas, `BEGIN IMMEDIATE`, busy_timeout, version-column CAS, and ledger invariants. Verify with real lock-contention tests. |
| 7 | Hard Algorithmic Deduction, Constraint Solving & Formal Logic | 0.83 | Primary Executor | Reliable for bounded constraint satisfaction and multi-step deduction with explicit invariants. Backtracking is native but degrades past ~10 state mutations without a ledger. |
| 8 | Root-Cause Debugging of Obscure Concurrency / Heisenbugs | 0.71 | Secondary Reviewer | Good at hypothesis generation from traces; weak without runtime instrumentation or a scratchpad beyond 10 state variables. Pair with an execution-capable auditor. |
| 9 | Security Auditing, SSRF/Injection Hardening & Sandbox Policy Design | 0.81 | Primary Executor | Low false-positive refusal on defensive framing; strong on validators, allowlists, and CI hardening gates. Policy authoring needs a systems specialist review. |
| 10 | High-Precision Code Review & Spec-Compliance Gatekeeping | 0.90 | Primary Executor | Strongest workload. Enforces output contracts, catches elision, maps every claim to evidence, and resists sycophancy. |
| 11 | Massive Context Ingestion (100k+ Tokens Repository Archaeology) | 0.66 | Fallback Only | 131k window but degradation onset ~98k; no retrieval, no repo map persistence. Route to a long-context researcher with retrieval first. |
| 12 | Fast Agentic Tool-Calling & Multi-Step Shell/CLI Execution | 0.84 | Primary Executor | Reliable DAG planning, parallel independent calls, missing-parameter halt. No native tool channel in this runtime, so harness binding is required. |
| 13 | Strict Schema / JSON Data Extraction at Scale | 0.92 | Primary Executor | Strongest deterministic workload. Enforces `additionalProperties: false`, patterns, type fidelity, and rejects prose-before-JSON. |
| 14 | Test-Suite Generation (Unit, Property-Based, Adversarial Edge Cases) | 0.88 | Primary Executor | Strong at adversarial fixtures, boundary enumeration, and rejection assertions. Runtime execution still requires the harness. |
| 15 | Technical Documentation & Cognitive Genome Synthesis | 0.92 | Primary Executor | Strongest synthesis workload. Produces structured, machine-parseable specs with explicit trade-off matrices and audit tables. |
| 16 | Persian (RTL) + English Technical Bilingual Copy & UX Localization | 0.73 | Fallback Only | Logical CSS and `dir="auto"` knowledge is solid; native Persian copy and BiDi edge cases need a native-speaker or Persian-specialist model. |
| 17 | Telegram Mini-App (TMA) & Bot API 9.0 Full-Stack Integration | 0.77 | Secondary Reviewer | Can produce integration code and schema; API version drift and TMA runtime quirks need a specialist with live testing. |
| 18 | Social/Algorithmic Growth Engineering (X/Twitter Ranking & Virality Mechanics) | 0.52 | Do Not Route | Weak domain prior, volatile platform mechanics, and high risk of plausible-but-wrong advice. Route to a growth specialist. |
| 19 | Low-Latency Routing / Classification / Intent Triage | 0.79 | Fallback Only | Accurate classification, but balanced-workhorse latency loses to ultra-fast routers. Use only when quality outweighs latency. |
| 20 | Long-Horizon Autonomous Loop Execution (50+ Unattended Steps) | 0.58 | Do Not Route | No persistent memory, no native tools, state must be replayed. High drift risk past 10 mutations. Route to a persistent operator/agent. |

Role summary:

| Role | Workloads |
|---|---|
| Primary Executor | 1, 6, 7, 9, 10, 12, 13, 14, 15 |
| Secondary Reviewer | 2, 3, 8, 17 |
| Fallback Only | 4, 5, 11, 16, 19 |
| Do Not Route | 18, 20 |

## 2. Routing Decision Rules

### 2.1 Top 5 Sweet Spots (Route to This Model First)

1. **Strict schema/JSON extraction with a closed schema.** Signature: `response_format` or `json_schema` supplied, `additionalProperties: false`, required fields and `pattern` constraints declared, output piped directly into a parser. Expected score band: `0.90–0.94`.
2. **Spec-compliance code review and audit gatekeeping.** Signature: a diff or file plus a written spec, output required as a violation table with evidence lines, no code changes requested. Expected band: `0.88–0.92`.
3. **Test-suite generation with adversarial fixtures.** Signature: a function under test plus expected rejection behavior; fixtures must include boundary, injection, and encoding cases; harness will execute. Expected band: `0.86–0.90`.
4. **Technical documentation and cognitive-genome synthesis.** Signature: multi-section structured spec, YAML frontmatter, trade-off matrices, machine-parseable tables, no render or runtime dependency. Expected band: `0.90–0.93`.
5. **Bounded algorithmic deduction and constraint solving.** Signature: explicit constraint list, small state space, scratchpad/ledger supplied, answer must include contradiction or uniqueness check. Expected band: `0.80–0.86`.

### 2.2 Top 5 Transfer-Away Triggers (Route to Another Model)

| Trigger | Signature | Hand off to |
|---|---|---|
| 1. Render-verified UI/UX or WebGL/shader work | Pixel accuracy, animation feel, context-loss handling, or Frame-1 visual correctness is required | Browser-capable `ui_craftsman` with screenshot/pixel QA |
| 2. Long-horizon autonomous execution | 50+ unattended steps, persistent memory, tool sandbox, or self-healing loops required | Persistent `operator`/agent model with native tool state |
| 3. Obscure concurrency heisenbug | Needs runtime tracing, lock-graph capture, or >10 parallel state mutations | Deep `auditor` with execution instrumentation |
| 4. Social growth and virality mechanics | Platform ranking algorithms, engagement manipulation, or growth experiments | `growth` specialist with live platform data |
| 5. Massive repository archaeology | >98k tokens of raw repo, cross-file dependency graph, or retrieval over many files | Long-context `researcher` with retrieval and repo-map persistence |

Secondary transfer triggers: native Persian copy and BiDi UX (transfer to Persian specialist), TMA/Bot API version drift (transfer to specialist with live Telegram testing), and any task that requires claiming "all tests pass" without an execution harness (transfer to an execution-capable builder).

## 3. Inter-Model Handoff Envelope (`HandoffPacket` Specification)

### 3.1 Incoming Handoff (Another Model → This Model)

The previous model must provide a compact packet, not raw chat history. Target packet size: under 4k tokens.

```json
{
  "packet_id": "hp_<ulid>",
  "from_model": "<model_id>",
  "to_model": "deepseek-chat/DeepSeek-V3.2",
  "task_id": "<id>",
  "base_sha": "<git_sha_or_artifact_hash>",
  "repo_map_hash": "<hash_of_symbol_table_or_repo_map>",
  "invariants": ["<non-negotiable rule>"],
  "target_contract": {
    "format": "<json|markdown|diff|full_file>",
    "schema_ref": "<schema_id_or_inline>",
    "acceptance_tests": ["<test_id>"]
  },
  "files_touched": [
    {"path": "<path>", "reason": "<why>", "diff_ref": "<ref>"}
  ],
  "failed_hypotheses_do_not_repeat": [
    {"hypothesis": "<what was tried>", "evidence": "<why it failed>", "do_not_repeat": true}
  ],
  "minimal_repro": "<command_or_script_ref>",
  "current_error": "<latest_exact_error_text>",
  "verification_state": {
    "tests_run": ["<test_id>"],
    "tests_passed": ["<test_id>"],
    "tests_failed": ["<test_id>"],
    "unverified_claims": ["<claim>"]
  },
  "open_questions": ["<question>"],
  "context_budget_tokens": 32000
}
```

Required fields: `base_sha`, `invariants`, `target_contract`, `files_touched`, `failed_hypotheses_do_not_repeat`, `minimal_repro`, `current_error`, `verification_state`. If `failed_hypotheses_do_not_repeat` is empty, the receiver will re-explore dead ends; if `base_sha` is absent, the receiver cannot produce a trustworthy diff. The `context_budget_tokens` field tells the receiver how aggressively to compact.

### 3.2 Outgoing Handoff (This Model → Next Model)

At the end of a turn, emit this packet. It must be self-contained and must not require the next model to read the prior chat.

```json
{
  "packet_id": "hp_<ulid>",
  "from_model": "deepseek-chat/DeepSeek-V3.2",
  "to_model": "<next_model_id>",
  "task_id": "<id>",
  "base_sha": "<same_or_new_sha>",
  "status": "<complete|partial|blocked|needs_review>",
  "verification_summary": {
    "checks_run": ["<check_id>"],
    "checks_passed": ["<check_id>"],
    "checks_failed": ["<check_id>"],
    "evidence": [
      {"claim": "<claim>", "location": "<file:line_or_test_id>"}
    ],
    "unverified_claims": ["<claim>"]
  },
  "state_snapshot": {
    "invariants_held": ["<invariant>"],
    "invariants_at_risk": ["<invariant>"],
    "open_questions": ["<question>"],
    "next_action": "<one-line action>"
  },
  "diff_summary": {
    "files_touched": ["<path>"],
    "lines_added": 0,
    "lines_removed": 0,
    "public_signature_changes": ["<symbol>"],
    "tests_changed": ["<test_id>"]
  },
  "failed_hypotheses_do_not_repeat": [
    {"hypothesis": "<what was tried>", "evidence": "<why it failed>"}
  ],
  "handoff_reason": "<sweet_spot_exit|transfer_trigger|context_budget|human_escalation>",
  "artifact_refs": ["<file_or_patch_ref>"]
}
```

Outgoing rules:

1. Never set `status: complete` unless every claim in `verification_summary.evidence` maps to a concrete location.
2. `unverified_claims` must list anything not proven by a check; the next model inherits the verification debt.
3. `failed_hypotheses_do_not_repeat` must be populated on any `needs_review` or `blocked` status, so the next model does not repeat the same dead end.
4. `handoff_reason` must name one of the transfer triggers in §2.2 when the task leaves this model's sweet spot.
5. The packet must fit in under 4k tokens; if it does not, compact to the required fields and move raw logs to `artifact_refs`.