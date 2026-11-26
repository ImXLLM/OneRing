# @kian/config — MAIRON Canonical Configuration

The single source of configuration truth for the MAIRON platform:

- `MAIRON_BRAND` — the sovereign identity
  ("MAIRON — Sovereign Intelligence, Quantitative Execution & Autonomous Capital OS").
- Base-8453 settlement constants (`BASE_CHAIN_ID = 8453`, USDC
  `0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913`, `$0xKIA`, Telegram Stars `XTR`),
  the four-stream charter rates, and the capital reserve partition.
- `MAIRON_ENDPOINTS` — the canonical endpoint table every client may call.
- Runtime endpoint resolvers — `resolveApiBase`, `resolveKernelUrl`,
  `resolveWarRoomUrl`, `resolveNeuralHudUrl` (same-origin aware, env-priority).
- `MAIRON_ENV_SPEC` — the 69-key, `P0 -> P5` priority-ordered environment
  schema with `auditEnvReadiness` / `envKeysOfTier` / `readEnv` helpers.
