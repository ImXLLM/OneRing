//! Founder war-room handlers (Wave 10 — Commit #30).
//!
//! Five new endpoints:
//! - [`war_room_state`] — public; day-zero status + the canonical 24-tick scenario;
//! - [`war_room_step`] — citizen session; single-step causal advance + tick persistence;
//! - [`war_room_directive`] — citizen session; chairman directive + three
//!   in-persona replies + log persistence;
//! - [`war_room_reset`] — citizen session; causal reset to day zero
//!   (ticks and chairman directives are cleared so `step` can be replayed);
//! - [`war_room_explain_simple`] — public; the "0 to 100" explanation for zero literacy.
//!
//! All numbers come from [`kian_hermes::war_room_simulation_engine`] — no
//! handler fabricates numbers on its own (single source of truth).

use std::collections::BTreeMap;
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::Json;
use sha2::{Digest, Sha256};

use kian_contracts::wave10::{
    FounderReplyView, FounderView, WalletLedgerView, WarRoomDirectiveRequest,
    WarRoomDirectiveResponse, WarRoomExplainResponse, WarRoomResetResponse,
    WarRoomStateResponse, WarRoomStepRequest, WarRoomStepResponse, WarRoomTickView,
};
use kian_db::now_iso;
use kian_db::wave10::{ChairmanDirectiveLogRow, FounderWarRoomTickRow};
use kian_hermes::war_room_simulation_engine::{
    canonical_day0_scenario, explain_zero_to_hundred_elif5, ChairmanIntent, FounderPersona, Locale,
    OrganizationWalletLedger, PipelineStage, RevenueStream, WarRoomError, WarRoomState,
    BASE_CHAIN_ID, BASE_USDC_CONTRACT, CANONICAL_TICK_COUNT, CREATOR_PAYOUT_BPS,
    DAY1_TOTAL_EARNED_MICROS, GITHUB_COMMIT_SHORT, GITHUB_REPO, MAX_TICK_DELTA_SECONDS,
    PLATFORM_TAKE_BPS_UNREFERRED, TELEGRAM_STARS_CURRENCY, WAR_ROOM_SIGNATURE,
};

use crate::error::{ApiFailure, ApiResult};
use kian_contracts::HttpFailure;
use crate::middleware::require_session;
use crate::state::AppState;

/// Maps a pipeline stage to its wire ID — for persistence and DTO.
const fn stage_id(stage: PipelineStage) -> &'static str {
    stage.stage_id()
}

/// Maps a revenue stream to its wire ID.
const fn stream_id(stream: RevenueStream) -> &'static str {
    stream.stream_id()
}

/// Builds the wallet-ledger view from engine state.
#[must_use]
pub fn wallet_view(wallet: &OrganizationWalletLedger) -> WalletLedgerView {
    WalletLedgerView {
        crypto_seed_micros: wallet.crypto_seed_micros,
        cash_seed_cents: wallet.cash_seed_cents,
        paper_reserve_locked_cents: wallet.paper_reserve_locked_cents,
        earned_revenue_micros: wallet.earned_revenue_micros,
        creator_payouts_micros: wallet.creator_payouts_micros,
        affiliate_payouts_micros: wallet.affiliate_payouts_micros,
        stream_marketplace_tax_micros: wallet.stream_marketplace_tax_micros,
        stream_affiliate_routing_micros: wallet.stream_affiliate_routing_micros,
        stream_featured_boost_micros: wallet.stream_featured_boost_micros,
        stream_settlement_spread_micros: wallet.stream_settlement_spread_micros,
        active_users: i64::try_from(wallet.active_users).unwrap_or(i64::MAX),
        funnel_visits: i64::try_from(wallet.funnel_visits).unwrap_or(i64::MAX),
        funnel_signups: i64::try_from(wallet.funnel_signups).unwrap_or(i64::MAX),
        funnel_purchases: i64::try_from(wallet.funnel_purchases).unwrap_or(i64::MAX),
        earned_revenue_usd: wallet.earned_revenue_usd_string(),
        crypto_seed_usd: wallet.crypto_seed_usd_string(),
        cash_seed_usd: wallet.cash_seed_usd_string(),
        paper_reserve_usd: wallet.paper_reserve_usd_string(),
    }
}

/// Builds the full wire view of one tick event.
#[must_use]
pub fn tick_view(
    event: &kian_hermes::war_room_simulation_engine::SimulationTickEvent,
) -> WarRoomTickView {
    WarRoomTickView {
        tick_index: i64::from(event.tick_index),
        simulation_second: event.simulation_second,
        speaker_id: event.speaker.wire_id().to_owned(),
        speaker_name_en: event.speaker.display_name_en().to_owned(),
        speaker_name_fa: event.speaker.display_name_fa().to_owned(),
        speaker_accent_hex: event.speaker.accent_hex().to_owned(),
        message_fa: event.message_fa.clone(),
        message_en: event.message_en.clone(),
        terminal_cmd: event.terminal_cmd.clone(),
        social_post_json: event.social_post_json.clone(),
        wallet_delta_usdc_micros: event.wallet_delta_usdc_micros,
        wallet_total_usdc_micros: event.wallet_total_usdc_micros,
        active_pipeline_stage: stage_id(event.active_pipeline_stage).to_owned(),
        revenue_stream: stream_id(event.revenue_stream).to_owned(),
        sale_volume_micros: event.sale_volume_micros,
        creator_payout_micros: event.creator_payout_micros,
        referred: event.referred,
    }
}

/// Maps an engine error to a wire error — every failure path is typed.
fn wire_error(err: &WarRoomError) -> ApiFailure {
    match err {
        WarRoomError::EmptyDirective | WarRoomError::DirectiveTooLong(_) => {
            ApiFailure::from(HttpFailure::invalid_request(2))
        }
        WarRoomError::UnknownSpeed(_)
        | WarRoomError::ScenarioExhausted
        | WarRoomError::TimeJumpDetected(_) => ApiFailure::from(HttpFailure::invalid_request(1)),
    }
}

/// The complete war-room personas — in fixed display order.
#[must_use]
pub fn founder_views() -> Vec<FounderView> {
    [
        FounderPersona::ElonMusk,
        FounderPersona::CzBinance,
        FounderPersona::JeffBezos,
        FounderPersona::Chairman,
    ]
    .iter()
    .map(|p| FounderView {
        persona_id: p.wire_id().to_owned(),
        handle: p.handle().to_owned(),
        display_name_en: p.display_name_en().to_owned(),
        display_name_fa: p.display_name_fa().to_owned(),
        mandate_en: p.mandate_en().to_owned(),
        mandate_fa: p.mandate_fa().to_owned(),
        accent_hex: p.accent_hex().to_owned(),
        is_co_founder: p.is_co_founder(),
    })
    .collect()
}

/// `GET /war-room/state` — public; full day-zero status + the canonical scenario.
///
/// # Errors
/// 500 db.
pub async fn war_room_state() -> ApiResult<Json<WarRoomStateResponse>> {
    let scenario = canonical_day0_scenario();
    let ticks: Vec<WarRoomTickView> = scenario
        .iter()
        .map(tick_view)
        .collect();
    Ok(Json(WarRoomStateResponse {
        signature: WAR_ROOM_SIGNATURE.to_owned(),
        github_repo: GITHUB_REPO.to_owned(),
        github_commit_short: GITHUB_COMMIT_SHORT.to_owned(),
        base_chain_id: i64::try_from(BASE_CHAIN_ID).unwrap_or(8453),
        base_usdc_contract: BASE_USDC_CONTRACT.to_owned(),
        telegram_stars_currency: TELEGRAM_STARS_CURRENCY.to_owned(),
        elapsed_seconds: 0,
        tick_cursor: 0,
        progress_percent: 0,
        playing: false,
        speed_multiplier: 1,
        wallet: wallet_view(&OrganizationWalletLedger::seed()),
        founders: founder_views(),
        scenario: ticks,
        charter_unreferred_take_bps: PLATFORM_TAKE_BPS_UNREFERRED,
        charter_creator_bps: CREATOR_PAYOUT_BPS,
        max_tick_delta_seconds: MAX_TICK_DELTA_SECONDS,
        day1_total_earned_micros: DAY1_TOTAL_EARNED_MICROS,
    }))
}

/// `POST /war-room/step` — citizen session; single-step causal advance.
///
/// Body: `{ "tickCursor": 0..=23, "speedMultiplier": 1|2|5 }` — the server replays
/// state from day zero to the cursor, advances exactly one tick, stores the tick row
/// idempotently, and returns the fresh state.
///
/// # Errors
/// 401 missing session; 400 invalid cursor/speed or scenario finished; 500 db.
pub async fn war_room_step(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    bytes: axum::body::Bytes,
) -> ApiResult<Json<WarRoomStepResponse>> {
    let _user = require_session(&state, &headers).await?;
    let req: WarRoomStepRequest = crate::handlers::parse_body(&bytes)?;
    let cursor = usize::try_from(req.tick_cursor.clamp(
        0,
        i64::from(u32::try_from(CANONICAL_TICK_COUNT).unwrap_or(u32::MAX)),
    ))
    .unwrap_or(0);
    let mut room = WarRoomState::new();
    if room
        .set_speed(u32::try_from(req.speed_multiplier).unwrap_or(1))
        .is_err()
    {
        return Err(HttpFailure::invalid_request(1).into());
    }
    // Causal replay to the cursor — every step passes through the same prisms.
    for _ in 0..cursor {
        room.advance_single_tick().map_err(|e| wire_error(&e))?;
    }
    let event = room.advance_single_tick().map_err(|e| wire_error(&e))?;
    // Idempotent persistence of the tick row.
    let row = FounderWarRoomTickRow {
        id: format!("warroom:{:04}", event.tick_index),
        tick_index: i64::from(event.tick_index),
        simulation_second: event.simulation_second,
        speaker_id: event.speaker.wire_id().to_owned(),
        speaker_name: event.speaker.display_name_en().to_owned(),
        message_fa: event.message_fa.clone(),
        message_en: event.message_en.clone(),
        terminal_cmd: event.terminal_cmd.clone(),
        social_post_json: event.social_post_json.clone(),
        wallet_delta_usdc_micros: event.wallet_delta_usdc_micros,
        wallet_total_usdc_micros: event.wallet_total_usdc_micros,
        active_pipeline_stage: stage_id(event.active_pipeline_stage).to_owned(),
        created_at: now_iso(),
    };
    state.store.insert_war_room_tick(&row).await.map_err(ApiFailure::from)?;
    let exhausted = room.tick_cursor >= CANONICAL_TICK_COUNT;
    Ok(Json(WarRoomStepResponse {
        executed: tick_view(&event),
        tick_cursor: i64::try_from(room.tick_cursor).unwrap_or(i64::MAX),
        progress_percent: i64::from(room.progress_percent()),
        elapsed_seconds: room.elapsed_seconds,
        wallet: wallet_view(&room.wallet),
        scenario_exhausted: exhausted,
    }))
}

/// `POST /war-room/directive` — citizen session; the chairman's tactical directive.
///
/// Body: `{ "directiveText": string, "locale": "fa"|"en", "triggeredTick": n }`
/// — deterministic intent classification, three in-persona replies, and log persistence with a
/// hash-based deterministic ID (the same directive on the same tick = no re-write).
///
/// # Errors
/// 401 missing session; 400 empty or over-cap text; 500 db.
pub async fn war_room_directive(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    bytes: axum::body::Bytes,
) -> ApiResult<Json<WarRoomDirectiveResponse>> {
    let _user = require_session(&state, &headers).await?;
    let req: WarRoomDirectiveRequest = crate::handlers::parse_body(&bytes)?;
    let locale = Locale::parse(req.locale.as_str());
    let mut room = WarRoomState::new();
    let tick = usize::try_from(req.triggered_tick.clamp(
        0,
        i64::from(u32::try_from(CANONICAL_TICK_COUNT).unwrap_or(u32::MAX)),
    ))
    .unwrap_or(0);
    for _ in 0..tick {
        room.advance_single_tick().map_err(|e| wire_error(&e))?;
    }
    let log = room
        .inject_chairman_directive(&req.directive_text, locale)
        .map_err(|e| wire_error(&e))?;
    let intent = log.intent;
    // Hash-based deterministic ID — idempotent on (text, tick).
    let mut hasher = Sha256::new();
    hasher.update(log.directive_text.as_bytes());
    hasher.update(tick.to_le_bytes());
    let digest = format!("{:x}", hasher.finalize());
    let replies = &log.replies;
    let row = ChairmanDirectiveLogRow {
        id: format!("directive:{}", &digest[..16]),
        directive_text: log.directive_text.clone(),
        target_founder: format!("intent:{}", intent_id(intent)),
        elon_response: bilingual(replies, 0),
        cz_response: bilingual(replies, 1),
        bezos_response: bilingual(replies, 2),
        triggered_tick: i64::try_from(tick).unwrap_or(0),
        created_at: now_iso(),
    };
    state
        .store
        .insert_chairman_directive(&row)
        .await
        .map_err(ApiFailure::from)?;
    Ok(Json(WarRoomDirectiveResponse {
        directive_text: log.directive_text,
        intent_id: intent_id(intent).to_owned(),
        intent_label_fa: intent.label_fa().to_owned(),
        intent_label_en: intent.label_en().to_owned(),
        replies: reply_views(replies),
        triggered_tick: i64::try_from(log.triggered_tick).unwrap_or(0),
        simulation_second: log.simulation_second,
    }))
}

/// `POST /war-room/reset` — citizen session; causal reset to day zero.
///
/// All recorded ticks and the chairman-directive log are cleared in one transaction so that
/// `POST /war-room/step` can be replayed without hitting scenario exhaustion.
/// Canonical status is read from [`kian_hermes::war_room_simulation_engine`] —
/// resetting the command ledger does not alter the public day-zero status.
///
/// # Errors
/// 401 missing session; 500 db.
pub async fn war_room_reset(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> ApiResult<Json<WarRoomResetResponse>> {
    let _user = require_session(&state, &headers).await?;
    let (deleted_ticks, deleted_directives) = state
        .store
        .reset_war_room_state()
        .await
        .map_err(ApiFailure::from)?;
    Ok(Json(WarRoomResetResponse {
        signature: WAR_ROOM_SIGNATURE.to_owned(),
        deleted_ticks: i64::try_from(deleted_ticks).unwrap_or(i64::MAX),
        deleted_directives: i64::try_from(deleted_directives).unwrap_or(i64::MAX),
        tick_cursor: 0,
        progress_percent: 0,
        message_fa: "\u{627}\u{62a}\u{627}\u{642}\u{650} \u{62c}\u{646}\u{6af} \u{628}\u{647} \u{631}\u{648}\u{632}\u{650} \u{635}\u{641}\u{631} \u{628}\u{627}\u{632}\u{646}\u{634}\u{627}\u{646}\u{6cc} \u{634}\u{62f} — \u{633}\u{646}\u{627}\u{631}\u{6cc}\u{648}\u{6cc}\u{650} \u{6f2}\u{6f4}\u{62a}\u{627}\u{6cc}\u{6cc} \u{627}\u{632} \u{646}\u{648} \u{642}\u{627}\u{628}\u{644}\u{650} \u{631}\u{627}\u{646}\u{634} \u{627}\u{633}\u{62a}".to_owned(),
        message_en: "war-room reset to Day 0 — the 24-tick scenario can be re-driven".to_owned(),
    }))
}

/// `GET /war-room/explain-simple?locale=fa|en` — public; the "0 to 100" explanation.
///
/// # Errors
/// Never for ordinary input — any unknown locale falls back to `en`.
pub async fn war_room_explain_simple(
    Query(q): Query<BTreeMap<String, String>>,
) -> ApiResult<Json<WarRoomExplainResponse>> {
    let locale = Locale::parse(q.get("locale").map_or("fa", String::as_str));
    let bundle = explain_zero_to_hundred_elif5(locale);
    Ok(Json(WarRoomExplainResponse {
        signature: WAR_ROOM_SIGNATURE.to_owned(),
        locale: bundle.locale.as_str().to_owned(),
        headline_fa: bundle.headline_fa,
        headline_en: bundle.headline_en,
        charter_fa: bundle.charter_fa,
        charter_en: bundle.charter_en,
        step_ids: bundle
            .steps
            .iter()
            .map(|s| s.step_id().to_owned())
            .collect(),
        step_titles_fa: bundle
            .steps
            .iter()
            .map(|s| s.title_fa().to_owned())
            .collect(),
        step_titles_en: bundle
            .steps
            .iter()
            .map(|s| s.title_en().to_owned())
            .collect(),
        step_bodies_fa: bundle
            .steps
            .iter()
            .map(|s| s.body_fa().to_owned())
            .collect(),
        step_bodies_en: bundle
            .steps
            .iter()
            .map(|s| s.body_en().to_owned())
            .collect(),
    }))
}

/// Wire ID of the intent.
fn intent_id(intent: ChairmanIntent) -> &'static str {
    match intent {
        ChairmanIntent::ViralHudLaunch => "viral_hud_launch",
        ChairmanIntent::PaymentRails => "payment_rails",
        ChairmanIntent::SimplifyForBeginners => "simplify_for_beginners",
        ChairmanIntent::StatusReport => "status_report",
        ChairmanIntent::QuantTrading => "quant_trading",
        ChairmanIntent::ExplainSimple => "explain_simple",
        ChairmanIntent::ShipNextStep => "ship_next_step",
    }
}

/// A partner's bilingual reply — "Persian --- English" in one column for audit.
fn bilingual(replies: &[kian_hermes::war_room_simulation_engine::FounderReply], index: usize) -> String {
    replies.get(index).map_or(String::new(), |r| {
        format!("{}\n---\n{}", r.message_fa, r.message_en)
    })
}

/// Maps engine replies to the wire view.
fn reply_views(
    replies: &[kian_hermes::war_room_simulation_engine::FounderReply],
) -> Vec<FounderReplyView> {
    replies
        .iter()
        .map(|r| FounderReplyView {
            speaker_id: r.speaker.wire_id().to_owned(),
            speaker_name_en: r.speaker.display_name_en().to_owned(),
            speaker_name_fa: r.speaker.display_name_fa().to_owned(),
            accent_hex: r.speaker.accent_hex().to_owned(),
            message_fa: r.message_fa.clone(),
            message_en: r.message_en.clone(),
            terminal_cmd: r.terminal_cmd.clone(),
            social_post_json: r.social_post_json.clone(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use kian_db::V3Store;
    use tower::ServiceExt;

    /// Full app with a dev session — same pattern as the routes.rs tests.
    async fn hardened_app() -> axum::Router {
        let mut cfg = crate::config::ServerConfig::base();
        cfg.admin_key = Some("test-admin-key-0123456789".to_owned());
        cfg.rate_limit = 10_000;
        cfg.cors_origins = vec!["https://mini.0xkia.app".to_owned()];
        cfg.dev_mode = true;
        let store = V3Store::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        store.ensure_system_accounts().await.unwrap();
        crate::routes::build_router(crate::state::AppState::new(store, cfg).shared())
    }

    /// Chairman session via the official dev-login path — bearer token for session-protected endpoints.
    async fn chairman_session(app: &axum::Router) -> String {
        let req = axum::http::Request::builder()
            .method("POST")
            .uri("/api/v1/auth/dev-login")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"tgUserId":"6398636666","username":"chairman"}"#))
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status().as_u16(), 201, "dev-login must mint the chairman session");
        let bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        json["token"].as_str().expect("session token").to_owned()
    }

    async fn json_of(res: axum::response::Response) -> serde_json::Value {
        let bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&bytes).expect("json body")
    }

    /// `GET /war-room/state` — public and canonical: signature + 24-tick scenario + honest day-zero wallet.
    #[tokio::test]
    async fn war_room_state_retrieval_is_public_and_canonical() {
        let app = hardened_app().await;
        let req = axum::http::Request::builder()
            .method("GET")
            .uri("/api/v1/war-room/state")
            .body(Body::empty())
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status().as_u16(), 200, "war-room/state is public");
        let json = json_of(res).await;
        assert_eq!(json["signature"], "MAIRON-WAR-ROOM");
        assert_eq!(json["scenario"].as_array().map(Vec::len), Some(24));
        assert_eq!(json["wallet"]["cryptoSeedMicros"], 33_000_000);
        assert_eq!(json["wallet"]["earnedRevenueMicros"], 0);
        assert_eq!(json["day1TotalEarnedMicros"], 9_550_000);
        assert_eq!(json["founders"].as_array().map(Vec::len), Some(4));
    }

    /// `POST /war-room/directive` — deterministic classification + three bilingual in-persona replies.
    #[tokio::test]
    async fn war_room_directive_executes_with_three_in_character_replies() {
        let app = hardened_app().await;
        let token = chairman_session(&app).await;
        let req = axum::http::Request::builder()
            .method("POST")
            .uri("/api/v1/war-room/directive")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::from(
                r#"{"directiveText":"What is today's wallet report?","locale":"fa","triggeredTick":3}"#,
            ))
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status().as_u16(), 200, "directive must execute for a citizen session");
        let json = json_of(res).await;
        assert!(json["intentId"].as_str().is_some(), "deterministic intent must be wired");
        assert_eq!(json["triggeredTick"], 3);
        let replies = json["replies"].as_array().expect("three replies");
        assert_eq!(replies.len(), 3, "elon + CZ + bezos must all answer");
        for r in replies {
            assert_ne!(r["messageFa"].as_str().unwrap_or_default(), "");
            assert_ne!(r["messageEn"].as_str().unwrap_or_default(), "");
        }
    }

    /// `POST /war-room/reset` — causal return to day zero; `step` can be replayed.
    #[tokio::test]
    async fn war_room_reset_restores_day0_and_step_can_be_redriven() {
        let app = hardened_app().await;
        let token = chairman_session(&app).await;
        let bearer = format!("Bearer {token}");
        // Two causal steps — ticks 0 and 1 persist in the command ledger.
        for cursor in [0, 1] {
            let req = axum::http::Request::builder()
                .method("POST")
                .uri("/api/v1/war-room/step")
                .header("content-type", "application/json")
                .header("authorization", &bearer)
                .body(Body::from(format!("{{\"tickCursor\":{cursor},\"speedMultiplier\":1}}")))
                .unwrap();
            let res = app.clone().oneshot(req).await.unwrap();
            assert_eq!(res.status().as_u16(), 200, "step must advance at cursor {cursor}");
        }
        // One chairman directive — persists in the command log.
        let req = axum::http::Request::builder()
            .method("POST")
            .uri("/api/v1/war-room/directive")
            .header("content-type", "application/json")
            .header("authorization", &bearer)
            .body(Body::from(r#"{"directiveText":"probe","locale":"en","triggeredTick":2}"#))
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status().as_u16(), 200);
        // Reset — exact count of cleared rows.
        let req = axum::http::Request::builder()
            .method("POST")
            .uri("/api/v1/war-room/reset")
            .header("authorization", &bearer)
            .body(Body::empty())
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status().as_u16(), 200, "reset must succeed for a citizen session");
        let json = json_of(res).await;
        assert_eq!(json["signature"], "MAIRON-WAR-ROOM");
        assert_eq!(json["deletedTicks"], 2, "exactly the two persisted ticks");
        assert_eq!(json["deletedDirectives"], 1, "exactly the one persisted directive");
        assert_eq!(json["tickCursor"], 0);
        assert_eq!(json["progressPercent"], 0);
        // Re-run — step succeeds without scenario exhaustion.
        let req = axum::http::Request::builder()
            .method("POST")
            .uri("/api/v1/war-room/step")
            .header("content-type", "application/json")
            .header("authorization", bearer)
            .body(Body::from(r#"{"tickCursor":0,"speedMultiplier":1}"#))
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status().as_u16(), 200, "step must be re-drivable after reset");
        let json = json_of(res).await;
        assert_eq!(json["executed"]["tickIndex"], 0, "fresh drive starts at tick 0");
    }
}
