---
file_id: "02_DNAwork"
schema_version: "1.0.0"
layer: architecture
product: "KIAN v2"
valuation: "COCOMO II"
language: "fa-IR + en-US"
---

# 02 — DNAwork: The KIAN v2 Working Architecture and the Ledger's Iron Rules

## 1. Official Valuation (COCOMO II — audit of 2026-11-24)

| Item | Value |
|---|---|
| Standalone KIAN v2 value | **$523,750** |
| Combined value with the 60-project KiaAgentX ecosystem | **~$1.18M** |
| Method | COCOMO II over the real Rust kernel lines + green tests |

## 2. Kernel Architecture — 9 Rust Crates

| Crate | Responsibility |
|---|---|
| `kian-domain` | Domain models, the viral x-algorithm 2026 ranker |
| `kian-contracts` | API contracts — versioning and validation |
| `kian-db` | The SQLite layer (WAL, double-entry transactions) |
| `kian-wallet` | The double-entry ledger (Posting/Tx), TON, USDT, Stars |
| `kian-nation` | The digital township: citizens, referrals, parliament (Bill/BillVote) |
| `kian-ai` | The AI gateway: three fallback slots, a hard budget, redaction, the model genome |
| `kian-hermes` | The business brain: the shield, skills, 7-layer memory, creator DNA |
| `kian-agents` | The 7-role fleet, Agent-Reach, a stop watchdog |
| `kian-bot` + `kian-server` | Bot API 9.0 / TMA + HTTP server |

## 3. Production Database Iron Rules (Production SQLite Invariants)

- Citizens: **466** (the `User` table) — never tampered with by injections.
- Referral lines: **247** (`ReferralLineage`) — the lineage dependency mechanism is preserved.
- Orders/escrow: **108** (`Escrow`) — no order cycle exists without escrow.
- Active products: **20** (`Product.active=1`) — changed only by a real transaction.
- Trial balance: `SUM(Posting.amount) = 0` across **58,700,200,000 micro-KIAN**
  — the double-entry ledger is always balanced; every skill injection must keep this figure intact.
- Identifiers: `KbEntry` uses the `kb:` prefix and injections are **idempotent**
  (title+body comparison, duplicates rejected).

## 4. The Agent Fleet — 7/7 Active

`scout`, `builder`, `broker`, `envoy`, `judge`, `sage`, **`guardian`** (the security/
verification handler). No role stays `Registered`; a role without a real handler is
filed, not hidden.

## 5. Iron Rules of Work

1. **No secrets in git:** no key/token ever reaches the repository; stores are
   sanitized with `DUMMY-TEST-FIXTURE`.
2. **No number without execution:** every functional claim must carry real output
   (`cargo test`/`cargo clippy`/live JSON).
3. **`#![forbid(unsafe_code)]`** on every crate; clippy clean with `-D warnings`.
4. **Isolated cargo:** `CARGO_TARGET_DIR=/tmp/my-project/target CARGO_BUILD_JOBS=2`.
5. **Deep Module > boilerplate:** every module has a small interface and a deep implementation.
