//! The Phase 8 parity engine — «same input → same response → same state».
//!
//! The golden rule of ADR-009 §6: contract fixed, implementation variable. This module runs
//! the parity suite:
//!
//! * Golden scenarios (HTTP input identical to the OpenAPI contract) run against the Rust router;
//!   the response is compared **byte-for-byte** (after normalizing volatile fields such as
//!   timestamp and uptime — which never match between two processes) against the golden,
//!   and the database state is checked verbatim as well.
//! * The «live comparison against TS» mode (`--ts-url`) in the `parity` binary — the same
//!   scenarios are fired at the TS server too and the byte diff is reported; this is the
//!   exit gate of each dual-run stage (ADR-009 §12).
//!
//! Golden files live in the crate's `tests/golden/`; `--record` regenerates them and
//! the replay test in CI blocks any byte drift.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use axum::Router;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tower::ServiceExt;

use dropagentx_db::V3Store;
use dropagentx_server::{build_router, AppState, ServerConfig};

/// Scenarios' admin key (>= 16 characters — the fail-closed rule).
pub const PARITY_KEY: &str = "parity-admin-key-0123456789";
/// Secret bot webhook token for the bot scenarios.
pub const PARITY_SECRET: &str = "parity-webhook-secret-0123456";

pub mod scenarios;

pub use scenarios::{scenarios, DbExpect, Scenario, Step};

/// One database state read after a scenario — holder of raw SQL values.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DbActual {
    /// Σ of user wallets (`Account.kind = 'user'`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_wallet_sum: Option<i64>,
    /// National treasury balance
    #[serde(skip_serializing_if = "Option::is_none")]
    pub treasury: Option<i64>,
    /// Tx row count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_count: Option<i64>,
    /// Outbox event count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outbox_count: Option<i64>,
    /// Escrow state — key `escrow:<id>`
    #[serde(flatten)]
    pub escrow_status: BTreeMap<String, String>,
}

/// One scenario's execution result — before comparison against the golden.
#[derive(Debug, Clone)]
pub struct ScenarioResult {
    /// Scenario name
    pub name: String,
    /// HTTP status code
    pub status: u16,
    /// `content-type` value
    pub content_type: String,
    /// Response body after normalization (exact bytes)
    pub body: Vec<u8>,
    /// Database read
    pub db: DbActual,
}

/// One scenario's golden — exactly what is stored/compared in the JSON file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Golden {
    /// Scenario name
    pub name: String,
    /// Expected status code
    pub status: u16,
    /// Expected `content-type`
    pub content_type: String,
    /// Expected normalized body (UTF-8 string)
    pub body: String,
    /// Expected database state
    pub db: DbActual,
}

/// Result of comparing one scenario against its golden.
#[derive(Debug, Clone)]
pub struct ScenarioCheck {
    /// Scenario name
    pub name: String,
    /// Full match (status + content-type + normalized body + database)
    pub ok: bool,
    /// List of found mismatches — empty means green
    pub diffs: Vec<String>,
}

/// Full parity-suite report — the input to the dual-run gate.
#[derive(Debug, Clone)]
pub struct ParityReport {
    /// Result per scenario, in registry order
    pub checks: Vec<ScenarioCheck>,
}

impl ParityReport {
    /// Are all scenarios green?
    #[must_use]
    pub fn ok(&self) -> bool {
        self.checks.iter().all(|c| c.ok)
    }

    /// Green scenarios / total.
    #[must_use]
    pub fn summary(&self) -> (usize, usize) {
        let green = self.checks.iter().filter(|c| c.ok).count();
        (green, self.checks.len())
    }

    /// Printable report text — for gate logging.
    #[must_use]
    pub fn render(&self) -> String {
        let (green, total) = self.summary();
        let mut out = vec![format!("parity: {green}/{total} green")];
        for c in &self.checks {
            if c.ok {
                out.push(format!("  ✅ {}", c.name));
            } else {
                out.push(format!("  ❌ {} — {}", c.name, c.diffs.join(" · ")));
            }
        }
        out.join("\n")
    }
}

/// RFC3339 pattern — every volatile timestamp on the wire is normalized to a fixed token.
/// The pattern is a compile-time constant; if it ever breaks, the
/// `normalizer_flattens_volatile_fields` test goes red in CI (fail-loud),
/// so no panic/expect is needed here.
fn ts_pattern() -> Option<&'static Regex> {
    static RE: std::sync::OnceLock<Option<Regex>> = std::sync::OnceLock::new();
    RE.get_or_init(|| {
        // matching example: `2026-11-12T09:41:00.123Z` or with a `+00:00` offset
        Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?(Z|[+-]\d{2}:?\d{2})$").ok()
    })
    .as_ref()
}

/// UUID pattern — server-generated ids (like `prod:<uuid>`) never match across engines;
/// they are normalized to a fixed token (the `prod:` prefix is preserved).
/// Same fail-loud policy as the timestamp pattern — the byte replay test is its gate.
fn uuid_pattern() -> Option<&'static Regex> {
    static RE: std::sync::OnceLock<Option<Regex>> = std::sync::OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"^([A-Za-z0-9]+:)?[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
        )
        .ok()
    })
    .as_ref()
}

/// JSON value normalization — recursive: timestamp → `<TS>`, uptime → `<UPTIME>`.
#[must_use]
pub fn normalize_value(v: &Value) -> Value {
    match v {
        Value::Object(map) => {
            let mut out = serde_json::Map::new();
            for (k, val) in map {
                if k == "uptime" {
                    out.insert(k.clone(), Value::String("<UPTIME>".into()));
                } else {
                    out.insert(k.clone(), normalize_value(val));
                }
            }
            Value::Object(out)
        }
        Value::Array(items) => Value::Array(items.iter().map(normalize_value).collect()),
        Value::String(s) if ts_pattern().is_some_and(|re| re.is_match(s)) => {
            Value::String("<TS>".into())
        }
        Value::String(s) if uuid_pattern().is_some_and(|re| re.is_match(s)) => {
            let prefix = s.split(':').next().map_or(String::new(), |p| format!("{p}:"));
            Value::String(format!("{prefix}<UUID>"))
        }
        other => other.clone(),
    }
}

/// Response-body normalization — JSON handled smartly, everything else by text substitution.
#[must_use]
pub fn normalize_body(bytes: &[u8]) -> Vec<u8> {
    match serde_json::from_slice::<Value>(bytes) {
        Ok(v) => serde_json::to_vec(&normalize_value(&v)).unwrap_or_default(),
        Err(_) => {
            // non-JSON body (such as the docs page) — text patterns over the full range;
            // the pattern is fixed and cannot fail, but no panic: a hypothetical failure =
            // passing the text through unchanged (the byte replay tests in CI catch that).
            let text = String::from_utf8_lossy(bytes).into_owned();
            match Regex::new(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?(Z|[+-]\d{2}:?\d{2})")
                .ok()
            {
                Some(re) => re.replace_all(&text, "<TS>").into_owned().into_bytes(),
                None => text.into_bytes(),
            }
        }
    }
}

/// Boot config for one scenario — minimal and environment-independent.
#[derive(Debug, Clone)]
pub struct BootCfg {
    /// Bot webhook enabled? (secret token + bot admin 42)
    pub bot_secret: bool,
    /// Rate cap (default 120 — the 429 scenario sets a small number)
    pub rate_limit: usize,
}

impl Default for BootCfg {
    fn default() -> Self {
        Self { bot_secret: false, rate_limit: 120 }
    }
}

/// Boot the full router on the given database — the same path as production `main.rs`.
///
/// # Errors
/// Database open/migration error (in a test this means the scenario itself failed).
pub async fn boot(cfg: &BootCfg, database_url: &str) -> anyhow::Result<(Router, V3Store)> {
    let server_cfg = ServerConfig {
        admin_key: Some(PARITY_KEY.into()),
        database_url: database_url.into(),
        static_dir: None,
        rate_limit: cfg.rate_limit,
        bot_webhook_secret: if cfg.bot_secret { Some(PARITY_SECRET.into()) } else { None },
        bot_admin_ids: if cfg.bot_secret { vec![42] } else { Vec::new() },
        ..ServerConfig::default()
    };
    let store = V3Store::connect(database_url).await?;
    store.migrate().await?;
    store.ensure_system_accounts().await?;
    let state = AppState::new(store.clone(), server_cfg).shared();
    Ok((build_router(state), store))
}

/// Dispatch one HTTP step — response (status, content-type, raw body).
///
/// # Errors
/// Network/body error (practically impossible with oneshot; a failure means a scenario bug).
pub async fn send_step(
    router: &Router,
    step: &Step,
) -> anyhow::Result<(StatusCode, String, Vec<u8>)> {
    // scenario paths are relative to /api/v1 — the same habit as Fastify's app.inject
    let full_path = format!("/api/v1{}", step.path);
    let mut builder = Request::builder().method(step.method).uri(full_path.as_str());
    if step.admin {
        builder = builder.header("x-kia-key", PARITY_KEY);
    }
    if step.bot_secret_header {
        builder = builder.header("x-telegram-bot-api-secret-token", PARITY_SECRET);
    }
    if step.body.is_some() {
        builder = builder.header(header::CONTENT_TYPE, "application/json");
    }
    let payload = step.body.as_ref().map_or_else(String::new, ToString::to_string);
    let req = builder.body(Body::from(payload))?;
    let res = router.clone().oneshot(req).await?;
    let status = res.status();
    let ctype = res
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_owned();
    let bytes = axum::body::to_bytes(res.into_body(), 4 << 20).await?;
    Ok((status, ctype, bytes.to_vec()))
}

/// Raw database-state read for parity comparison — read-only SQL, independent of the
/// domain and store (the spirit of `proof.rs`: the auditor must not inherit bugs from the
/// audited code).
///
/// # Errors
/// SQL error (the database is inconsistent — itself a parity finding).
pub async fn db_snapshot(pool: &sqlx::SqlitePool, expect: &DbExpect) -> anyhow::Result<DbActual> {
    use sqlx::Row;
    let mut actual = DbActual::default();
    if expect.user_wallet_sum.is_some() {
        let row = sqlx::query(
            r#"SELECT COALESCE(SUM(p."amount"),0) AS s
               FROM "Posting" p JOIN "Account" a ON a."id" = p."accountId"
               WHERE a."kind" = 'user'"#,
        )
        .fetch_one(pool)
        .await?;
        actual.user_wallet_sum = Some(row.get::<i64, _>("s"));
    }
    if expect.treasury.is_some() {
        let row = sqlx::query(
            r#"SELECT COALESCE(SUM(p."amount"),0) AS s
               FROM "Posting" p WHERE p."accountId" = 'treasury:nation'"#,
        )
        .fetch_one(pool)
        .await?;
        actual.treasury = Some(row.get::<i64, _>("s"));
    }
    if expect.tx_count.is_some() {
        let row = sqlx::query(r#"SELECT COUNT(*) AS n FROM "Tx""#).fetch_one(pool).await?;
        actual.tx_count = Some(row.get::<i64, _>("n"));
    }
    if expect.outbox_count.is_some() {
        let row = sqlx::query(r#"SELECT COUNT(*) AS n FROM "OutboxEvent""#).fetch_one(pool).await?;
        actual.outbox_count = Some(row.get::<i64, _>("n"));
    }
    for id in expect.escrow_status.keys() {
        let row = sqlx::query(r#"SELECT "status" FROM "Escrow" WHERE "id" = ?"#)
            .bind(id)
            .fetch_optional(pool)
            .await?;
        if let Some(r) = row {
            actual.escrow_status.insert(format!("escrow:{id}"), r.get::<String, _>("status"));
        }
    }
    Ok(actual)
}

/// Run a full scenario on a fresh router — raw result (no comparison).
///
/// # Errors
/// Boot/SQL error — the parity environment itself is unhealthy.
pub async fn run_scenario(
    scenario: &Scenario,
    database_url: &str,
) -> anyhow::Result<ScenarioResult> {
    let (router, store) = boot(&scenario.boot, database_url).await?;
    for step in &scenario.setup {
        let (status, _, _) = send_step(&router, step).await?;
        // setup steps must never return 5xx — a 5xx means the engine itself is broken
        if status.is_server_error() {
            anyhow::bail!("setup step {} {} → {status}", step.method, step.path);
        }
    }
    let (status, ctype, raw) = send_step(&router, &scenario.request).await?;
    let body = normalize_body(&raw);
    let db = db_snapshot(store.pool_ref(), &scenario.db).await?;
    Ok(ScenarioResult {
        name: scenario.name.to_owned(),
        status: status.as_u16(),
        content_type: ctype,
        body,
        db,
    })
}

/// Compare the result against the golden — list of mismatches.
#[must_use]
pub fn compare(result: &ScenarioResult, golden: &Golden) -> ScenarioCheck {
    let mut diffs = Vec::new();
    if result.status != golden.status {
        diffs.push(format!("status {} ≠ {}", result.status, golden.status));
    }
    if result.content_type != golden.content_type {
        diffs.push(format!("content-type {:?} ≠ {:?}", result.content_type, golden.content_type));
    }
    if result.body != golden.body.as_bytes() {
        let first_diff = result
            .body
            .iter()
            .zip(golden.body.as_bytes())
            .position(|(a, b)| a != b)
            .unwrap_or(result.body.len().min(golden.body.len()));
        diffs.push(format!(
            "body mismatch: {} bytes ≠ {} bytes — first diff at byte {first_diff}",
            result.body.len(),
            golden.body.len()
        ));
    }
    let golden_db = serde_json::to_value(&golden.db).unwrap_or(Value::Null);
    let actual_db = serde_json::to_value(&result.db).unwrap_or(Value::Null);
    if golden_db != actual_db {
        diffs.push(format!("db state {actual_db} ≠ {golden_db}"));
    }
    ScenarioCheck { name: result.name.clone(), ok: diffs.is_empty(), diffs }
}

/// Default golden folder — `crates/tooling/tests/golden`.
#[must_use]
pub fn default_golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

/// Golden file path for one scenario.
#[must_use]
pub fn golden_path(dir: &Path, name: &str) -> PathBuf {
    dir.join(format!("{name}.json"))
}

/// Load all goldens from the folder — missing → `None` (reported as «no golden»).
///
/// # Errors
/// I/O error or malformed JSON.
pub fn load_goldens(dir: &Path) -> anyhow::Result<Vec<Option<Golden>>> {
    scenarios::scenarios()
        .iter()
        .map(|s| {
            let path = golden_path(dir, s.name);
            if path.exists() {
                let raw = std::fs::read_to_string(&path)?;
                Ok(Some(serde_json::from_str::<Golden>(&raw)?))
            } else {
                Ok(None)
            }
        })
        .collect()
}

/// Run all scenarios and compare against the goldens — the core of the parity gate.
///
/// The function argument provisions a fresh database URL per scenario (scenario
/// isolation).
///
/// # Errors
/// Golden I/O error or boot failure (mismatches themselves live in the report).
pub async fn verify_all(
    database_url_factory: impl Fn(usize) -> String,
) -> anyhow::Result<ParityReport> {
    let dir = default_golden_dir();
    let goldens = load_goldens(&dir)?;
    let mut checks = Vec::new();
    for (idx, scenario) in scenarios::scenarios().iter().enumerate() {
        let url = database_url_factory(idx);
        let result = run_scenario(scenario, &url).await?;
        let check = match &goldens[idx] {
            Some(golden) => compare(&result, golden),
            None => ScenarioCheck {
                name: scenario.name.to_owned(),
                ok: false,
                diffs: vec!["golden file missing — run `parity --record`".into()],
            },
        };
        checks.push(check);
    }
    Ok(ParityReport { checks })
}

/// Record all goldens from the current Rust behavior — the parity starting point; the real
/// TS↔Rust comparison in production runs with `parity --ts-url`.
///
/// # Errors
/// I/O or boot error.
pub async fn record_all(database_url_factory: impl Fn(usize) -> String) -> anyhow::Result<usize> {
    let dir = default_golden_dir();
    std::fs::create_dir_all(&dir)?;
    let mut count = 0;
    for (idx, scenario) in scenarios::scenarios().iter().enumerate() {
        let url = database_url_factory(idx);
        let result = run_scenario(scenario, &url).await?;
        let golden = Golden {
            name: result.name.clone(),
            status: result.status,
            content_type: result.content_type.clone(),
            body: String::from_utf8(result.body.clone())?,
            db: result.db.clone(),
        };
        std::fs::write(golden_path(&dir, scenario.name), serde_json::to_vec_pretty(&golden)?)?;
        count += 1;
    }
    Ok(count)
}
