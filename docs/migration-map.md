# نقشهٔ مهاجرت legacy → KIAN

| قطعه | منبع legacy | خانهٔ جدید |
|---|---|---|
| Din + دفتر + امانت ۸ حالته | R4/pro `crates/domain` | `crates/domain` (همان کرت، بازتأیید تست‌ها) |
| کیف پول + خزانه | `crates/wallet` + 10x `treasury_worker.py` | `crates/wallet` |
| گیت‌وی AI | TS `back/ai-gateway` + Rust `crates/ai` | `crates/ai` |
| موتور هرمس + مهارت‌ها | 10x `hermes_engine.py` + `data/skills` | `crates/hermes` (مهارت‌ها = داده در `docs/skills`→db seed) |
| پارلمان | TS `packages/nation` + Rust `crates/nation` | `crates/nation` |
| A2A/MCP/ناوگان | 10x `a2a_server.py` + Rust `crates/agents` | `crates/agents` |
| پنل ادمین ۷۱ اندپوینت | 10x `web_admin.py` | `apps/admin` |
| هندلرهای ربات ۱۷گانه | 10x `engine/handlers` | `crates/bot` (orchestrator) |
| هندلرهای رشد/کوئست | 10x `quests/referral/growth` | `crates/superapp` |
| محافظان امنیتی | 10x `approval/moderation/sandbox/sqlsafe` | `crates/server` + `crates/hermes` |
| ویجت | Agent-One `packages/widget` | `packages/widget` |
| آزمایش x402/ERC-8004 | 0xKiaBOT | `crates/agents` (فاز بعدی) |
