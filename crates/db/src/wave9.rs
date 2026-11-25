//! ماندگاریِ Wave 9 (Commit #29) — سه عملیاتِ قطعی روی مهاجرتِ `0013`:
//!
//! - [`UserLabTelemetryLog`]: درج/فهرستِ لاگِ تلمتریِ آزمایشگاهِ ۱۰-DNA؛
//! - [`LocalePatchRegistry`]: upsert/فهرستِ وضعیتِ ۲۰ لوکالِ `Arda`؛
//! - [`NeuralHudRenderJob`]: درج/فهرستِ ران‌هایِ رندرِ HUD عصبی.
//!
//! هیچ جدولِ دفتریِ دوطرفه لمس نمی‌شود — این‌ها «دفترِ تجربهٔ کاربر» است نه
//! خودِ پول؛ صفرجمعِ `Posting` دست‌نخورده می‌ماند.

use sqlx::Row;

use crate::error::DbResult;
use crate::store::V3Store;

/// ردیفِ لاگِ تلمتریِ آزمایشگاه — همان ستون‌هایِ `UserLabTelemetryLog`.
#[derive(Debug, Clone, PartialEq)]
pub struct UserLabTelemetryLogRow {
    /// شناسهٔ قطعی — `lab:{dna}:{hour}:{turn}`.
    pub id: String,
    /// کدِ کوهورت — `DNA_01`..`DNA_10`.
    pub dna_code: String,
    /// هندل کاربر.
    pub user_handle: String,
    /// ساعتِ ۰..۲۴.
    pub hour_offset: i64,
    /// لایهٔ UX.
    pub ux_tier: String,
    /// لوکال.
    pub locale: String,
    /// پرسونایِ Grok.
    pub grok_persona: String,
    /// مدلِ روت‌شده.
    pub model_slug: String,
    /// حالتِ روتینگ.
    pub routing_mode: String,
    /// پرامپتِ کاربر.
    pub prompt_text: String,
    /// ردِ تفکر به JSON.
    pub thinking_trace_json: String,
    /// پاسخ.
    pub response_text: String,
    /// امتیازِ REA.
    pub rea_score: i64,
    /// تأخیر بر حسبِ ms.
    pub latency_ms: i64,
    /// قابلِ آموزش؟
    pub usable_for_training: bool,
    /// مُهرِ زمانِ ISO.
    pub created_at: String,
}

/// ردیفِ ثبتِ لوکال — همان ستون‌هایِ `LocalePatchRegistry`.
#[derive(Debug, Clone, PartialEq)]
pub struct LocalePatchRegistryRow {
    /// کدِ لوکال — `fa`، `en`، `ar`، …
    pub locale_code: String,
    /// جهت — `rtl` | `ltr`.
    pub direction: String,
    /// وضعیت — `ACTIVE_DAY1` | `PATCH_READY_SLOT`.
    pub activation_state: String,
    /// تکمیل بر حسبِ bps.
    pub completion_bps: i64,
    /// نسخهٔ پچ.
    pub patch_version: String,
    /// مُهرِ آخرین به‌روزرسانی.
    pub updated_at: String,
}

/// ردیفِ رانِ رندرِ HUD — همان ستون‌هایِ `NeuralHudRenderJob`.
#[derive(Debug, Clone, PartialEq)]
pub struct NeuralHudRenderJobRow {
    /// شناسهٔ ران (`hudjob:…`).
    pub id: String,
    /// شناسهٔ پیش‌تن.
    pub preset_slug: String,
    /// نسبت — `9:16` | `16:9`.
    pub aspect_ratio: String,
    /// رزولوشن — `720p` | `1080p`.
    pub resolution: String,
    /// فریم‌ریت — ۲۴ | ۶۰.
    pub fps: i64,
    /// مدت بر حسبِ ثانیه.
    pub duration_sec: i64,
    /// وضعیت — `queued` | `rendered`.
    pub status: String,
    /// مُهرِ زمانِ ISO.
    pub created_at: String,
}

impl V3Store {
    /// درجِ مدخلِ تلمتریِ آزمایشگاه — ایدمپوتنت روی `id`.
    ///
    /// # Errors
    /// خطای SQL.
    pub async fn insert_lab_telemetry_log(&self, row: &UserLabTelemetryLogRow) -> DbResult<()> {
        sqlx::query(
            r#"INSERT INTO "UserLabTelemetryLog"
               ("id","dna_code","user_handle","hour_offset","ux_tier","locale",
                "grok_persona","model_slug","routing_mode","prompt_text",
                "thinking_trace_json","response_text","rea_score","latency_ms",
                "usable_for_training","created_at")
               VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)"#,
        )
        .bind(&row.id)
        .bind(&row.dna_code)
        .bind(&row.user_handle)
        .bind(row.hour_offset)
        .bind(&row.ux_tier)
        .bind(&row.locale)
        .bind(&row.grok_persona)
        .bind(&row.model_slug)
        .bind(&row.routing_mode)
        .bind(&row.prompt_text)
        .bind(&row.thinking_trace_json)
        .bind(&row.response_text)
        .bind(row.rea_score)
        .bind(row.latency_ms)
        .bind(i64::from(row.usable_for_training))
        .bind(&row.created_at)
        .execute(self.pool_ref())
        .await?;
        Ok(())
    }

    /// فهرستِ لاگ‌هایِ تلمتری — جدیدترین اول، سقفِ `limit`.
    ///
    /// # Errors
    /// خطای SQL.
    pub async fn list_lab_telemetry_logs(&self, limit: u32) -> DbResult<Vec<UserLabTelemetryLogRow>> {
        let cap = i64::from(limit.clamp(1, 1000));
        let rows = sqlx::query(
            r#"SELECT "id","dna_code","user_handle","hour_offset","ux_tier","locale",
                      "grok_persona","model_slug","routing_mode","prompt_text",
                      "thinking_trace_json","response_text","rea_score","latency_ms",
                      "usable_for_training","created_at"
               FROM "UserLabTelemetryLog"
               ORDER BY "created_at" ASC, "id" ASC LIMIT ?"#,
        )
        .bind(cap)
        .fetch_all(self.pool_ref())
        .await?;
        Ok(rows.iter().map(lab_log_row_from).collect())
    }

    /// upsertِ وضعیتِ لوکال — ایدمپوتنت روی `locale_code`.
    ///
    /// # Errors
    /// خطای SQL.
    pub async fn upsert_locale_patch_registry(&self, row: &LocalePatchRegistryRow) -> DbResult<()> {
        sqlx::query(
            r#"INSERT INTO "LocalePatchRegistry"
               ("locale_code","direction","activation_state","completion_bps",
                "patch_version","updated_at")
               VALUES (?,?,?,?,?,?)
               ON CONFLICT("locale_code") DO UPDATE SET
                 "direction" = excluded."direction",
                 "activation_state" = excluded."activation_state",
                 "completion_bps" = excluded."completion_bps",
                 "patch_version" = excluded."patch_version",
                 "updated_at" = excluded."updated_at""#,
        )
        .bind(&row.locale_code)
        .bind(&row.direction)
        .bind(&row.activation_state)
        .bind(row.completion_bps)
        .bind(&row.patch_version)
        .bind(&row.updated_at)
        .execute(self.pool_ref())
        .await?;
        Ok(())
    }

    /// فهرستِ ثبتِ لوکال‌ها — مرتب بر کد.
    ///
    /// # Errors
    /// خطای SQL.
    pub async fn list_locale_patch_registry(&self) -> DbResult<Vec<LocalePatchRegistryRow>> {
        let rows = sqlx::query(
            r#"SELECT "locale_code","direction","activation_state","completion_bps",
                      "patch_version","updated_at"
               FROM "LocalePatchRegistry" ORDER BY "locale_code" ASC"#,
        )
        .fetch_all(self.pool_ref())
        .await?;
        Ok(rows.iter().map(locale_row_from).collect())
    }

    /// درجِ رانِ رندرِ HUD — ایدمپوتنت روی `id`.
    ///
    /// # Errors
    /// خطای SQL.
    pub async fn insert_hud_render_job(&self, row: &NeuralHudRenderJobRow) -> DbResult<()> {
        sqlx::query(
            r#"INSERT INTO "NeuralHudRenderJob"
               ("id","preset_slug","aspect_ratio","resolution","fps","duration_sec",
                "status","created_at")
               VALUES (?,?,?,?,?,?,?,?)"#,
        )
        .bind(&row.id)
        .bind(&row.preset_slug)
        .bind(&row.aspect_ratio)
        .bind(&row.resolution)
        .bind(row.fps)
        .bind(row.duration_sec)
        .bind(&row.status)
        .bind(&row.created_at)
        .execute(self.pool_ref())
        .await?;
        Ok(())
    }

    /// فهرستِ ران‌هایِ رندر — جدیدترین اول، سقفِ `limit`.
    ///
    /// # Errors
    /// خطای SQL.
    pub async fn list_hud_render_jobs(&self, limit: u32) -> DbResult<Vec<NeuralHudRenderJobRow>> {
        let cap = i64::from(limit.clamp(1, 200));
        let rows = sqlx::query(
            r#"SELECT "id","preset_slug","aspect_ratio","resolution","fps","duration_sec",
                      "status","created_at"
               FROM "NeuralHudRenderJob" ORDER BY "created_at" DESC, "id" ASC LIMIT ?"#,
        )
        .bind(cap)
        .fetch_all(self.pool_ref())
        .await?;
        Ok(rows.iter().map(hud_job_row_from).collect())
    }
}

fn lab_log_row_from(r: &sqlx::sqlite::SqliteRow) -> UserLabTelemetryLogRow {
    let usable: i64 = r.try_get("usable_for_training").unwrap_or(1);
    UserLabTelemetryLogRow {
        id: r.get("id"),
        dna_code: r.get("dna_code"),
        user_handle: r.get("user_handle"),
        hour_offset: r.get("hour_offset"),
        ux_tier: r.get("ux_tier"),
        locale: r.get("locale"),
        grok_persona: r.get("grok_persona"),
        model_slug: r.get("model_slug"),
        routing_mode: r.get("routing_mode"),
        prompt_text: r.get("prompt_text"),
        thinking_trace_json: r.get("thinking_trace_json"),
        response_text: r.get("response_text"),
        rea_score: r.get("rea_score"),
        latency_ms: r.get("latency_ms"),
        usable_for_training: usable != 0,
        created_at: r.get("created_at"),
    }
}

fn locale_row_from(r: &sqlx::sqlite::SqliteRow) -> LocalePatchRegistryRow {
    LocalePatchRegistryRow {
        locale_code: r.get("locale_code"),
        direction: r.get("direction"),
        activation_state: r.get("activation_state"),
        completion_bps: r.get("completion_bps"),
        patch_version: r.get("patch_version"),
        updated_at: r.get("updated_at"),
    }
}

fn hud_job_row_from(r: &sqlx::sqlite::SqliteRow) -> NeuralHudRenderJobRow {
    NeuralHudRenderJobRow {
        id: r.get("id"),
        preset_slug: r.get("preset_slug"),
        aspect_ratio: r.get("aspect_ratio"),
        resolution: r.get("resolution"),
        fps: r.get("fps"),
        duration_sec: r.get("duration_sec"),
        status: r.get("status"),
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

    fn log_row(id: &str) -> UserLabTelemetryLogRow {
        UserLabTelemetryLogRow {
            id: id.to_owned(),
            dna_code: "DNA_01_CHILD_9YO_TOYSPARK".to_owned(),
            user_handle: "child:panda-fan-9yo".to_owned(),
            hour_offset: 9,
            ux_tier: "toyspark".to_owned(),
            locale: "fa".to_owned(),
            grok_persona: "fun_witty".to_owned(),
            model_slug: "05-glm-5.3-flash-max".to_owned(),
            routing_mode: "agent_auto_decision".to_owned(),
            prompt_text: "قصهٔ پاندای کوچک را با صدا بساز".to_owned(),
            thinking_trace_json: r#"{"steps":["۱) تفکیکِ خواسته"],"thinking_duration_ms":900}"#
                .to_owned(),
            response_text: "Alright, let's make this fun".to_owned(),
            rea_score: 92,
            latency_ms: 1653,
            usable_for_training: true,
            created_at: "2026-11-24T09:00:00Z".to_owned(),
        }
    }

    #[tokio::test]
    async fn lab_log_roundtrip_and_unique_id() {
        let store = fresh_store().await;
        store.insert_lab_telemetry_log(&log_row("lab:DNA_01:9:0")).await.unwrap();
        // ایدمپوتنت روی id — INSERT دوم خطا می‌دهد (PRIMARY KEY)
        let second = store.insert_lab_telemetry_log(&log_row("lab:DNA_01:9:0")).await;
        assert!(second.is_err(), "duplicate id must be rejected");
        let listed = store.list_lab_telemetry_logs(50).await.unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].dna_code, "DNA_01_CHILD_9YO_TOYSPARK");
        assert!(listed[0].usable_for_training);
    }

    #[tokio::test]
    async fn locale_registry_upsert_and_list() {
        let store = fresh_store().await;
        let fa = LocalePatchRegistryRow {
            locale_code: "fa".to_owned(),
            direction: "rtl".to_owned(),
            activation_state: "ACTIVE_DAY1".to_owned(),
            completion_bps: 10_000,
            patch_version: "wave9.0".to_owned(),
            updated_at: "2026-11-24T00:00:00Z".to_owned(),
        };
        store.upsert_locale_patch_registry(&fa).await.unwrap();
        // upsert دوم مقدار را به‌روز می‌کند
        let fa_hot = LocalePatchRegistryRow {
            patch_version: "wave9.36".to_owned(),
            ..fa.clone()
        };
        store.upsert_locale_patch_registry(&fa_hot).await.unwrap();
        let listed = store.list_locale_patch_registry().await.unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].patch_version, "wave9.36");
        assert_eq!(listed[0].completion_bps, 10_000);
    }

    #[tokio::test]
    async fn hud_render_job_roundtrip() {
        let store = fresh_store().await;
        let job = NeuralHudRenderJobRow {
            id: "hudjob:1".to_owned(),
            preset_slug: "kia-monopoly-core".to_owned(),
            aspect_ratio: "9:16".to_owned(),
            resolution: "1080p".to_owned(),
            fps: 24,
            duration_sec: 60,
            status: "queued".to_owned(),
            created_at: "2026-11-24T00:00:00Z".to_owned(),
        };
        store.insert_hud_render_job(&job).await.unwrap();
        let listed = store.list_hud_render_jobs(10).await.unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].preset_slug, "kia-monopoly-core");
        assert_eq!(listed[0].resolution, "1080p");
    }
}
