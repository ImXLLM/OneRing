//! Domain → wire converters — the single place where DTO shapes are built (M2 subset).
//!
//! The phase 3 rule stands: `.nullable()` fields always stay explicit on the wire
//! (`null`), `.optional()` fields absent; escrow status in `SCREAMING_SNAKE_CASE`.
//! No handler builds JSON directly.

use kian_contracts::{AccountView, Din, EscrowStatus, EscrowView};
use kian_db::AccountBalance;
use kian_domain::Escrow;

/// Domain escrow status → wire status (same strings).
#[must_use]
pub fn escrow_status_wire(status: kian_domain::EscrowStatus) -> EscrowStatus {
    match status {
        kian_domain::EscrowStatus::Created => EscrowStatus::Created,
        kian_domain::EscrowStatus::Funded => EscrowStatus::Funded,
        kian_domain::EscrowStatus::Delivered => EscrowStatus::Delivered,
        kian_domain::EscrowStatus::Released => EscrowStatus::Released,
        kian_domain::EscrowStatus::Disputed => EscrowStatus::Disputed,
        kian_domain::EscrowStatus::Refunded => EscrowStatus::Refunded,
        kian_domain::EscrowStatus::Cancelled => EscrowStatus::Cancelled,
        kian_domain::EscrowStatus::Expired => EscrowStatus::Expired,
    }
}

/// Domain i64 → wire `Din` — the pure domain guarantees the `[0, 2^53−1]` range;
/// the second branch is effectively unreachable and only needed for type totality.
#[must_use]
pub fn wire_din(raw: i64) -> Din {
    match Din::new(raw) {
        Some(d) => d,
        None => match Din::new(0) {
            Some(zero) => zero,
            None => unreachable!("Din::new(0) is valid by definition"),
        },
    }
}

/// Wire escrow view — explicit nullable fields.
#[must_use]
pub fn escrow_view(e: &Escrow) -> EscrowView {
    EscrowView {
        id: e.id.clone(),
        buyer_id: e.buyer_id.clone(),
        seller_id: e.seller_id.clone(),
        amount: wire_din(e.amount.as_i64()),
        product_ref: e.product_ref.clone(),
        status: escrow_status_wire(e.status),
        created_at: e.created_at.clone(),
        funded_tx_id: e.funded_tx_id.clone(),
        settle_tx_id: e.settle_tx_id.clone(),
        dispute_reason: e.dispute_reason.clone(),
        delivery_ref: e.delivery_ref.clone(),
        expires_at: e.expires_at.clone(),
    }
}

/// Wire account view — derived balance (ADR-003).
#[must_use]
pub fn account_view(a: &AccountBalance) -> AccountView {
    AccountView {
        id: a.id.clone(),
        kind: a.kind.clone(),
        label: a.label.clone(),
        balance: a.balance,
        created_at: a.created_at.clone(),
    }
}
