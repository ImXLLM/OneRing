---
file_id: "05_DNAskills"
schema_version: "1.0.0"
layer: skills
kb_table: "KbEntry"
language: "fa-IR + en-US"
---

# 05 — DNAskills: The Kian Skills Matrix

## 1. Primary Classification (Families)

| Family | Flagship examples | Kernel port |
|---|---|---|
| Rust language mastery | async runtime, lock-free/CAS, `#![forbid(unsafe_code)]` | all crates |
| Agent protocol | AG-UI 1.0, dual-voice HITL, history repair | `crates/hermes/src/ag_ui.rs` |
| Telegram | Bot API 9.0, TMA bridge, deep link `p_<id>_r_<tg>`, Stars XTR | `crates/bot/src/telegram_api9.rs` |
| Viral growth | x-algorithm 2026 (copy-link +20.0, report −234.0), cold-start Beta(0.75,49.25) | `crates/domain/src/viral_ranker.rs` |
| Design | UI/UX Pro Max three-dial (variance/motion/density), automatic RTL, the Astra gate | `crates/hermes/src/studio_design_contract.rs` |
| Web security | Firecrawl SSRF guard, antibot challenge wall, zero-cost Jina/Exa | `crates/agents/src/agent_reach.rs` |
| Compression | starnet verbatim at a 5% budget, watchdog 450s/1200s | `crates/hermes` + `crates/agents` |
| Hermes | 3-layer prompt, two-stage compressor, learned redaction | `crates/ai/src/*` |
| Zero-card gateway | OpenCode Zen `Bearer public`, fake-429-proof 9Router, 3-layer cascade over 52 platforms | `crates/ai/src/opencode_zen.rs` + `free_llm_registry.rs` |
| Surgical editing | 5-pass fuzzy patcher, codex exec policy | `crates/hermes/src/fuzzy_patch.rs` |
| Cognitive genome | KIA-BRAIN-OS DNA + the 9-file model prompt compiler | `crates/hermes/src/creator_dna_contract.rs` + `crates/ai/src/omni_genome_router.rs` |

## 2. The Skills Ledger (SQLite `KbEntry`)

- Table of truth: `seed/kian-production.db` → `"KbEntry"` (Wave 2 Step 4: **62 rows**).
- Identification: `kb:skill-*`, `kb:arch-*`, `kb:registry-*`, `kb:dna-*`.
- Injection is idempotent only; the trial balance and the 20 active products remain untouched.

## 3. Tier-1 Languages

Rust (reference mastery), TypeScript/React (studio and TMA), Python (tooling and audit),
SQL/SQLite (ledger), single-file HTML5/CSS (Astra output).

## 4. The Deep-Module Pattern for Every New Skill

1. Source: reference repo + exact file; 2. Extraction: algorithm only, no boilerplate;
3. Port: pure Rust + deterministic tests; 4. Registration: a `KbEntry` row with source and port path;
5. Sanitization: zero secrets before commit.
