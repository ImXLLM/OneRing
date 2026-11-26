# @kian/contracts — MAIRON Wire Contracts (Zod 4)

The single authoritative TypeScript mirror of the kernel's Rust DTOs
(`crates/contracts`), plus the MAIRON domain modules under `src/mairon/`:

- `ledger.ts` — double-entry `Account` / `JournalEntry` / `Posting` with the
  `SUM(Posting.amount) == 0` invariant, the eight-state escrow machine, and the
  full 19-column `Product` row.
- `trinity.ts` — the Mairon Trinity Engine: Alpha IQ 136, CommandCenter IQ 139,
  Mairon IQ 150, the REA `>= 85` publication gate, two-key publish tickets, and
  Bayesian review summaries.
- `arda.ts` — the Mairon Arda i18n engine: 20-locale manifest (`en` native
  default, `PATCH_READY_SLOT` for pluggable patches including `fa-IR`),
  `SvgFlagSpec` geometry (`viewBox="0 0 20 14"`), and `SkillMdExport`.
- `superstudio.ts` — `UxMorphMode` (`ToySpark` / `CreatorStudio` / `ProTerminal`),
  the nine zero-cost modalities, `SearchReplacePatch`, `TimeTravelCheckpoint`,
  `McpToolInvocation`, and `MaironVerifiedSeal`.
- `quant.ts` — `FixedPointI128`, `TickSize`, `StrategyAst`, and the
  `BacktestCausalReport` with the `Sharpe >= 1.20` live-deployment gate and the
  SHA-256 claim hash.
- `treasury.ts` — Base-8453 settlement constants (USDC `0x8335...2913`,
  `$0xKIA`, XTR), the four-stream revenue split, the capital reserve partition,
  and the 5 x 9 = 45 `OmniModelGenomeSlot` registry.
- `neuralHud.ts` — the 180-node dual-shell `FibonacciSphereNode`, the
  `alimiter=limit=0.95` audio safety limiter, lab telemetry rows, SFT/DPO
  export lines, and the four Grok voice modes.
- `warRoom.ts` — `WarRoomStateSnapshot` mirrors: 24 hourly causal ticks,
  co-founder chat, chairman directives, the honest `+$9.55 -> $42.55` wallet
  ledger, and ELI5 pipeline cards.
- `worldState.ts` — the Single Authoritative World State contract
  (`MaironWorldState`) with non-destructive perspective switching.

Wire rules: camelCase, explicit null for nullable fields, integers only on
money, and every field the Rust struct sends must exist here.
