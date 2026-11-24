# Product Hunt & CMC Launch Kit — `$0xKIA` on Base (`Chain ID: 8453`)

> `docs/launch/PRODUCT_HUNT_AND_CMC_KIT.md` — Wave 7 (Step 4/4 — FINAL WAVE)
> منبعِ واحدِ کپیِ لانچ — همان متن در `ProductHuntCmcLaunchKit::canonical()`
> (`crates/hermes/src/sovereign_monopoly_engine.rs`) نیز زندگی می‌کند.
> مجوز: MIT · زنجیره: Ethereum **Base Mainnet** · مالکِ پرونده: `system:kia_sovereign_treasury`

---

## 1. Product Hunt listing (ready-to-paste)

**Title**
```
0xKIA — the sovereign creator economy on Telegram + Base
```

**Tagline** (≤ 60 chars)
```
Ship AI-native products inside Telegram, settle on Base (8453), keep 80-90% of your revenue.
```

**Maker comment** (post as @maker immediately after launch)
```
0xKIA (KIAN v2) is a 25-commit Rust kernel: double-entry zero-sum ledger, 8-state
escrow, ARDA 20-locale engine, causal Vibe-Trading quant, and now a Day-1 4-stream
treasury on Base with audited Solidity. Every split is zero-sum-verified with a
decimal engine — no float ever touches money. Ask us anything — the whole kernel
is MIT-licensed and ships with its own AI genome matrix (4 models × 9 DNA files).
```

**First comment body** ( Maker follow-up )
```
Why we built it:
* Creators inside Telegram lose 30-50% to platforms by default. Our Day-1 charter
  ("جنگ اول به از صلح آخر" — the first war for the last peace) caps the platform
  take at 10% charter tax + a programmable 10-50% affiliate leg. Unreferred
  direct sales route that affiliate leg straight into the sovereign treasury —
  platform total 20%, creator 80%.
* Everything settles on Base (Chain ID 8453): native ETH, native USDC
  (0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913) and $0xKIA.
* The Rust kernel proves every split: buyer_debit + treasury + affiliate +
  creator == 0, verified with an i128 decimal engine (Precise), persisted to
  sovereign_treasury_splits, and exposed at GET/POST /api/v1/monopoly/*.
```

**6 gallery slide specs** (1280×800, 16:9, dark canvas `#0B0E14`, accent `#FF4D4D`, mono type)

| # | Slide | Visual | Copy on slide |
|---|---|---|---|
| 1 | Cover | `$0xKIA` glyph over Base-blue ring, chain-id badge `8453` | `The sovereign creator economy — Telegram × Base` |
| 2 | Day-1 charter | 4-stream waterfall (10% tax / 10-50% affiliate / 100% boosts / 1.5% spread) | `Four revenue streams. Zero floats. Zero-sum verified.` |
| 3 | Unreferred rule | 20/80 bar split animating to treasury | `No referrer? The 10% affiliate leg still funds the commons — creators keep 80%.` |
| 4 | Contracts | `ZeroXKiaToken.sol` + `KiaBaseEscrowTreasury.sol` cards, GoPlus 100/100 badge | `Fixed 1B supply. EIP-2612 permit. Non-custodial atomic escrow.` |
| 5 | Genome matrix | 4×9 grid lighting up 36 cells | `Four frontier AI genomes. Nine DNA files each. One sovereign fleet.` |
| 6 | CTA | QR to BaseScan token page + EIP-681 deep-link | `Pay in one tap on Base. Own your upside.` |

## 2. CoinMarketCap / CoinGecko verification dossier

| Field | Value |
|---|---|
| Token name | `0xKIA` |
| Symbol | `0xKIA` |
| Chain | Ethereum Base Mainnet (`Chain ID 8453`) |
| Contract | deploy `contracts/base/ZeroXKiaToken.sol` → paste address here before submission |
| Decimals | `18` |
| Total / max supply | `1,000,000,000` (fixed — minted once to treasury in constructor, no mint authority) |
| Circulating formula | `total_supply - treasury_vesting - burn` (publish treasury wallet address) |
| Explorer | `https://basescan.org` |
| Official site | repo README (`README.md`) |
| Whitepaper | `KIA-BRAIN-OS/16_QUAD_MODEL_GENOME_AND_BASE_TREASURY.md` + `docs/history/EVOLUTION_LINEAGE.md` |
| Source-code note | "Fully MIT open-source; token is a plain fixed-supply ERC-20 with EIP-2612 permit — no owner functions, no blacklist, no pause, no tax." |
| Self-report checklist | logo 200×200 PNG (transparent), ticker `0xKIA`, contract verified on BaseScan, tag `Base ecosystem` + `Telegram` |

**GoPlus / TokenSniffer clean-target checklist (100/100)**
- [x] No post-constructor mint / no mint function at all
- [x] No blacklist / no freeze / no pause / no kill switch
- [x] No transfer tax / no mutable fee hook
- [x] Owner cannot modify balances; ownership is not even a concept (no owner)
- [x] EIP-2612 `permit` (gasless approvals) + permissionless `burn`

**BaseScan verification steps**
1. `solc --version 0.8.24` single file, optimization 200 runs, license MIT.
2. Paste `contracts/base/ZeroXKiaToken.sol` verbatim (no flattening needed — single file).
3. Constructor args: none (treasury = `msg.sender`).
4. Match `CHAIN_ID == 8453` constant in the deployed bytecode view.

## 3. Viral announcement templates

**X / Twitter (thread opener)**
```
$0xKIA is live on @base — Chain ID 8453. 1B fixed supply, EIP-2612 permit, audited
treasury. Day-1 creators keep 80-90%. 0xKIA. https://basescan.org
```
Thread slots: ① charter math (10/10-50/100/1.5) ② unreferred 20/80 rule ③ EIP-681 one-tap deep-link demo ④ genome matrix gif (36 cells lighting up) ⑤ MIT repo link.

**Telegram announcement channel (FA mirror)**
```
۰xKIA روی Base (8453) لانچ شد — خزانهٔ چهارجریانیِ روزِ اول، امانتِ غیرحضانتی،
تقسیمِ صفرجمعِ اتمی. سازنده‌ها ۸۰-۹۰٪ می‌گیرند. #0xKIA
```
پیوست‌ها: اسلاید ۲ و ۴ از گالری + لینکِ BaseScan.

**Community bounty post**
```
Ship anything on 0xKIA this week and the Mairon fast-track audit (100 XTR) is on
the house — Stream 3 boosts are 100%-margin, meaning 100% of every boost funds
the sovereign treasury that pays for your audits.
```

## 4. Wire endpoints backing this kit

| Endpoint | Backs |
|---|---|
| `GET /api/v1/monopoly/genome-matrix` | slide 5 (36-cell verification, live) |
| `GET /api/v1/monopoly/charter-and-base` | slides 2-4 (charter + contract audit JSON) |
| `POST /api/v1/monopoly/simulate-split` | slide 3 demo (zero-sum proof string + EIP-681 URI) |
| `GET /api/v1/quant/venues` | press kit claim "8 keyless venues, zero KYC" |
