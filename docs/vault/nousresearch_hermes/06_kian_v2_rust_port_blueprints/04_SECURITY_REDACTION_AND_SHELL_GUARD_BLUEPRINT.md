# Defense-in-Depth Secret Redaction, Unicode Smuggling & Smart Approval (`crates/hermes` + `crates/bot`)

## Source Gems
- `04_gateway_telegram_and_cron/agent/redact.py` & `redact_*.py`
- `04_gateway_telegram_and_cron/tools/approval_smart.py`
- `04_gateway_telegram_and_cron/tools/approval_variation_selectors.py`
- `04_gateway_telegram_and_cron/tools/threat_patterns.py`
- `03_memory_and_session_fts/agent/memory_manager.py` (`_redact_for_provider`, `_StuckCallGate`)

## Porting into KIAN v2
1. **Import-Time Immutable Redaction + Learned-Token Propagation**:
   - Freeze redaction policy at process startup so an LLM cannot disable redaction via env mutations.
   - Cap JSON traversal depth (`320` levels) on browser/CDP payloads, scrub `_SENSITIVE_QUERY_PARAMS`, and propagate any redacted secret detected in tool output to scrub bare repetitions in model prose (`_redact_for_provider`).
2. **Unicode Variation Selector & Comment-Stripped Smart Approval**:
   - Port `_HIDDEN_VARIATION_SELECTOR_RE` to block Glassworm/invisible Unicode smuggling in shell commands BEFORE NFKC normalization.
   - Strip unquoted shell comments (`_strip_shell_comments`, keeping `#` inside quotes/heredocs intact) before passing `<command>` to an auxiliary security reviewer LLM (`APPROVE` / `DENY` / `ESCALATE`).
