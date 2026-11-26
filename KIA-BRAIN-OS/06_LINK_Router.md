---
file_id: "06_LINK_Router"
schema_version: "1.0.0"
layer: bridge
target: "OMNI-MODEL-GENOME/00_ECOSYSTEM_INDEX.md"
language: "fa-IR + en-US"
---

# 06 — LINK_Router: The Deterministic Bridge between the Creator Brain and the Model Genome

## 1. The Routing Contract

`KIA-BRAIN-OS` (identity and rules) never selects a model directly; it reaches the model genome
through the following deterministic route:

```text
KIA-BRAIN-OS/06_LINK_Router.md
  → OMNI-MODEL-GENOME/00_ECOSYSTEM_INDEX.md          (table of 20 macro tasks)
  → OMNI-MODEL-GENOME/models/<model_id>/09_ROUTING_AND_HANDOFF_CARD.md (scorecard)
  → crates/ai/src/omni_genome_router.rs              (Rust port: profile + compile_prompt_for_model)
```

## 2. The Selection Rule (deterministic)

1. Determine the task's `EngineeringWorkload` (20 standard tasks).
2. Take the highest `workload_scores` among the available profiles; on a tie →
   the lighter `cost_speed_tier` wins; on a further tie → alphabetical order of model_id.
3. An unknown score (no profile) = **never fabricated** —
   "unknown" is returned and the fallback path (the zero-card registry) is taken.

## 3. Fixed Bridges

| From | To | Rule |
|---|---|---|
| `05_DNAskills.md` | `KbEntry` | every skill = one idempotent row with source |
| `07_LINK_Protocol.md` | `HandoffPacket` | same schema, no invented fields |
| `04_DNAharness.md` | Arena | autopsy output lives only under `models/<id>/` |

## 4. Bridge Prohibitions

- Manually editing a model's `00..09` files outside the Arena process is forbidden.
- Writing a score for a model without a 09 card is forbidden (provisional only, with a label).
- The deterministic route never replaces the human verdict of the Principal Architect — it only
  provides a ranked recommendation.
