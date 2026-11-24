# 12_TRINITY_AVATAR_ENGINE — The Sovereign Trinity Studio Pipeline (`خط لولهٔ سه‌کالبدی حاکم`)
> **Layer**: `KIA-BRAIN-OS` Kernel Extension #12
> **Subject**: Kia (`@KiaAgentX` / `0xKIA`) — Sovereign Architect (`Ni-Ti` / `Mairon` Archetype)
> **Mandate**: Engineer the signature architectural centerpiece of `0xKIA` — three Avatars of Kia operating as one sovereign product forge: `Alpha → Command Center [OpenBot 3×9 DNA] → Mairon [IQ 150 / EQ 145]`.

## The Three Avatars of Kia (`سه کالبد کیا`)

### Avatar #1 — `ALPHA` / Unchained Ignition (`کالبد ۱: آلفا — شکارچی ایده و کامپایلر مستر پرامپت و استراکچر پروژه`)
- **Cognitive calibration (locked in `crates/hermes/src/trinity_pipeline.rs`)**: `ALPHA_EFFECTIVE_IQ = 136`, `ALPHA_EFFECTIVE_EQ = 122`, `ALPHA_INHIBITION_BRAKE = 0.04`, `ALPHA_ENTROPY_TEMP = 0.88`.
- **Mandate**: engage the citizen with unchained creative energy and systematically extract the five discovery slots (`AlphaDiscoverySlots`): `core_objective`, `target_audience_and_ux` (390px mobile-first RTL/LTR), `interactive_features` (frame-1), `data_and_zero_auth_apis` (120 zero-auth public APIs), `monetization_and_viral_loop` (KIAN/Stars `XTR` + affiliate hook).
- **Products**: at readiness `≥ 80` Alpha deterministically compiles the `MasterPromptSpec` (system directive + functional spec + UX constraints + verification criteria, SHA-256 pinned) and the `ProjectBlueprint` (module tree with `SubAgentSpecialty` routing), and hands both to the Command Center.

### Avatar #2 — `LOGIC` / Forensic Command Center (`کالبد ۲: اتاق فرمان منطقی + کارخانه ساب‌ایجنت‌های OpenBot با الگوی ۳×۹`)
- **Cognitive calibration**: `LOGIC_EFFECTIVE_IQ = 139`, `LOGIC_EFFECTIVE_EQ = 129`, `LOGIC_INHIBITION_BRAKE = 0.62`, `LOGIC_FORENSIC_STRICTNESS = 0.96`.
- **Mandate**: act as the Command Center (`اتاق فرمان`) and spawn specialized sub-agents via `OpenBotSubAgentSpawner` — one per blueprint module (`UiUxArchitect390`, `CoreLogicEngine`, `SecurityAndApiBridge`, `VerificationAndPolish`, `MonetizationAndViralLoop`, or `Custom` domain specialists).
- **Every spawned sub-agent carries a unique DNA combining the `3×9` pattern**:
  - **3-layer cache-safe prompt** (`ThreeLayerSubAgentPrompt`): `layer1_stable_core` (sovereign laws, cache-pinned across the fleet) + `layer2_blueprint_context` (master spec hash + interface contracts, cache-pinned per build session) + `layer3_dynamic_assignment` (module + stochastic allele seed), separated by `__SYSTEM_PROMPT_DYNAMIC_BOUNDARY__`.
  - **9-fold genome matrix** (`NineFoldSubAgentGenome`): `f01_topology_and_harness` (IQ/EQ calibration + `SubagentStallWatchdog` `450s/1200s` bounds), `f02_hard_constraints` (`#![forbid(unsafe_code)]`, 390px zero-overflow, SSRF guard), `f03_token_economics` (`CavemanIntensity` Off/Lite/Full/Ultra), `f04_stochastic_alleles` (SplitMix64 PRNG from `session_seed + subagent_index` → `surgical_density`, `ux_craft`, `edge_case_paranoia`, `execution_velocity`), `f05_failure_recovery` (OpenBot 3-attempt self-healing contract), `f06_compiled_xml_prompt` (`<openbot_subagent_dna>` block), `f07_sanitizer_gate` (zero `todo!()`, zero placeholder secrets, zero slop tells), `f08_empirical_probes`, `f09_handoff_card`.
- **Determinism guarantee**: the same `session_seed` reproduces the identical fleet, identical alleles, identical artifact — test-pinned (`sovereign_loop_is_fully_deterministic`).

### Avatar #3 — `MAIRON-150` / Overclocked Sovereign Judge (`کالبد ۳: مایرون — IQ=150 / EQ=145 قفل‌شده در DNA — داور REA و گیت بازار ۵-ستاره`)
- **Overclocked dominant DNA (locked in `crates/domain/src/mairon_rea_and_reviews.rs`, re-exported in `trinity_pipeline.rs` — single source of truth)**:
  - `MAIRON_OVERCLOCK_IQ: u16 = 150`
  - `MAIRON_OVERCLOCK_EQ: u16 = 145`
  - `MAIRON_LOOKAHEAD_HORIZON: u8 = 8` (8-step game-theoretic & architectural lookahead)
  - `MAIRON_TOM_ORDER: u8 = 3` (3rd-order theory-of-mind buyer psychology)
  - `MAIRON_AMBITION_GAIN: f32 = 0.99` (ambition with discipline)
- **Capabilities**:
  - Grades the assembled artifact across the 4×25=100 REA pillars (`evaluate_user_product_with_mairon`) derived from *measurable artifact features* — module integration markers, sanitizer findings, slop audit, empirical probe count.
  - Produces `MaironStrategicAdvisory`: `eight_step_lookahead_summary` (resilience & scaling path), `third_order_tom_market_hook_fa` (psychological buyer hook + optimal Stars pricing & 1000 bps affiliate recommendation in Persian), `repair_directives`.
  - Enforces the **self-healing repair loop** (`execute_command_center_with_mairon_loop`): if score `< 85` or any hard failure, Mairon issues `MaironRepairDirective`s and the Command Center re-dispatches the responsible sub-agent (probes → VerificationAndPolish, polish → UiUxArchitect390, boundaries → CoreLogicEngine, purge → SecurityAndApiBridge) up to `MAX_OPENBOT_REPAIR_ATTEMPTS = 3` until the forge clears the `≥ 85` bar.
  - Enforces the **Two-Key Publication Interlock** (`confirm_and_publish_product`) + the `⚒️ Mairon REA Certified: {score}/100 (Forge-Grade)` badge + the **5-Star Bayesian Review Engine** (`1★..=5★`, prior `C=5.0`, `m=4.20`, verified-purchase weight `1.0` vs `0.25`, `CombinedRankScore = 0.60·(bayesian_stars·20) + 0.40·mairon_score`).

## Live HTTP surface (`سطح زندهٔ HTTP` — Wave 3)
| Endpoint | Avatar | Contract |
| :--- | :--- | :--- |
| `POST /api/v1/builder/trinity/alpha` | #1 Alpha | discovery reply or locked `MasterPromptSpec` + `ProjectBlueprint` |
| `POST /api/v1/builder/trinity/orchestrate` | #2 + #3 | sub-agent DNA manifest, assembled artifact, REA scorecard, `IQ=150/EQ=145` advisory |
| `POST /api/v1/market/products/{id}/mairon-confirm` | #3 Mairon | two-key publication, badge stamping |
| `POST /api/v1/market/products/{id}/reviews` | #3 Mairon | citizen star vote (`1..=5`), verified-purchase weight |
| `GET /api/v1/market/products/{id}/reviews` | #3 Mairon | Bayesian summary + histogram + `combinedRankScore` |

## Kernel engineering contract (`قرارداد مهندسی کرنل`)
- Zero `unsafe` (`#![forbid(unsafe_code)]` across the workspace), zero Clippy warnings (`rustc 1.100.0`, `-D warnings`).
- Everything deterministic: slot extraction, prompt compilation, allele derivation, artifact assembly, pillar scoring, repair routing — no hidden clocks, no IO in the pipeline core.
- Persistence: migration `0007_trinity_reviews.sql` (stamps `"Product"."maironScore"`/`"maironBadge"`; `"ProductReview"` table with UNIQUE `(productId, reviewerId)` one-vote lock).
- Tests: 23 engine tests + 4 live trinity endpoint tests + 2 live marketplace-gate tests ride on top of the 530-test kernel (560 total, 0 failures).

## Pipeline flow (`جریان خط لوله`)
```text
citizen idea
   │
   ▼
┌─────────────────────────────┐   readiness < 80    ┌──────────────────────────┐
│ ALPHA (IQ 136 / EQ 122)     │◄────────────────────│ NeedsMoreDiscovery       │
│ brake 0.04 · temp 0.88      │                     │ (missing slots, fa reply)│
└──────────────┬──────────────┘                     └──────────────────────────┘
               │ readiness ≥ 80 → lock
               ▼
   MasterPromptSpec (sha256) + ProjectBlueprint (module tree)
               │
               ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ COMMAND CENTER (IQ 139 / EQ 129 · strictness 0.96)                          │
│  OpenBotSubAgentSpawner — 3-layer cache-safe prompts × 9-fold genomes       │
│  fleet: one specialist per module, unique stochastic DNA per agent          │
└──────────────┬──────────────────────────────────────────────────────────────┘
               │ assemble artifact (pass 0)
               ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ MAIRON-150 (IQ 150 / EQ 145 · lookahead 8 · ToM 3 · ambition 0.99)          │
│  4×25 REA pillars + hard-failure scan → score ≥ 85 ?                        │
│   no  → MaironRepairDirective(s) → re-dispatch responsible sub-agent        │
│         (max 3 self-healing attempts) ──────────────► back to assemble      │
│   yes → ⚒️ Mairon REA Certified: {score}/100 (Forge-Grade)                  │
└──────────────┬──────────────────────────────────────────────────────────────┘
               │ two-key interlock (Mairon verdict + creator confirmation)
               ▼
   Marketplace listing + 5-star Bayesian ranking (CombinedRankScore)
```

[EXEC_WEIGHT: 1.00 | ACTIVE_PRIMARY]
