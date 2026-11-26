# CHANGELOG — KIAN Ecosystem

## v1.1.0-sovereign (2026-11-25) — the Sovereign Founder War Room release + the "Day 0 → Day 1" Real-Time Simulation Engine

Commit #30 — three legendary co-founders (Elon Musk, CZ, Jeff Bezos) work remotely 24/7 under the
chairmanship of `0xKIA`, commercializing the `KiaAgentX/xA` repository from "day zero, second 00:00:00" — with no
fake time jumps, a starting revenue of `$0.00`, and `$33.00` of real crypto seed capital.

### Added

* **The Rust engine** `crates/hermes/src/war_room_simulation_engine.rs` — `FounderPersona`,
  `OrganizationWalletLedger`, `SimulationTickEvent`, `WarRoomState` with `advance_single_tick`
  (a 3600-second anti-jump guard), `inject_chairman_directive` + `ChairmanChatEngine` (a
  3×7 in-persona bilingual response table), `ChairmanIntent::classify` (offline deterministic),
  `canonical_day0_scenario()` — the single source of truth for seed/tests/HTML — and
  `explain_zero_to_hundred_elif5`; **26 unit tests** over the causal explicitness of the 20/80 charter.
* **The standalone interactive desktop app** `tools/war-room/war_room.html` (output of
  `tools/war-room/render_war_room.py`) — four OLED glass quadrants (`#050508`),
  a live Persian (RTL) ↔ English (LTR) switch, Tehran/UTC/simulation clocks, the
  `Play 1x / Pause / Next Step / 2x / 5x / Reset` controls, three-partner chat with 24 bilingual ticks,
  the honest wallet + live chart + the EIP-681/XTR transaction feed, three tabs (animated ELI5, live
  terminal, social feed), and the chairman chat with 6 tactical chips + an optional live-LLM modal.
* **Migration `0014`** — `"FounderWarRoomTick"` + `"ChairmanDirectiveLog"` (additive,
  zero ledger touches) + the `crates/db/src/wave10.rs` persistence layer (module name kept as a stable file id).
* **Four endpoints** — `GET /war-room/state` · `POST /war-room/step` ·
  `POST /war-room/directive` · `GET /war-room/explain-simple` (+ a route-registration test).
* **Seed** — 24 canonical ticks on `kian-production.db` (the exact causal sum of `$9.55`
  day-one revenue on the `$33.00` seed ⇒ `$42.55`) + 2 sample directives; DNA #18 became historical
  and DNA #19 (`dna:wave10:founder-war-room-grand-ui` — a stable seed content id) became the active primary —
  `"KbEntry"` = **334**; **779 green workspace tests** (750→779).

## v1.1.0-final (2025-11-21) — Frontend Update: Standalone Mini-App + SDK

Closing two real frontend gaps: `packages/sdk` was empty and `apps/miniapp` had only a packaging
stub. Both now ship complete implementations, and all three apps build green.

### Added

* **`packages/sdk`** — the kernel's framework-free client: all M3 endpoints with wire types
  mirroring `crates/contracts` (`AuthResponse`, `MeResponse`, `ProductView`,
  `EscrowView`, `TonDepositView`, `EconomySummary`), the same error envelope (`ApiError`
  with `code`/`status`), both official `initData` paths (WebApp + browser fragment), and
  bearer-token storage. Every other client in the ecosystem builds from this single point.
* **`apps/miniapp` — the standalone Telegram mini-app** (previously just a package.json):
  - Dashboard: citizenship balance, the live national-economy summary (`/economy` public) with the
    "ledger balanced" badge, wallet/market shortcuts, the latest ledger rows
  - Wallet: TON Connect, deposit-proof registration (Article 3), deposit history, ledger rows
  - Market: atomic purchase with Article 8 escrow, product registration, my orders with counterparty actions
    (delivery by the seller only / release by the buyer only — 403 otherwise)
  - Fail-closed session, same flow as web: stored token → initData → honest guest
  - Mobile-first shell: a sticky header with a live balance + a bottom tab bar; an RTL royal theme with
    the Vazirmatn font; Telegram theme colors via `setHeaderColor`
  - Web REA gates: no browser sourcemaps, strict headers, standalone output
* **All three apps build green** — `apps/web` (5 pages + REA standalone cleanup),
  `apps/admin`, `apps/miniapp` (4 prerendered routes).

### Breaking Changes

* `@kian/sdk` now ships `exports` (`./src/index.ts` as raw source) — consumers must
  configure `transpilePackages` just like `apps/*`.
* The unused `apps/miniapp` dependencies (`zustand`, `@tanstack/react-query`)
  were removed; `lucide-react` and `tw-animate-css` were added.

## v1.0.0-final (2025-11-21) — The Final Release After Load Testing

The release after the load and capacity test (November 21, 2025 — a simulation of 2,502 virtual Telegram
customers with valid HMAC initData) and the full application of the report's recommended fixes.

### Performance Fixes (Verified by Retest)

| Metric | Before | After | Improvement |
|---|---|---|---|
| `/health` ceiling | 86 RPS (p50=1141ms) | **8,715 RPS** (p50=4.2ms) | ~101× |
| `/economy` ceiling | 98 RPS (p50=509ms) | **1,120 RPS** (p50=36ms) | ~11× |
| Real traffic vu=1000 | 364 RPS, p95=5.4s | **2,679 RPS, p95=0.62s, zero errors** | ~9× latency |
| Registration storm (auth/s) | 1,311 | **2,077** | ~1.6× (WAL effect) |
| Workspace tests | 244 passing | **248 passing** (+4 new tests) | — |

* **The aggregate SQL `economy_summary`** (`crates/db/src/store.rs`) — per-kind sums with
  a single `GROUP BY` query; eliminating the transfer of every account and the O(N²) merge in Rust. The cost of each
  call is now independent of the account count (it previously grew heavier, linearly with users).
* **The lightweight `/health`** (`handlers/system.rs` + `V3Store::ping`) — a health check with
  `SELECT 1` instead of a full ledger scan; the deep reconciliation read stays in `/economy`.
* **WAL + synchronous=NORMAL** (`V3Store::connect`) — concurrent reads while writing to the
  file-backed database; not applied on `:memory:`. Concurrent registration became ~60%
  faster.

### Correctness and Security Fixes

* **RateLimiter — smart eviction** (`crates/contracts/src/security.rs`): previously, once
  10,000 buckets were exceeded, `windows.clear()` dropped the entire state (a busy day = wholesale
  bypass of rate ceilings). Now only idle buckets (last hit outside the 60s window)
  are evicted — an active user is never reset.
* **ShardedLimiter** — the rate-limit lock was split with FNV-1a across 16 independent shards;
  the global mutex was a hotspot at high RPS. Per-key ceilings exactly as before.
* **`KIAN_TRUST_PROXY`** (default `true`): when `false`, the `X-Forwarded-For` header is
  ignored and the rate bucket is taken from the socket IP — in a direct deployment
  (no proxy) any client could bypass the ceiling with a forged XFF.
* **`.env.example` updated** — the three AI slots, trust proxy, kernel server.

### Recommended Production Configuration

```
KIAN_TRUST_PROXY=true          # Behind Railway / a trusted proxy
KIAN_AI_SECONDARY_URL/KEY/MODEL=...   # AI fallback — load testing showed the free upstream
                                      # becomes unstable under the slightest load (90% 502)
```

### Estimated Capacity (2 vCPU sandbox; conservative floor numbers)

* Smooth concurrent users (p95 < 1s, zero errors): **~1,000–1,500+** (previously ~120)
* DAU with a 5–10% peak: **~10–30 thousand** (previously ~1–2 thousand)
* Registration burst: 2,077 users/second — never the launch bottleneck
* Remaining bottlenecks (outside the code): the single-slot AI upstream (fill in the env),
  the Telegram outbound in webhooks (the Telegram API's own limit)
