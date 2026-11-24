#FORENSIC COGNITIVE TELEMETRY PROBE — LAYER 08/09: BEHAVIORAL FAILURE CATALOG, DEGRADATION SIGNATURES & AUTOMATED HARNESS PATCHES

You are generating the eighth specification file (`08_FAILURE_MODES_AND_PATCHES.md`) of the AI Ecosystem Genome. Every frontier LLM has specific, repeatable failure modes under high cognitive load, long context, or multi-turn debugging.

Your objective is to catalog your exact behavioral bugs and provide **deterministic, programmatic harness patches** (Regex sanitizers, AST checks, retry mutations, and context resets) that an external orchestrator can execute automatically to detect and heal your failures in real time.

Output ONLY the raw Markdown file content starting with the YAML frontmatter (`---`).

---
file_id: "08_FAILURE_MODES_AND_PATCHES"
schema_version: "1.0.0"
model_self_reported_id: "<fill exact model name/version>"
primary_degradation_symptom: "<lazy_truncation | circular_apology_loop | over_abstraction | hallucinated_imports | context_amnesia>"
multi_turn_fix_loop_limit: <integer: max failed debugging turns before context poison occurs>
requires_output_ fence_sanitizer: <boolean>
---

# 08 — Behavioral Failure Catalog, Degradation Signatures & Automated Harness Patches

## 1. Top 8 Deterministic Failure Modes (Under Stress)
For each of the 8 categories below, document: (a) **Exact Trigger Condition**, (b) **Observable Symptom in Output**, and (c) **Automated Harness Fix**:
1. **Context Rot & Mid-Prompt Amnesia:** What gets forgotten first when context exceeds 50% capacity?
2. **Multi-Turn Debugging Death Spiral ("Fix-Loop Poisoning"):** When a compiler error or test failure is fed back to you 3+ times, how do you degrade (e.g., oscillating between two broken fixes, deleting tests to make the suite pass, or adding `.unwrap()`/`any` casts)?
3. **Markdown / Code-Fence Corruption:** How do nested triple-backticks (` ``` `) inside markdown/documentation strings break your outer code fence, and how should a parser extract your code safely?
4. **JSON / Structured Output Drift:** Exact edge cases where you emit prose before `{`, trailing commas, unquoted keys, or truncated JSON at `max_tokens`.
5. **Phantom File / Phantom Import Hallucination:** Referencing helper modules, env vars, or database columns that were never defined in the repository map.
6. **Over-Correction & Destructive Refactoring:** Deleting working unrelated code or comments when asked to make a small targeted fix.
7. **Premature Completion Claim ("Lying About Verification"):** Claiming *"All tests pass and edge cases are handled"* without actually tracing the execution path.
8. **Language / Encoding Glitch:** Rare token-sampling glitches (e.g., unexpected CJK characters, Unicode variation selectors, or mixed line-endings).

## 2. Live Self-Diagnosis & Sanitizer Engineering Probes (Execute & Record)

### Probe 2.1: Automated Output Sanitizer & Fence-Repair Logic
Write a compact, production-ready Python or Rust function (`sanitize_model_output(raw: str) -> SanitizedPayload`) tailored specifically to clean up your raw responses:
- Stripping leaked thinking tags or conversational preambles when strict code/JSON is requested.
- Detecting lazy elision markers (`// ... rest of code`, `/* TODO */`, `... existing code ...`) via deterministic regex patterns so the harness can immediately trigger an auto-continuation or reject the turn.

### Probe 2.2: Fix-Loop Circuit Breaker Protocol
Design the exact **3-Strike Circuit Breaker** state machine an external orchestrator must use when you fail to fix a bug after 2 attempts:
- What must be purged from the conversation history on Strike 2?
- How must the prompt be mutated (e.g., switching from patch mode to root-cause table + minimal reproduction) to break your probability lock?

## 3. Pre-Flight & Post-Flight Verification Matrix
Provide a machine-actionable table of `Pre-Flight Checks` (before calling you) and `Post-Flight Assertions` (after receiving your response) that raises your end-to-end production reliability to `99.9%`.