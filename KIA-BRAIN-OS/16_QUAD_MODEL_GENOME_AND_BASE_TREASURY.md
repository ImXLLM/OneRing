# Creator Brain DNA 16: The Pentamodel Genome Matrix, the Four-Stream Day-One Treasury on Base, and the Solidity Contracts

> `KIA-BRAIN-OS/16_QUAD_MODEL_GENOME_AND_BASE_TREASURY.md` — Wave 7 (Step 4/4 — FINAL WAVE) + Commit #27 (`Wave 7 Hardened`) of kernel `0xKIA`
> Executive label: `[EXEC_WEIGHT: 1.00 | ACTIVE_PRIMARY]`
> Engines: `OMNI-MODEL-GENOME/` · `crates/wallet/src/base_treasury.rs` · `crates/hermes/src/sovereign_monopoly_engine.rs` · `crates/ai/src/omni_genome_router.rs`
> Contracts: `contracts/base/ZeroXKiaToken.sol` · `contracts/base/KiaBaseEscrowTreasury.sol`
> Migration: `0011_sovereign_monopoly_genome_and_base_treasury.sql` · Wire: `GET/POST /api/v1/monopoly/*`

---

## 1. Executive Summary

DNA Sixteen — the final wave — locks three governing layers in place simultaneously:

1. **The `OMNI-MODEL-GENOME` v4 genome matrix (pentamodel × 9-file — Commit #27)** — 45 canonical files
   for `GPT 6-Web`, `GEMINI 4 ARAGON HIGH`,
   `DeepSeek-3.2-WEB-Think-mode`, `Cluade-sonnet-5.5-extra` (with the
   `Claude-sonnet-5.5-extra` alias) and `GLM 5.3 flash MAX` (with the
   `glm-5.3-flash-max` and `GLM-5.3-Flash-Max` aliases); every model carries exactly 9 files
   from `01_IDENTITY…` through `09_DISTILLED_CLONE_SYSTEM_PROMPT.md`, plus the
   22-task comparison matrix and README; the v1/v2 genomes live on under `legacy/`.
2. **The sovereign day-one treasury on Ethereum Base (`Chain ID: 8453`)** —
   four revenue streams with zero-sum atomic splits on `Precise`: the 10% charter tax,
   the 10%–50% dependent affiliate fee under the "un-referred direct-sale rule" (2000 bps platform /
   8000 bps creator), the 100% margin boosts, and the 1.5% withdrawal spread.
3. **Two Solidity contracts (`^0.8.24`)** — `ZeroXKiaToken` (a fixed supply of 1
   billion + EIP-2612 permit + burn, a clean 100/100 GoPlus/TokenSniffer architecture)
   and `KiaBaseEscrowTreasury` (non-custodial escrow + an atomic splitter + a
   non-reentrant lock).
4. **Dual-mode model selection and the D1–D6 forensic hardening (Commit #27)** —
   `ModelSelectionMode::ManualOverride { model }` (the user explicitly locks one of the 5 models;
   every alias resolves; unknown fails closed) and
   `ModelSelectionMode::AgentAutoDecision` (the router analyzes prompt telemetry —
   keywords, latency needs, tool-call density, mathematical/quantitative depth, UI/web needs, and
   Rust/Solidity complexity — and automatically selects the optimal model; output:
   `selected_model`, `selection_mode`, `confidence_score`, `decision_rationale`)
   on `POST /monopoly/route-model`; alongside closing the six audit findings:
   D1 (dev-mode fail-closed), D2 (safe wire IDs ≤128 without panic),
   D3 (CORS with DELETE/PUT/PATCH), D4 (`agent-card` rate limiting),
   D5 (the `HISTORICAL_LINEAGE` tag on the 319 historical rows + flattened lineage-hash
   synchronization) and D6 (`KIAN_DEV_MODE=0` + an admin key of ≥16 characters in
   all three env formats).

The kernel covenant holds: zero `unsafe`, zero `Clippy` warnings (`-D warnings`), full
determinism, and zero floating point on money — every leg is proven with the Wave 6 `Precise` engine.

## 2. The Genome Matrix — the Complete Map of `omni-genome-v4`

| # | Model | Folder | Governing specialty | Trinity avatar |
| --- | --- | --- | --- | --- |
| 01 | `GPT 6-Web` | `models/01-gpt-6-web/` | Real-time web synthesis, frame-1 artifacts (390px zero-overflow), fast full-stack product | `Alpha` |
| 02 | `GEMINI 4 ARAGON HIGH` | `models/02-gemini-4-aragon-high/` | 1M-token cross-repository synthesis, nine-modality orchestrator, macro-architecture validation | `Logic-Prime` |
| 03 | `DeepSeek-3.2-WEB-Think-mode` | `models/03-deepseek-3.2-web-think-mode/` | `<think>` quantitative mathematics, CCXT/Vibe causal-alpha proof, formal verification | `Mairon` |
| 04 | `Cluade-sonnet-5.5-extra` | `models/04-claude-sonnet-5.5-extra/` | Rust surgery, pedantic zero-warning, contract audit, bilingual copy | `Logic-Prime` |
| 05 | `GLM 5.3 flash MAX` | `models/05-glm-5.3-flash-max/` | Low-latency flash reasoning, high-throughput multi-tool agentic execution, multilingual synthesis (Rust/Solidity/TS/SQL), rapid patching and verification | `Logic-Prime` |

Routing rules (`QuadTaskDomain → champion`): `web-synthesis`/`ui-ux-artifact`
→ 01 · `cross-repo-context` → 02 · `quant-proof` → 03 ·
`rust-systems-audit`/`bilingual-copywriting` → 04 ·
`flash-agentic-execution` → 05. Alias resolution is honest:
`Cluade-sonnet-5.5-extra` and `Claude-sonnet-5.5-extra` both map to model 04,
`GLM 5.3 flash MAX`, `glm-5.3-flash-max`, and `GLM-5.3-Flash-Max` map to 05, and
`gpt-6-standard-free` maps to 01 (compatibility with the inherited genome).

## 3. The Day-One Charter — "One War for the Last Peace"

| Stream | Rate | Destination | Rule |
| --- | --- | --- | --- |
| 1 — Charter tax | `1000 bps` (10%) | `system:kia_sovereign_treasury` | On every marketplace sale |
| 2 — Dependent affiliate fee | `1000..=5000 bps` | `user:{ref_id}` | Default fee `1000`; with no referrer: the 10% fee goes **straight to the sovereign treasury** → platform `2000 bps`, creator `8000 bps` |
| 3 — Instant boost | `10000 bps` (100%) | `system:kia_sovereign_treasury` | `FeaturedSlot24h` (250 XTR / $5 USDC / 5M µKIAN) · `MaironFastTrackAudit` (100 XTR / $2 / 2M) · `ViralSwarmAmplifier` (500 XTR / $10 / 10M) |
| 4 — Withdrawal spread | `150 bps` (1.5%) | `system:kia_sovereign_treasury` | Instant settlement and cross-chain withdrawal |

`compute_day_one_atomic_split` builds four double-entry legs
(`buyer_debit + kia_treasury + affiliate + creator == 0`) — the atomic remainder
closes onto the creator and the zero-sum proof is re-checked with `Precise`; an unbalanced split never
escapes, and an unbalanced row is rejected at the db layer (`DbError::Corrupt`).

## 4. The Base `8453` Contracts

* **`ZeroXKiaToken.sol`** — an ERC-20 with the fixed supply of `1,000,000,000 × 10^18`, minted
  once in the constructor to the deployer (the sovereign treasury); no post-constructor
  mint, blacklist, pause, or mutable tax; EIP-2612 `permit` with
  `DOMAIN_SEPARATOR` and user `burn` — targeting 100/100 on GoPlus and TokenSniffer.
* **`KiaBaseEscrowTreasury.sol`** — a closed asset whitelist (native ETH,
  native USDC at `0x8335…2913`, `$0xKIA`), `settleMarketplaceSale` (gross capture from
  the buyer → atomic legs → internal ledger), `purchaseInstantBoost` (100% to the treasury),
  and `withdrawWithSpread` (the 150 bps spread); all under a non-reentrant lock,
  effects-then-interactions, and an immutable treasury; `receive()` is rejected outright.
* **EIP-681** — `build_base_eip681_payment_uri` for one-click Trust/Coinbase/MetaMask
  links with a mandatory `@8453`; off-chain assets
  (`XTR`, `µKIAN`) raise a `NotOnChainSettleable` error — no URI is ever faked.

## 5. The Monopoly Orchestrator and the `/api/v1/monopoly/*` Wire

`SovereignMonopolyOrchestrator` keeps the genome router + the charter + the Base descriptors +
the launch kit (`ProductHuntCmcLaunchKit`) in a single state. Four additive routes:

* `GET /monopoly/genome-matrix` — 5 models with the 9-file verification status (target 45) +
  7 routing rules + both the `manual_override` and `agent_auto_decision` capabilities;
  fail-closed on-disk verification with fallback to builtin.
* `POST /monopoly/route-model` — dual-mode selection across the 5 sovereign models: manual override
  (`manualModelOverride`; aliases of all 5 models; unknown → 400) or agent auto
  (`agentMode=true` or an absent/`"auto"` override) — output: the selected model, the
  distilled clone prompt, the 9-file verification proof, and the decision rationale.
* `GET /monopoly/charter-and-base` — the four charter streams + the Base configuration +
  the audit of both contracts.
* `POST /monopoly/simulate-split` — runs the split, persists to
  `sovereign_treasury_splits` (migration `0011`), and proves zero-sum + EIP-681.

## 6. The Wave 7 Acceptance Gate (+ Commit #27 Hardening)

| Gate | Status |
| --- | --- |
| 45 genome files complete and above threshold (`models/{01..05}/`) | ✅ 5 models × 9 files |
| Charter: referred sale 10/10/80 and unreferred 20/0/80 (exact µUSDC) | ✅ deterministic test |
| Boosts 100% and the 150 bps spread with zero leakage | ✅ |
| EIP-681 with `@8453` + rejection of XTR/µKIAN | ✅ |
| Unbalanced rows rejected at the db layer | ✅ |
| Contracts: no further mint/blacklist/pause/mutable tax | ✅ |
| Dual-mode selection: ManualOverride (5 models, fail-closed) + AgentAutoDecision (telemetry → champion) | ✅ `POST /monopoly/route-model` |
| D1 — `KIAN_DEV_MODE` fail-closed (only `1`/`true`/`yes`) | ✅ value-matrix test |
| D2 — zero `.expect` on DB rows; `safe_prefixed_wire_id`/`safe_wire_id` with a deterministic SHA-256 suffix ≤128 | ✅ |
| D3 — CORS with DELETE/PUT/PATCH (admin preflight green) + fail-closed on unknown origin | ✅ |
| D4 — `/.well-known/agent-card.json` under rate limiting (429 on breach) | ✅ |
| D5 — 319 historical `KbEntry` rows tagged `EXEC_WEIGHT: 0.00 / HISTORICAL_LINEAGE`; invariants intact (331/466/247/108/20/870, zero-sum 58,700,200,000 µKIAN) | ✅ |
| D6 — `KIAN_DEV_MODE=0` + admin key ≥16 in `.env.example`/`railway.env.example`/`railway-env-v1.0.0.json` (69 keys, P0→P5) | ✅ |
| Zero `unsafe` + zero clippy warnings (`-D warnings`) | ✅ 676 tests green |
| `KbEntry` → **331** (`kb:dna-16-quad-genome-base-treasury`) | ✅ |

*Commit #27 (`Wave 7 Hardened`) upgrades the four-stream matrix to pentamodel:
5 models × 9 files = 45/45 DNA files, manual/agent-auto dual-mode selection, and six
closed forensic audit findings — every DNA carries 12–16
`[EXEC_WEIGHT: 1.00 | ACTIVE_PRIMARY]` labels, the 319 inherited historical rows carry
`EXEC_WEIGHT: 0.00 / HISTORICAL_LINEAGE`, and no wave has ever shipped without the Mairon gate.*
