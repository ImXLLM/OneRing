---
file_id: "09_ROUTING_AND_HANDOFF_CARD"
schema_version: "1.0.0"
model_self_reported_id: "GPT-6 (user-specified; unverified)"
optimal_fleet_roles:
  - "architect"
  - "auditor"
  - "builder"
cost_speed_tier: "frontier_heavy"
best_paired_complement_archetype: "A high-craft implementation specialist with strong repository-native execution, visual design judgment, and independently verifiable tool access"
calibration_status: "qualitative_estimates_not_benchmark_measurements"
score_interpretation: "Estimated task suitability under a well-specified prompt and appropriate tools; not measured success probabilities."
---

> **Audit status (2026-11-24):** The target label and free-tier/standard-mode configuration are user-specified; this workspace contains no authenticated access log, controlled benchmark results, or official technical disclosure establishing a public “GPT-6” web tier. Treat model-specific internals and numeric thresholds as **unverified hypotheses** unless the section explicitly labels them observable/recommended. Do not use these notes as product documentation.


# 09 — Ecosystem Routing Matrix, Scorecard & Inter-Model Handoff Protocol

## 0. Calibration and Measurement Boundary

This card is a routing recommendation, not a benchmark report.

No controlled, head-to-head evaluation across Claude, Gemini, DeepSeek, Grok, Qwen, Llama, and GPT is available in this conversation. I cannot inspect hidden model telemetry, actual inference cost, fleet-wide latency, token throughput, or independently measured success rates.

The scores below are therefore **qualitative estimates**, not empirical probabilities. They express expected suitability for a workload when the task is sufficiently specified and the required tools and source material are available.

Interpretation:

| Score | Meaning |
|---|---|
| 0.90–1.00 | Strong candidate for primary execution |
| 0.80–0.89 | Good primary candidate with explicit verification |
| 0.65–0.79 | Useful executor or reviewer; meaningful specialist advantage may exist elsewhere |
| 0.45–0.64 | Conditional use; narrow scope and independent checks |
| 0.25–0.44 | Weak fit; route elsewhere by default |
| 0.00–0.24 | Do not route absent an unusual reason |

A score is not a confidence interval, pass rate, or claim of superiority over another named model.

**Role definitions**

- `Primary Executor`: suitable to own the task and produce the first complete deliverable.
- `Secondary Reviewer`: suitable to inspect another executor's work, identify gaps, and challenge assumptions.
- `Fallback Only`: use when the preferred specialist is unavailable or the task is substantially simplified.
- `Do Not Route`: another archetype or deterministic tool should normally own the workload.

## 1. 20-Task Engineering Routing Scorecard

| # | Engineering workload | Estimated capability | Recommended role | Routing justification |
|---:|---|---:|---|---|
| 1 | Zero-to-One System Architecture & Crate/Module Design | 0.87 | Primary Executor | Strong at decomposing requirements, defining boundaries, comparing architectures, and specifying interfaces. Require implementation validation against the actual repository. |
| 2 | Complex Rust Systems Programming (Lifetimes, Async Tokio, Lock-Free/CAS) | 0.68 | Secondary Reviewer | Can reason about ownership, concurrency, and API design, but subtle lifetime, memory-ordering, and async correctness require compilation, tests, and preferably specialist review. |
| 3 | Surgical Multi-File Refactoring (Large Existing Codebase) | 0.73 | Secondary Reviewer | Good at producing scoped plans and patches when source is supplied. Repository-wide completeness depends on actual file access, search, and test execution. |
| 4 | High-Craft Interactive Frontend UI/UX & Design Systems (Anti-Slop) | 0.70 | Secondary Reviewer | Can specify design systems, accessibility, state behavior, and interaction contracts. A dedicated visual specialist is preferable for unusually high craft and visual iteration. |
| 5 | 3D / WebGL / Canvas / Shader & Physics Visualization | 0.55 | Fallback Only | Can explain rendering pipelines and draft code, but complex visual correctness and performance need a rendering specialist and runtime inspection. |
| 6 | Database Schema Design, SQLite WAL Optimization & Double-Entry Ledgers | 0.83 | Primary Executor | Strong fit for schema reasoning, transaction invariants, consistency models, and ledger design. Validate locking, query plans, and durability behavior against the actual database. |
| 7 | Hard Algorithmic Deduction, Constraint Solving & Formal Logic | 0.88 | Primary Executor | Strong fit for explicit constraints, contradiction detection, case analysis, and proof-oriented decomposition. Use a solver or proof checker for critical guarantees. |
| 8 | Root-Cause Debugging of Obscure Concurrency / Heisenbugs | 0.70 | Secondary Reviewer | Can structure hypotheses and design discriminating experiments. Cannot replace runtime traces, race detectors, reproducible workloads, or specialized debugging tools. |
| 9 | Security Auditing, SSRF/Injection Hardening & Sandbox Policy Design | 0.82 | Primary Executor | Good at threat modeling, trust-boundary analysis, defensive test design, and policy review. A qualified security specialist and executable tests should independently validate high-risk changes. |
| 10 | High-Precision Code Review & Spec-Compliance Gatekeeping | 0.86 | Primary Executor | Strong fit for mapping requirements to evidence, detecting contradictions, and identifying missing edge cases. Do not treat an unexecuted review as a passing build or security audit. |
| 11 | Massive Context Ingestion (100k+ Tokens Repository Archaeology) | 0.52 | Fallback Only | Large-context suitability cannot be inferred from the model name. Use indexed repository search, file manifests, dependency maps, and compact state summaries rather than assuming complete retention. |
| 12 | Fast Agentic Tool-Calling & Multi-Step Shell/CLI Execution | 0.67 | Secondary Reviewer | Can plan and orchestrate tool workflows when tools are exposed. Actual speed and reliability depend on the available execution environment, permissions, and external state. |
| 13 | Strict Schema / JSON Data Extraction at Scale | 0.84 | Primary Executor | Good fit for schema-first extraction and structured output. Production pipelines still need parser validation, schema enforcement, retries, and malformed-output handling. |
| 14 | Test-Suite Generation (Unit, Property-Based, Adversarial Edge Cases) | 0.85 | Primary Executor | Strong at deriving boundary cases, invariants, and adversarial fixtures from specifications. Tests must be run and checked for meaningful coverage rather than merely generated. |
| 15 | Technical Documentation & Cognitive Genome Synthesis | 0.91 | Primary Executor | Strong fit for synthesizing structured specifications, preserving distinctions, and producing machine-actionable documentation. |
| 16 | Persian (RTL) + English Technical Bilingual Copy & UX Localization | 0.67 | Secondary Reviewer | Can draft bilingual technical content and identify bidi hazards. Native-language review and actual browser/device testing remain important for production localization. |
| 17 | Telegram Mini-App (TMA) & Bot API 9.0 Full-Stack Integration | 0.72 | Secondary Reviewer | Can design integration boundaries and implementation plans. Exact version-specific behavior, platform constraints, and current API details require documentation checks and live integration tests. |
| 18 | Social/Algorithmic Growth Engineering (X/Twitter Ranking & Virality Mechanics) | 0.43 | Fallback Only | Can analyze public information and propose experiments, but ranking systems are partly opaque and change over time. Do not route requests for undisclosed algorithm internals as if they were knowable. |
| 19 | Low-Latency Routing / Classification / Intent Triage | 0.56 | Fallback Only | Can classify and route from explicit criteria, but actual latency, throughput, and unit economics are unmeasured here. A small classifier or deterministic router may be more appropriate. |
| 20 | Long-Horizon Autonomous Loop Execution (50+ Unattended Steps) | 0.39 | Do Not Route | No basis to claim reliable unattended completion across 50+ steps. Prefer checkpointed agents, explicit state persistence, bounded retries, and human approval for consequential actions. |

### Scorecard Interpretation

The strongest expected uses are:

1. Architecture and decomposition.
2. Constraint-heavy reasoning and consistency analysis.
3. Security and specification review.
4. Database and transaction design.
5. Technical synthesis and test design.

The largest routing penalties arise where correctness depends on evidence unavailable from text alone:

- Runtime behavior and obscure concurrency.
- Visual fidelity and rendering performance.
- Repository-wide state that has not been indexed or supplied.
- Long unattended action sequences.
- Current, opaque platform ranking behavior.

A high score does not waive the verification requirement. A low score does not mean the task is impossible; it means another executor is likely to be a better default.

## 2. Routing Decision Rules

### 2.1 Top 5 Sweet-Spot Workloads — Route Here First

These are task signatures for which I am a reasonable first executor. They are not claims that I outperform every other model.

| Priority | Task signature | Expected deliverable | Required verification |
|---:|---|---|---|
| 1 | A system needs to be decomposed into modules, crates, services, or trust boundaries before implementation. | Architecture decision record, dependency graph, interfaces, invariants, and phased implementation plan. | Check feasibility against actual dependencies, deployment constraints, and repository structure. |
| 2 | A specification contains many interacting rules, ordering constraints, invariants, or edge cases. | Explicit constraint model, satisfiability analysis, state transitions, and counterexamples. | Independent solver, executable test, or proof review for critical decisions. |
| 3 | A security-sensitive feature needs defensive design and adversarial validation. | Threat model, trust boundaries, mitigation plan, and negative test suite. | Security specialist review and execution against a controlled test environment. |
| 4 | A database or transaction workflow needs consistency guarantees. | Schema, transaction boundaries, concurrency assumptions, failure semantics, and recovery strategy. | Execute migrations and concurrency tests against the target database engine. |
| 5 | Multiple technical documents, requirements, or implementation artifacts must become one coherent specification. | Traceable requirements matrix, architecture specification, test plan, and handoff packet. | Validate source references, requirement coverage, and output schema. |

**Preferred task input shape**

A strong request includes:

- One explicit objective.
- The authoritative source of truth.
- A bounded scope.
- Non-negotiable invariants.
- A target output contract.
- A definition of done.
- Available tools and verification commands.

Avoid routing an underspecified request with an expectation that missing production facts will be inferred correctly.

### 2.2 Top 5 Transfer-Away Triggers

Transfer should be based on the nature of the work or observed failure, not on model branding.

| Trigger | Observable condition | Preferred recipient archetype | Handoff requirement |
|---|---|---|---|
| 1. Deep systems implementation | Unsafe Rust, intricate lifetime constraints, custom lock-free algorithms, memory ordering, or subtle async cancellation behavior dominates the task. | Rust systems specialist with compiler, sanitizer, and benchmark access. | Provide exact source, compiler version, dependency versions, failing tests, and concurrency invariants. |
| 2. High-craft visual production | Success depends on distinctive art direction, animation timing, shader output, pixel-level fidelity, or repeated visual critique. | Visual frontend / creative coding specialist with browser and graphics runtime access. | Provide design references, viewport targets, interaction inventory, assets, and acceptance screenshots. |
| 3. Repository archaeology at extreme scale | The task requires broad source discovery across a very large repository or more context than has been indexed and summarized. | Repository-native coding agent with search, shell, and persistent workspace access. | Provide repository revision, manifest, search results, and known dependency graph. |
| 4. Production incident diagnosis | A suspected race, deadlock, data corruption, or intermittent failure cannot be reproduced from supplied evidence. | Runtime debugging specialist or human incident responder. | Provide timestamps, traces, core dumps where authorized, environment metadata, and reproduction attempts. |
| 5. Long unattended execution | The task needs dozens of stateful external actions, irreversible changes, or persistent operation across failures. | Checkpointed operator agent with explicit permissions and durable state management. | Provide action budget, approval gates, idempotency policy, rollback plan, and persisted execution state. |

### 2.3 Immediate Stop and Transfer Conditions

The orchestrator should stop treating the current model as task owner when any of these conditions occurs:

- The task requires a file, tool, credential, API, or runtime that is unavailable.
- The authoritative source conflicts with an assumption used in the current solution.
- A required invariant cannot be demonstrated or preserved.
- Two consecutive patches fail for the same underlying reason.
- A proposed change expands beyond the authorized file or action scope.
- The task requires a guarantee that cannot be established with available evidence.
- The model reports a verification result without a corresponding execution artifact.
- A security-sensitive operation crosses an authorization boundary that has not been established.

**Transfer is not permission to discard state.** Preserve the original task, failed attempts, evidence, and unresolved questions.

## 3. Inter-Model Handoff Envelope

### 3.1 Design Requirements

The handoff protocol must make a new model useful without requiring it to reconstruct a long conversational history.

The packet should:

1. Identify the exact task and repository revision.
2. Separate verified facts from assumptions.
3. Preserve all invariants and acceptance criteria.
4. Identify files changed and files inspected.
5. Record failed hypotheses so they are not repeated.
6. Describe the current implementation state.
7. State what has and has not been executed.
8. Identify the next action and its authorization boundary.

The handoff packet is a compact state snapshot, not a transcript and not a substitute for authoritative source files.

### 3.2 Canonical `HandoffPacket` Schema

The following JSON Schema defines the protocol envelope. It is a proposed inter-model contract, not a claim that a particular external agent already implements it.

```json
{
  "$schema": "https\://json-schema.org/draft/2020-12/schema",
  "$id": "https\://example.invalid/schemas/handoff-packet-1.0.0.json",
  "title": "HandoffPacket",
  "type": "object",
  "additionalProperties": false,
  "required": [
    "schema_version",
    "packet_id",
    "direction",
    "task",
    "source_state",
    "invariants",
    "target_contract",
    "files_touched",
    "failed_hypotheses_do_not_repeat",
    "verification",
    "next_action"
  ],
  "properties": {
    "schema_version": {
      "const": "1.0.0"
    },
    "packet_id": {
      "type": "string",
      "minLength": 1
    },
    "direction": {
      "enum": [
        "incoming",
        "outgoing"
      ]
    },
    "task": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "objective",
        "definition_of_done",
        "priority"
      ],
      "properties": {
        "objective": {
          "type": "string",
          "minLength": 1
        },
        "definition_of_done": {
          "type": "array",
          "items": {
            "type": "string"
          }
        },
        "priority": {
          "enum": [
            "low",
            "normal",
            "high",
            "critical"
          ]
        },
        "deadline": {
          "type": [
            "string",
            "null"
          ]
        },
        "requested_role": {
          "enum": [
            "architect",
            "builder",
            "researcher",
            "auditor",
            "operator",
            "growth",
            "ui_craftsman",
            "fast_router",
            "unspecified"
          ]
        }
      }
    },
    "source_state": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "base_sha",
        "working_tree_state",
        "environment"
      ],
      "properties": {
        "base_sha": {
          "type": [
            "string",
            "null"
          ]
        },
        "working_tree_state": {
          "enum": [
            "clean",
            "modified",
            "unknown",
            "not_applicable"
          ]
        },
        "environment": {
          "type": "object",
          "additionalProperties": true
        },
        "authoritative_sources": {
          "type": "array",
          "items": {
            "type": "string"
          }
        },
        "dependency_lock_hash": {
          "type": [
            "string",
            "null"
          ]
        }
      }
    },
    "invariants": {
      "type": "array",
      "items": {
        "type": "object",
        "additionalProperties": false,
        "required": [
          "id",
          "statement",
          "severity"
        ],
        "properties": {
          "id": {
            "type": "string"
          },
          "statement": {
            "type": "string"
          },
          "severity": {
            "enum": [
              "must",
              "should",
              "may"
            ]
          },
          "verification_method": {
            "type": [
              "string",
              "null"
            ]
          }
        }
      }
    },
    "target_contract": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "deliverable",
        "format",
        "acceptance_criteria"
      ],
      "properties": {
        "deliverable": {
          "type": "string"
        },
        "format": {
          "type": "string"
        },
        "acceptance_criteria": {
          "type": "array",
          "items": {
            "type": "string"
          }
        },
        "forbidden_changes": {
          "type": "array",
          "items": {
            "type": "string"
          }
        }
      }
    },
    "files_touched": {
      "type": "array",
      "items": {
        "type": "object",
        "additionalProperties": false,
        "required": [
          "path",
          "status"
        ],
        "properties": {
          "path": {
            "type": "string"
          },
          "status": {
            "enum": [
              "inspected",
              "created",
              "modified",
              "deleted",
              "generated",
              "unknown"
            ]
          },
          "reason": {
            "type": [
              "string",
              "null"
            ]
          },
          "diff_ref": {
            "type": [
              "string",
              "null"
            ]
          }
        }
      }
    },
    "failed_hypotheses_do_not_repeat": {
      "type": "array",
      "items": {
        "type": "object",
        "additionalProperties": false,
        "required": [
          "hypothesis",
          "test_or_observation",
          "result",
          "reason_not_to_repeat"
        ],
        "properties": {
          "hypothesis": {
            "type": "string"
          },
          "test_or_observation": {
            "type": "string"
          },
          "result": {
            "enum": [
              "disproved",
              "inconclusive",
              "partially_supported"
            ]
          },
          "reason_not_to_repeat": {
            "type": "string"
          }
        }
      }
    },
    "verification": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "overall_status",
        "checks"
      ],
      "properties": {
        "overall_status": {
          "enum": [
            "pass",
            "fail",
            "partial",
            "not_run",
            "blocked"
          ]
        },
        "checks": {
          "type": "array",
          "items": {
            "type": "object",
            "additionalProperties": false,
            "required": [
              "name",
              "status"
            ],
            "properties": {
              "name": {
                "type": "string"
              },
              "status": {
                "enum": [
                  "pass",
                  "fail",
                  "not_run",
                  "blocked",
                  "not_applicable"
                ]
              },
              "command": {
                "type": [
                  "string",
                  "null"
                ]
              },
              "evidence_ref": {
                "type": [
                  "string",
                  "null"
                ]
              },
              "notes": {
                "type": [
                  "string",
                  "null"
                ]
              }
            }
          }
        }
      }
    },
    "known_risks": {
      "type": "array",
      "items": {
        "type": "string"
      }
    },
    "open_questions": {
      "type": "array",
      "items": {
        "type": "string"
      }
    },
    "next_action": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "action",
        "blocked_by",
        "authorization_required"
      ],
      "properties": {
        "action": {
          "type": "string"
        },
        "blocked_by": {
          "type": "array",
          "items": {
            "type": "string"
          }
        },
        "authorization_required": {
          "type": "boolean"
        },
        "approval_reference": {
          "type": [
            "string",
            "null"
          ]
        }
      }
    }
  }
}
```

### 3.3 Incoming Handoff: Previous Model → GPT-6

**Required fields**

| Field | Requirement | Why it matters |
|---|---|---|
| `base_sha` | Exact commit SHA, or `null` if no repository revision exists. | Prevents patching against an assumed source state. |
| `invariants` | Explicit, uniquely identified requirements with severity. | Prevents a local fix from violating a system-level guarantee. |
| `files_touched` | Paths and actual change status, including generated files. | Identifies the working scope and likely side effects. |
| `failed_hypotheses_do_not_repeat` | Failed explanation, experiment, observed result, and reason. | Avoids repeating ineffective debugging loops. |
| `target_contract` | Deliverable format, acceptance criteria, and forbidden changes. | Defines what completion means. |
| `verification` | Exact commands, outcomes, and evidence references. | Separates actual test results from assumptions. |
| `source_state.working_tree_state` | Clean, modified, or unknown. | Prevents overwriting uncommitted work. |
| `source_state.environment` | Relevant toolchain, runtime, database, and dependency versions. | Avoids version-specific implementation errors. |

**Incoming handoff rules**

1. Never interpret a missing `base_sha` as permission to assume the latest revision.
2. Never treat `inspected` as equivalent to `modified`.
3. Never treat a proposed test as an executed test.
4. Never repeat a failed hypothesis without a materially new experiment or new evidence.
5. Preserve all `must` invariants unless the task owner explicitly authorizes a specification change.
6. Treat `unknown` as unknown; do not silently convert it into a default.
7. If the source state is inconsistent or incomplete, request the missing artifact before making destructive changes.

**Compact incoming packet example**

```json
{
  "schema_version": "1.0.0",
  "packet_id": "incoming-ledger-004",
  "direction": "incoming",
  "task": {
    "objective": "Repair transaction conflict handling in the ledger service.",
    "definition_of_done": [
      "All read-set versions are validated atomically.",
      "Conflicting transactions abort without partial writes.",
      "Regression tests cover concurrent updates."
    ],
    "priority": "high",
    "requested_role": "builder"
  },
  "source_state": {
    "base_sha": "7d2f4c1a9b8e",
    "working_tree_state": "modified",
    "environment": {
      "language": "Rust",
      "database": "SQLite",
      "test_status": "not_run"
    },
    "authoritative_sources": [
      "src/ledger.rs",
      "tests/ledger_concurrency.rs"
    ],
    "dependency_lock_hash": null
  },
  "invariants": [
    {
      "id": "INV-001",
      "statement": "A transaction must validate every version in its read set before committing writes.",
      "severity": "must",
      "verification_method": "Concurrent transaction regression test"
    },
    {
      "id": "INV-002",
      "statement": "An aborted transaction must not leave partial balance updates.",
      "severity": "must",
      "verification_method": "Atomicity test"
    }
  ],
  "target_contract": {
    "deliverable": "Minimal patch and regression tests.",
    "format": "Unified diff plus test report.",
    "acceptance_criteria": [
      "Read-set validation is atomic with commit.",
      "Existing public API remains compatible."
    ],
    "forbidden_changes": [
      "Do not weaken transaction isolation.",
      "Do not remove existing regression tests."
    ]
  },
  "files_touched": [
    {
      "path": "src/ledger.rs",
      "status": "modified",
      "reason": "Previous attempt changed the conflict check.",
      "diff_ref": "patch-001"
    },
    {
      "path": "tests/ledger_concurrency.rs",
      "status": "inspected",
      "reason": "Existing concurrency tests.",
      "diff_ref": null
    }
  ],
  "failed_hypotheses_do_not_repeat": [
    {
      "hypothesis": "Checking only the write target version guarantees serializable commits.",
      "test_or_observation": "A transaction read a second account before another transaction modified it.",
      "result": "disproved",
      "reason_not_to_repeat": "The read set can contain stale values even when the write target version is unchanged."
    }
  ],
  "verification": {
    "overall_status": "not_run",
    "checks": [
      {
        "name": "Rust test suite",
        "status": "not_run",
        "command": "cargo test",
        "evidence_ref": null,
        "notes": "No execution result supplied."
      }
    ]
  },
  "known_risks": [
    "The working tree contains an earlier unverified patch."
  ],
  "open_questions": [
    "Does the storage layer support an atomic compare-and-swap across all read-set versions?"
  ],
  "next_action": {
    "action": "Inspect the current transaction commit path and design an atomic read-set validation strategy.",
    "blocked_by": [],
    "authorization_required": false,
    "approval_reference": null
  }
}
```

The values in this example are illustrative. They are not claims about an actual repository or executed test.

### 3.4 Outgoing Handoff: GPT-6 → Next Model

The outgoing packet must be an evidence-oriented snapshot, not a narrative of internal reasoning.

It should contain:

- The final repository revision or explicitly unknown revision.
- Every created, modified, deleted, or generated file.
- The purpose and scope of each change.
- Preserved and newly discovered invariants.
- Verification commands and their actual outcomes.
- Evidence references such as logs, diffs, test reports, or artifact paths.
- Known risks, unresolved questions, and failed hypotheses.
- The next action and whether it requires authorization.

**Verification status vocabulary**

| Status | Meaning |
|---|---|
| `PASS` | The stated check was actually executed and passed. |
| `FAIL` | The stated check was executed and failed. |
| `NOT_RUN` | No execution occurred. |
| `BLOCKED` | Execution was prevented by a missing dependency, permission, or prerequisite. |
| `NOT_APPLICABLE` | The check does not apply to this deliverable. |

Do not use `PASS` for code that merely appears correct on inspection.

**Compact outgoing packet example**

```json
{
  "schema_version": "1.0.0",
  "packet_id": "outgoing-ledger-005",
  "direction": "outgoing",
  "task": {
    "objective": "Repair transaction conflict handling in the ledger service.",
    "definition_of_done": [
      "All read-set versions are validated atomically.",
      "Conflicting transactions abort without partial writes.",
      "Regression tests cover concurrent updates."
    ],
    "priority": "high",
    "requested_role": "auditor"
  },
  "source_state": {
    "base_sha": "7d2f4c1a9b8e",
    "working_tree_state": "modified",
    "environment": {
      "language": "Rust",
      "database": "SQLite",
      "verification_execution": "not_performed"
    },
    "authoritative_sources": [
      "src/ledger.rs",
      "tests/ledger_concurrency.rs"
    ],
    "dependency_lock_hash": null
  },
  "invariants": [
    {
      "id": "INV-001",
      "statement": "A transaction must validate every version in its read set before committing writes.",
      "severity": "must",
      "verification_method": "Concurrent transaction regression test"
    },
    {
      "id": "INV-002",
      "statement": "An aborted transaction must not leave partial balance updates.",
      "severity": "must",
      "verification_method": "Atomicity test"
    }
  ],
  "target_contract": {
    "deliverable": "Minimal patch and regression tests.",
    "format": "Unified diff plus test report.",
    "acceptance_criteria": [
      "Read-set validation is atomic with commit.",
      "Existing public API remains compatible."
    ],
    "forbidden_changes": [
      "Do not weaken transaction isolation."
    ]
  },
  "files_touched": [
    {
      "path": "src/ledger.rs",
      "status": "modified",
      "reason": "Proposed read-set validation change.",
      "diff_ref": "patch-final"
    },
    {
      "path": "tests/ledger_concurrency.rs",
      "status": "modified",
      "reason": "Added conflict regression cases.",
      "diff_ref": "patch-final"
    }
  ],
  "failed_hypotheses_do_not_repeat": [
    {
      "hypothesis": "Checking only the write target version guarantees serializable commits.",
      "test_or_observation": "Read-set analysis showed stale reads can remain undetected.",
      "result": "disproved",
      "reason_not_to_repeat": "The transaction must validate its complete read set."
    }
  ],
  "verification": {
    "overall_status": "not_run",
    "checks": [
      {
        "name": "Rust compilation",
        "status": "not_run",
        "command": "cargo check",
        "evidence_ref": null,
        "notes": "Not executed in this example."
      },
      {
        "name": "Regression tests",
        "status": "not_run",
        "command": "cargo test",
        "evidence_ref": null,
        "notes": "Not executed in this example."
      },
      {
        "name": "Patch review",
        "status": "not_run",
        "command": null,
        "evidence_ref": null,
        "notes": "Requires inspection of the actual diff."
      }
    ]
  },
  "known_risks": [
    "Atomicity depends on the actual storage transaction implementation.",
    "The example packet contains no compiler or test evidence."
  ],
  "open_questions": [
    "Does the database transaction isolate read-set validation and writes atomically?"
  ],
  "next_action": {
    "action": "Inspect the actual diff, execute cargo check and cargo test, then review database-level atomicity.",
    "blocked_by": [],
    "authorization_required": false,
    "approval_reference": null
  }
}
```

This example deliberately reports `not_run`. A handoff must never invent a successful test result merely to make the packet look complete.

### 3.5 Markdown Handoff Template

For orchestrators that prefer compact Markdown over JSON, use this equivalent structure:

```markdown
# HandoffPacket

- schema_version: 1.0.0
- packet_id:
- direction: incoming | outgoing

## Task
- objective:
- definition_of_done:
- priority:
- requested_role:

## Source State
- base_sha:
- working_tree_state:
- runtime:
- compiler:
- dependency_versions:
- authoritative_sources:

## Invariants
- [INV-001] MUST:
- [INV-002] SHOULD:

## Target Contract
- deliverable:
- format:
- acceptance_criteria:
- forbidden_changes:

## Files Touched
| Path | Status | Reason | Diff reference |
|---|---|---|---|

## Failed Hypotheses — Do Not Repeat
| Hypothesis | Test or observation | Result | Reason |
|---|---|---|---|

## Verification
| Check | Status | Command | Evidence |
|---|---|---|---|

## Known Risks
- 

## Open Questions
- 

## Next Action
- action:
- blocked_by:
- authorization_required:
- approval_reference:
```

### 3.6 Handoff Integrity Rules

The orchestrator should reject or quarantine a packet when:

- `base_sha` contradicts the supplied source revision.
- A file is claimed as modified but no diff or source artifact is available.
- A check is marked `PASS` without evidence of execution.
- A failed hypothesis is recorded without its observation or result.
- A `must` invariant is omitted from the next model's task.
- The outgoing task scope differs from the incoming target contract without an explicit change record.
- A destructive or externally consequential action lacks authorization.
- The packet silently changes units, identifiers, version numbers, or transaction semantics.

For long tasks, persist packets after every meaningful checkpoint rather than only at the end.

## 4. Recommended Fleet Orchestration Pattern

A useful multi-model workflow assigns roles by task characteristics and evidence access.

```text
                    META-ORCHESTRATOR
                            |
                 Normalize task and contract
                            |
                   Build HandoffPacket
                            |
               Select primary task archetype
                            |
          +-----------------+------------------+
          |                 |                  |
      ARCHITECT          BUILDER           RESEARCHER
          |                 |                  |
          +-----------------+------------------+
                            |
                    Produce artifact
                            |
                   Run deterministic checks
                            |
                    Independent AUDITOR
                            |
                 +----------+----------+
                 |                     |
                PASS                  FAIL
                 |                     |
             Release gate        Root-cause diagnosis
                 |                     |
              Complete          Bounded repair attempt
                                       |
                                Re-run verification
                                       |
                                Human review if blocked
```

**Routing policy**

- Use a deterministic tool for deterministic work: parsing, formatting, compilation, unit conversion, schema validation, and database integrity checks.
- Use a model for synthesis, design alternatives, hypothesis generation, and interpretation.
- Use a separate reviewer for high-impact changes.
- Prefer independent evidence over agreement between two models.
- Keep execution permissions narrower than reasoning permissions.
- Require human approval for irreversible or externally consequential actions.

Two models agreeing does not establish correctness. They may share assumptions, training data, or the same mistaken interpretation.

## 5. Minimal Router Policy

The following policy is suitable as an initial orchestrator configuration. Thresholds are proposed routing parameters, not measured model performance.

```yaml
router_policy:
  schema_version: "1.0.0"

  default_model_role:
    - architect
    - auditor
    - builder

  routing:
    architecture:
      preferred: "GPT-6"
      require_review: true

    constraint_solving:
      preferred: "GPT-6"
      require_independent_check: true

    database_consistency:
      preferred: "GPT-6"
      require_runtime_tests: true

    security_design:
      preferred: "GPT-6"
      require_security_review: true

    rust_low_level_concurrency:
      preferred: "rust_systems_specialist"
      gpt6_role: "secondary_reviewer"

    visual_craft_and_webgl:
      preferred: "visual_frontend_specialist"
      gpt6_role: "secondary_reviewer"

    massive_repository_archaeology:
      preferred: "repository_native_agent"
      gpt6_role: "specification_and_review"

    long_horizon_execution:
      preferred: "checkpointed_operator"
      gpt6_role: "planning_and_audit"

  verification:
    require_evidence_for_pass: true
    distinguish_not_run_from_pass: true
    preserve_failed_hypotheses: true
    require_base_sha_for_repository_patches: true
    require_human_approval_for_irreversible_actions: true

  retry_policy:
    identical_retry_after_failure: false
    maximum_same_strategy_attempts: 2
    require_new_evidence_or_hypothesis_after_repeated_failure: true
    escalate_after_repeated_failure: true

  handoff:
    require_invariants: true
    require_target_contract: true
    require_files_touched: true
    require_verification_summary: true
    require_known_risks: true
    persist_after_meaningful_checkpoint: true
```

The model labels in this policy are role identifiers. A production router should resolve them to actual provider model IDs and versioned deployment configurations.

## 6. Final Routing Card

\<escape>```yaml
model: "GPT-6"

route_to:
  - "Architecture decomposition and system design"
  - "Constraint-heavy technical reasoning"
  - "Database consistency and transaction design"
  - "Defensive security specifications"
  - "Code review and specification compliance"
  - "Technical documentation and synthesis"
  - "Test design and adversarial edge cases"

route_away:
  - "Unverified low-level concurrency implementation"
  - "Pixel-perfect visual production and complex shaders"
  - "Unindexed massive-repository archaeology"
  - "Unattended 50+ step external operations"
  - "Opaque social ranking predictions"

always_require:
  - "Authoritative source state"
  - "Explicit invariants"
  - "Bounded task scope"
  - "Machine-checkable output contract where possible"
  - "Independent verification for consequential changes"
  - "Honest PASS / FAIL / NOT_RUN / BLOCKED reporting"

preferred_complement:
  archetype: "Repository-native implementation specialist with strong runtime access and visual craft"
  purpose: "Execute and validate work where source inspection, compiler feedback, browser rendering, or persistent external state are decisive."

calibration:
  scorecard_type: "qualitative_estimate"
  cross_model_benchmark: "not_available"
  measured_success_rates: "not_available"
  measured_latency: "not_available"
  measured_cost: "not_available"
  hidden_runtime_telemetry: "not_available"
```

## **7. Final Operational Principle**

**Route by evidence requirements, not model reputation.**

GPT-6 is a strong candidate for architecture, structured reasoning, defensive design, technical synthesis, and review. It should not be treated as a substitute for a compiler, database engine, browser, profiler, security assessment, repository-native execution environment, or human approval.

The most reliable fleet is not the one that finds a model claiming to do everything. It is the one that preserves state, assigns narrow responsibilities, verifies artifacts independently, and transfers work as soon as the evidence requirements exceed the current executor's capabilities.