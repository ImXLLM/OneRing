//! هندلرهای سیستم — health عمیق، بکاپ و شفافیت اقتصاد (مادهٔ ۶ منشور).
//!
//! M2: چیپ‌های bot/ai صادقانه «disabled» بودند (سیم‌کشی واقعی با M3/M4)؛
//! بکاپ `VACUUM INTO` از همین‌جا در دسترس ادمین است.

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

/// `GET /health` — liveness: سروری که به دیتابیسش نمی‌رسد سالم نیست.
///
/// اصلاح تست بار ۲۰۲۵-۱۱: چک سلامت حالا پینگ سبک است (`SELECT 1`)، نه
/// خوانش عمیق دفتر که این مسیر را به ۸۶ RPS محدود کرده بود.
///
/// # Errors
/// دیتابیس در دسترس نیست → 503 `DB_DOWN`.
pub async fn health(State(state): State<Arc<AppState>>) -> ApiResult<Json<HealthResponse>> {
    // پینگ سبک — گپ واقعی بین پروسه و دیتابیس را می‌گیرد؛ خوانش عمیقِ آشتی
    // در `/economy` ماند (اکنون با SQL تجمیعی، سبک و O(1) نسبت به تعداد اکانت).
    state.store.ping().await.map_err(|_| kian_contracts::HttpFailure::db_down())?;
    // M4 — گیت‌وی AI از پیکربندی بوت؛ اسلات کانفیگ‌نشده صادقانه disabled،
    // هرگز دروغ. بات (M6) فقط با هر دو راز توکن+وبهوک «enabled» است —
    // همان معناشناسی fail-closed؛ بدون راز صادقانه disabled.
    let primary = state
        .ai
        .slot_statuses()
        .into_iter()
        .find(|s| s.enabled)
        // چیپ health کوتاه می‌ماند — فقط اولین مدلِ گروهِ برنده (گروه چندمدلی با کاما می‌آید)
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

/// `POST /system/backup` — اسنپ‌شات سازگار SQLite (`VACUUM INTO`) + هرس. ادمین.
///
/// # Errors
/// فروشگاه درون‌حافظه‌ای → 503 `BACKUP_UNSUPPORTED`؛ شکست SQL → `BACKUP_FAILED`.
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

/// `GET /economy` — خلاصهٔ اقتصاد ملی؛ عمومی و فقط‌خواندنی (مادهٔ ۶).
///
/// # Errors
/// خطای SQL.
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
