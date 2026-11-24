# MASTER SYNTHESIS PROMPT — BUILD `00_ECOSYSTEM_INDEX.md`

> **Penta-model registry (Commit #27):** the fleet is now 5 sovereign models — `models/01-gpt-6-web/`, `models/02-gemini-4-aragon-high/`, `models/03-deepseek-3.2-web-think-mode/`, `models/04-claude-sonnet-5.5-extra/` (alias `Claude-sonnet-5.5-extra`) and `models/05-glm-5.3-flash-max/` (aliases `GLM 5.3 flash MAX`, `glm-5.3-flash-max`, `GLM-5.3-Flash-Max`) — 45/45 canonical DNA files. The canonical loader (`crates/ai/src/omni_genome_router.rs`) accepts both these canonical 9-file names and the legacy layer names below.

Read all `models/*/01_TOPOLOGY_AND_HARNESS.md` through `models/*/09_ROUTING_AND_HANDOFF_CARD.md` files currently present in the workspace.
Synthesize the root `00_ECOSYSTEM_INDEX.md` file containing:
1. **Registered Model Fleet Inventory Table:** Model ID, Reasoning Paradigm, Optimal Context Sweet Spot, Cost/Speed Tier, and Top 3 Assigned Roles.
2. **20-Task Champion Matrix:** For each of the 20 core engineering workloads defined in `09_ROUTING_AND_HANDOFF_CARD.md`, list the `#1 Primary Model`, `#2 Reviewer/Verifier Model`, and `#3 Zero-Cost Fallback Model` based on their calibrated scorecards.
3. **Cross-Model Pipeline Recipes:**
   - **Recipe A (Zero-to-One Feature Pipeline):** `Architect Model` -> `Builder Model` -> `UI Craftsman Model` -> `Auditor Model`.
   - **Recipe B (Hard Bug Triage & Fix Pipeline):** `Repo Archaeologist (Long Context)` -> `Root-Cause Reasoner` -> `Surgical Patch Builder`.
   - **Recipe C (Prompt Compilation Lookup):** Quick reference index mapping each model to its `07_PROMPT_COMPILER_LEXICON.md` template and `08_FAILURE_MODES_AND_PATCHES.md` sanitizer rules.
