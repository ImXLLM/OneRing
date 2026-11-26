#!/usr/bin/env bash
# mount_nousresearch_vault.sh — mounting a selected subset of the NousResearch/hermes-agent vault
# (Wave 2 - Step 2) under docs/vault/nousresearch_hermes/ with secret sanitization.
#
# ~60 selected files: listing and manifest + Rust port maps (folder 06) + the complete
# Hermes-Function-Calling schema (folder 05) + the agent-loop core and compressor (folder 01) +
# memory and FTS search (folder 03) + bot-loop security and guard (folder 04) + 9 flagship SKILL.md files.
set -euo pipefail

SRC="/tmp/vault_nousresearch"
DST="/home/z/my-project/kian-launch/kian/docs/vault/nousresearch_hermes"

rm -rf "$DST"
mkdir -p "$DST"

# ── 1. Listing, manifest, and port maps ────────────────────────────────────────
cp "$SRC/VAULT_INDEX_FA.md" "$SRC/MANIFEST.json" "$DST/"
mkdir -p "$DST/06_kian_v2_rust_port_blueprints" "$DST/05_hermes_function_calling_schemas" \
         "$DST/01_hermes_agent_core_loop" "$DST/03_memory_and_session_fts" \
         "$DST/04_gateway_telegram_and_cron" "$DST/02_hermes_builtin_skills_catalog"
cp "$SRC"/06_kian_v2_rust_port_blueprints/*.md "$DST/06_kian_v2_rust_port_blueprints/"

# ── 2. Complete Hermes-Function-Calling schema (folder 05) ───────────────────────
for f in README.md functioncall.py functions.py jsonmode.py prompter.py schema.py \
         utils.py validator.py requirements.txt chat_templates/chatml.j2 \
         chat_templates/vicuna.j2 chat_templates/zephyr.j2 prompt_assets/few_shot.json \
         prompt_assets/sys_prompt.yml; do
  mkdir -p "$DST/05_hermes_function_calling_schemas/$(dirname "$f")"
  cp "$SRC/05_hermes_function_calling_schemas/$f" "$DST/05_hermes_function_calling_schemas/$f"
done

# ── 3. Agent-loop core: stream accumulator, compressor, three-layer prompt (folder 01) ─
for f in SOUL.md AGENTS.md SECURITY.md run_agent.py toolsets.py model_tools.py \
         trajectory_compressor.py agent/chat_completion_helpers_tool_calls.py \
         agent/context_compressor.py agent/context_compressor_prune.py \
         agent/micro_compaction.py agent/system_prompt.py agent/prompt_builder.py \
         agent/transcript_repair.py agent/prompt_cache_boundary.py; do
  mkdir -p "$DST/01_hermes_agent_core_loop/$(dirname "$f")"
  cp "$SRC/01_hermes_agent_core_loop/$f" "$DST/01_hermes_agent_core_loop/$f"
done

# ── 4. Memory and session search (folder 03) ────────────────────────────────────────
for f in tools/memory_tool.py tools/memory_tool_store.py agent/memory_manager.py \
         agent/memory_context.py agent/memory_provider.py \
         persistence/sqlite/conversation/_store/search.py \
         persistence/sqlite/conversation/_store/compression.py \
         persistence/sqlite/conversation/_store/transcript_repair.py; do
  mkdir -p "$DST/03_memory_and_session_fts/$(dirname "$f")"
  cp "$SRC/03_memory_and_session_fts/$f" "$DST/03_memory_and_session_fts/$f"
done

# ── 5. Security, approval gate, and bot-loop guard (folder 04) ──────────────────────────
for f in agent/redact.py agent/redact_partial_keys.py gateway/bot_loop_guard.py \
         gateway/delivery_ledger.py tools/approval_smart.py \
         tools/approval_variation_selectors.py tools/approval_detection.py \
         tools/threat_patterns.py cron/scheduler.py cron/missed_runs.py; do
  mkdir -p "$DST/04_gateway_telegram_and_cron/$(dirname "$f")"
  cp "$SRC/04_gateway_telegram_and_cron/$f" "$DST/04_gateway_telegram_and_cron/$f"
done

# ── 6. 9 flagship SKILL.md files (folder 02) ────────────────────────────────────────
declare -A SKILLS=(
  ["popular-web-designs"]="creative"
  ["p5js"]="creative"
  ["architecture-diagram"]="creative"
  ["baoyu-infographic"]="creative"
  ["systematic-debugging"]="software-development"
  ["test-driven-development"]="software-development"
  ["codebase-inspection"]="software-development"
  ["grounded-citations"]="research"
  ["product-price-monitor"]="productivity"
)
for skill in "${!SKILLS[@]}"; do
  cat="${SKILLS[$skill]}"
  mkdir -p "$DST/02_hermes_builtin_skills_catalog/$cat/$skill"
  cp "$SRC/02_hermes_builtin_skills_catalog/skills/$cat/$skill/SKILL.md" \
     "$DST/02_hermes_builtin_skills_catalog/$cat/$skill/SKILL.md"
done

# ── 7. Secret sanitization: sk-… / ghp_… / eyJ… → DUMMY-TEST-FIXTURE ─────────────
python3 - <<'PYEOF'
import re
from pathlib import Path

DST = Path("/home/z/my-project/kian-launch/kian/docs/vault/nousresearch_hermes")
# Secret-like patterns — catches only the vault's test samples; real documentation stays untouched
PATTERNS = [
    re.compile(r"sk-[A-Za-z0-9_-]{16,}"),
    re.compile(r"ghp_[A-Za-z0-9]{10,}"),
    re.compile(r"gho_[A-Za-z0-9]{10,}"),
    re.compile(r"eyJ[A-Za-z0-9_-]{20,}(?:\.[A-Za-z0-9_=-]{4,}){1,2}"),
    re.compile(r"xox[baprs]-[A-Za-z0-9-]{10,}"),
    re.compile(r"AKIA[0-9A-Z]{16}"),
    re.compile(r"sk-or-v1-[A-Za-z0-9]{16,}"),
]
changed = 0
for path in DST.rglob("*"):
    if not path.is_file():
        continue
    try:
        text = path.read_text(encoding="utf-8")
    except (UnicodeDecodeError, ValueError):
        continue
    out = text
    for pat in PATTERNS:
        out = pat.sub("DUMMY-TEST-FIXTURE", out)
    if out != text:
        path.write_text(out, encoding="utf-8")
        changed += 1
        print(f"  sanitized: {path.relative_to(DST)}")
print(f"secret sanitization: {changed} file(s) rewritten")
PYEOF

COUNT=$(find "$DST" -type f | wc -l)
echo "mounted $COUNT files into docs/vault/nousresearch_hermes/"
