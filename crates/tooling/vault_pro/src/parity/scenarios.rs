//! The golden parity scenario registry — full wire-level coverage (ADR-009 phase 8).
//!
//! Each scenario = setup (HTTP steps) + the recorded request + a database-state
//! expectation. Registry order is stable (golden files are named by `name`).
//! Amounts and IDs are all deterministic so SQL reads stay stable across runs.

use std::collections::BTreeMap;

use serde_json::{json, Value};

use super::BootCfg;

/// A scenario's HTTP step — an environment-independent request.
#[derive(Debug, Clone)]
pub struct Step {
    /// HTTP method
    pub method: &'static str,
    /// Full path under `/api/v1`
    pub path: String,
    /// Send the admin `X-KIA-Key`?
    pub admin: bool,
    /// Send the Telegram secret header (correctly)? — false means no header (401)
    pub bot_secret_header: bool,
    /// Optional JSON body
    pub body: Option<Value>,
}

impl Step {
    /// GET without a key.
    pub fn get(path: &str) -> Self {
        Self {
            method: "GET",
            path: path.into(),
            admin: false,
            bot_secret_header: false,
            body: None,
        }
    }

    /// GET with the admin key.
    pub fn get_admin(path: &str) -> Self {
        Self { method: "GET", path: path.into(), admin: true, bot_secret_header: false, body: None }
    }

    /// POST with the admin key and a body.
    pub fn post_admin(path: &str, body: Value) -> Self {
        Self {
            method: "POST",
            path: path.into(),
            admin: true,
            bot_secret_header: false,
            body: Some(body),
        }
    }

    /// POST without a key or body.
    pub fn post_open(path: &str) -> Self {
        Self {
            method: "POST",
            path: path.into(),
            admin: false,
            bot_secret_header: false,
            body: None,
        }
    }

    /// POST with the bot's correct secret header and an update body.
    pub fn post_bot(path: &str, body: Value) -> Self {
        Self {
            method: "POST",
            path: path.into(),
            admin: false,
            bot_secret_header: true,
            body: Some(body),
        }
    }

    /// Arbitrary method and path — for PUT/PATCH and a wrong secret header.
    pub fn raw(
        method: &'static str,
        path: &str,
        admin: bool,
        bot_secret_header: bool,
        body: Option<Value>,
    ) -> Self {
        Self { method, path: path.into(), admin, bot_secret_header, body }
    }
}

/// Expected database state after the scenario — only set fields are compared.
#[derive(Debug, Clone, Default)]
pub struct DbExpect {
    /// Σ user wallets
    pub user_wallet_sum: Option<i64>,
    /// Treasury balance
    pub treasury: Option<i64>,
    /// Tx row count
    pub tx_count: Option<i64>,
    /// Outbox row count
    pub outbox_count: Option<i64>,
    /// Escrow statuses by ID
    pub escrow_status: BTreeMap<String, String>,
}

impl DbExpect {
    /// Empty expectation — no field is compared.
    #[must_use]
    pub fn none() -> Self {
        Self::default()
    }

    /// Set Σ user wallets.
    #[must_use]
    pub fn sum(mut self, v: i64) -> Self {
        self.user_wallet_sum = Some(v);
        self
    }

    /// Set the treasury balance.
    #[must_use]
    pub fn treasury(mut self, v: i64) -> Self {
        self.treasury = Some(v);
        self
    }

    /// Set one escrow's status.
    #[must_use]
    pub fn escrow(mut self, id: &str, status: &str) -> Self {
        self.escrow_status.insert(id.into(), status.into());
        self
    }
}

/// One complete parity scenario.
#[derive(Debug, Clone)]
pub struct Scenario {
    /// Name — also the golden file name
    pub name: &'static str,
    /// Boot config
    pub boot: BootCfg,
    /// Setup steps (their responses are only checked for being 5xx)
    pub setup: Vec<Step>,
    /// The recorded request — its response is compared against the golden
    pub request: Step,
    /// Database expectation
    pub db: DbExpect,
}

/// Scenario constructor — readable like a test table.
struct ScenarioBuilder {
    name: &'static str,
    boot: BootCfg,
    setup: Vec<Step>,
}

impl ScenarioBuilder {
    fn new(name: &'static str) -> Self {
        Self { name, boot: BootCfg::default(), setup: Vec::new() }
    }

    fn bot(mut self) -> Self {
        self.boot.bot_secret = true;
        self
    }

    fn rate(mut self, limit: usize) -> Self {
        self.boot.rate_limit = limit;
        self
    }

    fn step(mut self, step: Step) -> Self {
        self.setup.push(step);
        self
    }

    fn steps(mut self, steps: Vec<Step>) -> Self {
        self.setup.extend(steps);
        self
    }

    fn req(self, request: Step, db: DbExpect) -> Scenario {
        Scenario { name: self.name, boot: self.boot, setup: self.setup, request, db }
    }
}

// ── Repeated setup steps ───────────────────────────────────────────────

/// Create a user account.
fn user(id: &str) -> Step {
    Step::post_admin("/accounts", json!({ "id": id, "label": "parity" }))
}

/// Mint into the user wallet.
fn mint(id: &str, amount: i64) -> Step {
    Step::post_admin("/wallet/mint", json!({ "accountId": id, "amount": amount }))
}

/// Create an escrow.
fn escrow(id: &str, buyer: &str, seller: &str, amount: i64) -> Step {
    Step::post_admin(
        "/escrows",
        json!({ "id": id, "buyerId": buyer, "sellerId": seller, "amount": amount }),
    )
}

/// Act on the escrow (fund/deliver/release/...).
fn act(id: &str, action: &str, body: Value) -> Step {
    Step::post_admin(&format!("/escrows/{id}/{action}"), body)
}

/// A delivered escrow ready for settlement — buyer and seller with 100,000 Đ wallets.
fn delivered(id: &str, amount: i64) -> Vec<Step> {
    vec![
        user("user:p1"),
        user("user:p2"),
        mint("user:p1", 100_000),
        mint("user:p2", 100_000),
        escrow(id, "user:p1", "user:p2", amount),
        act(id, "fund", json!({})),
        act(id, "deliver", json!({})),
    ]
}

/// A Telegram update — a text command from a private-chat user.
fn tg_update(update_id: i64, user_id: i64, text: &str) -> Value {
    json!({
        "update_id": update_id,
        "message": {
            "message_id": update_id + 1000,
            "from": { "id": user_id, "first_name": "Kia", "language_code": "fa" },
            "chat": { "id": user_id, "type": "private" },
            "text": text
        }
    })
}

/// The full scenario registry — stable order, unique names.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn scenarios() -> Vec<Scenario> {
    vec![
        // ── system / meta ────────────────────────────────────────────────
        ScenarioBuilder::new("health_ok")
            .req(Step::get("/health"), DbExpect::none()),
        ScenarioBuilder::new("economy_summary")
            .req(Step::get("/economy"), DbExpect::none()),
        ScenarioBuilder::new("openapi_document")
            .req(Step::get("/openapi.json"), DbExpect::none()),
        ScenarioBuilder::new("docs_page")
            .req(Step::get("/docs"), DbExpect::none()),
        ScenarioBuilder::new("unknown_route_envelope")
            .req(Step::get("/definitely/nope"), DbExpect::none()),
        // ── Accounts ──────────────────────────────────────────────────────
        ScenarioBuilder::new("account_created_201")
            .req(user("user:p1"), DbExpect::none().sum(0)),
        ScenarioBuilder::new("account_duplicate_rejected")
            .step(user("user:p1"))
            .req(user("user:p1"), DbExpect::none()),
        ScenarioBuilder::new("accounts_list")
            .step(user("user:p1"))
            .step(user("user:p2"))
            .req(Step::get_admin("/accounts"), DbExpect::none()),
        ScenarioBuilder::new("account_balance_missing_404")
            .req(Step::get_admin("/accounts/user:ghost/balance"), DbExpect::none()),
        // ── Wallets ──────────────────────────────────────────────────────
        ScenarioBuilder::new("mint_ok")
            .step(user("user:p1"))
            .req(mint("user:p1", 100_000), DbExpect::none().sum(100_000)),
        ScenarioBuilder::new("mint_negative_422")
            .step(user("user:p1"))
            .req(mint("user:p1", -5), DbExpect::none().sum(0)),
        ScenarioBuilder::new("mint_replay_idempotent")
            .step(user("user:p1"))
            .step(Step::post_admin(
                "/wallet/mint",
                json!({ "accountId": "user:p1", "amount": 100_000, "idempotencyKey": "parity-mint-key-1" }),
            ))
            .req(
                Step::post_admin(
                    "/wallet/mint",
                    json!({ "accountId": "user:p1", "amount": 100_000, "idempotencyKey": "parity-mint-key-1" }),
                ),
                DbExpect::none().sum(100_000),
            ),
        ScenarioBuilder::new("burn_overdraft_422")
            .step(user("user:p1"))
            .req(
                Step::post_admin("/wallet/burn", json!({ "accountId": "user:p1", "amount": 10 })),
                DbExpect::none().sum(0),
            ),
        // ── Escrows ─────────────────────────────────────────────────────
        ScenarioBuilder::new("escrow_created_201")
            .step(user("user:p1"))
            .step(user("user:p2"))
            .step(mint("user:p1", 100_000))
            .req(
                escrow("escrow:e1", "user:p1", "user:p2", 10_000),
                DbExpect::none().escrow("escrow:e1", "CREATED"),
            ),
        ScenarioBuilder::new("escrow_unknown_seller_422")
            .step(user("user:p1"))
            .req(
                escrow("escrow:e2", "user:p1", "user:ghost", 10_000),
                DbExpect::none(),
            ),
        ScenarioBuilder::new("escrow_release_settles_90_10")
            .steps(delivered("escrow:e3", 10_000))
            .req(
                act("escrow:e3", "release", json!({})),
                DbExpect::none().sum(199_000).treasury(1_000).escrow("escrow:e3", "RELEASED"),
            ),
        ScenarioBuilder::new("escrow_release_replay_no_double_pay")
            .steps(delivered("escrow:e4", 10_000))
            .step(act("escrow:e4", "release", json!({})))
            .req(
                act("escrow:e4", "release", json!({})),
                DbExpect::none().sum(199_000).treasury(1_000).escrow("escrow:e4", "RELEASED"),
            ),
        ScenarioBuilder::new("escrow_dispute_then_refund")
            .step(user("user:p1"))
            .step(user("user:p2"))
            .step(mint("user:p1", 100_000))
            .step(escrow("escrow:e5", "user:p1", "user:p2", 7_000))
            .step(act("escrow:e5", "fund", json!({})))
            .step(act("escrow:e5", "dispute", json!({ "reason": "wrong item" })))
            .req(
                act("escrow:e5", "refund", json!({})),
                DbExpect::none().sum(200_000).treasury(0).escrow("escrow:e5", "REFUNDED"),
            ),
        ScenarioBuilder::new("escrow_cancel_funded")
            .step(user("user:p1"))
            .step(user("user:p2"))
            .step(mint("user:p1", 100_000))
            .step(escrow("escrow:e6", "user:p1", "user:p2", 7_000))
            .step(act("escrow:e6", "fund", json!({})))
            .req(
                act("escrow:e6", "cancel", json!({})),
                DbExpect::none().sum(200_000).escrow("escrow:e6", "CANCELLED"),
            ),
        ScenarioBuilder::new("escrow_expire_refunds")
            .step(user("user:p1"))
            .step(user("user:p2"))
            .step(mint("user:p1", 100_000))
            .step(escrow("escrow:e7", "user:p1", "user:p2", 7_000))
            .step(act("escrow:e7", "fund", json!({})))
            .req(
                act("escrow:e7", "expire", json!({})),
                DbExpect::none().sum(200_000).escrow("escrow:e7", "EXPIRED"),
            ),
        // ── HITL — Article 7 ───────────────────────────────────────────────
        ScenarioBuilder::new("escrow_release_needs_hitl_over_threshold")
            .steps(delivered("escrow:h0", 60_000))
            .req(
                act("escrow:h0", "release", json!({})),
                DbExpect::none().sum(40_000).treasury(0).escrow("escrow:h0", "DELIVERED"),
            ),
        ScenarioBuilder::new("hitl_created_201")
            .steps(delivered("escrow:h1", 60_000))
            .req(
                Step::post_admin(
                    "/hitl",
                    json!({ "id": "hitl:pa1", "kind": "ESCROW_RELEASE", "escrowId": "escrow:h1", "requesterId": "user:p1", "note": "big deal" }),
                ),
                DbExpect::none().escrow("escrow:h1", "DELIVERED"),
            ),
        ScenarioBuilder::new("hitl_approve_settles_90_10")
            .steps(delivered("escrow:h2", 60_000))
            .step(Step::post_admin(
                "/hitl",
                json!({ "id": "hitl:pa2", "kind": "ESCROW_RELEASE", "escrowId": "escrow:h2", "requesterId": "user:p1" }),
            ))
            .req(
                Step::post_admin("/hitl/hitl:pa2/approve", json!({ "decidedBy": "admin:kia" })),
                DbExpect::none().sum(94_000).treasury(6_000).escrow("escrow:h2", "RELEASED"),
            ),
        ScenarioBuilder::new("hitl_second_decision_typed_error")
            .steps(delivered("escrow:h3", 60_000))
            .step(Step::post_admin(
                "/hitl",
                json!({ "id": "hitl:pa3", "kind": "ESCROW_RELEASE", "escrowId": "escrow:h3", "requesterId": "user:p1" }),
            ))
            .step(Step::post_admin("/hitl/hitl:pa3/approve", json!({ "decidedBy": "admin:kia" })))
            .req(
                Step::post_admin("/hitl/hitl:pa3/approve", json!({ "decidedBy": "admin:kia" })),
                DbExpect::none().sum(94_000).treasury(6_000).escrow("escrow:h3", "RELEASED"),
            ),
        ScenarioBuilder::new("hitl_reject_moves_no_money")
            .steps(delivered("escrow:h4", 60_000))
            .step(Step::post_admin(
                "/hitl",
                json!({ "id": "hitl:pa4", "kind": "ESCROW_RELEASE", "escrowId": "escrow:h4", "requesterId": "user:p1" }),
            ))
            .req(
                Step::post_admin(
                    "/hitl/hitl:pa4/reject",
                    json!({ "decidedBy": "admin:kia", "note": "refund" }),
                ),
                DbExpect::none().sum(40_000).treasury(0).escrow("escrow:h4", "DELIVERED"),
            ),
        ScenarioBuilder::new("hitl_sweep_empty_is_zero")
            .req(Step::post_admin("/hitl/sweep", json!({})), DbExpect::none()),
        // ── Bot ──────────────────────────────────────────────────────────
        ScenarioBuilder::new("bot_disabled_503")
            .req(
                Step::raw("POST", "/bot/webhook", false, false, Some(tg_update(1, 77, "/balance"))),
                DbExpect::none(),
            ),
        ScenarioBuilder::new("bot_bad_secret_401")
            .bot()
            .req(
                Step::raw("POST", "/bot/webhook", false, false, Some(tg_update(2, 77, "/balance"))),
                DbExpect::none(),
            ),
        ScenarioBuilder::new("bot_balance_command")
            .bot()
            .req(
                Step::post_bot("/bot/webhook", tg_update(3, 77, "/balance")),
                DbExpect::none(),
            ),
        // ── AI ───────────────────────────────────────────────────────────
        ScenarioBuilder::new("ai_chat_disabled_503")
            .step(user("user:p1"))
            .req(
                Step::post_admin("/ai/chat", json!({ "userId": "user:p1", "text": "\u{633}\u{644}\u{627}\u{645}" })),
                DbExpect::none(),
            ),
        ScenarioBuilder::new("ai_agents_list")
            .req(Step::get_admin("/ai/agents"), DbExpect::none()),
        ScenarioBuilder::new("ai_skills_list")
            .req(Step::get_admin("/ai/skills"), DbExpect::none()),
        ScenarioBuilder::new("ai_usage_empty")
            .req(Step::get_admin("/ai/usage"), DbExpect::none()),
        // ── Dashboard ──────────────────────────────────────────────────────
        ScenarioBuilder::new("dashboard_overview")
            .req(Step::get_admin("/overview"), DbExpect::none()),
        ScenarioBuilder::new("dashboard_product_created_listed")
            .step(Step::post_admin(
                "/products",
                json!({ "title": "Gold pack", "priceCredits": 500 }),
            ))
            .req(Step::get_admin("/products"), DbExpect::none()),
        ScenarioBuilder::new("dashboard_orders_empty")
            .req(Step::get_admin("/orders"), DbExpect::none()),
        ScenarioBuilder::new("dashboard_wallets_listed")
            .step(user("user:p1"))
            .step(mint("user:p1", 1_000))
            .req(Step::get_admin("/wallets"), DbExpect::none().sum(1_000)),
        ScenarioBuilder::new("dashboard_approval_decide_settles")
            .steps(delivered("escrow:h5", 60_000))
            .step(Step::post_admin(
                "/hitl",
                json!({ "id": "hitl:pa5", "kind": "ESCROW_RELEASE", "escrowId": "escrow:h5", "requesterId": "user:p1" }),
            ))
            .req(
                Step::post_admin(
                    "/approvals/hitl:pa5/decide",
                    json!({ "approve": true, "note": "ok" }),
                ),
                DbExpect::none().sum(94_000).treasury(6_000).escrow("escrow:h5", "RELEASED"),
            ),
        ScenarioBuilder::new("settings_put_is_not_implemented")
            .req(
                Step::raw(
                    "PUT",
                    "/settings",
                    true,
                    false,
                    Some(json!({ "key": "shop.notice", "value": "\u{628}\u{627}\u{632}\u{627}\u{631} \u{628}\u{627}\u{632} \u{627}\u{633}\u{62a}" })),
                ),
                DbExpect::none(),
            ),
        ScenarioBuilder::new("settings_list_empty")
            .req(Step::get_admin("/settings"), DbExpect::none()),
        ScenarioBuilder::new("security_layers")
            .req(Step::get_admin("/security"), DbExpect::none()),
        ScenarioBuilder::new("admin_guard_401")
            .req(Step::get("/overview"), DbExpect::none()),
        ScenarioBuilder::new("rate_limit_429_meta_limit")
            .rate(3)
            .step(Step::get("/health"))
            .step(Step::get("/health"))
            .step(Step::get("/health"))
            .req(Step::get("/health"), DbExpect::none()),
    ]
}
