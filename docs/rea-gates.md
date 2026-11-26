# REA Gates — the 10/10 Target

Each row: the REA filter (`audit-release-exposure`) → our countermeasure → the metric.

| REA filter | Countermeasure in KIAN | Metric |
|---|---|---|
| `javascript.source_maps` / `source_map_sources` | `productionBrowserSourceMaps: false` + CI policing of `.map` in dist | audit → not_observed |
| `text.credential_markers` | Keys only from env; a grep gate over the bundle | audit → not_observed |
| `native.debug_companions` / `native.pdb_path` | `[profile.release] strip="symbols"` | audit → not_observed |
| `rust.dependency_metadata` | No metadata leak; fat LTO + codegen-units=1 | audit → not_observed |
| `electron.*` (future desktop) | Electron fuse flips: OnlyLoadAppFromAsar, no RunAsNode/Inspect | audit → not_observed |
| `packaging.extractable_payload` | Irrelevant for web; asar + integrity for desktop | audit → not_observed/low |
| — (outside the audit) | `compare-static-and-observed-endpoints`: frontend static endpoints ↔ the OpenAPI contract | zero diff |
| — (outside the audit) | `analyze-web-bundle` before release | clean report |

**Gate execution:** `make rea-gate` or `node tools/rea/release-exposure.mjs <artifact>` — exit 3 = failure, exit 0 = 10/10.

## Step 1 — Executed (M1, 2026-11-20)

- credential_markers: grep over src+public → zero hits ✓
- source_maps: productionBrowserSourceMaps=false + no `.map` in the source ✓
- Headers: HSTS, nosniff, Referrer-Policy, Permissions-Policy ✓
- sw.js: GET only, same-origin only, no caching of /api ✓
- Offline browser test: reload with the network offline → the cached shell was served ✓
- Full audit-release-exposure on the build artifact: in M1-finish (after the standalone release)

## Step 4 — Executed (M4, 2026-11-21)

- X-KIAN-Key: a tri-state fail-closed guard (ADMIN_DISABLED/ADMIN_KEY_REQUIRED/INVALID_ADMIN_KEY) with
  constant-time comparison (subtle ct_eq) — the guard matrix is tested green on the wire ✓
- redaction: six deterministic patterns (private_key_hex/bot_token/pem_key/jwt/mnemonic/long_hex) — applied before
  any model slot sees the input; evidence: 14 kian-ai tests green ✓
- sandbox: Hermes skills can only execute registered handlers (no eval/exec/Command); an input/output
  shield with an ALLOW/REDACTED/BLOCKED verdict — 17 kian-hermes tests green ✓
- All 10 require_admin call sites wired in the admin handlers, plus the /admin/*, /a2a/*, and /mcp/invoke routes ✓
- No key literal in the admin panel source (the key lives only in the tab's sessionStorage) ✓
- audit-release-exposure --fail_on low: kernel=0 admin=0 web=0 — zero exposed rows ✓
- Report: docs/rea/step4-m4-gate-report.txt — exit 0

## Step 5 — Executed (M5 final, 2026-11-21)

- Governance: a HITL parliament bound to Article 7 — thresholds come only from `kian-domain`
  (a single source of truth); an affirmative human vote is mandatory for amounts ≥ 50,000 Đ; quorum/majority/
  reason are deterministic in `kian-nation` — 14 pure tests green ✓
- Execution only through the pure M2 domain path: `bill.release/bill.refund` with
  `approved_by_admin: true` or a two-sided treasury posting — no side path for money ✓
- Four chains: addresses are accepted only under the official chain checksum — Tron Base58Check,
  Solana 32 bytes, TON CRC16-XMODEM (big-endian), EIP-55 with pure keccak-256
  (reference vectors: empty/abc/200 bytes + 4 official EIP-55 addresses) ✓
- The canonical USDT contracts for all four chains were verified against official sources (tron.network, Solscan,
  Tonscan, PancakeSwap) and locked into constants ✓
- USDT deposits: UNIQUE(chain, txHash) — "the same hash is never minted twice"; minting goes through the
  Article 3 path at the `KIAN_USDT_TO_DIN` rate and only with admin ✓
- cargo test --workspace green (43 server tests including the full marketplace→bill→execution cycle) ✓
- cargo clippy --workspace --all-targets -- -D warnings: zero errors (clippy 1.100 compatible) ✓
- No credentials in the M5 source or the frontends ✓
- audit-release-exposure --fail_on low: kernel=0 admin=0 web=0 — zero exposed rows ✓
- Report: docs/rea/step5-m5-final-gate-report.txt — exit 0

## Step 6 — Executed (M6 Telegram bot, 2026-11-21)

- Pure core: `crates/bot` — an orchestrator with zero I/O (no reqwest/teloxide/net)
  and a per-chat rate ceiling with an injected clock; one alert per window (no spam amplification);
  message truncation at the character boundary (the Telegram 4096 cap) — 15 offline tests with FakePort ✓
- Single-path identity: initData is minted only via `build_bot_init_data` (contracts::security —
  the single point) and verified only via `verify_telegram_init_data`; the long-poll shell takes
  the same `POST /auth/telegram` path as the mini-app; the in-process webhook follows the same
  mint→verify pair; **no side path for identity** — no X-KIAN-Key in
  kian-bot, no new identity endpoint ✓
- Fail-closed webhook: `POST /bot/webhook` with the official
  `X-Telegram-Bot-Api-Secret-Token` header and constant-time comparison; without
  `KIAN_BOT_WEBHOOK_SECRET` → 503 `BOT_DISABLED`; absent/incorrect header → 401
  `BOT_UNAUTHORIZED`; the runtime boots only with both secrets (token + a secret ≥ 16) ✓
- Replies to Telegram go behind the `TelegramSender` port — production: the official Bot API with rustls;
  testing: `RecordingSender` with zero network; a non-text update → 200 `handled:false` with
  no reply at all (never answer the unknown) ✓
- Two shells, one parse: teloxide Update → parts → `BotUpdate::from_parts` — the same
  path as the webhook (`from_json`); the long-poll shell uses teloxide 0.17 with rustls (no
  openssl-system) and stops cleanly on Ctrl+C/StopToken ✓
- cargo test --workspace green (244 tests; 48 server tests including the guard matrix + the full
  offline webhook flow) ✓
- cargo clippy --workspace --all-targets -- -D warnings: zero errors ✓
- No credentials in the M6 source or the frontends ✓
- audit-release-exposure --fail_on low: kernel=0 admin=0 web=0 ✓
- Report: docs/rea/step6-m6-gate-report.txt — exit 0
