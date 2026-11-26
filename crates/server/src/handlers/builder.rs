//! AI HTML product builder studio handlers (Phase 4).
//!
//! All routes in this module are authenticated with a citizen Bearer session (`require_session`)
//! and session ownership (`accountId`) is checked on every read and write.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;

use kian_ai::budget::{BudgetDecision, BudgetEnvelope};
use kian_contracts::{
    BuilderChatRequest, BuilderChatResponse, BuilderMessageView, BuilderSessionListResponse,
    BuilderSessionSummary, BuilderSessionView, CreateBuilderSessionRequest, HttpFailure,
    ProductView, PublishBuilderRequest, PublishBuilderResponse, MAX_DELIVERY_PAYLOAD_BYTES,
};
use kian_db::{
    now_iso, AiUsageRow, BuilderMessageRow, BuilderSessionPatch, BuilderSessionRow, ProductRow,
};
use kian_domain::{DomainError, DomainErrorCode, DEFAULT_AFFILIATE_BPS, MAX_AFFILIATE_BPS};
use kian_contracts::{
    TrinityAlphaRequest, TrinityAlphaResponse, TrinityAlleleView, TrinityAdvisoryView,
    TrinityBlueprintModuleView, TrinityBlueprintView, TrinityMasterPromptView,
    TrinityOrchestrateRequest, TrinityOrchestrateResponse, TrinityPillarsView,
    TrinityRepairDirectiveView, TrinityRepairRoundView, TrinitySlotsState, TrinitySubAgentDnaView,
};
use kian_hermes::trinity_pipeline::{
    AlphaIntakeRequest, MasterPromptSpec, ProjectBlueprint, SubagentStallWatchdog,
    ALPHA_READINESS_LOCK_THRESHOLD, LOGIC_EFFECTIVE_EQ, LOGIC_EFFECTIVE_IQ,
};
use kian_hermes::{
    build_preview_html, AlphaDiscoverySlots, AlphaIgnitionPersona, AlphaIntakeOutcome,
    BuilderEngine, CommandCenterOrchestrator, TrinityForgeOutcome, BUILDER_ESTIMATE_TOKENS,
    BUILDER_HISTORY_WINDOW, BUILDER_WELCOME_MESSAGE, MAIRON_AMBITION_GAIN,
};

use crate::error::{ApiFailure, ApiResult};
use crate::handlers::parse_body;
use crate::middleware::require_session;
use crate::state::AppState;

fn now_unix() -> i64 {
    chrono::Utc::now().timestamp()
}

fn message_view(m: &BuilderMessageRow) -> BuilderMessageView {
    BuilderMessageView {
        id: m.id,
        role: m.role.clone(),
        content: m.content.clone(),
        created_at: m.created_at,
    }
}

fn session_summary(s: &BuilderSessionRow) -> BuilderSessionSummary {
    BuilderSessionSummary {
        id: s.id.clone(),
        stage: s.stage.clone(),
        title: s.title.clone(),
        description: s.description.clone(),
        suggested_price: s.suggested_price,
        affiliate_bps: s.affiliate_bps,
        has_html: !s.generated_html.trim().is_empty(),
        published_product_id: s.published_product_id.clone(),
        created_at: s.created_at,
        updated_at: s.updated_at,
    }
}

fn session_view(s: &BuilderSessionRow, messages: &[BuilderMessageRow]) -> BuilderSessionView {
    let preview_html = build_preview_html(&s.generated_html, &s.title);
    BuilderSessionView {
        id: s.id.clone(),
        stage: s.stage.clone(),
        title: s.title.clone(),
        description: s.description.clone(),
        suggested_price: s.suggested_price,
        affiliate_bps: s.affiliate_bps,
        generated_html: s.generated_html.clone(),
        preview_html,
        published_product_id: s.published_product_id.clone(),
        messages: messages.iter().map(message_view).collect(),
        created_at: s.created_at,
        updated_at: s.updated_at,
    }
}

fn product_view(p: &ProductRow) -> ProductView {
    ProductView {
        id: p.id.clone(),
        seller_id: p.seller_id.clone(),
        title: p.title.clone(),
        description: p.description.clone(),
        price_din: crate::views::wire_din(p.price_din),
        category: p.category.clone(),
        preview_html: p.preview_html.clone(),
        affiliate_bps: p.affiliate_bps,
        artifact_kind: p.artifact_kind.clone(),
        active: p.active,
        created_at: p.created_at.clone(),
        mairon_score: p.mairon_score,
        mairon_badge: p.mairon_badge.clone(),
    }
}

/// `POST /builder/sessions` — create a new studio session along with the initial welcome message.
///
/// # Errors
/// 401 invalid session; 400 invalid body; 500 SQL error.
pub async fn create_session(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> ApiResult<(StatusCode, Json<BuilderSessionView>)> {
    let user = require_session(&state, &headers).await?;
    let req: CreateBuilderSessionRequest = parse_body(&body)?;
    let initial_title = req
        .initial_idea
        .as_deref()
        .map_or_default(|s| s.trim().chars().take(120).collect::<String>());

    let now = now_unix();
    let session = BuilderSessionRow {
        id: format!("bldr:{}", uuid::Uuid::now_v7()),
        account_id: user.account_id.clone(),
        stage: "discovery".to_owned(),
        title: initial_title,
        description: String::new(),
        suggested_price: 100,
        affiliate_bps: DEFAULT_AFFILIATE_BPS,
        generated_html: String::new(),
        published_product_id: None,
        created_at: now,
        updated_at: now,
    };
    state
        .store
        .create_builder_session(&session)
        .await
        .map_err(ApiFailure::from)?;
    let msg_id = state
        .store
        .append_builder_message(&session.id, "assistant", BUILDER_WELCOME_MESSAGE, now)
        .await
        .map_err(ApiFailure::from)?;
    let welcome = BuilderMessageRow {
        id: msg_id,
        session_id: session.id.clone(),
        role: "assistant".to_owned(),
        content: BUILDER_WELCOME_MESSAGE.to_owned(),
        created_at: now,
    };
    Ok((StatusCode::CREATED, Json(session_view(&session, &[welcome]))))
}

/// `GET /builder/sessions` — list the user's builder sessions (newest first).
///
/// # Errors
/// 401 invalid session; 500 SQL error.
pub async fn list_sessions(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> ApiResult<Json<BuilderSessionListResponse>> {
    let user = require_session(&state, &headers).await?;
    let rows = state
        .store
        .list_builder_sessions_for_account(&user.account_id, 50)
        .await
        .map_err(ApiFailure::from)?;
    let items = rows.iter().map(session_summary).collect();
    Ok(Json(BuilderSessionListResponse { items }))
}

/// `GET /builder/sessions/{id}` — full details of a session owned by the user + message history.
///
/// # Errors
/// 401 invalid session; 404 unknown session or owned by another user; 500 SQL error.
pub async fn get_session(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> ApiResult<Json<BuilderSessionView>> {
    let user = require_session(&state, &headers).await?;
    let session = state
        .store
        .get_builder_session(&id, &user.account_id)
        .await
        .map_err(ApiFailure::from)?
        .ok_or_else(|| ApiFailure::Http(HttpFailure::builder_session_not_found(&id)))?;
    let messages = state
        .store
        .list_builder_messages(&session.id, 50)
        .await
        .map_err(ApiFailure::from)?;
    Ok(Json(session_view(&session, &messages)))
}

/// `POST /builder/sessions/{id}/chat` — one turn of the multi-step conversation with the AI studio.
///
/// The user message is recorded, the budget gate is checked (when an external LLM is enabled),
/// the Builder engine produces the output (or the internal fallback interactive generator) and the session is updated.
///
/// # Errors
/// 401 session; 400 empty/too-long message; 404 unknown session; 429 daily AI budget exhausted; 500 SQL error.
#[allow(clippy::too_many_lines)]
pub async fn chat(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    body: axum::body::Bytes,
) -> ApiResult<Json<BuilderChatResponse>> {
    let user = require_session(&state, &headers).await?;
    let req: BuilderChatRequest = parse_body(&body)?;
    let user_msg = req.message.trim().to_owned();
    let msg_chars = user_msg.chars().count();
    if !(1..=4_000).contains(&msg_chars) {
        return Err(HttpFailure::invalid_request(1).into());
    }

    let session = state
        .store
        .get_builder_session(&id, &user.account_id)
        .await
        .map_err(ApiFailure::from)?
        .ok_or_else(|| ApiFailure::Http(HttpFailure::builder_session_not_found(&id)))?;

    // the daily budget check applies only when the external LLM slot is active
    if state.ai.enabled() {
        let Some(budget) = BudgetEnvelope::new(state.config.ai.daily_cap_tokens) else {
            return Err(HttpFailure::ai_disabled().into());
        };
        let (used_today, _calls) = state.store.ai_tokens_today(&user.id).await?;
        if let BudgetDecision::Exhausted { used, cap } =
            budget.precheck(used_today, BUILDER_ESTIMATE_TOKENS)
        {
            return Err(HttpFailure::ai_cap_exceeded(used, cap).into());
        }
    }

    let now = now_unix();
    state
        .store
        .append_builder_message(&session.id, "user", &user_msg, now)
        .await
        .map_err(ApiFailure::from)?;

    let history = state
        .store
        .list_builder_messages(&session.id, BUILDER_HISTORY_WINDOW)
        .await
        .map_err(ApiFailure::from)?;

    let force_generate = req.force_generate.unwrap_or(false);
    let outcome =
        BuilderEngine::run_turn(&state.ai, &session, &history, &user_msg, force_generate).await;

    let reply_ts = now_unix();
    state
        .store
        .append_builder_message(&session.id, "assistant", &outcome.reply, reply_ts)
        .await
        .map_err(ApiFailure::from)?;

    let patch = BuilderSessionPatch {
        stage: outcome.stage.clone(),
        title: outcome.title.clone(),
        description: outcome.description.clone(),
        suggested_price: outcome.suggested_price,
        affiliate_bps: outcome.affiliate_bps,
        generated_html: outcome.generated_html.clone(),
        published_product_id: session.published_product_id.clone(),
        updated_at: reply_ts,
    };
    state
        .store
        .update_builder_session_artifact(&session.id, &user.account_id, &patch)
        .await
        .map_err(ApiFailure::from)?;

    // record real token usage when the external LLM ran
    if !outcome.used_fallback && outcome.total_tokens > 0 {
        state
            .store
            .insert_ai_usage(&AiUsageRow {
                user_id: user.id.clone(),
                agent: "builder".to_owned(),
                provider: outcome.provider.clone().unwrap_or_else(|| "primary".to_owned()),
                model: outcome.model.clone().unwrap_or_default(),
                kind: "builder".to_owned(),
                prompt_tokens: outcome.prompt_tokens,
                completion_tokens: outcome.completion_tokens,
                total_tokens: outcome.total_tokens,
                outcome: "builder.ok".to_owned(),
            })
            .await
            .map_err(ApiFailure::from)?;
    }

    let updated_session = BuilderSessionRow {
        id: session.id.clone(),
        account_id: session.account_id,
        stage: patch.stage,
        title: patch.title,
        description: patch.description,
        suggested_price: patch.suggested_price,
        affiliate_bps: patch.affiliate_bps,
        generated_html: patch.generated_html,
        published_product_id: patch.published_product_id,
        created_at: session.created_at,
        updated_at: patch.updated_at,
    };
    let full_messages = state
        .store
        .list_builder_messages(&updated_session.id, 50)
        .await
        .map_err(ApiFailure::from)?;

    Ok(Json(BuilderChatResponse {
        reply: outcome.reply,
        session: session_view(&updated_session, &full_messages),
        used_fallback: outcome.used_fallback,
        provider: outcome.provider,
        total_tokens: outcome.total_tokens,
    }))
}

/// `POST /builder/sessions/{id}/publish` — one-click publish of a built product to the Kian market.
///
/// The user can accept the AI-proposed values or edit them in the body. If the session was
/// already published, the same product is returned idempotently along with `shareStartParam`.
///
/// # Errors
/// 401 session; 400 invalid input; 404 unknown session; 422 missing HTML code or `BPS_OUT_OF_RANGE`; 500 SQL error.
#[allow(clippy::too_many_lines)]
pub async fn publish(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    body: axum::body::Bytes,
) -> ApiResult<(StatusCode, Json<PublishBuilderResponse>)> {
    let user = require_session(&state, &headers).await?;
    let req: PublishBuilderRequest = parse_body(&body)?;

    let session = state
        .store
        .get_builder_session(&id, &user.account_id)
        .await
        .map_err(ApiFailure::from)?
        .ok_or_else(|| ApiFailure::Http(HttpFailure::builder_session_not_found(&id)))?;

    // if already published and the product exists in the Product table, return the same one (idempotent)
    if let Some(existing_pid) = &session.published_product_id
        && let Some(existing) = state
            .store
            .get_product(existing_pid)
            .await
            .map_err(ApiFailure::from)?
    {
        let share_start_param = format!("p_{}_r_{}", existing.id, user.telegram_id);
        return Ok((
            StatusCode::OK,
            Json(PublishBuilderResponse {
                product: product_view(&existing),
                session_id: session.id,
                share_start_param,
                already_published: true,
            }),
        ));
    }

    if session.generated_html.trim().is_empty() {
        return Err(ApiFailure::Http(HttpFailure::builder_artifact_empty()));
    }
    if session.generated_html.len() > MAX_DELIVERY_PAYLOAD_BYTES {
        return Err(HttpFailure::invalid_request(1).into());
    }

    let title = req
        .title
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map_or_else(
            || {
                if session.title.trim().is_empty() {
                    "\u{645}\u{62d}\u{635}\u{648}\u{644} \u{62a}\u{639}\u{627}\u{645}\u{644}\u{6cc} HTML".to_owned()
                } else {
                    session.title.trim().to_owned()
                }
            },
            str::to_owned,
        );
    let description = req
        .description
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map_or_else(
            || {
                if session.description.trim().is_empty() {
                    format!("\u{648}\u{628}‌\u{627}\u{67e}\u{644}\u{6cc}\u{6a9}\u{6cc}\u{634}\u{646} \u{62a}\u{639}\u{627}\u{645}\u{644}\u{6cc} \u{62a}\u{6a9}‌\u{635}\u{641}\u{62d}\u{647}‌\u{627}\u{6cc} {title} \u{628}\u{627} \u{62a}\u{62d}\u{648}\u{6cc}\u{644} \u{622}\u{646}\u{6cc} \u{62f}\u{631} \u{628}\u{627}\u{632}\u{627}\u{631} \u{6a9}\u{6cc}\u{627}\u{646}.")
                } else {
                    session.description.trim().to_owned()
                }
            },
            str::to_owned,
        );
    let price_din = req
        .price_din
        .map_or_else(|| session.suggested_price.max(1), kian_contracts::Din::get);
    let affiliate_bps = req.affiliate_bps.unwrap_or(session.affiliate_bps);

    if !(1..=120).contains(&title.chars().count())
        || !(1..=2_000).contains(&description.chars().count())
        || price_din < 1
        || req.category.as_ref().is_some_and(|c| c.chars().count() > 40)
    {
        return Err(HttpFailure::invalid_request(1).into());
    }
    if affiliate_bps > MAX_AFFILIATE_BPS {
        return Err(DomainError::new(
            DomainErrorCode::BpsOutOfRange,
            format!("affiliate_bps must be in [0, {MAX_AFFILIATE_BPS}], got {affiliate_bps}"),
        )
        .into());
    }

    let preview_html = build_preview_html(&session.generated_html, &title);
    let product = ProductRow {
        id: format!("prod:{}", uuid::Uuid::now_v7()),
        seller_id: user.account_id.clone(),
        title: title.clone(),
        description: description.clone(),
        category: req.category.or_else(|| Some("html-app".to_owned())),
        price_din,
        preview_html: preview_html.clone(),
        affiliate_bps,
        artifact_kind: "html".to_owned(),
        delivery_payload: Some(session.generated_html.clone()),
        active: true,
        created_at: now_iso(),
        mairon_score: 0,
        mairon_badge: String::new(),
        origin_locale: kian_hermes::arda_fishkal_engine::detect_origin_locale(&preview_html)
            .to_owned(),
        supported_locales_json: "[\"en\",\"fa\",\"ar\"]".to_owned(),
        skill_md: None,
        skill_json: None,
        arda_last_localized_at: None,
    };
    state
        .store
        .insert_product(&product)
        .await
        .map_err(ApiFailure::from)?;

    let patch = BuilderSessionPatch {
        stage: "published".to_owned(),
        title,
        description,
        suggested_price: price_din,
        affiliate_bps,
        generated_html: session.generated_html,
        published_product_id: Some(product.id.clone()),
        updated_at: now_unix(),
    };
    state
        .store
        .update_builder_session_artifact(&session.id, &user.account_id, &patch)
        .await
        .map_err(ApiFailure::from)?;

    let share_start_param = format!("p_{}_r_{}", product.id, user.telegram_id);
    Ok((
        StatusCode::CREATED,
        Json(PublishBuilderResponse {
            product: product_view(&product),
            session_id: session.id,
            share_start_param,
            already_published: false,
        }),
    ))
}


// ── Trinity Studio Pipeline (Wave 3) — Body 1 Alpha + Bodies 2/3 Command Room and Mairon ──

/// Convert slot state from the wire DTO to the engine — one-to-one mapping.
fn slots_from_wire(wire: &TrinitySlotsState) -> AlphaDiscoverySlots {
    AlphaDiscoverySlots {
        core_objective: wire.core_objective.clone(),
        target_audience_and_ux: wire.target_audience_and_ux.clone(),
        interactive_features: wire.interactive_features.clone(),
        data_and_zero_auth_apis: wire.data_and_zero_auth_apis.clone(),
        monetization_and_viral_loop: wire.monetization_and_viral_loop.clone(),
    }
}

/// Convert slot state from the engine to the wire DTO.
fn slots_to_wire(slots: &AlphaDiscoverySlots) -> TrinitySlotsState {
    TrinitySlotsState {
        core_objective: slots.core_objective.clone(),
        target_audience_and_ux: slots.target_audience_and_ux.clone(),
        interactive_features: slots.interactive_features.clone(),
        data_and_zero_auth_apis: slots.data_and_zero_auth_apis.clone(),
        monetization_and_viral_loop: slots.monetization_and_viral_loop.clone(),
    }
}

/// Map the engine's DNA manifest to the wire — the full `3×9` pattern.
fn manifest_to_wire(outcome: &TrinityForgeOutcome) -> Vec<TrinitySubAgentDnaView> {
    outcome
        .manifest
        .iter()
        .map(|agent| TrinitySubAgentDnaView {
            subagent_index: agent.subagent_index,
            specialty: agent.specialty.as_str(),
            specialty_fa: agent.specialty.fa_label(),
            assigned_modules: agent.assigned_modules.clone(),
            thread_id: agent.thread_id.clone(),
            logic_iq: LOGIC_EFFECTIVE_IQ,
            logic_eq: LOGIC_EFFECTIVE_EQ,
            watchdog_soft_secs: SubagentStallWatchdog::SOFT_STALL_SECS,
            watchdog_hard_secs: SubagentStallWatchdog::HARD_TIMEOUT_SECS,
            caveman: agent.genome.f03_token_economics.as_str().to_owned(),
            alleles: TrinityAlleleView {
                surgical_density: agent.genome.f04_stochastic_alleles.surgical_density,
                ux_craft: agent.genome.f04_stochastic_alleles.ux_craft,
                edge_case_paranoia: agent.genome.f04_stochastic_alleles.edge_case_paranoia,
                execution_velocity: agent.genome.f04_stochastic_alleles.execution_velocity,
                allele_seed: agent.genome.f04_stochastic_alleles.allele_seed,
            },
            layer1_chars: agent.prompt.layer1_stable_core.chars().count(),
            layer2_chars: agent.prompt.layer2_blueprint_context.chars().count(),
            layer3_chars: agent.prompt.layer3_dynamic_assignment.chars().count(),
            prompt_preview: agent.prompt.layer3_dynamic_assignment.chars().take(220).collect(),
            xml_prompt_chars: agent.genome.f06_compiled_xml_prompt.chars().count(),
            empirical_probes: agent.genome.f08_empirical_probes.clone(),
            handoff_card: agent.genome.f09_handoff_card.clone(),
            sanitizer_passed: agent.genome.f07_sanitizer_gate.passed,
        })
        .collect()
}

/// `POST /builder/trinity/alpha` — Body 1: Alpha (`IQ=136/EQ=122/Brake=0.04/Temp=0.88`).
///
/// Deterministically extract the five discovery slots from the user message; at readiness `>= 80` the master prompt
/// and project structure are locked and delivered. 200.
///
/// # Errors
/// 401 missing session; 400 invalid body.
#[allow(clippy::too_many_lines)]
pub async fn trinity_alpha(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> ApiResult<Json<TrinityAlphaResponse>> {
    let user = require_session(&state, &headers).await?;
    let req: TrinityAlphaRequest = parse_body(&body)?;
    let message_len = req.user_message.trim().chars().count();
    if !(1..=4_000).contains(&message_len) {
        return Err(kian_contracts::HttpFailure::invalid_request(1).into());
    }
    let session_id = match req.session_id.as_deref().map(str::trim) {
        Some(sid) if (4..=80).contains(&sid.chars().count()) => sid.to_owned(),
        _ => format!("trn:{}", uuid::Uuid::now_v7()),
    };
    let prior = req
        .prior_slots
        .as_ref()
        .map_or_else(AlphaDiscoverySlots::default, slots_from_wire);
    let outcome = AlphaIgnitionPersona.process_intake(&AlphaIntakeRequest {
        session_id: session_id.clone(),
        citizen_id: user.account_id.clone(),
        user_message: req.user_message.clone(),
        prior_slots: prior,
    });
    let mut updated = req
        .prior_slots
        .as_ref()
        .map_or_else(AlphaDiscoverySlots::default, slots_from_wire);
    let hay = req.user_message.to_lowercase();
    let spark = req.user_message.split_whitespace().collect::<Vec<_>>().join(" ");
    let spark = spark.chars().take(280).collect::<String>();
    for (kind, keywords) in AlphaIgnitionPersona::slot_keyword_table() {
        if !updated.is_filled(kind) && keywords.iter().any(|k| hay.contains(k)) {
            updated.fill(kind, spark.clone());
        }
    }
    let missing_slots = updated
        .missing_slots()
        .iter()
        .map(|k| k.as_str().to_owned())
        .collect::<Vec<_>>();
    let response = match outcome {
        AlphaIntakeOutcome::NeedsMoreDiscovery { alpha_reply_fa, readiness_pct, .. } => {
            TrinityAlphaResponse {
                status: "needs_more_discovery".to_owned(),
                session_id: session_id.clone(),
                citizen_id: user.account_id.clone(),
                alpha_reply_fa,
                missing_slots: missing_slots.clone(),
                readiness_pct,
                updated_slots: slots_to_wire(&updated),
                master_prompt: None,
                blueprint: None,
            }
        }
        AlphaIntakeOutcome::LockedForCommandCenter { alpha_handoff_fa, master_prompt, blueprint } => {
            TrinityAlphaResponse {
                status: "locked_for_command_center".to_owned(),
                session_id: master_prompt.session_id.clone(),
                citizen_id: user.account_id.clone(),
                alpha_reply_fa: alpha_handoff_fa,
                missing_slots: Vec::new(),
                readiness_pct: 100,
                updated_slots: slots_to_wire(&updated),
                master_prompt: Some(TrinityMasterPromptView {
                    spec_hash: master_prompt.spec_hash,
                    system_directive: master_prompt.system_directive,
                    functional_spec: master_prompt.functional_spec,
                    ux_390_constraints: master_prompt.ux_390_constraints,
                    verification_criteria: master_prompt.verification_criteria,
                    compiled_prompt: master_prompt.compiled_prompt,
                    project_name: master_prompt.project_name,
                    price_stars_hint: master_prompt.price_stars_hint,
                }),
                blueprint: Some(TrinityBlueprintView {
                    project_name: blueprint.project_name,
                    project_slug: blueprint.project_slug,
                    modules: blueprint
                        .modules
                        .iter()
                        .map(|m| TrinityBlueprintModuleView {
                            path: m.path.clone(),
                            role: m.role.clone(),
                            dependencies: m.dependencies.clone(),
                            specialty: m.specialty.as_str(),
                            specialty_fa: m.specialty.fa_label(),
                        })
                        .collect(),
                }),
            }
        }
    };
    Ok(Json(response))
}

/// `POST /builder/trinity/orchestrate` — Body 2 + Body 3: the command room
/// (`IQ=139/EQ=129/Strictness=0.96`) with the `OpenBot` subagent factory in the
/// `3×9` pattern and Mairon arbitration (`IQ=150/EQ=145`) with a self-heal loop up to the `>= 85` gate.
///
/// Input: Alpha's locked slots (readiness `>= 80` required). 200.
///
/// # Errors
/// 401 missing session; 400 invalid body; 422 readiness below 80 (the Alpha lock is required).
#[allow(clippy::too_many_lines)]
pub async fn trinity_orchestrate(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> ApiResult<Json<TrinityOrchestrateResponse>> {
    let user = require_session(&state, &headers).await?;
    let req: TrinityOrchestrateRequest = parse_body(&body)?;
    let session_id = req.session_id.trim().to_owned();
    if !(4..=80).contains(&session_id.chars().count()) {
        return Err(kian_contracts::HttpFailure::invalid_request(1).into());
    }
    let slots = slots_from_wire(&req.slots);
    let readiness = slots.readiness_pct();
    if readiness < ALPHA_READINESS_LOCK_THRESHOLD {
        return Err(DomainError::new(
            DomainErrorCode::InvalidTransition,
            format!(
                "Alpha lock requires readiness >= 80 — current {readiness}% (call /builder/trinity/alpha first)"
            ),
        )
        .into());
    }
    let seed = req
        .session_seed
        .unwrap_or_else(|| CommandCenterOrchestrator::session_seed_from_session(&session_id));
    let master = MasterPromptSpec::compile(&slots, &session_id, &user.account_id);
    let blueprint = ProjectBlueprint::compile(&slots, &master.project_name);
    let outcome = CommandCenterOrchestrator::new(&session_id, &user.account_id, seed)
        .execute_command_center_with_mairon_loop(&master, &blueprint);
    let verdict = &outcome.final_verdict.verdict;
    let advisory_view = TrinityAdvisoryView {
        iq: outcome.final_verdict.advisory.iq,
        eq: outcome.final_verdict.advisory.eq,
        lookahead_horizon: outcome.final_verdict.advisory.lookahead_horizon,
        tom_order: outcome.final_verdict.advisory.tom_order,
        ambition_gain: MAIRON_AMBITION_GAIN,
        eight_step_lookahead_summary: outcome
            .final_verdict
            .advisory
            .eight_step_lookahead_summary
            .clone(),
        third_order_tom_market_hook_fa: outcome
            .final_verdict
            .advisory
            .third_order_tom_market_hook_fa
            .clone(),
    };
    let repair_history = outcome
        .repair_history
        .iter()
        .map(|round| TrinityRepairRoundView {
            attempt: round.attempt,
            score_before: round.score_before,
            score_after: round.score_after,
            approved: round.approved,
            directives: round
                .directives
                .iter()
                .map(|d| TrinityRepairDirectiveView {
                    attempt: d.attempt,
                    target_specialty: d.target_specialty.as_str(),
                    pillar: d.pillar.clone(),
                    directive_en: d.directive_en.clone(),
                    directive_fa: d.directive_fa.clone(),
                })
                .collect(),
        })
        .collect();
    let pillars = verdict.pillars;
    let artifact_bytes = outcome.artifact.bytes();
    Ok(Json(TrinityOrchestrateResponse {
        manifest: manifest_to_wire(&outcome),
        session_id: outcome.session_id,
        project_name: outcome.project_name,
        artifact_html: outcome.artifact.html,
        artifact_bytes,
        probe_count: outcome.artifact.probe_count,
        integrated_modules: outcome.artifact.integrated_modules,
        final_score: verdict.total_score,
        approved_for_listing: verdict.approved_for_listing,
        mairon_badge: verdict.visible_badge.clone(),
        pillars: TrinityPillarsView {
            architecture: pillars.architecture,
            security_hardening: pillars.security_hardening,
            ux_and_anti_slop: pillars.ux_and_anti_slop,
            verification_proof: pillars.verification_proof,
        },
        advisory: advisory_view,
        repair_attempts_used: outcome.repair_attempts_used,
        repair_history,
    }))
}

#[cfg(test)]
mod builder_tests {
    use std::sync::Arc;

    use axum::body::Body;
    use http_body_util::BodyExt;
    use kian_ai::client::{ScriptedClient, UnavailableClient};
    use kian_ai::gateway::{AiConfig, CompletionPacket, SlotConfig};
    use kian_contracts::make_test_init_data;
    use kian_db::V3Store;
    use serde_json::{json, Value};
    use tower::ServiceExt;

    const BOT: &str = "123456:TEST-BOT-TOKEN-BUILDER";
    const ADMIN: &str = "test-admin-key-0123456789";

    fn base_cfg() -> crate::config::ServerConfig {
        let mut cfg = crate::config::ServerConfig::base();
        cfg.admin_key = Some(ADMIN.to_owned());
        cfg.bot_token = Some(BOT.to_owned());
        cfg
    }

    async fn app_without_llm() -> axum::Router {
        let store = V3Store::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        store.ensure_system_accounts().await.unwrap();
        let mut cfg = base_cfg();
        cfg.ai = AiConfig { slots: vec![], daily_cap_tokens: 10_000 };
        crate::routes::build_router(
            crate::state::AppState::with_gateway(store, cfg, Arc::new(UnavailableClient)).shared(),
        )
    }

    async fn app_with_scripted_llm(script: Vec<Result<CompletionPacket, String>>) -> axum::Router {
        let store = V3Store::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        store.ensure_system_accounts().await.unwrap();
        let mut cfg = base_cfg();
        cfg.ai = AiConfig {
            slots: vec![SlotConfig {
                name: "primary".into(),
                url: "http://mock-llm".into(),
                key: "k".into(),
                model: "studio-v1".into(),
            }],
            daily_cap_tokens: 10_000,
        };
        crate::routes::build_router(
            crate::state::AppState::with_gateway(store, cfg, Arc::new(ScriptedClient::new(script)))
                .shared(),
        )
    }

    async fn json_body(res: axum::response::Response) -> Value {
        let b = res.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&b).unwrap()
    }

    async fn auth(app: &axum::Router, tg_id: &str, username: &str) -> String {
        let init_data = make_test_init_data(BOT, tg_id, Some(username));
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/telegram")
                    .body(Body::from(json!({ "initData": init_data }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 201, "auth of {username} must succeed");
        json_body(res).await["token"].as_str().unwrap().to_owned()
    }

    async fn mint(app: &axum::Router, account: &str, amount: i64) {
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/wallet/mint")
                    .header("x-kian-key", ADMIN)
                    .body(Body::from(json!({ "accountId": account, "amount": amount }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200, "mint to {account} must succeed");
    }

    /// Full end-to-end flow without an external LLM key (with the internal fallback):
    /// create session → discovery → generate HTML → publish → purchase with referral → delivery → release
    #[tokio::test]
    #[allow(clippy::too_many_lines)]
    async fn full_builder_to_market_referral_purchase_flow_with_fallback() {
        let app = app_without_llm().await;
        let creator = auth(&app, "801", "maker").await;
        let buyer = auth(&app, "802", "shopper").await;
        let referrer = auth(&app, "803", "sharer").await;
        mint(&app, "citizen:802", 1_000).await;

        // 1) create the studio session with the welcome message
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/builder/sessions")
                    .header("authorization", format!("Bearer {creator}"))
                    .body(Body::from(b"{}".to_vec()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 201);
        let created = json_body(res).await;
        let session_id = created["id"].as_str().unwrap().to_owned();
        assert_eq!(created["stage"], "discovery");
        assert_eq!(created["messages"].as_array().unwrap().len(), 1);

        // ownership: another user cannot see the session → 404
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri(format!("/api/v1/builder/sessions/{session_id}"))
                    .header("authorization", format!("Bearer {buyer}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 404);

        // publish before HTML generation → 422 BUILDER_ARTIFACT_EMPTY
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/builder/sessions/{session_id}/publish"))
                    .header("authorization", format!("Bearer {creator}"))
                    .body(Body::from(b"{}".to_vec()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 422);
        assert_eq!(json_body(res).await["error"]["code"], "BUILDER_ARTIFACT_EMPTY");

        // 2) first chat turn (Discovery)
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/builder/sessions/{session_id}/chat"))
                    .header("authorization", format!("Bearer {creator}"))
                    .body(Body::from(json!({ "message": "\u{633}\u{644}\u{627}\u{645} \u{627}\u{6cc}\u{62f}\u{647} \u{62f}\u{627}\u{631}\u{645}" }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let turn1 = json_body(res).await;
        assert_eq!(turn1["usedFallback"], true);
        assert_eq!(turn1["session"]["stage"], "discovery");

        // 3) second chat turn (full HTML build)
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/builder/sessions/{session_id}/chat"))
                    .header("authorization", format!("Bearer {creator}"))
                    .body(Body::from(
                        json!({
                            "message": "\u{6cc}\u{6a9} \u{645}\u{627}\u{634}\u{6cc}\u{646} \u{62d}\u{633}\u{627}\u{628} \u{633}\u{648}\u{62f} \u{645}\u{631}\u{6a9}\u{628} \u{627}\u{631}\u{632} \u{62f}\u{6cc}\u{62c}\u{6cc}\u{62a}\u{627}\u{644} \u{628}\u{627} \u{62a}\u{645} \u{62a}\u{6cc}\u{631}\u{647} \u{628}\u{633}\u{627}\u{632}",
                            "forceGenerate": true
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let turn2 = json_body(res).await;
        assert_eq!(turn2["session"]["stage"], "ready");
        let generated_html = turn2["session"]["generatedHtml"].as_str().unwrap().to_owned();
        assert!(generated_html.contains("<!DOCTYPE html>"));
        assert!(turn2["session"]["previewHtml"]
            .as_str()
            .unwrap()
            .contains("Live preview on KIAN Market"));

        // 4) one-click publish to the market with a 20% commission and a 400 Đ price
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/builder/sessions/{session_id}/publish"))
                    .header("authorization", format!("Bearer {creator}"))
                    .body(Body::from(
                        json!({
                            "priceDin": 400,
                            "affiliateBps": 2_000
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 201);
        let published = json_body(res).await;
        let product_id = published["product"]["id"].as_str().unwrap().to_owned();
        assert_eq!(published["alreadyPublished"], false);
        assert_eq!(published["shareStartParam"], format!("p_{product_id}_r_801"));

        // republishing is idempotent and returns the same product
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/builder/sessions/{session_id}/publish"))
                    .header("authorization", format!("Bearer {creator}"))
                    .body(Body::from(b"{}".to_vec()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        assert_eq!(json_body(res).await["alreadyPublished"], true);

        // 5) purchase with citizen 803 as the referrer
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/market/purchase")
                    .header("authorization", format!("Bearer {buyer}"))
                    .body(Body::from(
                        json!({ "productId": product_id, "referrer": "803" }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 201);
        let escrow_id = json_body(res).await["escrow"]["id"].as_str().unwrap().to_owned();

        // 6) the buyer receives the full HTML code
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri(format!("/api/v1/market/escrows/{escrow_id}/delivery"))
                    .header("authorization", format!("Bearer {buyer}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        assert_eq!(json_body(res).await["deliveryPayload"], generated_html);

        // 7) release and settle 3 beneficiaries (400 → seller 280 + treasury 40 + referrer 80)
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/market/escrows/{escrow_id}/release"))
                    .header("authorization", format!("Bearer {buyer}"))
                    .body(Body::from(b"{}".to_vec()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);

        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/me")
                    .header("authorization", format!("Bearer {creator}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(json_body(res).await["balance"], 280);

        let res = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/me")
                    .header("authorization", format!("Bearer {referrer}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(json_body(res).await["balance"], 80);
    }

    /// Real execution with a scripted LLM slot: HTML + JSON CAST extraction and token usage recording.
    #[tokio::test]
    async fn builder_chat_with_scripted_llm_extracts_artifact_and_records_tokens() {
        let llm_reply = r#"Your product is ready!

```html
<!DOCTYPE html>
<html lang="fa" dir="rtl"><body><main><h1>Pomodoro Timer Tool</h1></main></body></html>
```

```json
{
  "title": "Pomodoro Focus Timer",
  "description": "Double your focus with a lightweight, offline single-page timer.",
  "suggested_price": 320,
  "affiliate_bps": 1500
}
```"#;
        let app = app_with_scripted_llm(vec![Ok(CompletionPacket {
            content: llm_reply.into(),
            prompt_tokens: 210,
            completion_tokens: 490,
        })])
        .await;
        let maker = auth(&app, "811", "llm_maker").await;

        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/builder/sessions")
                    .header("authorization", format!("Bearer {maker}"))
                    .body(Body::from(b"{}".to_vec()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let session_id = json_body(res).await["id"].as_str().unwrap().to_owned();

        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/builder/sessions/{session_id}/chat"))
                    .header("authorization", format!("Bearer {maker}"))
                    .body(Body::from(
                        json!({ "message": "\u{6cc}\u{6a9} \u{62a}\u{627}\u{6cc}\u{645}\u{631} \u{67e}\u{648}\u{645}\u{648}\u{62f}\u{648}\u{631}\u{648} \u{628}\u{633}\u{627}\u{632}", "forceGenerate": true })
                            .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let body = json_body(res).await;
        assert_eq!(body["usedFallback"], false);
        assert_eq!(body["provider"], "primary");
        assert_eq!(body["totalTokens"], 700);
        assert_eq!(body["session"]["stage"], "ready");
        assert_eq!(body["session"]["title"], "Pomodoro Focus Timer");
        assert_eq!(body["session"]["suggestedPrice"], 320);
        assert_eq!(body["session"]["affiliateBps"], 1_500);
        assert!(body["session"]["generatedHtml"]
            .as_str()
            .unwrap()
            .contains("Pomodoro Timer Tool"));

        // token usage is recorded in /ai/usage
        let res = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/ai/usage")
                    .header("authorization", format!("Bearer {maker}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(json_body(res).await["totalTokens"], 700);
    }
    // ── Trinity Studio Pipeline — live HTTP routes (Wave 3) ────────────────

    async fn trinity_post(app: &axum::Router, token: &str, uri: &str, body: Value) -> Value {
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(uri)
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 200, "trinity POST {uri} must succeed");
        json_body(res).await
    }

    /// First Alpha turn — discovery; then locked delivery at full readiness.
    #[tokio::test]
    async fn trinity_alpha_discovers_then_locks_master_prompt() {
        let app = app_without_llm().await;
        let token = auth(&app, "777", "trinity-maker").await;

        // turn 1 — a two-slot message → discovery continues with a sparky Persian reply
        let body = trinity_post(
            &app,
            &token,
            "/api/v1/builder/trinity/alpha",
            json!({
                "sessionId": "trn:e2e-alpha-1",
                "userMessage": "\u{645}\u{6cc}‌\u{62e}\u{648}\u{627}\u{647}\u{645} \u{6cc}\u{6a9} \u{645}\u{62d}\u{635}\u{648}\u{644} \u{628}\u{633}\u{627}\u{632}\u{645} \u{6a9}\u{647} \u{645}\u{62e}\u{627}\u{637}\u{628}\u{634} \u{641}\u{631}\u{6cc}\u{644}\u{646}\u{633}\u{631}\u{647}\u{627} \u{647}\u{633}\u{62a}\u{646}\u{62f} \u{648} \u{631}\u{627}\u{628}\u{637} \u{645}\u{648}\u{628}\u{627}\u{6cc}\u{644} rtl \u{62f}\u{627}\u{631}\u{62f}"
            }),
        ).await;
        assert_eq!(body["status"], "needs_more_discovery");
        assert!(body["alphaReplyFa"].as_str().unwrap().contains("\u{622}\u{644}\u{641}\u{627}"));
        assert_eq!(body["readinessPct"], 40);
        let missing = body["missingSlots"].as_array().unwrap();
        assert_eq!(missing.len(), 3);

        // turn 2 — previous slots + the full message → lock at 80%+
        let body = trinity_post(
            &app,
            &token,
            "/api/v1/builder/trinity/alpha",
            json!({
                "sessionId": "trn:e2e-alpha-1",
                "userMessage": "\u{641}\u{631}\u{645} \u{633}\u{641}\u{627}\u{631}\u{634} \u{62a}\u{639}\u{627}\u{645}\u{644}\u{6cc} \u{645}\u{6cc}‌\u{62e}\u{648}\u{627}\u{647}\u{645} \u{648} \u{62f}\u{627}\u{62f}\u{647}‌\u{647}\u{627} \u{62f}\u{631} api \u{639}\u{645}\u{648}\u{645}\u{6cc} \u{628}\u{62f}\u{648}\u{646} \u{644}\u{627}\u{6af}\u{6cc}\u{646} \u{630}\u{62e}\u{6cc}\u{631}\u{647} \u{645}\u{6cc}‌\u{634}\u{648}\u{646}\u{62f}",
                "priorSlots": body["updatedSlots"]
            }),
        ).await;
        assert_eq!(body["status"], "locked_for_command_center", "body: {body}");
        let master = &body["masterPrompt"];
        assert_eq!(master["specHash"].as_str().unwrap().len(), 64);
        assert!(master["compiledPrompt"].as_str().unwrap().contains("390px"));
        let blueprint = &body["blueprint"];
        let paths: Vec<&str> = blueprint["modules"]
            .as_array()
            .unwrap()
            .iter()
            .map(|m| m["path"].as_str().unwrap())
            .collect();
        assert!(paths.contains(&"src/index.html"));
        assert!(paths.contains(&"src/js/api-bindings.js"));
        assert_eq!(
            blueprint["modules"].as_array().unwrap()[0]["specialty"],
            "ui_ux_architect_390"
        );
    }

    /// Alpha without a session → 401 fail-closed.
    #[tokio::test]
    async fn trinity_alpha_requires_session() {
        let app = app_without_llm().await;
        let res = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/builder/trinity/alpha")
                    .body(Body::from(json!({"userMessage": "\u{633}\u{644}\u{627}\u{645}"}).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 401);
    }

    /// Full forging — the `3×9` command room + Mairon's self-heal loop up to the 85+ gate.
    #[tokio::test]
    async fn trinity_orchestrate_forges_through_mairon_gate() {
        let app = app_without_llm().await;
        let token = auth(&app, "778", "forge-captain").await;
        let body = trinity_post(
            &app,
            &token,
            "/api/v1/builder/trinity/orchestrate",
            json!({
                "sessionId": "trn:e2e-forge-1",
                "slots": {
                    "coreObjective": "\u{633}\u{627}\u{632}\u{646}\u{62f}\u{647}\u{654} \u{627}\u{628}\u{632}\u{627}\u{631} \u{628}\u{647}\u{631}\u{647}‌\u{648}\u{631}\u{6cc} \u{628}\u{631}\u{627}\u{6cc} \u{641}\u{631}\u{6cc}\u{644}\u{646}\u{633}\u{631}\u{647}\u{627}\u{6cc} \u{641}\u{627}\u{631}\u{633}\u{6cc}‌\u{632}\u{628}\u{627}\u{646}",
                    "targetAudienceAndUx": "\u{641}\u{631}\u{6cc}\u{644}\u{646}\u{633}\u{631} \u{641}\u{627}\u{631}\u{633}\u{6cc}‌\u{632}\u{628}\u{627}\u{646}\u{60c} \u{645}\u{648}\u{628}\u{627}\u{6cc}\u{644}‌\u{641}\u{631}\u{633}\u{62a} \u{6f3}\u{6f9}\u{6f0} \u{67e}\u{6cc}\u{6a9}\u{633}\u{644} rtl",
                    "interactiveFeatures": "\u{641}\u{631}\u{645} \u{633}\u{641}\u{627}\u{631}\u{634} \u{62a}\u{639}\u{627}\u{645}\u{644}\u{6cc} \u{648} \u{62c}\u{631}\u{6cc}\u{627}\u{646} \u{67e}\u{631}\u{62f}\u{627}\u{62e}\u{62a}",
                    "dataAndZeroAuthApis": "\u{630}\u{62e}\u{6cc}\u{631}\u{647} \u{62f}\u{631} localStorage \u{648} api \u{639}\u{645}\u{648}\u{645}\u{6cc} \u{628}\u{62f}\u{648}\u{646} \u{644}\u{627}\u{6af}\u{6cc}\u{646}",
                    "monetizationAndViralLoop": "\u{642}\u{6cc}\u{645}\u{62a} \u{6f2}\u{6f0}\u{6f0} \u{633}\u{62a}\u{627}\u{631}\u{647} \u{628}\u{627} \u{67e}\u{648}\u{631}\u{633}\u{627}\u{646}\u{62a} \u{645}\u{639}\u{631}\u{641}"
                }
            }),
        ).await;
        assert!(body["finalScore"].as_u64().unwrap() >= 85, "score: {body}");
        assert_eq!(body["approvedForListing"], true);
        assert!(body["maironBadge"].as_str().unwrap().contains("Forge-Grade"));
        assert!(body["artifactHtml"].as_str().unwrap().contains("data-probe"));
        assert!(body["integratedModules"].as_array().unwrap().len() >= 4);
        let manifest = body["manifest"].as_array().unwrap();
        assert!(manifest.len() >= 4, "one OpenBot sub-agent per module");
        let agent = &manifest[0];
        assert_eq!(agent["logicIq"], 139);
        assert_eq!(agent["logicEq"], 129);
        assert_eq!(agent["watchdogSoftSecs"], 450);
        assert_eq!(agent["watchdogHardSecs"], 1200);
        assert_eq!(agent["sanitizerPassed"], true);
        assert!(agent["promptPreview"].as_str().unwrap().contains("ASSIGNMENT (dynamic)"));
        assert!(agent["xmlPromptChars"].as_u64().unwrap() > 200);
        let advisory = &body["advisory"];
        assert_eq!(advisory["iq"], 150);
        assert_eq!(advisory["eq"], 145);
        assert_eq!(advisory["lookaheadHorizon"], 8);
        assert_eq!(advisory["tomOrder"], 3);
        assert!(advisory["thirdOrderTomMarketHookFa"]
            .as_str()
            .unwrap()
            .contains("200 \u{633}\u{62a}\u{627}\u{631}\u{647}\u{654} \u{62a}\u{644}\u{6af}\u{631}\u{627}\u{645} (XTR)"));
        // self-heal loop — the score improvement history is recorded
        let history = body["repairHistory"].as_array().unwrap();
        assert!(!history.is_empty(), "base assembly starts below the 85 gate");
        let first = &history[0];
        assert!(first["scoreAfter"].as_u64().unwrap() > first["scoreBefore"].as_u64().unwrap());
        assert_eq!(body["pillars"]["verificationProof"], 25);
    }

    /// Command room execution without the Alpha lock → 422 (the two-step contract is preserved).
    #[tokio::test]
    async fn trinity_orchestrate_rejects_sub_threshold_readiness() {
        let app = app_without_llm().await;
        let token = auth(&app, "779", "too-early").await;
        let res = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/builder/trinity/orchestrate")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::from(
                        json!({"sessionId": "trn:e2e-early", "slots": {}}).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 422);
        let body = json_body(res).await;
        assert!(body["error"]["message"]
            .as_str()
            .unwrap()
            .contains("readiness >= 80"));
    }

}
