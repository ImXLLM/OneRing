//! `kian-contracts` — قراردادهای وایر KIAN — ADR-005/ADR-009.
//!
//! منبع واحد حقیقتِ سیم: DTOهای serde با deriveهای utoipa، طبقه‌بندی خطای HTTP
//! و لایهٔ امنیت قراردادی (X-KIAN-Key با مقایسهٔ زمان‌ثابت + احراز initData
//! تلگرام برای M3). سازگاری وایر با TS با همان قواعد legacy (camelCase،
//! null صریح، صحیح تا 2^53−1) حفظ می‌شود.
//!
//! M2 — زیرمجموعهٔ کرنل پول: common/error/security/accounts/escrows/economy/system.
//! M3 — هویت تلگرام، نشست‌ها، بازار و واریز TON: auth/market.
#![forbid(unsafe_code)]

pub mod accounts;
pub mod auth;
pub mod brain;
pub mod builder;
pub mod chains;
pub mod common;
pub mod economy;
pub mod error;
pub mod escrows;
pub mod market;
pub mod nation;
pub mod security;
pub mod system;
pub mod wallet;

/// شناسه کرِیت — برای smoke test و لاگ‌ها.
pub const NAME: &str = "kian-contracts";

/// نسخه کرِیت — از `Cargo.toml` خوانده می‌شود.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub use accounts::{AccountListResponse, AccountView, BalanceResponse, CreateAccountRequest};
pub use auth::{
    AuthResponse, MeResponse, SetTonAddressRequest, TelegramAuthRequest, TonDepositCreateRequest,
    TonDepositListResponse, TonDepositView, UserView, WalletTxView,
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
    CreateProductRequest, EscrowDeliveryView, MAX_DELIVERY_PAYLOAD_BYTES, MAX_PREVIEW_HTML_BYTES,
    MyMarketResponse, ProductListResponse, ProductView, PurchaseRequest, PurchaseResponse,
};
pub use builder::{
    BuilderChatRequest, BuilderChatResponse, BuilderMessageView, BuilderSessionListResponse,
    BuilderSessionSummary, BuilderSessionView, CreateBuilderSessionRequest, PublishBuilderRequest,
    PublishBuilderResponse,
};
pub use nation::{
    BillKind, BillListResponse, BillView, CastVoteRequest, CreateBillRequest, EscrowResolution,
};
pub use security::{
    INIT_DATA_MAX_AGE_SECS, INIT_DATA_MAX_FUTURE_SECS, RATE_SHARD_COUNT, RateLimiter,
    ShardedLimiter, TelegramIdentity, build_bot_init_data, make_test_init_data, verify_kian_key,
    verify_telegram_init_data, verify_telegram_init_data_at,
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

    /// کلید ادمین — fail-closed با نام جدید X-KIAN-Key.
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

    /// مالیات ۱۰٪ منشور از مسیر دامنه — قرارداد وایر روی آن سوار است.
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
