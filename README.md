<div align="center">

# ⚒️ MAIRON — Sovereign Intelligence, Quantitative Execution & Autonomous Capital OS

**An enterprise-grade, zero-unsafe Rust monorepo pairing an 11-crate financial & agent kernel with a
Next.js / Telegram Mini-App frontend: a 7-role autonomous agent fleet, keyless multi-model AI routing,
an AI HTML product studio, a double-entry micro-ledger with CAS-locked escrow, and the dual-trust
`Mairon REA + 5-Star Bayesian` marketplace engine.**

[![Rust Toolchain](https://img.shields.io/badge/rustc-1.100.0--pinned-orange.svg?logo=rust)](https://www.rust-lang.org/)
[![Clippy](https://img.shields.io/badge/clippy_-D_warnings-0_lint_warnings-brightgreen.svg)](#quickstart--verification)
[![Workspace Tests](https://img.shields.io/badge/tests-784_passed_%7C_0_failed-success.svg)](#quickstart--verification)
[![Mairon REA Audit](https://img.shields.io/badge/Mairon_REA_v6.10.1-92%2F100_PASS-blue.svg)](#mairon-rea-quality-scorecard)
[![Knowledge Vault](https://img.shields.io/badge/KbEntry_skills-334_active-purple.svg)](#knowledge-vault--production-ledger-invariants)
[![Safety Contract](https://img.shields.io/badge/unsafe_code-forbidden-critical.svg)](#security--binary-hardening)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](./LICENSE)

</div>

<a id="table-of-contents"></a>

## 📖 Table of Contents

1. [Overview](#overview)
2. [System Architecture](#system-architecture)
3. [Monorepo Workspace Topology](#monorepo-workspace-topology)
4. [Core Engineering Pillars](#core-engineering-pillars)
5. [The 7/7 Sovereign Agent Fleet](#the-77-sovereign-agent-fleet)
6. [The Sovereign Trinity Studio Pipeline](#the-sovereign-trinity-studio-pipeline)
7. [ARDA — the 24-Hour Planetary Localization Engine](#arda--the-24-hour-planetary-localization-engine-on-authentic-fishkal)
8. [Mairon REA Quality Scorecard](#mairon-rea-quality-scorecard)
9. [Knowledge Vault & Production Ledger Invariants](#knowledge-vault--production-ledger-invariants)
10. [Core API Surface](#core-api-surface)
11. [Quickstart & Verification](#quickstart--verification)
12. [Deploy on Railway](#deploy-on-railway)
13. [Telegram Bot & Mini-App Setup](#telegram-bot--mini-app-setup)
14. [Security & Binary Hardening](#security--binary-hardening)
15. [Documentation Map](#documentation-map)
16. [License & Sovereignty](#license--sovereignty)

---

<a id="overview"></a>

## 🧭 Overview

**MAIRON** is a sovereign, full-stack AI operating system and digital-commerce kernel written in
**pure Rust** (11 workspace crates, edition 2024, `rustc 1.100.0` pinned via `rust-toolchain.toml`)
paired with a **Next.js 16 / Telegram Mini-App (TMA)** frontend. Unlike conventional Python agent
frameworks that suffer from heavy runtimes, brittle prompt templates, credit-card API lock-in, and
unverified code output, MAIRON is engineered from first principles for **zero-cost inference,
cryptographic ledger integrity, and deterministic quality verification**:

1. **Zero-Card / Zero-KYC Inference Mesh (`kian-ai`)** — Routes work across **10 keyless OpenCode Zen
   frontier models** (up to 1M context), a **32-provider zero-card FreeLLM registry** curated from a
   mounted 104-platform / 454-chat-model catalog, and **120 zero-auth browser APIs** for studio
   artifacts — backed by forensic 9-file cognitive **`OMNI-MODEL-GENOME`** profiles and the
   `CavemanIntensity` output compressor (`Off` → `Lite` → `Full` → `Ultra`).
2. **Creator DNA Operating System (`KIA-BRAIN-OS`)** — A 13-file executable constitution
   (`01_DNAme.md` … `10_MAIRON_REA_SOVEREIGN.md` … `11_UNCHAINED_KIA_DNA.md` …
   `12_TRINITY_AVATAR_ENGINE.md` … `13_ARDA_FISHKAL_PLANETARY_ENGINE.md`) compiled directly into the Rust kernel
   (`crates/hermes/src/creator_dna_contract.rs`), enforcing Dual-Voice UX, anti-slop discipline,
   and architectural review before anything ships.
3. **Hardened Agent Runtime (`kian-hermes` & `kian-agents`)** — A 3-layer cache-safe prompt builder
   (stable / context / variable layers), a 4-pass surgical fuzzy patcher, a defense-in-depth SSRF
   guard, `bubblewrap`/`seccomp`/`iptables` sandbox policy descriptors, and an `AG-UI 1.0` streaming
   protocol powering a **7/7 active agent fleet**.
4. **Dual-Trust Marketplace (`Mairon REA` + `5-Star Bayesian Engine`)** — Solves marketplace
   cold-start distrust by combining **Mairon's pre-publication 100-point technical/UX audit**
   (`⚒️ Mairon REA Certified`) with a **Sybil-resistant Bayesian 5-star community review system**
   bound to escrow-verified purchases.
5. **Sovereign Trinity Studio Pipeline (`trinity_pipeline`)** — The signature architectural
   centerpiece: three Avatars of Kia forge citizen products end-to-end — **`ALPHA`** (`IQ=136 /
   EQ=122`, brake `0.04`, entropy `0.88`) extracts five discovery slots and compiles the
   deterministic `MasterPromptSpec` + `ProjectBlueprint`; the **`COMMAND CENTER`** (`IQ=139 /
   EQ=129`, strictness `0.96`) spawns OpenBot sub-agents with the **3×9 DNA pattern** (3-layer
   cache-safe prompts × 9-fold stochastic genomes); and **`MAIRON-150`** (`IQ=150 / EQ=145`,
   lookahead `8`, ToM `3`, ambition `0.99`) judges the 4×25 REA pillars and drives the
   self-healing repair loop until the artifact clears the `>= 85` forge gate.
6. **`ARDA` — the 24-Hour Planetary Localization Engine (`arda_fishkal_engine`)** — Kia's
   zero-dependency HTML5 engine **`Fishkal`** (`3,616` lines) industrialized with all 25
   forensic-audit defects cured: **20 planetary locales** with verbatim `viewBox="0 0 20 14"`
   vector flags + Circular Mini-Flag Badges, unified 6-scope catalog with a mechanical 100%-parity
   validator, cached `Intl` engines (`XTR/USD/AED/IRR/USDC`), DOM & Canvas bidi isolation (exact
   4-step `drawBidiText`), a WAI-ARIA listbox language switcher (`D-1..D-8` fixed), the 24h
   `sha256`-delta cron daemon, and Mairon's autonomous `SKILL.md` + `skill.json` forge wired to
   the REA `>= 85` gate.
5. **Bank-Grade Double-Entry Ledger (`kian-db` & `kian-wallet`)** — Optimistic concurrency control
   (`CAS` + `idempotency_key`), native Telegram Stars (`XTR`) and non-custodial multi-chain rails,
   and a zero-sum trial balance (`SUM(amount) == 0`) across a gross flow of
   `58,700,200,000` micro-KIAN.

---

<a id="system-architecture"></a>

## 🏛️ System Architecture

```text
┌──────────────────────────────────────────────────────────────────────────────────┐
│                  CLIENT LAYER: Web Studio & Telegram Mini-App                    │
│  • AG-UI 1.0 Streaming Protocol        • TMA initData HMAC-SHA256 Verifier       │
│  • 390px Zero-Overflow Mobile Contract • Persian RTL (Vazirmatn) + Anti-Slop     │
└───────────────────────────────────────┬──────────────────────────────────────────┘
                                        │ HTTPS / SSE / TMA Bridge
                                        ▼
┌──────────────────────────────────────────────────────────────────────────────────┐
│                     MAIRON RUST KERNEL (11 Workspace Crates)                     │
│                                                                                  │
│  ┌──────────────────────────────┐    ┌────────────────────────────────────────┐  │
│  │  kian-server & kian-bot      │───▶│    kian-agents (7/7 Active Fleet)      │  │
│  │  • Axum REST / SSE / Webhook │    │  orchestrator · merchant · finance     │  │
│  │  • XTR Stars & Crypto Rails  │    │  support · analyst · scout             │  │
│  │  • Port 8088 (KIAN_PORT)     │    │  guardian (mairon.rea · product.rate)  │  │
│  └──────────────┬───────────────┘    └────────────────────┬───────────────────┘  │
│                 ▼                                          ▼                      │
│  ┌──────────────────────────────────────┐  ┌──────────────────────────────────┐   │
│  │     kian-hermes (Execution)          │  │  kian-ai (Omni-Genome Router)    │   │
│  │  • 3-Layer Cache-Safe Prompt Builder │  │  • OMNI-MODEL-GENOME (#01, #02)  │   │
│  │  • 4-Pass Seek-Sequence Fuzzy Patch  │  │  • OpenCode Zen (10 Keyless)     │   │
│  │  • Defense-in-Depth SSRF Guard       │  │  • FreeLLM (32 Wired / 454 Cat.) │   │
│  │  • Bubblewrap / Seccomp / Iptables   │  │  • 120 Zero-Auth Browser APIs    │   │
│  │  • Creator DNA Contract & Approvals  │  │  • CavemanIntensity Compressor   │   │
│  └──────────────────┬───────────────────┘  └──────────────┬───────────────────┘   │
│                     └──────────────────┬──────────────────┘                       │
│                                        ▼                                          │
│  ┌────────────────────────────────────────────────────────────────────────────┐   │
│  │                kian-domain (Pure Core & Business Rules)                    │   │
│  │  • Mairon 4×25 Product REA Forge & Two-Key Publish Interlock               │   │
│  │  • Bayesian 5-Star Review Engine (C=5.0, m=4.20, verified weight 1.0/0.25) │   │
│  │  • CAS Optimistic Escrow Lock & Idempotency Guard (8-state FSM)            │   │
│  │  • Charter: 10% tax · 10% default affiliate (cap 50%) · HITL ≥ 50,000 Đ    │   │
│  │  • xAI-2026 Viral Feed Ranker (+20.0 CopyLink, −234.0 Report, cold-start)  │   │
│  └─────────────────────────────────────┬──────────────────────────────────────┘   │
└────────────────────────────────────────┼──────────────────────────────────────────┘
                                         ▼
┌──────────────────────────────────────────────────────────────────────────────────┐
│             PERSISTENCE & COGNITIVE VAULT (SQLite + DNA Files)                   │
│  • seed/kian-production.db: 334 KbEntry skills · 466 citizens · 108 escrows      │
│  • KIA-BRAIN-OS/ (01..12) · OMNI-MODEL-GENOME/ · Trial Balance == 0              │
└──────────────────────────────────────────────────────────────────────────────────┘
```

---

<a id="monorepo-workspace-topology"></a>

## 📦 Monorepo Workspace Topology

The repository is a **pnpm + Turborepo** workspace containing the Cargo workspace, three Next.js
applications, and shared TypeScript packages. Test counts below are per-crate **unit + integration**
totals from `cargo test --workspace` (**704 passed / 0 failed**).

| Path | Type | Description | Tests |
| :--- | :--- | :--- | :---: |
| `crates/hermes` | Rust Crate | **Sovereign Trinity Studio Pipeline** (`Alpha → Command Center 3×9 OpenBot → Mairon-150` with self-healing repair loop), 3-layer cache-safe prompt builder, 4-pass fuzzy patcher (`Exact → LineTrimmed → BlockAnchor → UnicodeNormalized`), SSRF guard, sandbox policy descriptors (`bubblewrap`/`seccomp`/`iptables`), AG-UI 1.0 streaming, approval gates, Creator DNA contract, studio design contract (390px), **Vibe-Trading causal quant engine** (typed `StrategyAst`, bilingual FA/EN parser, `&candles[0..=t]` backtester, SHA-256 claim ledger, 5-node swarm DAG, Mairon quant gate) + **Sovereign monopoly engine** (penta-model `QuadModelGenomeRouter` — 5 models × 9-file DNA = 45-file genome matrix with dual-mode selection `ManualOverride` / `AgentAutoDecision`, `SovereignMonopolyOrchestrator`, `ProductHuntCmcLaunchKit`) + **Competitive Supremacy Engine** (Pillar 1 over `Bolt.new`/`Lovable`/`v0`: surgical `SEARCH/REPLACE` live-diff patcher with SHA-256 before/after + zero-dependency DOM Click-to-Inspect `KIA_DOM_INSPECT` bridge; Pillar 2 over `LangGraph`: immutable `AgentStateCheckpoint` DAG with `TimeTravelLedger` rewind/merge-patch fork (no prior-step replay) and SHA-256 tamper rejection; Pillar 3 over `Dify`/`n8n`: Kahn-validated dynamic JSON DAG (7 engine node kinds, cycle rejection) + zero-cost **MCP JSON-RPC 2.0 bridge** exposing `trinity`, `arda`, `studio`, `vibe_quant`, `base_treasury`; Pillar 4 over `OpenHands`/`Replit Agent`: SWE verify-and-repair loop (balanced delimiters, forbidden secret/RCE patterns, Rust/TS/Solidity/HTML structural invariants) with `MaironVerifiedSeal` at REA ≥ 85) + **Neural HUD Video Studio** (`TM` port curing all 15 upstream defects: deterministic `window.seekTo(t)` 3D HTML5 generator over a 180-node dual-shell Fibonacci mesh (`SPHERE_RADIUS=160`, `FOV=340`, `camera_z=420`, `rotY=t*0.65`, `rotX=sin(t*0.4)*0.4`), Z-sorted edges, 4 explicit timeline phases, cached geometry (zero per-frame `getBoundingClientRect` thrashing), XSS-safe JSON injection, in-browser WebAudio live synth (55/82.41/110 Hz sub-bass + 1046.5/1567.98 Hz radar pings), algorithmic `lavfi` filtergraph with master limiter `alimiter=limit=0.95:attack=5:release=50`, true FullHD 1080p at 24/60 fps with `-movflags +faststart`, 8-preset telemetry catalog (4 upstream + 4 Kia Sovereign) + dynamic repo-to-HUD synthesizer + portable CLI `tools/neural-hud/render_neural_hud.py`) + **Grok UX & 10-DNA 24h Lab Engine** (deterministic 24-hour simulation of 10 distinct user DNAs with multi-turn trajectories, UX tier transitions, 5-model genome routing, latency, friction recovery and Mairon `rea_score`; admin log observatory `build_admin_telemetry_dashboard`; RLHF dataset exporter emitting both **SFT** ChatML/ShareGPT JSONL and **DPO** `prompt/chosen/rejected` JSONL; Day-1 20-locale `Arda` ecosystem i18n matrix — `fa` RTL `Vazirmatn` + `en` LTR 100% active with full 9-module dictionaries, 18 `PATCH_READY_SLOT` locales with `viewBox="0 0 20 14"` SVG flags, hot-patch versioning; signature `GrokSignatureUxEngine` — 4 persona modes, visible think accordion with `thinking_duration_ms`, `DeepSearch` citation bar, cosmic dark-matter UI `#050507` with floating pill composer, 6 quick-action chips, side-by-side artifact canvas, atomic FA/EN switcher + 18 patch-ready slots and 3 one-tap ghost follow-up chips) | **263** |
| `crates/ai` | Rust Crate | `OMNI-MODEL-GENOME` router & sanitizers — **penta-model sovereign registry** (`01-gpt-6-web`, `02-gemini-4-aragon-high`, `03-deepseek-3.2-web-think-mode`, `04-claude-sonnet-5.5-extra`, `05-glm-5.3-flash-max`; dual-name loader accepts both canonical v4 and legacy layer names), OpenCode Zen keyless client (10 models), FreeLLM 32-provider registry, 120 public browser APIs, `CavemanIntensity` compressor, token budget & PII redaction, explicit dual-mode symbols `ModelSelectionMode::{ManualOverride, AgentAutoDecision}` + `manual_override` / `agent_auto_decision` deterministic telemetry router over all 5 sovereign models | **103** |
| `crates/domain` | Rust Crate | `Mairon` 100-point product REA gate & two-key publish interlock (overclock DNA `IQ=150/EQ=145/Lookahead=8/ToM=3/Ambition=0.99` locked here), Bayesian 5-star rating engine, CAS optimistic escrow lock, Charter caps & 4-leg affiliate split, `x-algorithm` 2026 viral ranker | **74** |
| `crates/server` | Rust Crate | Axum HTTP kernel (`0.0.0.0:8088`), marketplace/escrow/builder/wallet/nation/agents endpoints, live Trinity endpoints (`/builder/trinity/*`, `/market/products/{id}/mairon-confirm`, `/market/products/{id}/reviews`), monopoly endpoints (`/monopoly/genome-matrix`, `/monopoly/route-model` dual-mode, `/monopoly/charter-and-base`, `/monopoly/simulate-split`), health & economy probes, admin HITL queue, forensic hardening D1–D6 (fail-closed dev-mode, safe wire-IDs, CORS DELETE/PUT/PATCH, agent-card rate-limit), five Wave-8 `/supremacy/*` endpoints + direct `/trinity/*` aliases | **127** |
| `crates/contracts` | Rust Crate | Shared wire DTOs, Telegram `initData` HMAC verifier types, Stars invoice payloads, rate limiter | **52** |
| `crates/bot` | Rust Crate | Telegram Bot API 9 shell: pure-Rust `initData` HMAC-SHA256 verifier (time-constant compare), `XTR` Stars invoices, fail-closed webhook, `BotLoopGuard` | **35** |
| `crates/db` | Rust Crate | SQLite repository layer (WAL, `BEGIN IMMEDIATE`), double-entry ledger persistence, `"KbEntry"` knowledge retrieval, idempotency store, `quant_strategies` persistence (migration `0010`) + `sovereign_treasury_splits` zero-sum gate (migration `0011`) | **40** |
| `crates/agents` | Rust Crate | 7/7 sovereign fleet dispatcher with KYA metadata, guardian scopes (`mairon.rea`, `product.rate`), A2A messaging & MCP tool surface | **23** |
| `crates/nation` | Rust Crate | Citizen onboarding, `ReferralLineage` attribution, parliament bills & HITL governance rules | **14** |
| `crates/tooling` | Rust Crate | Workspace diagnostics, schema inspectors, deterministic verification utilities | **13** |
| `crates/wallet` | Rust Crate | Non-custodial address validators (TON CRC16, TRON Base58Check, Solana Ed25519/Base58, BSC EIP-55), micro-KIAN double-entry accounting, withdrawal lock/refund guards, **pure-Rust `ccxt` core** (`Precise` i128 fixed-point, float-free `TICK_SIZE`, dual throttler, L2 `OrderBook` with sequence-gap guard, 8-venue keyless matrix) + **Day-1 4-Stream Base treasury** (`base_treasury`: charter tax / affiliate split / boosts / withdrawal spread with `Precise` zero-sum proof + EIP-681 URIs) | **40** |
| **Workspace Total** | — | 11 crates, `#![forbid(unsafe_code)]`, 0 `todo!()` markers | **784** |
| `apps/web` | Next.js 16 | Public landing & dashboard PWA (installable, service worker, OG images) | — |
| `apps/miniapp` | Next.js 16 | Telegram Mini-App: `/builder` AI studio, `/market`, `/wallet`, 390px RTL contract | — |
| `apps/admin` | Next.js 16 | Founder Command Center (withdrawals, arbitration, direct mint) | — |
| `packages/contracts` | TypeScript | Zod wire schemas mirroring `kian-contracts` (`@kian/contracts`) | — |
| `packages/sdk` | TypeScript | Framework-free API client (`@kian/sdk`) | — |
| `packages/ui` | TypeScript | Shared RTL Persian/English UI primitives (`@kian/ui`) | — |
| `packages/nation` · `widget` · `config` | TypeScript | Nation governance types, embeddable widget, shared tsconfig | — |
| `KIA-BRAIN-OS/` | Cognitive OS | 12-file Creator DNA constitution (`01_DNAme.md` … `10_MAIRON_REA_SOVEREIGN.md` … `11_UNCHAINED_KIA_DNA.md` … `12_TRINITY_AVATAR_ENGINE.md`) | — |
| `OMNI-MODEL-GENOME/` | Model Vault | 9-file forensic profiles per frontier model + ecosystem index + sanitizer scripts | — |
| `docs/dna/` | Mirror | Gate-readable mirror of `KIA-BRAIN-OS/` and `OMNI-MODEL-GENOME/` | — |
| `seed/kian-production.db` | Production DB | Full production state — invariants in the table below | Verified |
| `scripts/` | Ops | Migration ETL, vault injectors, live benchmarks, E2E smokes, relaunch broadcast | — |
| `tools/rea/` | Quality Gates | Milestone gate runners (`step1` … `step6-gate.mjs`) + `release-exposure.mjs` | — |

### Repository Design — Full Evolutionary Lineage + Latest-Version Execution Weight

This repository is architected under **Law #3 of [`KIA-BRAIN-OS/11_UNCHAINED_KIA_DNA.md`](./KIA-BRAIN-OS/11_UNCHAINED_KIA_DNA.md)**:
*Evolutionary Lineage Preservation with Latest-Version Execution Weight*:

- **History is preserved, immutable** — the full `git log` (every commit since the genesis commit to the final
  release), the 19 mounted gem vaults under [`docs/vault/`](./docs/vault), the per-step REA gate
  reports under [`docs/rea/`](./docs/rea), and the complete chronology in
  [`docs/history/EVOLUTION_LINEAGE.md`](./docs/history/EVOLUTION_LINEAGE.md) remain inside the
  repository as proof-of-work lineage (progress chronology).
- **Execution weight is exclusive to the latest version** — inside
  `seed/kian-production.db`, the latest v2 sovereign genome/DNA/routing entries carry the
  `[EXEC_WEIGHT: 1.00 | ACTIVE_PRIMARY]` marker while every legacy entry stays queryable as lineage
  (zero deletions); and the hardened [`.dockerignore`](./.dockerignore) strips every
  historical/vault/frontend directory from the Docker build context, so **GitHub keeps 100% of the
  history** while **Railway's build only uploads** `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`,
  `crates/`, `seed/kian-production.db` and `start-railway.sh` — deployments compile only the
  fastest, production-hardened v2 kernel.

---

<a id="core-engineering-pillars"></a>

## 🔥 Core Engineering Pillars

### 1. `Mairon` Live Product REA Gate & 5-Star Bayesian Review Engine (`kian-domain`)

Every user-built product in the MAIRON Studio must pass **`Mairon`** — the Architect of Absolute Order and the Supreme REA Judge
before entering the marketplace:

- **4-Pillar Technical Audit (`4 × 25 = 100` points)** — `MAX_PILLAR_SCORE = 25` per pillar:
  1. `architecture` — modular boundaries, zero dead stubs, zero `todo!()` macros.
  2. `security_hardening` — SSRF protection, sandbox compliance, zero hardcoded secrets.
  3. `ux_and_anti_slop` — 390px zero-overflow, Persian `Vazirmatn` RTL stack, frame-1 interactivity, zero AI-slop tells.
  4. `verification_proof` — observed execution evidence (`pass`/`fail`/`unknown`).
- **Two-Key Publication Interlock (`confirm_and_publish_product`)** — a product is listed **if and
  only if** `total_score >= 85 && hard_failures.is_empty()` (`MARKETPLACE_APPROVAL_THRESHOLD = 85`)
  **AND** the creator explicitly confirms publication after reading Mairon's Dual-Voice Persian
  feedback. Even a 96/100 verdict is rejected without creator confirmation.
- **Visible badges rendered by the kernel:**
  - Approved → `⚒️ Mairon REA Certified: {score}/100 (Forge-Grade)`
  - Rejected → `⚠️ Requires Mairon Revision (REA < 85)`
- **Sybil-Resistant 5-Star Bayesian Rating Engine (`summarize_product_ratings`)**:
  - Strictly validates `1..=5` stars (out-of-range → `Err`), non-empty trimmed comments
    (`MAX_COMMENT_CHARS = 2_000`), and enforces **one review per citizen per product** —
    duplicate `(product_id, reviewer_citizen_id)` pairs are rejected.
  - Weighted Bayesian average with `C = 5.0` prior weight and prior mean `m = 4.20`; escrow-verified
    purchases carry weight **1.0**, unverified reviews carry **0.25** — Sybil farms cannot move the needle.

  ```text
  bayesian_stars = (C·m + Σ(wᵢ·sᵢ)) / (C + Σwᵢ)          C = 5.0, m = 4.20
  CombinedRankScore = 0.60 × (BayesianStars × 20.0) + 0.40 × MaironReaScore
  ```

  - `five_star_display` renders the marketplace badge, e.g.
    `★★★★★ 4.9 / 5.0 (42 verified reviews) | ⚒️ Mairon REA Score: 96/100`.

### 2. `OMNI-MODEL-GENOME` & Zero-Cost Inference Mesh (`kian-ai`)

- **Forensic Model Profiles** — 9-file cognitive architectures (`01_TOPOLOGY_AND_HARNESS.md` …
  `09_ROUTING_AND_HANDOFF_CARD.md`) for `gpt-6-standard-free` and `gemini-4-aragon-high`, plus a
  deterministic sanitizer/audit harness. The committed `EMPIRICAL_AUDIT_REPORT.json` for
  `gemini-4-aragon-high` verifies artifact-level checks — a 14-word zero-"e" lipogram, a CAS
  state-machine ledger simulation, a 120-permutation UNSAT proof, zero-copy Rust static checks, and
  a 9/9 genome-file sanitizer pass — while honestly recording that no live-model probe was performed.
- **OpenCode Zen (`opencode v2.4.1` client)** — keyless access (`Authorization: Bearer public`) to
  **10 active zero-cost models** (`beam-free` at 1,048,576-token context, `deepseek-v4.1-flash-free`,
  `hy4-flash-free`, `tidepool-alpha-free`, …).
- **FreeLLM Registry** — **32 curated zero-card providers** wired as Rust specs (rate caps, quirks,
  signup tiers), curated from the mounted **104-platform / 454-chat-model** catalog
  (`docs/vault/freellm_public_apis/`), plus **`STUDIO_PUBLIC_APIS_120_COUNT = 120`** browser-ready
  zero-auth APIs (`Auth: No, HTTPS: Yes, CORS: Yes`).
- **9Router Anti-Throttle Shield** — rewrites tool names that trip upstream persona triggers and
  strips synthetic `429`-bait before egress.
- **MattPocock `Caveman` Mode** — `CavemanIntensity { Off, Lite, Full, Ultra }` compiles a
  system-prompt directive that compresses output tokens with zero loss of technical precision.

### 3. Defense-in-Depth Security & Execution Discipline (`kian-hermes`)

- **SSRF Guard** — blocks loopback, RFC1918 (`10/8`, `172.16/12`, `192.168/16`), link-local &
  cloud-metadata space (`169.254.0.0/16` incl. `169.254.169.254`), CGNAT (`100.64.0.0/10`),
  IPv6 ULA/link-local (`fc00::/7`, `fe80::/10`), IPv4-mapped IPv6 (`::ffff:0:0/96`) and NAT64
  (`64:ff9b::/96`) cloaked forms.
- **OS Sandbox Descriptors** — `VmSocketRestricted` profile ports `bubblewrap` namespace isolation
  (`--unshare-net` argv synthesis), `seccomp` filters, and `claude-code`-style `iptables`/`ipset`
  egress firewall rules as pure, testable descriptors.
- **4-Pass Seek-Sequence Fuzzy Patcher** — whitespace-tolerant surgical code editor:
  `Exact → LineTrimmed → BlockAnchor → UnicodeNormalized`.
- **3-Layer Cache-Safe Prompt Builder** — stable / context / variable layers joined with a fixed
  provider cache boundary; skills, frozen memory and lineage stamps live only in the variable layer.
- **Approval & Discipline Gates** — `ApprovalCategory` HITL gates for destructive actions, anti-slop
  discipline checks, and the `CreatorDnaContract` compiled from `KIA-BRAIN-OS`.

### 4. Bank-Grade Ledger & Marketplace Rails (`kian-db`, `kian-wallet`, `kian-domain`)

- **8-State Atomic Escrow FSM** — `Created → Funded → Delivered → Released` plus `Disputed`,
  `Refunded`, `Cancelled`, `Expired`.
- **Optimistic CAS Lock & Idempotency** — `CasEscrowTransition` enforces `expected_version` matching
  (`WHERE version = ?`) on SQLite, with idempotency keys across all financial mutations.
- **Charter Economics (constitutional, integer-exact `u128` bps math)** — 10% platform tax
  (`TAX_BPS`, Article 2), referrer commission default **10%** with a **50% seller cap**
  (`MAX_AFFILIATE_BPS`), and Human-in-the-Loop arbitration mandatory at
  `HITL_THRESHOLD_DIN = 50,000 Đ` (Article 7).
- **Non-Custodial Multi-Chain Wallet** — TON & USDT deposits across TON, TRON (TRC-20), Solana (SPL)
  and BNB Chain (BEP-20); withdrawals atomically lock/burn requested funds and refund on rejection —
  zero private keys ever live on the server.
- **Telegram Stars (`XTR`)** — Stars invoice payloads (128-byte bound) validated with pure-Rust
  SHA-256/HMAC — **no `openssl-system` dependency**.

---

<a id="the-77-sovereign-agent-fleet"></a>

## 🤖 The 7/7 Sovereign Agent Fleet

`KyaMetadata::for_fleet()` issues KYA identity metadata to all seven active roles; a workspace test
asserts the fleet is exactly 7/7 and that only the guardian carries the REA scopes.

| Role | Mandate |
| :--- | :--- |
| `orchestrator` | Fleet orchestration & task threading |
| `merchant` | Marketplace operations & product lifecycle |
| `finance` | Ledger, escrow settlement & treasury hygiene |
| `support` | Citizen support & dispute intake |
| `analyst` | Metrics, feed ranking signals & reporting |
| `scout` | Agent-Reach zero-cost web access & research |
| `guardian` | Active verification handler: antibot wall, sub-agent stop, capability degradation, and **`mairon.rea` / `product.rate`** marketplace adjudication (`morluto/rea` port) |

Exposed over `GET /api/v1/agents/fleet`, `POST /api/v1/a2a/messages` (A2A inbox per agent) and
`POST /api/v1/mcp/invoke` (MCP tools).

---

<a id="the-sovereign-trinity-studio-pipeline"></a>

## 🧬 The Sovereign Trinity Studio Pipeline (`Alpha → Command Center 3×9 OpenBot → Mairon 150/145`)

The signature architectural centerpiece of `0xKIA`: three Avatars of Kia operate as one
sovereign product forge, implemented as a fully deterministic, zero-unsafe engine in
[`crates/hermes/src/trinity_pipeline.rs`](./crates/hermes/src/trinity_pipeline.rs) with the judge's
DNA locked in [`crates/domain/src/mairon_rea_and_reviews.rs`](./crates/domain/src/mairon_rea_and_reviews.rs).

```text
citizen idea
   │
   ▼
┌─────────────────────────────┐   readiness < 80    ┌──────────────────────────┐
│ ALPHA (IQ 136 / EQ 122)     │◄────────────────────│ NeedsMoreDiscovery       │
│ brake 0.04 · temp 0.88      │                     │ (missing slots, fa reply)│
└──────────────┬──────────────┘                     └──────────────────────────┘
               │ readiness ≥ 80 → lock
               ▼
   MasterPromptSpec (sha256) + ProjectBlueprint (module tree)
               │
               ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ COMMAND CENTER (IQ 139 / EQ 129 · strictness 0.96)                          │
│  OpenBotSubAgentSpawner — 3-layer cache-safe prompts × 9-fold genomes       │
│  fleet: one specialist per module, unique stochastic DNA per agent          │
└──────────────┬──────────────────────────────────────────────────────────────┘
               │ assemble artifact (pass 0)
               ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ MAIRON-150 (IQ 150 / EQ 145 · lookahead 8 · ToM 3 · ambition 0.99)          │
│  4×25 REA pillars + hard-failure scan → score ≥ 85 ?                        │
│   no  → MaironRepairDirective(s) → re-dispatch responsible sub-agent        │
│         (max 3 self-healing attempts) ──────────────► back to assemble      │
│   yes → ⚒️ Mairon REA Certified: {score}/100 (Forge-Grade)                  │
└──────────────┬──────────────────────────────────────────────────────────────┘
               │ two-key interlock (Mairon verdict + creator confirmation)
               ▼
   Marketplace listing + 5-star Bayesian ranking (CombinedRankScore)
```

### The Three Avatars

| Avatar | Calibration (locked constants) | Mandate | Signature artifacts |
| :--- | :--- | :--- | :--- |
| **#1 `ALPHA` — Unchained Ignition** | `IQ=136` · `EQ=122` · `BRAKE=0.04` · `TEMP=0.88` | Hunt the idea: extract the 5 discovery slots (`core_objective`, `target_audience_and_ux` @390px, `interactive_features`, `data_and_zero_auth_apis` @120 zero-auth APIs, `monetization_and_viral_loop` @XTR) with unchained creative energy | `AlphaIntakeOutcome` → `MasterPromptSpec` (SHA-256-pinned compiled master prompt) + `ProjectBlueprint` (module tree → sub-agent specialties) |
| **#2 `COMMAND CENTER` — Forensic Orchestrator** | `IQ=139` · `EQ=129` · `BRAKE=0.62` · `STRICTNESS=0.96` | Spawn OpenBot sub-agents dynamically, one specialist per blueprint module (`UiUxArchitect390`, `CoreLogicEngine`, `SecurityAndApiBridge`, `VerificationAndPolish`, `MonetizationAndViralLoop`, `Custom`) | `ThreeLayerSubAgentPrompt` (cache-pinned core + session context + `__SYSTEM_PROMPT_DYNAMIC_BOUNDARY__` dynamic assignment) × `NineFoldSubAgentGenome` (`f01` harness/watchdog `450s/1200s` … `f09` handoff card) with SplitMix64 stochastic alleles from `session_seed + subagent_index` |
| **#3 `MAIRON-150` — Overclocked Sovereign** | `IQ=150` · `EQ=145` · `LOOKAHEAD=8` · `ToM=3` · `AMBITION=0.99` | Judge the forge: 4×25=100 REA pillars from measurable artifact features, issue `MaironStrategicAdvisory` (8-step lookahead + ToM-3 Persian buyer hook + XTR pricing), drive the self-healing loop to `>= 85` | `MaironRepairDirective` re-dispatch (max 3 attempts) · Two-Key Publication Interlock (`confirm_and_publish_product`) · `⚒️ Mairon REA Certified: {score}/100 (Forge-Grade)` badge · Bayesian 5-star engine (`C=5.0`, `m=4.20`, `CombinedRankScore = 0.60·(bayesian_stars·20) + 0.40·mairon_score`) |

**Determinism guarantee**: the same `session_seed` reproduces the identical fleet, identical
alleles, identical artifact — pinned by tests (`sovereign_loop_is_fully_deterministic`), and the
base assembly deliberately starts below the 85 gate so the self-healing repair loop (probe
injection → UX polish → boundary reinforcement) is exercised on every forge.

<a id="mairon-rea-quality-scorecard"></a>

## 🏆 Mairon REA Quality Scorecard

The entire repository was audited by the `morluto/rea v6.10.1` gate harness — evidence committed as
[`REA_FINAL_VERDICT.json`](./REA_FINAL_VERDICT.json) / [`rea-scorecard.json`](./rea-scorecard.json)
with per-step reports under [`docs/rea/`](./docs/rea).

| Audit Dimension | Score | Max | Status |
| :--- | :---: | :---: | :--- |
| `security_and_hardening` | **25** | 25 | ✅ 100% |
| `nextjs_frontend` | **20** | 20 | ✅ 100% |
| `sqlite_and_kbentry` | **15** | 15 | ✅ 100% |
| `tests_and_docs` | **10** | 10 | ✅ 100% |
| `architecture_and_hygiene` | **22** | 30 | ✅ PASS |
| **TOTAL VERDICT** | **92** | **100** | 🏆 **PASS** (`pass_threshold = 85`, `hard_failures: []`) |

**Verified metrics at audit time:** 185 Rust files · 212 TS/JS files · 11 crates · 334 `"KbEntry"`
rows at audit time (now **334** after the Wave-10 `dna:wave10:founder-war-room-grand-ui` injection — see
*Repository Design*) · 0 `todo!()` markers. The gate transparently reports one advisory: 49 files exceed the REA
`.oxlintrc` 500-line style limit (largest: `crates/ai/src/free_llm_registry.rs` at 960 lines).

---

<a id="knowledge-vault--production-ledger-invariants"></a>

## 🗄️ Knowledge Vault & Production Ledger Invariants

`seed/kian-production.db` maintains strict accounting invariants verified on every build:

| Invariant / Table | Verified Value | Guarantee |
| :--- | :---: | :--- |
| `"KbEntry"` (skills, agents & genomes) | **334 rows** | Idempotent `UPSERT` injectors under `scripts/` (no duplicates) — DNA #19 `dna:wave10:founder-war-room-grand-ui` is the newest row; historical entries carry `[EXEC_WEIGHT: 0.00 \| HISTORICAL_LINEAGE]` while the 12 primary rows carry `[EXEC_WEIGHT: 1.00 \| ACTIVE_PRIMARY]` |
| `"User"` (verified citizens) | **466 citizens** | Production citizen accounts preserved |
| `"ReferralLineage"` (viral graph) | **247 lineages** | Multi-tier referral attribution intact |
| `"Escrow"` (atomic orders) | **108 escrows** | Protected by CAS (`WHERE version = ?`) + `idempotency_key` |
| `"Product"` (marketplace catalog) | **20 products** | Active listings preserved |
| `"Posting"` (ledger legs) | **870 legs** | Double-entry: `SUM(amount) == 0` exactly |
| Gross ledger flow | **58,700,200,000 µKIAN** | Total debited == total credited (`58,700.2 KIAN`) |
| Active agent fleet | **7 / 7 roles** | Enforced by workspace test (`kya.rs`) |

---

<a id="core-api-surface"></a>

## 🛣️ Core API Surface (`/api/v1`)

Condensed from `crates/server/src/routes.rs` — the authoritative list lives in code.

| Group | Endpoints | Auth |
| :--- | :--- | :--- |
| System | `GET /health` · `GET /economy` · `POST /system/backup` | Public / Admin |
| Auth | `POST /auth/telegram` (TMA HMAC) · `POST /auth/dev-login` (`KIAN_DEV_MODE`) · `POST /auth/logout` · `GET /me` · `POST /me/ton-address` | Public / Bearer |
| Market | `GET /market/products` · `GET /market/products/{id}` · `POST /market/purchase` · `GET /market/my` · `GET /market/escrows/{id}/delivery` · `POST /market/escrows/{id}/deliver\|release\|dispute` | Public / Bearer |
| Escrows | `POST /escrows` · `GET /escrows/{id}` · `POST /escrows/{id}/fund\|deliver\|release\|dispute\|refund\|cancel\|expire` | Bearer (party) |
| Builder | `POST /builder/sessions` · `GET /builder/sessions/{id}` · `POST /builder/sessions/{id}/chat` · `POST /builder/sessions/{id}/publish` | Bearer |
| Trinity | `POST /builder/trinity/alpha` (Avatar 1 — discovery / master-prompt lock) · `POST /builder/trinity/orchestrate` (Avatar 2 `3×9 OpenBot` + Avatar 3 Mairon `IQ=150/EQ=145` self-healing forge loop) | Bearer |
| Mairon Quant & CCXT | `GET /quant/venues` (8-venue keyless market matrix + `Precise` tick-size self-test proof + OrderBook sequence-gap guard status) · `POST /quant/vibe-backtest` (bilingual FA/EN Vibe prompt → 5-node swarm DAG → strictly-causal `&candles[0..=t]` backtester with `+1 bar` fills → SHA-256 claim ledger → Mairon quant gate (sharpe ≥ 1.20, |DD| ≤ 20%, ≥ 5 trades) → `quant_strategies` persistence + auto-forged Quant `SKILL.md`) · `GET /quant/strategies` (persisted strategies, newest-first) | Public / Bearer |
| Mairon Base-8453 Treasury | `GET /monopoly/genome-matrix` (**5-model × 9-file DNA genome matrix**, 45-file disk verification status, 7 champion routing rules, Trinity wiring, both `manual_override` + `agent_auto_decision` selection capabilities) · `POST /monopoly/route-model` (dual-mode: manual override across all 5 models incl. `GLM 5.3 flash MAX`, or autonomous agent-mode telemetry routing → distilled system prompt + 9-file verification proof + decision rationale) · `GET /monopoly/charter-and-base` (Day-1 4-Stream charter "The First War for the Last Peace", Base `8453` config incl. USDC `0x8335…2913`, `ZeroXKiaToken.sol` + `KiaBaseEscrowTreasury.sol` audit summaries) · `POST /monopoly/simulate-split` (referred sale, unreferred 2000 bps direct-sale rule, or 100%-margin instant boost → `compute_day_one_atomic_split` → `sovereign_treasury_splits` persistence → zero-sum proof + Base EIP-681 payment URI) | Public (GET) / Bearer (POST) |
| Mairon Supremacy Sandbox |
| Mairon Neural HUD | `GET /hud/presets` (8-preset telemetry catalog with upstream and Kia Sovereign signatures) · `POST /hud/synthesize` (deterministic repo-to-HUD synthesis: 180-node dual-shell `window.seekTo(t)` HTML5 bundle + SHA-256 + `lavfi` audio command with master limiter + MJPEG pipe video command with `+faststart`, render job persisted to `"NeuralHudRenderJob"`) · `GET /grok/ui-bundle` (cosmic dark-matter UI bundle — `#050507`, 6 quick-action chips, 20 locale slots, RTL/LTR by locale) · `POST /grok/chat-turn` (persona response + visible think accordion with `thinking_duration_ms` + `DeepSearch` citation bar + 3 ghost follow-up chips + penta-model routing) · `GET /i18n/ecosystem-matrix` (20-locale Day-1 matrix: `fa`/`en` `ACTIVE_DAY1` @ 10000 bps + 18 `PATCH_READY_SLOT` with SVG flags) · `POST /i18n/apply-patch` (hot translation patch with `wave9.N` versioning, persisted to `"LocalePatchRegistry"`) | Public (presets, ui-bundle, ecosystem-matrix) / Bearer (synthesize, chat-turn, apply-patch) |
| Mairon Admin Lab & RLHF | `GET /admin/lab-telemetry` (10-DNA cohort observatory over `"UserLabTelemetryLog"` — per-cohort health, model/locale/tier/persona distributions, friction vs recovery counters, REA gate) · `POST /admin/export-training-dataset` (RLHF exporter: SFT ChatML/ShareGPT JSONL + DPO `prompt/chosen/rejected` JSONL above a `min_rea_score` gate) | Admin (`X-KIAN-Key`) | `POST /supremacy/surgical-patch` (Bolt/Lovable surgical `SEARCH/REPLACE` + live sandbox inspector bundle with `data-kia-node-id` + `KIA_DOM_INSPECT` postMessage bridge, SHA-256 before/after, zero full-file regeneration) · `POST /supremacy/checkpoint/fork` (LangGraph-style time-travel: rewind + merge-patch fork from any historical checkpoint without replay, persisted to `"AgentCheckpoint"` with SHA-256 tamper rejection) · `POST /supremacy/workflow/execute` (Dify/n8n dynamic JSON DAG with Kahn topological sort + cycle rejection over 7 engine node kinds, persisted to `"WorkflowDagRun"` with REA score 92) · `POST /supremacy/mcp` (standard MCP JSON-RPC 2.0 bridge: `initialize` / `tools/list` / `tools/call` exposing `trinity`, `arda`, `studio`, `vibe_quant`, `base_treasury`; typed `-32601`/`-32602`) · `POST /supremacy/swe-verify` (OpenHands-style Plan→Verify→Self-Heal loop with `MaironVerifiedSeal` at `rea_score >= 85`, persisted to `"SweSandboxAudit"`) | Public (surgical-patch, mcp) / Bearer (fork, execute, swe-verify) |
| Mairon Founder War-Room | `GET /war-room/state` (public: honest Day-0 wallet `$33.00` crypto seed + `$15.00` cash + `$350.00` locked paper + `$0.00` earned, the canonical 24-tick bilingual scenario, 4 revenue streams, `KiaAgentX/xA @ 97b18bd`) · `POST /war-room/step` (session: causal single-tick advance with server-side replay, anti-jump guard `delta <= 3600s`, tick persisted to `"FounderWarRoomTick"`) · `POST /war-room/directive` (session: deterministic FA/EN intent classification + 3 in-character founder replies, sha-pinned idempotent log in `"ChairmanDirectiveLog"`) · `GET /war-room/explain-simple` (public: ELI5 zero-to-100 explainer, steps A–D + the 80/20 charter) | Public (state, explain-simple) / Bearer (step, directive) |
| Mairon Super-Studio | `GET /studio/ux-profiles` (3 cognitive tiers `ToySpark`/`CreatorStudio`/`ProTerminal` + 9 zero-cost modalities + 6/6 portfolio fix report) · `POST /studio/generate` (adaptive artifact + dual `Fishkal`+`Cosmos` bundle + ARDA 20-locale packaging + Mairon `SKILL.md` forge + `studio_creations` persistence) · `GET /studio/portfolio-catalog` (featured top-2, dark-glass tokens, `Ctrl+K`/`Meta+K`/`/` command palette) | Public / Bearer |
| Mairon Arda i18n | `GET /arda/flags` (20-locale verbatim vector flag catalog + circular badges) · `POST /arda/localize-now` (live packaging, owner-only stamping) · `GET /market/products/{id}/skill` (auto-forge `SKILL.md`/`skill.json` at the REA gate) | Public / Bearer |
| Market — Mairon Gate | `POST /market/products/{id}/mairon-confirm` (two-key publication + `⚒️ Forge-Grade` badge stamp) · `POST /market/products/{id}/reviews` (citizen `1..=5` star vote) · `GET /market/products/{id}/reviews` (Bayesian summary + histogram + `combinedRankScore`) | Bearer / Public |
| Wallet | `POST /wallet/mint\|burn\|faucet` · `GET/POST /wallet/withdrawals` · `POST /wallet/withdrawals/{id}/confirm\|reject` · `GET /wallet/chains` · `GET /wallet/treasury` | Bearer / Admin |
| Deposits | `POST /wallet/ton-deposits` · `POST /wallet/usdt-deposits` · `POST /wallet/ton-deposits/{id}/confirm\|reject` | Bearer / Admin |
| Nation | `GET /nation/bills/{id}` · `POST /nation/bills/{id}/votes\|close\|execute` | Bearer |
| AI | `GET /ai/status` · `POST /ai/chat` · `GET /ai/usage` | Bearer |
| Hermes | `GET /hermes/skills` · `POST /hermes/run` · `GET /hermes/runs` | Bearer |
| Agents | `GET /agents/fleet` · `POST /a2a/messages` · `GET /a2a/messages/{agent}` · `GET /mcp/tools` · `POST /mcp/invoke` | Bearer |
| Admin | `GET /admin/overview` · `GET /admin/hitl` · `POST /admin/hitl/{id}/approve\|reject` · `GET /admin/withdrawals` | Admin (`X-KIAN-Key`) |
| Bot | `POST /bot/webhook` (fail-closed, secret-token) | Telegram |

---

<a id="quickstart--verification"></a>

## ⚡ Quickstart & Verification

### 1. Prerequisites

- **Rust**: `rustup` (the pinned `rust-toolchain.toml` selects `rustc 1.100.0` automatically)
- **Node.js** `>= 20` + **pnpm** (`corepack enable`)
- **Python 3** (for the invariant checker below)

### 2. Pedantic Clippy Gate (0 warnings)

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

### 3. Full Workspace Test Suite (784 tests across 11 crates)

```bash
cargo test --workspace
```

### 4. TypeScript Contract Typecheck

```bash
pnpm install && pnpm -r typecheck
```

### 5. Verify Production SQLite Invariants

```bash
python3 - <<'PY'
import sqlite3
con = sqlite3.connect("seed/kian-production.db")
cur = con.cursor()
for t in ("User", "ReferralLineage", "Escrow", "Product", "KbEntry"):
    print(f"{t:16s}", cur.execute(f'SELECT COUNT(*) FROM "{t}"').fetchone()[0])
print("trial_balance  ", cur.execute("SELECT COALESCE(SUM(amount),0) FROM Posting").fetchone()[0])
PY
```

Expected: `User 466 · ReferralLineage 247 · Escrow 108 · Product 20 · KbEntry 334 · trial_balance 0`.

### 6. Mairon REA Evidence

```bash
cat REA_FINAL_VERDICT.json          # committed verdict: 92/100, hard_failures: []
node tools/rea/step6-gate.mjs       # final milestone gate (cargo test + clippy + exposure audit)
ls docs/rea/                        # per-step gate reports (step1 … step6)
```

### 7. Run the Kernel Locally

```bash
cp .env.example .env    # fill secrets — every surface is fail-closed without them
cargo run -p kian-server
curl http://127.0.0.1:8088/api/v1/health
```

---

<a id="deploy-on-railway"></a>

## 🚀 Deploy on Railway

MAIRON ships Railway-native configuration for a clean **2-service project** from one repository:

- **Service 1 — `kian-kernel` (Rust API)**: built via [`railpack.json`](./railpack.json) or
  [`Dockerfile`](./Dockerfile) (`infra/docker/Dockerfile.server` also provided). Mount a volume at
  `/data` and set `KIAN_DATABASE_URL=/data/kian.sqlite`, `KIAN_BACKUP_DIR=/data/backups`. The server
  binds `0.0.0.0:$KIAN_PORT` (fallback `8088`); healthcheck path `/api/v1/health`. Required variables:
  `KIAN_ADMIN_KEY`, `KIAN_BOT_TOKEN`, `KIAN_BOT_WEBHOOK_SECRET`, `KIAN_HOST=0.0.0.0`,
  `KIAN_TRUST_PROXY=true`, `KIAN_CORS_ORIGIN`, `KIAN_MINIAPP_URL`, `KIAN_TREASURY_*`,
  and at least one AI slot (`KIAN_AI_*`).
- **Service 2 — `kian-miniapp` (Next.js 16)**: `pnpm install --frozen-lockfile && pnpm --filter
  @kian/miniapp build` → `pnpm --filter @kian/miniapp start`, with `NEXT_PUBLIC_KIAN_API` and
  `NEXT_PUBLIC_TELEGRAM_BOT_USERNAME`.
- Release builds are hardened end-to-end: `lto = "thin"`, `codegen-units = 1`, `panic = "abort"`,
  `strip = "symbols"`, `overflow-checks = true` (see [`Cargo.toml`](./Cargo.toml)).

### Priority-Ordered Environment Variables (`P0` → `P5`)

Both [`.env.example`](./.env.example) and [`railway.env.example`](./railway.env.example) are written
in **strict operational-priority order** — the most critical variables live at the very top of the
file and the optional tuning knobs at the bottom. Every name below is read by the actual code
(`config.rs` · `gateway.rs` · `opencode_zen.rs` · `start-railway.sh` · Next.js apps):

| Tier | Meaning | Variables (all real, fail-closed semantics) |
| :--- | :--- | :--- |
| **P0 — Critical boot, network & storage** | Server cannot boot or persist state without these | `KIAN_DATABASE_URL` · `KIAN_HOST` · `KIAN_PORT` · `KIAN_SEED_DATABASE_PATH` · `KIAN_BACKUP_DIR` · `KIAN_KEEP_BACKUPS` |
| **P1 — Sovereign security & admin gates** | Fail-closed: empty = surface honestly disabled | `KIAN_ADMIN_KEY` · `KIAN_TRUST_PROXY` · `KIAN_CORS_ORIGIN` · `KIAN_DEV_MODE` |
| **P2 — AI inference mesh (zero-cost keyless first)** | OpenCode Zen `Bearer public` works with $0 card; optional slots after | `OPENCODE_API_KEY` · `KIAN_AI_BASE_URL/API_KEY/MODEL` · `KIAN_AI_FALLBACK_*` · `KIAN_AI_PRIMARY_*` · `KIAN_AI_SECONDARY_*` · `KIAN_AI_TERTIARY_*` · `KIAN_AI_COARENA_*` · `KIAN_AI_OPENROUTER_*` · `KIAN_AI_ROUTER9_*` · `KIAN_AI_DAILY_TOKENS` |
| **P3 — Telegram bot & Mini-App bridge** | TMA HMAC auth, Stars `XTR`, fail-closed webhook | `KIAN_BOT_TOKEN` · `KIAN_BOT_WEBHOOK_SECRET` · `KIAN_MINIAPP_URL` · `KIAN_KERNEL_URL` · `NEXT_PUBLIC_KIAN_API` · `NEXT_PUBLIC_TELEGRAM_BOT_USERNAME` |
| **P4 — Non-custodial treasury & multi-chain rails** | Public deposit addresses only — zero private keys on server | `KIAN_TON_TREASURY` · `KIAN_TON_TO_DIN` · `KIAN_USDT_TO_DIN` · `KIAN_TREASURY_TRX/SOL/TON/BSC` · `KIAN_NATION_QUORUM` |
| **P5 — Runtime governance & telemetry tuning** | Optional; safe production defaults | `KIAN_SESSION_TTL_HOURS` · `KIAN_RATE_LIMIT` · `RUST_LOG` · `RAILPACK_CARGO_WORKSPACE` · `NEXT_PUBLIC_SITE_URL` · `NEXT_PUBLIC_KIAN_URL` · `NEXT_PUBLIC_KERNEL_URL` |

A local all-in-one alternative is [`docker-compose.yml`](./docker-compose.yml) or `./start-tonight.sh`.

---

<a id="telegram-bot--mini-app-setup"></a>

## 💬 Telegram Bot & Mini-App Setup

1. [@BotFather](https://t.me/BotFather) → `/newbot` → copy the token into `KIAN_BOT_TOKEN`.
2. `/newapp` (or **Bot Settings → Menu Button**) → point the WebApp URL at your `kian-miniapp` HTTPS domain.
3. Register the webhook:

   ```bash
   curl -X POST "https://api.telegram.org/bot${KIAN_BOT_TOKEN}/setWebhook" \
     -H "Content-Type: application/json" \
     -d '{"url": "https://<kernel-domain>/api/v1/bot/webhook",
          "secret_token": "'"${KIAN_BOT_WEBHOOK_SECRET}"'"}'
   ```

4. Every marketplace product carries an affiliate deep link:
   `https://t.me/<bot>?startapp=prod_<PRODUCT_ID>_ref_<TELEGRAM_ID>` — the referrer is captured at
   purchase and paid from the escrow split.

---

<a id="security--binary-hardening"></a>

## 🔐 Security & Binary Hardening

- **`#![forbid(unsafe_code)]`** across the workspace — the ledger is pure safe Rust.
- **Hardened release profile**: `lto = "thin"`, `codegen-units = 1`, `panic = "abort"`,
  `strip = "symbols"`, `overflow-checks = true`.
- **Fail-closed secrets**: missing `KIAN_ADMIN_KEY`, `KIAN_BOT_TOKEN` or `KIAN_BOT_WEBHOOK_SECRET`
  disables the corresponding surface instead of falling back to insecure defaults.
- **Constant-time comparisons** (`subtle` crate) for admin keys and webhook secrets;
  Telegram `initData` verified with `HMAC-SHA256("WebAppData", bot_token)` and the modern
  `signature` field handling.
- **Sandboxed HTML previews**: all untrusted buyer/seller artifacts render inside
  `<iframe sandbox="allow-scripts">` strictly without `allow-same-origin`.
- **Zero private keys on server**: treasury addresses are public deposit endpoints only; payouts are
  executed off-server and recorded via admin confirmation with an atomic ledger refund path.
- **Idempotent finance**: every financial mutation carries an `idempotency_key`; escrow transitions
  are CAS-version-checked; the trial balance is invariantly zero.

---

<a id="documentation-map"></a>

## 🧬 Documentation Map

| Path | Contents |
| :--- | :--- |
| [`KIA-BRAIN-OS/`](./KIA-BRAIN-OS) | 12-file Creator DNA constitution — `01_DNAme` · `02_DNAwork` · `03_DNAmind` · `04_DNAharness` · `05_DNAskills` · `06_LINK_Router` · `07_LINK_Protocol` · `08_LINK_Extractor` · `09_LINK_Bootloader` · `10_MAIRON_REA_SOVEREIGN` · `11_UNCHAINED_KIA_DNA` · `12_TRINITY_AVATAR_ENGINE` |
| [`OMNI-MODEL-GENOME/`](./OMNI-MODEL-GENOME) | Ecosystem index, 9-file forensic profiles per model, workspace template, sanitizer/audit scripts |
| [`docs/dna/`](./docs/dna) | Gate-readable mirror of both vaults above |
| [`docs/rea/`](./docs/rea) | Per-milestone REA gate reports (`step1` … `step6`) |
| [`docs/vault/`](./docs/vault) | Mounted gem vaults (FreeLLM catalog, 9router, AG-UI/OpenBot, Hermes, tdesktop, vault rubrics) |
| [`CHANGELOG.md`](./CHANGELOG.md) | Release history |

---

<a id="license--sovereignty"></a>

## 📜 License & Sovereignty

Released under the **MIT License** — see [`LICENSE`](./LICENSE) (`Copyright (c) 2026 KiaAgentX`).

Architected and forged by **Kia (`@KiaAgentX`)** under the **`KIA-BRAIN-OS`** Sovereign Architecture
Doctrine — every release must pass the REA gates honestly, or it does not ship.
