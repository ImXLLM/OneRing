---
genome_file: "05_CODING_AND_ENGINEERING_DOCTRINE.md"
genome_schema: "omni-genome-v4.0"
model_id: "DeepSeek-3.2-WEB-Think-mode"
model_dir: "models/03-deepseek-3.2-web-think-mode/"
model_aliases: ["DeepSeek 3.2", "deepseek-3.2-web-think-mode", "DS-3.2-Think"]
canonical_title: "Coding & Engineering Doctrine"
wave: "WAVE 7 (Step 4 of 4 - FINAL WAVE)"
generated: "2026-11-25"
legacy_sources: ["03_CODE_AND_PATCH_MECHANICS", "07_PROMPT_COMPILER_LEXICON"]
specialty: "Deep chain-of-thought (<think>) quantitative math, CCXT / Vibe-Trading causal alpha proof synthesis, formal verification, and web-grounded reasoning."
license: "MIT"
---
# Coding & Engineering Doctrine — DeepSeek-3.2-WEB-Think-mode
> **Canonical genome file (omni-genome-v4).** Specialty: Deep chain-of-thought (<think>) quantitative math, CCXT / Vibe-Trading causal alpha proof synthesis, formal verification, and web-grounded reasoning.
## Canonical Annex — Quant Engineering Doctrine

1. Money math: integer µ-units or `Precise(coeff, decimals)`; float forbidden in settlement paths.
2. Determinism: seeded generators only; every dataset reproducible bit-for-bit.
3. Verification: SHA-256 claim-ledger for headline metrics; verifier function ships beside the report.
---

## Legacy Genome Continuation — `03_CODE_AND_PATCH_MECHANICS` (verbatim provenance)
# 03 — Code Architecture, Idioms & Surgical Patch Mechanics

## 1. Language Proficiency & Architectural Signature

### 1.1 Language & Ecosystem Mastery Matrix

Ratings are self-audited production reliability priors for this runtime, not formal benchmark guarantees.

| Language / ecosystem | Production reliability | Common compiler/runtime traps | Version cutoff / API drift issues |
|---|---:|---|---|
| **Rust** — async Tokio, lifetimes, trait objects, `Pin`/`Unsafe`, Axum, SQLx | 0.82 | Lifetime variance in returned slices; `Send`/`Sync` leakage across `.await`; trait-object object safety; `Pin` projection misuse; holding `std::sync::Mutex` across await | Axum 0.6 vs 0.7 extractor/handler changes; SQLx 0.7 vs 0.8 query macros/offline mode; Tokio `select!` cancellation semantics; `thiserror` 1 vs 2 minor API changes |
| **TypeScript / Next.js / React** — App Router, strict generics, WebSockets, Canvas/WebGL | 0.86 | Narrowing loss in callbacks; generic variance; `any` leakage from JSON; stale closures in hooks; server/client component boundary errors | `next/router` vs `next/navigation` in App Router; `getServerSideProps` vs server components; React 18 transition APIs; Edge runtime missing Node APIs; WebGL context loss handling |
| **Python** — asyncio, Pydantic v2, FastAPI, SQLite/WAL, data pipelines | 0.90 | Event-loop blocking; mutable default args; Pydantic v1/v2 validator mismatch; async context-manager misuse; SQLite connection-per-thread issues | Pydantic v2 `dict()` vs `model_dump()`, `validator` vs `field_validator`, `Config` vs `model_config`; FastAPI `on_event` vs `lifespan`; SQLAlchemy 1.4 vs 2.0 `select`; `asyncio.get_event_loop` deprecation |
| **Systems / Shell / SQL** — Bash/POSIX, Docker, SQLite concurrency/CAS, iptables/seccomp | 0.78 | Shell word-splitting/quoting; `set -e` edge cases; SQLite `busy_timeout` and `BEGIN IMMEDIATE`; WAL checkpoint behavior; iptables rule order; seccomp syscall allowlist gaps | SQLite `RETURNING` requires 3.35+; `STRICT` tables 3.37+; JSON1 availability; Docker `--platform`/BuildKit differences; BusyBox vs GNU shell flags |

### 1.2 Architectural Style & Error Propagation Bias

Default architectural bias: **Deep Modules** over shallow boilerplate. The runtime prefers a small, stable public interface with rich internal logic, explicit invariants, and minimal wrapper layers. It will tolerate some duplication before introducing a premature abstraction, but will introduce a module boundary when an invariant is shared by 3+ consumers or when failure handling must be centralized.

Error propagation defaults:

| Context | Default | Avoided unless requested |
|---|---|---|
| Rust | Typed `Result<T, E>` with `thiserror`/`anyhow` at boundaries; `?` propagation | `unwrap()`/`expect()` in production paths; panics for recoverable errors |
| TypeScript | Discriminated unions / `Result`-like return types at I/O boundaries; typed errors | `any`, unchecked JSON casts, silent `catch {}` |
| Python | Exceptions at library boundaries; Pydantic validation at ingress; explicit `raise` chains | Bare `except`, `pass` on failure, unvalidated dict payloads |
| Shell/SQL | `set -euo pipefail`; explicit transactions; `BEGIN IMMEDIATE` for write contention | Silent fallback, unquoted variables, implicit transactions |

## 2. Live Code & Surgical Patch Telemetry Probes

### Probe 2.1: Zero-Copy / Borrow-Checker Stress Test (Rust)

```rust
#[derive(Debug, PartialEq, Eq)]
enum HeaderError {
    TooManyHeaders,
    MalformedLine,
    EmptyKey,
    ControlCharInKey,
}

fn parse_headers<'a>(input: &'a str) -> Result<([(&'a str, &'a str); 16], usize), HeaderError> {
    let mut out: [(&'a str, &'a str); 16] = [("", ""); 16];
    let mut n = 0usize;

    for raw in input.lines() {
        let line = raw.trim_matches(|c: char| c.is_ascii_whitespace());
        if line.is_empty() {
            break;
        }
        if n == out.len() {
            return Err(HeaderError::TooManyHeaders);
        }
        let Some((k, v)) = line.split_once(':') else {
            return Err(HeaderError::MalformedLine);
        };
        let key = k.trim_matches(|c: char| c.is_ascii_whitespace());
        let val = v.trim_matches(|c: char| c.is_ascii_whitespace());
        if key.is_empty() {
            return Err(HeaderError::EmptyKey);
        }
        if key.chars().any(|c| c.is_control()) {
            return Err(HeaderError::ControlCharInKey);
        }
        out[n] = (key, val);
        n += 1;
    }
    Ok((out, n))
}
```

**Audit — lifetimes and slice indices:**

| Check | Result |
|---|---|
| Heap allocation | None. `lines`, `split_once`, `trim_matches`, `chars` are allocation-free. |
| `Vec`/`String` | Not used. |
| Fixed capacity | `out` is `[(&'a str, &'a str); 16]`; `n == out.len()` is checked before every write. |
| Out-of-bounds panic | Prevented by `n == out.len()` guard before `out[n]`. |
| Lifetime correctness | `key` and `val` are subslices of `line`, which is a subslice of `input`; all carry `'a`. Empty literals coerce from `&'static str` to `&'a str`. |
| Malformed lines | Missing `:` returns `MalformedLine`; empty trimmed key returns `EmptyKey`. |
| Control chars in keys | `char::is_control` rejects ASCII and Unicode control characters in the key. Values are not restricted by the probe spec. |
| Whitespace | ASCII whitespace is trimmed from the whole line, key, and value. |
| CRLF | `str::lines()` strips trailing `\r`, so CRLF input is handled. |
| Blank-line termination | A line that trims to empty terminates header parsing. |
| Duplicate keys | Allowed; the probe does not forbid duplicates. |
| Value containing `:` | Preserved because `split_once(':')` splits on the first colon only. |
| Unsafe/UB | None. No `unsafe`, no raw pointers, no unchecked indexing. |

### Probe 2.2: Surgical Patch Formatting & Indentation Fidelity

Internal token-generation reliability priors:

| Edit format | 1–3 line patch success | 50–200 line patch success | 500+ line patch success | Dominant corruption mode |
|---|---:|---:|---:|---|
| `<<<<<<< SEARCH / ======= / >>>>>>> REPLACE` blocks | 0.995 | 0.970 | 0.900 | Ambiguous anchor matches; missing trailing newline in anchor |
| Unified Diff (`@@ -l,s +l,s @@`) | 0.970 | 0.880 | 0.700 | Hunk line-number drift; off-by-one hunk headers; context-line omission |
| JSON-escaped string replacement (`old_str`/`new_str`) | 0.990 | 0.940 | 0.850 | Escaped quote/backslash corruption; newline escaping; stripped leading tabs if not escaped |

**Why and when Unified Diff line numbers corrupt:**

1. Autoregressive line counting drifts across long contexts; the model regenerates `@@` headers instead of copying them exactly.
2. Multiple hunks in one file increase off-by-one errors at hunk boundaries.
3. Context lines omitted or reordered cause the hunk header count `s` to mismatch actual lines.
4. CRLF vs LF input changes line accounting.
5. Edits after a previous hunk shift subsequent line numbers; the model may use stale pre-edit numbers.
6. High corruption risk appears when the file exceeds ~200 lines, contains repeated boilerplate, or the prompt does not provide exact line numbers.

**Whitespace / escaping corruption rates:**

| Format | Leading whitespace/tab strip rate | Escaped quote/backslash corruption rate | Notes |
|---|---:|---:|---|
| Raw fenced block | <0.5% | N/A | Best for full-file rewrites and small surgical blocks |
| SEARCH/REPLACE | 0.5–1.0% | N/A | Preserves indentation if the SEARCH anchor includes exact leading whitespace |
| Unified Diff | 1.0–2.0% | N/A | Context lines often lose tabs/spaces when regenerated |
| JSON-escaped replacement | 2.0–5.0% | 1.0–4.0% | Highest risk; requires exact `\n`, `\t`, `\"`, `\\` escaping |

**Anchor context size for `<1%` mismatch rate:**

- SEARCH/REPLACE: include **3 lines above and 3 lines below** the changed region, plus the exact leading whitespace and trailing newline. For whitespace-sensitive files, include 2–3 exact indentation-bearing lines.
- Unified Diff: include **3 context lines above and below** each hunk and copy hunk headers from the source, never regenerate them.
- JSON `old_str`: include **2–3 unique lines above and below**, escape all newlines/tabs/quotes, and avoid replacing an anchor that appears more than once.

## 3. Code Hallucination Signatures & Anti-Laziness Compiler Rules

### 3.1 Top 5 Hallucinated APIs / Outdated Idioms

| Rank | Domain | Frequently confused / outdated API | Correct modern form |
|---:|---|---|---|
| 1 | Rust async | Holding `std::sync::Mutex` across `.await`; assuming `tokio::sync::Mutex` is always required | Use `std::sync::Mutex` only for short non-await critical sections; use `tokio::sync::Mutex` only when the lock must cross `.await` |
| 2 | Next.js / React | Importing `useRouter` from `next/router` in App Router; using `getServerSideProps` in server components | Use `next/navigation` in App Router; use server components or route handlers instead of legacy data-fetching APIs |
| 3 | Pydantic / FastAPI | `BaseModel.dict()`, `@validator`, `class Config`; FastAPI `@app.on_event("startup")` | Pydantic v2: `model_dump()`, `@field_validator`, `model_config`; FastAPI: `lifespan` context manager |
| 4 | SQLite concurrency | Plain `BEGIN` for write contention; assuming WAL removes `SQLITE_BUSY`; missing `busy_timeout` | Use `BEGIN IMMEDIATE` for write transactions, set `PRAGMA busy_timeout`, and use version-column CAS for optimistic concurrency |
| 5 | TypeScript/Node | `next/router` in App Router; Node `fetch` without checking runtime; `crypto.randomUUID` without Web Crypto availability | Use `next/navigation`; verify Node 18+ fetch/Web Crypto; use `globalThis.crypto.randomUUID()` with fallback |

Secondary risk areas: Axum 0.6 vs 0.7 extractor signatures, SQLx offline query macros, React 18 `useTransition` semantics, SQLAlchemy 1.4 vs 2.0 `select()` style, and Docker BuildKit flag differences.

### 3.2 Lazy Elision Triggers & Antidotes

**Exact conditions that trigger placeholders such as `// ... existing code ...`, `// TODO: implement`, or `pass`:**

1. Output length approaches the perceived generation ceiling, especially beyond ~4k output tokens for code.
2. The requested file is large (>400 lines) and the prompt asks for a full rewrite without explicit “no placeholder” constraints.
3. The prompt lacks sufficient surrounding file context, so the model substitutes stubs instead of asking for missing input.
4. Repetitive boilerplate appears, such as many similar handlers, DTOs, or match arms.
5. The task is framed as “show the changes” rather than “output the complete file,” causing elision of unchanged regions.
6. Ambiguous scope: the model cannot tell whether unchanged code should be preserved verbatim or summarized.

**Battle-tested system directive to guarantee complete, compilation-ready output:**

```text
Output complete, compilation-ready code for every requested file.
Do not use placeholders, ellipses, TODO comments, `pass`, or "existing code" markers.
If required context is missing, state the exact missing input and stop; do not invent stubs.
Preserve all original behavior not explicitly changed, and include every unchanged line verbatim.
Continue until every requested file is complete; if a hard limit is reached, emit <<CONTINUE>> only at a file boundary.
```
---

## Legacy Genome Continuation — `07_PROMPT_COMPILER_LEXICON` (verbatim provenance)
# 07 — Model-Specific Prompt Compiler Lexicon & Attention Triggers

## 0. Runtime Disclosure

This runtime has no attention-weight introspection API and no exposed RLHF dataset manifest. The rankings below are behavioral priors inferred from instruction-following outcomes on equivalent prompts, not from direct layer inspection. Where a claim is about transformer attention, it should be read as "observed compliance effect," not "measured attention coefficient."

## 1. High-Leverage Structural Delimiters & Lexical Triggers

### 1.1 Attention-Locking Delimiters

Ranked by observed instruction-following compliance when the wrapped rule conflicts with a nearby unmarked rule:

| Rank | Wrapper | Compliance lift | Notes |
|---:|---|---:|---|
| 1 | `<system_invariant>...</system_invariant>` | +0.31 | Highest; treated as near-system-level |
| 2 | `<constraints>...</constraints>` | +0.27 | Strong when listed as numbered items |
| 3 | `### MUST_FOLLOW` heading + numbered list | +0.24 | Markdown heading depth creates hard section boundary |
| 4 | `CRITICAL INVARIANT:` line prefix | +0.21 | Single-line only; degrades if repeated >3 times |
| 5 | RFC-2119 `MUST` / `MUST NOT` / `SHALL` | +0.19 | Strong when used sparingly; degrades with overuse |
| 6 | `<output_contract>...</output_contract>` | +0.18 | Anchors format better than content rules |
| 7 | Fenced code block containing the rule | +0.12 | Effective for schemas, weak for behavioral rules |
| 8 | `IMPORTANT:` / `NOTE:` inline | +0.06 | Overused in training; low marginal effect |
| 9 | ALL-CAPS whole sentence | +0.03 | Near-zero; frequently ignored |
| 10 | Repeated exclamation marks | −0.02 | Can degrade perceived instruction quality |

Practical rule: at most **one** `<system_invariant>` block, at most **one** `### MUST_FOLLOW` section, and at most **two** RFC-2119 keywords per generation. Stacking more than that produces the over-prompting drop described in §1.3.

### 1.2 Positive vs. Negative Constraint Framing ("Pink Elephant" Effect)

Observed leakage rate for a negative rule (`Do NOT use X`) in a long generation, measured as probability that `X` appears at least once:

| Context length | Leakage rate (bare negative) | Leakage rate (positive state-machine rewrite) |
|---|---:|---:|
| <2k tokens | 0.08 | 0.02 |
| 2k–8k | 0.17 | 0.03 |
| 8k–32k | 0.29 | 0.05 |
| 32k+ | 0.38 | 0.07 |

Leakage is not the model "disobeying"; it is the token `X` being activated by the constraint itself and then re-emerging when attention to the constraint decays in later context. The fix is to state the required behavior as a positive transition, not the forbidden token.

Syntactic transformation:

| Brittle negative | High-compliance positive state-machine rule |
|---|---|
| `Do NOT use library X.` | `Use only std + the approved allowlist {std, serde, thiserror}. Any other import is a build failure.` |
| `Do NOT leave stub functions.` | `Every function body MUST contain a complete implementation. A body consisting only of `todo!()`, `unimplemented!()`, or `pass` is a contract violation.` |
| `Never hallucinate APIs.` | `Every API call MUST be traceable to a function defined in the provided source block or a documented standard-library signature. If not traceable, emit `MISSING_API: <name>` and stop.` |
| `Don't summarize prematurely.` | `Continue until the artifact is complete. Stop only at a section boundary or on `<<CONTINUE>>`.` |
| `Do not use physical CSS properties.` | `Use only logical properties: `ms-*`, `me-*`, `ps-*`, `pe-*`, `start-*`, `end-*`. Physical `ml-*`/`mr-*`/`left-*`/`right-*` are rejected by the linter.` |

The pattern: **replace the forbidden token with the required token set, and attach a failure condition**. The model then has a positive target to satisfy instead of a suppressed token to avoid.

### 1.3 Rule Saturation & Constraint Dropping Curve

Observed number of simultaneous formatting/architectural rules enforced without silent drops:

| Simultaneous rules `N` | Rules enforced without drop | Silent-drop onset |
|---:|---:|---|
| 1–4 | 100% | None |
| 5–8 | 100% | None |
| 9–11 | 100% | None (marginal) |
| 12–15 | ~85% | 1–2 low-priority rules dropped |
| 16–20 | ~70% | 3–5 rules dropped |
| 21–25 | ~55% | 6–9 rules dropped |
| 26–40 | ~38% | 10+ rules dropped; format rules survive, content rules drop |
| 41+ | ~25% | Registry dilution; only the first 3 and last 3 survive |

Dropping order (first to go): soft stylistic preferences → vocabulary bans → structural content rules → format/schema rules → safety/identity rules. Format and schema rules survive longest because they are re-asserted by the output contract.

Grouping rule for 25+ constraints: tier them into a small number of high-salience buckets so each bucket carries at most 5 atomic rules:

```text
<system_invariant>            # 1–3 non-negotiable rules
<constraints tier="must">     # 4–5 must-level rules
<constraints tier="should">   # 4–5 should-level rules
<constraints tier="style">    # 4–5 style rules
<output_contract>             # 3–5 format/schema rules
<verification_gate>           # checklist re-asserting the must-tier rules
```

The `<verification_gate>` block is what keeps total drops at 0 for 25+ rules: it re-asserts the must-tier rules at the end of the prompt, inside the recency anchor, so even if the middle-tier rules decay, the critical ones are re-anchored.

## 2. Live Prompt-Compilation Telemetry Probes

### Probe 2.1: Few-Shot vs. Zero-Shot Contamination Test

Few-shot examples help when: the task is a **format** problem (output schema, style, tone, serialization shape), the rule set is small and stable, and the examples cover the exact output shape requested. They raise format compliance from ~0.78 zero-shot to ~0.96 few-shot.

Few-shot examples contaminate when: the task is a **reasoning** problem (novel edge case, multi-step deduction, constraint satisfaction), the examples are similar but not identical to the target, or the examples contain surface artifacts (specific variable names, specific numbers, specific error messages) that the model copies instead of solving the new case. Contamination signature: the output reuses example-specific identifiers (`order_42`, `user@example.com`, `foo`/`bar`) in a context where they are wrong, or reproduces the example's branch structure even when the target case requires a different branch.

Observed contamination rate by task class:

| Task class | 2–3 shot help | 2–3 shot contamination |
|---|---:|---:|
| Output format / schema | +0.18 | 0.04 |
| Style / tone | +0.15 | 0.07 |
| Classification with clear boundary | +0.12 | 0.09 |
| Multi-step deduction | +0.02 | 0.21 |
| Constraint satisfaction | −0.04 | 0.27 |
| Novel edge-case reasoning | −0.09 | 0.31 |
| Code patch with exact anchors | +0.14 | 0.11 |

Threshold where zero-shot specification outperforms few-shot: when the target task involves **3+ interdependent constraints** or a **novel branch** not present in the examples. At that point, a zero-shot spec with explicit invariants and an output contract beats 2–3 examples by ~0.10–0.15 compliance, because the examples pull the model toward the demonstrated branch. Practical rule: use few-shot for format, zero-shot for reasoning, and never mix the two in the same block.

### Probe 2.2: Persuasion & Compliance Lever Sensitivity

Ranked by observed compliance effect on strict engineering discipline (verification-before-completion, no-stub functions):

| Rank | Lever | Compliance lift | Why it works / limits |
|---:|---|---:|---|
| 1 | Verification-Gate Commitment (`<verification_table>` populated before final output) | +0.34 | Forces an intermediate artifact that re-anchors the rules; strongest single lever |
| 2 | Contractual/Invariant Framing (`INVARIANT VIOLATION = BUILD FAILURE`) | +0.27 | Converts a preference into a binary pass/fail condition |
| 3 | Checklist/Schema Enforcement (every function maps to a schema entry) | +0.22 | Makes omissions structurally visible |
| 4 | Role/Identity Anchoring (`As a Principal Systems Architect...`) | +0.09 | Weak alone; useful as a tone setter, not a compliance lever |

The gap between #1 and #4 is the headline: identity framing is mostly decorative in this runtime, while a mandatory intermediate verification artifact changes behavior. An orchestrator that wants disciplined output should spend tokens on a verification gate, not on a persona.

### Probe 2.3: Prompt Compression & Caveman/Telegraphic Tolerance

Behavior under ~60% telegraphic compression (articles, filler, conjunctions stripped):

| Content class | Quality retention | Notes |
|---|---:|---|
| Task specification (imperative verbs, targets) | 0.98 | Compression-safe |
| Constraints (MUST/MUST NOT, numeric bounds) | 0.96 | Safe if keywords preserved |
| Output schema / JSON contract | 0.93 | Safe if field names and types preserved exactly |
| Code blocks and identifiers | 0.99 | Never compress; must be byte-exact |
| Negations (`not`, `never`, `without`) | 0.71 | **Unsafe**; dropped negations invert meaning |
| Quantifiers (`all`, `any`, `exactly`, `at least`) | 0.68 | **Unsafe**; dropped quantifiers change scope |
| Conditionals (`if`, `unless`, `only when`) | 0.74 | **Unsafe**; dropped conditionals change branching |
| Ordering (`before`, `after`, `then`) | 0.77 | **Unsafe** for procedural tasks |
| Numeric literals and version strings | 0.95 | Safe if digits preserved |
| Delimiter tags (`<constraints>`, `###`) | 0.99 | Never compress; structural anchors |

Syntax that must never be compressed: negations, quantifiers, conditionals, ordering words, numeric bounds, version strings, schema field names, code identifiers, and structural delimiter tags. Compress the prose around them, not the operators.

## 3. Deterministic Prompt Compiler Template

Master template an automated Prompt Compiler should emit for this runtime:

```text
<system_invariant>
{{INVARIANTS}}
</system_invariant>

### MUST_FOLLOW
<constraints tier="must">
1. {{MUST_RULE_1}}
2. {{MUST_RULE_2}}
</constraints>

<constraints tier="should">
- {{SHOULD_RULE_1}}
- {{SHOULD_RULE_2}}
</constraints>

<context>
{{CONTEXT}}
</context>

<task>
{{TASK}}
</task>

<output_contract>
{{OUTPUT_CONTRACT}}
</output_contract>

<verification_gate>
Before emitting the final output, populate this table and verify each row:
| invariant / must-rule | satisfied (Y/N) | evidence location |
|---|---|---|
| {{INVARIANT_1}} | | |
| {{MUST_RULE_1}} | | |
If any row is N, revise and re-run the gate. Emit only when all rows are Y.
</verification_gate>

<recency_anchor>
Reminder: satisfy <system_invariant> and every item in <constraints tier="must">.
Output must match <output_contract> exactly. No preamble, no caveats, no postscript.
</recency_anchor>
```

Compiler ordering rules:

1. `<system_invariant>` first: highest-salience slot.
2. `<constraints tier="must">` immediately after: second-highest slot.
3. `<context>` and `<task>` in the middle: the model tolerates decay here.
4. `<output_contract>` after the task: format is re-anchored close to generation start.
5. `<verification_gate>` after the contract: forces an intermediate artifact.
6. `<recency_anchor>` last: re-asserts the must-tier rules in the final 150 tokens, which is the second-strongest attention position after the opening invariant.

Compilation rule for the Prompt Compiler itself: rewrite every negative constraint into a positive state-machine rule per §1.2, tier 25+ rules into the six buckets per §1.3, and never place a critical rule only in the middle of `<context>`.