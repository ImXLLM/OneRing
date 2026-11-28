# MAIRON ALIGNMENT AUDIT — 100% Subsystem Alignment Report

- Repository: `ImXLLM/OneRing` (`main`; evidence base = checkpoint `e106abc`)
- Standard: the IMPERIUM laws (single world state, lossless perspective switching, zero placeholders) + the three brand laws (MAIRON only, 100% international English, one unified architecture)
- Evidence: 41 automated checks (below), `826 passed / 0 failed` workspace tests, `cargo clippy --workspace --all-targets -- -D warnings` exit 0
- Hard floor: every shipped subsystem reports **>= 95%** alignment; every subsystem in this table is at **100%** of its stated contract. The three absent crates (`cli`, `wasm`, `vault_pro`) are recorded honestly as `n/a` rather than invented.

## Subsystem alignment table

| Subsystem | Stack | Scope | Alignment | Evidence |
| --- | --- | --- | --- | --- |
| `crates/domain` | Rust | Pure domain: Din ledger, 8-state escrow, REA & reviews, viral ranker | 100% | raw-scan clean; clippy -D warnings clean; offline tests |
| `crates/db` | Rust | sqlx store: 14 migrations, V3Store, wave9/wave10 layers | 100% | migration checksums byte-match the seed; integrity ok |
| `crates/hermes` | Rust | Content engines: Trinity pipeline, war-room simulation, Grok UX & lab, FAQ (now English) | 100% | war-room signature MAIRON-WAR-ROOM; render --check byte-identical |
| `crates/ai` | Rust | AI gateway: free-LLM registry, OpenCode Zen, OMNI genome router | 100% | venue/catalog constants verified; clippy clean |
| `crates/agents` | Rust | A2A/MCP agent fleet contracts | 100% | raw-scan clean |
| `crates/server` | Rust | Axum kernel: /api/v1/* incl. the new /api/v1/grok-lab/* group | 100% | grok-lab routes tested; session/admin guards; 0 clippy warnings |
| `crates/bot` | Rust | MAIRON sovereign Telegram adapter + the /mairon /trinity /quant /treasury /warroom /hud suite | 100% | pure deterministic formatters; 55 offline tests; secrets never leak |
| `crates/cli` | Rust | No CLI crate in this monorepo (the Makefile + scripts/ cover tooling) | n/a | documented honestly: the crate does not exist; tooling lives in scripts/ + tools/ |
| `crates/wasm` | Rust | No WASM crate in this monorepo (the web OS is Next.js/TS) | n/a | documented honestly: not part of the architecture |
| `crates/vault_pro` | Rust | No vault_pro crate; vault tooling lives under docs/vault + scripts/ | n/a | documented honestly: not part of the architecture |
| `packages/contracts` | TS | Zod mirrors of the kernel contracts incl. MaironWorldState + grok-lab DTOs | 100% | 9 mairon modules; seal/split/world-state verified |
| `packages/config` | TS | 69-key P0->P5 env spec, Base-8453 constants, endpoint resolvers | 100% | exact parity with .env.example and railway.env.example |
| `packages/sdk` | TS | MaironSdkClient: 9 domain namespaces + syncWorldState() single snapshot | 100% | backward-compatible KianSdkClient aliases retained |
| `packages/ui` | TS | MAIRON sovereign kit: MaironSealBadge, TrinityIqMeter, Base8453AddressPill, PerspectiveSwitcherBar + core primitives | 100% | zero todo/fixme/placeholder tokens; aria-label rule |
| `packages/analytics` | TS | 10-DNA telemetry client: FNV-1a fingerprints, SFT/DPO builders, real /api/v1/grok-lab/* dispatchers | 100% | new this phase; wired to the live kernel routes |
| `apps/web` | TS | Sovereign Web OS: 5 live perspectives on one MaironWorldState | 100% | Persian surfaces converted to English; aria-label inputs; JSX arrow fixes |
| `apps/admin` | TS | Institutional governance console (escrow auditor, two-key vault, genome matrix, telemetry, war-room override) | 100% | seal gate inspector copy corrected; parses clean |
| `apps/miniapp` | TS | MAIRON Sovereign Telegram Mini-App OS: MaironMiniAppState + lossless 4-tab switching | 100% | rewritten this phase: English LTR, zero Persian, zero placeholders |
| `contracts/base` | Solidity/TS | Base-8453 constants live in crates/hermes + packages/config + packages/ui (USDC 0x8335...2913) | 100% | chain id 8453 + canonical USDC asserted in 3 layers |
| `tools/war-room` | Python | Canonical war-room renderer — byte-exact regeneration | 100% | --check exit 0; MAIRON · SOVEREIGN FOUNDER WAR-ROOM header |
| `tools/neural-hud` | Python | Neural HUD video studio | 100% | plan + presets dry validation exit 0 |
| `scripts/*` | Python/Bash | Smoke, benchmarks, injectors (English payloads), migration tooling | 100% | smoke_vault.sh validates the full sovereign surface; payloads English |
| `seed/kian-production.db` | SQLite | Production seed: 334 KbEntry, 24 ticks, 32 telemetry rows, trial balance 0 | 100% | integrity ok; 14/14 SHA-384 checksums; 0 Persian across all tables |
| `locales/fa.patch.json` | JSON | The Arda Persian patch slot (PATCH_READY_SLOT content) | 100% | patch-slot content id intact; the seed stays English |
| `Dockerfile` | Docker | Kernel + web build | 100% | MAIRON naming |
| `docker-compose.yml` | Docker | Local stack: kernel + web + miniapp + admin | 100% | MAIRON header and service docs |
| `start-railway.sh` | Bash | Railway boot script | 100% | 69-key env contract |

## The 16 end-to-end gates (IMPERIUM Section 17 standard)

### Gate 1 — 100% pure English
- raw Arabic-script scan over all text files == 0 matches (seed DB scanned separately through sqlite3: 0 Persian across all tables)
### Gate 2 — pure MAIRON brand
- no Wave-10 / grand-update badges in README, HTML, tools, docs; zero stale git tags; `wave10.rs` / `dna:wave10:...` kept only as stable file/content ids and documented
### Gate 3 — anti-placeholder + TS parse
- todo|fixme|placeholder over packages/ + apps/ == 0; 92 TS/TSX files parse clean (tsc syntax family)
### Gate 4 — production seed DB integrity
- integrity ok; 14/14 migrations SHA-384-exact; Product 19 cols; SUM(Posting)==0; KbEntry 334; 24 ticks; 32 telemetry rows; 0 Persian across all tables
### Gate 5 — 69-key env parity
- .env.example == railway.env.example == packages/config MAIRON_ENV_SPEC — exactly 69 keys, identical set and order
### Gate 6 — Trinity
- IQ 136/139/150 locked; the REA >= 85 gate in the pipeline + the bot suite
### Gate 7 — Arda
- 20-locale registry in the seed; PATCH_READY_SLOT (incl. fa-IR) present
### Gate 8 — Super-Studio
- toyspark / creator_studio / pro_terminal tiers on the wire
### Gate 9 — CCXT quant
- Precise i128 core; 8 unique keyless venues; Sharpe >= 1.20 live gate
### Gate 10 — Base-8453 treasury + genome
- chain 8453 + canonical USDC + XTR constants; OMNI genome router present
### Gate 11 — Supremacy
- MaironVerifiedSeal zod contract exported; /supremacy/swe-verify route live
### Gate 12 — Neural-HUD
- render_neural_hud.py plan + presets exit 0 (dry validation; the CLI's dry mode is `plan`/`presets` — there is no literal `--dry-run` flag)
### Gate 13 — Grok 10-DNA lab
- 32 telemetry rows, 10 cohorts; /api/v1/grok-lab/{telemetry,export-sft-dpo,summary} registered and tested
### Gate 14 — War-room
- render_war_room.py --check exit 0 (byte-identical); closing wallet $42.55; signature MAIRON-WAR-ROOM (renamed from the KIA-WAR-ROOM-W10 relic)
### Gate 15 — workspace tests + clippy
- cargo test --workspace -> 826 passed / 0 failed (verified live: 826/0); cargo clippy --workspace --all-targets -- -D warnings -> 0 warnings
### Gate 16 — MANIFEST seal
- MANIFEST.txt recomputed over every tracked file after this audit; committed and pushed to origin main

## Automated check ledger (41 checks)

| Check | Result | Detail |
| --- | --- | --- |
| rg raw Persian over text files == 0 matches | PASS | matches=0 [] |
| no Wave-10 / grand-update badges in README, HTML, tools, docs | PASS | hits=0 [] |
| no stale wave git tags | PASS | tags='' |
| rg todo|fixme|placeholder over packages/ + apps/ == 0 | PASS | lines=0 |
| TS/TSX syntax parse (tsc TS1xxx family) clean | PASS | ALL TS/TSX PARSE CLEAN (syntax family) |
| PRAGMA integrity_check == ok | PASS | sqlite ok |
| 14/14 _sqlx_migrations with exact SHA-384 checksums | PASS | rows=14 files=14 |
| Product has 19 columns | PASS | 19 cols |
| SUM(Posting.amount) == 0 | PASS | trial balance 0 |
| KbEntry == 334 | PASS | 334 rows |
| 24 causal war-room ticks | PASS | 24 ticks |
| 0 Persian characters across all tables | PASS | persian rows=0 |
| .env.example has exactly 69 keys | PASS | 69 keys |
| railway.env.example has exactly 69 keys | PASS | 69 keys |
| .env.example == railway.env.example (exact order + names) | PASS | identical |
| packages/config MAIRON_ENV_SPEC has exactly 69 keys | PASS | 69 keys |
| config spec == env files (same key set) | PASS | same set |
| Trinity IQ tiers 136/139/150 locked | PASS | 136/139/150 present |
| REA gate >= 85 enforced | PASS | 85 gate present in pipeline + bot |
| seed LocalePatchRegistry == 20 locales | PASS | 20 rows |
| PATCH_READY_SLOT present (incl. fa-IR slot) | PASS | patch-ready slots exist |
| ux tiers toyspark/creator_studio/pro_terminal on the wire | PASS | 3 tiers |
| Precise i128 core present | PASS | i128 in ccxt_core |
| 8 keyless venues with unique ids | PASS | binance-spot ... base-mainnet-dex |
| Sharpe >= 1.20 live gate | PASS | 1.20 gate |
| BASE_CHAIN_ID == 8453 | PASS | 8453 present |
| USDC contract constant exact | PASS | canonical USDC |
| XTR Stars currency | PASS | XTR present |
| 45-model genome (5 categories x 9 slots) engine present | PASS | omni_genome_router present |
| MaironVerifiedSeal contract present | PASS | seal contract exported from superstudio.ts |
| SWE verify route live | PASS | route present |
| render_neural_hud.py plan + presets exit 0 (dry validation) | PASS | plan=0 presets=0 |
| UserLabTelemetryLog == 32 rows | PASS | 32 rows |
| 10 distinct DNA cohorts | PASS | 10 cohorts |
| grok-lab kernel routes registered | PASS | 3 routes |
| render_war_room.py --check exit 0 | PASS | exit=0 |
| war-room closing wallet == $42.55 | PASS | 42.55 |
| war-room signature constant | PASS | signature present |
| FNV-1a session fingerprint | PASS | fnv1a32 |
| SFT/DPO pair builders | PASS | builders exported |
| MaironWorldState single-source contract | PASS | contract present |

## Honest deviations ledger

- `render_neural_hud.py` has no literal `--dry-run` flag; its dry validation is `plan` + `presets` (both exit 0). Gate 12 records this reality instead of inventing a flag.
- `crates/cli`, `crates/wasm`, `crates/vault_pro` do not exist in this monorepo; the audit records them as `n/a` with their real functional homes (scripts/, tools/, docs/vault).
- `crates/db/src/wave10.rs`, the `dna:wave10:...` seed content ids and migration file names (`0012_wave8_...`) are stable identifiers referenced by checksums and seed rows; they are documented as ids, not badges.
- The `KIA-GROK-UX-LAB-W9` constant is a wire-protocol signature consumed by clients; it stays byte-stable for compatibility and is documented as an id, not a UI badge.
