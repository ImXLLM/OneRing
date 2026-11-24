# KIAN — AI HTML Artifact Studio & Direct-Link Crypto Marketplace

[![Rust](https://img.shields.io/badge/Kernel-Rust%20%2F%20Axum%20%2F%20SQLite-orange)](#architecture)
[![Next.js](https://img.shields.io/badge/MiniApp-Next.js%2016%20%2F%20React%2019-black)](#workspace-layout)
[![Telegram](https://img.shields.io/badge/Platform-Telegram%20MiniApp-26A5E4)](#telegram-bot--miniapp-setup)
[![Railway](https://img.shields.io/badge/Deploy-Railway%20Ready-blueviolet)](#deploy-on-railway)

**KIAN** is a production-ready Telegram MiniApp and Rust financial kernel for creating, previewing, and selling standalone single-page HTML digital products (landing pages, micro-SaaS calculators, interactive resumes, toolkits, widgets) via direct crypto-settled links with built-in viral affiliate splits.

> **Interactive Setup Guide:** Open [`setup.html`](./setup.html) locally in any browser for a step-by-step Founder & Developer checklist, an in-browser `.env` + CSPRNG secret generator, and copy-paste Railway & Telegram BotFather commands.

---

## Key Capabilities

- **Multi-Turn AI HTML Builder Studio (`/builder`):**
  - 3-stage guided interview & synthesis workflow powered by `kian-hermes` + `kian-ai` (OpenAI-compatible 3-slot automatic failover gateway with token budgets & PII redaction, plus deterministic offline HTML fallback).
  - Real-time **sandboxed live preview** (`<iframe sandbox="allow-scripts">` — no `allow-same-origin`, no parent DOM/cookie access).
  - **One-click publish** directly into the Marketplace with custom title, price (`Đ`), description, and affiliate commission rate (`0%–50%`, default `10%`).
- **Direct-Link Crypto Marketplace (`/market`):**
  - Every product gets a shareable Telegram MiniApp deep link (`https://t.me/<bot>?startapp=prod_<id>_ref_<tg_id>`).
  - Buyers inspect live HTML previews before paying, fund atomic 8-state escrows in one click, unlock the full `deliveryPayload` HTML artifact immediately upon funding, copy/download the source, release funds, or raise an inline dispute.
- **3-Way Double-Entry Affiliate Split:**
  - Every escrow release posts a mathematically balanced 4-leg transaction (`trial_balance == 0`):
    - **Platform Treasury fee** (Charter Article 2)
    - **Referrer affiliate commission** (`0..5000 bps`, when referred by a distinct non-buyer/non-seller citizen)
    - **Seller net payout** (remainder)
- **Non-Custodial Multi-Chain Wallet (`/wallet`):**
  - **Deposits:** TON and USDT across **TON, TRON (TRC-20), Solana (SPL), and BNB Chain (BEP-20)** with `i128` overflow-safe conversion (`1..=MAX_DIN`) and atomic idempotent minting.
  - **Anti-Double-Spend Withdrawals:** Citizens request `TON` or `USDT` payouts to checksum-validated addresses (`kian-wallet`). Requested `Đ` is atomically burned/locked (`withdrawal:{id}:lock`) at submission—making double-spend and overdraft impossible. Admins confirm with on-chain `txHash` or reject with atomic ledger refund (`withdrawal:{id}:refund`) without consuming the yearly mint cap.
- **Constitutional Safety & Privacy:**
  - Fail-closed Telegram WebApp `initData` HMAC-SHA256 verification and CSPRNG Bearer sessions.
  - Owner-or-Admin privacy lockdown on `GET /api/v1/accounts/{id}`, `GET /api/v1/accounts/{id}/balance`, and `GET /api/v1/escrows/{id}`.
  - Automatic **Human-in-the-Loop (HITL)** queueing (`HitlRequest`) whenever a buyer release exceeds the Charter threshold (`50,000 Đ`).
  - SQLite WAL mode with `BEGIN IMMEDIATE` write serialization and idempotency keys across all financial mutations.

---

## Architecture

```text
┌──────────────────────────────┐        HTTPS / Bearer JWT        ┌────────────────────────────────────┐
│  Telegram MiniApp (Next.js)  │ ───────────────────────────────▶ │     KIAN Kernel (Rust / Axum)      │
│  apps/miniapp                │                                  │     crates/server (:8088)          │
│  • /builder  (AI Studio)     │ ◀─────────────────────────────── │  • Auth (Telegram HMAC + CSPRNG)   │
│  • /market   (Store + Links) │        JSON Wire Contracts       │  • Market, Escrows & HITL Queue    │
│  • /wallet   (TON/USDT)      │                                  │  • Multi-Chain Deposits/Withdraws  │
└──────────────────────────────┘                                  └───────┬────────────────────┬───────┘
                                                                          │                    │
                                                          ┌───────────────▼───────┐  ┌─────────▼─────────┐
                                                          │  Pure Domain & Ledger │  │  AI Gateway       │
                                                          │  kian-domain + SQLite │  │  kian-hermes +    │
                                                          │  (Double-Entry WAL)   │  │  kian-ai (3-slot) │
                                                          └───────────────────────┘  └───────────────────┘
```

---

## Workspace Layout

| Path | Role |
| :--- | :--- |
| `crates/domain` | Pure double-entry ledger (`trial_balance == 0`), 8-state escrow FSM, Charter caps, 3-way affiliate split, HITL threshold |
| `crates/db` | SQLite WAL store, `BEGIN IMMEDIATE` transactions, 5 migrations (`0001`–`0005`), builder sessions, withdrawals, HITL queue |
| `crates/contracts` | Rust JSON wire DTOs, OpenAPI schemas, Telegram HMAC verifier, rate limiter |
| `crates/ai` | OpenAI-compatible 3-slot failover gateway, daily token budget, redaction, `BUILDER_MAX_TOKENS` |
| `crates/hermes` | Multi-turn AI HTML interviewer, strict HTML/JSON extractor, sandboxed preview generator, offline fallback |
| `crates/wallet` | Non-custodial address validators (TON CRC16, TRON Base58Check, Solana Ed25519/Base58, BSC EIP-55) |
| `crates/nation` | Parliament proposals & HITL governance rules |
| `crates/agents` & `crates/bot` | Telegram bot command router & polling/webhook shell |
| `crates/server` | Production Axum HTTP kernel (`kian-server`, default `0.0.0.0:8088`) |
| `packages/contracts` | TypeScript Zod wire schemas (`@kian/contracts`) |
| `packages/sdk` | Framework-free TypeScript API client (`@kian/sdk`) |
| `packages/ui` | Shared RTL Persian/English UI primitives (`@kian/ui`) |
| `apps/miniapp` | Next.js 16 / React 19 / Tailwind 4 Telegram MiniApp (`/builder`, `/market`, `/wallet`) |
| `apps/web` | Public landing & dashboard web shell |

---

## Quick Start (Local Development)

### Prerequisites
- **Rust** toolchain (`rustup`, stable — edition 2024)
- **Node.js** `>= 20` and **pnpm** (`corepack enable`)
- **SQLite3** (optional, for inspecting `data/kian.sqlite`)

### 1. Configure Environment
```bash
cp .env.example .env
```
Generate strong secrets (minimum 16 characters, recommended 32 bytes hex):
```bash
openssl rand -hex 32   # → KIAN_ADMIN_KEY
openssl rand -hex 32   # → KIAN_BOT_WEBHOOK_SECRET
```
Fill in `KIAN_BOT_TOKEN` (from [@BotFather](https://t.me/BotFather)) and at least one AI slot (`KIAN_AI_PRIMARY_URL`, `KIAN_AI_PRIMARY_KEY`, `KIAN_AI_PRIMARY_MODEL`). If no AI key is provided, the Builder Studio cleanly falls back to deterministic offline HTML synthesis.

### 2. Run the Rust Kernel (`:8088`)
```bash
set -a && source .env && set +a
cargo run -p kian-server
```
Verify health:
```bash
curl http://127.0.0.1:8088/api/v1/health
```

### 3. Run the Next.js MiniApp (`:3000`)
```bash
pnpm install
NEXT_PUBLIC_KIAN_API=http://127.0.0.1:8088/api/v1 \
NEXT_PUBLIC_TELEGRAM_BOT_USERNAME=your_bot_username \
pnpm --filter @kian/miniapp dev
```

### 4. Run Test Suites
```bash
# Rust workspace unit & integration tests (ledger, affiliate split, builder, withdrawals, privacy)
cargo test --workspace

# TypeScript strict typecheck
pnpm -r typecheck
```

---

## Core API Reference (`/api/v1`)

| Method & Path | Auth | Purpose |
| :--- | :--- | :--- |
| `GET /health` · `GET /economy` | Public | Kernel liveness, trial balance invariant (`balanced: true`) |
| `POST /auth/telegram` · `GET /me` | Public / Bearer | Verify Telegram `initData` HMAC, issue CSPRNG session, fetch balance |
| `GET /accounts/{id}` · `GET /accounts/{id}/balance` | Admin or Owner | Account details & derived balance (privacy-locked) |
| `GET /escrows/{id}` | Admin or Party | Escrow details (`buyer`, `seller`, or `referrer` only) |
| `POST /builder/sessions` · `GET /builder/sessions` | Bearer | Start or list multi-turn AI HTML Builder sessions |
| `POST /builder/sessions/{id}/chat` | Bearer | Multi-turn interview & HTML artifact synthesis |
| `POST /builder/sessions/{id}/publish` | Bearer | One-click publish synthesized HTML artifact to Marketplace |
| `GET /market/products` · `GET /market/products/{id}` | Public | Browse catalog or direct-link product with `previewHtml` |
| `POST /market/purchase` | Bearer | Atomic `create_escrow` + `fund_escrow` (captures `referrer` from `start_param`) |
| `GET /market/escrows/{id}/delivery` | Bearer (Buyer) | Unlock full purchased `deliveryPayload` HTML source |
| `POST /market/escrows/{id}/release` | Bearer (Buyer) | Release escrow (4-leg affiliate split; auto-enqueues HITL if `>= 50,000 Đ`) |
| `POST /market/escrows/{id}/dispute` | Bearer (Buyer) | Freeze escrow funds for arbitration |
| `POST /wallet/ton-deposits` · `POST /wallet/usdt-deposits` | Bearer | Submit TON or multi-chain USDT deposit proofs |
| `POST /wallet/withdrawals` · `GET /wallet/withdrawals` | Bearer | Request `TON`/`USDT` withdrawal (atomically locks/burns `Đ`) |
| `GET /admin/withdrawals` | Admin (`X-KIAN-Key`) | Review pending/resolved withdrawals |
| `POST /wallet/withdrawals/{id}/confirm` | Admin (`X-KIAN-Key`) | Mark withdrawal paid with on-chain `txHash` |
| `POST /wallet/withdrawals/{id}/reject` | Admin (`X-KIAN-Key`) | Reject withdrawal and atomically refund `Đ` to citizen |

---

## Deploy on Railway

KIAN is designed for a clean **2-service Railway project** from a single GitHub repository:

### Service 1: `kian-kernel` (Rust API Server)
- **Builder:** Uses root [`railpack.json`](./railpack.json) automatically (or `infra/docker/Dockerfile.server`, which exposes port `8088`).
- **Persistent Volume:** Mount a Railway Volume at `/data` and set:
  - `KIAN_DATABASE_URL=/data/kian.sqlite`
  - `KIAN_BACKUP_DIR=/data/backups`
- **Networking:** Generate a public domain (e.g. `https://api.yourdomain.com`). Railway injects `PORT`; `kian-server` binds `0.0.0.0:$PORT` (fallback `8088`).
- **Healthcheck Path:** `/api/v1/health`
- **Required Variables:**
  - `KIAN_ADMIN_KEY`, `KIAN_BOT_TOKEN`, `KIAN_BOT_WEBHOOK_SECRET`
  - `KIAN_HOST=0.0.0.0`, `KIAN_TRUST_PROXY=true`
  - `KIAN_ALLOWED_ORIGINS=https://<your-miniapp-domain>`
  - `KIAN_MINIAPP_URL=https://<your-miniapp-domain>`
  - `KIAN_TON_TREASURY`, `KIAN_TREASURY_TRX`, `KIAN_TREASURY_SOL`, `KIAN_TREASURY_TON`, `KIAN_TREASURY_BSC`
  - `KIAN_AI_PRIMARY_URL`, `KIAN_AI_PRIMARY_KEY`, `KIAN_AI_PRIMARY_MODEL`

### Service 2: `kian-miniapp` (Next.js 16 Frontend)
- **Root Directory:** `/` (monorepo) — Build: `pnpm install --frozen-lockfile && pnpm --filter @kian/miniapp build` · Start: `pnpm --filter @kian/miniapp start`
- **Build & Runtime Variables:**
  - `NEXT_PUBLIC_KIAN_API=https://<your-kernel-domain>/api/v1`
  - `NEXT_PUBLIC_TELEGRAM_BOT_USERNAME=<your_bot_username_without_at>`

---

## Telegram Bot & MiniApp Setup

1. Open [@BotFather](https://t.me/BotFather) → `/newbot` → copy the token into `KIAN_BOT_TOKEN`.
2. Run `/newapp` (or **Bot Settings → Menu Button → Configure Menu Button**) and point the WebApp URL to your `kian-miniapp` HTTPS URL.
3. Register the webhook with your kernel secret:
   ```bash
   curl -X POST "https://api.telegram.org/bot${KIAN_BOT_TOKEN}/setWebhook" \
     -H "Content-Type: application/json" \
     -d "{
       \"url\": \"https://<your-kernel-domain>/api/v1/bot/webhook\",
       \"secret_token\": \"${KIAN_BOT_WEBHOOK_SECRET}\"
     }"
   ```
4. Share any Marketplace product using its generated affiliate link:
   `https://t.me/<BOT_USERNAME>?startapp=prod_<PRODUCT_ID>_ref_<TELEGRAM_ID>`

---

## Environment Variables Reference

See [`.env.example`](./.env.example) and [`railway.env.example`](./railway.env.example) for the complete, zero-dead-variable configuration list synced directly with `crates/server/src/config.rs`, `crates/ai/src/gateway.rs`, and `apps/miniapp`.

---

## Security Policy & Non-Custodial Guarantee

- **Zero private keys on server:** Treasury addresses (`KIAN_TREASURY_*`, `KIAN_TON_TREASURY`) are public deposit addresses only. Payouts are executed from your offline/hardware/multisig wallet and recorded via admin `POST /api/v1/wallet/withdrawals/{id}/confirm`.
- **Fail-closed defaults:** Missing `KIAN_ADMIN_KEY`, `KIAN_BOT_TOKEN`, or `KIAN_BOT_WEBHOOK_SECRET` disables the corresponding protected surface rather than falling back to insecure defaults.
- **Sandboxed HTML previews:** All untrusted buyer/seller HTML previews render inside `<iframe sandbox="allow-scripts">` strictly without `allow-same-origin`.

---

## License

MIT — see workspace manifests for details.
