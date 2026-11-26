# @kian/web — MAIRON Sovereign Web OS

The primary web application of the MAIRON platform: one authoritative world
state, five non-destructively switchable perspectives, and a Ctrl+K command
palette (`ProTerminal`) available everywhere.

- **P1 Cognitive Super-Studio** — `ToySpark` / `CreatorStudio` / `ProTerminal`
  morph modes, the Trinity 3-avatar pipeline (Alpha IQ 136 -> CommandCenter IQ
  139 -> Mairon IQ 150), the `REA >= 85` seal gate, the nine zero-cost
  modalities, and the dual Ocean/Cosmos WebAudio synthesizer.
- **P2 Quantitative Execution Desk** — causal `&candles[0..=t]` backtests over
  the eight keyless CCXT venues, the `Sharpe >= 1.20` live-gate badge, and the
  SHA-256 verifiable claim ledger.
- **P3 Base-8453 Treasury & Genome Router** — the four-stream Day-1 charter,
  the USDC `0x8335...2913` + Telegram Stars `XTR` settlement feed, the capital
  reserve partition ($33.00 / $15.00 / $350.00), and the 5 x 9 = 45
  Omni-Model Genome switcher (`ManualOverride` / `AgentAutoDecision`).
- **P4 3D Neural-HUD & Grok Lab** — the 180-node dual-shell Fibonacci topology
  viewer, HUD preset synthesis under `alimiter=limit=0.95`, and the four Grok
  voice modes over the 10-DNA lab.
- **P5 Founder War-Room Command Bridge** — live sync with
  `/api/v1/war-room/{state,step,directive,reset,explain-simple}`, the 24-tick
  causal feed, and the honest `+$9.55 -> $42.55` wallet ledger.

Architecture (IMPERIUM):
- `lib/mairon/world-state.tsx` — the Single Authoritative World State store;
  perspective switching is a pointer move and never resets subsystem slices,
  drafts, jobs, or telemetry. The war-room autoplay clock is provider-owned.
- `lib/mairon` + `@kian/sdk` — every action wraps a real `/api/v1/*` route.

Legacy routes remain: the Telegram mini-app under `app/(miniapp)/` and the
static artifacts under `public/`. Stack: Next.js 16 App Router, React 19,
Tailwind 4, `@kian/contracts`, `@kian/sdk`, `@kian/config`.
