//! هندلرهای انحصارِ حاکم — ژنومِ پنتامدلی، منشورِ روزِ اول و خزانهٔ Base
//! (Wave 7 — Step 4/4 + Commit #27 پنتا-مدل و انتخابِ دومُد).
//!
//! چهار مسیر additive:
//! - `GET /monopoly/genome-matrix` — ماتریسِ ۵مدل × ۹فایل با تأییدِ
//!   دیسکیِ fail-closed (۴۵ فایل) + قواعدِ مسیریابی + هر دو حالتِ انتخابِ
//!   دومُد — عمومی؛
//! - `POST /monopoly/route-model` — اجرایِ `ManualOverride` یا
//!   `AgentAutoDecision` روی ۵ مدلِ حاکم؛ خروجی: مدلِ انتخاب‌شده،
//!   پرامپتِ کلونِ تقطیرشده، اثباتِ تأییدِ ۹فایلی و rationale تصمیم؛
//! - `GET /monopoly/charter-and-base` — منشورِ چهارجریانیِ روزِ اول +
//!   پیکربندیِ Base Mainnet (`8453`) + خلاصهٔ ممیزیِ `ZeroXKiaToken.sol` و
//!   `KiaBaseEscrowTreasury.sol` — عمومی؛
//! - `POST /monopoly/simulate-split` — اجرایِ `compute_day_one_atomic_split`
//!   (فروشِ معرف‌دار / فروشِ مستقیمِ بی‌معرف با قانونِ `2000 bps` /
//!   بوستِ ۱۰۰٪)، ماندگاری در `sovereign_treasury_splits` و بازگشتِ
//!   اثباتِ صفرجمع + URI پرداختِ EIP-681.

use std::sync::Arc;

use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;

use kian_contracts::{
    MonopolyAuditPoint, MonopolyCharterAndBaseResponse, MonopolyCharterStreamView,
    MonopolyContractAuditView, MonopolyGenomeMatrixResponse, MonopolyGenomeModelView,
    MonopolyRouteModelRequest, MonopolyRouteModelResponse, MonopolyRoutingRuleView,
    MonopolySimulateSplitRequest, MonopolySimulateSplitResponse, MonopolySplitLegView,
};
use kian_db::now_iso;
use kian_hermes::sovereign_monopoly_engine::{
    QuadModelGenomeRouter, QuadTaskDomain, SovereignMonopolyOrchestrator, CANONICAL_GENOME_FILES,
    SOVEREIGN_GENOME_FILE_COUNT, SOVEREIGN_MONOPOLY_SIGNATURE,
};
use kian_wallet::base_treasury::{
    build_base_eip681_payment_uri, compute_day_one_atomic_split, compute_instant_boost_split,
    BoostTier, SettlementAsset, AFFILIATE_MAX_BPS, AFFILIATE_MIN_BPS, AFFILIATE_DEFAULT_BPS,
    BASE_BLOCK_EXPLORER, BASE_MAINNET_CHAIN_ID, BASE_MAINNET_RPC_URL, BASE_TREASURY_SIGNATURE,
    BASE_USDC_CONTRACT, CHARTER_TAX_BPS, KIA_SOVEREIGN_TREASURY_ACCOUNT, UNREFERRED_AFFILIATE_TO_TREASURY_BPS,
    WITHDRAWAL_SPREAD_BPS,
};

use crate::error::{ApiFailure, ApiResult};
use crate::handlers::parse_body;
use crate::middleware::require_session;
use crate::state::AppState;

/// نگاشتِ داراییِ وایر — شناسهٔ `wire_id` به enum؛ ناشناخته → خطای ۴۰۰.
fn parse_asset(wire_id: &str) -> Result<SettlementAsset, ApiFailure> {
    SettlementAsset::ALL
        .iter()
        .copied()
        .find(|a| a.wire_id() == wire_id.trim())
        .ok_or_else(|| ApiFailure::from(kian_contracts::HttpFailure::invalid_request(1)))
}

/// نگاشتِ لایهٔ بوستِ وایر.
fn parse_boost_tier(wire_id: &str) -> Result<BoostTier, ApiFailure> {
    match wire_id.trim() {
        "featured-slot-24h" => Ok(BoostTier::FeaturedSlot24h),
        "mairon-fast-track-audit" => Ok(BoostTier::MaironFastTrackAudit),
        "viral-swarm-amplifier" => Ok(BoostTier::ViralSwarmAmplifier),
        _ => Err(ApiFailure::from(kian_contracts::HttpFailure::invalid_request(1))),
    }
}

/// روترِ ژنومِ آمادهٔ هندلر — تأییدِ دیسکی با fallback به رجیستریِ builtin.
fn genome_router() -> QuadModelGenomeRouter {
    QuadModelGenomeRouter::production()
}

/// `GET /monopoly/genome-matrix` — ماتریسِ ژنومِ پنتامدلی (عمومی).
///
/// # Errors
/// رخ نمی‌دهد — خروجیِ قطعی از رجیستریِ تأییدشده.
#[allow(clippy::too_many_lines)]
pub async fn genome_matrix() -> Json<MonopolyGenomeMatrixResponse> {
    let router = genome_router();
    let models = router
        .verifications()
        .iter()
        .map(|v| MonopolyGenomeModelView {
            model_id: v.model.exact_id().to_owned(),
            aliases: v.model.aliases().iter().map(|s| (*s).to_owned()).collect(),
            genome_dir: format!("models/{}", v.genome_dir),
            specialty: v.model.specialty().to_owned(),
            canonical_files: CANONICAL_GENOME_FILES.iter().map(|s| (*s).to_owned()).collect(),
            files_verified: v.verified_count(),
            is_complete: v.is_complete(),
            trinity_avatar: kian_hermes::sovereign_monopoly_engine::TrinityAvatar::for_model(v.model)
                .wire_id()
                .to_owned(),
            verified_against: v
                .verified_against
                .clone()
                .unwrap_or_else(|| "builtin-registry".to_owned()),
        })
        .collect();
    let routing_rules = QuadTaskDomain::ALL
        .iter()
        .map(|d| MonopolyRoutingRuleView {
            domain: d.wire_id().to_owned(),
            champion_model_id: d.champion().exact_id().to_owned(),
            trinity_avatar: kian_hermes::sovereign_monopoly_engine::TrinityAvatar::for_model(
                d.champion(),
            )
            .wire_id()
            .to_owned(),
        })
        .collect();
    Json(MonopolyGenomeMatrixResponse {
        signature: SOVEREIGN_MONOPOLY_SIGNATURE.to_owned(),
        total_files_verified: router.total_files_verified(),
        expected_file_count: SOVEREIGN_GENOME_FILE_COUNT,
        // انتخابِ دومُد — هر دو قابلیت روی این ماتریس حاضرند (Commit #27).
        selection_modes: vec!["manual_override".to_owned(), "agent_auto_decision".to_owned()],
        models,
        routing_rules,
    })
}

/// `POST /monopoly/route-model` — انتخابِ دومُدِ مدل روی ۵ مدلِ حاکم
/// (نشستِ شهروند).
///
/// `manualModelOverride` غیرِ خالی و غیرِ `"auto"` → `ManualOverride`
/// (alias هایِ هر ۵ مدل حل می‌شوند؛ ناشناخته ۴۰۰)؛ در Agent Mode یا آوریِ
/// غایب/`"auto"` → `AgentAutoDecision` با تحلیلِ تلمتریِ `taskPrompt`.
/// خروجی: مدلِ انتخاب‌شده، پرامپتِ کلونِ تقطیرشده، اثباتِ تأییدِ ۹فایلی
/// و rationale تصمیم.
///
/// # Errors
/// ۴۰۱ نشستِ غایب؛ ۴۰۰ آوریِ دستیِ ناشناخته.
///
/// # Panics
/// تنها روی نقضِ ناورنِ داخلیِ رجیستری — مدلِ انتخاب‌شده همیشه در
/// رجیستریِ پنتا حاضر است؛ این `expect` روی دادهٔ DB نیست و برای هر ورودیِ
/// واقعی غیرِممکن است.
#[allow(clippy::too_many_lines)]
pub async fn route_model(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> ApiResult<Json<MonopolyRouteModelResponse>> {
    require_session(&state, &headers).await?;
    let req: MonopolyRouteModelRequest = parse_body(&body)?;
    let router = genome_router();
    let decision = router
        .select_model(&req.task_prompt, req.agent_mode, req.manual_model_override.as_deref())
        .map_err(|_| ApiFailure::from(kian_contracts::HttpFailure::invalid_request(1)))?;
    let verification = router
        .verifications()
        .iter()
        .find(|v| v.model == decision.selected_model)
        .expect("selected model must exist in the penta registry");
    Ok(Json(MonopolyRouteModelResponse {
        selected_model: decision.selected_model.exact_id().to_owned(),
        selection_mode: decision.selection_mode_wire.to_owned(),
        confidence_score: f64::from(decision.confidence_score),
        decision_rationale: decision.decision_rationale,
        routed_domain: decision.domain.wire_id().to_owned(),
        trinity_avatar: decision.prompt.trinity_avatar.wire_id().to_owned(),
        distilled_system_prompt: decision.prompt.system_prompt,
        genome_dir: format!("models/{}", verification.genome_dir),
        files_verified: verification.verified_count(),
        expected_files: CANONICAL_GENOME_FILES.len(),
    }))
}

/// بندِ ممیزی — کلید/مقدار.
fn audit_point(key: &str, value: impl Into<String>) -> MonopolyAuditPoint {
    MonopolyAuditPoint { key: key.to_owned(), value: value.into() }
}

/// descriptor ممیزیِ `ZeroXKiaToken.sol`.
#[allow(clippy::too_many_lines)]
fn token_audit_view() -> MonopolyContractAuditView {
    let a = kian_wallet::base_treasury::ZeroXKiaTokenAudit::canonical();
    MonopolyContractAuditView {
        contract_name: "ZeroXKiaToken".to_owned(),
        source_path: a.source_path,
        solidity_version: a.solidity_version,
        audit_points: vec![
            audit_point("chain_id", BASE_MAINNET_CHAIN_ID.to_string()),
            audit_point("total_supply", "1,000,000,000 * 10^18 (fixed, immutable)"),
            audit_point("post_constructor_mint", a.post_constructor_mint.to_string()),
            audit_point("blacklist", a.blacklist.to_string()),
            audit_point("pause", a.pause.to_string()),
            audit_point("mutable_tax_trap", a.mutable_tax_trap.to_string()),
            audit_point("eip2612_permit", a.eip2612_permit.to_string()),
            audit_point("user_burn", a.user_burn.to_string()),
            audit_point("goplus_tokensniffer_target", "100/100 clean"),
            audit_point("license", a.license),
        ],
    }
}

/// descriptor ممیزیِ `KiaBaseEscrowTreasury.sol`.
fn escrow_audit_view() -> MonopolyContractAuditView {
    let a = kian_wallet::base_treasury::KiaBaseEscrowTreasuryAudit::canonical();
    MonopolyContractAuditView {
        contract_name: "KiaBaseEscrowTreasury".to_owned(),
        source_path: a.source_path,
        solidity_version: a.solidity_version,
        audit_points: vec![
            audit_point("chain_id", BASE_MAINNET_CHAIN_ID.to_string()),
            audit_point("kia_treasury", a.kia_treasury),
            audit_point("allowed_assets", a.allowed_assets.join(", ")),
            audit_point("entrypoints", a.entrypoints.join(", ")),
            audit_point("non_reentrant_guard", a.non_reentrant_guard.to_string()),
            audit_point("charter_tax_bps", a.charter_tax_bps.to_string()),
            audit_point("affiliate_bps_window", a.affiliate_bps_window),
            audit_point("withdrawal_spread_bps", a.withdrawal_spread_bps.to_string()),
            audit_point("license", a.license),
        ],
    }
}

/// `GET /monopoly/charter-and-base` — منشور + Base + ممیزیِ قراردادها (عمومی).
///
/// # Errors
/// رخ نمی‌دهد — خروجیِ قطعی از descriptor هایِ کانونیک.
pub async fn charter_and_base() -> Json<MonopolyCharterAndBaseResponse> {
    Json(MonopolyCharterAndBaseResponse {
        signature: BASE_TREASURY_SIGNATURE.to_owned(),
        charter_motto: "جنگ اول به از صلح آخر".to_owned(),
        streams: vec![
            MonopolyCharterStreamView {
                stream: 1,
                name: "Mandatory Platform Charter Tax".to_owned(),
                bps: CHARTER_TAX_BPS,
                destination_account: KIA_SOVEREIGN_TREASURY_ACCOUNT.to_owned(),
                note: "Deducted on every marketplace sale, routed to the sovereign treasury."
                    .to_owned(),
            },
            MonopolyCharterStreamView {
                stream: 2,
                name: "Programmable Affiliate & Viral Split".to_owned(),
                bps: AFFILIATE_DEFAULT_BPS,
                destination_account: format!("user:{{referrer}} | {KIA_SOVEREIGN_TREASURY_ACCOUNT}"),
                note: format!(
                    "Creator-configurable {AFFILIATE_MIN_BPS}..={AFFILIATE_MAX_BPS} bps; unreferred \
                     direct sales route the default {UNREFERRED_AFFILIATE_TO_TREASURY_BPS} bps leg \
                     directly to the sovereign treasury (platform total 2000 bps / 20%, creator \
                     8000 bps / 80%)."
                ),
            },
            MonopolyCharterStreamView {
                stream: 3,
                name: "Instant Boosts, Featured Pinning & Mairon Fast-Track Audits".to_owned(),
                bps: 10_000,
                destination_account: KIA_SOVEREIGN_TREASURY_ACCOUNT.to_owned(),
                note: "100%-margin tiers: FeaturedSlot24h (250 XTR / $5.00 USDC / 5,000,000 µKIAN), \
                      MaironFastTrackAudit (100 XTR / $2.00 USDC / 2,000,000 µKIAN), \
                      ViralSwarmAmplifier (500 XTR / $10.00 USDC / 10,000,000 µKIAN)."
                    .to_owned(),
            },
            MonopolyCharterStreamView {
                stream: 4,
                name: "Instant Settlement & Cross-Chain Withdrawal Spread".to_owned(),
                bps: WITHDRAWAL_SPREAD_BPS,
                destination_account: KIA_SOVEREIGN_TREASURY_ACCOUNT.to_owned(),
                note: "Deducted on external cross-chain withdrawals, routed to the sovereign \
                       treasury."
                    .to_owned(),
            },
        ],
        base_chain_id: BASE_MAINNET_CHAIN_ID,
        base_rpc_url: BASE_MAINNET_RPC_URL.to_owned(),
        base_block_explorer: BASE_BLOCK_EXPLORER.to_owned(),
        base_usdc_contract: BASE_USDC_CONTRACT.to_owned(),
        token_audit: token_audit_view(),
        escrow_audit: escrow_audit_view(),
    })
}

/// `POST /monopoly/simulate-split` — تقسیمِ اتمیِ روزِ اول (نشستِ شهروند).
///
/// سناریوها: فروشِ معرف‌دار (`1000..=5000 bps` پایش)، فروشِ مستقیمِ بی‌معرف
/// (قانونِ `2000 bps` به خزانهٔ حاکم / `8000 bps` به سازنده) و بوستِ
/// لحظه‌ای (`10000 bps` به خزانه). خروجی ماندگار می‌شود و اثباتِ صفرجمع +
/// EIP-681 (دارایی‌های on-chain با `recipientAddress`) برمی‌گردد.
///
/// # Errors
/// ۴۰۱ نشستِ غایب؛ ۴۰۰ سناریو/دارایی/لایهٔ نامعتبر یا مبلغِ نامجاز.
#[allow(clippy::too_many_lines)]
pub async fn simulate_split(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> ApiResult<Json<MonopolySimulateSplitResponse>> {
    let user = require_session(&state, &headers).await?;
    let req: MonopolySimulateSplitRequest = parse_body(&body)?;

    let asset = parse_asset(&req.settlement_asset)?;
    let scenario = req.scenario.trim().to_ascii_lowercase();

    let split = match scenario.as_str() {
        "instant_boost" => {
            let tier = parse_boost_tier(req.boost_tier.as_deref().unwrap_or(""))?;
            compute_instant_boost_split(asset, tier, &user.id)
                .map_err(|_| ApiFailure::from(kian_contracts::HttpFailure::invalid_request(1)))?
        }
        "marketplace_sale" => {
            let referrer = req.referrer_user_id.as_deref().map(str::trim).filter(|s| !s.is_empty());
            compute_day_one_atomic_split(
                &kian_wallet::base_treasury::DayOneRevenueCharter::DAY_ONE,
                asset,
                i128::from(req.gross_amount_micros.max(1)),
                &user.id,
                req.creator_user_id.trim(),
                referrer,
                req.affiliate_bps,
            )
            .map_err(|_| ApiFailure::from(kian_contracts::HttpFailure::invalid_request(1)))?
        }
        _ => return Err(ApiFailure::from(kian_contracts::HttpFailure::invalid_request(1))),
    };

    // ماندگاری — شناسهٔ قطعی از uuid v7.
    let split_id = format!("split:{}", uuid::Uuid::now_v7());
    let row = kian_db::monopoly::SovereignTreasurySplitRow {
        id: split_id.clone(),
        buyer_user_id: user.id.clone(),
        creator_user_id: req.creator_user_id.trim().to_owned(),
        referrer_user_id: req
            .referrer_user_id
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_owned),
        settlement_asset: asset.wire_id().to_owned(),
        gross_amount_micros: split.gross_amount_micros.try_into().unwrap_or(i64::MAX),
        kia_treasury_micros: split.kia_treasury_micros.try_into().unwrap_or(i64::MAX),
        referrer_micros: split.referrer_micros.try_into().unwrap_or(0),
        creator_net_micros: split.creator_net_micros.try_into().unwrap_or(i64::MAX),
        effective_platform_bps: split.effective_platform_bps,
        zero_sum_verified: split.zero_sum_verified,
        created_at: now_iso(),
    };
    state.store.create_sovereign_split(&row).await?;

    // URI پرداختِ EIP-681 — فقط دارایی‌های on-chain با آدرسِ گیرنده.
    let eip681 = match (&req.recipient_address, asset.is_onchain_settleable()) {
        (Some(addr), true) if !addr.trim().is_empty() && addr.trim().starts_with("0x") => {
            build_base_eip681_payment_uri(asset, addr.trim(), split.gross_amount_micros).ok()
        }
        _ => None,
    };

    // اثباتِ صفرجمع — متنِ قابلِ ممیزی از جمعِ دقیقِ پاها.
    let leg_sum: i128 = split.legs.iter().map(|l| l.amount_micros).sum();
    let zero_sum_proof = format!(
        "double-entry: buyer_debit({}) + kia_treasury({}) + affiliate({}) + creator({}) = {} == 0 \
         [Precise-verified: {}] [db-row: {}]",
        split.legs[0].amount_micros,
        split.kia_treasury_micros,
        split.referrer_micros,
        split.creator_net_micros,
        leg_sum,
        split.verify_zero_sum_precise().unwrap_or(false),
        split_id,
    );

    let orch = SovereignMonopolyOrchestrator::canonical();
    Ok(Json(MonopolySimulateSplitResponse {
        split_id,
        scenario: split.scenario.to_owned(),
        settlement_asset: asset.wire_id().to_owned(),
        gross_amount_micros: split
            .gross_amount_micros
            .try_into()
            .unwrap_or(i64::MAX),
        kia_treasury_micros: split.kia_treasury_micros.try_into().unwrap_or(i64::MAX),
        referrer_micros: split.referrer_micros.try_into().unwrap_or(0),
        creator_net_micros: split.creator_net_micros.try_into().unwrap_or(i64::MAX),
        effective_platform_bps: split.effective_platform_bps,
        legs: split
            .legs
            .iter()
            .map(|l| MonopolySplitLegView {
                leg_kind: l.leg_kind.wire_id().to_owned(),
                account_id: l.account_id.clone(),
                amount_micros: l.amount_micros.try_into().unwrap_or(i64::MAX),
                bps: l.bps,
                note: l.note.clone(),
            })
            .collect(),
        zero_sum_proof,
        persisted: true,
        eip681_payment_uri: eip681,
        orchestrator_status: orch.status_summary(),
    }))
}
