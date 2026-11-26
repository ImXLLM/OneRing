# Legacy → KIAN Migration Map

| Component | Legacy source | New home |
|---|---|---|
| Din + ledger + 8-state escrow | R4/pro `crates/domain` | `crates/domain` (same crate, tests re-validated) |
| Wallet + treasury | `crates/wallet` + 10x `treasury_worker.py` | `crates/wallet` |
| AI gateway | TS `back/ai-gateway` + Rust `crates/ai` | `crates/ai` |
| Hermes engine + skills | 10x `hermes_engine.py` + `data/skills` | `crates/hermes` (skills = data in `docs/skills`→db seed) |
| Parliament | TS `packages/nation` + Rust `crates/nation` | `crates/nation` |
| A2A/MCP/fleet | 10x `a2a_server.py` + Rust `crates/agents` | `crates/agents` |
| Admin panel, 71 endpoints | 10x `web_admin.py` | `apps/admin` |
| The 17 bot handlers | 10x `engine/handlers` | `crates/bot` (orchestrator) |
| Growth/quest handlers | 10x `quests/referral/growth` | `crates/superapp` |
| Security guardians | 10x `approval/moderation/sandbox/sqlsafe` | `crates/server` + `crates/hermes` |
| Widget | Agent-One `packages/widget` | `packages/widget` |
| x402/ERC-8004 experiment | 0xKiaBOT | `crates/agents` (next phase) |
