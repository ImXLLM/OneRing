---
file_id: "08_LINK_Extractor"
schema_version: "1.0.0"
layer: protocol
purpose: "forensic harvesting"
language: "fa-IR + en-US"
---

# 08 — LINK_Extractor: The Vault Extraction Protocol

## 1. Input → Output

| Input | Output | Destination |
|---|---|---|
| `upload/*-gems.zip` | numbered step folders | `/tmp/stepN_gems/<vault>/` |
| open reference repo | pure algorithm (no boilerplate) | Rust kernel + tests |
| frontier model (Arena) | 9-file genome | `OMNI-MODEL-GENOME/models/<id>/` |
| every exploration | knowledge row | `"KbEntry"` (idempotent) |

## 2. The Selection Rule (harvest bar)

1. Only **algorithm-centric** files: ranking engines, patch editors, security guards,
   fallback cascades — junk logs and sample UIs are not extracted.
2. Mount cap: **<100KB/file** — a larger file must be trimmed or summarized.
3. Every mount must state its kernel port path in its body.

## 3. The Model Autopsy Protocol (Arena → 9 files)

- The `01..09` files use the same canonical names as in `omni_genome_router.rs`.
- Mandatory YAML frontmatter: `file_id`, `schema_version: "1.0.0"`,
  `model_self_reported_id`, `calibration_status`.
- No number without evidence — guesses carry the `unverified_hypothesis` label.
- Rust route flags read from the genome:
  `think_mode_active`, `requires_external_state_trace`, `suppress_web_canvas_hijack`,
  `max_simultaneous_rules_per_phase`.

## 4. Mandatory End-of-Extraction Sanitization

Every extraction path ends with the secret scanner (the `sanitize_step4_vaults.py` pattern);
anything other than `Bearer public` means failure — it is cleaned before commit.
