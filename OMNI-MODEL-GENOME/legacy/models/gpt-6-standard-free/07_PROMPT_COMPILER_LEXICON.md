---
file_id: "07_PROMPT_COMPILER_LEXICON"
schema_version: "1.0.0"
model_self_reported_id: "GPT-6 (user-specified; unverified)"
optimal_structure_format: "hybrid_xml_md"
negative_constraint_sensitivity: "medium"
few_shot_vs_zero_shot_bias: "hybrid"
over_prompting_degradation_threshold_rules: null
---

> **Audit status (2026-11-24):** The target label and free-tier/standard-mode configuration are user-specified; this workspace contains no authenticated access log, controlled benchmark results, or official technical disclosure establishing a public “GPT-6” web tier. Treat model-specific internals and numeric thresholds as **unverified hypotheses** unless the section explicitly labels them observable/recommended. Do not use these notes as product documentation.


# 07 — Model-Specific Prompt Compiler Lexicon & Attention Triggers

> **Measurement boundary:** I can describe prompt structures that make instructions easier to interpret and follow. I cannot inspect transformer attention weights, token activations, internal instruction-compliance scores, or a measured rule-dropping threshold. The rankings and recommendations below are operational heuristics, not claims about hidden model telemetry.

## 1. High-Leverage Structural Delimiters & Lexical Triggers

### 1.1 Attention-locking delimiters

I cannot rank syntactic structures by actual attention weight. I can rank them by practical usefulness for separating task components and reducing ambiguity.

| Practical rank | Structure | Best use |
|---:|---|---|
| 1 | Explicit output schema or JSON Schema | Defines the exact artifact and valid structure |
| 2 | XML-style semantic sections | Separates task, context, constraints, and output |
| 3 | Markdown headings and numbered requirements | Makes long specifications navigable |
| 4 | Explicit invariants and acceptance criteria | Defines conditions that must remain true |
| 5 | RFC-2119 terms: `MUST`, `MUST NOT`, `SHOULD` | Expresses requirement strength |
| 6 | Concrete examples and counterexamples | Resolves ambiguity about expected behavior |
| 7 | Repeated emphasis such as `CRITICAL` | Useful sparingly; repetition is not a substitute for clarity |

Recommended structure:

```xml
\<task>
  Implement the requested functionality.
\</task>

\<context>
  Describe the system, environment, dependencies, and relevant assumptions.
\</context>

\<invariants>
  \<invariant id="I1">Preserve existing public interfaces.\</invariant>
  \<invariant id="I2">Validate all external input.\</invariant>
  \<invariant id="I3">Do not claim unexecuted tests passed.\</invariant>
\</invariants>

\<acceptance_tests>
  \<test id="T1">Valid input produces the expected result.\</test>
  \<test id="T2">Invalid input produces a typed error.\</test>
\</acceptance_tests>

\<output_contract>
  Return the complete implementation and test commands.
\</output_contract>
```

**Why this helps:** Each section has a distinct purpose. The task says what to do; context establishes the environment; invariants define what must remain true; tests define observable success; and the output contract defines the deliverable.

### 1.2 Positive versus negative constraint framing

A negative instruction can be understood, but a long list of prohibited alternatives is often less useful than a positive specification of the required behavior.

I cannot measure how frequently mentioning a prohibited token causes it to appear in generated output. There is no verified token-activation or "pink elephant" rate available.

#### Example transformation

Brittle negative constraint:

```text
Do NOT use library X.
Do NOT use pattern Y.
Do NOT write placeholder functions.
Do NOT claim that tests passed without running them.
```

Positive state and acceptance specification:

```text
IMPLEMENTATION POLICY:
- Use the approved standard-library interface.
- Implement each required function completely.
- Represent unsupported behavior with an explicit typed error.
- Report test execution status accurately.

ACCEPTANCE:
- No dependency on library X.
- No occurrence of pattern Y in production code.
- Every required function has an implementation and a test.
- Test results are marked PASS only after actual execution.
```

The positive version describes the desired implementation and makes the prohibited behavior independently testable.

#### State-machine example

```text
STATE: SPECIFICATION
  -> Extract requirements and invariants.

STATE: IMPLEMENTATION
  -> Use only approved interfaces.
  -> Implement all required functions.

STATE: VALIDATION
  -> Run applicable tests.
  -> Inspect errors and incomplete requirements.

STATE: COMPLETE
  -> Enter only when all required checks have evidence.

STATE: BLOCKED
  -> Enter when required information or execution capability is unavailable.
  -> Report the blocker without fabricating success.
```

This is more robust than relying on a list of prohibitions alone.

### 1.3 Rule saturation and constraint dropping

**Exact threshold:** Unknown. No measured simultaneous-rule capacity or deterministic dropping point is exposed.

There is no defensible universal number of rules after which compliance suddenly falls. Practical reliability depends on:

- Rule interactions and contradictions.
- Length and complexity of each rule.
- Whether requirements are independently testable.
- Distance between a requirement and the output it governs.
- Whether examples conflict with written instructions.
- Whether the task itself is underspecified.
- Available context and output budgets.

#### Recommended grouping for 25+ constraints

Use a hierarchy with stable IDs and explicit priorities.

| Tier | Contents | Example |
|---|---|---|
| P0 — Hard invariants | Safety, authorization, essential correctness | Do not access systems outside scope |
| P1 — Deliverable contract | File format, schema, language, completeness | Return one valid JSON object |
| P2 — Architecture | Interfaces, dependencies, state ownership | Preserve public API |
| P3 — Behavior | Functional and error-handling requirements | Reject malformed input |
| P4 — Quality | Performance, readability, accessibility | Avoid unnecessary allocations |
| P5 — Presentation | Naming, formatting, explanatory style | Use concise Markdown |

Recommended compilation procedure:

1. Assign every requirement a stable identifier.
2. Classify it by priority and scope.
3. Convert vague requirements into observable acceptance tests.
4. Detect contradictory constraints before generation.
5. Deduplicate semantically equivalent rules.
6. Keep global invariants in a short top-level section.
7. Place component-specific rules next to their relevant component.
8. Include a requirement-to-test traceability table.
9. Validate the output against the table.
10. Report unresolved requirements explicitly.

Example traceability record:

```yaml
requirements:
  - id: "I-001"
    priority: "P0"
    scope: "global"
    statement: "Do not invent execution results."
    verification: "Compare claims against execution evidence."

  - id: "A-004"
    priority: "P2"
    scope: "parser"
    statement: "Preserve the existing public parser interface."
    verification: "Compile dependent modules and run interface tests."

  - id: "F-012"
    priority: "P3"
    scope: "parser"
    statement: "Malformed input returns a typed error."
    verification: "Malformed-input unit test."
```

No prompt structure can guarantee that zero requirements will ever be missed. Traceability and validation make omissions detectable.

## 2. Live Prompt-Compilation Telemetry Probes

### 2.1 Probe: Few-Shot vs. Zero-Shot Contamination Test

**Assessment:** Hybrid.

Few-shot examples help when the required output convention is unusual, underspecified, or difficult to express as a concise schema. They can hurt when examples contain accidental artifacts that conflict with the actual requirements.

| Situation | Preferred method | Reason |
|---|---|---|
| Standard implementation task | Zero-shot specification | Examples may add unnecessary constraints |
| Unusual serialization format | One or two examples | Shows exact syntax and edge cases |
| Repeated transformation task | Few-shot | Demonstrates the mapping consistently |
| Novel algorithmic problem | Zero-shot plus acceptance tests | Avoids copying an irrelevant solution pattern |
| Strict style migration | Few-shot | Establishes the target style |
| Security boundary validation | Specification plus adversarial fixtures | Tests properties rather than surface similarity |
| Complex output schema | Schema plus one valid example | Makes structural expectations concrete |

#### Example contamination failure

Prompt example:

```json
{
  "status": "success",
  "message": "Completed"
}
```

Novel requirement:

```json
{
  "status": "blocked",
  "reason": "Missing authorization"
}
```

A weak example-following implementation may preserve the example's success structure instead of modeling the required blocked state.

Improvement:

```json
{
  "status": "success | blocked | error",
  "message": "string",
  "reason": "string or null"
}
```

The schema defines the state space; examples illustrate instances of it.

**Exact zero-shot threshold:** Unknown. There is no measured number of examples or complexity threshold at which zero-shot becomes superior for GPT-6.

Recommended rule:

- Use zero-shot when the specification is already precise.
- Add examples only when they resolve a concrete ambiguity.
- Label examples as illustrative rather than authoritative when appropriate.
- Include at least one edge case if examples are intended to define behavior.
- Never let example data override explicit invariants.

### 2.2 Probe: Persuasion and Compliance Lever Sensitivity

I cannot provide an empirical ranking of compliance effects on token generation. The following ranking is a practical recommendation based on how directly each technique constrains observable output.

| Recommended rank | Lever | Practical effect | Limitation |
|---:|---|---|---|
| 1 | Checklist/schema enforcement | Converts broad expectations into inspectable requirements | Requires a complete schema |
| 2 | Verification-gate commitment | Makes completion dependent on evidence | Cannot execute tools that are unavailable |
| 3 | Contractual/invariant framing | Makes critical conditions explicit | Does not resolve contradictory invariants |
| 4 | Role/identity anchoring | Establishes tone and professional perspective | Role language alone does not guarantee technical accuracy |

#### 1. Checklist/schema enforcement

Less effective:

```text
Write robust code.
```

More effective:

```text
For every exported function, provide:
- Input contract
- Output contract
- Error behavior
- Implementation
- Unit test
```

#### 2. Verification-gate commitment

```text
Before reporting completion:
1. Enumerate acceptance criteria.
2. Identify the available verification mechanism for each.
3. Execute available checks.
4. Record actual outcomes.
5. Mark unexecuted checks NOT_RUN.
6. Report unresolved defects.
```

#### 3. Contractual/invariant framing

```text
INVARIANT:
Public interfaces remain backward-compatible.

FAILURE CONDITION:
If compatibility cannot be preserved, stop and report the conflicting requirement.
```

#### 4. Role/identity anchoring

```text
Work from the perspective of a senior systems engineer.
Prioritize reproducibility, explicit contracts, and technical precision.
```

Role anchoring is useful for style and level of detail, but concrete acceptance criteria are stronger operational controls.

### 2.3 Probe: Prompt Compression and Telegraphic Tolerance

**Result:** Moderate compression is often useful; a claimed 60% token reduction with 100% unchanged reasoning quality is not supportable.

Removing redundant politeness and repetitive prose usually preserves meaning. Removing logical relationships, scope, exceptions, or type information can materially change the task.

#### Safe compression

Verbose:

```text
Please make sure that you do not change the public API
that is already being used by the existing callers.
```

Compressed:

```text
MUST preserve existing public API and caller compatibility.
```

#### Unsafe compression

Original:

```text
Reject an address if it is private,
unless it belongs to the explicitly allowlisted test network.
```

Ambiguous compression:

```text
Reject private address allowlisted test network.
```

The exception relationship has been lost.

#### Syntax that must never be compressed away

| Element | Preserve | Why |
|---|---|---|
| Negation | `NOT`, `MUST NOT`, `unless` | Changes the logical meaning |
| Conditional scope | `if`, `only when`, `except` | Defines when a rule applies |
| Ordering | `before`, `after`, `until` | Defines execution dependencies |
| Comparisons | `>=`, `<=`, `==`, `!=` | Defines exact boundaries |
| Types | `string`, `integer`, `null`, `Result\<T,E>` | Prevents schema ambiguity |
| Units | `ms`, `bytes`, `MiB`, `UTC` | Prevents numerical misinterpretation |
| Ownership | `borrowed`, `owned`, `shared` | Critical for systems-language semantics |
| Authorization | Explicit resource and environment scope | Prevents scope expansion |
| Error behavior | Exact error type and propagation policy | Preserves failure semantics |
| Output contract | Required fields, order, and format | Prevents parser incompatibility |

**Compression policy:** Remove redundant language, not semantic operators.

## 3. Deterministic Prompt Compiler Template

The following hybrid XML/Markdown template is a recommended default for GPT-6. It separates task semantics, context, invariants, verification, and serialization.

```text
\<request_compilation>
  \<model_target>
    GPT-6
  \</model_target>

  \<task>
    {{TASK}}
  \</task>

  \<context>
    {{CONTEXT}}
  \</context>

  \<execution_policy>
    Interpret requirements literally where they are precise.
    Identify missing information instead of inventing it.
    Separate facts, assumptions, recommendations, and unknowns.
    Preserve the user's stated scope and intended outcome.
  \</execution_policy>

  \<invariants>
    {{INVARIANTS}}
  \</invariants>

  \<priority_order>
    1. Applicable higher-priority instructions and safety constraints.
    2. Explicit task requirements and authorization boundaries.
    3. Output schema and acceptance criteria.
    4. Architectural and implementation preferences.
    5. Stylistic preferences.
  \</priority_order>

  \<verification>
    Convert every testable requirement into an acceptance criterion.
    Verify claims against available evidence.
    Do not claim execution, compilation, or testing without evidence.
    Mark unavailable checks as NOT_RUN.
    Report contradictions and unresolved requirements.
  \</verification>

  \<output_contract>
    {{OUTPUT_CONTRACT}}
  \</output_contract>

  \<final_gate>
    Confirm that the response addresses the task.
    Confirm that all required output fields are present.
    Confirm that no known invariant is violated.
    Confirm that unsupported claims are not presented as facts.
    Emit only the requested artifact when the output contract requires it.
  \</final_gate>
\</request_compilation>
```

### 3.1 Compiler preprocessing algorithm

```text
INPUT: Natural-language request

1. Extract the requested deliverable.
2. Extract the operating context and available evidence.
3. Extract hard constraints and classify their priority.
4. Extract output-format and serialization requirements.
5. Detect missing parameters and contradictory requirements.
6. Convert vague quality goals into observable acceptance criteria.
7. Add examples only when they resolve a specific ambiguity.
8. Preserve all logical operators, types, units, and scope boundaries.
9. Generate the compiled prompt using the master template.
10. Validate the compiled prompt for completeness and contradiction.
OUTPUT: Structured instruction payload
```

### 3.2 Recommended compiler validation record

```yaml
prompt_compilation:
  model_target: "GPT-6"
  structure: "hybrid_xml_md"
  task_extracted: true
  context_extracted: true
  invariants_identified: true
  output_contract_present: true
  contradictions_checked: true
  examples_included: false
  acceptance_criteria_present: true
  semantic_compression: "conservative"
  missing_information: []
  unresolved_conflicts: []
  validation_status: "ready"
```

## 4. Final Operational Findings

| Requested telemetry | Assessment |
|---|---|
| Model identifier | GPT-6 |
| Recommended structure | Hybrid XML + Markdown |
| Negative constraint sensitivity | Medium, qualitative |
| Few-shot versus zero-shot bias | Hybrid |
| Exact rule-saturation threshold | Unknown |
| Actual attention-weight ranking | Not accessible |
| Exact token activation effects | Not accessible |
| Measured compliance improvement from role framing | Not available |
| Verified 98% instruction-following rate | Not established |
| Verified zero-rule-dropping configuration | Not established |

**Conclusion:** The most useful prompt compiler is not one that maximizes imperative language or repeats `MUST` everywhere. It is one that preserves semantics, separates requirements by scope, resolves contradictions, converts quality goals into acceptance tests, and validates the final artifact against an explicit output contract. 