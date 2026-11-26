//! Domain events — typed, deterministic IDs, no hidden clock.
//! `at` is always supplied by the caller (a handler command or the adapter clock).

/// Deterministic default time when the caller omits `at`.
/// The domain has no real clock — time is the adapter's responsibility.
pub const EPOCH_ISO: &str = "1970-01-01T00:00:00.000Z";

/// Escrow event type — the 8 allowed state-machine transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum EscrowEventKind {
    /// Escrow created
    Created,
    /// Funded (buyer → escrow)
    Funded,
    /// Goods/service delivered
    Delivered,
    /// Released in favor of the seller + tax
    Released,
    /// Dispute raised
    Disputed,
    /// Refunded to the buyer
    Refunded,
    /// Cancelled before funding
    Cancelled,
    /// Expired with automatic refund
    Expired,
}

impl EscrowEventKind {
    /// Canonical string — identical to the TS version (for event IDs and dual-run).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Created => "EscrowCreated",
            Self::Funded => "EscrowFunded",
            Self::Delivered => "EscrowDelivered",
            Self::Released => "EscrowReleased",
            Self::Disputed => "EscrowDisputed",
            Self::Refunded => "EscrowRefunded",
            Self::Cancelled => "EscrowCancelled",
            Self::Expired => "EscrowExpired",
        }
    }
}

impl std::fmt::Display for EscrowEventKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Escrow event — deterministic ID: `{escrowId}#{kind}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EscrowEvent {
    /// Deterministic event ID
    pub id: String,
    /// ISO time (supplied by the caller)
    pub at: String,
    /// Event kind
    pub kind: EscrowEventKind,
    /// Escrow ID
    pub escrow_id: String,
    /// Buyer (on Created)
    pub buyer_id: Option<String>,
    /// Seller (on Created)
    pub seller_id: Option<String>,
    /// Amount (Funded/Refunded/Expired/Created)
    pub amount: Option<i64>,
    /// Ledger transaction ID (Funded/Released/Refunded/Expired)
    pub tx_id: Option<String>,
    /// Seller net share (Released)
    pub seller_net: Option<i64>,
    /// Treasury tax (Released)
    pub tax: Option<i64>,
    /// Dispute reason (Disputed)
    pub reason: Option<String>,
    /// Optional correlation ID for tracing
    pub correlation_id: Option<String>,
}

impl EscrowEvent {
    /// Build the event with a deterministic ID — each transition happens at most once per escrow,
    /// so `{escrowId}#{kind}` is unique and replayable.
    #[must_use]
    pub fn new(kind: EscrowEventKind, escrow_id: &str, at: &str) -> Self {
        Self {
            id: format!("{}#{}", escrow_id, kind.as_str()),
            at: at.to_owned(),
            kind,
            escrow_id: escrow_id.to_owned(),
            buyer_id: None,
            seller_id: None,
            amount: None,
            tx_id: None,
            seller_net: None,
            tax: None,
            reason: None,
            correlation_id: None,
        }
    }
}

/// The ledger-transaction-posted event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionPostedEvent {
    /// Deterministic ID: `{txId}#posted`
    pub id: String,
    /// ISO time
    pub at: String,
    /// Transaction ID
    pub tx_id: String,
    /// Memo
    pub memo: Option<String>,
    /// Dedup key
    pub idempotency_key: Option<String>,
    /// Correlation ID
    pub correlation_id: Option<String>,
}

impl TransactionPostedEvent {
    /// Build the event with a deterministic ID.
    #[must_use]
    pub fn new(tx_id: &str, at: &str) -> Self {
        Self {
            id: format!("{tx_id}#posted"),
            at: at.to_owned(),
            tx_id: tx_id.to_owned(),
            memo: None,
            idempotency_key: None,
            correlation_id: None,
        }
    }
}

/// Every domain event — the same union as `DomainEvent` in TS.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainEvent {
    /// Ledger transaction posted
    TransactionPosted(TransactionPostedEvent),
    /// Escrow event
    Escrow(EscrowEvent),
}

impl DomainEvent {
    /// Deterministic event ID.
    #[must_use]
    pub fn id(&self) -> &str {
        match self {
            Self::TransactionPosted(e) => &e.id,
            Self::Escrow(e) => &e.id,
        }
    }

    /// Event time.
    #[must_use]
    pub const fn at(&self) -> &String {
        match self {
            Self::TransactionPosted(e) => &e.at,
            Self::Escrow(e) => &e.at,
        }
    }
}
