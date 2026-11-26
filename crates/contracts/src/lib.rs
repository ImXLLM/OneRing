//! `kian-contracts` — the KIAN wire contracts — ADR-005/ADR-009.
//!
//! The single source of truth for the wire: serde DTOs with utoipa derives, HTTP error
//! classification, and the contractual security layer (X-KIAN-Key with constant-time comparison + initData
//! authentication of Telegram for M3). Wire compatibility with TS follows the same legacy rules (camelCase,
//! explicit null, integers up to 2^53−1).
//!
//! M2 — the money kernel subset: common/error/security/accounts/escrows/economy/system.
//! M3 — Telegram identity, sessions, market, and TON deposits: auth/market.
#![forbid(unsafe_code)]

pub mod accounts;
pub mod arda;
pub mod auth;
pub mod brain;
pub mod builder;
pub mod chains;
pub mod common;
pub mod economy;
pub mod error;
pub mod escrows;
pub mod grok_lab;
pub mod market;
pub mod monopoly;
pub mod nation;
pub mod quant;
pub mod security;
pub mod studio;
pub mod supremacy;
pub mod system;
pub mod trinity;
pub mod wave10;
pub mod wave9;
pub mod wallet;

/// Crate ID — for the smoke test and logs.
pub const NAME: &str = "kian-contracts";

/// Crate version — read from `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub use accounts::{AccountListResponse, AccountView, BalanceResponse, CreateAccountRequest};
pub use arda::{
    ArdaFlagView, ArdaFlagsCatalogResponse, ArdaLocaleParityView, ArdaLocalizeRequest,
    ArdaLocalizeResponse, ArdaParityReportView, ProductSkillPackageResponse,
};
pub use quant::{
    QuantGateView, QuantMetricsView, QuantStrategiesResponse, QuantStrategyView,
    QuantSwarmNodeView, QuantTickSelftestView, QuantVibeBacktestRequest,
    QuantVibeBacktestResponse, QuantVenueEndpointView, QuantVenueView, QuantVenuesResponse,
};
pub use auth::{
    AuthResponse, DevLoginRequest, FaucetRequest, MeResponse, SetTonAddressRequest,
    TelegramAuthRequest, TonDepositCreateRequest, TonDepositListResponse, TonDepositView, UserView,
    WalletTxView, FAUCET_DEFAULT_MICRO, FAUCET_MAX_MICRO,
};
pub use brain::{
    A2aInboxResponse, A2aMessageView, A2aSendRequest, AdminAuditEntry, AdminAuditListResponse,
    AdminDepositListResponse, AdminDepositView, AdminHitlListResponse, AdminHitlView,
    AdminOverviewResponse, AdminProductListResponse, AgentCardProvider, AgentCardResponse,
    AgentCardSkill, AgentView, AiChatRequest, AiChatResponse, AiMessage, AiSlotView,
    AiStatusResponse, AiUsageResponse, FleetResponse, HermesRunListResponse, HermesRunRequest,
    HermesRunResponse, HermesRunView, HermesSkillListResponse, HermesSkillView,
    HitlDecisionRequest, HitlDecisionResponse, McpInvokeRequest, McpInvokeResponse, McpToolView,
    ProductSalesView, RevenuePoint, SellerSalesView, ShieldView,
};
pub use chains::{
    ChainView, ChainsResponse, DepositAddressView, MyAddressesResponse, SetDepositAddressRequest,
    TreasuryChainView, TreasuryResponse, UsdtDepositCreateRequest, UsdtDepositListResponse,
    UsdtDepositView,
};
pub use common::{
    ApiError, DIN_MAX, Din, ErrorEnvelope, EscrowStatus, ID_PATTERN, Id, IdempotencyKey,
    IsoDateTime, Meta, MetaValue,
};
pub use economy::{Currency, DinSymbol, EconomySummary};
pub use error::{DomainFailure, HttpFailure, status_for_domain_code};
pub use escrows::{
    CancelEscrowRequest, CreateEscrowRequest, DeliverEscrowRequest, DisputeEscrowRequest,
    EscrowActionResponse, EscrowListResponse, EscrowView, ExpireEscrowRequest, FundEscrowRequest,
    RefundEscrowRequest, ReleaseEscrowRequest,
};
pub use market::{
    CreateProductRequest, EscrowDeliveryView, MaironConfirmResponse, MAX_DELIVERY_PAYLOAD_BYTES,
    MAX_PREVIEW_HTML_BYTES, MyMarketResponse, ProductListResponse, ProductRatingSummaryResponse,
    ProductReviewSubmitRequest, ProductReviewView, ProductView, PurchaseRequest, PurchaseResponse,
};
pub use monopoly::{
    MonopolyAuditPoint, MonopolyCharterAndBaseResponse, MonopolyCharterStreamView,
    MonopolyContractAuditView, MonopolyGenomeMatrixResponse, MonopolyGenomeModelView,
    MonopolyRouteModelRequest, MonopolyRouteModelResponse, MonopolyRoutingRuleView,
    MonopolySimulateSplitRequest, MonopolySimulateSplitResponse, MonopolySplitLegView,
};
pub use supremacy::{
    SupremacyCheckpointForkRequest, SupremacyCheckpointForkResponse, SupremacyMaironSealView,
    SupremacyMcpErrorView, SupremacyMcpRequest, SupremacyMcpResponse,
    SupremacySandboxBundleView, SupremacySweCheckView, SupremacySweVerifyRequest,
    SupremacySweVerifyResponse, SupremacyWorkflowExecuteRequest, SupremacyWorkflowExecuteResponse,
    SupremacyWorkflowNodeDto, SupremacyWorkflowNodeResultView, SupremacyWorkflowSpecDto,
    SupremacySurgicalPatchRequest, SupremacySurgicalPatchResponse,
};
pub use wave9::{
    AdminExportTrainingDatasetRequest, AdminExportTrainingDatasetResponse,
    AdminLabDashboardView, AdminLabTelemetryResponse, CohortHealthView,
    GrokChatTurnRequest, GrokChatTurnResponse, GrokCitationView, GrokThinkStepView,
    GrokUiBundleQuery, GrokUiBundleResponse, HudPresetView, HudPresetsResponse,
    HudSynthesizeRequest, HudSynthesizeResponse, I18nApplyPatchRequest,
    I18nApplyPatchResponse, I18nLocaleView, I18nMatrixResponse, I18nPatchEntryDto,
    LabTelemetryEntryView,
};
pub use builder::{
    BuilderChatRequest, BuilderChatResponse, BuilderMessageView, BuilderSessionListResponse,
    BuilderSessionSummary, BuilderSessionView, CreateBuilderSessionRequest, PublishBuilderRequest,
    PublishBuilderResponse,
};
pub use trinity::{
    TrinityAdvisoryView, TrinityAlleleView, TrinityAlphaRequest, TrinityAlphaResponse,
    TrinityBlueprintModuleView, TrinityBlueprintView, TrinityOrchestrateRequest,
    TrinityOrchestrateResponse, TrinityPillarsView, TrinityRepairDirectiveView,
    TrinityRepairRoundView, TrinitySlotsState, TrinitySubAgentDnaView, TrinityMasterPromptView,
};
pub use nation::{
    BillKind, BillListResponse, BillView, CastVoteRequest, CreateBillRequest, EscrowResolution,
};
pub use security::{
    INIT_DATA_MAX_AGE_SECS, INIT_DATA_MAX_FUTURE_SECS, RATE_SHARD_COUNT, RateLimiter,
    ShardedLimiter, TelegramIdentity, build_bot_init_data, make_test_init_data, verify_kian_key,
    verify_telegram_init_data, verify_telegram_init_data_at,
};
pub use studio::{
    CognitiveTierView, CommandPaletteView, DualBundleView, PortfolioCatalogResponse,
    PortfolioSafetyFixView, PortfolioSafetyReportView, StorefrontProjectCardView,
    StudioGenerateRequest, StudioGenerateResponse, StudioModalityView, StudioUxProfilesResponse,
};
pub use system::{BackupCreated, HealthResponse, HealthService, HealthStatus};
pub use wallet::{
    BurnRequest, ConfirmWithdrawalRequest, CreateWithdrawalRequest, MintRequest,
    WalletMoveResponse, WithdrawalListResponse, WithdrawalRequestView,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke() {
        assert_eq!(NAME, "kian-contracts");
        assert_ne!(VERSION, "", "version string present");
    }

    /// Admin key — fail-closed, renamed to X-KIAN-Key.
    #[test]
    fn kian_key_fail_closed() {
        const KEY: &str = "kian-admin-key-0123456789";
        assert!(!verify_kian_key(Some(KEY), None));
        assert!(!verify_kian_key(Some(KEY), Some("short")));
        assert!(!verify_kian_key(None, Some(KEY)));
        assert!(verify_kian_key(Some(KEY), Some(KEY)));
        assert!(!verify_kian_key(
            Some("kian-admin-key-0123456788"),
            Some(KEY)
        ));
    }

    /// The charter's 10% tax from the domain path — the wire contract is built on it.
    #[test]
    fn envelope_wire_is_camel_case_with_explicit_codes() {
        let f = HttpFailure::rate_limited(120);
        assert_eq!(f.status, 429);
        assert_eq!(f.code, "RATE_LIMITED");
        let env = f.to_envelope();
        assert_eq!(env.error.code, "RATE_LIMITED");
        assert!(env.error.meta.as_ref().unwrap().contains_key("limit"));
    }
}
