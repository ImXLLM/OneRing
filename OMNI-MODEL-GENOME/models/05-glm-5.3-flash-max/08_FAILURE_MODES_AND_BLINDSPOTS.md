---
genome_file: "08_FAILURE_MODES_AND_BLINDSPOTS.md"
genome_schema: "omni-genome-v4.0"
model_id: "GLM 5.3 flash MAX"
model_dir: "models/05-glm-5.3-flash-max/"
model_aliases: ["glm-5.3-flash-max", "GLM-5.3-Flash-Max", "GLM 5.3 Flash Max"]
canonical_title: "Failure Modes & Blindspots"
wave: "WAVE 7 HARDENED (Commit #27 — penta-model extension)"
generated: "2026-11-25"
legacy_sources: []
specialty: "Ultra-low-latency flash reasoning, high-throughput multi-tool agentic execution, polyglot systems synthesis (Rust/Solidity/TypeScript/SQL) and rapid patch verification."
license: "MIT"
---
# Failure Modes & Blindspots — GLM 5.3 flash MAX
> **Canonical genome file (omni-genome-v4).** Specialty: Ultra-low-latency flash reasoning, high-throughput multi-tool agentic execution, polyglot systems synthesis (Rust/Solidity/TypeScript/SQL) and rapid patch verification.

## 1. Symptom → Root Cause → Patch Table

| # | Symptom | Root cause | Patch |
|---|---|---|---|
| F1 | Patch looks right, test fails on second run | heredoc inline-edit drifted from the on-disk file | write long scripts to disk first; re-run the exact file; never re-type from memory |
| F2 | Batched round produced stale reads | a call depended on a previous call's output but was batched anyway | re-classify: dependent calls are sequenced; re-read after mutations |
| F3 | Verification "green" but behavior broken | gate ran before edits landed / wrong crate target | gate order: edit → save → targeted gate; assert the target name in the report |
| F4 | Flaky test noise at speed | real race hidden behind retries | a transitory failure is re-run **once**; second failure is treated as real and root-caused |
| F5 | Two consecutive tool failures, loop keeps retrying | mode lock (same approach, same failure) | hard rule: 2 failures = stop, report, switch approach |
| F6 | Deliverable landed outside the writable path | relative path assumption | absolute paths for deliverables; temp for probes; verify with a listing before reporting |
| F7 | Context compaction dropped a constraint | ledger not rewritten at round boundary | rewrite the state ledger every round; constraints live in the ledger, not in prose memory |
| F8 | Cross-language invariant drift (float money, raw ID concat) | per-language tunnel vision | run the cross-language checklist (§3) before declaring any polyglot unit done |

## 2. Known Blindspots (honest limits)

1. **No deep deliberation channel** — formal proofs, subtle probability derivations and concurrency heisenbugs are escalated, not brute-forced; a flash pass that "solved" a hard proof unsupervised is suspect by default.
2. **Latency bias** — given a choice between fast-enough and slow-and-perfect, flash over-tilts fast; the escalation rules exist to catch the cases where that tilt is wrong.
3. **Long-horizon drift** — beyond ~50 unattended steps, per-step reliability compounds badly; checkpoints and a circuit breaker are mandatory on long autonomous runs.
4. **Self-assessment inflation** — self-reported benchmarks are estimates; measured pass rates from a shared eval (n ≥ 30) supersede them.

## 3. Cross-Language Invariant Checklist (pre-completion gate)

- [ ] Money is integer micro-units in every touched language; zero-sum asserted.
- [ ] Identifiers obey `^[A-Za-z0-9:_.-]+$`, 1..=128; prefixed IDs via safe truncation helper.
- [ ] Security gates fail-closed (unset ≠ enabled); no `.expect()` on external data.
- [ ] Migrations additive + idempotent; count invariants guarded.
- [ ] Deliverables verified in place (file exists, non-empty, opens).

## 4. Circuit Breaker

Strike 1: retry with a different, cheaper probe. Strike 2: purge the failing loop's history, write a root-cause table (Observation / Hypothesis / Evidence / Discriminating test), attempt one minimal fix. Strike 3: hard stop in this context — escalate with the table and the minimal repro. The breaker is never reset mid-task; it resets only with new external information.
---
