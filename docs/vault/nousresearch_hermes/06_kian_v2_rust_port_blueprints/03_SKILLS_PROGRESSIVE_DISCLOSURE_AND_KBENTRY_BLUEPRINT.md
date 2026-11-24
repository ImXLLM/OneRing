# Progressive-Disclosure Skills Engine & `"KbEntry"` Integration (`crates/hermes` + `seed/kian-production.db`)

## Source Gems
- `02_hermes_builtin_skills_catalog/skills/**/SKILL.md` (44 curated production skills)
- `02_hermes_builtin_skills_catalog/engine/agent/skill_utils.py` & `tools/skill_manager_tool.py`
- `02_hermes_builtin_skills_catalog/engine/tools/skill_manager_scan.py` & `skills_guard.py` & `skills_ast_audit.py`
- `01_hermes_agent_core_loop/self_evolution/evolution/skills/evolve_skill.py`

## Porting into KIAN v2
1. **Index-Only Prompt Injection + Lazy `skill_view`**:
   - Inject ONLY `name + description` (capped via `SKILL_PROMPT_DESC_LIMIT`) into the system prompt index; load full `SKILL.md` and `references/` / `scripts/` / `templates/` only when invoked.
   - Seed high-value skills (`arxiv`, `systematic-debugging`, `test-driven-development`, `codebase-inspection`, `grounded-citations`, `architecture-diagram`, `baoyu-infographic`, `humanizer`, `sdlc-review`, `simplify-code`, `product-price-monitor`, `competitor-news-monitor`) directly into `"KbEntry"` (`kind='skill'`) in `seed/kian-production.db`.
2. **Differential Security Gate for Self-Created Skills**:
   - When `crates/bot` / `crates/hermes` creates or patches a skill autonomously, acquire per-skill SHA-256 lock and run differential security scanning (`_scan_baseline` -> `_introduced_findings`) + AST audit before persisting.
