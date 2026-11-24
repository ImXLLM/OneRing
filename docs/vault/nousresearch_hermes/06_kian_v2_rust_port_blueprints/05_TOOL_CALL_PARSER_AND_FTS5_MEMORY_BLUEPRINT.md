# Hybrid XML/JSON Tool-Call Parser & Dual FTS5+Trigram Session Search (`crates/ai` + SQLite)

## Source Gems
- `01_hermes_agent_core_loop/agent/chat_completion_helpers_tool_calls.py`
- `05_hermes_function_calling_schemas/{utils.py,validator.py,jsonmode.py,prompt_assets/sys_prompt.yml}`
- `03_memory_and_session_fts/persistence/sqlite/conversation/_store/search.py`
- `04_gateway_telegram_and_cron/cron/{scheduler.py,missed_runs.py}` & `gateway/bot_loop_guard.py`

## Porting into KIAN v2
1. **Resilient Streaming Tool-Call Accumulator + `<tool_call>` Fallback**:
   - Port `chat_completion_helpers_tool_calls.py`: overwrite (`=`) rather than append (`+=`) streamed function names (fixes MiniMax/NIM duplicate names), track JSON brace depth to split reused tool indices, and fall back to `<tool_call>...</tool_call>` XML/JSON parsing (`Hermes-Function-Calling/utils.py`) for open-weight / free-tier models.
2. **Hybrid FTS5 + Trigram Search + Bot Loop Guard**:
   - Pair standard SQLite FTS5 with trigram fallback for CJK/substring queries (`_CONTEXT_WINDOW_SQL` fetches hit ±1 surrounding messages).
   - Port `gateway/bot_loop_guard.py` + `cron/missed_runs.py` into `crates/hermes` to prevent bot-to-bot infinite loops and missed-cron replay storms.
