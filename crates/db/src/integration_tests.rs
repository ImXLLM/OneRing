//! Phase-2 integration tests — the full money journey on real SQLite + the legacy
//! migration with day-one numbers (466 users · Σ 187,672.8 Đ · 1,554 transactions · 30 products).

use crate::migrate::migrate_legacy;
use crate::store::{ApplyOptions, CommandOutcome, V3Store};
use crate::{genesis_proof, read_legacy};
use kian_domain::{
    create_escrow, deliver_escrow, fund_escrow, release_escrow, AccountKind, CreateEscrowCommand,
    DeliverEscrowCommand, Din, DomainErrorCode, DomainEvent, EscrowRefCommand, PostCommand,
    Posting, ReleaseEscrowCommand, TransactionPostedEvent, MINT_ACCOUNT_ID, TREASURY_ACCOUNT_ID,
};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    Row,
};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

/// Fixed deterministic time for tests.
const AT: &str = "2026-11-11T10:00:00.000Z";

fn temp_path(tag: &str) -> PathBuf {
    static N: AtomicU32 = AtomicU32::new(0);
    let n = N.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("kian-db-test-{}-{tag}-{n}.sqlite", std::process::id()))
}

async fn fresh_store() -> V3Store {
    let s = V3Store::connect_in_memory().await.unwrap();
    s.migrate().await.unwrap();
    s.ensure_system_accounts().await.unwrap();
    s
}

fn tx_posted_event(tx: &kian_domain::PostedTransaction) -> DomainEvent {
    let mut ev = TransactionPostedEvent::new(&tx.id, &tx.at);
    ev.memo = tx.memo.clone();
    ev.idempotency_key = tx.idempotency_key.clone();
    DomainEvent::TransactionPosted(ev)
}

async fn count(store: &V3Store, sql: &str) -> i64 {
    sqlx::query(crate::sqlsafe::audited(sql.to_owned())).fetch_one(store.pool_ref()).await.unwrap().get::<i64, _>("c")
}

// ── Base postings: mint, derived balance, outbox, and audit ──────────────────────────

#[tokio::test]
async fn mint_via_apply_derives_balance_and_writes_outbox_audit() {
    let store = fresh_store().await;
    store.create_account_if_missing("user:kia", AccountKind::User, Some("\u{6a9}\u{6cc}\u{627}")).await.unwrap();

    let opts = ApplyOptions {
        actor: "test".into(),
        action: "wallet.mint".into(),
        target: Some("user:kia".into()),
        escrow_id: None,
        idempotency_key: Some("mint:kia:1000".into()),
    };
    let res = store
        .apply(opts, |ledger, _| {
            let out = kian_domain::post(
                ledger,
                &PostCommand {
                    id: None,
                    idempotency_key: Some("mint:kia:1000".into()),
                    postings: vec![
                        Posting::new(MINT_ACCOUNT_ID, -1_000),
                        Posting::new("user:kia", 1_000),
                    ],
                    memo: Some("\u{62a}\u{648}\u{644}\u{6cc}\u{62f} \u{6f1}\u{6f0}\u{6f0}\u{6f0} \u{62f}\u{6cc}\u{646}\u{627}\u{631}".into()),
                    at: Some(AT.into()),
                },
            )?;
            Ok(CommandOutcome {
                ledger: out.state.clone(),
                tx: Some(out.tx.clone()),
                escrow: None,
                events: vec![tx_posted_event(&out.tx)],
            })
        })
        .await
        .unwrap();
    assert!(!res.replayed);
    assert_eq!(res.tx.as_ref().unwrap().postings.len(), 2);

    // Balances are derived — no balance column exists
    assert_eq!(store.balance_of("user:kia").await.unwrap(), Some(1_000));
    assert_eq!(store.balance_of(MINT_ACCOUNT_ID).await.unwrap(), Some(-1_000));
    let eco = store.economy_summary().await.unwrap();
    assert!(eco.balanced);
    assert_eq!(eco.trial_sum, 0);
    assert_eq!(eco.users_sum, 1_000);
    assert_eq!(eco.circulating_supply, 1_000);
    assert_eq!(eco.mint_balance, -1_000);
    assert_eq!(eco.minted_this_year, 1_000);

    // outbox + audit were written inside the same transaction
    let outbox = store.pending_outbox(10).await.unwrap();
    assert_eq!(outbox.len(), 1);
    assert_eq!(outbox[0].kind, "TransactionPosted");
    assert!(outbox[0].payload.contains("\"txId\""));
    assert_eq!(count(&store, r#"SELECT COUNT(*) AS c FROM "AuditLog""#).await, 1);
    let hist = store.recent_postings_for("user:kia", 5).await.unwrap();
    assert_eq!(hist.len(), 1);
    assert_eq!(hist[0].amount, 1_000);
    assert_eq!(hist[0].memo.as_deref(), Some("\u{62a}\u{648}\u{644}\u{6cc}\u{62f} \u{6f1}\u{6f0}\u{6f0}\u{6f0} \u{62f}\u{6cc}\u{646}\u{627}\u{631}"));
}

#[tokio::test]
async fn idempotent_replay_does_not_double_spend() {
    let store = fresh_store().await;
    store.create_account_if_missing("user:kia", AccountKind::User, None).await.unwrap();

    let opts = ApplyOptions {
        actor: "test".into(),
        action: "wallet.mint".into(),
        target: None,
        escrow_id: None,
        idempotency_key: Some("mint:kia:100".into()),
    };
    let run = |ledger: &kian_domain::LedgerState, _: &_| {
        let out = kian_domain::post(
            ledger,
            &PostCommand {
                id: None,
                idempotency_key: Some("mint:kia:100".into()),
                postings: vec![Posting::new(MINT_ACCOUNT_ID, -100), Posting::new("user:kia", 100)],
                memo: None,
                at: Some(AT.into()),
            },
        )?;
        Ok(CommandOutcome {
            ledger: out.state.clone(),
            tx: Some(out.tx.clone()),
            escrow: None,
            events: vec![tx_posted_event(&out.tx)],
        })
    };

    let first = store.apply(opts.clone(), run).await.unwrap();
    assert!(!first.replayed);
    let replay = store.apply(opts, run).await.unwrap();
    assert!(replay.replayed);
    assert_eq!(replay.tx.as_ref().unwrap().id, first.tx.as_ref().unwrap().id);

    // Nothing was doubled — no row, no event, no audit
    assert_eq!(store.balance_of("user:kia").await.unwrap(), Some(100));
    assert_eq!(count(&store, r#"SELECT COUNT(*) AS c FROM "Posting""#).await, 2);
    assert_eq!(count(&store, r#"SELECT COUNT(*) AS c FROM "Tx""#).await, 1);
    assert_eq!(store.pending_outbox(10).await.unwrap().len(), 1);
    assert_eq!(count(&store, r#"SELECT COUNT(*) AS c FROM "AuditLog""#).await, 1);
}

// ── Full escrow journey on the store ───────────────────────────────────────────────

#[tokio::test]
#[allow(clippy::too_many_lines)] // The full journey — in-place readability matters more than brevity
async fn escrow_full_journey_persists_everything() {
    let store = fresh_store().await;
    for (id, label) in [("user:buyer", Some("\u{62e}\u{631}\u{6cc}\u{62f}\u{627}\u{631}")), ("user:seller", Some("\u{641}\u{631}\u{648}\u{634}\u{646}\u{62f}\u{647}"))]
    {
        store.create_account_if_missing(id, AccountKind::User, label).await.unwrap();
    }
    // Buyer wallet: 1000 Đ
    store
        .apply(
            ApplyOptions {
                actor: "test".into(),
                action: "wallet.mint".into(),
                target: None,
                escrow_id: None,
                idempotency_key: Some("mint:buyer".into()),
            },
            |ledger, _| {
                let out = kian_domain::post(
                    ledger,
                    &PostCommand {
                        id: None,
                        idempotency_key: Some("mint:buyer".into()),
                        postings: vec![
                            Posting::new(MINT_ACCOUNT_ID, -1_000),
                            Posting::new("user:buyer", 1_000),
                        ],
                        memo: None,
                        at: Some(AT.into()),
                    },
                )?;
                Ok(CommandOutcome {
                    ledger: out.state.clone(),
                    tx: Some(out.tx.clone()),
                    escrow: None,
                    events: vec![tx_posted_event(&out.tx)],
                })
            },
        )
        .await
        .unwrap();

    let step = |action: &str, idem: Option<&str>, escrow_id: Option<&str>| ApplyOptions {
        actor: "bot".into(),
        action: action.into(),
        target: escrow_id.map(str::to_owned),
        escrow_id: escrow_id.map(str::to_owned),
        idempotency_key: idem.map(str::to_owned),
    };

    // create — no money movement
    let r = store
        .apply(step("escrow.create", Some("create:e1"), Some("e1")), |ledger, reg| {
            let o = create_escrow(
                ledger,
                reg,
                &CreateEscrowCommand {
                    id: "e1".into(),
                    buyer_id: "user:buyer".into(),
                    seller_id: "user:seller".into(),
                    amount: Din::new(400).unwrap(),
                    product_ref: Some("legacy-1".into()),
                    expires_at: None,
                    at: Some(AT.into()),
                },
            )?;
            Ok(CommandOutcome {
                ledger: o.ledger,
                tx: o.tx,
                escrow: Some(o.escrow),
                events: o.events.into_iter().map(DomainEvent::Escrow).collect(),
            })
        })
        .await
        .unwrap();
    assert!(!r.replayed);
    assert!(r.tx.is_none());

    // fund — buyer → escrow
    store
        .apply(step("escrow.fund", Some("fund:e1"), Some("e1")), |ledger, reg| {
            let o = fund_escrow(
                ledger,
                reg,
                &EscrowRefCommand {
                    escrow_id: "e1".into(),
                    idempotency_key: Some("fund:e1".into()),
                    at: Some(AT.into()),
                },
            )?;
            Ok(CommandOutcome {
                ledger: o.ledger,
                tx: o.tx,
                escrow: Some(o.escrow),
                events: o.events.into_iter().map(DomainEvent::Escrow).collect(),
            })
        })
        .await
        .unwrap();

    // deliver — no money
    store
        .apply(step("escrow.deliver", None, Some("e1")), |ledger, reg| {
            let o = deliver_escrow(
                ledger,
                reg,
                &DeliverEscrowCommand {
                    escrow_id: "e1".into(),
                    idempotency_key: None,
                    at: Some(AT.into()),
                    delivery_ref: Some("https://dl.example/1".into()),
                },
            )?;
            Ok(CommandOutcome {
                ledger: o.ledger,
                tx: o.tx,
                escrow: Some(o.escrow),
                events: o.events.into_iter().map(DomainEvent::Escrow).collect(),
            })
        })
        .await
        .unwrap();

    // release — escrow → seller + treasury tax
    store
        .apply(step("escrow.release", Some("settle:e1"), Some("e1")), |ledger, reg| {
            let o = release_escrow(
                ledger,
                reg,
                &ReleaseEscrowCommand {
                    escrow_id: "e1".into(),
                    idempotency_key: Some("settle:e1".into()),
                    at: Some(AT.into()),
                    approved_by_admin: false,
                },
            )?;
            Ok(CommandOutcome {
                ledger: o.ledger,
                tx: o.tx,
                escrow: Some(o.escrow),
                events: o.events.into_iter().map(DomainEvent::Escrow).collect(),
            })
        })
        .await
        .unwrap();

    // The final escrow state in the DB
    let escrow = store.get_escrow("e1").await.unwrap().unwrap();
    assert_eq!(escrow.status.as_str(), "RELEASED");
    assert_eq!(escrow.funded_tx_id.as_deref(), Some("tx_2"));
    assert_eq!(escrow.settle_tx_id.as_deref(), Some("tx_3"));
    assert_eq!(escrow.delivery_ref.as_deref(), Some("https://dl.example/1"));
    assert_eq!(store.list_escrows(None).await.unwrap().len(), 1);

    // Money: buyer 600 · seller 360 · treasury 40 · escrow zero
    assert_eq!(store.balance_of("user:buyer").await.unwrap(), Some(600));
    assert_eq!(store.balance_of("user:seller").await.unwrap(), Some(360));
    assert_eq!(store.balance_of(TREASURY_ACCOUNT_ID).await.unwrap(), Some(40));
    assert_eq!(store.balance_of("escrow:e1").await.unwrap(), Some(0));

    // outbox — the mint event + exactly 4 escrow events in production order
    let kinds: Vec<String> =
        store.pending_outbox(10).await.unwrap().into_iter().map(|r| r.kind).collect();
    assert_eq!(
        kinds,
        vec![
            "TransactionPosted",
            "EscrowCreated",
            "EscrowFunded",
            "EscrowDelivered",
            "EscrowReleased"
        ]
    );

    // audit — one per command (mint + 4 escrow transitions)
    assert_eq!(count(&store, r#"SELECT COUNT(*) AS c FROM "AuditLog""#).await, 5);
    let eco = store.economy_summary().await.unwrap();
    assert!(eco.balanced);
    assert_eq!(eco.escrow_locked, 0);
    assert_eq!(eco.treasury_balance, 40);
    assert_eq!(eco.open_escrows, 0);

    // outbox dispatch
    let pending = store.pending_outbox(10).await.unwrap();
    let first_id = pending[0].id.clone();
    let n = store.mark_outbox_published(&[first_id]).await.unwrap();
    assert_eq!(n, 1);
    assert_eq!(store.pending_outbox(10).await.unwrap().len(), 4);

    // Buyer wallet history — fund before mint (descending)
    let hist = store.recent_postings_for("user:buyer", 10).await.unwrap();
    assert_eq!(hist.len(), 2);
    assert_eq!(hist[0].amount, -400);
    assert_eq!(hist[1].amount, 1_000);
}

#[tokio::test]
async fn duplicate_escrow_is_domain_error_and_rolls_back() {
    let store = fresh_store().await;
    store.create_account_if_missing("user:buyer", AccountKind::User, None).await.unwrap();
    store.create_account_if_missing("user:seller", AccountKind::User, None).await.unwrap();

    let create = |ledger: &kian_domain::LedgerState,
                  reg: &kian_domain::EscrowRegistry| {
        create_escrow(
            ledger,
            reg,
            &CreateEscrowCommand {
                id: "e1".into(),
                buyer_id: "user:buyer".into(),
                seller_id: "user:seller".into(),
                amount: Din::new(400).unwrap(),
                product_ref: None,
                expires_at: None,
                at: Some(AT.into()),
            },
        )
        .map(|o| CommandOutcome {
            ledger: o.ledger,
            tx: o.tx,
            escrow: Some(o.escrow),
            events: o.events.into_iter().map(DomainEvent::Escrow).collect(),
        })
    };
    let opts = |idem: &str| ApplyOptions {
        actor: "bot".into(),
        action: "escrow.create".into(),
        target: Some("e1".into()),
        escrow_id: Some("e1".into()),
        idempotency_key: Some(idem.into()),
    };

    store.apply(opts("create:e1"), create).await.unwrap();
    let second = store.apply(opts("create:e1-TWO"), create).await; // A different key → the domain rejects
    let err = second.unwrap_err();
    assert!(
        matches!(err, crate::DbError::Domain(e) if e.code == DomainErrorCode::EscrowAlreadyExists)
    );

    // Full rollback — no partial rows remain
    assert_eq!(count(&store, r#"SELECT COUNT(*) AS c FROM "Escrow""#).await, 1);
    assert_eq!(count(&store, r#"SELECT COUNT(*) AS c FROM "OutboxEvent""#).await, 1);
    assert_eq!(count(&store, r#"SELECT COUNT(*) AS c FROM "AuditLog""#).await, 1);
}

// ── Legacy migration + external audit ───────────────────────────────────────────────

/// Build a synthetic legacy database with "exactly" the day-one numbers.
#[allow(clippy::too_many_lines)] // Synthetic fixture — insert order matters
async fn build_legacy_fixture(
    path: &std::path::Path,
    users_n: usize,
    products_n: usize,
    tx_rows: i64,
) {
    let opts = SqliteConnectOptions::new().filename(path).create_if_missing(true);
    let pool = SqlitePoolOptions::new().max_connections(1).connect_with(opts).await.unwrap();
    let mut tx = pool.begin().await.unwrap();
    sqlx::query(
        "CREATE TABLE users (user_id INTEGER PRIMARY KEY, username TEXT, first_name TEXT, credits REAL, is_banned INTEGER)",
    ).execute(&mut *tx).await.unwrap();
    sqlx::query(
        "CREATE TABLE products (id INTEGER PRIMARY KEY, creator_id INTEGER, title TEXT, description TEXT, price_credits REAL, category TEXT, link TEXT, is_active INTEGER)",
    ).execute(&mut *tx).await.unwrap();
    sqlx::query("CREATE TABLE transactions (id INTEGER PRIMARY KEY, note TEXT)")
        .execute(&mut *tx)
        .await
        .unwrap();
    // 465 × 403.0 + 1 × 277.8 = 187,672.8 → Σ floor = 187,672
    for i in 0..users_n {
        let credits = if i + 1 == users_n { 277.8 } else { 403.0 };
        sqlx::query("INSERT INTO users VALUES (?, ?, ?, ?, ?)")
            .bind(i64::try_from(i).map_or(501_070_000, |v| 501_070_000 + v))
            .bind(format!("user{i:03}"))
            .bind(format!("\u{634}\u{647}\u{631}\u{648}\u{646}\u{62f} {i}"))
            .bind(credits)
            .bind(i32::from(i % 100 == 0))
            .execute(&mut *tx)
            .await
            .unwrap();
    }
    for i in 0..products_n {
        let title = format!("\u{639}\u{646}\u{648}\u{627}\u{646} \u{645}\u{62d}\u{635}\u{648}\u{644} {i} — {}", "\u{632}".repeat(150)); // More than 120 characters
        sqlx::query("INSERT INTO products VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(i64::try_from(i).map_or(1, |v| v + 1))
            .bind(501_070_000i64 + i64::try_from(i % users_n).unwrap_or(0))
            .bind(title)
            .bind(format!("\u{62a}\u{648}\u{636}\u{6cc}\u{62d} \u{645}\u{62d}\u{635}\u{648}\u{644} {i}"))
            .bind(if i % 2 == 0 { 49.9 } else { 120.0 })
            .bind(format!("cat-{}", i % 5))
            .bind(if i % 3 == 0 { format!("https://dl.example/{i}") } else { String::new() })
            .bind(i64::from(i % 4 == 0))
            .execute(&mut *tx)
            .await
            .unwrap();
    }
    for i in 0..tx_rows {
        sqlx::query("INSERT INTO transactions (note) VALUES (?)")
            .bind(format!("legacy tx {i}"))
            .execute(&mut *tx)
            .await
            .unwrap();
    }
    tx.commit().await.unwrap();
    pool.close().await;
}

#[tokio::test]
#[allow(clippy::too_many_lines)] // Migration + audit + replay — the full cycle
async fn migrate_synthetic_legacy_then_proof_ok_and_replay_safe() {
    let legacy_path = temp_path("legacy");
    let target_path = temp_path("target");
    build_legacy_fixture(&legacy_path, 466, 30, 1_554).await;

    // Legacy-ledger read — read-only
    let snap = read_legacy(legacy_path.to_str().unwrap()).await.unwrap();
    assert_eq!(snap.users.len(), 466);
    assert_eq!(snap.products.len(), 30);
    assert_eq!(snap.tx_count, 1_554);
    let sum: f64 = snap.users.iter().map(|u| u.credits).sum();
    assert!((sum - 187_672.8).abs() < 1e-6);

    // Target store + versioned schema
    let store = V3Store::connect(target_path.to_str().unwrap()).await.unwrap();
    store.migrate().await.unwrap();

    // Dry run — nothing written
    let dry = migrate_legacy(
        &store,
        &snap.users,
        &snap.products,
        snap.tx_count,
        "marketplace-20260924-0400.db",
        true,
    )
    .await
    .unwrap();
    assert!(!dry.replayed && dry.dry_run);
    assert_eq!(count(&store, r#"SELECT COUNT(*) AS c FROM "Tx""#).await, 0);
    assert_eq!(count(&store, r#"SELECT COUNT(*) AS c FROM "Product""#).await, 0);

    // Real execution
    let report = migrate_legacy(
        &store,
        &snap.users,
        &snap.products,
        snap.tx_count,
        "marketplace-20260924-0400.db",
        false,
    )
    .await
    .unwrap();
    assert!(!report.replayed);
    for line in &report.lines {
        println!("{line}");
    }

    // Accounts: 466 users + 3 system = 469
    assert!(report.lines.iter().any(|l| l.contains("Σ user wallets        = 187672 Đ")));
    let accounts = store.list_accounts_with_balances().await.unwrap();
    assert_eq!(accounts.len(), 469);
    assert_eq!(accounts.iter().filter(|a| a.kind == "user").count(), 466);

    // genesis — one transaction with a deterministic key
    assert_eq!(count(&store, r#"SELECT COUNT(*) AS c FROM "Tx""#).await, 1);
    let eco = store.economy_summary().await.unwrap();
    assert_eq!(eco.users_sum, 187_672);
    assert_eq!(eco.mint_balance, -187_672);
    assert!(eco.balanced);

    // The banned account keeps its money — i=0 is banned and holds 403 Đ
    assert_eq!(store.balance_of("user:tg:501070000").await.unwrap(), Some(403));
    // Label from username
    let kia = accounts.iter().find(|a| a.id == "user:tg:501070000").unwrap();
    assert_eq!(kia.label.as_deref(), Some("user000"));

    // Products: truncated title, floored price, empty link → NULL
    let n_adjust =
        count(&store, r#"SELECT COUNT(*) AS c FROM "AuditLog" WHERE "action"='legacy.adjustment'"#)
            .await;
    assert_eq!(n_adjust, 1);
    let title: String = sqlx::query(r#"SELECT "title" FROM "Product" WHERE "id"='legacy-1'"#)
        .fetch_one(store.pool_ref())
        .await
        .unwrap()
        .get("title");
    assert!(title.chars().count() <= 120);
    let price: i64 = sqlx::query(r#"SELECT "priceDin" FROM "Product" WHERE "id"='legacy-1'"#)
        .fetch_one(store.pool_ref())
        .await
        .unwrap()
        .get("priceDin");
    assert_eq!(price, 49); // floor(49.9) — legacy-1 = product i=0 (even)
    let price1: i64 = sqlx::query(r#"SELECT "priceDin" FROM "Product" WHERE "id"='legacy-2'"#)
        .fetch_one(store.pool_ref())
        .await
        .unwrap()
        .get("priceDin");
    assert_eq!(price1, 120); // floor(120.0) — legacy-2 = product i=1 (odd)
    let delivery: Option<String> =
        sqlx::query(r#"SELECT "deliveryPayload" FROM "Product" WHERE "id"='legacy-2'"#)
            .fetch_one(store.pool_ref())
            .await
            .unwrap()
            .get("deliveryPayload");
    assert!(delivery.is_none()); // Empty link → NULL
    let active0: bool = sqlx::query(r#"SELECT "active" FROM "Product" WHERE "id"='legacy-1'"#)
        .fetch_one(store.pool_ref())
        .await
        .unwrap()
        .get("active");
    assert!(active0); // i=0 → 1

    // External audit — raw SQL, read-only, must be green
    let proof = genesis_proof(target_path.to_str().unwrap()).await.unwrap();
    assert!(proof.ok, "problems: {:?}", proof.problems);
    assert!(proof.lines.iter().any(|l| l.contains("Σ user wallets           : 187672 Đ")));
    assert!(proof.lines.iter().any(|l| l.contains("PROOF OK")));

    // Second run — replay-safe: no double mint, no double audit
    let again = migrate_legacy(
        &store,
        &snap.users,
        &snap.products,
        snap.tx_count,
        "marketplace-20260924-0400.db",
        false,
    )
    .await
    .unwrap();
    assert!(again.replayed);
    assert_eq!(count(&store, r#"SELECT COUNT(*) AS c FROM "Tx""#).await, 1);
    assert_eq!(
        count(&store, r#"SELECT COUNT(*) AS c FROM "AuditLog" WHERE "action"='legacy.adjustment'"#)
            .await,
        1
    );
    let eco2 = store.economy_summary().await.unwrap();
    assert_eq!(eco2.users_sum, 187_672);

    let _ = std::fs::remove_file(&legacy_path);
    let _ = std::fs::remove_file(&target_path);
}

#[tokio::test]
async fn proof_fails_on_unbalanced_corruption() {
    let target_path = temp_path("corrupt");
    let legacy_path = temp_path("legacy-corrupt");
    // The full 466-user fixture — before tampering, the 187,672 Đ expectation must hold
    build_legacy_fixture(&legacy_path, 466, 0, 0).await;
    let store = V3Store::connect(target_path.to_str().unwrap()).await.unwrap();
    store.migrate().await.unwrap();
    let snap = read_legacy(legacy_path.to_str().unwrap()).await.unwrap();
    migrate_legacy(&store, &snap.users, &snap.products, snap.tx_count, "fixture.db", false)
        .await
        .unwrap();
    let before = genesis_proof(target_path.to_str().unwrap()).await.unwrap();
    assert!(before.ok);

    // Deliberate tampering: an extra row that breaks double-entry sanctity
    sqlx::query(r#"INSERT INTO "Posting" ("txId", "accountId", "amount") VALUES ('tx_legacy_genesis', 'user:tg:501070000', 777)"#)
        .execute(store.pool_ref())
        .await
        .unwrap();
    let after = genesis_proof(target_path.to_str().unwrap()).await.unwrap();
    assert!(!after.ok);
    assert!(after.problems.iter().any(|p| p.contains("Σ user wallets")));
    assert!(after.lines.iter().any(|l| l.contains("PROOF FAILED")));

    let _ = std::fs::remove_file(&legacy_path);
    let _ = std::fs::remove_file(&target_path);
}

#[tokio::test]
async fn corrupt_account_kind_is_reported_not_panicked() {
    let store = fresh_store().await;
    sqlx::query(r#"INSERT INTO "Account" ("id", "kind", "label", "createdAt") VALUES ('x:1', 'hacker', NULL, ?)"#)
        .bind(crate::store::now_iso())
        .execute(store.pool_ref())
        .await
        .unwrap();
    let err = store.economy_summary().await.unwrap_err();
    assert!(matches!(err, crate::DbError::Corrupt(s) if s.contains("hacker")));
}

#[tokio::test]
async fn read_only_connection_refuses_writes() {
    let store = fresh_store().await;
    store.create_account_if_missing("user:x", AccountKind::User, None).await.unwrap();
    // A read-only store over the same file concept — with a temp path
    let path = temp_path("ro");
    let rw = V3Store::connect(path.to_str().unwrap()).await.unwrap();
    rw.migrate().await.unwrap();
    drop(rw);
    let ro = V3Store::connect_read_only(path.to_str().unwrap()).await.unwrap();
    let err = ro.create_account_if_missing("user:y", AccountKind::User, None).await;
    assert!(err.is_err());
    let _ = std::fs::remove_file(&path);
}

