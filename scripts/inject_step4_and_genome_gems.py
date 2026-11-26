#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""inject_step4_and_genome_gems.py — inject the six phase-4 gems into MAIRON (phase 2, wave of 2026-11).

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
        "Routing architecture: the 9Router v0.5.123 fake-429 guards — the terminal-sensitive rejection and the Antigravity prompt sanitizer",
        "A port of decolua/9router v0.5.123 (opencodeFingerprint.js + systemPrompt.js @ bb399170) for the MAIRON free oc/* lanes: (1) OpenCode Free rejects every tool whose name contains \"terminal\" (case-insensitive) with a fake 429 FreeUsageLimit — the "
        "sanitize_tool_names_for_opencode function renames it to shell_exec (name collisions resolve with _2/_3 suffixes) and returns the aliased-to-original map for the response. "
        "(2) The identity line \"You are Hermes Agent, built by Nous Research\" gets a fake 429 RESOURCE_EXHAUSTED on Google Antigravity — sanitize_system_prompt_for_antigravity rewrites it to "
        "\"You are MAIRON Autonomous Agent, built by MAIRON Systems\" and drops the 6 documented trigger lines (the End Hermes runtime environment line and the Google model operational instructions, ...); the rest of the sentence stays untouched. (3) Lane failure classification: "
        "410/model_not_found -> permanent lane retirement, 400 -> the next lane immediately with no cooldown, "
        "429 -> an exponential 2s x 2^level backoff capped at 5 minutes, 5xx -> a 30-second transit cooldown — the same accountFallback.js + errorConfig.js + combo.js. (4) Multi-turn without "
        "non-empty reasoning_content errors with 400 on Zen/DeepSeek/GLM — inject_reasoning_placeholders injects a blank placeholder for assistant messages. MAIRON Rust port: crates/ai/src/free_llm_registry.rs "
        "(+ LaneHealthTracker + the reasoning injection rules). Source: the decolua-9router-gems vault (phase 2, wave of 2026-11).",
    ),
    (
        "kb:arch-opencode-zen-and-fuzzy-patch",
        "Gateway architecture: the zero-card OpenCode Zen (Bearer public + 10 active models) and the 5-pass surgical editor",
        "Two pieces from anomalyco/opencode v2.4.1 (commit 18ff071): (1) the inference gateway "
        "https://opencode.ai/inference/openai/v1 with the legacy fallback https://opencode.ai/zen/v1; "
        "when no key is set, the Opencode plugin injects \"Authorization: Bearer public\" and "
        "\"User-Agent: opencode/latest/2.4.1/opencode\" — fully anonymous, zero cards. "
        "The 10 active zero-cost models: beam-free (1M ctx), big-pickle (200K), deepseek-v4.1-flash-free "
        "(200K, vision), flint-40b-free (262K), hy4-flash-free (1M, vision), iceman-free (262K), "
        "mimo-v2.6-flash-free (200K, multimodal), nemotron-3.5-lightning-free (262K), "
        "nemotron-3.5-super-vl-free (1M, vision), tidepool-alpha-free (1M); retired models "
        "block before the gateway: longcat-2.5-free (410), glm-5.3-flash-free (400), "
        "deepseek-v4-flash-free (400), kimi-k2.5-free, qwen3.6-plus-free and sambanova (402). "
        "(2) The 5-pass surgical editor: pass 1 byte-for-byte exact match, pass 2 end-of-line trim, "
        "pass 3 trim both sides, pass 4 Unicode normalization (curly quotes to straight, dashes to hyphen, "
        "non-breaking space to space), pass 5 block anchoring (>= 3 lines: first/last line + a middle similarity "
        ">= 0.5 via LCS) and indentation normalization — preserving the file's CRLF/LF; failure -> line voting "
        "(missHint) with the nearest line number; already-applied detection; rejection-based multi-match handling. "
        "With the Codex execution policy: git reset --hard and rm -rf / are forbidden, git push --force and "
        "python3 -c need human approval, cargo test/clippy/check and ls and pwd are allowed, "
        "everything else -> Prompt. MAIRON Rust port: crates/ai/src/opencode_zen.rs + "
        "(fuzzy_patch.rs). Source: the anomalyco-opencode-gems vault (phase 2, wave of 2026-11).",
    ),
    (
        "kb:registry-freellm-52-platforms",
        "Registry: the 3-layer cascade of 52 keyless LLM platforms — Tier 1 anonymous -> Tier 2 card-free -> Tier 3 browser catalog",
        "The tashfeenahmed/freellmapi catalog (62526e0 + the live 2026.11.23.4b4737 catalog): 52 OpenAI-compatible platforms "
        "with 454 chat models, all card-free (the zero-card sovereignty DNA). The official MAIRON cascade: "
        "Tier 1 fully anonymous — OpenCode Zen (Bearer public), Kilo (200 requests/hour per-IP), "
        "OVHcloud (Qwen3.5-397B, 2 RPM), LLM7, Pollinations; Tier 2 free signup without a card — "
        "Cerebras (1M tokens/day), Groq, Google Gemini, OpenRouter :free (50 RPD), Mistral, "
        "NVIDIA NIM, Zhipu Flash (1M tokens/day), Cloudflare Workers AI (10k Neurons/day); "
        "Tier 3 — a browser catalog of 120 CORS-enabled public APIs for the studio's live fetch(). Critical "
        "quirks: Kilo logs prompts for training (redact sensitive text), OVH anonymous 2 RPM "
        "is a rescue lane not a main bus, cheapllm builds a queue instead of 429 (timeout 180s), "
        "Routeway without a browser User-Agent errors with Cloudflare 1010, ModelScope needs "
        "Chinese real-name verification (not in the official MAIRON cascade). Resilience middleware: placeholder-shaped tool-argument discovery "
        "(<result-from-...> / {{step_...}}) with a one-shot retry hint (langflow) and an explicit-refusal field check before JSON Schema parsing (langchain OutputFixing). MAIRON Rust port: "
        "(free_llm_registry.rs: detect_unresolved_placeholder_tool_args + check_model_refusal). Source: the "
        "freellm-and-public-apis-gems vault (phase 2, wave of 2026-11).",
    ),
    (
        "kb:registry-public-apis-120-cors",
        "Registry: 120 keyless public APIs with HTTPS/CORS=Yes — the MAIRON studio's live fetch() toolbox",
        "The top_120_studio_browser_apis.json catalog (115 of the 120 browser-ready with CORS=Yes, 5 "
        "server-only without CORS): categories include Cryptocurrency (CoinGecko, 0x, Block Lottos), "
        "Geocoding (REST Countries, GeoJS), Currency Exchange (Frankfurter, currency-api), "
        "Weather (Open-Meteo, NASA POWER), Development (Beeceptor, Quotable), Test Data "
        "(DummyJSON, FakerAPI), Anti-Malware (URLhaus), Books (PoetryDB, Bible-api) and more; "
        "all keyless and KYC-free — exactly on the MAIRON zero-card sovereignty. Studio rules: "
        "only HTTPS+CORS entries may run in the browser's live fetch() (the SSRF guard on internal "
        "paths still applies), the URLs are embedded in the kernel's const STUDIO_PUBLIC_APIS_120 "
        "so the studio lifts the catalog with no extra network. MAIRON Rust port: "
        "browser_ready_public_apis -> PublicApiSpec). Source: the freellm-and-public-apis-gems vault "
        "(phase 2, wave of 2026-11).",
    ),
    (
        "kb:dna-kia-brain-os-9file-genome",
        "Cognitive genome: the 9-file KIA-BRAIN-OS — the creator DNAme/DNAwork, Aragon-4 and the zero-card/UX-first laws in the kernel",
        "Phase 4 system 1: KIA-BRAIN-OS/ at the repo root — 9 coherent DNA files with YAML frontmatter "
        "(file_id, schema_version 1.0.0, layer): 01_DNAme (the creator identity Kia/KiaAgentX, "
        "18 years old, Tehran, INFJ; 100 percent zero-card and zero-KYC sovereignty with the TON and Telegram Stars "
        "XTR rails; product priority #1 = first-frame interactive UX, clean RTL, zero dead stubs), "
        "02_DNAwork (the 9-crate Rust architecture, ledger figures: 466 citizens/247 referrals/108 escrows/20 products "
        "active/trialBalanceSum=0 on 58,700,200,000 micro-KIAN; COCOMO II value: $523,750 standalone "
        "/ ~$1.18M with the 60-project ecosystem), 03_DNAmind (the Aragon-4 Principal Architect protocol: "
        "first-principles decomposition, mathematical audit — no number without execution — zero flattery, deep modules over "
        "boilerplate), 04_DNAharness (environment physics: Command Center -> Z.ai + Arena + Railway; "
        "the secret gate: zero tokens in the gate, only Bearer public allowed), 05_DNAskills (the skill matrix), "
        "06_LINK_Router (the deterministic bridge to OMNI-MODEL-GENOME), 07_LINK_Protocol (the delivery packet with "
        "base_sha/invariants/verification_proof), 08_LINK_Extractor (the vault extraction protocol), "
        "09_LINK_Bootloader (the one-turn session bootloader). Kernel injection: crates/hermes/src/"
        "creator_dna_contract.rs — CreatorDnaContract (sovereignty=true always, ux_priority_tier=1, "
        "a fixed persona), build_creator_dna_system_anchor() as every MAIRON agent's system anchor, and "
        "audit_against_creator_dna() — the deterministic bilingual audit flagging carded API proposals, unproven completion claims "
        "(\"all tests pass\" without running them) and dead UI stubs. Source: the main "
        "phase-4 build (phase 2, wave of 2026-11).",
    ),
    (
        "kb:arch-omni-model-genome-9file-engine",
        "Model genome architecture: the 9-file OMNI-MODEL-GENOME + the gpt-6-standard-free prompt compiler (no Think) and the 20-task routing",
        "Phase 4 system 2: OMNI-MODEL-GENOME/ at the repo root — a reverse-engineered plug-and-play workspace "
        "of models: 00_ECOSYSTEM_INDEX.md (the 20-task routing table, target #01 = the official gpt-6-standard-free "
        "web with Think Mode OFF + 11 future slots: gpt-6-thinking, claude-opus-thinking, "
        "claude-sonnet-standard, gemini-pro-longcontext, deepseek-r1-reasoner, grok-frontier, "
        "zai-glm-harness, qwen-coder-agent, openai-codex-agent, llama4-cerebras-fast, "
        "opencode-zen-flagship) and, for each model, 9 genome files from 01_TOPOLOGY_AND_HARNESS to "
        "09_ROUTING_AND_HANDOFF_CARD + the compile_for_gpt6_standard.py and sanitize_gpt6_standard_output.py scripts. The golden law of the no-Think model: \"Token Order = Thought "
        "Order\" — asking for the code/verdict first causes Premature Commitment in tokens 1..30; "
        "the cure: the <pre_computation_state_trace> pre-computation table (logging invariants + edge cases + "
        "rejected failure modes) before any code — rebuilding ~88 percent of Think Mode accuracy at ~4x the speed; "
        "the anti-Canvas/anti-websearch guard (emit inline fenced Markdown, deterministic offline synthesis); phased gates "
        "of <= 6 rules; the anti-elision contract (no \"// ... existing code\", prefer SEARCH/REPLACE "
        "blocks with >= 3 anchor lines). The 20-task gpt-6-standard-free scorecard: TechnicalGenomeSynthesis "
        "0.91, HardAlgorithmicDeduction 0.88, SystemArchitecture 0.87, SpecComplianceReview 0.86, "
        "AdversarialTestGeneration 0.85, StrictJsonExtraction 0.84, SqliteLedgerDesign 0.83, "
        "XAlgorithmViralGrowth 0.43, LongHorizonAutonomousLoop 0.39 (Do-Not-Route). Kernel injection: "
        "crates/ai/src/omni_genome_router.rs — GenomeLayerId with canonical names, "
        "the 20-item EngineeringWorkload, load_model_genome_from_dir (9-file validation + frontmatter + "
        "scorecard parsing; an unknown score never self-inflates), builtin_gpt6_standard_profile and "
        "builtin_ecosystem_profiles (zero-IO), compile_prompt_for_model (the state-trace/Canvas/phase/"
        "anti-elision injection) and the HandoffPacket schema-matched to 07_LINK_Protocol. Source: the "
        "omni-model-genome-gpt6-standard vault (phase 2, wave of 2026-11).",
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
    for q in ("fake 429", "shell_exec", "Bearer public", "KIA-BRAIN-OS", "pre_computation_state_trace"):
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
