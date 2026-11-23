//! تبدیل‌گرهای دامنه → وایر — تنها جایی که شکل DTOها ساخته می‌شود (زیرمجموعهٔ M2).
//!
//! قاعدهٔ فاز ۳ سر جایش است: فیلدهای `.nullable()` همیشه صریح روی سیم می‌مانند
//! (`null`)، فیلدهای `.optional()` غایب؛ وضعیت اسکرو `SCREAMING_SNAKE_CASE`.
//! هیچ هندلری مستقیم JSON نمی‌سازد.

use kian_contracts::{AccountView, Din, EscrowStatus, EscrowView};
use kian_db::AccountBalance;
use kian_domain::Escrow;

/// وضعیت امانت دامنه → وضعیت وایر (همان رشته‌ها).
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

/// i64 دامنه → `Din` وایر — دامنهٔ خالص بازهٔ `[0, 2^53−1]` را تضمین می‌کند؛
/// شاخهٔ دوم عملاً غیرقابل‌دسترس است و فقط برای کلن‌بودن تایپ لازم می‌شود.
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

/// نمای امانت وایر — فیلدهای nullable صریح.
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

/// نمای حساب وایر — تراز مشتق‌شده (ADR-003).
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
