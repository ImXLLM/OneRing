---
file_id: "04_DNAharness"
schema_version: "1.0.0"
layer: environment
runtime: "Z.ai + Arena + Railway"
language: "fa-IR + en-US"
---

# 04 — DNAharness: The Physics of the Execution Environment

## 1. Environment Map

| Environment | Role | Rules |
|---|---|---|
| **Command Center** (Local) | Command, vault acceptance, architecture decisions | Owner: Kia; keys live here only |
| **Z.ai** (Sandbox) | Rust/DB builder — code, tests, database injection | `CARGO_TARGET_DIR=/tmp/my-project/target`, `CARGO_BUILD_JOBS=2` |
| **Arena** | Forensic harvester of models — autopsy of frontier model behavior | Output → `OMNI-MODEL-GENOME/models/<id>/` |
| **Railway** | Production runtime of kian-server + 9router | Environment variables; zero secrets in git |

## 2. Hard Rules of Z.ai

- Deliverable files are only visible under `/home/z/my-project/download/`.
- Uploads land in `/home/z/my-project/upload/` — input only, never modified in place.
- Long scripts are stored in `/home/z/my-project/scripts/` and executed from file.
- The Kian project: `/home/z/my-project/kian-launch/kian/` — the primary git repository.

## 3. The Zero-Secret Git Gate

Before every commit:
1. `git diff --cached` is scanned: `sk-`, `ghp_`, `eyJ`, `AKIA`, `xox`,
   bot tokens (`\d+:[A-Za-z0-9_-]{30,}`), `Bearer <non-public>` → zero allowed;
2. the only token permitted in documentation: `Bearer public` (the anonymous OpenCode Zen sentinel);
3. the replacement text is always `DUMMY-TEST-FIXTURE`.

## 4. The Vault Cycle

`upload/*.zip` → unpacked into `/tmp/stepN_gems/` → curation (<100KB/file) →
mounted under `docs/vault/<name>/` → sanitized → Rust/SQLite kernel → commit.

## 5. Failure Behavior

- Two consecutive tool failures → halt and report honestly; never bypass policy.
- Long cargo timeouts → still run with the same env; never execute without
  `CARGO_TARGET_DIR` (job-isolated frugality).
