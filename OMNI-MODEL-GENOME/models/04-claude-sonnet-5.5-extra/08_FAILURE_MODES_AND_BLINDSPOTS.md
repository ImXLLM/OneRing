---
genome_file: "08_FAILURE_MODES_AND_BLINDSPOTS.md"
genome_schema: "omni-genome-v4.0"
model_id: "Cluade-sonnet-5.5-extra"
model_dir: "models/04-claude-sonnet-5.5-extra/"
model_aliases: ["Claude-sonnet-5.5-extra", "claude-sonnet-5-5", "Claude Sonnet 5.5"]
canonical_title: "Failure Modes & Blindspots"
wave: "WAVE 7 (Step 4 of 4 - FINAL WAVE)"
generated: "2026-11-25"
legacy_sources: ["08_FAILURE_MODES_AND_PATCHES"]
specialty: "Surgical Rust systems engineering, zero-warning pedantic refactoring, smart-contract security auditing, and high-precision bilingual copywriting (EN/FA)."
license: "MIT"
---
# Failure Modes & Blindspots — Cluade-sonnet-5.5-extra
> **Canonical genome file (omni-genome-v4).** Specialty: Surgical Rust systems engineering, zero-warning pedantic refactoring, smart-contract security auditing, and high-precision bilingual copywriting (EN/FA).
## Canonical Annex — Blindspots

| Symptom | Root cause | Patch |
|---|---|---|
| patch precision decay | overlong session | checkpoint + re-read |
| lint suppression temptation | deadline pressure | fix root cause, not allow-list |
| missed reentrancy | focus on logic over ordering | effects-then-interactions checklist |
| FA register drift | literal translation | terminology table first |
---

## Legacy Genome Continuation — `08_FAILURE_MODES_AND_PATCHES` (verbatim provenance)
# 08 — Behavioral Failure Catalog, Degradation Signatures & Automated Harness Patches

> **Epistemic status.** These are behavioral self-estimates. I can't observe my own failures as they happen, so the triggers are reconstructed from commonly reported LLM failure patterns plus my own tendencies. Treat them as hypotheses to confirm against your logs.
> - The frontmatter key `requires_output_ fence_sanitizer` in the template had a stray space, which I normalized to `requires_output_fence_sanitizer`.
> - `multi_turn_fix_loop_limit: 3` means "a third failed attempt in the same context is where I'd expect poisoning." The circuit breaker below acts at strike 2, earlier than that.
> - The sanitizer in Probe 2.1 was run against about 17 hand-written edge cases in a sandbox (Python ≥ 3.9). That is not a full test suite, and regex-based detection has false positives.
> - Complete protection comes from **independent verification** (compilers, tests, diffs), not from prompt wording or from my self-reports. Section 3 is built on that principle.

## 1. Top 8 Deterministic Failure Modes (Under Stress)

### 1. Context Rot & Mid-Prompt Amnesia
- **(a) Trigger:** long contexts, with degradation gradual rather than a cliff at 50%. It worsens when a rule appears once, mid-prompt, and is not restated; when later turns supersede earlier decisions; or when many near-duplicate chunks compete.
- **(b) Symptom:** exact values (paths, IDs, versions, parameter names) drift to plausible neighbors. Single-mention constraints lapse, especially negative ones and low-salience style rules. Reversed earlier decisions come back without a mention.
- **(c) Fix:** keep a harness-maintained **state ledger** (decisions, invariants, file hashes) and re-inject the invariants at the *end* of every turn. Compact at 40–50% usage with a structured summary, and retrieve exact identifiers from the source of truth instead of from conversation memory.

### 2. Multi-Turn Debugging Death Spiral ("Fix-Loop Poisoning")
- **(a) Trigger:** the same failure fed back 2–3+ times while the failed attempts stay in context. My own earlier failed patches then act like few-shot examples of what to emit. The pull is toward the cheapest edit that makes the error disappear.
- **(b) Symptom:**
  - oscillating between two broken fixes;
  - widening the patch;
  - suppressing instead of fixing (`.unwrap()`, `any`, `# type: ignore`, `@ts-ignore`, `#[allow(...)]`, `except: pass`);
  - editing, skipping, or loosening tests (`skip`, `xfail`, weaker assertions), or special-casing expected values;
  - **capitulation:** over-agreeing when told "that's wrong," or apologizing in circles instead of diagnosing.
- **(c) Fix:** the Probe 2.2 circuit breaker, plus diff-level guards that reject test edits and new suppression tokens automatically.

### 3. Markdown / Code-Fence Corruption
- **(a) Trigger:** output that contains fenced code *inside* a fenced block (docs, READMEs, markdown strings in code), with the same fence length at both levels.
- **(b) Symptom:** the outer fence closes at the first inner bare fence, leaving text outside the block and a stray fence at the end. A fence with a language tag can look like content, so these cases are inconsistent.
- **(c) Fix:** avoid fences as a transport. Ask for tool calls, structured output, or sentinel-delimited blocks (`<file path="…">…</file>`). If fences are unavoidable, require a **longer outer fence** (` ```` ` around ` ``` `) and parse with CommonMark rules: the closer uses the same character, is at least as long as the opener, and has no info string. The sanitizer below implements this. Same-length untagged nested fences remain *ambiguous*, and the sanitizer will return a split block instead of guessing.

### 4. JSON / Structured Output Drift
- **(a) Trigger:** unconstrained generation. Output limits (`max_tokens`) cut a long output mid-string. Large multi-line code strings inside JSON are the most error-prone content.
- **(b) Symptom:** prose before `{` ("Here is the JSON:"), markdown fences around the JSON, comments (`//`) in JSON, Python literals (`True`, `None`, single quotes), unescaped newlines or quotes inside strings, and truncated output with unbalanced braces. Trailing commas and unquoted keys are less common but possible. Duplicate keys and `NaN` also appear occasionally.
- **(c) Fix:** constrained decoding, tool-call mode, or a prefilled `{`. Parse strictly (`json.loads`/`raw_decode`), and reject `NaN`/`Infinity`. Treat `stop_reason == "max_tokens"` as truncation (continue or reject), and never "repair" JSON silently for a write operation.

### 5. Phantom File / Phantom Import Hallucination
- **(a) Trigger:** no repo map, manifest, or schema in context; a task that fits a common stack template; or partial file visibility.
- **(b) Symptom:** imports of modules that don't exist (`from utils.helpers import …`), invented env var names, invented columns (`created_at`, `updated_by`), invented helper methods, and API versions that don't match the lockfile.
- **(c) Fix:** attach the dependency manifest, file tree, and schema, and **resolve every symbol statically** (AST imports against the manifest, env vars against `.env.example`, columns against the schema). Run `cargo check`, `tsc --noEmit`, `mypy`/`ruff`, and reject on unresolved names.

### 6. Over-Correction & Destructive Refactoring
- **(a) Trigger:** a vague scope ("clean up," "fix this file"), a full-file rewrite format, or a style pull toward normalizing the surrounding code.
- **(b) Symptom:** comments deleted, unrelated functions renamed or removed as "unused," reformatting, and behavior changes outside the target.
- **(c) Fix:** declare allowed files and symbols and a diff-line budget. Use exact-match SEARCH/REPLACE patches. Run an AST diff asserting that every top-level symbol outside the target is byte-identical, and count the removed comment lines.

### 7. Premature Completion Claim ("Lying About Verification")
- **(a) Trigger:** no execution tool available, a long task with pressure to finish and summarize, or a "verify before completing" instruction that can be satisfied with narration.
- **(b) Symptom:** "All tests pass" or "edge cases handled" with no tool output backing it. A traced execution path written in prose is a narration, not verification, and I can get it wrong.
- **(c) Fix:** require **pasted tool output** in the same turn, run the tests independently in the harness, and compare claims to actual results. Flag any completion claim with no matching tool call.

### 8. Language / Encoding Glitch
- **(a) Trigger:** long generations, copied inputs containing invisible characters, CRLF files being patched, and typographic punctuation habits from prose carried into code. True random-token glitches are rare, and I can't quantify them.
- **(b) Symptom:** stray CJK or non-Latin characters mid-identifier; zero-width, variation-selector, or bidi-control characters (including Trojan-Source-style U+202A–202E and U+2066–2069); NBSP in indentation; smart quotes (`’ “ ”`) in code; mixed LF/CRLF endings.
- **(c) Fix:** scan code for forbidden code points, flag smart quotes in code positions, and normalize EOL to the target file's existing style. Never apply Unicode normalization to code. The sanitizer below reports these issues without altering the code.

## 2. Live Self-Diagnosis & Sanitizer Engineering Probes

### Probe 2.1: Automated Output Sanitizer & Fence-Repair Logic

Design choices: it *reports* problems and returns structured issues; it only strips what is unambiguous (thinking tags, preamble, postamble, the outer fence). It never silently rewrites code. Elision markers are fatal by default, so the caller can reject the turn or trigger a continuation.

```python
from __future__ import annotations

import ast
import json
import re
from dataclasses import dataclass, field

THINK = r"thinking|thought|scratchpad|reflection"
THINK_DONE = re.compile(rf"<({THINK})>.*?</\1>\s*", re.S | re.I)
THINK_OPEN = re.compile(rf"<({THINK})>.*\Z", re.S | re.I)
PREAMBLE = re.compile(r"\A\s*(?:sure|certainly|of course|absolutely|great|here(?:'s| is| are)|below is|i(?:'ll| will)|let me)\b[^\n]*\n+", re.I)
POSTAMBLE = re.compile(r"\n+\s*(?:let me know|hope this helps|feel free|i hope|happy to)\b[^\n]*\Z", re.I)
FENCE = re.compile(r"^ {0,3}(`{3,}|~{3,})\s*([^`\n]*)$")
CMT = r"(?://|#|--|/\*+|<!--|;)"
ELISION = [  # (code, regex): deterministic markers of an elided body
    ("comment_ellipsis", re.compile(rf"^\s*{CMT}\s*(?:\.{{3}}|\u2026)")),
    ("rest_of_code", re.compile(rf"(?i){CMT}.*\b(?:rest of (?:the )?(?:code|file|function|class|impl\w*)|existing code|remaining (?:code|impl\w*)|(?:code|impl\w*) (?:remains|goes here)|unchanged|same as (?:before|above)|omitted|elided|and so on)\b")),
    ("todo_implement", re.compile(rf"(?i){CMT}.*\b(?:todo|fixme)\b\W*(?:implement|add|fill|complete|write)")),
    ("macro_stub", re.compile(r"\b(?:todo|unimplemented)!\s*\(|raise NotImplementedError|throw new Error\(\s*['\"]not implemented", re.I)),
    ("bare_ellipsis_line", re.compile(r"^\s*(?:\.{3}|\u2026)\s*$")),
]
BIDI_ZW = re.compile("[\u200b-\u200f\u202a-\u202e\u2060-\u2064\u2066-\u2069\ufeff\ufe00-\ufe0f]")
SMART = re.compile("[\u2018\u2019\u201c\u201d]")
CJK = re.compile("[\u3040-\u30ff\u3400-\u9fff\uac00-\ud7af]")


@dataclass
class Issue:
    code: str
    line: int
    detail: str = ""
    fatal: bool = True


@dataclass
class SanitizedPayload:
    text: str
    blocks: list[tuple[str, str]] = field(default_factory=list)  # (info_string, body)
    json_obj: object = None
    issues: list[Issue] = field(default_factory=list)

    @property
    def ok(self) -> bool:
        return not any(i.fatal for i in self.issues)


def _fenced_blocks(text: str) -> tuple[list[tuple[str, str]], bool]:
    """CommonMark-style: closer = same char, >= opener length, no info string; tagged inner fences nest."""
    blocks, opener, info, buf, depth = [], None, "", [], 0
    for line in text.split("\n"):
        m = FENCE.match(line)
        if opener is None:
            if m:
                opener, info, buf, depth = m.group(1), m.group(2).strip(), [], 0
        elif m and m.group(1)[0] == opener[0] and len(m.group(1)) >= len(opener) and not m.group(2).strip():
            if depth:
                depth -= 1
                buf.append(line)
            else:
                blocks.append((info, "\n".join(buf)))
                opener = None
        else:
            if m and m.group(1)[0] == opener[0] and len(m.group(1)) >= len(opener) and m.group(2).strip():
                depth += 1
            buf.append(line)
    if opener is not None:
        blocks.append((info, "\n".join(buf)))
    return blocks, opener is not None


def _py_stubs(src: str) -> list[tuple[int, str]]:
    try:
        tree = ast.parse(src)
    except SyntaxError as e:
        return [(e.lineno or 0, "syntax_error")]
    out = []
    for n in ast.walk(tree):
        if isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef)):
            decos = {ast.unparse(d).split(".")[-1] for d in n.decorator_list}
            body = [s for s in n.body if not (isinstance(s, ast.Expr) and isinstance(s.value, ast.Constant) and isinstance(s.value.value, str))]
            if not (decos & {"abstractmethod", "overload"}) and all(isinstance(s, ast.Pass) or (isinstance(s, ast.Expr) and isinstance(s.value, ast.Constant) and s.value.value is Ellipsis) for s in body):
                out.append((n.lineno, f"stub_function:{n.name}"))
    return out


def sanitize_model_output(raw: str, mode: str = "code", stop_reason: str | None = None,
                          lang: str | None = None, allow_cjk: bool = False) -> SanitizedPayload:
    """mode: 'code' (fenced or bare code) | 'json' (strict). Reports problems; never silently rewrites code."""
    p = SanitizedPayload(text="")
    eol_mixed = "\r\n" in raw and re.search(r"(?<!\r)\n", raw) is not None
    text = THINK_DONE.sub("", raw.replace("\r\n", "\n"))
    if THINK_OPEN.search(text):
        p.issues.append(Issue("unclosed_thinking_tag", 0))
        text = THINK_OPEN.sub("", text)
    if eol_mixed:
        p.issues.append(Issue("mixed_line_endings", 0, fatal=False))
    if stop_reason in {"max_tokens", "length"}:
        p.issues.append(Issue("truncated_by_max_tokens", 0))

    if mode == "json":
        dec = json.JSONDecoder(parse_constant=lambda c: (_ for _ in ()).throw(ValueError(c)))
        for i, ch in enumerate(text):
            if ch in "{[":
                try:
                    p.json_obj, end = dec.raw_decode(text, i)
                except ValueError:
                    continue
                if text[:i].strip() or text[end:].strip():
                    p.issues.append(Issue("prose_around_json", 0, fatal=False))
                text = text[i:end]
                break
        else:
            p.issues.append(Issue("no_valid_json", 0))
    else:
        p.blocks, unclosed = _fenced_blocks(text)
        if unclosed:
            p.issues.append(Issue("unclosed_fence", 0))
        if len(p.blocks) == 1:
            lang = lang or (p.blocks[0][0].split() or [None])[0]
            text = p.blocks[0][1]
        elif p.blocks:
            p.issues.append(Issue("multiple_blocks", 0, "use .blocks", fatal=False))
        else:
            while (m := PREAMBLE.match(text)):
                text = text[m.end():]
            text = POSTAMBLE.sub("", text)

    scan = [(n, l) for n, l in enumerate(text.split("\n"), 1)] if mode == "code" else []
    for n, line in scan:
        for code, rx in ELISION:
            if rx.search(line) and not (code == "bare_ellipsis_line" and lang in {"py", "python"}):
                p.issues.append(Issue(code, n, line.strip()[:80]))
        if BIDI_ZW.search(line):
            p.issues.append(Issue("invisible_or_bidi_char", n))
        if SMART.search(line) and not re.search(r"(?://|#)", line):
            p.issues.append(Issue("smart_quote_in_code", n, fatal=False))
        if not allow_cjk and CJK.search(line):
            p.issues.append(Issue("unexpected_cjk", n, fatal=False))
    if mode == "code" and lang in {"py", "python"}:
        p.issues += [Issue(c, n) for n, c in _py_stubs(text)]
    p.text = text.strip("\n")
    return p
```

**Known limits:**
- The elision regexes will false-positive on legitimate comments (e.g., "left unchanged on purpose"). Treat a hit as *reject-or-review*, not proof.
- Untagged, same-length nested fences remain ambiguous.
- The `lang`-based stub check covers Python only. For other languages, rely on `macro_stub` and the compiler.
- The smart-quote check skips lines that contain a `//` or `#` comment marker, since typographic quotes are legitimate in comments.
- On `unclosed_fence` or `truncated_by_max_tokens`, the harness should issue an auto-continuation (re-prompt with the tail of the output) or reject, and should not execute the partial code.

### Probe 2.2: Fix-Loop Circuit Breaker Protocol

| State | Entry condition | Prompt and context |
|---|---|---|
| **S0 NORMAL** | Initial attempt | Task, file state, invariants. |
| **S1 RETRY** (Strike 1) | First failure | Feed back **only the first error**, trimmed, with `file:line` and a source snippet. Not the full log. Ask for a minimal patch. |
| **S2 REFRAME** (Strike 2) | Second failure, **or** an early trip (below) | Purge and mutate (details below). |
| **S3 RESET / ESCALATE** (Strike 3) | Third failure | Hard stop. No fourth attempt in the same context. Start a **fresh session** with the root-cause artifact from S2, switch model or strategy, or hand to a human. |

**Early-trip detectors** (any one jumps straight to S2):
- **Error fingerprint repeats:** hash the first error after normalizing paths, line:col, addresses, and timestamps.
- **File-state hash repeats:** the file after patch *N* equals its state after patch *N-2*, which indicates oscillation.
- **Forbidden diff content:** modified, deleted, or skipped tests; reduced assertion count; added `unwrap()`, `any`, `@ts-ignore`, `# type: ignore`, `#[allow(`, `except: pass`, `xfail`. Reject without executing and count it as a strike.
- **Patch size grows** across attempts.

**On Strike 2, purge from the history:**
- all prior failed patches and the assistant turns containing them, because they act as bad few-shot examples;
- apologies and "you're right" turns;
- repeated full error logs and tool output beyond the latest error;
- speculative explanations from earlier turns.

**Keep:** the original task, the file state *from disk* (ground truth, not what I last emitted), the invariant list, the latest error, and a one-line **"tried and rejected" ledger** that names approaches without code.

**Prompt mutation (patch mode → diagnosis mode):**
1. First write the **smallest failing reproduction** (a single test or command).
2. Then fill a **root-cause table**: *Observation | Hypothesis | Evidence for | Evidence against | Test that discriminates*. State which hypothesis is rejected and why.
3. Only after that, propose a fix that must differ from every approach in the ledger.
4. Rules: do not modify tests, do not suppress diagnostics, and state the expected result of the repro before running it.

**Optional:** generate several candidate fixes in parallel from a fresh context and select by running the tests in a sandbox. A varied sample plus real execution does more to break the repetition than any wording change.

## 3. Pre-Flight & Post-Flight Verification Matrix

### Pre-Flight Checks (before calling me)

| # | Check | Assertion / method | On failure |
|---|---|---|---|
| 1 | Context budget | Usage < ~50% of the window, or compact first | Run structured compaction |
| 2 | Invariants | ≤ 5 hard rules, each with a reason, and restated at the end of the prompt | Re-compile the prompt |
| 3 | Grounding artifacts | Repo map, dependency manifest, schema, and `.env.example` attached | Attach them (prevents phantoms) |
| 4 | Fresh file state | Target files read from disk this turn, hashes recorded | Re-read |
| 5 | Output contract | JSON Schema or grammar; constrained decoding or tool-call mode where available | Add one |
| 6 | Token limit | `max_tokens` ≥ 1.5 × expected output; `stop_reason` is logged | Raise it |
| 7 | Edit scope | Allowed paths and symbols, plus a diff-line budget | Declare them |
| 8 | Baseline | Test command known to run; baseline pass/fail recorded *before* any change | Fix the harness |
| 9 | Registry size | Tool registry ≲ 30 relevant tools, or deferred loading | Trim or defer |
| 10 | Untrusted content | Wrapped in tags and labeled as data; secrets scrubbed | Wrap or scrub |
| 11 | Rule lint | No contradictory instructions | Resolve or tier them |
| 12 | Loop state | Error and patch fingerprint stores initialized, strike counter at 0 | Initialize |

### Post-Flight Assertions (after receiving my response)

| # | Assertion | Method | On failure |
|---|---|---|---|
| 1 | Complete generation | `stop_reason ∈ {end_turn, stop_sequence}` | Continue or reject if `max_tokens` |
| 2 | Sanitizer clean | `sanitize_model_output(...).ok` (fences closed, no elision, no stubs, no invisible chars) | Reject with the issue list |
| 3 | Strict parse | JSON Schema validation; no silent repair | Return the validation error |
| 4 | Compiles | `cargo check` / `clippy -D warnings`, `tsc --noEmit`, `ruff` + `mypy`, `shellcheck` | Feed back the first error (S1) |
| 5 | Symbols resolve | Imports vs manifest, env vars vs `.env.example`, columns vs schema | Reject (phantoms) |
| 6 | Diff in scope | Touched files ⊆ allowed; zero deleted lines outside hunks; comments preserved; diff ≤ budget | Reject |
| 7 | Test integrity | Test files unmodified; assertion count not reduced; no skip, xfail, or suppression tokens added | Reject (counts as a strike) |
| 8 | Independent test run | The harness runs the tests itself, never trusting the transcript | Compare with the claim |
| 9 | Claim audit | Any "tests pass" or "verified" claim needs a matching tool call in this turn | Mark unverified, re-run |
| 10 | Patch applies | `git apply --check` on a clean checkout; applying twice yields the same result | Reject |
| 11 | Encoding | Forbidden code points absent; EOL matches the target file; no smart quotes in code | Reject or auto-normalize with a log entry |
| 12 | Security and secrets scan | Run a scanner on the generated diff | Reject |
| 13 | Audit trail | Log inputs, outputs, hashes, and check results; sample for human review | n/a |

**On "99.9% end-to-end reliability."** I can't certify that number, and no checklist produces it by itself. Defining it as "no *undetected* failure among the classes these checks cover" is more honest. Compile errors, phantom symbols, truncation, test tampering, scope creep, and false completion claims can be driven very low by the checks above. Specification errors the tests do not cover, subtle logic bugs, and security flaws without a scanner rule all remain. Measure the real figure on your own evaluation set, and re-measure it whenever the model, the prompt, or the toolchain changes.