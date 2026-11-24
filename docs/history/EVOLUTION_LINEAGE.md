# EVOLUTION_LINEAGE — Full Evolutionary History of KIAN v2 (`تاریخچهٔ پیشرفت`)

> **Doctrine**: [`KIA-BRAIN-OS/11_UNCHAINED_KIA_DNA.md`](../../KIA-BRAIN-OS/11_UNCHAINED_KIA_DNA.md) — Law #3:
> *Evolutionary Lineage Preservation with Latest-Version Execution Weight* (`حفظ تاریخچهٔ تکامل + وزن اجرایی روی آخرین نسخه`).
> Every commit, vault and milestone below is **immutable proof-of-work lineage** preserved inside Git
> and the `"KbEntry"` knowledge vault, while **100% of runtime execution weight** is routed
> exclusively to the latest, fastest, hardened release (`HEAD`).

---

## How the lineage is preserved vs. executed

| Layer | History (preserved, read-only lineage) | Execution weight (latest version only) |
| :--- | :--- | :--- |
| **Git** | Full `git log` from day zero — every commit of every wave stays in the repository forever | `HEAD` on `main` is the only pushed/deployed ref |
| **Docs & vaults** | `docs/vault/` (19 mounted gem collections), `docs/rea/` (per-step gate reports), `docs/dna/` (gate-readable DNA mirror), `docs/history/` (this file) | — |
| **Knowledge vault (`"KbEntry"`)** | Every legacy row kept, zero deletions; legacy entries remain queryable as lineage | Latest v2 sovereign genome/DNA/routing entries carry `[EXEC_WEIGHT: 1.00 \| ACTIVE_PRIMARY]` |
| **Docker / Railway build** | — | `.dockerignore` strips all historical/vault/frontend directories from the build context so only the runtime kernel (`Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `crates/`, `seed/kian-production.db`, `start-railway.sh`) is uploaded and compiled — fastest possible deploy |
| **Model routing** | Historical model genomes (`gpt-6-standard-free`, `gemini-4-aragon-high`) and older gateway lanes stay documented | `OMNI-MODEL-GENOME` router + keyless OpenCode Zen (`beam-free`, 1,048,576-token context) = primary zero-cost lane |

---

## Chronology (git-verified, branch `main`)

### Wave 0 — Foundation & National Kernel Core

| Commit | Milestone |
| :--- | :--- |
| `7c699ad` | **Initial commit** — project genesis |
| `2026ee7` | KIAN v2 launch-ready: Rust compile/test fixes, serde/Zod/SDK contract sync, world-class UX polish, zero build errors |
| `aeb8c23` | KIAN v2 turnkey: 1-click browser personas + demo faucet (`KIAN_DEV_MODE`), Founder Command Center, charter mint cap, live E2E smoke 21/21 |
| `eb82694` | **Railway migration**: dropagent 30-day community (466 users, 247 referrals, sanitized genesis 58,700.2 KIAN, 108 released escrows) → deterministic idempotent ETL into `kian-production.db`, migration 0006, Railway `Dockerfile`/`start-railway.sh` with seed-on-first-boot, live E2E 39/39, workspace tests **282 green** |

### Wave 1 — AI Gateway Forges & Hidden-Gems Vault

| Commit | Milestone |
| :--- | :--- |
| `14c4e36` | AI gateway single-router aliases (`KIAN_AI_BASE_URL/API_KEY/MODEL` + fallback), 9router 91-lane live benchmark, deterministic local HTML synthesizer fallback |
| `b45b647` | OpenRouter live `:free` lanes — kernel-faithful no-`max_tokens` chat turns, attribution headers, winner config slot1 `openrouter/free` (9/9 contract) |
| `b54b14c` | post-reset proof harness: zero-model `/auth/key` polling until daily counter reset |
| `5bd3100` | post-reset proof zip subtree layout fix |
| `013fae2` | ShopifyHermes `brain.py` pool mechanics port: cross-product multi-model/multi-key pool, last-good stickiness, quality gate, least-bad stash |
| `7771aeb` | **Hidden Gems Vault**: 9-folder surgical merge into KIAN v2 |
| `a077ade` | Vault injection + smoke scripts (`VAULT_INDEX_FA.md` tooling) |

### Wave 2 — Sovereign Kernel Hardening (Steps 1..6)

| Commit | Milestone |
| :--- | :--- |
| `0ccbb28` | **Step 1**: OpenBot history repair, AG-UI 1.0 protocol, Dual-Voice HITL, Studio HTML guard |
| `011fffd` | **Step 2**: Hermes 3-layer prompt builder, two-phase compressor, learned secret redaction, tool parser, bot loop guard |
| `040d42a` | **Step 3**: x-algorithm 2026 viral ranker, UI/UX Pro Max dials, Astra studio gate, starnet verbatim compaction, Agent-Reach zero-cost web, **guardian 7/7 activation**, Telegram Bot API 9 TMA adapters |
| `b7352f5` | Step 4 working checkpoint (job-id signed commit) |
| `443b075` | **Step 4 complete**: zero-card LLM gateway, 5-pass surgical fuzzy patcher, sandbox exec-policy, **`KIA-BRAIN-OS` & `OMNI-MODEL-GENOME`** mounted (62 KB skills) |
| `9875a9f` | **Step 5**: 261-skill vault injection, `gemini-4-aragon-high` genome, firecrawl SSRF & CAS guards — 516 tests green, `"KbEntry"` 323 |
| `1e301f8` | **Step 6**: Mairon DNA REA agent, live product REA gate (two-key publish interlock), 5-star Bayesian review engine, final REA audit **92/100 PASS** — 530 tests green, `"KbEntry"` 325 |
| `1d30894` | Repo hygiene: untrack sandbox-local `.env` & `download/` artifacts ahead of GitHub publication |
| `115c8b1` | World-class English README with verified Wave 2 metrics |

### Release — `0xKIA` definitive MIT cut

| Milestone |
| :--- |
| DNA #11 [`11_UNCHAINED_KIA_DNA`](../../KIA-BRAIN-OS/11_UNCHAINED_KIA_DNA.md) injected into `"KbEntry"` → **326 rows** |
| `docs/history/EVOLUTION_LINEAGE.md` (this file) committed as the lineage record |
| Execution-weight tagging: latest v2 kernel/genome entries marked `[EXEC_WEIGHT: 1.00 \| ACTIVE_PRIMARY]` |
| Environment variables strictly reordered by operational priority (`P0` → `P5`) in `.env.example` & `railway.env.example` |
| `.dockerignore` hardened: history stays in Git, build context carries runtime weight only |
| `README.md` + `README.fa.md` bilingual mirror with 100% forensically verified numbers |


---

## Wave 3 — Sovereign Trinity Studio Pipeline (`خط لولهٔ سه‌کالبدی حاکم`)

| Commit | Milestone |
| :--- | :--- |
| *(this release)* | **Wave 3**: the signature architectural centerpiece — `Alpha → Command Center [OpenBot 3×9 DNA] → Mairon [IQ 150 / EQ 145]` wired end-to-end: `trinity_pipeline.rs` engine in `kian-hermes` (Alpha slot extraction & deterministic `MasterPromptSpec`/`ProjectBlueprint` compilation, OpenBot dynamic sub-agent factory with 3-layer cache-safe prompts + 9-fold stochastic genomes + SplitMix64 alleles, self-healing repair loop `execute_command_center_with_mairon_loop` max 3 attempts to the REA `>= 85` gate), Mairon overclock DNA constants (`IQ=150/EQ=145/Lookahead=8/ToM=3/Ambition=0.99`) locked in `kian-domain`, live endpoints `POST /builder/trinity/alpha`, `POST /builder/trinity/orchestrate`, `POST /market/products/{id}/mairon-confirm`, `GET/POST /market/products/{id}/reviews`, migration `0007_trinity_reviews.sql` (mairon stamps + `"ProductReview"` one-vote lock) — **560 tests green, `"KbEntry"` 327, trial balance `0`** |

### Wave 3 — DNA & release milestones

| Milestone |
| :--- |
| DNA #12 [`12_TRINITY_AVATAR_ENGINE.md`](../../KIA-BRAIN-OS/12_TRINITY_AVATAR_ENGINE.md) created + mirrored to `docs/dna/kia-brain-os/` |
| DNA #12 injected into `"KbEntry"` as `kb:dna-12-trinity-avatar-engine` → **327 rows**, tagged `[EXEC_WEIGHT: 1.00 \| ACTIVE_PRIMARY]` |
| `KIA-BRAIN-OS` is now the **12-file** creator brain (`01_DNAme.md` … `12_TRINITY_AVATAR_ENGINE.md`) |
| Trinity Avatars: `ALPHA (136/122 · brake 0.04 · temp 0.88)` → `COMMAND CENTER (139/129 · strictness 0.96 · 3×9 OpenBot DNA)` → `MAIRON-150 (150/145 · lookahead 8 · ToM 3 · ambition 0.99)` |
| Two-key marketplace gate live: `⚒️ Mairon REA Certified: {score}/100 (Forge-Grade)` + 5-star Bayesian `combinedRankScore` |

------

## Growth ledger

| Metric | Wave 0 (Railway migration) | Step 5 | Step 6 | MIT cut | Wave 3 (Trinity) |
| :--- | :---: | :---: | :---: | :---: | :---: |
| Workspace tests | 282 | 516 | **530** | 530 | **560** |
| `"KbEntry"` rows | 62 | 323 | 325 | **326** | **327** |
| Guardian fleet | — | 7/7 | 7/7 | 7/7 | 7/7 |
| Mairon REA verdict | — | step-5 gate | **92/100 PASS** | 92/100 PASS | **92/100 PASS** + live Trinity forge gate (85+) |
| Trial balance (`SUM(Posting.amount)`) | 0 | 0 | 0 | **0** | **0** |

*The trial balance over 58,700,200,000 micro-KIAN has been exactly zero at every milestone — the
ledger has never moved out of balance.*
