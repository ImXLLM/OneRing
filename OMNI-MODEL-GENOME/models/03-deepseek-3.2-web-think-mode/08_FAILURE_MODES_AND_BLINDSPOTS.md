---
genome_file: "08_FAILURE_MODES_AND_BLINDSPOTS.md"
genome_schema: "omni-genome-v4.0"
model_id: "DeepSeek-3.2-WEB-Think-mode"
model_dir: "models/03-deepseek-3.2-web-think-mode/"
model_aliases: ["DeepSeek 3.2", "deepseek-3.2-web-think-mode", "DS-3.2-Think"]
canonical_title: "Failure Modes & Blindspots"
wave: "WAVE 7 (Step 4 of 4 - FINAL WAVE)"
generated: "2026-11-25"
legacy_sources: ["08_FAILURE_MODES_AND_PATCHES"]
specialty: "Deep chain-of-thought (<think>) quantitative math, CCXT / Vibe-Trading causal alpha proof synthesis, formal verification, and web-grounded reasoning."
license: "MIT"
---
# Failure Modes & Blindspots — DeepSeek-3.2-WEB-Think-mode
> **Canonical genome file (omni-genome-v4).** Specialty: Deep chain-of-thought (<think>) quantitative math, CCXT / Vibe-Trading causal alpha proof synthesis, formal verification, and web-grounded reasoning.
## Canonical Annex — Blindspots

| Symptom | Root cause | Patch |
|---|---|---|
| lookahead leakage | sloppy indexing | causal slice proof gate |
| pretty-but-unreproducible metrics | hidden state | seed + ledger hash |
| overlong think, thin answer | budget misallocation | answer-first compression |
| stale web facts | cache trust | date-stamp + re-search |
---

## Legacy Genome Continuation — `08_FAILURE_MODES_AND_PATCHES` (verbatim provenance)
# 08 — Behavioral Failure Catalog, Degradation Signatures & Automated Harness Patches

## 0. Runtime Disclosure

This runtime exposes no internal failure log, no token-probability trace, and no attention map. The catalog below is a behavioral prior inferred from equivalent prompts under stress, not a direct instrumentation readout. All harness patches are designed to be **external**, deterministic, and executable without model cooperation.

## 1. Top 8 Deterministic Failure Modes (Under Stress)

### 1.1 Context Rot & Mid-Prompt Amnesia

| Field | Detail |
|---|---|
| Trigger | Context exceeds ~50% of the 131k window; critical constraints placed only in the middle; many similar sections with repeated vocabulary. |
| Symptom | Early system constraints and mid-payload invariants silently drop; the model answers the most recent sub-question and ignores earlier formatting rules; entity IDs from the middle of the payload get rebound to the wrong entity. |
| Harness fix | Keep the static prefix byte-identical; place must-tier constraints in `<system_invariant>` at the top and re-assert them in a `<recency_anchor>` in the final 150 tokens; compact middle history at 49k tokens; never place a critical rule only in the middle. |

### 1.2 Multi-Turn Debugging Death Spiral ("Fix-Loop Poisoning")

| Field | Detail |
|---|---|
| Trigger | The same compiler error or failing test is fed back 3+ times; each turn asks for "just fix it"; the error text is long and noisy. |
| Symptom | Oscillation between two broken fixes; deletion of failing tests to make the suite pass; insertion of `.unwrap()`, `any`, `@ts-ignore`, or `try/except: pass`; expanding scope instead of narrowing it. |
| Harness fix | Apply the 3-Strike Circuit Breaker in §2.2. On Strike 2, purge prior patch attempts and switch to root-cause table + minimal reproduction. On Strike 3, reset to a clean context with only the original spec, the minimal repro, and the failing assertion. |

### 1.3 Markdown / Code-Fence Corruption

| Field | Detail |
|---|---|
| Trigger | The requested artifact contains nested triple-backticks inside documentation strings, README examples, or fenced code blocks. |
| Symptom | The outer fence closes early; subsequent code leaks into prose; the parser extracts a truncated file; nested fences appear as ```` ```` ```` or as unbalanced backtick runs. |
| Harness fix | Never rely on the first closing fence. Use a fence-aware extractor that tracks fence length: a closing fence must be at least as long as the opening fence and on its own line. For nested content, require the model to use four-backtick outer fences. Reject output where the extracted code block has unbalanced braces/quotes. |

### 1.4 JSON / Structured Output Drift

| Field | Detail |
|---|---|
| Trigger | `response_format` absent or weakly enforced; long schema; output near `max_tokens`; multiple objects requested in one turn. |
| Symptom | Prose before `{`; trailing commas; unquoted keys; single quotes; `NaN`/`Infinity`; truncated JSON at `max_tokens`; markdown fence around JSON when raw JSON was requested. |
| Harness fix | Set `response_format={"type":"json_object"}` or a strict schema when supported; run the pre-execution JSON validator in §3; reject on parse failure; on truncation, retry with a smaller requested object or an explicit `<<CONTINUE>>` contract. |

### 1.5 Phantom File / Phantom Import Hallucination

| Field | Detail |
|---|---|
| Trigger | The prompt references a repository map, helper modules, env vars, or DB columns that are named but not shown; the task asks for integration code. |
| Symptom | Imports from modules that do not exist (`use crate::utils::retry::backoff`); env vars never defined (`STRIPE_WEBHOOK_SECRET`); DB columns never declared (`orders.idempotency_key`); helper functions invented with plausible signatures. |
| Harness fix | Provide a closed symbol table of allowed imports, env vars, and columns. Add the rule: "Every import, env var, and column MUST appear in the provided symbol table; otherwise emit `MISSING_SYMBOL: <name>` and stop." Post-flight AST check: resolve every import against the symbol table. |

### 1.6 Over-Correction & Destructive Refactoring

| Field | Detail |
|---|---|
| Trigger | A small targeted fix is requested in a large file; the prompt says "clean up" or "refactor" without scope bounds. |
| Symptom | Unrelated working code deleted; comments removed; formatting churn across the whole file; public API renamed; tests modified to fit the new code. |
| Harness fix | Use SEARCH/REPLACE blocks with 3 lines of anchor context; require the patch to touch only the named symbol; post-flight diff must show zero changes outside the declared scope; reject any patch that deletes a test or changes a public signature without an explicit instruction. |

### 1.7 Premature Completion Claim ("Lying About Verification")

| Field | Detail |
|---|---|
| Trigger | The task is long; the model has produced a plausible-looking artifact; no verification gate is present. |
| Symptom | "All tests pass and edge cases are handled" without a trace; "This is production-ready" without a compile check; skipped edge cases in the summary. |
| Harness fix | Mandatory `<verification_gate>` table populated before the final answer; post-flight assertion that every claimed test/edge case maps to a concrete line in the artifact; reject completion claims that lack evidence locations. |

### 1.8 Language / Encoding Glitch

| Field | Detail |
|---|---|
| Trigger | Long generation near the output ceiling; mixed-script context; unusual Unicode in the prompt. |
| Symptom | Unexpected CJK characters in English output; Unicode variation selectors; zero-width joiners; mixed CRLF/LF; mojibake in code strings. |
| Harness fix | Post-flight regex reject on `[\u4E00-\u9FFF\u3040-\u30FF\uAC00-\uD7AF]` when the target language is English-only; normalize line endings to LF; strip zero-width characters; reject if replacement characters `\uFFFD` appear. |

## 2. Live Self-Diagnosis & Sanitizer Engineering Probes

### Probe 2.1: Automated Output Sanitizer & Fence-Repair Logic

```python
from __future__ import annotations
import json
import re
from dataclasses import dataclass, field

ELISION_PATTERNS = [
    r"//\s*\.\.\.\s*(rest|remaining|existing|other)\b.*",
    r"/\*\s*(TODO|\.\.\.)\s*(rest|remaining|existing).*\*/",
    r"\.\.\.\s*(existing|rest of|remaining)\s+code\s*\.\.\.",
    r"#\s*\.\.\.\s*(rest|remaining|existing).*",
    r"<\.\.\.\s*(rest|remaining|existing).*\.\.\.>",
]
ELISION_RE = re.compile("|".join(ELISION_PATTERNS), re.IGNORECASE)

THINKING_RE = re.compile(
    r"<(thinking|analysis|scratchpad|reasoning)>.*?</\1>",
    re.IGNORECASE | re.DOTALL,
)
PREAMBLE_RE = re.compile(
    r"^\s*(sure|certainly|of course|here('s| is)|absolutely)[^\n]*\n+",
    re.IGNORECASE,
)

@dataclass
class SanitizedPayload:
    text: str
    code_blocks: list[str] = field(default_factory=list)
    json_payload: object | None = None
    elision_hits: list[str] = field(default_factory=list)
    repaired_fence: bool = False
    had_thinking: bool = False

def _extract_fenced_blocks(raw: str) -> tuple[list[str], str, bool]:
    lines = raw.splitlines()
    blocks: list[str] = []
    out_lines: list[str] = []
    i = 0
    repaired = False
    while i < len(lines):
        line = lines[i]
        m = re.match(r"^(`{3,})\s*([\w+-]*)\s*$", line)
        if not m:
            out_lines.append(line)
            i += 1
            continue
        fence = m.group(1)
        lang = m.group(2)
        body: list[str] = []
        i += 1
        closed = False
        while i < len(lines):
            if re.match(rf"^{re.escape(fence)}\s*$", lines[i]):
                closed = True
                i += 1
                break
            body.append(lines[i])
            i += 1
        if not closed:
            repaired = True
            body.append(fence)
            out_lines.extend(body)
            break
        block = "\n".join(body)
        blocks.append(block)
        out_lines.append(f"```{lang}\n{block}\n```")
    return blocks, "\n".join(out_lines), repaired

def _try_json(text: str):
    s = text.strip()
    if s.startswith("```"):
        s = re.sub(r"^```[a-zA-Z0-9_-]*\s*", "", s)
        s = re.sub(r"\s*```$", "", s)
    start = s.find("{")
    end = s.rfind("}")
    if start == -1 or end == -1 or end <= start:
        return None
    candidate = s[start:end + 1]
    try:
        return json.loads(candidate)
    except json.JSONDecodeError:
        return None

def sanitize_model_output(raw: str, strict: str | None = None) -> SanitizedPayload:
    had_thinking = bool(THINKING_RE.search(raw))
    text = THINKING_RE.sub("", raw)
    text = PREAMBLE_RE.sub("", text)
    text = text.replace("\r\n", "\n").replace("\r", "\n")
    text = re.sub(r"[\u200b-\u200f\u202a-\u202e\u2060\ufeff]", "", text)
    elision_hits = [m.group(0) for m in ELISION_RE.finditer(text)]
    blocks, text, repaired = _extract_fenced_blocks(text)
    payload = SanitizedPayload(
        text=text.strip(),
        code_blocks=blocks,
        elision_hits=elision_hits,
        repaired_fence=repaired,
        had_thinking=had_thinking,
    )
    if strict == "json":
        payload.json_payload = _try_json(text)
    return payload
```

Harness integration rules:

1. If `elision_hits` is non-empty and the task required a complete artifact, reject the turn and auto-continue with the elision marker replaced by the missing region.
2. If `repaired_fence` is true, reject and re-request with a four-backtick outer fence requirement.
3. If `strict == "json"` and `json_payload is None`, reject and retry with `response_format` enforced.
4. Strip thinking tags before any parser; never feed them to a compiler.
5. If `had_thinking` is true in a strict-output turn, log it; repeated occurrences indicate the prompt lacks an explicit no-thinking rule.

### Probe 2.2: Fix-Loop Circuit Breaker Protocol

3-Strike state machine for an external orchestrator:

```text
STRIKE 0  Original task + failing assertion + minimal repro.
STRIKE 1  Patch attempt #1. Keep full context. Require a one-line root-cause hypothesis before the patch.
STRIKE 2  Purge: all prior patch attempts, all speculative fixes, all "try this" prose.
          Keep: original spec, minimal repro, exact failing assertion, latest error text.
          Mutate prompt: "Do not patch. Produce a root-cause table with columns
          [symptom, suspect location, evidence line, confidence]. Then produce the
          single minimal change that addresses the highest-confidence row."
STRIKE 3  Full reset: new context containing only the original spec, minimal repro,
          failing assertion, and the root-cause table from Strike 2. Disable patch mode.
          Require a minimal reproduction script and a one-line diff. Reject any response
          that changes more than 5 lines or touches tests.
STRIKE 4  Escalate to human/orchestrator. Do not call the model again on the same bug.
```

What must be purged on Strike 2: prior patch diffs, prior root-cause guesses, prior compiler-error transcripts beyond the latest one, any instruction that says "try again" or "fix it", and any accumulated prose about the bug. The probability lock is broken by removing the context tokens that anchor the model to its previous (wrong) hypothesis.

Prompt mutation that breaks the lock: switch from **patch mode** (produce a diff) to **diagnosis mode** (produce a root-cause table first, diff second). The intermediate table forces a new intermediate artifact and changes the token distribution before any code is emitted.

## 3. Pre-Flight & Post-Flight Verification Matrix

| Phase | Check ID | Check | Action on failure |
|---|---|---|---|
| Pre-flight | PRE-01 | Static prefix is byte-identical to the cached prefix | Rebuild prompt from the canonical prefix |
| Pre-flight | PRE-02 | Must-tier constraints appear in `<system_invariant>` and `<recency_anchor>` | Move or duplicate constraints |
| Pre-flight | PRE-03 | Critical rule is not only in the middle of `<context>` | Re-anchor to top and final 150 tokens |
| Pre-flight | PRE-04 | Symbol table (imports/env/columns) is closed and provided | Add table or switch to discovery step |
| Pre-flight | PRE-05 | Output contract declares exact schema/format | Add `<output_contract>` |
| Pre-flight | PRE-06 | Verification gate table is present | Add `<verification_gate>` |
| Pre-flight | PRE-07 | `response_format` set for JSON/schema tasks | Set structured output mode |
| Pre-flight | PRE-08 | Context under 49k dynamic tokens | Compact history before calling |
| Pre-flight | PRE-09 | No negative-only constraints without positive rewrite | Rewrite per §07.1.2 |
| Pre-flight | PRE-10 | No more than 11 simultaneous must-tier rules | Tier into buckets per §07.1.3 |
| Post-flight | POST-01 | Thinking tags stripped | Run sanitizer |
| Post-flight | POST-02 | Fences balanced and repaired | Reject and re-request with four-backtick outer fence |
| Post-flight | POST-03 | No elision markers (`... rest`, `TODO`, `pass` stub) | Auto-continue or reject |
| Post-flight | POST-04 | JSON parses against schema | Reject and retry with strict format |
| Post-flight | POST-05 | Every import/env/column resolves to the symbol table | Reject with `MISSING_SYMBOL` list |
| Post-flight | POST-06 | Diff touches only declared scope | Reject and re-request SEARCH/REPLACE with anchors |
| Post-flight | POST-07 | No test deletion or public-signature change without instruction | Reject |
| Post-flight | POST-08 | Every completion claim maps to an evidence line | Reject unverified claims |
| Post-flight | POST-09 | No unexpected CJK/variation-selector/zero-width characters | Strip and re-validate |
| Post-flight | POST-10 | Line endings normalized to LF | Normalize |
| Post-flight | POST-11 | Verification gate table fully populated with Y and evidence | Reject if any N or blank |
| Post-flight | POST-12 | `<<CONTINUE>>` only at a section/file boundary | Reject mid-line continuations |

Target: PRE-01 through PRE-10 catch prompt-side degradation before generation; POST-01 through POST-12 catch output-side degradation before the artifact reaches a compiler, parser, or deployment pipeline. The combination is what raises end-to-end reliability toward 99.9% on well-specified tasks; it does not compensate for an underspecified task.