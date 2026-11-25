-- مهاجرت 0012 — موتورِ برتریِ رقابتی (Wave 8 — Commit #28)
--
-- سه جدولِ additive برای چهار ستونِ برتری:
-- - `AgentCheckpoint`: چک‌پوینت‌هایِ تغییرناپذیرِ وضعیتِ ایجنت (LangGraph-style
--   time-travel؛ انشعاب از هر گامِ تاریخی بدونِ اجرایِ مجددِ گام‌هایِ پیشین)؛
-- - `WorkflowDagRun`: ران‌هایِ DAG پویا با ترتیبِ توپولوژیکِ Kahn (Dify/n8n)؛
-- - `SweSandboxAudit`: ممیزیِ سندباکسِ `Plan → Verify → Self-Heal` با مُهرِ
--   مایرون (OpenHands/Replit Agent).
--
-- هیچ جدولِ دفتریِ دوطرفه‌ای لمس نمی‌شود — صفرجمعِ `Posting` دست‌نخورده می‌ماند.

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
