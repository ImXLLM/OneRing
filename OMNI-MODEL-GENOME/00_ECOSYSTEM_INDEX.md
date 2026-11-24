# OMNI-MODEL-GENOME — Ecosystem Index

**Audit date:** 2026-11-24. **Evidence status:** one model profile is present. GPT-6 / Free Web / Standard is a user-specified target label; no authenticated model session or benchmark evidence was supplied. Its scorecard is a qualitative routing heuristic, not measured telemetry. Do not treat unknown models as verified fleet members.

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
