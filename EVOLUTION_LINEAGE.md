# EVOLUTION_LINEAGE — Full Evolutionary History of KIAN v2 (`تاریخچهٔ پیشرفت`)

> **Doctrine**: [`KIA-BRAIN-OS/11_UNCHAINED_KIA_DNA.md`](KIA-BRAIN-OS/11_UNCHAINED_KIA_DNA.md) — Law #3:
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

> `[EXEC_WEIGHT: 0.00 | HISTORICAL_LINEAGE]` — preserved proof-of-work lineage; execution weight lives on `HEAD`.

| Commit | Milestone |
| :--- | :--- |
| `7c699ad` | **Initial commit** — project genesis |
| `2026ee7` | KIAN v2 launch-ready: Rust compile/test fixes, serde/Zod/SDK contract sync, world-class UX polish, zero build errors |
| `aeb8c23` | KIAN v2 turnkey: 1-click browser personas + demo faucet (`KIAN_DEV_MODE`), Founder Command Center, charter mint cap, live E2E smoke 21/21 |
| `eb82694` | **Railway migration**: dropagent 30-day community (466 users, 247 referrals, sanitized genesis 58,700.2 KIAN, 108 released escrows) → deterministic idempotent ETL into `kian-production.db`, migration 0006, Railway `Dockerfile`/`start-railway.sh` with seed-on-first-boot, live E2E 39/39, workspace tests **282 green** |

### Wave 1 — AI Gateway Forges & Hidden-Gems Vault

> `[EXEC_WEIGHT: 0.00 | HISTORICAL_LINEAGE]` — preserved proof-of-work lineage.

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

> `[EXEC_WEIGHT: 0.00 | HISTORICAL_LINEAGE]` — preserved proof-of-work lineage.

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

> `[EXEC_WEIGHT: 0.00 | HISTORICAL_LINEAGE]` — preserved proof-of-work lineage.

| Milestone |
| :--- |
| DNA #11 [`11_UNCHAINED_KIA_DNA`](KIA-BRAIN-OS/11_UNCHAINED_KIA_DNA.md) injected into `"KbEntry"` → **326 rows** |
| `EVOLUTION_LINEAGE.md` (this file, repo root; mirrored at `docs/history/EVOLUTION_LINEAGE.md`) committed as the lineage record |
| Execution-weight tagging: latest v2 kernel/genome entries marked `[EXEC_WEIGHT: 1.00 \| ACTIVE_PRIMARY]` |
| Environment variables strictly reordered by operational priority (`P0` → `P5`) in `.env.example` & `railway.env.example` |
| `.dockerignore` hardened: history stays in Git, build context carries runtime weight only |
| `README.md` + `README.fa.md` bilingual mirror with 100% forensically verified numbers |


---

## Wave 3 — Sovereign Trinity Studio Pipeline (`خط لولهٔ سه‌کالبدی حاکم`)

> `[EXEC_WEIGHT: 0.00 | HISTORICAL_LINEAGE]` — preserved proof-of-work lineage.

| Commit | Milestone |
| :--- | :--- |
| *(this release)* | **Wave 3**: the signature architectural centerpiece — `Alpha → Command Center [OpenBot 3×9 DNA] → Mairon [IQ 150 / EQ 145]` wired end-to-end: `trinity_pipeline.rs` engine in `kian-hermes` (Alpha slot extraction & deterministic `MasterPromptSpec`/`ProjectBlueprint` compilation, OpenBot dynamic sub-agent factory with 3-layer cache-safe prompts + 9-fold stochastic genomes + SplitMix64 alleles, self-healing repair loop `execute_command_center_with_mairon_loop` max 3 attempts to the REA `>= 85` gate), Mairon overclock DNA constants (`IQ=150/EQ=145/Lookahead=8/ToM=3/Ambition=0.99`) locked in `kian-domain`, live endpoints `POST /builder/trinity/alpha`, `POST /builder/trinity/orchestrate`, `POST /market/products/{id}/mairon-confirm`, `GET/POST /market/products/{id}/reviews`, migration `0007_trinity_reviews.sql` (mairon stamps + `"ProductReview"` one-vote lock) — **560 tests green, `"KbEntry"` 327, trial balance `0`** |

### Wave 3 — DNA & release milestones

| Milestone |
| :--- |
| DNA #12 [`12_TRINITY_AVATAR_ENGINE.md`](KIA-BRAIN-OS/12_TRINITY_AVATAR_ENGINE.md) created + mirrored to `docs/dna/kia-brain-os/` |
| DNA #12 injected into `"KbEntry"` as `kb:dna-12-trinity-avatar-engine` → **327 rows**, tagged `[EXEC_WEIGHT: 1.00 \| ACTIVE_PRIMARY]` |
| `KIA-BRAIN-OS` is now the **12-file** creator brain (`01_DNAme.md` … `12_TRINITY_AVATAR_ENGINE.md`) |
| Trinity Avatars: `ALPHA (136/122 · brake 0.04 · temp 0.88)` → `COMMAND CENTER (139/129 · strictness 0.96 · 3×9 OpenBot DNA)` → `MAIRON-150 (150/145 · lookahead 8 · ToM 3 · ambition 0.99)` |
| Two-key marketplace gate live: `⚒️ Mairon REA Certified: {score}/100 (Forge-Grade)` + 5-star Bayesian `combinedRankScore` |


---

## Wave 4 — Step 1/4: `ARDA` 24-Hour Planetary Localization Engine on authentic `Fishkal` architecture + Mairon's Autonomous `SKILL.md` Forge (`موتور بومی‌سازی سیعه‌ای آردا + فورج مهارت مایرون`)

> `[EXEC_WEIGHT: 0.00 | HISTORICAL_LINEAGE]` — preserved proof-of-work lineage.

| Commit | Milestone |
| :--- | :--- |
| *(this release)* | **Wave 4 Step 1**: `Fishkal` (Kia's zero-dependency HTML5 engine — `3,616` lines, `667,241` bytes) industrialized into `ARDA` (`آردا`): `arda_fishkal_engine.rs` in `kian-hermes` — verbatim `viewBox="0 0 20 14"` vector flag dictionary for the original 13 locales + 7 planetary additions (`de/ru/it/he/ur/vi/th`) = exactly 20 locales with native-script glyphs (`'فا'`,`'ع'`,`'اردو'`,`'עב'`), Circular Mini-Flag Badges (`border-radius:50%`, defensive caret-rotation override) with `detect_origin_locale` script detection and `render_product_flag_strip`, `ArdaFishkalBundleGenerator` curing all **25 audit defects** (atomic 3-attribute root flip, per-script typography `.34em→.16em/.07em+text-indent` for RTL under Vazirmatn, unified 6-scope `ArdaLocaleCatalog` + mechanical 100%-parity validator with exact `coverage_score`, cached `Intl` engines with region subtags + `XTR/USD/AED/IRR/USDC` currencies, DOM & Canvas bidi isolation with the exact 4-step `drawBidiText` procedure, WAI-ARIA listbox switcher with `D-1..D-8` fixed — logical `inset-inline-start`, localized `aria-label`, idempotent builder, visible `✓`+`--gold` (WCAG 1.4.1), roving focus + `preventScroll`, outside `pointerdown` dismissal, 44px touch targets in 40px glass pill), `ArdaPlanetaryCronDaemon` (24h `86_400`s ticks with `sha256` content-delta guard), `MaironSkillForge::forge_skill_package` (autonomous `SKILL.md` + `skill.json` below-85 refusal) — live endpoints `GET /api/v1/arda/flags`, `POST /api/v1/arda/localize-now` (owner-only stamping), `GET /api/v1/market/products/{id}/skill` (auto-forge on the fly, `422` below gate), migration `0008_arda_locales_and_skill_forge.sql` (`originLocale`/`supportedLocalesJson`/`skillMd`/`skillJson`/`ardaLastLocalizedAt` on `"Product"`, all 20 rows preserved) + `build.rs` migration watcher — **583 tests green, `"KbEntry"` 328, trial balance `0`** |

### Wave 4 — DNA & release milestones

| Milestone |
| :--- |
| DNA #13 [`13_ARDA_FISHKAL_PLANETARY_ENGINE.md`](KIA-BRAIN-OS/13_ARDA_FISHKAL_PLANETARY_ENGINE.md) created + mirrored to `docs/dna/kia-brain-os/` |
| DNA #13 injected into `"KbEntry"` as `kb:dna-13-arda-fishkal-planetary-engine` → **328 rows**, tagged `[EXEC_WEIGHT: 1.00 \| ACTIVE_PRIMARY]` |
| `KIA-BRAIN-OS` is now the **13-file** creator brain (`01_DNAme.md` … `13_ARDA_FISHKAL_PLANETARY_ENGINE.md`) |
| `Fishkal` heritage test-pinned: `original_13_svgs_are_verbatim_fishkal` (byte-identical SVGs) + node-syntax-checked runtime bundle |
| Marketplace cards carry circular flag strips; certified products self-publish `SKILL.md`/`skill.json` to agent registries |

------

## Wave 5 — Step 2/4: The Universal Cognitive UX Engine (`Age 9` ↔ `Age 28`), Zero-Cost 9-Modality Super-Studio & `Portfolio` Creator Storefront (`استودیوی ادراک‌سیار + ویترین سازنده`)

> `[EXEC_WEIGHT: 0.00 | HISTORICAL_LINEAGE]` — preserved proof-of-work lineage.

| Commit | Milestone |
| :--- | :--- |
| **Commit #24** `[EXEC_WEIGHT: 0.00 \| HISTORICAL_LINEAGE]` | **Wave 5 Step 2**: `super_studio_engine.rs` in `kian-hermes` — three deterministic sub-engines on one zero-key core: (1) `CognitiveUxMorphEngine` — 3-tier cognitive morph `ToySpark` (Age 9 / IQ 50: 3-giant-button card `✨ بساز / 🎨 قشنگش کن / 🪙 بفروش و ستاره بگیر`, `max_actions_per_screen=3`, `min_tap_target_px=64`, `max_words_per_label=5`, automatic `speechSynthesis` voice wizard with exactly 3 prompts, 1-click locked pricing `50 XTR / $1.00 USDC / 1,000,000 µKIAN`, `zero_loss_sandbox=true`) ↔ `CreatorStudio` ↔ `ProTerminal` (Age 28: dense multi-pane, keyboard-first `Ctrl+K`/`Meta+K` + `/` command palette) with deterministic `infer_or_override_tier` from telemetry (prompt length, technical tokens, typing cadence) + explicit override always wins + 1-click tier switching without losing project state; **`build_toyspark_instant_creation`** — one sentence (`یه دایناسور آبی که توی فضا بستنی می‌فروشه`) → complete playable HTML5 mini-storybook/game with 3 procedural `sha256`-seeded SVG scenes, procedural WebAudio SFX and a ready-to-sell listing; (2) `OmniModalStudioEngine` — **9 zero-cost modalities** (`search` Agent-Reach Jina/Exa MCP + 120 public CORS APIs, `deep` Mairon 8-step verification tree, `agent` 9-Fold Stochastic DNA + downloadable `SKILL.md`, `workspace` multi-file canvas + sandboxed preview + code inspector, `voice` SpeechSynthesis/MediaRecorder + procedural ambient, `image` deterministic SVG/Canvas2D + zero-key cascade, `video` 60fps `Fishkal TL` timeline + variable-lambda `damp` + WebM/frame recorder, `copy` 20-locale RTL/LTR sales copy, `sfx` Cosmos 5-trigger rack) with the **dual procedural bundle** `emit_dual_cosmos_fishkal_bundle`: Engine A `Fishkal` ocean surf & harmonic bloom (`index.html:L709–728, L1665–1750, L2172` — 4s brown-noise loop `b0=(b0+0.02*white)/1.02` scaled `0.18`, lowpass `520Hz` with `0.06Hz` sine LFO `±220Hz` sweep, D-major pentatonic bloom `[587.33, 880, 1174.66]` + 4-voice signature chord `[293.66, 440, 587.33, 880]`, deterministic `xorshift32`, 4-octave `fbm` (`f *= 2.03`), cubic-Hermite `smoothstep`, framerate-independent `damp` with lambda `2.3` inside the `0.42..0.63` transition window / `5.0` normal) + Engine B `Portfolio Cosmos` starfield & feedback-delay synth (`cosmos.js:L6–60, L83–149, L216–269` — DPR cap `2`, `110` mobile / `260` desktop stars, radial nebula gradients, `0.05` pointer-parallax damping, `0.16` HUD easing, 7 milestone nodes; `DelayNode(0.22s)` + feedback `0.25` + lowpass `1600Hz` with all 5 SFX triggers `ping(220+(idx%7)*40, 0.12s)`, `open(180→520 exponential, 0.28s)`, `whoosh(260→1200 bandpass, Q=2.2, 0.32s)`, `select([440,660] triangle, 0.07s)`, `milestone([330,495,660] sine, 0.09s)`) + shared `__fkPauseDepth` RAF lifecycle + deterministic `js_syntax_sanity_check`; (3) `PortfolioStorefrontEngine` — all **12 canonical `projects.json` keys** preserved + 7 live-commerce extensions, top-2 featured auto-promotion (`index.js:L17–18`), verbatim dark-glass tokens (`style.css:L1–25, L53–57, L112–139`: `#05050a`, `rgba(16,18,32,.68)`, `#7c5cff`, `#19d3da`, `#ff5ca8`, `cubic-bezier(.2,.7,.2,1)`, `blur(14px)`, `clamp(36px,5.6vw,62px)`, `1240px`, `Space Grotesk`/`Inter`/`JetBrains Mono`/`Vazirmatn`), keyboard-first command palette config (`index.js:L396–459`) and **all 6 forensic `KiaAgentX/portfolio` defects cured** (Fix #1 portable root `./seed/portfolio_catalog` + `KIAN_PORTFOLIO_CATALOG_PATH` — no `path.resolve(PORT, '..')`; Fix #2 `let loaded = false;` declared before `setTab` hash routing — TDZ `ReferenceError` eliminated; Fix #3 `sandbox="allow-scripts"` strictly forbidding `allow-same-origin`; Fix #4 `.env`/`.env.*`/`*.pem`/`*.key`/`id_rsa`/`credentials.json`/`*.sqlite` never copied or exposed; Fix #5 filter chips as `<button type="button" class="chip" aria-pressed>`; Fix #6 Cosmos RAF wired into the shared ref-counted `__fkPauseDepth`/`fkPause()`/`fkResume()` + `document.hidden` guard) — live endpoints `GET /api/v1/studio/ux-profiles` (3 tiers + 9 modalities + 6/6 fix report), `POST /api/v1/studio/generate` (tier inference → artifact generation with dual bundle → ARDA 20-locale packaging with idempotent injection + circular flag strip → Mairon `SKILL.md`/`skill.json` forge at the REA gate with deterministic `88..=97` scores → persisted to `studio_creations`), `GET /api/v1/studio/portfolio-catalog` — migration `0009_super_studio_and_portfolio_storefront.sql` (new `studio_creations` table + 2 indexes, no ledger table touched) — **607 tests green (583 + 24 new), `"KbEntry"` 329, trial balance `0`** |

### Wave 5 — DNA & release milestones

| Milestone |
| :--- |
| DNA #14 [`14_OMNI_STUDIO_AND_ADAPTIVE_UX.md`](KIA-BRAIN-OS/14_OMNI_STUDIO_AND_ADAPTIVE_UX.md) created + mirrored to `crates/hermes/KIA-BRAIN-OS/` and `docs/dna/kia-brain-os/` |
| DNA #14 injected into `"KbEntry"` as `kb:dna-14-omni-studio-adaptive-ux` → **329 rows**, tagged `[EXEC_WEIGHT: 1.00 \| ACTIVE_PRIMARY]` |
| `KIA-BRAIN-OS` is now the **14-file** creator brain (`01_DNAme.md` … `14_OMNI_STUDIO_AND_ADAPTIVE_UX.md`) |
| 9-modality zero-cost gate test-pinned: every spec `auth: none` + `zero_cost: true`; dual-bundle `sha256` byte-stable |
| `PortfolioSafetyReport` served on the wire: 6/6 `fixed` with mechanical evidence checks |

------

### Wave 6 — Quant & financial engine milestones

> `[EXEC_WEIGHT: 0.00 | HISTORICAL_LINEAGE]` — preserved proof-of-work lineage.

| Milestone |
| :--- |
| DNA #15 [`15_CCXT_AND_VIBE_TRADING_QUANT_ENGINE.md`](KIA-BRAIN-OS/15_CCXT_AND_VIBE_TRADING_QUANT_ENGINE.md) created + mirrored to `crates/hermes/KIA-BRAIN-OS/` and `docs/dna/kia-brain-os/` |
| DNA #15 injected into `"KbEntry"` as `kb:dna-15-ccxt-vibe-quant` → **330 rows**, tagged `[EXEC_WEIGHT: 1.00 \| ACTIVE_PRIMARY]` |
| `KIA-BRAIN-OS` is now the **15-file** creator brain (`01_DNAme.md` … `15_CCXT_AND_VIBE_TRADING_QUANT_ENGINE.md`) |
| Pure-Rust `ccxt` core: `Precise` i128 fixed-point + float-free `TICK_SIZE` (forensic fix #1) + L2 `OrderBook` sequence-gap guard (forensic fix #2) + dual throttler + 8-venue keyless matrix |
| `Vibe-Trading` causal quant engine: typed `StrategyAst` (zero dynamic execution — fix #1), strictly-causal `&candles[0..=t]` backtester with `+1 bar` fills (fix #2), SHA-256 `VerifiedNumericClaimLedger` (fix #3), `PortfolioRiskGuard` hard kill-switch (fix #4) |
| Mairon sovereign quant gate: sharpe ≥ 1.20 ∧ \|DD\| ≤ 20% ∧ ≥ 5 trades ∧ causal proof ∧ ledger verified — with 4 deterministic regime-boost variants |
| `quant_strategies` live on migration `0010`; `GET /quant/venues`, `POST /quant/vibe-backtest`, `GET /quant/strategies` on the wire |

------

### Wave 7 — Quad-model genome matrix, Day-1 Base treasury & final MIT cut milestones

> `[EXEC_WEIGHT: 0.00 | HISTORICAL_LINEAGE]` — superseded by **Wave 7 Hardened (Commit #27)** below.

| Milestone |
| :--- |
| DNA #16 [`16_QUAD_MODEL_GENOME_AND_BASE_TREASURY.md`](KIA-BRAIN-OS/16_QUAD_MODEL_GENOME_AND_BASE_TREASURY.md) created + mirrored to `crates/hermes/KIA-BRAIN-OS/` and `docs/dna/kia-brain-os/` |
| DNA #16 injected into `"KbEntry"` as `kb:dna-16-quad-genome-base-treasury` → **331 rows**, tagged `[EXEC_WEIGHT: 1.00 \| ACTIVE_PRIMARY]` |
| `KIA-BRAIN-OS` is now the **16-file** creator brain (`01_DNAme.md` … `16_QUAD_MODEL_GENOME_AND_BASE_TREASURY.md`) — the 4-step wave plan is complete |
| `OMNI-MODEL-GENOME` v4 quad-model matrix: **4 frontier models × 9 canonical DNA files = 36 verified files** (`GPT 6-Web`, `GEMINI 4 ARAGON HIGH`, `DeepSeek-3.2-WEB-Think-mode`, `Cluade-sonnet-5.5-extra` with both exact+alias identifiers) + 20-workload champion matrix; legacy v1/v2 workspaces preserved under `legacy/` |
| Day-1 Ironclad 4-Stream Sovereign Revenue Charter ("جنگ اول به از صلح آخر"): 1000 bps charter tax, 1000..=5000 bps programmable affiliate split with unreferred direct-sale rule (2000 bps platform / 8000 bps creator), 100%-margin instant boosts, 150 bps withdrawal spread — all zero-sum-verified with the Wave-6 `Precise` engine |
| Production Solidity on Ethereum Base (`Chain ID 8453`): `ZeroXKiaToken.sol` (fixed 1B supply, EIP-2612 `permit`, `burn`, 100/100 GoPlus/TokenSniffer clean target) + `KiaBaseEscrowTreasury.sol` (non-custodial atomic 4-stream splitter, `nonReentrant`, closed asset whitelist) |
| EIP-681 one-tap payment URIs bound to `@8453`; XTR/µKIAN refuse on-chain settlement (fail-closed) |
| `sovereign_treasury_splits` live on migration `0011`; `GET /monopoly/genome-matrix`, `GET /monopoly/charter-and-base`, `POST /monopoly/simulate-split` on the wire |
| `QuadModelGenomeRouter` (36-file disk verification, alias resolution, Trinity wiring `Alpha → Logic-Prime → Mairon`) + `SovereignMonopolyOrchestrator` + `ProductHuntCmcLaunchKit` (`docs/launch/PRODUCT_HUNT_AND_CMC_KIT.md`) |

------


------

### Wave 7 Hardened — Commit #27: penta-model 45-file DNA matrix, dual-mode routing & D1–D6 forensic fixes

> `[EXEC_WEIGHT: 1.00 | ACTIVE_PRIMARY]` — the current execution weight of the repository.
> Commit message: `feat(wave7-hardened): 5-model 45-file DNA matrix with GLM 5.3 flash MAX, manual/auto agent router & D1-D6 audit fixes`

| Milestone |
| :--- |
| `OMNI-MODEL-GENOME` v4 upgraded to the **penta-model matrix: 5 frontier models × 9 canonical DNA files = 45 verified files** — model `#05 GLM 5.3 flash MAX` (`models/05-glm-5.3-flash-max/`, aliases `glm-5.3-flash-max` / `GLM-5.3-Flash-Max`) added with the full 9-file canonical DNA set (ultra-low-latency flash reasoning, high-throughput multi-tool agentic execution with batched rounds + state ledger, polyglot Rust/Solidity/TypeScript/SQL synthesis, rapid patch verification, 20-task self-reported scorecard) |
| `QuadModelGenomeRouter` upgraded in place (type name preserved): `SovereignModelId` alias, `SOVEREIGN_MODEL_COUNT = 5`, `SOVEREIGN_GENOME_FILE_COUNT = 45`, new `QuadTaskDomain::FlashAgenticExecution` champion rule, `ModelSelectionMode` dual-mode — `ManualOverride { model }` (user pins any of the 5 models; aliases resolve; unknown alias fails closed) and `AgentAutoDecision` (router analyzes task telemetry — keywords, latency requirement, tool-call density, math/quant depth, UI/web need, Rust/Solidity systems complexity — and autonomously picks the optimal model, returning `selected_model`, `selection_mode`, `confidence_score`, `decision_rationale`) |
| Wire surface: `GET /api/v1/monopoly/genome-matrix` now reports 5 models / 45-file verification + both `manual_override` and `agent_auto_decision` capabilities; new `POST /api/v1/monopoly/route-model` executes either mode and returns the chosen model's distilled system prompt + 9-file verification proof + decision rationale |
| `crates/ai/src/omni_genome_router.rs` unified: directory loader accepts **both** the canonical v4 names (`01_IDENTITY_AND_PERSONA_GENOME.md`…`09_DISTILLED_CLONE_SYSTEM_PROMPT.md`) and the legacy methodology names (`01_TOPOLOGY_AND_HARNESS.md`…`09_ROUTING_AND_HANDOFF_CARD.md`); `SOVEREIGN_MODEL_DIRS` registers all 5 model directories; `resolve_sovereign_model_alias` covers `Cluade-sonnet-5.5-extra` ≡ `Claude-sonnet-5.5-extra` and all three `GLM 5.3 flash MAX` spellings; `load_sovereign_penta_registry` + `builtin_sovereign_penta_profiles` expose the 5-model registry |
| **D1** — `KIAN_DEV_MODE` strictly fail-closed: only explicit `1`/`true`/`yes` (case-insensitive) enables dev endpoints; unset flag never enables them, even without a bot token (fail-open hole closed) |
| **D2** — zero `.expect()` on DB-sourced rows: `safe_prefixed_wire_id` / `safe_wire_id` sanitize to `^[A-Za-z0-9:_.-]+$`, keep IDs within `1..=128` with a deterministic SHA-256 hex-16 suffix (128-char escrow IDs no longer panic the admin HITL view) |
| **D3** — CORS allowlist extended with `DELETE`/`PUT`/`PATCH` (+ preflight test proving `DELETE` on `/admin/products/{id}`); unknown origins stay fail-closed |
| **D4** — `/.well-known/agent-card.json` moved under `rate_limit_middleware` (429 over the limit; blind spot closed) |
| **D5** — all 319 historical `KbEntry` rows tagged `[EXEC_WEIGHT: 0.00 | HISTORICAL_LINEAGE]` (12 primary rows keep `1.00 | ACTIVE_PRIMARY`); `KbEntry=331`, `User=466`, `ReferralLineage=247`, `Escrow=108`, `Product=20`, `Posting=870` and the 58,700,200,000 µKIAN zero-sum trial balance untouched; the release flattener rewrites both `EVOLUTION_LINEAGE.md` hash tables with the **actual flattened commit hashes** so 100% of listed hashes resolve in the exported `.git` |
| **D6** — `.env.example` + `railway.env.example` pin `KIAN_DEV_MODE=0` and `KIAN_ADMIN_KEY=replace-with-32-char-sovereign-admin-key-min16`; `railway-env-v1.0.0.json` synced to all 69 P0–P5 keys (incl. `KIAN_SEED_DATABASE_PATH` and every Wave 3–7 key) |
| **676 workspace tests green** (583→661→676), 0 failures; `cargo clippy --workspace --all-targets -- -D warnings` 0 warnings; `KbEntry` **331** (D5 tagging, zero new rows) |

------

## Growth ledger

| Metric | Wave 0 (Railway migration) | Step 5 | Step 6 | MIT cut | Wave 3 (Trinity) | Wave 4 (ARDA) | Wave 5 (Studio) | Wave 6 (Quant) | Wave 7 (Final) | **Wave 7 Hardened (#27)** |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| Workspace tests | 282 | 516 | **530** | 530 | **560** | **583** | **607** | **641** | 661 | **676** |
| `"KbEntry"` rows | 62 | 323 | 325 | **326** | **327** | **328** | **329** | **330** | **331** | **331** (D5 lineage tagging — 12 active primary / 319 historical) |
| Guardian fleet | — | 7/7 | 7/7 | 7/7 | 7/7 | 7/7 | 7/7 | 7/7 | 7/7 | 7/7 |
| Mairon REA verdict | — | step-5 gate | **92/100 PASS** | 92/100 PASS | **92/100 PASS** + live Trinity forge gate (85+) | **92/100 PASS** + planetary skill forge gate (85+) | **92/100 PASS** + adaptive UX gate | **92/100 PASS** + sovereign quant gate (sharpe ≥ 1.20) | **92/100 PASS** + Day-1 treasury zero-sum gate (4 streams, `Precise`-verified) | **92/100 PASS** + penta-model 45-file DNA gate + dual-mode router gate + D1–D6 forensic fixes (676 tests green) |
| Trial balance (`SUM(Posting.amount)`) | 0 | 0 | 0 | **0** | **0** | **0** | **0** | **0** | **0** | **0** |

*The trial balance over 58,700,200,000 micro-KIAN has been exactly zero at every milestone — the
ledger has never moved out of balance.*
