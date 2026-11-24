# Deterministic Task Threads, Routines & Fatigue-Protected Scheduler Blueprint for KIAN v2

## Source Gems
- `01_openbot_core/shared/orchestration.ts` (`delegate`, `follow_up`, `stop`, `link_to_thread`, `DELEGATING_GUIDANCE`)
- `01_openbot_core/shared/subagent.ts` (`subagentThreadId` via SHA-256 UUIDv8)
- `01_openbot_core/docs/routines.md` & `01_openbot_core/shared/routine-firing.ts`

## Where to Inject in KIAN v2
1. **Deterministic Subagent Thread IDs (`crates/nation` & `apps/web`)**:
   - Derive child task thread IDs deterministically via `SHA-256("subagent\n" + parentThreadId + "\n" + toolCallId)` formatted as UUIDv8 carrying the parent's 6-byte deployment prefix. Both frontend and backend compute the exact same thread ID before the task result returns!
2. **Production-Hardened Routine Scheduler (`crates/hermes` / PostgreSQL)**:
   - **15-minute minimum interval floor** & **20-active-routines-per-user cap** (cannot be bypassed by prompt injection).
   - **Fatigue rule**: Alert once on first failure; auto-disable after **10 consecutive failures** so broken tokens don't spam forever.
   - **Skip missed windows**: Advance `next_run_at` instead of replaying 30 stale missed runs after downtime.
   - **Multi-replica safety**: PostgreSQL advisory lock leader + idempotent `(routine_id, minute_bucket)` work queue claim.
3. **Channels SDK Bridge (`crates/hermes` / Node Channel Worker)**:
   - Mirror `@copilotkit/channels` JSX-to-native adapter (`Slack Block Kit`, `Telegram InlineKeyboard`, `Discord Components`) for cross-channel KIAN agent delivery.
