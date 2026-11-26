-- Migration 0012 — the competitive supremacy engine (Wave 8 — Commit #28)
--
-- three additive tables for the four supremacy pillars:
-- - `AgentCheckpoint`: immutable agent-state checkpoints (LangGraph-style
--   time-travel; branch from any historical step without re-running earlier steps);
-- - `WorkflowDagRun`: dynamic DAG runs with Kahn topological ordering (Dify/n8n);
-- - `SweSandboxAudit`: the `Plan → Verify → Self-Heal` sandbox audit sealed
--   by Mairon (OpenHands/Replit Agent).
--
-- no double-entry ledger table is touched — the `Posting` zero-sum stays untouched.

CREATE TABLE IF NOT EXISTS "AgentCheckpoint" (
    "id" TEXT PRIMARY KEY,
    "parent_id" TEXT,
    "workflow_id" TEXT NOT NULL,
    "step_index" INTEGER NOT NULL,
    "agent_role" TEXT NOT NULL,
    "state_json" TEXT NOT NULL,
    "state_sha256" TEXT NOT NULL,
    "created_at" TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS "idx_agent_checkpoint_workflow"
    ON "AgentCheckpoint" ("workflow_id", "step_index");

CREATE TABLE IF NOT EXISTS "WorkflowDagRun" (
    "id" TEXT PRIMARY KEY,
    "dag_id" TEXT NOT NULL,
    "trigger_kind" TEXT NOT NULL,
    "topo_order_json" TEXT NOT NULL,
    "status" TEXT NOT NULL,
    "rea_score" INTEGER NOT NULL,
    "created_at" TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS "idx_workflow_dag_run_dag"
    ON "WorkflowDagRun" ("dag_id");

CREATE TABLE IF NOT EXISTS "SweSandboxAudit" (
    "id" TEXT PRIMARY KEY,
    "artifact_sha256" TEXT NOT NULL,
    "language" TEXT NOT NULL,
    "attempts_used" INTEGER NOT NULL,
    "passed" INTEGER NOT NULL,
    "diagnostics_json" TEXT NOT NULL,
    "created_at" TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS "idx_swe_sandbox_audit_created"
    ON "SweSandboxAudit" ("created_at" DESC);
