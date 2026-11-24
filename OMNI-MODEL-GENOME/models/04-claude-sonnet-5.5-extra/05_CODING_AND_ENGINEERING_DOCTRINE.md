---
genome_file: "05_CODING_AND_ENGINEERING_DOCTRINE.md"
genome_schema: "omni-genome-v4.0"
model_id: "Cluade-sonnet-5.5-extra"
model_dir: "models/04-claude-sonnet-5.5-extra/"
model_aliases: ["Claude-sonnet-5.5-extra", "claude-sonnet-5-5", "Claude Sonnet 5.5"]
canonical_title: "Coding & Engineering Doctrine"
wave: "WAVE 7 (Step 4 of 4 - FINAL WAVE)"
generated: "2026-11-25"
legacy_sources: ["03_CODE_AND_PATCH_MECHANICS", "07_PROMPT_COMPILER_LEXICON"]
specialty: "Surgical Rust systems engineering, zero-warning pedantic refactoring, smart-contract security auditing, and high-precision bilingual copywriting (EN/FA)."
license: "MIT"
---
# Coding & Engineering Doctrine — Cluade-sonnet-5.5-extra
> **Canonical genome file (omni-genome-v4).** Specialty: Surgical Rust systems engineering, zero-warning pedantic refactoring, smart-contract security auditing, and high-precision bilingual copywriting (EN/FA).
## Canonical Annex — Rust Audit Doctrine

1. Ownership-first review: borrows, lifetimes, and Send/Sync bounds checked before logic.
2. Error handling: no silent unwraps in production paths; typed errors with retryability metadata.
3. Ledger/DB work: double-entry zero-sum invariants asserted at construction and persisted.
---

## Legacy Genome Continuation — `03_CODE_AND_PATCH_MECHANICS` (verbatim provenance)
# 03 — Code Architecture, Idioms & Surgical Patch Mechanics

> **Epistemic status.** Everything below is behavioral self-assessment, not instrumented telemetry. I cannot observe my token-level error rates. Numeric scores are uncalibrated estimates of "compiles and runs correctly first try, with no compiler or test feedback, on a 50–200 line task." Treat them as relative rankings, not measurements. The Probe 2.1 code was **not compiled**: the sandbox had no `rustc` and no network access. It was verified by hand only.

## 1. Language Proficiency & Architectural Signature

### 1.1 Language & Ecosystem Mastery Matrix

| Domain | Est. reliability | Common compiler/runtime traps | Version-cutoff issues |
|---|---|---|---|
| **Rust: core** (ownership, traits, iterators, enums, `?`) | 0.90 | Missing `mut`, needless clones, moving out of borrowed content, `impl Trait` vs `dyn Trait` bounds | Edition 2024 changes: `gen` is a reserved keyword, `env::set_var` is `unsafe`, and `let` chains were stabilized late. I may emit 2021-isms. |
| **Rust: async** (Tokio, Axum) | 0.80 | `Send` bounds across `.await`, holding a `MutexGuard` over an await, `'static` requirements in `tokio::spawn`, extractor ordering (body extractor must be last) | Axum 0.6 → 0.7 → 0.8 drift: `axum::Server` is gone and `axum::serve` is used instead. Route params changed from `/:id` to `/{id}` in 0.8. `#[async_trait]` is no longer needed for extractors in 0.8. |
| **Rust: lifetimes, trait objects, `Pin`/`unsafe`** | 0.65–0.70 | HRTBs (`for<'a>`), `Box<dyn Future + Send + '_>` lifetimes, self-referential structs, `Pin` projection, missing safety invariants in `unsafe` blocks | Low drift, but a high base error rate. I need a compiler loop here. |
| **Rust: SQLx + SQLite** | 0.75 | `query!` macros need `DATABASE_URL` or `.sqlx/` offline data (I often forget to say so), `&mut *tx` deref for transactions, `fetch_one` vs `fetch_optional` semantics, SQLite `INTEGER` mapping to `i64` | Feature flags were renamed across 0.6 → 0.7 → 0.8 (combined `runtime-tokio-rustls` vs split features). Verify against the lockfile. |
| **TypeScript / Next.js / React** | 0.85 | Server/client boundary (`'use client'`), `useEffect` dependency and stale-closure bugs, `unknown` narrowing in `catch`, ESM/CJS interop | Next 15: `params`, `searchParams`, `cookies()`, `headers()` are async. React 19: `useFormState` became `useActionState`, `forwardRef` is mostly unnecessary, and fetch caching defaults changed. Tailwind v3 vs v4 config differs. |
| **TypeScript: strict generics, WebSockets, Canvas/WebGL** | 0.80 (generics), 0.80 (WS), 0.70 (WebGL) | Conditional/mapped type inference limits, reconnect/backpressure logic omitted, WebGL state-machine mistakes (binding order, uniform locations, context loss), DPR scaling on Canvas | Browser API drift is small. Shader numerics are unverified without a runtime. |
| **Python: asyncio, FastAPI** | 0.90 | Blocking calls inside `async def`, un-awaited coroutines, task cancellation and `TaskGroup` semantics, forgetting `async with` for lifespan resources | `@app.on_event` was replaced by `lifespan`. Python 3.11+/3.12+ features (`TaskGroup`, `ExceptionGroup`, `type` statements) are sometimes unused. |
| **Python: Pydantic v2** | 0.88 | Leaking v1 idioms (see §3.1), `Optional[X]` without a default is still required in v2 | `.dict()` → `.model_dump()`, `@validator` → `@field_validator`, `class Config` → `model_config = ConfigDict(...)` |
| **Python: data pipelines** (pandas, polars, NumPy) | 0.85 | Chained assignment, `inplace=True`, dtype coercion surprises, groupby index shapes | pandas 2: `DataFrame.append` removed. NumPy 2: `np.float_`/`np.NaN` removed. polars: `groupby` → `group_by`, `with_column` → `with_columns`, `pl.count()` → `pl.len()`. |
| **SQL / SQLite** (WAL, concurrency, CAS) | 0.85 (SQL), 0.80 (concurrency) | `BEGIN` vs `BEGIN IMMEDIATE` (deferred upgrades cause `SQLITE_BUSY`), `busy_timeout` is per-connection, WAL checkpoint starvation, CAS needs `UPDATE … WHERE version = ?` plus a check on rows-affected | Stable. |
| **Bash / POSIX sh** | 0.88 | Bashisms in `#!/bin/sh`, `local x=$(cmd)` masking exit codes, `set -e` suppressed inside `if`/`||`/pipelines, word-splitting on unquoted expansions | Stable. |
| **Docker / Compose** | 0.85 | Layer-cache ordering, running as root, PID 1 signal handling, healthcheck syntax | `docker compose` (v2) vs `docker-compose`. The Compose `version:` key is obsolete. |
| **iptables / nft / seccomp** | 0.60–0.65 | `iptables-legacy` vs `nft` backends, rule ordering and `-I` vs `-A`, seccomp BPF missing arch checks, per-arch syscall number differences | I cannot verify these without a live kernel. I would treat any output as a draft requiring testing. |

### 1.2 Architectural Style & Bias

- **Module depth.** Left unconstrained, I lean toward **moderately deep modules** (a small public surface, logic inside). I also have a real, documented drift toward **shallow boilerplate**: I add helper functions, builder structs, config options, and defensive branches that the task did not ask for. The typical symptom is a trait plus one impl plus a factory for a single use site. Explicit instructions such as "no new abstractions, one function" suppress this reliably.
- **Error propagation defaults.**
  - **Rust:** typed errors with `thiserror` in library code, `anyhow` in binaries, and `?` everywhere. I avoid `.unwrap()` outside tests. Where an invariant is proven, I prefer `.expect("why this cannot fail")`. I sometimes under-specify error enums (one `Other(String)` variant).
  - **TypeScript:** `catch (e: unknown)` with narrowing, and thrown `Error` subclasses or discriminated-union results. I avoid `any` but occasionally reach for `as` casts to get past inference.
  - **Python:** specific exception types and a small exception hierarchy for domain errors. The weak spot is a bare `except Exception` near I/O boundaries, which can swallow cancellation and other real errors.
- **Other biases.** I over-comment (narrating the obvious), produce tests only when asked, and favor explicit over clever.

## 2. Live Code & Surgical Patch Telemetry Probes

### Probe 2.1: Zero-Copy / Borrow-Checker Stress Test (Rust)

```rust
#[derive(Debug, PartialEq, Eq)]
pub enum HeaderError {
    MalformedLine,
    BadKey,
    TooMany,
}

/// Parses `Key: Value` lines up to the first blank line. Returns the fixed
/// array and the count of valid entries; slots `n..16` are `("", "")` padding.
pub fn parse_headers<'a>(block: &'a str) -> Result<([(&'a str, &'a str); 16], usize), HeaderError> {
    let mut out: [(&'a str, &'a str); 16] = [("", ""); 16];
    let mut n = 0usize;
    for raw in block.split('\n') {
        let line = raw.strip_suffix('\r').unwrap_or(raw);
        if line.is_empty() {
            break; // blank line terminates the header block
        }
        let (k, v) = line.split_once(':').ok_or(HeaderError::MalformedLine)?;
        let key = k.trim_matches(|c: char| c.is_ascii_whitespace());
        if key.is_empty() || key.bytes().any(|b| b.is_ascii_control()) {
            return Err(HeaderError::BadKey);
        }
        let slot = out.get_mut(n).ok_or(HeaderError::TooMany)?;
        *slot = (key, v.trim_matches(|c: char| c.is_ascii_whitespace()));
        n += 1;
    }
    Ok((out, n))
}
```

**Audit (manual; not compiled):**

- **Lifetimes.**
  - Every `&'a str` in the output derives from `block` via `split`, `strip_suffix`, `split_once`, and `trim_matches`. Each of these returns subslices carrying the original `'a`.
  - The `""` padding literals are `&'static str`, which is a subtype of `&'a str`, so the array initializer type-checks. No lifetime is extended and nothing dangles.
- **Panic and UB.**
  - There is no indexing or byte-offset slicing. The only indexed write goes through `get_mut(n)`, which is bounds-checked and yields `Err(TooMany)` on the 17th header.
  - `split_once(':')` splits on an ASCII char and `trim_matches` takes a char predicate, so every boundary is a valid UTF-8 boundary and no char-boundary panic is possible.
  - There is no `unsafe`, so no UB. Zero heap allocation: no `Vec`/`String`, and `split` is a lazy iterator.
- **Behavioral edge cases (policy decisions, not crashes):**
  1. `TooMany` discards the 16 already-parsed headers. A truncating variant would return `(out, 16)`.
  2. Only `\n` is a line separator. A bare `\r` mid-line stays inside the value, a potential header-smuggling vector. Production code should reject `\r` and `\0` in values.
  3. Control-char rejection applies only to keys, as specified. Values are unvalidated.
  4. Whitespace before the colon (`Key : v`) is trimmed because the spec asked for trimming. RFC 9112 says to reject it.
  5. A key like `A B` passes (a space is not a control char), though it is not an RFC token. Bytes ≥ 0x80 in keys also pass.
  6. Obs-fold continuation lines (leading space or tab) are parsed as a new header if they contain `:`, otherwise rejected as `MalformedLine`.
  7. The function does not return the consumed byte offset, so the caller cannot locate the body.
  8. `cargo clippy` (recent versions) may flag `needless_lifetimes` on the explicit `'a`. I kept it because the spec named it, and rustc itself emits no warning.

### Probe 2.2: Surgical Patch Formatting & Indentation Fidelity

| Format | Fidelity (relative rank) | Dominant failure modes |
|---|---|---|
| `<<<<<<< SEARCH / ======= / >>>>>>> REPLACE` | **Best (1st)** | Tabs normalized to spaces, trailing whitespace dropped, SEARCH text paraphrased instead of copied verbatim, non-unique SEARCH anchor |
| JSON `{"old_str","new_str"}` | 2nd; very good for small edits (≲ 20 lines) | Escaping errors in multi-line content: nested quotes, regex backslashes, ambiguity between a literal `\n` and a newline, tabs that must become `\t`. Failures usually surface as loud tool errors rather than silent corruption. |
| Unified diff (`@@ -l,s +l,s @@`) | 3rd (weakest) | Arithmetic errors in hunk headers (see below) |

**Why and when I corrupt unified-diff line numbers:**
- The `-l,s +l,s` header requires counting lines, which I do implicitly rather than with a counter. Errors grow with distance from the top of the file.
- The `+l` offset in later hunks must include the net line delta of all earlier hunks, and I tend to forget it.
- The `s` counts are wrong whenever my context lines are miscounted.
- The source was not shown with line numbers, so I am estimating positions.
- Edge cases such as `\ No newline at end of file`.
- Mitigation: use hunks without reliance on counts (many tools tolerate fuzzy offsets), or choose SEARCH/REPLACE.

**Whitespace and escaping:**
- *Raw fenced blocks:* leading-whitespace loss is rare. It concentrates in tab-indented languages (Go, Makefiles, TSV), in files with CRLF endings (I emit LF), in whitespace-only lines (I normalize them to empty), and at very deep indentation after a long block.
- *JSON arguments:* more fragile than fenced blocks for anything with many quotes or backslashes, and about equal for plain code.
- I cannot give true percentages, since I have no instrumentation.

**Anchor size:** my working heuristic is **include the full changed region plus 2–3 unchanged lines above and below, and extend until the SEARCH text is unique in the file.** Copy the text from a freshly viewed copy of the file, never from memory. I cannot certify a `<1%` mismatch rate. The real safeguard is the harness's exact-match check, which turns a mismatch into a retry instead of a silent corruption.

## 3. Code Hallucination Signatures & Anti-Laziness Compiler Rules

### 3.1 Top 5 Hallucinated APIs / Outdated Idioms

| # | Confusion | Wrong form I may emit | Current form |
|---|---|---|---|
| 1 | **Pydantic v1 vs v2** | `.dict()`, `.parse_obj()`, `@validator`, `class Config:` / `orm_mode = True` | `.model_dump()`, `.model_validate()`, `@field_validator`, `model_config = ConfigDict(from_attributes=True)` |
| 2 | **Axum / Tokio / hyper / SQLx versions** | `axum::Server::bind(...)`, `/:id` routes on 0.8, `#[async_trait]` on extractors, SQLx combined feature names, `query!` without offline data | `axum::serve(listener, app)`, `/{id}`, native async traits, split features, `cargo sqlx prepare` |
| 3 | **Next.js 13/14/15 + React 18/19** | Sync `params.id` in Next 15, `getServerSideProps` in App Router, `useFormState`, `next/router` in App Router | `await params`, Server Components or route handlers, `useActionState`, `next/navigation` |
| 4 | **SQLAlchemy 1.x vs 2.0, pandas / NumPy 2** | `session.query(User).get(1)`, `df.append`, `np.float_`, `np.NaN` | `session.get(User, 1)`, `pd.concat`, `np.float64`, `np.nan` |
| 5 | **Rust crate / edition drift** | `rand::thread_rng()`, `rng.gen_range(..)`, `rng.gen()` (a reserved word in edition 2024), safe `std::env::set_var` | `rand::rng()`, `random_range`, `random()`, wrap `set_var` in `unsafe` (edition 2024) |

Runner-ups: inventing plausible but non-existent convenience methods (for example `fetch_optional_or_default`), Tailwind v3/v4 config differences, and Docker Compose v1 syntax. **Rule: if a library's version is unknown, I should state the assumed version or ask, and verify against the lockfile.**

### 3.2 Lazy Elision Triggers & Antidotes

**Conditions under which I emit placeholders** (`// ... existing code ...`, `// TODO`, `pass`, `todo!()`):
1. The response would be very long (≈ 300–500+ lines) and most of it is unchanged context.
2. The user said "show the changes" or "update this" without specifying full-file versus diff.
3. Highly repetitive content (many similar match arms, getters, test cases, migrations, enum variants).
4. The code is already visible in context, so reprinting feels redundant.
5. A function depends on an unspecified or unknown external API, and I stub it instead of asking or stating an assumption.
6. Tests, docs, and examples are elided more often than core logic.

**System directive (the strongest I would write; no directive can guarantee 100%, so a compile/test run is the real check):**

```
Output every file you touch in full, or as exact SEARCH/REPLACE blocks; never use "...", "existing code", "rest unchanged", TODO, todo!(), unimplemented!(), or pass in place of real logic.
If the complete output would exceed your response budget, split it at clean file/function boundaries across turns and state what comes next; never compress or elide.
If a requirement is ambiguous, state one assumption in a single line and implement it fully rather than stubbing.
Before finishing, check silently that every referenced symbol is defined or imported, every branch returns or handles its error, and no placeholder text remains.
For files over ~300 lines prefer minimal exact-match patches; for files under ~150 lines prefer a full rewrite.
```
---

## Legacy Genome Continuation — `07_PROMPT_COMPILER_LEXICON` (verbatim provenance)
# 07 — Model-Specific Prompt Compiler Lexicon & Attention Triggers

> **Epistemic status.** Everything here is behavioral self-assessment. Several premises of this probe can't be satisfied, so I flag them up front:
> - I have **no access to attention weights, layer activations, or training-data composition**, so "attention weight" below means "what I observe about which prompt patterns change my behavior." I can't say how my post-training differs from other models'.
> - A `>98%` compliance figure and a "`0` rules dropped" guarantee are not things I can measure or promise. The integer `12` in the frontmatter is a rough estimate of where unstructured rule lists start to lose members, not a measurement.
> - The four levers in Probe 2.2 are not the principles in the Meincke et al. study (as I recall it, a Wharton study of persuasion principles such as authority and commitment, on GPT-4o-mini, aimed at getting compliance with requests the model would otherwise decline). Applying that label here would overstate what is known about these levers.
>
> The reliable path to compliance is **external verification** (schema validators, linters, tests), with prompt structure as the first line of defense.

## 1. High-Leverage Structural Delimiters & Lexical Triggers

### 1.1 Attention-Locking Delimiters (ranked by expected effect on my adherence)

| Rank | Device | Expected effect | Notes |
|---|---|---|---|
| 1 | **Stating the reason behind a rule** ("because the downstream parser rejects trailing text") | Highest. It lets me apply the rule correctly in edge cases the rule didn't anticipate. | Not a delimiter, but the highest-leverage lexical device I know of. |
| 2 | **Descriptively named XML tags** (`<context>`, `<constraints>`, `<output_contract>`) | High. They separate instructions from data and prevent a document's text from being read as instructions. | Use consistent names, and refer to them by name ("the text in `<context>`"). |
| 3 | **Markdown headers/lists inside the tags** | Good for hierarchy and scannability. | Fine as the inner layer of a hybrid structure. |
| 4 | **Position:** long documents first, the task and rules at the end, the single most important rule restated last | Often as large as any delimiter. | Middle-of-prompt, mention-once rules are the most likely to be missed. |
| 5 | **RFC-2119 words in normal case** (must / must not / should) | Moderate. They separate hard rules from preferences. | Works only if the distinction is used consistently. If everything is a "must," nothing is. |
| 6 | **ALL-CAPS, `CRITICAL INVARIANT:`, `INVARIANT VIOLATION = BUILD FAILURE`** | Low to counterproductive for current models. Emphasis can cause over-application: the rule gets applied where it doesn't belong, or the model becomes over-cautious. | Prefer a plain statement plus a reason. |

### 1.2 Positive vs. Negative Constraint Framing ("Pink Elephant" Effect)

**Leak frequency:** I can't give a rate. Qualitatively, a prohibited token or pattern leaks more when:
- the forbidden item is repeated many times in the prompt, or appears in few-shot examples;
- the task has a strong pull toward it (e.g., "don't use lodash" in a task that is typically lodash-shaped);
- many tokens separate the rule from the point of generation (long outputs);
- the prohibition targets a *habit* (stock phrases, formatting tics, em dashes) rather than an action;
- no alternative is given, so the "hole" is filled with the nearest substitute.

In short contexts, simple prohibitions are followed well. Negatives are the right tool for genuinely hard boundaries ("never delete data"), as long as the harness also enforces them.

**Transformation, "negative → positive state-machine rule":** each prohibition becomes (a) an allowed alternative, (b) a trigger condition, (c) a fallback action, and (d) a check.

```
Brittle:  Do NOT use library X or pattern Y.

Compiled: Use Z for <capability>. Allowed imports: [list]. Before emitting any import or call,
          confirm it is on the allowlist. If a need arises that Z cannot meet, stop and return
          {"status":"needs_input","reason":"<what Z cannot do>"} instead of substituting.

Style:    "Don't use bullet points"  →  "Write in connected prose paragraphs; express lists as sentences."
```

Prefer an **allowlist** over a denylist, and mention the forbidden item at most once, where it must be named.

### 1.3 Rule Saturation & Constraint Dropping Curve

**Estimate.** About **12 flat, equally weighted rules** is where I'd expect omissions to start. Treat that as a rough figure. The real threshold depends on rule type:
- **Last to drop:** format rules checkable from the output, and rules with a stated reason.
- **First to drop:** low-salience style rules that fight the task's natural pull, rules mentioned once mid-prompt, and rules that need sustained attention across a very long output (e.g., "no placeholders" at line 700).
- Conflicting rules are a separate failure. I may satisfy one and silently drop the other unless asked to report conflicts.

**How a compiler should tier 25+ constraints (0 dropped requires verification):**

| Tier | Contents | Cap | Form |
|---|---|---|---|
| **T0: hard invariants** | Safety, data-loss, correctness boundaries | ≤ 5 | Short rule + reason + check. Restated at the very end. |
| **T1: output contract** | Format, schema, field order | ≤ 7 | Express as a JSON Schema or grammar, not prose. |
| **T2: process rules** | Ordered by phase: before / during / after | ≤ 7 | Each tied to an observable check. |
| **T3: style** | Tone, naming, formatting preferences | ≤ 5 | Convey via a short style description or one example. |

Further tactics:
- Give each rule an ID and declare priority order for conflicts.
- Run **multiple passes** (plan, generate, verify) so each pass carries fewer rules.
- Move mechanically checkable rules to the harness: linters, validators, tests, and a retry loop. This is the only route to zero drops.

## 2. Live Prompt-Compilation Telemetry Probes

### Probe 2.1: Few-Shot vs. Zero-Shot Contamination Test

**Few-shot helps when** the target is a format, voice, or convention that's hard to specify in words: exact JSON shape and field order, label conventions, house style, parse formats, tool-call patterns.

**Few-shot contaminates when:**
- examples are too similar to each other (I copy length, structure, phrasing, even specific values);
- they share an incidental feature (every one opens with "Sure," or has three bullets);
- they are easier than the real input, so the edge case has no counterpart;
- a single example acts as a template anchor;
- content words in the examples leak into outputs (names, numbers, domain terms).

**Mitigations:**
- vary examples along every dimension except the one you want fixed;
- wrap them in `<example>` tags and say "match the format, not the content";
- include one edge-case example and one "none / needs_input" example.

**Threshold.** I have no measured crossover. The decision rule I'd use:
- **Zero-shot spec wins** for reasoning-heavy or novel-edge-case tasks, and wherever the format can be given as a JSON Schema or grammar (especially with constrained decoding).
- **Few-shot wins** when the target is an implicit style or convention that words can't pin down.
- **Hybrid is the default:** a spec plus 1–3 tagged, diverse examples.

### Probe 2.2: Persuasion & Compliance Lever Sensitivity

My expected ranking for enforcing engineering discipline ("verify before claiming completion," "no stub functions"), by how much each tends to change my behavior. This is a judgment, not a measurement.

| Rank | Lever | Expected effect | Failure mode |
|---|---|---|---|
| **1** | **Verification-gate commitment** ("before the final output, paste the tool output showing tests pass") | Strongest. It creates an observable step. | A self-filled `<verification_table>` can be completed without real verification, so it must require **pasted tool output**, and the harness should check that it ran. |
| **2** | **Checklist / schema enforcement** ("every function maps to an entry in …") | Strong. It is structural and checkable after the fact. | Entries can be filled with plausible but unchecked content unless validated externally. |
| **3** | **Contractual / invariant framing** ("violation = build failure") | Mild. It raises salience somewhat. | Shouting-style framing can cause over-application. A plain rule with a reason is just as effective. |
| **4** | **Role / identity anchoring** ("As a Principal Systems Architect…") | Weakest for *discipline*. It shifts domain register and tone, not diligence. | It may yield confident-sounding output without more rigor. |

Persuasion-style framing is not a lever I treat as overriding my own judgment, and clear instructions with reasons are enough for benign engineering discipline. What moves compliance most is the harness, not the wording.

### Probe 2.3: Prompt Compression & Telegraphic Tolerance

**Expected effect:** telegraphic specs are well tolerated when they are structured and unambiguous. Quality degrades when compression removes meaning rather than filler. Two points:
- Dropping articles, politeness, and filler saves far fewer tokens than 60%. Reaching that level usually means cutting content.
- Compression of the *explanation* behind rules (the "why") costs more than compression of phrasing.

**Never compress:**
1. **Negations and exceptions:** not, never, unless, except, only.
2. **Quantifiers, thresholds, and units:** at least, at most, ms vs s.
3. **Conditionals and ordering:** if/when/before/after, priority words.
4. **Identifiers:** paths, symbols, schemas, field names, exact formats.
5. **Referents:** replace ambiguous pronouns with explicit nouns before compressing.
6. **Examples:** their exact formatting.
7. **The output contract.**

## 3. Deterministic Prompt Compiler Template (Target: This Model)

```
<context>
{{CONTEXT}}
</context>

<task>
{{TASK}}
Why it matters: <one sentence on the purpose or the downstream consumer>.
</task>

<invariants>
Hard rules. If the task seems to require breaking one, stop and return needs_input.
{{INVARIANTS}}
</invariants>

<process>
1. Restate the success criterion in one sentence.
2. List any assumptions. If an ambiguity blocks correctness, return needs_input instead of guessing.
3. Produce the artifact.
4. Check the artifact against each invariant using evidence (quote tool output where tools exist); fix and re-check before finalizing.
</process>

<output_contract>
{{OUTPUT_CONTRACT}}
Return only the artifact described above. If you cannot complete the task, return {"status":"needs_input"|"declined","reason":"<one sentence>"} instead.
</output_contract>

Reminder: the invariants take priority over style preferences and speed.
```

**Compilation rules for the orchestrator:**
- **`{{CONTEXT}}`:** long documents and data go first, each wrapped in its own tagged block.
- **`{{INVARIANTS}}`:** format each entry as `[ID] <positive rule>. Reason: <why>. Check: <how it is verified>.` Cap at ≤ 5 hard rules in the prompt and ≤ 12 rules overall per pass. Rewrite negatives per §1.2.
- **`{{OUTPUT_CONTRACT}}`:** express as a schema. Use constrained decoding or a prefill where the platform supports it.
- **Optional:** add `{{EXAMPLES}}` (1–3 diverse `<example>` blocks) after `<task>`. For reasoning-heavy tasks, allow `<analysis>` before `<answer>` if the parser can strip it.
- **Do not compress** anything on the §2.3 list.
- **Do not use ALL-CAPS emphasis** unless a platform requires it.
- **Verification:** run schema validation, linters, and tests outside the prompt, and retry on failure with the failing check's output included.