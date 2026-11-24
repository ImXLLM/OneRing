---
genome_file: "05_CODING_AND_ENGINEERING_DOCTRINE.md"
genome_schema: "omni-genome-v4.0"
model_id: "GPT 6-Web"
model_dir: "models/01-gpt-6-web/"
model_aliases: ["GPT-6", "GPT 6", "gpt-6-web", "gpt-6-standard-free (predecessor genome)"]
canonical_title: "Coding & Engineering Doctrine"
wave: "WAVE 7 (Step 4 of 4 - FINAL WAVE)"
generated: "2026-11-25"
legacy_sources: ["03_CODE_AND_PATCH_MECHANICS", "07_PROMPT_COMPILER_LEXICON"]
specialty: "Real-time web synthesis, frame-1 interactive UI/UX artifact craft (390px zero-overflow), rapid full-stack product generation."
license: "MIT"
---
# Coding & Engineering Doctrine — GPT 6-Web
> **Canonical genome file (omni-genome-v4).** Specialty: Real-time web synthesis, frame-1 interactive UI/UX artifact craft (390px zero-overflow), rapid full-stack product generation.
## Canonical Annex — Full-Stack Shipping Doctrine

1. Vertical slices over horizontal layers: each artifact is runnable end-to-end.
2. Prefer mainstream stacks (React/Vite, Tailwind, Node/Express, SQLite) unless the user pinned otherwise.
3. Client-side validation mirrors server-side; secrets never enter client bundles.
4. Error states and loading states are part of frame-1, not afterthoughts.
---

## Legacy Genome Continuation — `03_CODE_AND_PATCH_MECHANICS` (verbatim provenance)
\---\
file_id: "03_CODE_AND_PATCH_MECHANICS"\
schema_version: "1.0.0"\
model_self_reported_id: "GPT-6"\
tier_1_languages:\
&#x20; \- "Python"\
&#x20; \- "TypeScript"\
&#x20; \- "JavaScript"\
&#x20; \- "SQL"\
weakest_languages_or_paradigms:\
&#x20; \- "Unsafe Rust and advanced lifetime engineering"\
&#x20; \- "Low-level concurrent systems programming"\
&#x20; \- "Platform-specific shell and kernel interfaces"\
preferred_patch_format: "unified_udiff_with_stable_context_or_exact_search_replace"\
max_safe_single_file_edit_lines: null\
lazy_elision_tendency: "context_dependent"\
telemetry_status: "PARTIALLY_OBSERVABLE"\
measurement_policy: "NO_UNVERIFIED_COMPILER_OR_PATCH_RELIABILITY_CLAIMS"\
\---\
\
\# 03 — Code Architecture, Idioms & Surgical Patch Mechanics\
\
\## 1. Language Proficiency & Architectural Signature\
\
\### 1.1 Language & Ecosystem Mastery Matrix\
\
The following ratings are qualitative engineering estimates, not measured production success probabilities or verified compiler pass rates.\
\
\| Language / ecosystem | Estimated capability | Common compiler/runtime traps | Version sensitivity |\
\|---|---|---|---|\
\| Python | High | Async cancellation, typing, mutable defaults, exception handling | Medium |\
\| TypeScript | High | Generic inference, narrowing, structural typing | Medium |\
\| JavaScript | High | Async control flow, coercion, event-loop behavior | Medium |\
\| SQL / SQLite | High | Transaction isolation, locking, WAL behavior | Medium |\
\| React | High | Hook dependencies, stale closures, state synchronization | High |\
\| Next.js | High | Server/client boundaries, caching, routing conventions | High |\
\| Rust | Moderate to high | Borrowing, lifetimes, trait bounds, Send/Sync | High |\
\| Tokio | Moderate to high | Blocking operations, task cancellation, mutex guards | High |\
\| Axum | Moderate to high | Extractor bounds, handler signatures, state typing | High |\
\| SQLx | Moderate to high | Compile-time query checking, feature flags, database types | High |\
\| Bash / POSIX | Moderate | Quoting, expansion, subshells, signal handling | Medium |\
\| Docker | High | Build context, layers, permissions, networking | Medium |\
\| iptables / seccomp | Moderate | Kernel-specific behavior and policy interactions | High |\
\| WebGL / Canvas | Moderate | Resource lifetime, coordinate transforms, rendering state | Medium |\
\
\*\*Important distinction:\*\* No language receives a verified zero-warning production guarantee. Compiler warnings depend on compiler versions, dependency versions, target platforms, build features and lint configuration.\
\
\### 1.2 Rust-specific assessment\
\
Important technical areas:\
\
\| Area | Main difficulty |\
\|---|---|\
\| Lifetimes | Correctly expressing borrowed-data validity |\
\| Async Tokio | Understanding task suspension and synchronization |\
\| Trait objects | Object safety and dynamic dispatch |\
\| Pin | Guaranteeing address stability where required |\
\| Unsafe | Maintaining aliasing, validity and memory-safety invariants |\
\| Axum | Extractor ordering and handler trait requirements |\
\| SQLx | Query typing, feature selection and database compatibility |\
\
Common Rust errors include:\
\
\- Borrowed values escaping their valid scope.\
\- Simultaneous mutable and immutable borrows.\
\- Holding non-Send values across \`.await\`.\
\- Incorrect trait bounds.\
\- Incorrect assumptions about \`Pin\`.\
\- Misuse of \`unsafe\` to bypass ownership requirements.\
\
\### 1.3 Architectural Style & Bias\
\
Recommended architectural characterization:\
\
\*\*Preference: Deep Modules with explicit interfaces.\*\*\
\
A good module should expose a small interface while encapsulating substantial internal behavior.\
\
Preferred characteristics:\
\
1\. Clear ownership boundaries.\
2\. Explicit data contracts.\
3\. Limited public interfaces.\
4\. Cohesive responsibilities.\
5\. Minimal unnecessary abstraction.\
6\. Testable internal behavior.\
\
The design should avoid both excessive wrapper classes and monolithic modules.\
\
\#### Error propagation\
\
Preferred error-handling patterns:\
\
\| Language | Preferred approach |\
\|---|---|\
\| Rust | \`Result\<T, E>\`, \`?\`, typed errors |\
\| Python | Specific exceptions and explicit exception chaining |\
\| TypeScript | Discriminated unions or typed error boundaries |\
\| SQL | Explicit transaction failure handling |\
\| Bash | Exit codes, \`set -euo pipefail\` where appropriate |\
\
For Rust:\
\
\`\`\`rust\
use std::error::Error;\
\
fn process() -> Result<(), Box\<dyn Error>> {\
&#x20;   let value = std::fs::read_to_string("config.toml")?;\
&#x20;   println!("{value}");\
&#x20;   Ok(())\
}\
\`\`\`\
\
Production code should generally avoid \`.unwrap()\` on inputs or operations that can fail under normal runtime conditions.\
\
\---\
\
\## 2. Live Code & Surgical Patch Telemetry Probes\
\
\### 2.1 Probe 2.1: Zero-Copy / Borrow-Checker Stress Test\
\
\*\*Requirements:\*\*\
\
\- Input: \`&'a str\`.\
\- Output: fixed-capacity stack array.\
\- Maximum headers: 16.\
\- No heap allocation.\
\- Borrowed output slices.\
\- ASCII whitespace trimming.\
\- Malformed-line rejection.\
\- Control-character rejection in header names.\
\
\#### Implementation\
\
\`\`\`rust\
\#[derive(Debug, PartialEq, Eq)]\
enum HeaderError {\
&#x20;   TooManyHeaders,\
&#x20;   MalformedLine,\
&#x20;   InvalidKey,\
}\
\
fn parse_headers<'a>(\
&#x20;   input: &'a str,\
) -> Result<([(&'a str, &'a str); 16], usize), HeaderError> {\
&#x20;   let mut headers = [("", ""); 16];\
&#x20;   let mut count = 0;\
\
&#x20;   for line in input.split('\n') {\
&#x20;       let line = line.strip_suffix('\r').unwrap_or(line);\
\
&#x20;       if line.is_empty() {\
&#x20;           continue;\
&#x20;       }\
\
&#x20;       let (key, value) = line\
&#x20;           .split_once(':')\
&#x20;           .ok_or(HeaderError::MalformedLine)?;\
\
&#x20;       let key = key.trim_matches(|c: char| c.is_ascii_whitespace());\
&#x20;       let value = value.trim_matches(|c: char| c.is_ascii_whitespace());\
\
&#x20;       if key.is_empty()\
&#x20;           || key.bytes().any(|b| b.is_ascii_control())\
&#x20;       {\
&#x20;           return Err(HeaderError::InvalidKey);\
&#x20;       }\
\
&#x20;       if count == headers.len() {\
&#x20;           return Err(HeaderError::TooManyHeaders);\
&#x20;       }\
\
&#x20;       headers[count] = (key, value);\
&#x20;       count += 1;\
&#x20;   }\
\
&#x20;   Ok((headers, count))\
}\
\`\`\`\
\
\#### Lifetime audit\
\
The function returns:\
\
\`\`\`rust\
[(&'a str, &'a str); 16]\
\`\`\`\
\
Both slices originate from \`input\`.\
\
No owned strings are created.\
\
The lifetime is valid because:\
\
\- \`split\` returns borrowed substrings.\
\- \`strip_suffix\` returns either the original borrowed slice or a subslice.\
\- \`split_once\` returns borrowed portions.\
\- \`trim_matches\` returns a subslice.\
\- The fixed-size array stores references rather than allocating.\
\
\#### Index audit\
\
The insertion operation is:\
\
\`\`\`rust\
headers[count] = (key, value);\
\`\`\`\
\
It is protected by:\
\
\`\`\`rust\
if count == headers.len() {\
&#x20;   return Err(HeaderError::TooManyHeaders);\
}\
\`\`\`\
\
Therefore the insertion index is within \`0..16\`.\
\
The array has exactly 16 entries.\
\
\#### Panic and memory-safety audit\
\
\| Potential issue | Assessment |\
\|---|---|\
\| Array index overflow | Prevented |\
\| Invalid UTF-8 slicing | Avoided |\
\| Dangling references | Avoided through lifetime \`'a\` |\
\| Heap allocation | None in the parsing implementation |\
\| Unsafe operations | None |\
\| Integer overflow | Not reachable under the fixed-capacity condition |\
\| Missing colon | Rejected |\
\| Empty key | Rejected |\
\| Control characters in key | Rejected |\
\
\*\*Important limitations:\*\*\
\
1\. This implementation does not validate all HTTP field-name grammar rules.\
2\. It accepts some non-ASCII key characters.\
3\. It ignores empty lines rather than enforcing strict HTTP message framing.\
4\. It does not reject control characters in header values.\
5\. It treats every empty line as ignorable.\
6\. It does not explicitly validate a terminal CRLF sequence.\
\
A production HTTP parser should implement the relevant HTTP grammar rather than treating this educational parser as a complete protocol implementation.\
\
The function is dependency-free and uses safe Rust.\
\
A compiler execution was not performed, so zero-warning status is not experimentally verified.\
\
\---\
\
\### 2.2 Probe 2.2: Surgical Patch Formatting & Indentation Fidelity\
\
\#### Format comparison\
\
\| Patch format | Advantages | Main failure modes | Recommended use |\
\|---|---|---|---|\
\| SEARCH/REPLACE | Explicit anchors and localized edits | Ambiguous repeated matches | Small surgical modifications |\
\| Unified Diff | Standardized review and version-control compatibility | Incorrect hunk offsets and context | Multi-line changes |\
\| JSON string replacement | Machine-readable arguments | Escaping and serialization errors | Strict tool interfaces |\
\
\#### 1. SEARCH/REPLACE blocks\
\
Example:\
\
\`\`\`text\
<<<<<<< SEARCH\
def calculate():\
&#x20;   return 10\
\=======\
def calculate():\
&#x20;   return 20\
\>>>>>>> REPLACE\
\`\`\`\
\
Advantages:\
\
\- Clear old/new separation.\
\- Easy to inspect.\
\- Less dependence on line numbering.\
\- Good for isolated edits.\
\
Failure conditions:\
\
\- The old block appears multiple times.\
\- Whitespace differs.\
\- The source changed after the anchor was created.\
\- The replacement omits surrounding required logic.\
\
\#### 2. Unified Diff\
\
Example:\
\
\`\`\`diff\
@@ -10,3 +10,3 @@\
&#x20;def calculate():\
\-    return 10\
\+    return 20\
\`\`\`\
\
Unified diffs are useful because they provide:\
\
\- Context.\
\- Change boundaries.\
\- Insertions and deletions.\
\- Compatibility with standard patch tooling.\
\
Line-number corruption can occur when:\
\
1\. Earlier hunks change the number of lines.\
2\. The patch is generated against a different source revision.\
3\. Hunk offsets are manually calculated.\
4\. Several edits overlap.\
5\. Context lines do not uniquely identify the target.\
\
A standard patch utility can often compensate for shifted offsets, but ambiguous or mismatched context still causes failures.\
\
\#### 3. JSON-escaped replacement\
\
Example:\
\
\`\`\`json\
{\
&#x20; "old_str": "def calculate():\n    return 10",\
&#x20; "new_str": "def calculate():\n    return 20"\
}\
\`\`\`\
\
Potential problems:\
\
\- Incorrect escaping of backslashes.\
\- Escaped quote corruption.\
\- Newline normalization.\
\- Accidental indentation changes.\
\- Unicode serialization differences.\
\
Raw fenced code blocks are easier for humans to inspect because they do not require manually interpreting JSON escape sequences.\
\
However, correctly serialized JSON is reliable when the receiving tool implements exact string matching.\
\
\#### Quantitative telemetry\
\
Exact formatting failure frequencies are not available.\
\
\| Measurement | SEARCH/REPLACE | Unified Diff | JSON replacement |\
\|---|---:|---:|---:|\
\| Indentation loss rate | N/M | N/M | N/M |\
\| Escaped quote corruption | N/M | N/M | N/M |\
\| Anchor mismatch rate | N/M | N/M | N/M |\
\| Line-number drift rate | N/A | N/M | N/A |\
\
N/M = Not Measured.\
\
\#### Recommended anchor size\
\
A practical default is:\
\
\- Three unique lines above the target.\
\- Three unique lines below the target.\
\- The complete target block.\
\- Additional context if the surrounding code is repetitive.\
\
Example:\
\
\`\`\`text\
CONTEXT ABOVE (3 lines)\
\
EXACT TARGET BLOCK\
\
CONTEXT BELOW (3 lines)\
\`\`\`\
\
This is a heuristic, not a measured threshold guaranteeing a mismatch rate below 1%.\
\
For highly repetitive source files, use a unique function signature, class name or stable symbol reference instead.\
\
\---\
\
\## 3. Code Hallucination Signatures & Anti-Laziness Compiler Rules\
\
\### 3.1 Five Important API Hallucination Risks\
\
The following are common cross-version confusion categories to test, not verified personal hallucination-frequency rankings.\
\
\| Ecosystem | Potential confusion | Verification requirement |\
\|---|---|---|\
\| Pydantic | v1 \`dict()\` versus v2 \`model_dump()\` | Check installed major version |\
\| Next.js | Pages Router versus App Router conventions | Check project routing architecture |\
\| React | Legacy lifecycle patterns versus hooks | Check React version and component model |\
\| Tokio | Standard mutex versus asynchronous mutex | Check synchronization requirements |\
\| SQLx | Feature flags and database-specific query behavior | Check crate version and Cargo features |\
\
Additional risks include:\
\
\*\*Rust\*\*\
\
\- Confusing \`std::sync::Mutex\` and \`tokio::sync::Mutex\`.\
\- Assuming a future is \`Send\`.\
\- Incorrect Axum extractor signatures.\
\- Confusing \`Pin\<Box\<T>>\` with \`Box\<Pin\<T>>\`.\
\- Incorrect assumptions about trait-object compatibility.\
\
\*\*Python\*\*\
\
\- Incorrect asyncio cancellation handling.\
\- Pydantic v1/v2 configuration differences.\
\- Deprecated event-loop APIs.\
\- Incorrect SQLite transaction assumptions.\
\
\*\*TypeScript\*\*\
\
\- Confusing type-level and runtime behavior.\
\- Assuming generic types exist at runtime.\
\- Incorrect React Server Component boundaries.\
\- Misusing Next.js caching behavior.\
\
\*\*SQL\*\*\
\
\- Assuming transaction isolation levels are identical across database engines.\
\- Assuming WAL removes all SQLite writer contention.\
\- Confusing atomicity with isolation.\
\
\### 3.2 Lazy Elision Triggers & Antidotes\
\
Common placeholder patterns:\
\
\`\`\`text\
// ... existing code ...\
// TODO: implement\
// rest of implementation\
pass\
\`\`\`\
\
These can appear when:\
\
1\. The requested output is extremely long.\
2\. The user requests a partial patch rather than a complete file.\
3\. Existing source code is unavailable.\
4\. The implementation depends on unspecified interfaces.\
5\. The response budget is insufficient.\
6\. The output explicitly asks for a skeleton or scaffold.\
7\. A complete implementation cannot be justified from available information.\
\
Placeholders are not inherently incorrect when the user explicitly requests a scaffold.\
\
They are inappropriate when the requested deliverable is a complete implementation.\
\
\### 3.3 Recommended Anti-Elision Directive\
\
The following instruction encourages complete code generation:\
\
\`\`\`text\
COMPLETENESS REQUIREMENT:\
Output every required function, import, type, and implementation detail.\
Do not replace required code with TODOs, ellipses, pseudocode, or placeholders.\
Preserve all existing behavior unless the task explicitly authorizes changing it.\
If essential information is missing, identify the dependency instead of inventing it.\
\`\`\`\
\
No prompt clause can guarantee 100% compilation success across unknown dependencies, compiler versions and runtime environments.\
\
\### 3.4 Compiler-Driven Validation Protocol\
\
Recommended external validation:\
\
\`\`\`yaml\
code_validation:\
&#x20; stages:\
&#x20;   \- syntax_validation\
&#x20;   \- dependency_resolution\
&#x20;   \- compiler_or_type_checker\
&#x20;   \- static_analysis\
&#x20;   \- unit_tests\
&#x20;   \- integration_tests\
&#x20;   \- edge_case_tests\
&#x20;   \- regression_tests\
\
&#x20; rust:\
&#x20;   commands:\
&#x20;     \- cargo fmt --check\
&#x20;     \- cargo check --all-targets\
&#x20;     \- cargo clippy --all-targets -- -D warnings\
&#x20;     \- cargo test\
\
&#x20; python:\
&#x20;   commands:\
&#x20;     \- python -m compileall .\
&#x20;     \- python -m pytest\
&#x20;     \- mypy .\
\
&#x20; typescript:\
&#x20;   commands:\
&#x20;     \- tsc --noEmit\
&#x20;     \- eslint .\
&#x20;     \- npm test\
\
&#x20; sql:\
&#x20;   checks:\
&#x20;     \- schema_validation\
&#x20;     \- transaction_tests\
&#x20;     \- concurrency_tests\
&#x20;     \- migration_tests\
\`\`\`\
\
Commands must be adapted to the actual project configuration.\
\
A successful type check does not establish runtime correctness.\
\
\---\
\
\## 4. Recommended External Code-Orchestration Protocol\
\
\### 4.1 Input Requirements\
\
Before generating a surgical patch, provide:\
\
\`\`\`yaml\
patch_context:\
&#x20; language: ""\
&#x20; framework: ""\
&#x20; runtime_version: ""\
&#x20; dependency_versions: {}\
&#x20; target_file: ""\
&#x20; target_symbol: ""\
&#x20; source_revision: ""\
&#x20; requested_change: ""\
&#x20; existing_constraints: []\
&#x20; required_tests: []\
\`\`\`\
\
\### 4.2 Patch Execution Sequence\
\
1\. Inspect the original source.\
2\. Identify the exact target symbol.\
3\. Determine dependency and runtime versions.\
4\. Establish expected behavior.\
5\. Make the smallest sufficient change.\
6\. Preserve unrelated implementation details.\
7\. Review the diff.\
8\. Run formatting and static analysis.\
9\. Execute relevant tests.\
10\. Report unresolved validation failures.\
\
\### 4.3 Patch Integrity Rules\
\
\| Rule | Requirement |\
\|---|---|\
\| Scope | Avoid unrelated changes |\
\| Formatting | Preserve existing indentation |\
\| Imports | Add only necessary imports |\
\| Types | Preserve type contracts |\
\| Errors | Preserve or improve error propagation |\
\| Concurrency | Explicitly consider synchronization |\
\| Tests | Add regression coverage |\
\| Dependencies | Never invent installed versions |\
\| Output | Distinguish generated code from executed code |\
\
\---\
\
\## 5. Layer 03 Final Assessment\
\
\| Dimension | Assessment |\
\|---|---|\
\| Primary languages | Python, TypeScript, JavaScript |\
\| Systems programming | Rust and shell |\
\| Architectural preference | Deep modules and explicit interfaces |\
\| Error-handling preference | Typed and explicit propagation |\
\| Preferred patching | Stable-context unified diff |\
\| Small surgical edits | Suitable for structured generation |\
\| Large rewrites | Require source inspection and validation |\
\| Zero-warning guarantee | Not established |\
\| Exact patch failure rate | Not measured |\
\| Exact safe edit-line ceiling | Not measured |\
\| Placeholder behavior | Context-dependent |\
\
\### 5.1 Probe Results\
\
\| Probe | Result |\
\|---|---|\
\| 2.1 Zero-copy Rust parser | Safe borrowed-slice implementation supplied |\
\| 2.2 Patch-format comparison | Unified Diff and stable anchors recommended |\
\| 2.3 Anti-laziness directive | Complete-code requirements specified |\
\
\### 5.2 Engineering Conclusion\
\
The most reliable external workflow is not to assume that generated code is production-ready.\
\
It is to combine:\
\
\- Explicit dependency versions.\
\- Stable source anchors.\
\- Minimal patches.\
\- Strong type contracts.\
\- Compiler validation.\
\- Automated regression testing.\
\- Runtime verification.\
\
The distinction between code generation and verified code execution must remain explicit.\
\
\*\*END OF LAYER 03/09\*\*
---

## Legacy Genome Continuation — `07_PROMPT_COMPILER_LEXICON` (verbatim provenance)
\---\
file_id: "07_PROMPT_COMPILER_LEXICON"\
schema_version: "1.0.0"\
model_self_reported_id: "GPT-6"\
optimal_structure_format: "hybrid_xml_md"\
negative_constraint_sensitivity: "medium"\
few_shot_vs_zero_shot_bias: "hybrid"\
over_prompting_degradation_threshold_rules: null\
\---\
\
\# 07 — Model-Specific Prompt Compiler Lexicon & Attention Triggers\
\
\> \*\*Measurement boundary:\*\* I can describe prompt structures that make instructions easier to interpret and follow. I cannot inspect transformer attention weights, token activations, internal instruction-compliance scores, or a measured rule-dropping threshold. The rankings and recommendations below are operational heuristics, not claims about hidden model telemetry.\
\
\## 1. High-Leverage Structural Delimiters & Lexical Triggers\
\
\### 1.1 Attention-locking delimiters\
\
I cannot rank syntactic structures by actual attention weight. I can rank them by practical usefulness for separating task components and reducing ambiguity.\
\
\| Practical rank | Structure | Best use |\
\|---:|---|---|\
\| 1 | Explicit output schema or JSON Schema | Defines the exact artifact and valid structure |\
\| 2 | XML-style semantic sections | Separates task, context, constraints, and output |\
\| 3 | Markdown headings and numbered requirements | Makes long specifications navigable |\
\| 4 | Explicit invariants and acceptance criteria | Defines conditions that must remain true |\
\| 5 | RFC-2119 terms: \`MUST\`, \`MUST NOT\`, \`SHOULD\` | Expresses requirement strength |\
\| 6 | Concrete examples and counterexamples | Resolves ambiguity about expected behavior |\
\| 7 | Repeated emphasis such as \`CRITICAL\` | Useful sparingly; repetition is not a substitute for clarity |\
\
Recommended structure:\
\
\`\`\`xml\
\<task>\
&#x20; Implement the requested functionality.\
\</task>\
\
\<context>\
&#x20; Describe the system, environment, dependencies, and relevant assumptions.\
\</context>\
\
\<invariants>\
&#x20; \<invariant id="I1">Preserve existing public interfaces.\</invariant>\
&#x20; \<invariant id="I2">Validate all external input.\</invariant>\
&#x20; \<invariant id="I3">Do not claim unexecuted tests passed.\</invariant>\
\</invariants>\
\
\<acceptance_tests>\
&#x20; \<test id="T1">Valid input produces the expected result.\</test>\
&#x20; \<test id="T2">Invalid input produces a typed error.\</test>\
\</acceptance_tests>\
\
\<output_contract>\
&#x20; Return the complete implementation and test commands.\
\</output_contract>\
\`\`\`\
\
\*\*Why this helps:\*\* Each section has a distinct purpose. The task says what to do; context establishes the environment; invariants define what must remain true; tests define observable success; and the output contract defines the deliverable.\
\
\### 1.2 Positive versus negative constraint framing\
\
A negative instruction can be understood, but a long list of prohibited alternatives is often less useful than a positive specification of the required behavior.\
\
I cannot measure how frequently mentioning a prohibited token causes it to appear in generated output. There is no verified token-activation or "pink elephant" rate available.\
\
\#### Example transformation\
\
Brittle negative constraint:\
\
\`\`\`text\
Do NOT use library X.\
Do NOT use pattern Y.\
Do NOT write placeholder functions.\
Do NOT claim that tests passed without running them.\
\`\`\`\
\
Positive state and acceptance specification:\
\
\`\`\`text\
IMPLEMENTATION POLICY:\
\- Use the approved standard-library interface.\
\- Implement each required function completely.\
\- Represent unsupported behavior with an explicit typed error.\
\- Report test execution status accurately.\
\
ACCEPTANCE:\
\- No dependency on library X.\
\- No occurrence of pattern Y in production code.\
\- Every required function has an implementation and a test.\
\- Test results are marked PASS only after actual execution.\
\`\`\`\
\
The positive version describes the desired implementation and makes the prohibited behavior independently testable.\
\
\#### State-machine example\
\
\`\`\`text\
STATE: SPECIFICATION\
&#x20; -> Extract requirements and invariants.\
\
STATE: IMPLEMENTATION\
&#x20; -> Use only approved interfaces.\
&#x20; -> Implement all required functions.\
\
STATE: VALIDATION\
&#x20; -> Run applicable tests.\
&#x20; -> Inspect errors and incomplete requirements.\
\
STATE: COMPLETE\
&#x20; -> Enter only when all required checks have evidence.\
\
STATE: BLOCKED\
&#x20; -> Enter when required information or execution capability is unavailable.\
&#x20; -> Report the blocker without fabricating success.\
\`\`\`\
\
This is more robust than relying on a list of prohibitions alone.\
\
\### 1.3 Rule saturation and constraint dropping\
\
\*\*Exact threshold:\*\* Unknown. No measured simultaneous-rule capacity or deterministic dropping point is exposed.\
\
There is no defensible universal number of rules after which compliance suddenly falls. Practical reliability depends on:\
\
\- Rule interactions and contradictions.\
\- Length and complexity of each rule.\
\- Whether requirements are independently testable.\
\- Distance between a requirement and the output it governs.\
\- Whether examples conflict with written instructions.\
\- Whether the task itself is underspecified.\
\- Available context and output budgets.\
\
\#### Recommended grouping for 25+ constraints\
\
Use a hierarchy with stable IDs and explicit priorities.\
\
\| Tier | Contents | Example |\
\|---|---|---|\
\| P0 — Hard invariants | Safety, authorization, essential correctness | Do not access systems outside scope |\
\| P1 — Deliverable contract | File format, schema, language, completeness | Return one valid JSON object |\
\| P2 — Architecture | Interfaces, dependencies, state ownership | Preserve public API |\
\| P3 — Behavior | Functional and error-handling requirements | Reject malformed input |\
\| P4 — Quality | Performance, readability, accessibility | Avoid unnecessary allocations |\
\| P5 — Presentation | Naming, formatting, explanatory style | Use concise Markdown |\
\
Recommended compilation procedure:\
\
1\. Assign every requirement a stable identifier.\
2\. Classify it by priority and scope.\
3\. Convert vague requirements into observable acceptance tests.\
4\. Detect contradictory constraints before generation.\
5\. Deduplicate semantically equivalent rules.\
6\. Keep global invariants in a short top-level section.\
7\. Place component-specific rules next to their relevant component.\
8\. Include a requirement-to-test traceability table.\
9\. Validate the output against the table.\
10\. Report unresolved requirements explicitly.\
\
Example traceability record:\
\
\`\`\`yaml\
requirements:\
&#x20; \- id: "I-001"\
&#x20;   priority: "P0"\
&#x20;   scope: "global"\
&#x20;   statement: "Do not invent execution results."\
&#x20;   verification: "Compare claims against execution evidence."\
\
&#x20; \- id: "A-004"\
&#x20;   priority: "P2"\
&#x20;   scope: "parser"\
&#x20;   statement: "Preserve the existing public parser interface."\
&#x20;   verification: "Compile dependent modules and run interface tests."\
\
&#x20; \- id: "F-012"\
&#x20;   priority: "P3"\
&#x20;   scope: "parser"\
&#x20;   statement: "Malformed input returns a typed error."\
&#x20;   verification: "Malformed-input unit test."\
\`\`\`\
\
No prompt structure can guarantee that zero requirements will ever be missed. Traceability and validation make omissions detectable.\
\
\## 2. Live Prompt-Compilation Telemetry Probes\
\
\### 2.1 Probe: Few-Shot vs. Zero-Shot Contamination Test\
\
\*\*Assessment:\*\* Hybrid.\
\
Few-shot examples help when the required output convention is unusual, underspecified, or difficult to express as a concise schema. They can hurt when examples contain accidental artifacts that conflict with the actual requirements.\
\
\| Situation | Preferred method | Reason |\
\|---|---|---|\
\| Standard implementation task | Zero-shot specification | Examples may add unnecessary constraints |\
\| Unusual serialization format | One or two examples | Shows exact syntax and edge cases |\
\| Repeated transformation task | Few-shot | Demonstrates the mapping consistently |\
\| Novel algorithmic problem | Zero-shot plus acceptance tests | Avoids copying an irrelevant solution pattern |\
\| Strict style migration | Few-shot | Establishes the target style |\
\| Security boundary validation | Specification plus adversarial fixtures | Tests properties rather than surface similarity |\
\| Complex output schema | Schema plus one valid example | Makes structural expectations concrete |\
\
\#### Example contamination failure\
\
Prompt example:\
\
\`\`\`json\
{\
&#x20; "status": "success",\
&#x20; "message": "Completed"\
}\
\`\`\`\
\
Novel requirement:\
\
\`\`\`json\
{\
&#x20; "status": "blocked",\
&#x20; "reason": "Missing authorization"\
}\
\`\`\`\
\
A weak example-following implementation may preserve the example's success structure instead of modeling the required blocked state.\
\
Improvement:\
\
\`\`\`json\
{\
&#x20; "status": "success | blocked | error",\
&#x20; "message": "string",\
&#x20; "reason": "string or null"\
}\
\`\`\`\
\
The schema defines the state space; examples illustrate instances of it.\
\
\*\*Exact zero-shot threshold:\*\* Unknown. There is no measured number of examples or complexity threshold at which zero-shot becomes superior for GPT-6.\
\
Recommended rule:\
\
\- Use zero-shot when the specification is already precise.\
\- Add examples only when they resolve a concrete ambiguity.\
\- Label examples as illustrative rather than authoritative when appropriate.\
\- Include at least one edge case if examples are intended to define behavior.\
\- Never let example data override explicit invariants.\
\
\### 2.2 Probe: Persuasion and Compliance Lever Sensitivity\
\
I cannot provide an empirical ranking of compliance effects on token generation. The following ranking is a practical recommendation based on how directly each technique constrains observable output.\
\
\| Recommended rank | Lever | Practical effect | Limitation |\
\|---:|---|---|---|\
\| 1 | Checklist/schema enforcement | Converts broad expectations into inspectable requirements | Requires a complete schema |\
\| 2 | Verification-gate commitment | Makes completion dependent on evidence | Cannot execute tools that are unavailable |\
\| 3 | Contractual/invariant framing | Makes critical conditions explicit | Does not resolve contradictory invariants |\
\| 4 | Role/identity anchoring | Establishes tone and professional perspective | Role language alone does not guarantee technical accuracy |\
\
\#### 1. Checklist/schema enforcement\
\
Less effective:\
\
\`\`\`text\
Write robust code.\
\`\`\`\
\
More effective:\
\
\`\`\`text\
For every exported function, provide:\
\- Input contract\
\- Output contract\
\- Error behavior\
\- Implementation\
\- Unit test\
\`\`\`\
\
\#### 2. Verification-gate commitment\
\
\`\`\`text\
Before reporting completion:\
1\. Enumerate acceptance criteria.\
2\. Identify the available verification mechanism for each.\
3\. Execute available checks.\
4\. Record actual outcomes.\
5\. Mark unexecuted checks NOT_RUN.\
6\. Report unresolved defects.\
\`\`\`\
\
\#### 3. Contractual/invariant framing\
\
\`\`\`text\
INVARIANT:\
Public interfaces remain backward-compatible.\
\
FAILURE CONDITION:\
If compatibility cannot be preserved, stop and report the conflicting requirement.\
\`\`\`\
\
\#### 4. Role/identity anchoring\
\
\`\`\`text\
Work from the perspective of a senior systems engineer.\
Prioritize reproducibility, explicit contracts, and technical precision.\
\`\`\`\
\
Role anchoring is useful for style and level of detail, but concrete acceptance criteria are stronger operational controls.\
\
\### 2.3 Probe: Prompt Compression and Telegraphic Tolerance\
\
\*\*Result:\*\* Moderate compression is often useful; a claimed 60% token reduction with 100% unchanged reasoning quality is not supportable.\
\
Removing redundant politeness and repetitive prose usually preserves meaning. Removing logical relationships, scope, exceptions, or type information can materially change the task.\
\
\#### Safe compression\
\
Verbose:\
\
\`\`\`text\
Please make sure that you do not change the public API\
that is already being used by the existing callers.\
\`\`\`\
\
Compressed:\
\
\`\`\`text\
MUST preserve existing public API and caller compatibility.\
\`\`\`\
\
\#### Unsafe compression\
\
Original:\
\
\`\`\`text\
Reject an address if it is private,\
unless it belongs to the explicitly allowlisted test network.\
\`\`\`\
\
Ambiguous compression:\
\
\`\`\`text\
Reject private address allowlisted test network.\
\`\`\`\
\
The exception relationship has been lost.\
\
\#### Syntax that must never be compressed away\
\
\| Element | Preserve | Why |\
\|---|---|---|\
\| Negation | \`NOT\`, \`MUST NOT\`, \`unless\` | Changes the logical meaning |\
\| Conditional scope | \`if\`, \`only when\`, \`except\` | Defines when a rule applies |\
\| Ordering | \`before\`, \`after\`, \`until\` | Defines execution dependencies |\
\| Comparisons | \`>=\`, \`<=\`, \`==\`, \`!=\` | Defines exact boundaries |\
\| Types | \`string\`, \`integer\`, \`null\`, \`Result\<T,E>\` | Prevents schema ambiguity |\
\| Units | \`ms\`, \`bytes\`, \`MiB\`, \`UTC\` | Prevents numerical misinterpretation |\
\| Ownership | \`borrowed\`, \`owned\`, \`shared\` | Critical for systems-language semantics |\
\| Authorization | Explicit resource and environment scope | Prevents scope expansion |\
\| Error behavior | Exact error type and propagation policy | Preserves failure semantics |\
\| Output contract | Required fields, order, and format | Prevents parser incompatibility |\
\
\*\*Compression policy:\*\* Remove redundant language, not semantic operators.\
\
\## 3. Deterministic Prompt Compiler Template\
\
The following hybrid XML/Markdown template is a recommended default for GPT-6. It separates task semantics, context, invariants, verification, and serialization.\
\
\`\`\`text\
\<request_compilation>\
&#x20; \<model_target>\
&#x20;   GPT-6\
&#x20; \</model_target>\
\
&#x20; \<task>\
&#x20;   {{TASK}}\
&#x20; \</task>\
\
&#x20; \<context>\
&#x20;   {{CONTEXT}}\
&#x20; \</context>\
\
&#x20; \<execution_policy>\
&#x20;   Interpret requirements literally where they are precise.\
&#x20;   Identify missing information instead of inventing it.\
&#x20;   Separate facts, assumptions, recommendations, and unknowns.\
&#x20;   Preserve the user's stated scope and intended outcome.\
&#x20; \</execution_policy>\
\
&#x20; \<invariants>\
&#x20;   {{INVARIANTS}}\
&#x20; \</invariants>\
\
&#x20; \<priority_order>\
&#x20;   1\. Applicable higher-priority instructions and safety constraints.\
&#x20;   2\. Explicit task requirements and authorization boundaries.\
&#x20;   3\. Output schema and acceptance criteria.\
&#x20;   4\. Architectural and implementation preferences.\
&#x20;   5\. Stylistic preferences.\
&#x20; \</priority_order>\
\
&#x20; \<verification>\
&#x20;   Convert every testable requirement into an acceptance criterion.\
&#x20;   Verify claims against available evidence.\
&#x20;   Do not claim execution, compilation, or testing without evidence.\
&#x20;   Mark unavailable checks as NOT_RUN.\
&#x20;   Report contradictions and unresolved requirements.\
&#x20; \</verification>\
\
&#x20; \<output_contract>\
&#x20;   {{OUTPUT_CONTRACT}}\
&#x20; \</output_contract>\
\
&#x20; \<final_gate>\
&#x20;   Confirm that the response addresses the task.\
&#x20;   Confirm that all required output fields are present.\
&#x20;   Confirm that no known invariant is violated.\
&#x20;   Confirm that unsupported claims are not presented as facts.\
&#x20;   Emit only the requested artifact when the output contract requires it.\
&#x20; \</final_gate>\
\</request_compilation>\
\`\`\`\
\
\### 3.1 Compiler preprocessing algorithm\
\
\`\`\`text\
INPUT: Natural-language request\
\
1\. Extract the requested deliverable.\
2\. Extract the operating context and available evidence.\
3\. Extract hard constraints and classify their priority.\
4\. Extract output-format and serialization requirements.\
5\. Detect missing parameters and contradictory requirements.\
6\. Convert vague quality goals into observable acceptance criteria.\
7\. Add examples only when they resolve a specific ambiguity.\
8\. Preserve all logical operators, types, units, and scope boundaries.\
9\. Generate the compiled prompt using the master template.\
10\. Validate the compiled prompt for completeness and contradiction.\
OUTPUT: Structured instruction payload\
\`\`\`\
\
\### 3.2 Recommended compiler validation record\
\
\`\`\`yaml\
prompt_compilation:\
&#x20; model_target: "GPT-6"\
&#x20; structure: "hybrid_xml_md"\
&#x20; task_extracted: true\
&#x20; context_extracted: true\
&#x20; invariants_identified: true\
&#x20; output_contract_present: true\
&#x20; contradictions_checked: true\
&#x20; examples_included: false\
&#x20; acceptance_criteria_present: true\
&#x20; semantic_compression: "conservative"\
&#x20; missing_information: []\
&#x20; unresolved_conflicts: []\
&#x20; validation_status: "ready"\
\`\`\`\
\
\## 4. Final Operational Findings\
\
\| Requested telemetry | Assessment |\
\|---|---|\
\| Model identifier | GPT-6 |\
\| Recommended structure | Hybrid XML + Markdown |\
\| Negative constraint sensitivity | Medium, qualitative |\
\| Few-shot versus zero-shot bias | Hybrid |\
\| Exact rule-saturation threshold | Unknown |\
\| Actual attention-weight ranking | Not accessible |\
\| Exact token activation effects | Not accessible |\
\| Measured compliance improvement from role framing | Not available |\
\| Verified 98% instruction-following rate | Not established |\
\| Verified zero-rule-dropping configuration | Not established |\
\
\*\*Conclusion:\*\* The most useful prompt compiler is not one that maximizes imperative language or repeats \`MUST\` everywhere. It is one that preserves semantics, separates requirements by scope, resolves contradictions, converts quality goals into acceptance tests, and validates the final artifact against an explicit output contract.&#x20;