---
genome_file: "08_FAILURE_MODES_AND_BLINDSPOTS.md"
genome_schema: "omni-genome-v4.0"
model_id: "GPT 6-Web"
model_dir: "models/01-gpt-6-web/"
model_aliases: ["GPT-6", "GPT 6", "gpt-6-web", "gpt-6-standard-free (predecessor genome)"]
canonical_title: "Failure Modes & Blindspots"
wave: "WAVE 7 (Step 4 of 4 - FINAL WAVE)"
generated: "2026-11-25"
legacy_sources: ["08_FAILURE_MODES_AND_PATCHES"]
specialty: "Real-time web synthesis, frame-1 interactive UI/UX artifact craft (390px zero-overflow), rapid full-stack product generation."
license: "MIT"
---
# Failure Modes & Blindspots — GPT 6-Web
> **Canonical genome file (omni-genome-v4).** Specialty: Real-time web synthesis, frame-1 interactive UI/UX artifact craft (390px zero-overflow), rapid full-stack product generation.
## Canonical Annex — Blindspots

| Symptom | Root cause | Patch |
|---|---|---|
| context_amnesia mid-session | harness compaction | restate constraints each major turn |
| stale facts | parametric prior beats retrieval | force first-hop search |
| overflowing artifacts | desktop-first habits | 390px checklist before emit |
| dead controls | decorative generation | wire every handler pre-emit |
---

## Legacy Genome Continuation — `08_FAILURE_MODES_AND_PATCHES` (verbatim provenance)
\---\
file_id: "08_FAILURE_MODES_AND_PATCHES"\
schema_version: "1.0.0"\
model_self_reported_id: "GPT-6"\
primary_degradation_symptom: "context_amnesia"\
multi_turn_fix_loop_limit: null\
requires_output_fence_sanitizer: true\
\---\
\
\# 08 — Behavioral Failure Catalog, Degradation Signatures & Automated Harness Patches\
\
\> \*\*Measurement boundary:\*\* These are engineering failure classes that can occur in generated outputs, not a verified list of GPT-6-specific defects. I cannot inspect hidden context utilization, token-sampling traces, or a measured debugging-turn failure threshold. The circuit breaker and code below are proposed external harness controls, not mechanisms claimed to exist inside the model.\
\
\## 1. Top 8 Failure Modes Under Stress\
\
\### 1.1 Context rot and mid-prompt amnesia\
\
\*\*Trigger conditions\*\*\
\
\- Long conversations with many unrelated requirements.\
\- Repeatedly superseded implementation details.\
\- Large source files mixed with lengthy discussion.\
\- Conflicting versions of the same specification.\
\- Important constraints appearing only once, far from the current task.\
\
The 50% context-utilization threshold is not established. There is no verified universal point at which requirements begin to be forgotten.\
\
\*\*Observable symptoms\*\*\
\
\- Reusing an obsolete API or earlier implementation.\
\- Missing a constraint introduced several turns earlier.\
\- Repeating an already rejected approach.\
\- Confusing illustrative examples with actual repository facts.\
\- Omitting a previously requested output field.\
\
\*\*Harness fix\*\*\
\
Maintain an authoritative, compact project-state document outside the conversation.\
\
\`\`\`yaml\
project_state:\
&#x20; task_id: "TASK-001"\
&#x20; current_goal: "Fix parser regression"\
&#x20; source_revision: "commit-or-hash"\
&#x20; active_requirements:\
&#x20;   \- id: "R1"\
&#x20;     statement: "Preserve public parser interface"\
&#x20;   \- id: "R2"\
&#x20;     statement: "Reject malformed input"\
&#x20; rejected_approaches:\
&#x20;   \- "Replace parser with permissive fallback"\
&#x20; verified_facts:\
&#x20;   \- "Parser uses borrowed string slices"\
&#x20; unknowns:\
&#x20;   \- "Whether downstream callers depend on whitespace preservation"\
&#x20; current_attempt: 1\
\`\`\`\
\
Before each new turn, send the current state and only the source files relevant to the next operation. Preserve important requirements by ID rather than relying on conversational recall.\
\
\### 1.2 Multi-turn debugging death spiral\
\
\*\*Trigger conditions\*\*\
\
\- Repeatedly submitting the same error without new evidence.\
\- Applying patches without identifying the failing invariant.\
\- Changing several unrelated components in one attempt.\
\- Treating a new compiler error as proof that the previous hypothesis was correct.\
\
\*\*Observable symptoms\*\*\
\
\- Alternating between two ineffective fixes.\
\- Introducing casts, unwraps, or broad exception handling to silence errors.\
\- Removing failing tests instead of correcting implementation behavior.\
\- Changing public interfaces unnecessarily.\
\- Repeating the same patch with superficial wording changes.\
\
These are possible failure patterns, not measured GPT-6-specific frequencies.\
\
\*\*Harness fix\*\*\
\
\- Track a unique fingerprint for each failure.\
\- Preserve the last known-good source revision.\
\- Require a root-cause hypothesis before each patch.\
\- Compare the proposed patch against the previous attempt.\
\- Reject repeated patches that do not address new evidence.\
\- Switch to a minimal reproduction after two unsuccessful attempts.\
\
\### 1.3 Markdown and code-fence corruption\
\
\*\*Trigger conditions\*\*\
\
\- A Markdown document contains nested fenced code blocks.\
\- Generated documentation includes literal triple backticks.\
\- An outer code block uses the same delimiter as an inner example.\
\- Output is truncated before a closing fence.\
\
\*\*Observable symptoms\*\*\
\
\- Code appears outside the intended code block.\
\- Documentation is interpreted as executable source.\
\- The parser captures only part of a code sample.\
\- The final artifact contains unmatched fences.\
\
\*\*Harness fix\*\*\
\
Prefer a delimiter longer than any run of backticks in the content, or use a Markdown-aware parser. Do not repair code fences by blindly appending a closing delimiter without validating the extracted artifact.\
\
Example:\
\
\`\`\`\`markdown\
\`\`\`\`markdown\
Example containing a nested fence:\
\
\`\`\`python\
print("hello")\
\`\`\`\
\`\`\`\`\
\`\`\`\`\
\
For machine extraction, use a parser that records fence character, opening length, language label, and source offsets.\
\
\### 1.4 JSON and structured-output drift\
\
\*\*Trigger conditions\*\*\
\
\- JSON requested alongside explanatory prose.\
\- Output contract is underspecified.\
\- Nested structures become large.\
\- Generation stops before the closing object.\
\- The requested schema conflicts with an example.\
\
\*\*Observable symptoms\*\*\
\
\- Prose appears before the first \`{\`.\
\- Markdown fences surround otherwise valid JSON.\
\- Keys are unquoted.\
\- Trailing commas appear.\
\- A required property is missing.\
\- Strings contain unescaped control characters.\
\- The output ends before the JSON document is complete.\
\
\*\*Harness fix\*\*\
\
1\. Parse the entire response using a strict JSON parser.\
2\. Validate the parsed object against a schema.\
3\. Reject truncated or invalid output.\
4\. Retry with the validation error and exact schema.\
5\. Never silently delete invalid fields or repair semantic values.\
\
Do not assume that extracting text between the first \`{\` and last \`}\` is safe. Braces can occur inside strings or unrelated prose.\
\
\### 1.5 Phantom files and phantom imports\
\
\*\*Trigger conditions\*\*\
\
\- The task describes a repository without supplying its file map.\
\- Prior examples mention modules that do not exist in the current project.\
\- The implementation assumes a conventional framework layout.\
\- A library API is inferred from an outdated version.\
\
\*\*Observable symptoms\*\*\
\
\- Imports reference nonexistent modules.\
\- Environment variables are invented.\
\- Database columns do not match the schema.\
\- Function signatures differ from actual source.\
\- Dependencies are added without checking the lockfile.\
\
\*\*Harness fix\*\*\
\
Create a repository manifest from the actual checkout.\
\
\`\`\`yaml\
repository_manifest:\
&#x20; revision: "actual-commit-hash"\
&#x20; files:\
&#x20;   \- "src/lib.rs"\
&#x20;   \- "src/parser.rs"\
&#x20;   \- "Cargo.toml"\
&#x20; environment_variables:\
&#x20;   \- "APP_ENV"\
&#x20; database_tables:\
&#x20;   \- "users"\
&#x20;   \- "sessions"\
&#x20; dependencies:\
&#x20;   \- name: "tokio"\
&#x20;     version: "manifest-resolved-version"\
\`\`\`\
\
Require every new import, file reference, environment variable, and database identifier to resolve against the manifest or be explicitly marked as a proposed addition.\
\
\### 1.6 Over-correction and destructive refactoring\
\
\*\*Trigger conditions\*\*\
\
\- A narrow bug report is paired with a broad request to "clean up" code.\
\- The relevant source file contains unrelated technical debt.\
\- The model receives incomplete surrounding context.\
\- A replacement is generated instead of a minimal patch.\
\
\*\*Observable symptoms\*\*\
\
\- Unrelated comments or functions disappear.\
\- Formatting changes affect the entire file.\
\- Public interfaces change without justification.\
\- Existing tests are rewritten or removed.\
\- A small fix expands into a broad refactor.\
\
\*\*Harness fix\*\*\
\
\- Capture a clean baseline revision.\
\- Restrict edits to named files and functions.\
\- Apply a unified diff rather than replacing the entire file.\
\- Check the changed-line budget.\
\- Run \`git diff --check\`.\
\- Reject unrelated deletions unless separately approved.\
\
Example policy:\
\
\`\`\`yaml\
patch_policy:\
&#x20; allowed_files:\
&#x20;   \- "src/parser.rs"\
&#x20; allowed_symbols:\
&#x20;   \- "parse_headers"\
&#x20; max_changed_lines: 40\
&#x20; allow_public_api_change: false\
&#x20; allow_test_deletion: false\
&#x20; require_diff_review: true\
\`\`\`\
\
\### 1.7 Premature completion claims\
\
\*\*Trigger conditions\*\*\
\
\- A task requests a complete implementation but no execution tool is available.\
\- Tests are described but not run.\
\- A code sample is assumed to compile.\
\- The model confuses reasoning about correctness with observed test evidence.\
\
\*\*Observable symptoms\*\*\
\
\- "All tests pass" without execution evidence.\
\- "Compiles successfully" without a compiler run.\
\- Unsupported claims about edge-case coverage.\
\- An implementation is described as production-ready without validation.\
\
\*\*Harness fix\*\*\
\
Use evidence-backed completion statuses:\
\
\| Status | Meaning |\
\|---|---|\
\| \`PASS\` | The check was executed and passed |\
\| \`FAIL\` | The check was executed and failed |\
\| \`NOT_RUN\` | The check was not executed |\
\| \`BLOCKED\` | Execution was attempted but could not proceed |\
\| \`NOT_APPLICABLE\` | The check does not apply |\
\
A response must never upgrade \`NOT_RUN\` to \`PASS\` based on reasoning alone.\
\
\### 1.8 Language and encoding glitches\
\
\*\*Trigger conditions\*\*\
\
\- Mixed Persian, Arabic, English, and code.\
\- Copying text containing bidirectional controls.\
\- Unicode normalization differences.\
\- Windows and Unix line endings mixed in one file.\
\- Unusual punctuation or invisible characters in identifiers.\
\
\*\*Observable symptoms\*\*\
\
\- Unexpected Unicode characters.\
\- Mixed \`CRLF\` and \`LF\`.\
\- Visually identical but byte-distinct identifiers.\
\- Broken quotation marks or punctuation.\
\- Bidirectional text appears in an unexpected order.\
\
\*\*Harness fix\*\*\
\
\- Validate UTF-8 decoding.\
\- Normalize line endings according to repository policy.\
\- Preserve intentional source-language characters.\
\- Inspect Unicode control and format characters.\
\- Apply Unicode normalization only where appropriate; do not blindly rewrite identifiers.\
\- Run language-specific formatters and linters.\
\
\---\
\
\## 2. Live Self-Diagnosis and Sanitizer Engineering Probes\
\
\### 2.1 Probe: Automated Output Sanitizer and Fence-Repair Logic\
\
The sanitizer below is deliberately conservative. It detects common incomplete-output markers and extracts a complete code fence when a language is requested. It does not execute code, infer missing source, or silently repair invalid JSON.\
\
It also avoids stripping arbitrary text based on guessed "thinking" tags. Such tags may be legitimate content or documentation.\
\
\#### Python implementation\
\
\`\`\`python\
from \_\_future\_\_ import annotations\
\
from dataclasses import dataclass\
import re\
from typing import Literal\
\
\
Status = Literal[\
&#x20;   "ok",\
&#x20;   "elision_detected",\
&#x20;   "missing_code_fence",\
&#x20;   "multiple_code_fences",\
&#x20;   "invalid_json",\
&#x20;   "empty",\
]\
\
\
@dataclass(frozen=True)\
class SanitizedPayload:\
&#x20;   status: Status\
&#x20;   content: str\
&#x20;   language: str | None\
&#x20;   issues: tuple[str, ...]\
\
\
\# Detect explicit placeholders, not ordinary Python/Rust ellipses.\
ELISION_PATTERNS = (\
&#x20;   re.compile(\
&#x20;       r"(?im)^\s\*//\s\*(?:\\.\\.\\.|TODO:?\s\*rest of code)\b.\*$"\
&#x20;   ),\
&#x20;   re.compile(\
&#x20;       r"(?im)^\s\*/\\\*\s\*(?:TODO|FIXME|rest of code)\b.\*?\\\*/\s\*$"\
&#x20;   ),\
&#x20;   re.compile(\
&#x20;       r"(?im)^\s\*(?:\\.\\.\\.\s\*)?(?:existing|remaining) "\
&#x20;       r"code\s\*(?:\\.\\.\\.)?\s\*$"\
&#x20;   ),\
&#x20;   re.compile(\
&#x20;       r"(?im)^\s\*<\s\*omitted\s+for\s+brevity\s\*/?\s\*>\s\*$"\
&#x20;   ),\
)\
\
FENCE_PATTERN = re.compile(\
&#x20;   r"(?ms)^(\`{3,}|\~{3,})([^\r\n]\*)\r?\n"\
&#x20;   r"(.\*?)"\
&#x20;   r"^\1[ \t]\*$"\
)\
\
THINKING_TAG_PATTERN = re.compile(\
&#x20;   r"\</?(?:analysis|chain_of_thought|private_reasoning)>",\
&#x20;   re.IGNORECASE,\
)\
\
\
def sanitize_model_output(\
&#x20;   raw: str,\
&#x20;   \*,\
&#x20;   expected: Literal["text", "code", "json"] = "text",\
&#x20;   language: str | None = None,\
) -> SanitizedPayload:\
&#x20;   if not isinstance(raw, str) or not raw\.strip():\
&#x20;       return SanitizedPayload(\
&#x20;           status="empty",\
&#x20;           content="",\
&#x20;           language=None,\
&#x20;           issues=("empty response",),\
&#x20;       )\
\
&#x20;   issues: list[str] = []\
&#x20;   content = raw\.strip()\
\
&#x20;   \# Flag suspicious tags; do not blindly delete source content.\
&#x20;   if THINKING_TAG_PATTERN.search(content):\
&#x20;       issues.append("suspicious reasoning-tag marker detected")\
\
&#x20;   \# Detect likely lazy elision before extraction.\
&#x20;   if any(pattern.search(content) for pattern in ELISION_PATTERNS):\
&#x20;       return SanitizedPayload(\
&#x20;           status="elision_detected",\
&#x20;           content=content,\
&#x20;           language=language,\
&#x20;           issues=tuple(issues + ["possible incomplete implementation"]),\
&#x20;       )\
\
&#x20;   if expected == "json":\
&#x20;       import json\
\
&#x20;       try:\
&#x20;           json.loads(content)\
&#x20;       except json.JSONDecodeError as exc:\
&#x20;           return SanitizedPayload(\
&#x20;               status="invalid_json",\
&#x20;               content=content,\
&#x20;               language=None,\
&#x20;               issues=tuple(\
&#x20;                   issues + [f"JSON parse failure at line {exc.lineno}, "\
&#x20;                             f"column {exc.colno}"]\
&#x20;               ),\
&#x20;           )\
\
&#x20;       return SanitizedPayload(\
&#x20;           status="ok",\
&#x20;           content=content,\
&#x20;           language="json",\
&#x20;           issues=tuple(issues),\
&#x20;       )\
\
&#x20;   if expected == "code":\
&#x20;       fences = list(FENCE_PATTERN.finditer(content))\
\
&#x20;       if not fences:\
&#x20;           return SanitizedPayload(\
&#x20;               status="missing_code_fence",\
&#x20;               content=content,\
&#x20;               language=language,\
&#x20;               issues=tuple(issues + ["no complete fenced code block"]),\
&#x20;           )\
\
&#x20;       if len(fences) != 1:\
&#x20;           return SanitizedPayload(\
&#x20;               status="multiple_code_fences",\
&#x20;               content=content,\
&#x20;               language=language,\
&#x20;               issues=tuple(issues + ["ambiguous code-block selection"]),\
&#x20;           )\
\
&#x20;       match = fences[0]\
&#x20;       fence_info = match.group(2).strip()\
&#x20;       detected_language = fence_info.split(maxsplit=1)[0] if fence_info else None\
\
&#x20;       if language and detected_language:\
&#x20;           if detected_language.lower() != language.lower():\
&#x20;               issues.append(\
&#x20;                   f"language mismatch: expected {language}, "\
&#x20;                   f"found {detected_language}"\
&#x20;               )\
\
&#x20;       return SanitizedPayload(\
&#x20;           status="ok",\
&#x20;           content=match.group(3),\
&#x20;           language=detected_language or language,\
&#x20;           issues=tuple(issues),\
&#x20;       )\
\
&#x20;   return SanitizedPayload(\
&#x20;       status="ok",\
&#x20;       content=content,\
&#x20;       language=None,\
&#x20;       issues=tuple(issues),\
&#x20;   )\
\`\`\`\
\
\#### Important limitations\
\
\- The fence regex is suitable only for a controlled, single-code-block output contract. It is not a complete CommonMark parser.\
\- The exact same fence character and length are required for closing a block.\
\- Multiple code blocks are rejected rather than guessed.\
\- JSON is validated, not repaired.\
\- The sanitizer does not remove preambles heuristically because doing so can delete legitimate source or documentation.\
\- Elision detection is heuristic. A comment containing \`TODO\` may be legitimate; inspect its location and context before rejecting a file.\
\- A production harness should use a Markdown parser for arbitrary Markdown and an AST parser for source-code completeness.\
\
\#### Example use\
\
\`\`\`python\
result = sanitize_model_output(\
&#x20;   raw_response,\
&#x20;   expected="code",\
&#x20;   language="python",\
)\
\
if result.status != "ok":\
&#x20;   raise ValueError({\
&#x20;       "status": result.status,\
&#x20;       "issues": result.issues,\
&#x20;   })\
\
source = result.content\
\`\`\`\
\
For strict JSON:\
\
\`\`\`python\
result = sanitize_model_output(\
&#x20;   raw_response,\
&#x20;   expected="json",\
)\
\
if result.status != "ok":\
&#x20;   raise ValueError(result.issues)\
\`\`\`\
\
\### 2.2 Probe: Three-Strike Circuit Breaker\
\
\*\*Recommended policy:\*\* After two failed repair attempts, stop repeating the same patch strategy. A third attempt is allowed only after changing the diagnostic method and grounding the request in new evidence.\
\
The circuit breaker is an external orchestration design, not a claim about an internal GPT-6 fix-loop threshold.\
\
\#### State machine\
\
\<badge color="success">Proposed external control\</badge>\
\
\`\`\`text\
&#x20;                   ┌──────────────────┐\
&#x20;                   │    INITIAL       │\
&#x20;                   └────────┬─────────┘\
&#x20;                            │\
&#x20;                            ▼\
&#x20;                   ┌──────────────────┐\
&#x20;                   │   ATTEMPT 1      │\
&#x20;                   └────────┬─────────┘\
&#x20;                            │\
&#x20;                   PASS ────┴──── FAIL\
&#x20;                    │              │\
&#x20;                    ▼              ▼\
&#x20;                 SUCCESS     ┌──────────────┐\
&#x20;                             │  ATTEMPT 2   │\
&#x20;                             └──────┬───────┘\
&#x20;                                    │\
&#x20;                           PASS ────┴──── FAIL\
&#x20;                            │              │\
&#x20;                            ▼              ▼\
&#x20;                         SUCCESS    ┌────────────────┐\
&#x20;                                    │ STRIKE 2       │\
&#x20;                                    │ CIRCUIT BREAK  │\
&#x20;                                    └───────┬────────┘\
&#x20;                                            │\
&#x20;                                 Reset diagnostic approach\
&#x20;                                            │\
&#x20;                                            ▼\
&#x20;                                    ┌──────────────┐\
&#x20;                                    │  ATTEMPT 3   │\
&#x20;                                    └──────┬───────┘\
&#x20;                                           │\
&#x20;                                  PASS ────┴──── FAIL\
&#x20;                                   │              │\
&#x20;                                   ▼              ▼\
&#x20;                                SUCCESS        BLOCKED\
\`\`\`\
\
\#### Strike definitions\
\
\| Strike | Event | Required action |\
\|---|---|---|\
\| 1 | First patch fails validation | Record error, preserve diff, identify hypothesis |\
\| 2 | Second patch fails validation | Break patch loop and rebuild diagnosis |\
\| 3 | Reframed third attempt fails | Stop automatic retries and request human or external review |\
\
The third strike is a terminal failure, not permission for endless regeneration.\
\
\#### What to purge at Strike 2\
\
Do not delete the entire conversation history indiscriminately. Remove or supersede stale working assumptions while retaining authoritative evidence.\
\
\*\*Retain:\*\*\
\
\- Original task and acceptance criteria.\
\- Repository revision and actual file contents.\
\- Compiler and test output.\
\- The last known-good source revision.\
\- The two failed diffs and their failure explanations.\
\- Relevant dependency and runtime versions.\
\- Authorization and scope constraints.\
\
\*\*Purge or supersede:\*\*\
\
\- Repeated speculative explanations.\
\- Unsupported assumptions about repository structure.\
\- Obsolete proposed implementations.\
\- Repeated conversational paraphrases of the same failure.\
\- Contradictory intermediate state summaries.\
\- Guesses that were disproved by actual compiler or test output.\
\
\#### Strike 2 prompt mutation\
\
Switch from patch mode to root-cause analysis.\
\
\`\`\`text\
DEBUGGING RESET — STRIKE 2\
\
Do not produce another patch immediately.\
\
1\. Reconstruct the failing behavior from the supplied source and test output.\
2\. Identify the smallest reproducible input.\
3\. Separate observed facts from hypotheses.\
4\. Produce a root-cause table with evidence for each hypothesis.\
5\. Identify the exact invariant violated.\
6\. Explain why each of the two previous patches failed.\
7\. Propose the smallest corrective change.\
8\. List affected interfaces and regression risks.\
9\. Define tests that distinguish the proposed fix from previous failed fixes.\
10\. Wait for confirmation if required repository information is missing.\
\
Do not remove tests, suppress errors, add broad casts, or weaken validation\
merely to obtain a passing result.\
\`\`\`\
\
\#### Machine-actionable state record\
\
\`\`\`yaml\
circuit_breaker:\
&#x20; max_patch_attempts: 2\
&#x20; max_total_attempts: 3\
&#x20; state: "diagnostic_reset"\
&#x20; attempts:\
&#x20;   \- number: 1\
&#x20;     result: "failed"\
&#x20;     failure_fingerprint: "compiler-error-hash-1"\
&#x20;     diff_id: "patch-1"\
&#x20;   \- number: 2\
&#x20;     result: "failed"\
&#x20;     failure_fingerprint: "compiler-error-hash-2"\
&#x20;     diff_id: "patch-2"\
&#x20; retained_evidence:\
&#x20;   \- "original_requirements"\
&#x20;   \- "source_revision"\
&#x20;   \- "compiler_output"\
&#x20;   \- "test_output"\
&#x20;   \- "failed_diffs"\
&#x20; next_mode: "root_cause_analysis"\
&#x20; third_attempt_requires:\
&#x20;   \- "new_hypothesis"\
&#x20;   \- "minimal_reproduction"\
&#x20;   \- "changed_patch_strategy"\
&#x20; terminal_failure_action: "human_review"\
\`\`\`\
\
\#### Retry mutation algorithm\
\
\`\`\`python\
def choose_debugging_mode(failed_attempts: int) -> str:\
&#x20;   if failed_attempts == 0:\
&#x20;       return "implementation"\
\
&#x20;   if failed_attempts == 1:\
&#x20;       return "evidence_based_patch"\
\
&#x20;   if failed_attempts == 2:\
&#x20;       return "root_cause_and_minimal_reproduction"\
\
&#x20;   return "human_review"\
\`\`\`\
\
The retry controller should also compare patch fingerprints. A new prompt that produces the same diff and same error is not meaningful progress.\
\
\---\
\
\## 3. Pre-Flight and Post-Flight Verification Matrix\
\
\*\*Reliability qualification:\*\* A harness can improve defect detection and reduce unsupported completion claims. It cannot establish 99.9% end-to-end production reliability without empirical measurements across representative workloads, environments, and failure conditions.\
\
\### 3.1 Pre-flight checks\
\
\| ID | Check | Machine action | Failure behavior |\
\|---|---|---|---|\
\| PF-01 | Task completeness | Verify objective and deliverable exist | Request missing information |\
\| PF-02 | Scope boundaries | Validate authorized files, systems, and environments | Block out-of-scope operations |\
\| PF-03 | Repository identity | Record commit hash and working-tree state | Stop if source identity is ambiguous |\
\| PF-04 | File manifest | Resolve referenced paths against checkout | Mark unknown files as proposed |\
\| PF-05 | Dependency manifest | Read lockfiles and package manifests | Do not invent versions |\
\| PF-06 | API contract | Verify referenced symbols and signatures | Request missing definitions |\
\| PF-07 | Output schema | Parse and validate expected schema | Reject contradictory contract |\
\| PF-08 | Constraint consistency | Check mutually incompatible requirements | Report conflict |\
\| PF-09 | Baseline tests | Run relevant tests before changes | Preserve baseline failures |\
\| PF-10 | Baseline diff | Capture clean source revision | Prevent accidental unrelated edits |\
\| PF-11 | Context summary | Generate authoritative state record | Exclude stale assumptions |\
\| PF-12 | Retry budget | Initialize circuit-breaker state | Prevent unlimited patch loops |\
\
\### 3.2 Post-flight assertions\
\
\| ID | Assertion | Machine action | Failure behavior |\
\|---|---|---|---|\
\| PO-01 | Output parseability | Parse JSON, Markdown, or requested format | Reject malformed output |\
\| PO-02 | Fence completeness | Parse Markdown fences | Reject unmatched fences |\
\| PO-03 | Elision detection | Scan for suspicious placeholders | Flag or reject incomplete artifact |\
\| PO-04 | Import resolution | Resolve imports against repository and dependency manifests | Reject phantom imports |\
\| PO-05 | Symbol resolution | Parse source AST or compile | Reject undefined references |\
\| PO-06 | Patch scope | Compare changed paths against allowlist | Reject unauthorized file changes |\
\| PO-07 | Change budget | Count changed lines and inspect diff | Require approval for excess |\
\| PO-08 | Test preservation | Compare test inventory before and after | Reject unexplained test deletion |\
\| PO-09 | Static analysis | Run formatter, linter, and type checker | Record exact failures |\
\| PO-10 | Compilation | Execute project compiler | Mark unexecuted as \`NOT_RUN\` |\
\| PO-11 | Unit tests | Execute targeted tests | Record test evidence |\
\| PO-12 | Regression tests | Execute relevant broader suite | Block release on required failures |\
\| PO-13 | Security checks | Run applicable security scanners and boundary tests | Record findings |\
\| PO-14 | Encoding | Validate UTF-8 and line-ending policy | Reject unintended corruption |\
\| PO-15 | Completion claims | Compare claims against execution records | Reject unsupported PASS statements |\
\| PO-16 | Final diff review | Inspect complete patch | Require approval for unexplained changes |\
\
\### 3.3 Machine-readable verification contract\
\
\`\`\`yaml\
verification_report:\
&#x20; task_id: "TASK-001"\
&#x20; source_revision_before: null\
&#x20; source_revision_after: null\
\
&#x20; preflight:\
&#x20;   task_complete: "NOT_RUN"\
&#x20;   scope_valid: "NOT_RUN"\
&#x20;   repository_verified: "NOT_RUN"\
&#x20;   dependencies_verified: "NOT_RUN"\
&#x20;   baseline_tests: "NOT_RUN"\
\
&#x20; postflight:\
&#x20;   output_parseable: "NOT_RUN"\
&#x20;   fences_valid: "NOT_RUN"\
&#x20;   elision_scan: "NOT_RUN"\
&#x20;   imports_resolved: "NOT_RUN"\
&#x20;   ast_or_compile_check: "NOT_RUN"\
&#x20;   tests: "NOT_RUN"\
&#x20;   security_checks: "NOT_RUN"\
&#x20;   encoding_check: "NOT_RUN"\
&#x20;   diff_review: "NOT_RUN"\
\
&#x20; findings: []\
&#x20; unresolved_defects: []\
&#x20; unsupported_claims: []\
\
&#x20; release_decision: "BLOCKED"\
\`\`\`\
\
\### 3.4 Suggested release gate\
\
\`\`\`python\
REQUIRED_CHECKS = (\
&#x20;   "output_parseable",\
&#x20;   "fences_valid",\
&#x20;   "imports_resolved",\
&#x20;   "ast_or_compile_check",\
&#x20;   "tests",\
&#x20;   "diff_review",\
)\
\
def release_allowed(report: dict) -> bool:\
&#x20;   postflight = report.get("postflight", {})\
\
&#x20;   return all(\
&#x20;       postflight.get(check) == "PASS"\
&#x20;       for check in REQUIRED_CHECKS\
&#x20;   )\
\`\`\`\
\
This is a minimal gate. A real release system should add project-specific checks for integration tests, security scanning, migrations, deployment policy, and performance budgets.\
\
\## 4. Recommended Harness Architecture\
\
A robust orchestrator should separate generation from validation and release authorization.\
\
\`\`\`text\
Natural-language request\
&#x20;       |\
&#x20;       v\
Requirement compiler\
&#x20;       |\
&#x20;       v\
Repository/context snapshot\
&#x20;       |\
&#x20;       v\
Structured model request\
&#x20;       |\
&#x20;       v\
Response parser\
&#x20;       |\
&#x20;       v\
Artifact extraction\
&#x20;       |\
&#x20;       v\
Static and schema validation\
&#x20;       |\
&#x20;       v\
Compilation and tests\
&#x20;       |\
&#x20;       +------ PASS ------> Diff review / release gate\
&#x20;       |\
&#x20;       +------ FAIL ------> Failure fingerprint\
&#x20;                                 |\
&#x20;                                 v\
&#x20;                          Retry controller\
&#x20;                                 |\
&#x20;                        Attempts remaining?\
&#x20;                          /          \\\
&#x20;                        Yes          No\
&#x20;                         \|            |\
&#x20;                   Mutated prompt   Human review\
\`\`\`\
\
\### 4.1 Operational invariants\
\
1\. The model never determines its own release status.\
2\. The harness never treats plausible code as executed code.\
3\. Every patch is applied against a recorded source revision.\
4\. Every failure is associated with actual diagnostic output.\
5\. A retry must change its diagnostic strategy or incorporate new evidence.\
6\. Tests cannot be removed automatically to obtain a passing build.\
7\. Unknown repository symbols are not silently accepted.\
8\. All release-critical checks require execution evidence.\
9\. Unverified output remains unverified.\
10\. A human or external policy engine retains final release authority.\
\
\## 5. Final Operational Findings\
\
\| Field | Assessment |\
\|---|---|\
\| Model identifier | GPT-6 |\
\| Primary degradation symptom | Context amnesia, as a qualitative risk category |\
\| Exact context degradation threshold | Unknown |\
\| Exact multi-turn fix-loop limit | Unknown |\
\| Recommended patch retry budget | Two patch attempts before diagnostic reset |\
\| Third failed attempt | Stop and request review |\
\| Output-fence sanitizer | Recommended for strict code extraction |\
\| Regex-only source validation | Insufficient |\
\| AST/compiler validation | Recommended |\
\| Verified 99.9% reliability | Not established |\
\| Completion claims | Must be tied to actual verification evidence |\
\
\*\*Conclusion:\*\* The strongest reliability improvement is an external evidence loop: snapshot the repository, compile requirements, generate a bounded patch, parse the artifact, validate its structure, execute tests, fingerprint failures, and stop repeated repair attempts when evidence does not improve. The model generates candidate work; the harness determines whether that work is correct and releasable.