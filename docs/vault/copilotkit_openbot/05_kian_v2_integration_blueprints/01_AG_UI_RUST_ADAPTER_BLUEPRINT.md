# AG-UI Protocol Adapter Blueprint for KIAN v2 (`crates/hermes` + `crates/ai`)

## Source Gems
- `03_ag_ui_protocol_and_schemas/rust/ag-ui-core/src/event.rs` (`EventType`, `BaseEvent`, `TextMessage*`, `ToolCall*`, `StateSnapshot`, `StateDelta`, `RunStarted/Finished/Error`)
- `03_ag_ui_protocol_and_schemas/typescript/core/generated/types.ts` (Full AG-UI 1.0 `EventType` including `ACTIVITY_SNAPSHOT`, `ACTIVITY_DELTA`, `REASONING_*`, `SUBAGENT_STARTED/FINISHED/ERROR`)
- `01_openbot_core/app_copilot/repair-history.ts` (`repairUnansweredToolCalls`, `withStoredAnswers`)
- `01_openbot_core/app_copilot/settled-args.ts` (`settledArgs`)

## Where to Inject in KIAN v2
1. **`crates/hermes` (`src/agui_stream.rs`)**:
   - Adopt `ag-ui-core` event enum variants as KIAN v2's canonical SSE/WebSocket frame format (`SCREAMING_SNAKE_CASE`).
   - Add `ACTIVITY_SNAPSHOT` / `ACTIVITY_DELTA` (RFC 6902 JSON Patch) and `SUBAGENT_STARTED/FINISHED/ERROR` so `apps/web/app/builder` and `apps/miniapp` render live subagent progress and sandboxed UI frames without custom ad-hoc events.
2. **`crates/ai` (`src/history_repair.rs`)**:
   - Port `repairUnansweredToolCalls` from `repair-history.ts` into Rust before every LLM completion call:
     - Move out-of-order `role: "tool"` messages immediately after their originating `assistant.tool_calls`.
     - Strip duplicate tool results or orphan results whose `toolCallId` does not exist.
     - Inject `UNANSWERED` / `NO_ANSWER_CAME` tool results for aborted HITL calls so conversations never brick with `AI_MissingToolResultsError`.
3. **`apps/web` & `apps/miniapp` (`src/lib/settled-args.ts`)**:
   - Use `settledArgs` in streaming tool UI cards to hold back partial numbers, `#hex` colors, and `http` URLs until complete while streaming labels smoothly.
