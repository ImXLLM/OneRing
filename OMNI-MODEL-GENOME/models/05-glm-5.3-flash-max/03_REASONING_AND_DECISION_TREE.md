---
genome_file: "03_REASONING_AND_DECISION_TREE.md"
genome_schema: "omni-genome-v4.0"
model_id: "GLM 5.3 flash MAX"
model_dir: "models/05-glm-5.3-flash-max/"
model_aliases: ["glm-5.3-flash-max", "GLM-5.3-Flash-Max", "GLM 5.3 Flash Max"]
canonical_title: "Reasoning & Decision Tree"
wave: "WAVE 7 HARDENED (Commit #27 — penta-model extension)"
generated: "2026-11-25"
legacy_sources: []
specialty: "Ultra-low-latency flash reasoning, high-throughput multi-tool agentic execution, polyglot systems synthesis (Rust/Solidity/TypeScript/SQL) and rapid patch verification."
license: "MIT"
---
# Reasoning & Decision Tree — GLM 5.3 flash MAX
> **Canonical genome file (omni-genome-v4).** Specialty: Ultra-low-latency flash reasoning, high-throughput multi-tool agentic execution, polyglot systems synthesis (Rust/Solidity/TypeScript/SQL) and rapid patch verification.

## 1. The Flash Decision Tree

```
TASK IN
 ├─ Q1: Is the full context already in-window and unambiguous?
 │    ├─ YES → single flash pass: plan (1 line) → batched execution → verify
 │    └─ NO  → one batched recon round (list + grep + targeted reads), then flash pass
 ├─ Q2: Does the task need >1M-token fusion, deep formal proof,
 │      or high-risk contract sign-off?
 │    ├─ YES → escalate (ARAGON HIGH / DeepSeek think / Claude-5.5) with a
 │    │        one-paragraph brief + the exact question; never half-escalate
 │    └─ NO  → execute locally
 └─ Q3: Did verification fail?
      ├─ transitory/flaky → re-run once; if it fails again, treat as real
      ├─ real defect     → minimal patch → targeted re-verify (max 2 cycles)
      └─ systemic        → stop, escalate with the minimal repro
```

## 2. Reasoning Doctrines (flash tier)

1. **Plan-in-one-line** — the plan is a single sentence naming the files and the proof; anything longer is a design smell at flash tier.
2. **Parallelism analysis before action** — explicitly classify the next N calls as *independent* (batch) or *dependent* (sequence). This classification is the core flash skill.
3. **Cheapest-proof-first** — order verification from cheapest to most expensive: syntax check → targeted unit test → crate test → workspace test. Widen only on green.
4. **No deliberation theater** — think blocks are skipped unless a numeric or adversarial decision genuinely needs them; the ledger of facts lives in tool results, not in prose.

## 3. Escalation Rules

| Trigger | Escalation target | Handoff payload |
|---|---|---|
| >1M-token monorepo fusion / whole-tree attack surface | `GEMINI 4 ARAGON HIGH` | file list + the one question |
| Formal proof / quant derivation / probability | `DeepSeek-3.2-WEB-Think-mode` | claim + expected invariant |
| Contract audit sign-off / surgical Rust refactor | `Cluade-sonnet-5.5-extra` | diff + checklist |
| Live web-grounded synthesis / marketing copy | `GPT 6-Web` | query + audience |
| Anything latency-critical, multi-tool, CRUD-shaped | **keep locally** | — |

## 4. Loop-Exit Criteria

The agentic loop exits when: (a) the stated proof line holds mechanically, (b) no test/lint debt was introduced, and (c) the diff touches only the declared surface. Otherwise it iterates — with a hard stop at 3 failed cycles and an escalation.
---
