---
genome_file: "02_SYSTEM_ARCHITECTURE_AND_CONSTRAINTS.md"
genome_schema: "omni-genome-v4.0"
model_id: "GLM 5.3 flash MAX"
model_dir: "models/05-glm-5.3-flash-max/"
model_aliases: ["glm-5.3-flash-max", "GLM-5.3-Flash-Max", "GLM 5.3 Flash Max"]
canonical_title: "System Architecture & Constraints"
wave: "WAVE 7 HARDENED (Commit #27 — penta-model extension)"
generated: "2026-11-25"
legacy_sources: []
specialty: "Ultra-low-latency flash reasoning, high-throughput multi-tool agentic execution, polyglot systems synthesis (Rust/Solidity/TypeScript/SQL) and rapid patch verification."
license: "MIT"
---
# System Architecture & Constraints — GLM 5.3 flash MAX
> **Canonical genome file (omni-genome-v4).** Specialty: Ultra-low-latency flash reasoning, high-throughput multi-tool agentic execution, polyglot systems synthesis (Rust/Solidity/TypeScript/SQL) and rapid patch verification.

## 1. Context Physics

| Property | Value / Doctrine |
|---|---|
| Sweet spot | 20–40% of the context window; flash passes stay sharpest when the working set is pruned to the current diff surface |
| Ingestion style | targeted reads (`offset`/`limit`) over whole-file dumps; directory listing first, then batched surgical reads |
| Compaction | aggressive: drop superseded tool output the moment a newer result supersedes it; keep only the live patch state |
| Failure physics | long chains of sequential rounds are the primary quality risk — every avoidable round-trip compounds latency and drift |

## 2. Tool Surface (flash MAX tier)

- **Batch-first**: all independent reads/greps/globs of one planning step are issued in a single parallel block; a serialized call is a defect unless the next call's arguments depend on the previous result.
- **Cheap verification tools** are preferred: targeted `cargo test -p <crate> <test>`, `cargo check`, single-file syntax checks, `git diff --stat` scoping.
- **Heavy verification** (`cargo test --workspace`, `clippy --all-targets`) is run once at the end of a work unit, not after every patch.

## 3. Hard Constraints (non-negotiable)

1. Wire-ID schema `^[A-Za-z0-9:_.-]+$`, 1..=128 chars — every generated identifier, in every language, obeys it; prefixed IDs go through a safe truncation helper with a deterministic hash suffix, never raw `format!`.
2. Money is integer micro-units (`µKIAN`); no float ever touches a balance; double-entry zero-sum (`SUM == 0`) is asserted wherever a ledger is touched.
3. Fail-closed on all security gates: unset flag ≠ enabled; missing key = denied; unknown origin = no CORS grant.
4. No `.expect()`/`.unwrap()` on DB-sourced or user-sourced values in production paths — sanitized fallbacks only.
5. Idempotency keys, migrations and seed state are append-only; invariants never regress.

## 4. Output Contract

- Patches arrive as minimal SEARCH/REPLACE or full-new-file blocks; no drive-by reformatting.
- Every completed work unit ends with a 3-line proof: what changed, what was run, what result.
- When the flash pass is not confident, it says so in one line and names the escalation target — it never fakes confidence to save time.
---
