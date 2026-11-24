# OMNI-MODEL-GENOME — Ecosystem Index

**Audit date:** 2026-11-24. **Evidence status:** two provisional profiles are present. Both model identities/configurations and all scorecards are user-supplied claims without authenticated live sessions or comparative benchmarks. Treat scores as qualitative routing estimates, not measured telemetry.

## Fleet

| Model ID | Reasoning paradigm | Context sweet spot | Cost/speed tier | Top roles | Status |
|---|---|---|---|---|---|
| `gpt-6-standard-free` | Product-mode behavior not established; direct-response preference only | Unknown (no verified token telemetry) | Free-tier label supplied by user; speed/cost unmeasured | Structured synthesis, coding drafts with tests, prompt compilation | Provisional profile; unverified identity/configuration |

Profile: [`models/gpt-6-standard-free/`](models/gpt-6-standard-free/). Prompt compiler: [`scripts/compile_for_gpt6_standard.py`](scripts/compile_for_gpt6_standard.py).

## 20-task suitability matrix

Scores are qualitative estimates copied/adapted from the supplied routing card. They are not pass rates. `Primary` means a reasonable first executor with verification; the other models are deliberately marked unassigned because none were supplied or benchmarked.

| # | Workload | gpt-6-standard-free (estimate) | Role | Reviewer / verifier | Zero-cost fallback |
|---:|---|---:|---|---|---|
| 1 | System architecture and module design | 0.87 | Primary, verify against repo | Human/independent reviewer | Unassigned |
| 2 | Complex Rust, async lifetimes, lock-free/CAS | 0.68 | Scaffold + specialist review | Rust specialist + compiler/tests | Unassigned |
| 3 | Multi-file refactor | 0.73 | Conditional executor | Repo-aware reviewer + test suite | Unassigned |
| 4 | Interactive frontend/UI systems | 0.70 | Conditional executor | Browser/accessibility review | Unassigned |
| 5 | 3D/WebGL/shaders/physics | 0.55 | Fallback only | Rendering specialist/runtime | Unassigned |
| 6 | DB schema, WAL, ledgers | 0.83 | Primary, verify transaction behavior | DB tests/independent reviewer | Unassigned |
| 7 | Algorithms, constraints, formal logic | 0.88 | Primary with explicit steps | Solver/proof checker where critical | Unassigned |
| 8 | Concurrency/Heisenbug RCA | 0.70 | Hypothesis and experiment design | Traces, race detector, specialist | Unassigned |
| 9 | Defensive security / sandbox design | 0.82 | Primary draft, security review | Qualified security reviewer + tests | Unassigned |
| 10 | Spec compliance/code review | 0.86 | Primary reviewer | Tests/build + independent reviewer | Unassigned |
| 11 | 100k+ token repository archaeology | 0.52 | Search-assisted only | Indexed repo tools | Unassigned |
| 12 | Multi-step shell/tool execution | 0.67 | Checkpointed orchestration | Tool logs/human approval | Unassigned |
| 13 | Strict schema/JSON extraction | 0.84 | Primary with parser validation | JSON Schema/validator | Unassigned |
| 14 | Unit/property/adversarial tests | 0.85 | Primary test author | Execute suite / coverage review | Unassigned |
| 15 | Technical docs/spec synthesis | 0.91 | Primary | Source cross-check | Unassigned |
| 16 | Persian RTL + English localization | 0.67 | Draft + browser/native review | Native-language and bidi testing | Unassigned |
| 17 | Telegram/webhook integration | 0.72 | Conditional; verify current docs | Integration tests / API docs | Unassigned |
| 18 | Opaque social ranking/growth | 0.43 | Fallback/experiment design | Current evidence and experiments | Unassigned |
| 19 | Low-latency routing/classification | 0.56 | Prototype only | Benchmark deterministic router | Unassigned |
| 20 | 50+ step unattended execution | 0.39 | Do not route unattended | Checkpointed agent + human approval | Unassigned |

## Routing rules and pipelines

- **Feature delivery:** architect (this profile may draft) → implementation owner → UI/accessibility review where relevant → tests/security audit. These are role slots, not claims about unavailable models.
- **Hard bug:** repo archaeology with indexed files → root-cause hypotheses → minimal patch → execute regression tests. Escalate concurrency and Rust memory-model issues to a specialist.
- **Prompt lookup:** use profile `07_PROMPT_COMPILER_LEXICON.md` and `08_FAILURE_MODES_AND_PATCHES.md`; compiler script adds externalized checklists, not hidden reasoning access.
- The KIA-BRAIN-OS Router can query `OMNI-MODEL-GENOME/models/<model_id>/` by `model_id`, loading `09_ROUTING_AND_HANDOFF_CARD.md` for role selection and `07` for compilation. Missing profile/fields must resolve to “unknown,” never invented metrics.

## Source inventory and limitations

The 19 supplied inputs are preserved under `uploads/`. They comprise nine numbered OMNI layer drafts and ten unnumbered legacy/companion drafts (including an index synthesis prompt). No KIA-BRAIN-OS router/bootloader source files were supplied, so the bridge is specified above but cannot be verified against a real `06_LINK_Router.md`. See the workspace audit report for exact paths and counts.

## Model #02 — `gemini-4-aragon-high` (provisional)

| Model ID | Display label | Reported reasoning mode | Reported context | Reported role | Evidence status |
|---|---|---|---|---|---|
| `gemini-4-aragon-high` | GEMINI 4 Aragon HIGH | Source claims hybrid/thinking | Source claims 131k sweet spot; unverified | Architecture, Rust, review, synthesis (source scorecard) | Artifact checks only; no live model/identity verification |

Profile: [`models/gemini-4-aragon-high/`](models/gemini-4-aragon-high/). Compiler: [`scripts/compile_for_gemini_aragon_high.py`](scripts/compile_for_gemini_aragon_high.py). Empirical artifact check report: [`EMPIRICAL_AUDIT_REPORT.json`](models/gemini-4-aragon-high/EMPIRICAL_AUDIT_REPORT.json).

## Dual-model 20-workload matrix

The GPT column reuses the earlier qualitative estimates; Gemini column transcribes the newly supplied 09 routing-card scores. Both columns are claims/estimates, not benchmark results. Choose based on tools, repository evidence, and independent verification—not score alone.

| # | Workload | GPT-6 profile estimate | Gemini Aragon profile estimate | Routing note |
|---:|---|---:|---:|---|
| 1 | System architecture / crate design | 0.87 | 0.99 | Either may draft; review repo fit |
| 2 | Complex Rust / async / CAS | 0.68 | 0.97 | Compile and test; independent Rust review |
| 3 | Multi-file refactoring | 0.73 | 0.96 | Requires repo-aware edits and tests |
| 4 | Interactive frontend UX | 0.70 | 0.96 | Browser and accessibility validation |
| 5 | 3D / WebGL / shaders | 0.55 | 0.96 | Runtime visual/performance check |
| 6 | SQLite / ledgers | 0.83 | 0.99 | Run transaction and recovery tests |
| 7 | Algorithms / formal deduction | 0.88 | 0.99 | Solver/proof verification if critical |
| 8 | Concurrency root cause | 0.70 | 0.98 | Needs traces, reproducer, race detector |
| 9 | Security / sandbox review | 0.82 | 0.98 | Qualified review and adversarial tests |
| 10 | Spec compliance / code review | 0.86 | 0.99 | A passing review is not a passing build |
| 11 | Large repository archaeology | 0.52 | 0.99 | Use indexed tools; these context claims are unverified |
| 12 | Agentic tool calling | 0.67 | 0.92 | Actual tools and permissions govern |
| 13 | Structured data extraction | 0.84 | 0.98 | Enforce parser/schema checks |
| 14 | Test-suite generation | 0.85 | 0.97 | Execute tests and inspect coverage |
| 15 | Technical genome synthesis | 0.91 | 1.00 | Source cross-check; scores not empirical |
| 16 | Persian RTL + English UX | 0.67 | 0.99 | Native and browser/device review |
| 17 | Telegram Mini-App integration | 0.72 | 0.98 | Verify current platform docs and live integration |
| 18 | Social ranking / growth | 0.43 | 0.97 | Opaque/current claims require evidence; avoid assumed internals |
| 19 | Low-latency routing | 0.56 | 0.65 | Benchmark actual latency; deterministic router may win |
| 20 | 50+ step autonomous orchestration | 0.39 | 0.99 | Do not infer reliability; checkpoint and human approval |

## Step-5 merge notes

`kian_v2_step5_merge/` provides a Rust profile adapter and SQLite injector template. No KIAN repository or `seed/kian-production.db` was present in this workspace, so the Rust struct mapping, DB schema compatibility, and actual injection remain **unverified/not executed**. The injector refuses to create a missing DB. The Gemini profile's 20 scores are encoded as supplied estimates; `think_mode_active` and `requires_external_state_trace` reflect profile claims, not independently tested facts.
