# Three-Tier Prompt Assembly & Prefix Cache Stability Blueprint (`crates/hermes` + `crates/ai`)

## Source Gems
- `01_hermes_agent_core_loop/SOUL.md`
- `01_hermes_agent_core_loop/agent/system_prompt.py`
- `01_hermes_agent_core_loop/agent/prompt_builder.py` (`DEFAULT_AGENT_IDENTITY`, `SKILLS_GUIDANCE`, `build_memory_guidance`)
- `01_hermes_agent_core_loop/agent/prompt_cache_boundary.py`

## Porting into KIAN v2 (`crates/hermes` & `crates/ai`)
1. **3-Tier Prompt Split (`stable` -> `context` -> `volatile`)**:
   - Build the system prompt ONCE per session in Rust and keep `stable` (identity + behavioral sizing rules + tool rules) completely invariant across turns so Anthropic/OpenAI/OpenRouter prompt prefix caching stays warm.
   - Freeze `MEMORY.md` / `"KbEntry"` snapshots at session start; mid-session writes persist to SQLite/Postgres immediately without mutating the active session's system prompt.
   - Keep `SKILLS_GUIDANCE` terse and filter-safe (`"When you work out a non-trivial workflow, record it with skill_manage for future reuse."`) and avoid imperative self-instructions in persistent memory (always declarative facts!).
2. **Thread-Timed Context Loader**:
   - Read workspace files (`AGENTS.md`, `SOUL.md`, `.cursorrules`) with a hard timeout (`5s`) and scan via `threat_patterns` (`scope="context"`) before injection.
