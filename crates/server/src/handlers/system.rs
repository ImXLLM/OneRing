//! System handlers — deep health, backup, and economy transparency (charter article 6).
//!
//! M2: the bot/ai chips were honestly "disabled" (real wiring came with M3/M4);
//! the `VACUUM INTO` backup is admin-accessible from here.

use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use std::sync::Arc;

use kian_contracts::{
    BackupCreated, Currency, DinSymbol, EconomySummary, HealthResponse, HealthService, HealthStatus,
};
use kian_domain::Charter;

use crate::error::ApiResult;
use crate::middleware::require_admin;
use crate::state::AppState;
use crate::views::wire_din;

/// `GET /health` — liveness: a server that cannot reach its database is not healthy.
///
/// 2025-11 load-test fix: the health check is now a light ping (`SELECT 1`), not
/// a deep ledger read, which had capped this path at 86 RPS.
///
/// # Errors
/// Database unreachable → 503 `DB_DOWN`.
pub async fn health(State(state): State<Arc<AppState>>) -> ApiResult<Json<HealthResponse>> {
    // light ping — catches a real gap between the process and the database; the deep reconciliation
    // read stayed in `/economy` (now with aggregate SQL, light and O(1) in account count).
    state.store.ping().await.map_err(|_| kian_contracts::HttpFailure::db_down())?;
    // M4 — the AI gateway comes from boot configuration; an unconfigured slot is honestly disabled,
    // never a lie. The bot (M6) is "enabled" only with both token+webhook secrets —
    // the same fail-closed semantics; without secrets, honestly disabled.
    let primary = state
        .ai
        .slot_statuses()
        .into_iter()
        .find(|s| s.enabled)
        // the health chip stays short — only the winning group's first model (a multi-model group arrives comma-separated)
        .map(|s| s.model.split(',').next().unwrap_or_default().trim().to_owned())
        .filter(|m| !m.is_empty());
    let bot_enabled = state.bot.is_some();
    Ok(Json(HealthResponse {
        status: HealthStatus::Ok,
        service: HealthService::KianKernelServer,
        version: crate::VERSION.to_owned(),
        llm_configured: state.ai.enabled(),
        llm_model: primary,
        db: Some("ok".into()),
        bot: Some(if bot_enabled { "enabled" } else { "disabled" }.into()),
        ai: Some(if state.ai.enabled() { "enabled" } else { "disabled" }.into()),
        uptime: Some(state.started_at.elapsed().as_secs_f64()),
    }))
}

/// `POST /system/backup` — SQLite-consistent snapshot (`VACUUM INTO`) + pruning. Admin.
///
/// # Errors
/// In-memory store → 503 `BACKUP_UNSUPPORTED`; SQL failure → `BACKUP_FAILED`.
pub async fn create_backup(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> ApiResult<Json<BackupCreated>> {
    require_admin(&headers, state.config.admin_key.as_deref())?;
    if state.store.file_path().is_none() {
        return Err(kian_contracts::HttpFailure::backup_unsupported().into());
    }
    let info = state
        .store
        .backup_to_dir(std::path::Path::new(&state.config.backup_dir), state.config.keep_backups)
        .await
        .map_err(|e| kian_contracts::HttpFailure {
            status: 500,
            code: "BACKUP_FAILED".into(),
            message: format!("VACUUM INTO failed — {e}"),
            meta: None,
        })?;
    Ok(Json(BackupCreated {
        file: info.file,
        size_bytes: i64::try_from(info.size_bytes).unwrap_or(i64::MAX),
        kept: info.kept,
        created_at: info.created_at,
    }))
}

/// `GET /economy` — the national economy summary; public and read-only (article 6).
///
/// # Errors
/// SQL error.
pub async fn get_economy(State(state): State<Arc<AppState>>) -> ApiResult<Json<EconomySummary>> {
    let s = state.store.economy_summary().await?;
    let minted_all_time = state.store.minted_all_time().await?;
    let burned_all_time = state.store.burned_all_time().await?;
    let account_count = state.store.account_count().await?;
    Ok(Json(EconomySummary {
        currency: Currency::Dnx,
        symbol: DinSymbol::Dinar,
        circulating_supply: wire_din(s.circulating_supply),
        treasury_balance: wire_din(s.treasury_balance),
        escrow_locked: wire_din(s.escrow_locked),
        minted_all_time: wire_din(minted_all_time),
        burned_all_time: wire_din(burned_all_time),
        minted_this_year: wire_din(s.minted_this_year),
        yearly_mint_cap: wire_din(Charter::YEARLY_MINT_CAP_DIN.as_i64()),
        genesis_supply: wire_din(Charter::GENESIS_SUPPLY_DIN.as_i64()),
        trial_balance_sum: s.trial_sum,
        balanced: s.balanced,
        account_count,
        open_escrows: s.open_escrows,
    }))
}
