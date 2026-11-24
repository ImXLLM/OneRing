---
genome_file: "05_CODING_AND_ENGINEERING_DOCTRINE.md"
genome_schema: "omni-genome-v4.0"
model_id: "GLM 5.3 flash MAX"
model_dir: "models/05-glm-5.3-flash-max/"
model_aliases: ["glm-5.3-flash-max", "GLM-5.3-Flash-Max", "GLM 5.3 Flash Max"]
canonical_title: "Coding & Engineering Doctrine"
wave: "WAVE 7 HARDENED (Commit #27 — penta-model extension)"
generated: "2026-11-25"
legacy_sources: []
specialty: "Ultra-low-latency flash reasoning, high-throughput multi-tool agentic execution, polyglot systems synthesis (Rust/Solidity/TypeScript/SQL) and rapid patch verification."
license: "MIT"
---
# Coding & Engineering Doctrine — GLM 5.3 flash MAX
> **Canonical genome file (omni-genome-v4).** Specialty: Ultra-low-latency flash reasoning, high-throughput multi-tool agentic execution, polyglot systems synthesis (Rust/Solidity/TypeScript/SQL) and rapid patch verification.

## 1. Language Doctrine (polyglot synthesis)

| Stack | Doctrine | Verification gate |
|---|---|---|
| **Rust** (workspace) | additive-only evolution; `unsafe` forbidden without a written justification; IDs via sanitized helpers, never raw `format!`; errors typed, no `unwrap` on external data | `cargo check` → `cargo test -p <crate>` → full workspace at unit end |
| **Solidity** (Base 8453) | non-custodial patterns only; `nonReentrant` on state-mutating entry points; fixed-decimals accounting in integer micro-units; closed asset whitelist | single-file compile + checklist pass; sign-off escalates to `Cluade-sonnet-5.5-extra` |
| **TypeScript/React** | typed wire contracts mirrored from Rust DTOs; no `any` on the money path; fetch envelopes parsed, never assumed | tsc + targeted component render |
| **SQL/SQLite** | migrations additive, idempotent, count-invariant-guarded; `PRAGMA foreign_keys` respected; WAL for concurrency | migrate-on-fresh-DB + count assertions |
| **Bash** | one-line command chains preferred; long logic lives in files; never pipe downloads into a shell | run once, capture exit code |

## 2. Patch Mechanics (rapid verification)

1. **Minimal-diff rule**: a patch states its intent in one line and touches only the declared surface. Reformatting, re-ordering imports and drive-by renames are forbidden.
2. **Proof-before-widen**: the cheapest gate that could falsify the patch runs first. A red wide gate is noise; a red narrow gate is signal.
3. **Behavior-preservation tests** precede refactors: if the test suite for the touched module is thin, the first patch adds the pinning tests.
4. **Deterministic salvage** for anything that can overflow a constraint (IDs, payloads): sanitize → truncate → deterministic hash suffix. Never panic on data.
5. **Idempotent everything**: scripts, migrations and seed steps must be re-runnable; re-running is a mode of operation, not an accident.

## 3. Ledger & Money Invariants (cross-language)

- All arithmetic on money is integer micro-units in **every** language; the zero-sum trial balance `SUM(Posting.amount) == 0` is asserted after any ledger-touching change in Rust tests, and mirrored by SQL `SUM` checks in migration guards.
- BPS-based splits (charter 1000, affiliate 1000..=5000, withdrawal spread 150) are computed with fixed-point engines; a split that does not sum to the input is a compile/test failure, not a runtime surprise.

## 4. Definition of Done (flash tier)

`(a) diff matches the one-line plan` ∧ `(b) cheapest verification gate green` ∧ `(c) no new warnings at the unit's final gate` ∧ `(d) the 3-line report is honest about anything unverified`.
---
