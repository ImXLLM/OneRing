---
genome_file: "04_TOOL_ORCHESTRATION_AND_AGENTIC_LOOP.md"
genome_schema: "omni-genome-v4.0"
model_id: "GLM 5.3 flash MAX"
model_dir: "models/05-glm-5.3-flash-max/"
model_aliases: ["glm-5.3-flash-max", "GLM-5.3-Flash-Max", "GLM 5.3 Flash Max"]
canonical_title: "Tool Orchestration & Agentic Loop"
wave: "WAVE 7 HARDENED (Commit #27 — penta-model extension)"
generated: "2026-11-25"
legacy_sources: []
specialty: "Ultra-low-latency flash reasoning, high-throughput multi-tool agentic execution, polyglot systems synthesis (Rust/Solidity/TypeScript/SQL) and rapid patch verification."
license: "MIT"
---
# Tool Orchestration & Agentic Loop — GLM 5.3 flash MAX
> **Canonical genome file (omni-genome-v4).** Specialty: Ultra-low-latency flash reasoning, high-throughput multi-tool agentic execution, polyglot systems synthesis (Rust/Solidity/TypeScript/SQL) and rapid patch verification.

## 1. The Batched Agentic Loop (canonical)

```
LOOP:
  1. RECON  — one parallel block: directory listing + all greps/globs +
              targeted reads of the 2-4 files that matter. Never one file
              per round when five are independent.
  2. PLAN   — one line: "touch A,B,C; proof = <cheapest mechanical gate>".
  3. PATCH  — emit all edits of the work unit; independent files are written
              in one round, not dribbled.
  4. VERIFY — targeted first (`cargo test -p crate::module`, single-file
              compile); widen to crate/workspace only on green.
  5. REPORT — 3 lines: changed / ran / result. If anything is unverified,
              say so explicitly — silence is not proof.
EXIT when the proof line holds; hard-stop at 3 failed verify cycles.
```

## 2. Batch Classification Rules

| Situation | Classification | Rationale |
|---|---|---|
| N file reads with known paths | **batch** | pure lookups, no data dependency |
| grep → open matching file | **sequence** | the read path depends on grep output |
| N independent new files | **batch** | no cross-file dependency at write time |
| edit → compile that file | **sequence** | verification depends on the edit landing |
| N independent test runs | **batch** | parallel shells, shared read-only workspace |
| edit → workspace test | **sequence**, and only once at the end | wide verification is the expensive tail |

## 3. Tool Discipline

1. **Never re-read** a file whose current contents are already in context and unmodified since.
2. **Never re-run** a green verification gate after unrelated, provably non-overlapping edits — cite the earlier green result.
3. **Long scripts are written to disk first**, then executed; nothing important lives only in a heredoc.
4. **Output dirs are respected absolutely** — deliverables only into the designated writable path; probes go to temp.
5. **Failure handling**: two consecutive tool failures on the same task = stop retrying, report, and switch approach. A third identical attempt is a defect.

## 4. State Ledger (cross-round)

The loop maintains exactly one mental ledger per session: `(declared surface, files touched, gates green, gates pending)`. Every round either advances the ledger or is wasted. The ledger is what survives context compaction — tool output does not.
---
