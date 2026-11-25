//! ماندگاریِ موتورِ برتریِ رقابتی (Wave 8 — Commit #28).
//!
//! سه عملیاتِ قطعی روی مهاجرتِ `0012`: درج/خوانشِ چک‌پوینت‌هایِ ایجنت
//! (`AgentCheckpoint`)، درجِ ران‌هایِ DAG (`WorkflowDagRun`) و درجِ ممیزی‌های
//! سندباکسِ SWE (`SweSandboxAudit`). هیچ جدولِ دفتریِ دوطرفه لمس نمی‌شود —
//! این‌ها «دفترِ برتری» است نه خودِ پول.

use sqlx::Row;

use crate::error::{DbError, DbResult};
use crate::store::V3Store;

/// ردیفِ چک‌پوینتِ ایجنت — همان ستون‌هایِ `AgentCheckpoint`.
#[derive(Debug, Clone, PartialEq)]
pub struct AgentCheckpointRow {
    /// شناسهٔ چک‌پوینت (`ckpt:…`).
    pub id: String,
    /// والد — `None` برای ریشهٔ گردش‌کار.
    pub parent_id: Option<String>,
    /// شناسهٔ گردش‌کار.
    pub workflow_id: String,
    /// شمارهٔ گام.
    pub step_index: i64,
    /// نقشِ ایجنت.
    pub agent_role: String,
    /// وضعیتِ JSON.
    pub state_json: String,
    /// SHA-256 وضعیت.
    pub state_sha256: String,
    /// مُهرِ زمانِ ISO.
    pub created_at: String,
}

/// ردیفِ رانِ DAG — همان ستون‌هایِ `WorkflowDagRun`.
#[derive(Debug, Clone, PartialEq)]
pub struct WorkflowDagRunRow {
    /// شناسهٔ ران (`dagrun:…`).
    pub id: String,
    /// شناسهٔ DAG.
    pub dag_id: String,
    /// نوعِ تریگر (`manual` | `cron` | `webhook`).
    pub trigger_kind: String,
    /// ترتیبِ توپولوژیک به JSON.
    pub topo_order_json: String,
    /// وضعیتِ ران (`completed`).
    pub status: String,
    /// امتیاز REA.
    pub rea_score: i64,
    /// مُهرِ زمانِ ISO.
    pub created_at: String,
}

/// ردیفِ ممیزیِ سندباکسِ SWE — همان ستون‌هایِ `SweSandboxAudit`.
#[derive(Debug, Clone, PartialEq)]
pub struct SweSandboxAuditRow {
    /// شناسهٔ ممیزی (`sweaudit:…`).
    pub id: String,
    /// SHA-256 آرتیفکت.
    pub artifact_sha256: String,
    /// زبان (`rust` | `typescript` | `solidity` | `html`).
    pub language: String,
    /// تلاش‌هایِ مصرف‌شده.
    pub attempts_used: i64,
    /// عبور؟
    pub passed: bool,
    /// تشخیص‌ها به JSON.
    pub diagnostics_json: String,
    /// مُهرِ زمانِ ISO.
    pub created_at: String,
}

impl V3Store {
    /// درجِ چک‌پوینتِ ایجنت — ایدمپوتنت روی `id`.
    ///
    /// # Errors
    /// خطای SQL یا نقضِ ناورنیِ والد (والدِ غایب با [`DbError::Corrupt`] رد می‌شود).
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

    /// خوانشِ یک چک‌پوینت با شناسه.
    ///
    /// # Errors
    /// خطای SQL.
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

    /// چک‌پوینت‌هایِ یک گردش‌کار — به ترتیبِ گام (صعودی).
    ///
    /// # Errors
    /// خطای SQL.
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

    /// درجِ رانِ DAG.
    ///
    /// # Errors
    /// خطای SQL.
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

    /// درجِ ممیزیِ سندباکسِ SWE.
    ///
    /// # Errors
    /// خطای SQL.
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
        // والدِ غایب → ردِ قطعی
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
        // ردیفِ سالم با والدِ حاضر
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
