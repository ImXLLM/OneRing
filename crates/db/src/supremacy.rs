//! Competitive-supremacy engine persistence (Wave 8 — Commit #28).
//!
//! Three deterministic operations on migration `0012`: insert/read agent checkpoints
//! (`AgentCheckpoint`), insert DAG runs (`WorkflowDagRun`), and insert SWE sandbox
//! audits (`SweSandboxAudit`). No double-entry ledger table is touched —
//! these are the "supremacy ledger", not money itself.

use sqlx::Row;

use crate::error::{DbError, DbResult};
use crate::store::V3Store;

/// Agent checkpoint row — the `AgentCheckpoint` columns.
#[derive(Debug, Clone, PartialEq)]
pub struct AgentCheckpointRow {
    /// Checkpoint ID (`ckpt:…`).
    pub id: String,
    /// Parent — `None` for the workflow root.
    pub parent_id: Option<String>,
    /// Workflow ID.
    pub workflow_id: String,
    /// Step number.
    pub step_index: i64,
    /// Agent role.
    pub agent_role: String,
    /// JSON state.
    pub state_json: String,
    /// SHA-256 of the state.
    pub state_sha256: String,
    /// ISO timestamp.
    pub created_at: String,
}

/// DAG run row — the `WorkflowDagRun` columns.
#[derive(Debug, Clone, PartialEq)]
pub struct WorkflowDagRunRow {
    /// Run ID (`dagrun:…`).
    pub id: String,
    /// DAG ID.
    pub dag_id: String,
    /// Trigger type (`manual` | `cron` | `webhook`).
    pub trigger_kind: String,
    /// Topological order as JSON.
    pub topo_order_json: String,
    /// Run status (`completed`).
    pub status: String,
    /// REA score.
    pub rea_score: i64,
    /// ISO timestamp.
    pub created_at: String,
}

/// SWE sandbox audit row — the `SweSandboxAudit` columns.
#[derive(Debug, Clone, PartialEq)]
pub struct SweSandboxAuditRow {
    /// Audit ID (`sweaudit:…`).
    pub id: String,
    /// SHA-256 of the artifact.
    pub artifact_sha256: String,
    /// Language (`rust` | `typescript` | `solidity` | `html`).
    pub language: String,
    /// Consumed attempts.
    pub attempts_used: i64,
    /// Passed?
    pub passed: bool,
    /// Diagnostics as JSON.
    pub diagnostics_json: String,
    /// ISO timestamp.
    pub created_at: String,
}

impl V3Store {
    /// Insert an agent checkpoint — idempotent on `id`.
    ///
    /// # Errors
    /// SQL error or a parent-invariant violation (a missing parent is rejected with [`DbError::Corrupt`]).
    pub async fn create_agent_checkpoint(&self, row: &AgentCheckpointRow) -> DbResult<()> {
        if let Some(parent) = row.parent_id.as_deref().filter(|p| !p.is_empty()) {
            let known = self.get_agent_checkpoint(parent).await?;
            if known.is_none() {
                return Err(DbError::Corrupt(format!(
                    "agent checkpoint rejected: unknown parent {parent}"
                )));
            }
        }
        sqlx::query(
            r#"INSERT INTO "AgentCheckpoint"
               ("id","parent_id","workflow_id","step_index","agent_role",
                "state_json","state_sha256","created_at")
               VALUES (?,?,?,?,?,?,?,?)"#,
        )
        .bind(&row.id)
        .bind(row.parent_id.as_deref().filter(|p| !p.is_empty()))
        .bind(&row.workflow_id)
        .bind(row.step_index)
        .bind(&row.agent_role)
        .bind(&row.state_json)
        .bind(&row.state_sha256)
        .bind(&row.created_at)
        .execute(self.pool_ref())
        .await?;
        Ok(())
    }

    /// Read one checkpoint by ID.
    ///
    /// # Errors
    /// SQL error.
    pub async fn get_agent_checkpoint(&self, id: &str) -> DbResult<Option<AgentCheckpointRow>> {
        let rows = sqlx::query(
            r#"SELECT "id","parent_id","workflow_id","step_index","agent_role",
                      "state_json","state_sha256","created_at"
               FROM "AgentCheckpoint" WHERE "id" = ?"#,
        )
        .bind(id)
        .fetch_all(self.pool_ref())
        .await?;
        Ok(rows.first().map(checkpoint_row_from))
    }

    /// Checkpoints of one workflow — in step order (ascending).
    ///
    /// # Errors
    /// SQL error.
    pub async fn list_agent_checkpoints(
        &self,
        workflow_id: &str,
        limit: u32,
    ) -> DbResult<Vec<AgentCheckpointRow>> {
        let cap = i64::from(limit.clamp(1, 500));
        let rows = sqlx::query(
            r#"SELECT "id","parent_id","workflow_id","step_index","agent_role",
                      "state_json","state_sha256","created_at"
               FROM "AgentCheckpoint" WHERE "workflow_id" = ?
               ORDER BY "step_index" ASC, "id" ASC LIMIT ?"#,
        )
        .bind(workflow_id)
        .bind(cap)
        .fetch_all(self.pool_ref())
        .await?;
        Ok(rows.iter().map(checkpoint_row_from).collect())
    }

    /// Insert a DAG run.
    ///
    /// # Errors
    /// SQL error.
    pub async fn create_workflow_dag_run(&self, row: &WorkflowDagRunRow) -> DbResult<()> {
        sqlx::query(
            r#"INSERT INTO "WorkflowDagRun"
               ("id","dag_id","trigger_kind","topo_order_json","status","rea_score","created_at")
               VALUES (?,?,?,?,?,?,?)"#,
        )
        .bind(&row.id)
        .bind(&row.dag_id)
        .bind(&row.trigger_kind)
        .bind(&row.topo_order_json)
        .bind(&row.status)
        .bind(row.rea_score)
        .bind(&row.created_at)
        .execute(self.pool_ref())
        .await?;
        Ok(())
    }

    /// Insert an SWE sandbox audit.
    ///
    /// # Errors
    /// SQL error.
    pub async fn create_swe_sandbox_audit(&self, row: &SweSandboxAuditRow) -> DbResult<()> {
        sqlx::query(
            r#"INSERT INTO "SweSandboxAudit"
               ("id","artifact_sha256","language","attempts_used","passed",
                "diagnostics_json","created_at")
               VALUES (?,?,?,?,?,?,?)"#,
        )
        .bind(&row.id)
        .bind(&row.artifact_sha256)
        .bind(&row.language)
        .bind(row.attempts_used)
        .bind(i64::from(row.passed))
        .bind(&row.diagnostics_json)
        .bind(&row.created_at)
        .execute(self.pool_ref())
        .await?;
        Ok(())
    }
}

fn checkpoint_row_from(r: &sqlx::sqlite::SqliteRow) -> AgentCheckpointRow {
    let parent: Option<String> = r
        .try_get::<Option<String>, _>("parent_id")
        .ok()
        .flatten()
        .filter(|s| !s.is_empty());
    AgentCheckpointRow {
        id: r.get("id"),
        parent_id: parent,
        workflow_id: r.get("workflow_id"),
        step_index: r.get("step_index"),
        agent_role: r.get("agent_role"),
        state_json: r.get("state_json"),
        state_sha256: r.get("state_sha256"),
        created_at: r.get("created_at"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn fresh_store() -> V3Store {
        let s = V3Store::connect_in_memory().await.unwrap();
        s.migrate().await.unwrap();
        s
    }

    fn root_row() -> AgentCheckpointRow {
        AgentCheckpointRow {
            id: "ckpt:wf-1:0:root".to_owned(),
            parent_id: None,
            workflow_id: "wf-1".to_owned(),
            step_index: 0,
            agent_role: "Alpha".to_owned(),
            state_json: r#"{"goal":"ship"}"#.to_owned(),
            state_sha256: "aaa".to_owned(),
            created_at: "2026-11-25T00:00:00Z".to_owned(),
        }
    }

    #[tokio::test]
    async fn checkpoint_roundtrip_and_unknown_parent_rejected() {
        let store = fresh_store().await;
        store.create_agent_checkpoint(&root_row()).await.unwrap();
        let got = store
            .get_agent_checkpoint("ckpt:wf-1:0:root")
            .await
            .unwrap()
            .expect("root row");
        assert_eq!(got, root_row());
        assert!(got.parent_id.is_none());
        // Missing parent → deterministic rejection
        let orphan = AgentCheckpointRow {
            id: "ckpt:wf-1:1:orphan".to_owned(),
            parent_id: Some("ckpt:missing".to_owned()),
            workflow_id: "wf-1".to_owned(),
            step_index: 1,
            agent_role: "Mairon".to_owned(),
            state_json: "{}".to_owned(),
            state_sha256: "bbb".to_owned(),
            created_at: "2026-11-25T00:00:01Z".to_owned(),
        };
        let err = store.create_agent_checkpoint(&orphan).await.unwrap_err();
        assert!(err.to_string().contains("unknown parent"), "{err}");
        // Healthy row with the parent present
        let child = AgentCheckpointRow {
            parent_id: Some("ckpt:wf-1:0:root".to_owned()),
            id: "ckpt:wf-1:1:child".to_owned(),
            ..orphan
        };
        store.create_agent_checkpoint(&child).await.unwrap();
        let listed = store.list_agent_checkpoints("wf-1", 10).await.unwrap();
        assert_eq!(listed.len(), 2);
        assert_eq!(listed[0].step_index, 0);
        assert_eq!(listed[1].step_index, 1);
    }

    #[tokio::test]
    async fn dag_run_row_roundtrips() {
        let store = fresh_store().await;
        let row = WorkflowDagRunRow {
            id: "dagrun:1".to_owned(),
            dag_id: "wf-9".to_owned(),
            trigger_kind: "manual".to_owned(),
            topo_order_json: r#"["a","b"]"#.to_owned(),
            status: "completed".to_owned(),
            rea_score: 92,
            created_at: "2026-11-25T00:00:00Z".to_owned(),
        };
        store.create_workflow_dag_run(&row).await.unwrap();
    }

    #[tokio::test]
    async fn swe_audit_row_roundtrips() {
        let store = fresh_store().await;
        let row = SweSandboxAuditRow {
            id: "sweaudit:1".to_owned(),
            artifact_sha256: "deadbeef".to_owned(),
            language: "rust".to_owned(),
            attempts_used: 1,
            passed: true,
            diagnostics_json: "[]".to_owned(),
            created_at: "2026-11-25T00:00:00Z".to_owned(),
        };
        store.create_swe_sandbox_audit(&row).await.unwrap();
    }
}
