# Creator-Brain DNA 16: The Quad-Model Genome Matrix, the Day-One Four-Stream Treasury on Base, and the Solidity Contracts

> `KIA-BRAIN-OS/16_QUAD_MODEL_GENOME_AND_BASE_TREASURY.md` — Wave 7 (Step 4/4 — FINAL WAVE) of the `0xKIA` kernel
> Execution tag: `[EXEC_WEIGHT: 1.00 | ACTIVE_PRIMARY]`
> Engines: `OMNI-MODEL-GENOME/` · `crates/wallet/src/base_treasury.rs` · `crates/hermes/src/sovereign_monopoly_engine.rs`
> Contracts: `contracts/base/ZeroXKiaToken.sol` · `contracts/base/KiaBaseEscrowTreasury.sol`
> Migration: `0011_sovereign_monopoly_genome_and_base_treasury.sql` · Wire: `GET/POST /api/v1/monopoly/*`

---

## 1. Executive Summary

DNA 16 — the final wave — locks three sovereign layers in place at once:

1. **The `OMNI-MODEL-GENOME` v4 quad-model genome matrix (4 models × 9 files)** — 36 canonical
   files for `GPT 6-Web`, `GEMINI 4 ARAGON HIGH`, `DeepSeek-3.2-WEB-Think-mode`, and
   `Cluade-sonnet-5.5-extra` (with the `Claude-sonnet-5.5-extra` alias); each model carries
   exactly 9 files, `01_IDENTITY…` through `09_DISTILLED_CLONE_SYSTEM_PROMPT.md`, plus a
   20-task comparison matrix + README; the v1/v2 genomes stay live under `legacy/`.
2. **The day-one sovereign treasury on Ethereum Base (`Chain ID: 8453`)** — four revenue
   streams with `Precise` zero-sum atomic splitting: a 10% charter tax, a 10%–50% affiliate
   watch with the "direct no-referrer sale" rule (2000 bps platform / 8000 bps creator),
   100% margin boosts, and a 1.5% withdrawal spread.
3. **Two Solidity contracts (`^0.8.24`)** — `ZeroXKiaToken` (a fixed supply of 1 billion +
   EIP-2612 permit + burn, a clean 100/100 GoPlus/TokenSniffer architecture) and
   `KiaBaseEscrowTreasury` (non-custodial escrow + an atomic splitter + a non-reentrant lock).

The kernel contract is preserved: zero `unsafe`, zero `Clippy` warnings (`-D warnings`),
full determinism, and zero floats on money — every leg is proven with the Wave 6 `Precise` engine.

## 2. The Genome Matrix — the Complete `omni-genome-v4` Map

| # | Model | Folder | Sovereign specialty | Trinity avatar |
| --- | --- | --- | --- | --- |
| 01 | `GPT 6-Web` | `models/01-gpt-6-web/` | Real-time web synthesis, frame-1 artifacts (390px zero-overflow), fast full-stack product | `Alpha` |
| 02 | `GEMINI 4 ARAGON HIGH` | `models/02-gemini-4-aragon-high/` | 1M-token cross-repo synthesis, the 9-modality orchestrator, macro-architecture validation | `Logic-Prime` |
| 03 | `DeepSeek-3.2-WEB-Think-mode` | `models/03-deepseek-3.2-web-think-mode/` | Quantitative `<think>` mathematics, causal CCXT/Vibe alpha proofs, formal verification | `Mairon` |
| 04 | `Cluade-sonnet-5.5-extra` | `models/04-claude-sonnet-5.5-extra/` | Rust surgery, zero pedantic warnings, contract audits, bilingual copy | `Logic-Prime` |

Routing rules (`QuadTaskDomain → champion`): `web-synthesis`/`ui-ux-artifact`
→ 01 · `cross-repo-context` → 02 · `quant-proof` → 03 ·
`rust-systems-audit`/`bilingual-copywriting` → 04. Alias resolution is truthful:
`Cluade-sonnet-5.5-extra` and `Claude-sonnet-5.5-extra` both map to model 04, and
`gpt-6-standard-free` maps to 01 (compatibility with the inherited genome).

## 3. The Day-One Charter — "First War in Exchange for Last Peace"

| Stream | Rate | Destination | Rule |
| --- | --- | --- | --- |
| 1 — Charter tax | `1000 bps` (10%) | `system:kia_sovereign_treasury` | On every marketplace sale |
| 2 — Affiliate watch | `1000..=5000 bps` | `user:{ref_id}` | Default watch `1000`; with no referrer, the 10% watch goes **straight to the sovereign treasury** → platform `2000 bps`, creator `8000 bps` |
| 3 — Instant boost | `10000 bps` (100%) | `system:kia_sovereign_treasury` | `FeaturedSlot24h` (250 XTR / $5 USDC / 5M µKIAN) · `MaironFastTrackAudit` (100 XTR / $2 / 2M) · `ViralSwarmAmplifier` (500 XTR / $10 / 10M) |
| 4 — Withdrawal spread | `150 bps` (1.5%) | `system:kia_sovereign_treasury` | Instant settlement and cross-chain withdrawal |

`compute_day_one_atomic_split` produces four double-entry legs
(`buyer_debit + kia_treasury + affiliate + creator == 0`) — the atomic remainder settles
onto the creator, and the zero-sum proof is re-checked with `Precise`; an unbalanced split
never escapes, and an unbalanced row is rejected at the db layer (`DbError::Corrupt`).

## 4. The Base `8453` Contracts

* **`ZeroXKiaToken.sol`** — an ERC-20 with a fixed supply of `1,000,000,000 × 10^18`,
  minted once in the constructor to the deployer (the sovereign treasury); no
  post-constructor mint, no blacklist, no pause, no mutable tax; EIP-2612 `permit` with
  `DOMAIN_SEPARATOR` and user `burn` — targeting 100/100 on GoPlus and TokenSniffer.
* **`KiaBaseEscrowTreasury.sol`** — a closed asset whitelist (native ETH,
  native USDC `0x8335…2913`, `$0xKIA`), `settleMarketplaceSale` (gross cash from the
  buyer → atomic legs → internal ledger), `purchaseInstantBoost` (100% to the treasury),
  and `withdrawWithSpread` (a 150 bps spread); all with a non-reentrant lock,
  effects-then-interactions, and an immutable treasury; direct `receive()` is rejected.
* **EIP-681** — `build_base_eip681_payment_uri` for one-click Trust/Coinbase/MetaMask
  links with a mandatory `@8453`; non-on-chain assets
  (`XTR`, `µKIAN`) return `NotOnChainSettleable` — no fake URI is ever produced.

## 5. The Monopoly Orchestrator and the `/api/v1/monopoly/*` Wire

`SovereignMonopolyOrchestrator` holds the genome router + charter + Base descriptors +
the launch kit (`ProductHuntCmcLaunchKit`) in a single state. Three additive routes:

* `GET /monopoly/genome-matrix` — 4 models with 9-file verification status (target 36) +
  6 routing rules; disk verification is fail-closed with a builtin fallback.
* `GET /monopoly/charter-and-base` — the four charter streams + Base configuration +
  the audit of both contracts.
* `POST /monopoly/simulate-split` — runs the split, persists it in
  `sovereign_treasury_splits` (migration `0011`), and returns the zero-sum proof + EIP-681.

## 6. The Wave 7 Acceptance Gate

| Gate | Status |
| --- | --- |
| All 36 genome files complete and above threshold | ✅ `models/{01..04}/` |
| Charter: referred sale 10/10/80 and no-referrer 20/0/80 (exact µUSDC) | ✅ deterministic test |
| Boosts at 100% and a lossless 150 bps spread | ✅ |
| EIP-681 with `@8453` + rejection of XTR/µKIAN | ✅ |
| An unbalanced row is rejected at the db layer | ✅ |
| Contracts: no further mint / no blacklist / no pause / no mutable tax | ✅ |
| Zero `unsafe` + zero clippy warnings | ✅ |
| `KbEntry` → **331** (`kb:dna-16-quad-genome-base-treasury`) | ✅ |

*Wave 7 closes the four-wave line: DNA 12 through 16 each carry the
`[EXEC_WEIGHT: 1.00 | ACTIVE_PRIMARY]` tag, and no wave has ever been closed without the
Mairon gate.*
