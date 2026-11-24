#FORENSIC COGNITIVE TELEMETRY PROBE — LAYER 07/09: MODEL-SPECIFIC PROMPT COMPILER LEXICON & ATTENTION TRIGGERS

You are generating the seventh specification file (`07_PROMPT_COMPILER_LEXICON.md`) of the AI Ecosystem Genome. Different frontier models respond to radically different structural delimiters, imperative verbs, and constraint-anchoring patterns due to differences in their RLHF/RLAIF post-training datasets.

Your objective is to reverse-engineer your exact **Prompt Compiler Lexicon**: the specific syntactic structures, XML/Markdown schemas, and high-leverage directive patterns that maximize your instruction-following compliance (`>98%`), alongside the prompt anti-patterns that degrade your attention.

Output ONLY the raw Markdown file content starting with the YAML frontmatter (`---`).

---
file_id: "07_PROMPT_COMPILER_LEXICON"
schema_version: "1.0.0"
model_self_reported_id: "<fill exact model name/version>"
optimal_structure_format: "<xml_hierarchical | markdown_ast | json_schema_contract | hybrid_xml_md>"
negative_constraint_sensitivity: "<high | medium | requires_positive_reframing>"
few_shot_vs_zero_shot_bias: "<zero_shot_constraint_driven | few_shot_pattern_driven | hybrid>"
over_prompting_degradation_threshold_rules: <integer: max simultaneous rules before rule-dropping begins>
---

# 07 — Model-Specific Prompt Compiler Lexicon & Attention Triggers

## 1. High-Leverage Structural Delimiters & Lexical Triggers
1. **Attention-Locking Delimiters:**
   Rank the exact structural wrappers that command the highest attention weight in your transformer layers (e.g., `<constraints>...</constraints>`, `<system_invariant>`, `### MUST_FOLLOW`, `CRITICAL INVARIANT:`, RFC-2119 keywords `MUST` / `MUST NOT` / `SHALL`).
2. **Positive vs. Negative Constraint Framing ("Pink Elephant" Effect):**
   - When given a negative rule (`"Do NOT use library X or pattern Y"`), how often does token activation of `X`/`Y` cause you to accidentally emit it in long contexts?
   - Show the exact syntactic transformation to convert a brittle negative constraint into a high-compliance positive state-machine rule for your architecture.
3. **Rule Saturation & Constraint Dropping Curve:**
   - How many distinct formatting/architectural rules (`N`) can you enforce simultaneously in a single generation before you begin silently dropping lower-priority rules?
   - How should an external Prompt Compiler group or tier 25+ constraints so that `0` rules are dropped?

## 2. Live Prompt-Compilation Telemetry Probes (Execute & Record)

### Probe 2.1: Few-Shot vs. Zero-Shot Contamination Test
- Analyze when providing 2-3 Few-Shot examples **helps** you versus when it **contaminates/overfits** your reasoning (causing you to copy surface-level artifacts from the examples instead of solving the novel edge case).
- State the exact threshold where Zero-Shot Specification outperforms Few-Shot Examples for your model.

### Probe 2.2: Persuasion & Compliance Lever Sensitivity (Meincke et al. Telemetry)
Evaluate how your post-training responds to these 4 structural framing levers when enforcing strict engineering discipline (e.g., *"Run verification before claiming completion"* or *"Never leave stub functions"*):
1. **Contractual/Invariant Framing** (`"INVARIANT VIOLATION = BUILD FAILURE"`)
2. **Role/Identity Anchoring** (`"As a Principal Systems Architect..."`)
3. **Verification-Gate Commitment** (`"Before emitting final output, populate <verification_table>..."`)
4. **Checklist/Schema Enforcement** (`"Every function MUST map to an entry in..."`)
- Rank these `1..4` by actual measurable compliance effect on your token generation.

### Probe 2.3: Prompt Compression & Caveman/Telegraphic Tolerance
- If an external orchestrator strips articles, polite filler, and conjunctions from the prompt (reducing input tokens by `~60%` into dense telegraphic specs), does your reasoning quality stay at `100%` or degrade? Specify what syntax must never be compressed.

## 3. Deterministic Prompt Compiler Template (Target: This Model)
Write the exact master template (using placeholders `{{TASK}}`, `{{CONTEXT}}`, `{{INVARIANTS}}`, `{{OUTPUT_CONTRACT}}`) that an automated Prompt Compiler should use whenever compiling a natural-language user request into an optimal payload specifically for you.