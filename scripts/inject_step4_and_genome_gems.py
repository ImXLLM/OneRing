#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""inject_step4_and_genome_gems.py — \u062a\u0632\u0631\u06cc\u0642 \u06f6 \u0645\u0647\u0627\u0631\u062a \u06af\u0627\u0645 \u06f4 (Wave 2) \u0628\u0647 KIAN v2.

Sections:
  1. Idempotent upsert of 6 curated flagship "KbEntry" records into
     seed/kian-production.db:
       - kb:arch-decolua-9router-anti-429            (decolua/9router v0.5.123)
       - kb:arch-opencode-zen-and-fuzzy-patch        (anomalyco/opencode v2.4.1)
       - kb:registry-freellm-52-platforms            (tashfeenahmed/freellmapi)
       - kb:registry-public-apis-120-cors            (top_120 zero-auth APIs)
       - kb:dna-kia-brain-os-9file-genome            (KIA-BRAIN-OS 9-file DNA)
       - kb:arch-omni-model-genome-9file-engine      (OMNI-MODEL-GENOME engine)
  2. Verification: KbEntry 56 -> 62, active products stay 20, citizens 466,
     referral lines 247, escrows 108, trial balance sum stays 0
     (58,700,200,000 micro-KIAN untouched), LIKE search < 5 ms.
  3. Sync of seed/kian-production.db -> /home/z/my-project/download/kian-production.db.

Idempotency: every upsert compares title+body first; unchanged rows are skipped
("0 inserted, 0 updated, 6 skipped" on the second run), changed rows are replaced.
Posting (double-entry ledger) is never touched by this script.
"""

from __future__ import annotations

import shutil
import sqlite3
import sys
import time
from pathlib import Path

REPO = Path("/home/z/my-project/kian-launch/kian")
SEED_DB = REPO / "seed" / "kian-production.db"
DOWNLOAD_DB = Path("/home/z/my-project/download/kian-production.db")
CREATED_AT = "2026-11-24T02:00:00.000Z"

# ── 1. The 6 curated flagship KB entries ─────────────────────────────────────

KB_ITEMS: list[tuple[str, str, str]] = [
    (
        "kb:arch-decolua-9router-anti-429",
        "\u0645\u0639\u0645\u0627\u0631\u06cc \u0645\u0633\u06cc\u0631\u06cc\u0627\u0628\u06cc: \u06af\u0627\u0631\u062f\u0647\u0627\u06cc \u0636\u062f-429 \u062c\u0639\u0644\u06cc 9Router v0.5.123 — \u0633\u0627\u0646\u062a\u06cc‌\u0633\u0627\u06cc\u0632 terminal \u0648 \u067e\u0627\u06a9‌\u0633\u0627\u0632 \u067e\u0631\u0627\u0645\u067e\u062a Antigravity",
        "\u067e\u0648\u0631\u062a decolua/9router v0.5.123 (opencodeFingerprint.js + systemPrompt.js @ bb399170) \u0628\u0631\u0627\u06cc "
        "\u0644\u0650\u06cc\u0646‌\u0647\u0627\u06cc \u0631\u0627\u06cc\u06af\u0627\u0646 oc/* \u06a9\u06cc\u0627\u0646: (\u06f1) OpenCode Free \u0647\u0631 \u0627\u0628\u0632\u0627\u0631\u06cc \u06a9\u0647 \u0646\u0627\u0645\u0634 \u062d\u0627\u0648\u06cc «terminal» \u0628\u0627\u0634\u062f "
        "(\u063a\u06cc\u0631\u062d\u0633\u0627\u0633 \u0628\u0647 \u0628\u0632\u0631\u06af\u06cc \u062d\u0631\u0641) \u0631\u0627 \u0628\u0627 429 FreeUsageLimit \u062c\u0639\u0644\u06cc \u0631\u062f \u0645\u06cc‌\u06a9\u0646\u062f — \u062a\u0627\u0628\u0639 "
        "sanitize_tool_names_for_opencode \u0646\u0627\u0645 \u0631\u0627 \u0628\u0647 shell_exec \u0628\u0627\u0632\u0646\u0648\u06cc\u0633\u06cc \u0645\u06cc‌\u06a9\u0646\u062f (\u0628\u0631\u062e\u0648\u0631\u062f \u0646\u0627\u0645‌\u0647\u0627 \u0628\u0627 "
        "\u067e\u0633\u0648\u0646\u062f _2/_3 \u062d\u0644 \u0645\u06cc‌\u0634\u0648\u062f) \u0648 \u0646\u0642\u0634\u0647\u0654 \u0628\u0627\u0632\u06af\u0634\u062a aliased→original \u0631\u0627 \u0628\u0631\u0627\u06cc \u067e\u0627\u0633\u062e \u0628\u0631\u0645\u06cc‌\u06af\u0631\u062f\u0627\u0646\u062f. "
        "(\u06f2) \u062e\u0637 \u0647\u0648\u06cc\u062a\u06cc «You are Hermes Agent, built by Nous Research» \u0631\u0648\u06cc Google Antigravity "
        "429 RESOURCE_EXHAUSTED \u062c\u0639\u0644\u06cc \u0645\u06cc‌\u06af\u06cc\u0631\u062f — sanitize_system_prompt_for_antigravity \u0622\u0646 \u0631\u0627 \u0628\u0647 "
        "«You are KIAN Autonomous Agent, built by KIAN Systems» \u0628\u0627\u0632\u0646\u0648\u06cc\u0633\u06cc \u0645\u06cc‌\u06a9\u0646\u062f \u0648 \u06f6 \u062e\u0637 \u0645\u062d\u0631\u06a9\u0650 "
        "\u0645\u0633\u062a\u0646\u062f\u0634\u062f\u0647 (End Hermes runtime environment \u0648 \u062f\u0633\u062a\u0648\u0631\u0627\u0644\u0639\u0645\u0644‌\u0647\u0627\u06cc Google model operational \u0648 "
        "…) \u0631\u0627 \u062d\u0630\u0641 \u0645\u06cc‌\u06a9\u0646\u062f\u061b \u0628\u0642\u06cc\u0647\u0654 \u062c\u0645\u0644\u0647 \u062f\u0633\u062a‌\u0646\u062e\u0648\u0631\u062f\u0647 \u0645\u06cc‌\u0645\u0627\u0646\u062f. (\u06f3) \u0637\u0628\u0642\u0647‌\u0628\u0646\u062f\u06cc \u0634\u06a9\u0633\u062a \u0644\u0650\u06cc\u0646: "
        "410/model_not_found → \u0628\u0627\u0632\u0646\u0634\u0633\u062a\u06af\u06cc \u062f\u0627\u0626\u0645\u06cc \u0644\u0650\u06cc\u0646\u060c 400 → \u0641\u0648\u0631\u06cc \u0644\u0650\u06cc\u0646 \u0628\u0639\u062f\u06cc \u0628\u062f\u0648\u0646 \u06a9\u0648\u0644‌\u062f\u0627\u0648\u0646\u060c "
        "429 → \u0628\u06a9‌\u0622\u0641 \u0646\u0645\u0627\u06cc\u06cc 2s×2^level \u0628\u0627 \u0633\u0642\u0641 \u06f5 \u062f\u0642\u06cc\u0642\u0647\u060c 5xx → \u06a9\u0648\u0644‌\u062f\u0627\u0648\u0646 \u062a\u0631\u0627\u0646\u0632\u06cc\u062a \u06f3\u06f0 \u062b\u0627\u0646\u06cc\u0647 — "
        "\u0647\u0645\u0627\u0646 accountFallback.js + errorConfig.js + combo.js. (\u06f4) \u0645\u0627\u0644\u062a\u06cc‌\u062a\u0648\u0631\u0646 \u0628\u062f\u0648\u0646 "
        "reasoning_content \u063a\u06cc\u0631\u062e\u0627\u0644\u06cc \u0631\u0648\u06cc Zen/DeepSeek/GLM \u062e\u0637\u0627\u06cc 400 \u0645\u06cc‌\u062f\u0647\u062f — inject_reasoning_placeholders "
        "\u0628\u0631\u0627\u06cc \u067e\u06cc\u0627\u0645‌\u0647\u0627\u06cc assistant \u06cc\u06a9 « » \u062a\u0632\u0631\u06cc\u0642 \u0645\u06cc‌\u06a9\u0646\u062f. \u067e\u0648\u0631\u062a Rust \u06a9\u06cc\u0627\u0646: crates/ai/src/free_llm_registry.rs "
        "(sanitize_tool_names_for_opencode + sanitize_system_prompt_for_antigravity + classify_http_failure "
        "+ LaneHealthTracker + inject \u0642\u0648\u0627\u0639\u062f reasoning). \u0645\u0646\u0628\u0639: \u06af\u0646\u062c\u06cc\u0646\u0647\u0654 decolua-9router-gems (Wave 2 - Step 4).",
    ),
    (
        "kb:arch-opencode-zen-and-fuzzy-patch",
        "\u0645\u0639\u0645\u0627\u0631\u06cc \u062f\u0631\u06af\u0627\u0647: OpenCode Zen \u0635\u0641\u0631-\u06a9\u0627\u0631\u062a (Bearer public + \u06f1\u06f0 \u0645\u062f\u0644 \u0641\u0639\u0627\u0644) \u0648 \u0648\u06cc\u0631\u0627\u06cc\u0634\u06af\u0631 \u062c\u0631\u0627\u062d\u06cc \u06f5\u06af\u0630\u0631",
        "\u062f\u0648 \u0642\u0637\u0639\u0647\u0654 anomalyco/opencode v2.4.1 (commit 18ff071): (\u06f1) \u062f\u0631\u06af\u0627\u0647 \u0627\u0633\u062a\u0646\u062a\u0627\u062c "
        "https://opencode.ai/inference/openai/v1 \u0628\u0627 \u0641\u0627\u0644‌\u0628\u06a9 legacy https://opencode.ai/zen/v1\u061b "
        "\u0648\u0642\u062a\u06cc \u0647\u06cc\u0686 \u06a9\u0644\u06cc\u062f\u06cc \u062a\u0646\u0638\u06cc\u0645 \u0646\u0634\u062f\u0647\u060c \u067e\u0644\u0627\u06af\u06cc\u0646 Opencode «Authorization: Bearer public» \u0648 "
        "«User-Agent: opencode/latest/2.4.1/opencode» \u062a\u0632\u0631\u06cc\u0642 \u0645\u06cc‌\u06a9\u0646\u062f — \u0646\u0627\u0634\u0646\u0627\u0633 \u0645\u0637\u0644\u0642\u060c \u0635\u0641\u0631 \u06a9\u0627\u0631\u062a. "
        "\u06f1\u06f0 \u0645\u062f\u0644 \u0641\u0639\u0627\u0644 \u0635\u0641\u0631-\u0647\u0632\u06cc\u0646\u0647: beam-free (1M ctx)\u060c big-pickle (200K)\u060c deepseek-v4.1-flash-free "
        "(200K\u060c vision)\u060c flint-40b-free (262K)\u060c hy4-flash-free (1M\u060c vision)\u060c iceman-free (262K)\u060c "
        "mimo-v2.6-flash-free (200K\u060c multimodal)\u060c nemotron-3.5-lightning-free (262K)\u060c "
        "nemotron-3.5-super-vl-free (1M\u060c vision)\u060c tidepool-alpha-free (1M)\u061b \u0645\u062f\u0644‌\u0647\u0627\u06cc \u0628\u0627\u0632\u0646\u0634\u0633\u062a\u0647 "
        "\u067e\u06cc\u0634 \u0627\u0632 \u062f\u0631\u06af\u0627\u0647 \u0628\u0644\u0627\u06a9 \u0645\u06cc‌\u0634\u0648\u0646\u062f: longcat-2.5-free (410)\u060c glm-5.3-flash-free (400)\u060c "
        "deepseek-v4-flash-free (400)\u060c kimi-k2.5-free\u060c qwen3.6-plus-free \u0648 sambanova (402). "
        "(\u06f2) \u0648\u06cc\u0631\u0627\u06cc\u0634\u06af\u0631 \u062c\u0631\u0627\u062d\u06cc \u06f5\u06af\u0630\u0631: \u06af\u0630\u0631 \u06f1 \u062a\u0637\u0628\u06cc\u0642 \u062f\u0642\u06cc\u0642 \u0628\u0627\u06cc\u062a‌\u0628\u0647‌\u0628\u0627\u06cc\u062a\u060c \u06af\u0630\u0631 \u06f2 trim \u0627\u0646\u062a\u0647\u0627\u06cc \u062e\u0637\u060c "
        "\u06af\u0630\u0631 \u06f3 trim \u062f\u0648 \u0637\u0631\u0641\u060c \u06af\u0630\u0631 \u06f4 \u0646\u0631\u0645\u0627\u0644‌\u0633\u0627\u0632\u06cc \u06cc\u0648\u0646\u06cc\u06a9\u062f (\u06af\u06cc\u0648\u0645\u0647\u0654 \u0641\u0631\u0641\u0631\u06cc ’“→'\"\u060c \u062f\u064e\u0634 —–→-\u060c "
        "\u0641\u0627\u0635\u0644\u0647\u0654 non-breaking → \u0641\u0627\u0635\u0644\u0647)\u060c \u06af\u0630\u0631 \u06f5 \u0627\u0646\u06a9\u0631 \u0628\u0644\u0648\u06a9\u06cc (≥\u06f3 \u062e\u0637: \u062e\u0637 \u0627\u0648\u0644/\u0622\u062e\u0631 + \u0634\u0628\u0627\u0647\u062a \u0645\u06cc\u0627\u0646\u06cc "
        "≥\u06f0\u066b\u06f5 \u0628\u0627 LCS) \u0648 \u0646\u0631\u0645\u0627\u0644‌\u0633\u0627\u0632\u06cc \u062a\u0648\u0631\u0641\u062a\u06af\u06cc — \u0628\u0627 \u062d\u0641\u0638 CRLF/LF \u0641\u0627\u06cc\u0644\u061b \u0634\u06a9\u0633\u062a → \u0631\u0623\u06cc‌\u06af\u06cc\u0631\u06cc \u062e\u0637\u0648\u0637 "
        "(missHint) \u0628\u0627 \u0634\u0645\u0627\u0631\u0647\u0654 \u062e\u0637 \u0646\u0632\u062f\u06cc\u06a9\u061b \u062a\u0634\u062e\u06cc\u0635 «\u0627\u0632 \u0642\u0628\u0644 \u0627\u0639\u0645\u0627\u0644‌\u0634\u062f\u0647»\u061b \u0686\u0646\u062f\u062a\u0637\u0628\u06cc\u0642 \u0645\u0628\u062a\u0646\u06cc \u0628\u0631 \u0631\u062f. "
        "\u0647\u0645\u0631\u0627\u0647 \u0633\u06cc\u0627\u0633\u062a \u0627\u062c\u0631\u0627\u06cc \u06a9\u062f\u06a9\u0633: git reset --hard \u0648 rm -rf / \u0645\u0645\u0646\u0648\u0639\u060c git push --force \u0648 "
        "python3 -c \u0646\u06cc\u0627\u0632\u0645\u0646\u062f \u062a\u0623\u06cc\u06cc\u062f \u0627\u0646\u0633\u0627\u0646\u06cc\u060c cargo test/clippy/check \u0648 ls \u0648 pwd \u0645\u062c\u0627\u0632\u060c "
        "\u0628\u0642\u06cc\u0647 → Prompt. \u067e\u0648\u0631\u062a Rust \u06a9\u06cc\u0627\u0646: crates/ai/src/opencode_zen.rs + "
        "crates/hermes/src/fuzzy_patch.rs (apply_surgical_patch + miss_hint + evaluate_exec_policy). "
        "\u0645\u0646\u0628\u0639: \u06af\u0646\u062c\u06cc\u0646\u0647\u0654 anomalyco-opencode-gems (Wave 2 - Step 4).",
    ),
    (
        "kb:registry-freellm-52-platforms",
        "\u0631\u062c\u06cc\u0633\u062a\u0631\u06cc: \u0622\u0628\u0634\u0627\u0631 \u06f3\u0644\u0627\u06cc\u0647\u0654 \u06f5\u06f2 \u067e\u0644\u062a\u0641\u0631\u0645 LLM \u0631\u0627\u06cc\u06af\u0627\u0646 \u0628\u062f\u0648\u0646 \u06a9\u0627\u0631\u062a — Tier1 \u0646\u0627\u0634\u0646\u0627\u0633 → Tier2 \u0628\u062f\u0648\u0646 \u06a9\u0627\u0631\u062a → Tier3 \u0645\u0631\u0648\u0631\u06af\u0631",
        "\u06a9\u0627\u062a\u0627\u0644\u0648\u06af tashfeenahmed/freellmapi (62526e0 + \u06a9\u0627\u062a\u0627\u0644\u0648\u06af \u0632\u0646\u062f\u0647\u0654 2026.11.23.4b4737): \u06f5\u06f2 \u067e\u0644\u062a\u0641\u0631\u0645 "
        "OpenAI-compatible \u0628\u0627 \u06f4\u06f5\u06f4 \u0645\u062f\u0644 \u0686\u062a\u060c \u0647\u0645\u0647 \u0628\u062f\u0648\u0646 \u06a9\u0627\u0631\u062a \u0628\u0627\u0646\u06a9\u06cc (\u062d\u0627\u06a9\u0645\u06cc\u062a \u0635\u0641\u0631-\u06a9\u0627\u0631\u062a DNAme). \u0622\u0628\u0634\u0627\u0631 \u0631\u0633\u0645\u06cc \u06a9\u06cc\u0627\u0646: "
        "Tier 1 \u0646\u0627\u0634\u0646\u0627\u0633 \u0645\u0637\u0644\u0642 — OpenCode Zen (Bearer public)\u060c Kilo (\u06f2\u06f0\u06f0 \u062f\u0631\u062e\u0648\u0627\u0633\u062a/\u0633\u0627\u0639\u062a per-IP)\u060c "
        "OVHcloud (Qwen3.5-397B\u060c \u06f2RPM)\u060c LLM7\u060c Pollinations\u061b Tier 2 \u062b\u0628\u062a‌\u0646\u0627\u0645 \u0631\u0627\u06cc\u06af\u0627\u0646 \u0628\u062f\u0648\u0646 \u06a9\u0627\u0631\u062a — "
        "Cerebras (\u06f1M \u062a\u0648\u06a9\u0646/\u0631\u0648\u0632)\u060c Groq\u060c Google Gemini\u060c OpenRouter :free (\u06f5\u06f0 RPD)\u060c Mistral\u060c "
        "NVIDIA NIM\u060c Zhipu Flash (\u06f1M \u062a\u0648\u06a9\u0646/\u0631\u0648\u0632)\u060c Cloudflare Workers AI (\u06f1\u06f0\u0647\u0632\u0627\u0631 Neuron/\u0631\u0648\u0632)\u061b "
        "Tier 3 \u06a9\u0627\u062a\u0627\u0644\u0648\u06af \u0645\u0631\u0648\u0631\u06af\u0631\u06cc \u06f1\u06f2\u06f0 API \u0639\u0645\u0648\u0645\u06cc CORS-\u062f\u0627\u0631 \u0628\u0631\u0627\u06cc fetch() \u0632\u0646\u062f\u0647\u0654 \u0627\u0633\u062a\u0648\u062f\u06cc\u0648. \u06a9\u0648\u0627\u0631\u06a9‌\u0647\u0627\u06cc "
        "\u0628\u062d\u0631\u0627\u0646\u06cc: Kilo \u067e\u0631\u0627\u0645\u067e\u062a‌\u0647\u0627 \u0631\u0627 \u0628\u0631\u0627\u06cc \u0622\u0645\u0648\u0632\u0634 \u0644\u0627\u06af \u0645\u06cc‌\u06a9\u0646\u062f (\u0645\u062a\u0646 \u062d\u0633\u0627\u0633 redact \u0634\u0648\u062f)\u060c OVH \u0646\u0627\u0634\u0646\u0627\u0633 \u06f2RPM "
        "\u0644\u0650\u06cc\u0646 \u0646\u062c\u0627\u062a \u0627\u0633\u062a \u0646\u0647 \u06af\u0630\u0631\u06af\u0627\u0647 \u0627\u0635\u0644\u06cc\u060c cheapllm \u0628\u0647‌\u062c\u0627\u06cc 429 \u0635\u0641 \u0645\u06cc‌\u0633\u0627\u0632\u062f (timeout 180s)\u060c "
        "Routeway \u0628\u062f\u0648\u0646 User-Agent \u0645\u0631\u0648\u0631\u06af\u0631\u06cc \u062e\u0637\u0627\u06cc Cloudflare 1010 \u0645\u06cc‌\u062f\u0647\u062f\u060c ModelScope \u0646\u06cc\u0627\u0632 \u0628\u0647 "
        "real-name \u0686\u06cc\u0646 \u062f\u0627\u0631\u062f (\u062f\u0631 \u0622\u0628\u0634\u0627\u0631 \u0631\u0633\u0645\u06cc \u06a9\u06cc\u0627\u0646 \u0646\u06cc\u0633\u062a). \u0645\u06cc\u0627\u0646‌\u0627\u0641\u0632\u0627\u0631 \u062a\u0627\u0628‌\u0622\u0648\u0631\u06cc: \u06a9\u0634\u0641 \u0622\u0631\u06af\u0648\u0645\u0627\u0646 \u0627\u0628\u0632\u0627\u0631\u0650 "
        "placeholder-\u0645\u0627\u0646\u0646\u062f (<result-from-…> / {{step_…}}) \u0628\u0627 hint \u062a\u0644\u0627\u0634 \u06cc\u06a9‌\u0628\u0627\u0631\u0647 (langflow) \u0648 \u0686\u06a9 "
        "\u0641\u06cc\u0644\u062f refusal \u0635\u0631\u06cc\u062d \u067e\u06cc\u0634 \u0627\u0632 \u067e\u0627\u0631\u0633 JSON Schema (langchain OutputFixing). \u067e\u0648\u0631\u062a Rust \u06a9\u06cc\u0627\u0646: "
        "crates/ai/src/free_llm_registry.rs (FREE_LLM_PROVIDERS + zero_card_waterfall + "
        "detect_unresolved_placeholder_tool_args + check_model_refusal). \u0645\u0646\u0628\u0639: \u06af\u0646\u062c\u06cc\u0646\u0647\u0654 "
        "freellm-and-public-apis-gems (Wave 2 - Step 4).",
    ),
    (
        "kb:registry-public-apis-120-cors",
        "\u0631\u062c\u06cc\u0633\u062a\u0631\u06cc: \u06f1\u06f2\u06f0 API \u0639\u0645\u0648\u0645\u06cc \u0628\u062f\u0648\u0646 \u06a9\u0644\u06cc\u062f \u0628\u0627 HTTPS/CORS=Yes — \u062c\u0639\u0628\u0647‌\u0627\u0628\u0632\u0627\u0631 fetch() \u0632\u0646\u062f\u0647\u0654 \u0627\u0633\u062a\u0648\u062f\u06cc\u0648\u06cc \u06a9\u06cc\u0627\u0646",
        "\u06a9\u0627\u062a\u0627\u0644\u0648\u06af top_120_studio_browser_apis.json (115 \u0627\u0632 \u06f1\u06f2\u06f0 \u0645\u0631\u0648\u0631\u06af\u0631-\u0622\u0645\u0627\u062f\u0647 \u0628\u0627 CORS=Yes\u060c \u06f5 \u0645\u0648\u0631\u062f "
        "\u0628\u062f\u0648\u0646 CORS \u0641\u0642\u0637 \u0633\u0631\u0648\u0631\u06cc): \u062f\u0633\u062a\u0647‌\u0647\u0627 \u0634\u0627\u0645\u0644 Cryptocurrency (CoinGecko\u060c 0x\u060c Block Lottos)\u060c "
        "Geocoding (REST Countries\u060c GeoJS)\u060c Currency Exchange (Frankfurter\u060c currency-api)\u060c "
        "Weather (Open-Meteo\u060c NASA POWER)\u060c Development (Beeceptor\u060c Quotable)\u060c Test Data "
        "(DummyJSON\u060c FakerAPI)\u060c Anti-Malware (URLhaus)\u060c Books (PoetryDB\u060c Bible-api) \u0648 …\u061b "
        "\u0647\u0645\u0647 \u0628\u062f\u0648\u0646 \u06a9\u0644\u06cc\u062f API \u0648 \u0628\u062f\u0648\u0646 KYC — \u062f\u0642\u06cc\u0642\u0627\u064b \u0631\u0648\u06cc \u062d\u0627\u06a9\u0645\u06cc\u062a \u0635\u0641\u0631-\u06a9\u0627\u0631\u062a \u06a9\u06cc\u0627\u0646. \u0642\u0648\u0627\u0639\u062f \u0627\u0633\u062a\u0648\u062f\u06cc\u0648: "
        "\u0641\u0642\u0637 \u0645\u062f\u062e\u0644‌\u0647\u0627\u06cc HTTPS+CORS-\u062f\u0627\u0631 \u062f\u0631 fetch() \u0632\u0646\u062f\u0647\u0654 \u0645\u0631\u0648\u0631\u06af\u0631 \u0645\u062c\u0627\u0632\u0646\u062f (SSRF guard \u0631\u0648\u06cc \u0645\u0633\u06cc\u0631\u0647\u0627\u06cc "
        "\u062f\u0627\u062e\u0644\u06cc \u0647\u0645\u0686\u0646\u0627\u0646 \u0627\u0639\u0645\u0627\u0644 \u0645\u06cc‌\u0634\u0648\u062f)\u060c URL\u0647\u0627 \u062f\u0631 const STUDIO_PUBLIC_APIS_120 \u06a9\u0631\u0646\u0644 \u06a9\u06cc\u0627\u0646 embed "
        "\u0634\u062f\u0647‌\u0627\u0646\u062f \u062a\u0627 \u0627\u0633\u062a\u0648\u062f\u06cc\u0648 \u0628\u062f\u0648\u0646 \u0634\u0628\u06a9\u0647\u0654 \u0627\u0636\u0627\u0641\u0647 \u06a9\u0627\u062a\u0627\u0644\u0648\u06af \u0631\u0627 \u0628\u0644\u0646\u062f \u06a9\u0646\u062f. \u067e\u0648\u0631\u062a Rust \u06a9\u06cc\u0627\u0646: "
        "crates/ai/src/free_llm_registry.rs (STUDIO_PUBLIC_APIS_120 + "
        "browser_ready_public_apis → PublicApiSpec). \u0645\u0646\u0628\u0639: \u06af\u0646\u062c\u06cc\u0646\u0647\u0654 freellm-and-public-apis-gems "
        "(Wave 2 - Step 4).",
    ),
    (
        "kb:dna-kia-brain-os-9file-genome",
        "\u0698\u0646\u0648\u0645 \u0634\u0646\u0627\u062e\u062a\u06cc: KIA-BRAIN-OS \u06f9\u0641\u0627\u06cc\u0644\u06cc — DNAme/DNAwork \u0633\u0627\u0632\u0646\u062f\u0647\u060c Aragon-4 \u0648 \u0642\u0648\u0627\u0646\u06cc\u0646 \u0635\u0641\u0631-\u06a9\u0627\u0631\u062a/UX-\u0627\u0648\u0644 \u062f\u0631 \u06a9\u0631\u0646\u0644",
        "\u0633\u06cc\u0633\u062a\u0645 \u06f1 \u06af\u0627\u0645 \u06f4: KIA-BRAIN-OS/ \u0631\u06cc\u0634\u0647\u0654 \u0645\u062e\u0632\u0646 — \u06f9 \u0641\u0627\u06cc\u0644 DNA \u0628\u0647‌\u0647\u0645‌\u067e\u06cc\u0648\u0633\u062a\u0647 \u0628\u0627 frontmatter "
        "YAML (file_id\u060c schema_version 1.0.0\u060c layer): 01_DNAme (\u0647\u0648\u06cc\u062a \u0633\u0627\u0632\u0646\u062f\u0647 Kia/KiaAgentX\u060c "
        "\u06f1\u06f8 \u0633\u0627\u0644\u0647\u060c \u062a\u0647\u0631\u0627\u0646\u060c INFJ\u061b \u062d\u0627\u06a9\u0645\u06cc\u062a \u06f1\u06f0\u06f0\u066a \u0635\u0641\u0631-\u06a9\u0627\u0631\u062a \u0648 \u0635\u0641\u0631-KYC \u0628\u0627 \u0631\u06cc\u0644‌\u0647\u0627\u06cc TON \u0648 Telegram Stars "
        "XTR\u061b \u0627\u0648\u0644\u0648\u06cc\u062a #1 \u0645\u062d\u0635\u0648\u0644 = UX \u0641\u0631\u06cc\u0645‌\u0627\u0648\u0644 \u062a\u0639\u0627\u0645\u0644\u06cc\u060c RTL \u062a\u0645\u06cc\u0632 \u0628\u0627 Vazirmatn\u060c \u0635\u0641\u0631 stub \u0645\u0631\u062f\u0647)\u060c "
        "02_DNAwork (\u0645\u0639\u0645\u0627\u0631\u06cc \u06f9 \u06a9\u0631\u06cc\u062a Rust\u060c \u0627\u0631\u0642\u0627\u0645 \u062f\u0641\u062a\u0631: \u06f4\u06f6\u06f6 \u0634\u0647\u0631\u0648\u0646\u062f/\u06f2\u06f4\u06f7 \u0631\u0641\u0631\u0627\u0644/\u06f1\u06f0\u06f8 \u0627\u0633\u06a9\u0631\u0648/\u06f2\u06f0 \u0645\u062d\u0635\u0648\u0644 "
        "\u0641\u0639\u0627\u0644/trialBalanceSum=0 \u0631\u0648\u06cc \u06f5\u06f8\u066c\u06f7\u06f0\u06f0\u066c\u06f2\u06f0\u06f0\u066c\u06f0\u06f0\u06f0 micro-KIAN\u061b \u0627\u0631\u0632\u0634 COCOMO II: $523,750 \u0645\u0633\u062a\u0642\u0644 "
        "/ ~$1.18M \u0628\u0627 \u0627\u06a9\u0648\u0633\u06cc\u0633\u062a\u0645 \u06f6\u06f0 \u067e\u0631\u0648\u0698\u0647)\u060c 03_DNAmind (\u067e\u0631\u0648\u062a\u06a9\u0644 Aragon-4 Principal Architect: "
        "\u062a\u062c\u0632\u06cc\u0647\u0654 \u0627\u0635\u0644 \u0627\u0648\u0644\u060c \u0645\u0645\u06cc\u0632\u06cc \u0631\u06cc\u0627\u0636\u06cc — \u0639\u062f\u062f \u0628\u062f\u0648\u0646 \u0627\u062c\u0631\u0627 \u0645\u0645\u0646\u0648\u0639 — \u0635\u0641\u0631 \u062a\u0645\u0644\u0642\u060c \u0645\u0627\u0698\u0648\u0644 \u0639\u0645\u06cc\u0642 \u0628\u0631 "
        "boilerplate)\u060c 04_DNAharness (\u0641\u06cc\u0632\u06cc\u06a9 \u0645\u062d\u06cc\u0637: Command Center → Z.ai + Arena + Railway\u061b "
        "\u06af\u06cc\u062a \u0631\u0627\u0632: \u0635\u0641\u0631 \u062a\u0648\u06a9\u0646 \u062f\u0631 \u06af\u06cc\u062a\u060c \u062a\u0646\u0647\u0627 \u0645\u062c\u0627\u0632 Bearer public)\u060c 05_DNAskills (\u0645\u0627\u062a\u0631\u06cc\u0633 \u0645\u0647\u0627\u0631\u062a‌\u0647\u0627)\u060c "
        "06_LINK_Router (\u067e\u0644 \u0642\u0637\u0639\u06cc \u0628\u0647 OMNI-MODEL-GENOME)\u060c 07_LINK_Protocol (\u067e\u0627\u06a9\u062a \u062a\u062d\u0648\u06cc\u0644 \u0628\u0627 "
        "base_sha/invariants/verification_proof)\u060c 08_LINK_Extractor (\u067e\u0631\u0648\u062a\u06a9\u0644 \u0627\u0633\u062a\u062e\u0631\u0627\u062c \u06af\u0646\u062c\u06cc\u0646\u0647)\u060c "
        "09_LINK_Bootloader (\u0628\u0648\u062a‌\u0644\u0648\u062f\u0631 \u062a\u06a9‌\u0646\u0648\u0628\u062a\u06cc \u0646\u0634\u0633\u062a). \u062a\u0632\u0631\u06cc\u0642 \u06a9\u0631\u0646\u0644: crates/hermes/src/"
        "creator_dna_contract.rs — CreatorDnaContract (sovereignty=true \u0647\u0645\u06cc\u0634\u06af\u06cc\u060c ux_priority_tier=1\u060c "
        "persona \u062b\u0627\u0628\u062a)\u060c build_creator_dna_system_anchor() \u0644\u0646\u06af\u0631 \u0633\u06cc\u0633\u062a\u0645 \u0647\u0631 \u0627\u06cc\u062c\u0646\u062a \u06a9\u06cc\u0627\u0646\u060c \u0648 "
        "audit_against_creator_dna() \u0645\u0645\u06cc\u0632\u06cc \u0642\u0637\u0639\u06cc \u062f\u0648\u0632\u0628\u0627\u0646\u0647: \u067e\u06cc\u0634\u0646\u0647\u0627\u062f API \u06a9\u0627\u0631\u062a‌\u062f\u0627\u0631\u060c \u0627\u062f\u0639\u0627\u06cc \u062a\u06a9\u0645\u06cc\u0644 "
        "\u0628\u06cc‌\u0627\u062b\u0628\u0627\u062a (all tests pass \u0628\u062f\u0648\u0646 \u0627\u062c\u0631\u0627) \u0648 stub \u0645\u0631\u062f\u0647\u0654 UI \u0631\u0627 \u067e\u0631\u0686\u0645 \u0645\u06cc‌\u0632\u0646\u062f. \u0645\u0646\u0628\u0639: \u0633\u0627\u062e\u062a "
        "\u0627\u0635\u0644\u06cc \u06af\u0627\u0645 \u06f4 (Wave 2 - Step 4).",
    ),
    (
        "kb:arch-omni-model-genome-9file-engine",
        "\u0645\u0639\u0645\u0627\u0631\u06cc \u0698\u0646\u0648\u0645 \u0645\u062f\u0644: OMNI-MODEL-GENOME \u06f9\u0641\u0627\u06cc\u0644\u06cc + \u06a9\u0627\u0645\u067e\u0627\u06cc\u0644\u0631 \u067e\u0631\u0627\u0645\u067e\u062a gpt-6-standard-free (\u0628\u062f\u0648\u0646 Think) \u0648 \u0631\u0648\u062a\u06cc\u0646\u06af \u06f2\u06f0\u062a\u0633\u06a9\u06cc",
        "\u0633\u06cc\u0633\u062a\u0645 \u06f2 \u06af\u0627\u0645 \u06f4: OMNI-MODEL-GENOME/ \u0631\u06cc\u0634\u0647\u0654 \u0645\u062e\u0632\u0646 — \u0648\u0631\u06a9‌\u0627\u0633\u067e\u06cc\u0633 plug-and-play \u0645\u0647\u0646\u062f\u0633\u06cc \u0645\u0639\u06a9\u0648\u0633 "
        "\u0645\u062f\u0644‌\u0647\u0627: 00_ECOSYSTEM_INDEX.md (\u062c\u062f\u0648\u0644 \u0645\u0633\u06cc\u0631\u06cc\u0627\u0628\u06cc \u06f2\u06f0 \u062a\u0633\u06a9 \u06a9\u0644\u0627\u0646\u060c \u0647\u062f\u0641 #01 = gpt-6-standard-free "
        "\u0648\u0628 \u0631\u0633\u0645\u06cc \u0628\u0627 Think Mode OFF + \u06f1\u06f1 \u0627\u0633\u0644\u0627\u062a \u0622\u06cc\u0646\u062f\u0647: gpt-6-thinking\u060c claude-opus-thinking\u060c "
        "claude-sonnet-standard\u060c gemini-pro-longcontext\u060c deepseek-r1-reasoner\u060c grok-frontier\u060c "
        "zai-glm-harness\u060c qwen-coder-agent\u060c openai-codex-agent\u060c llama4-cerebras-fast\u060c "
        "opencode-zen-flagship) \u0648 \u0628\u0631\u0627\u06cc \u0647\u0631 \u0645\u062f\u0644 \u06f9 \u0641\u0627\u06cc\u0644 \u0698\u0646\u0648\u0645 01_TOPOLOGY_AND_HARNESS \u062a\u0627 "
        "09_ROUTING_AND_HANDOFF_CARD + \u0627\u0633\u06a9\u0631\u06cc\u067e\u062a‌\u0647\u0627\u06cc compile_for_gpt6_standard.py \u0648 "
        "sanitize_gpt6_standard_output.py. \u0642\u0627\u0646\u0648\u0646 \u0637\u0644\u0627\u06cc\u06cc \u0645\u062f\u0644 \u0628\u062f\u0648\u0646 Think: «Token Order = Thought "
        "Order» — \u067e\u0631\u0633\u06cc\u062f\u0646 \u06a9\u062f/\u062d\u06a9\u0645 \u0627\u0648\u0644 \u0628\u0627\u0639\u062b Premature Commitment \u062f\u0631 \u062a\u0648\u06a9\u0646‌\u0647\u0627\u06cc 1..30 \u0645\u06cc‌\u0634\u0648\u062f\u061b "
        "\u062f\u0631\u0645\u0627\u0646: \u062c\u062f\u0648\u0644 \u067e\u06cc\u0634‌\u0645\u062d\u0627\u0633\u0628\u0647\u0654 <pre_computation_state_trace> (\u062b\u0628\u062a invariant‌\u0647\u0627 + edge case‌\u0647\u0627 + "
        "\u062d\u0627\u0644\u062a‌\u0647\u0627\u06cc \u0634\u06a9\u0633\u062a\u0650 \u0631\u062f\u0634\u062f\u0647) \u0642\u0628\u0644 \u0627\u0632 \u0647\u0631 \u06a9\u062f — \u0628\u0627\u0632\u0633\u0627\u0632\u06cc ~\u06f8\u06f8\u066a \u062f\u0642\u062a Think Mode \u0628\u0627 ~\u06f4× \u0633\u0631\u0639\u062a\u061b "
        "\u06af\u0627\u0631\u062f \u0636\u062f-Canvas/\u0636\u062f-\u0648\u0628‌\u0633\u0631\u0686 (emit inline fenced Markdown\u060c \u0633\u0646\u062a\u0632 \u0642\u0637\u0639\u06cc \u0622\u0641\u0644\u0627\u06cc\u0646)\u061b \u06af\u06cc\u062a‌\u0647\u0627\u06cc "
        "\u0641\u0627\u0632\u06cc ≤\u06f6 \u0642\u0627\u0639\u062f\u0647\u061b \u0642\u0631\u0627\u0631\u062f\u0627\u062f \u0636\u062f-elision (\u0645\u0645\u0646\u0648\u0639\u06cc\u062a «// … existing code»\u060c \u062a\u0631\u062c\u06cc\u062d \u0628\u0644\u0648\u06a9 "
        "SEARCH/REPLACE \u0628\u0627 ≥\u06f3 \u062e\u0637 \u0627\u0646\u06a9\u0631). \u0627\u0633\u06a9\u0648\u0631\u06a9\u0627\u0631\u062f \u06f2\u06f0\u062a\u0633\u06a9\u06cc gpt-6-standard-free: TechnicalGenomeSynthesis "
        "0.91\u060c HardAlgorithmicDeduction 0.88\u060c SystemArchitecture 0.87\u060c SpecComplianceReview 0.86\u060c "
        "AdversarialTestGeneration 0.85\u060c StrictJsonExtraction 0.84\u060c SqliteLedgerDesign 0.83\u060c "
        "XAlgorithmViralGrowth 0.43\u060c LongHorizonAutonomousLoop 0.39 (Do-Not-Route). \u062a\u0632\u0631\u06cc\u0642 \u06a9\u0631\u0646\u0644: "
        "crates/ai/src/omni_genome_router.rs — GenomeLayerId \u0628\u0627 \u0646\u0627\u0645‌\u0647\u0627\u06cc canonical\u060c "
        "EngineeringWorkload \u06f2\u06f0\u062a\u0627\u06cc\u06cc\u060c load_model_genome_from_dir (\u0627\u0639\u062a\u0628\u0627\u0631\u0633\u0646\u062c\u06cc \u06f9 \u0641\u0627\u06cc\u0644 + frontmatter + "
        "\u067e\u0627\u0631\u0633 \u0627\u0633\u06a9\u0648\u0631\u06a9\u0627\u0631\u062f\u061b \u0627\u0645\u062a\u06cc\u0627\u0632 \u0646\u0627\u0645\u0639\u0644\u0648\u0645 \u0647\u0631\u06af\u0632 \u062e\u0648\u062f\u0633\u0627\u0632\u06cc \u0646\u0645\u06cc‌\u0634\u0648\u062f)\u060c builtin_gpt6_standard_profile \u0648 "
        "builtin_ecosystem_profiles (\u0635\u0641\u0631-IO)\u060c compile_prompt_for_model (\u062a\u0632\u0631\u06cc\u0642 state-trace/Canvas/\u0641\u0627\u0632/"
        "anti-elision) \u0648 HandoffPacket \u0647\u0645‌\u0627\u0633\u06a9\u06cc\u0645\u0627 \u0628\u0627 07_LINK_Protocol. \u0645\u0646\u0628\u0639: \u06af\u0646\u062c\u06cc\u0646\u0647\u0654 "
        "omni-model-genome-gpt6-standard (Wave 2 - Step 4).",
    ),
]

# ── 2. Idempotent upsert ─────────────────────────────────────────────────────


def upsert_kb(db: sqlite3.Connection) -> dict:
    """Compare-then-write: unchanged rows are skipped, changed rows replaced."""
    stats = {"inserted": 0, "updated": 0, "skipped": 0}
    cur = db.cursor()
    for kb_id, title, body in KB_ITEMS:
        row = cur.execute('SELECT "title","body" FROM "KbEntry" WHERE "id"=?', (kb_id,)).fetchone()
        if row is not None and row[0] == title and row[1] == body:
            stats["skipped"] += 1
            continue
        cur.execute(
            'INSERT INTO "KbEntry" ("id","title","body","createdAt","updatedAt") VALUES (?,?,?,?,?)'
            ' ON CONFLICT("id") DO UPDATE SET "title"=excluded."title", "body"=excluded."body",'
            ' "updatedAt"=excluded."updatedAt"',
            (kb_id, title, body, CREATED_AT, CREATED_AT))
        if row is None:
            stats["inserted"] += 1
        else:
            stats["updated"] += 1
    return stats


# ── 3. Verification ──────────────────────────────────────────────────────────


def verify(db: sqlite3.Connection) -> dict:
    cur = db.cursor()
    kb = cur.execute('SELECT COUNT(*) FROM "KbEntry"').fetchone()[0]
    active = cur.execute('SELECT COUNT(*) FROM "Product" WHERE "active"=1').fetchone()[0]
    citizens = cur.execute('SELECT COUNT(*) FROM "User"').fetchone()[0]
    referrals = cur.execute('SELECT COUNT(*) FROM "ReferralLineage"').fetchone()[0]
    escrows = cur.execute('SELECT COUNT(*) FROM "Escrow"').fetchone()[0]
    trial = cur.execute("SELECT COALESCE(SUM(amount),0) FROM Posting").fetchone()[0]
    timings = []
    for q in ("429 \u062c\u0639\u0644\u06cc", "shell_exec", "Bearer public", "KIA-BRAIN-OS", "pre_computation_state_trace"):
        t0 = time.perf_counter()
        rows = cur.execute(
            'SELECT COUNT(*) FROM "KbEntry" WHERE "title" LIKE ? OR "body" LIKE ?',
            (f"%{q}%", f"%{q}%")).fetchone()[0]
        timings.append((q, rows, (time.perf_counter() - t0) * 1000))
    return {
        "kb": kb, "active": active, "trial_sum": trial,
        "citizens": citizens, "referrals": referrals, "escrows": escrows,
        "search": timings,
    }


def main() -> int:
    assert REPO.is_dir() and SEED_DB.is_file(), "repo/seed db missing"

    before = verify(sqlite3.connect(SEED_DB))
    print(f"[1/3] before: KbEntry={before['kb']} | active products={before['active']} | "
          f"citizens={before['citizens']} | referrals={before['referrals']} | "
          f"escrows={before['escrows']} | trial balance sum={before['trial_sum']}")

    db = sqlite3.connect(SEED_DB)
    db.row_factory = sqlite3.Row
    try:
        ks = upsert_kb(db)
        db.commit()
    finally:
        db.close()
    print(f"[2/3] kb upsert: +{ks['inserted']} new, {ks['updated']} updated, {ks['skipped']} skipped")

    after = verify(sqlite3.connect(SEED_DB))
    print(f"[3/3] after: KbEntry={after['kb']} (56 -> 62 expected) | active products={after['active']} | "
          f"citizens={after['citizens']} | referrals={after['referrals']} | escrows={after['escrows']} | "
          f"trial balance sum={after['trial_sum']}")
    for q, rows, ms in after["search"]:
        flag = "OK" if ms < 5.0 else "SLOW"
        print(f"      search '{q}': {rows} hits in {ms:.2f} ms [{flag}]")

    # sync to download deliverable
    DOWNLOAD_DB.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(SEED_DB, DOWNLOAD_DB)
    print(f"      synced seed db -> {DOWNLOAD_DB}")

    ok = (
        after["kb"] == 62 and after["active"] == 20 and after["trial_sum"] == 0
        and after["citizens"] == 466 and after["referrals"] == 247 and after["escrows"] == 108
        and ks["updated"] == 0 and (ks["inserted"] == 6 or ks["skipped"] == 6)
        and all(ms < 5.0 for _, _, ms in after["search"])
    )
    print("RESULT:", "PASS" if ok else "FAIL")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
