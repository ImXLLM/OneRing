# Two-Stage Context Compressor & Micro-Compaction Blueprint (`crates/ai`)

## Source Gems
- `01_hermes_agent_core_loop/agent/context_compressor.py`
- `01_hermes_agent_core_loop/agent/context_compressor_prune.py` (`terminal_stub`, `delegate_stub`, `_keep_running_turn`, `_COLD_CACHE_MIN_RECLAIM_TOKENS`)
- `01_hermes_agent_core_loop/agent/micro_compaction.py`
- `01_hermes_agent_core_loop/compression_eval/{DESIGN.md,rubric.py}`

## Porting into KIAN v2 (`crates/ai/src/compressor.rs`)
1. **Stage 1 — Zero-Cost Deterministic Tool-Output Pruning**:
   - Before invoking an auxiliary LLM summarizer, compact old bulky `terminal` / `delegate_task` tool outputs into 1-line deterministic stubs (`[terminal] ran 'cmd' -> exit 0, 142 lines output`, preserving the last non-empty line on non-zero exit).
   - **Critical Invariant (`_keep_running_turn`)**: NEVER prune tool outputs belonging to the currently executing turn (`turn_start`), otherwise the agent forgets what it just read and repeats tool calls blindly.
   - Gate cold-cache pruning behind a minimum reclamation floor (`16,000` tokens) so you don't bust provider caches for negligible savings.
2. **Stage 2 — Exchange-Aware Rolling Summary + Probe Eval**:
   - Summarize complete `(assistant + tool*)` exchanges (never absorbing `user` turns or splitting tool pairs) using a cheap aux model, and evaluate compression quality offline against the 6 Factory.ai dimensions in `compression_eval/rubric.py`.
