//! `parity` — the phase 8 parity gate entry point (ADR-009 §7 phase 8 / §12).
//!
//! Three modes:
//! * `--verify` (default) — replay all scenarios on Rust and compare
//!   byte-for-byte with the `tests/golden/` goldens + read the database state.
//! * `--record` — regenerate the goldens from current Rust behavior (after every intentional
//!   contract change).
//! * `--ts-url <base>` — live TS↔Rust comparison: the same scenarios are fired at both engines;
//!   TS must run on a fresh database (both sides get identical
//!   setup) — this is the exit gate of every dual-run step.
//!
//! Output: full report + `--report out.json` (the dual-run gate input) + exit 0/1.

use std::path::PathBuf;

use dropagentx_tooling::parity::{
    self, normalize_body, run_scenario, scenarios as scenario_registry, ScenarioCheck,
    ScenarioResult,
};
use dropagentx_tooling::parity::{compare, Golden};

/// Read a `--flag value` argument.
fn arg_of(args: &[String], flag: &str) -> Option<String> {
    args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1)).cloned()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let record = args.iter().any(|a| a == "--record");
    let ts_url = arg_of(&args, "--ts-url");
    let report_path = arg_of(&args, "--report");
    let mode = if record {
        "record"
    } else if ts_url.is_some() {
        "live-ts"
    } else {
        "verify"
    };
    println!("parity harness — mode: {mode}");

    if record {
        let count = parity::record_all(|_| "sqlite::memory:".into()).await?;
        println!("recorded {count} golden scenarios → {:?}", parity::default_golden_dir());
        return Ok(());
    }

    let report = match ts_url {
        Some(base) => verify_against_ts(&base).await?,
        None => parity::verify_all(|_| "sqlite::memory:".into()).await?,
    };

    println!("{}", report.render());
    if let Some(path) = report_path {
        let doc = serde_json::json!({
            "mode": mode,
            "green": report.summary().0,
            "total": report.summary().1,
            "ok": report.ok(),
            "checks": report.checks.iter().map(|c| serde_json::json!({
                "name": c.name,
                "ok": c.ok,
                "diffs": c.diffs,
            })).collect::<Vec<_>>(),
        });
        let path = PathBuf::from(path);
        std::fs::write(&path, serde_json::to_vec_pretty(&doc)?)?;
        println!("report → {}", path.display());
    }
    if report.ok() {
        println!("PARITY OK — gate green.");
        Ok(())
    } else {
        println!("PARITY FAILED — rollback rule: RUST_TRAFFIC_PCT=0");
        std::process::exit(1);
    }
}

/// Live TS↔Rust comparison — the same scenarios at both engines; TS must run on a fresh
/// database (identical setup is applied to both sides).
async fn verify_against_ts(base: &str) -> anyhow::Result<parity::ParityReport> {
    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(30)).build()?;
    let base = base.trim_end_matches('/').to_owned();
    let mut checks = Vec::new();
    for scenario in scenario_registry() {
        // Rust side — in-memory, environment-independent
        let rust: ScenarioResult = run_scenario(&scenario, "sqlite::memory:").await?;
        // TS side — over HTTP with identical setup
        let ts = send_to_ts(&client, &base, &scenario).await;
        let check = match ts {
            Ok(ts) => compare_cross(&rust, &ts),
            Err(e) => ScenarioCheck {
                name: scenario.name.to_owned(),
                ok: false,
                diffs: vec![format!("TS unreachable: {e}")],
            },
        };
        checks.push(check);
    }
    Ok(parity::ParityReport { checks })
}

/// Send the setup + scenario request to the TS server — the normalized result.
async fn send_to_ts(
    client: &reqwest::Client,
    base: &str,
    scenario: &dropagentx_tooling::parity::Scenario,
) -> anyhow::Result<ScenarioResult> {
    for step in &scenario.setup {
        send_step_ts(client, base, step).await?;
    }
    let (status, ctype, raw) = send_step_ts(client, base, &scenario.request).await?;
    Ok(ScenarioResult {
        name: scenario.name.to_owned(),
        status,
        content_type: ctype,
        body: normalize_body(&raw),
        db: dropagentx_tooling::parity::DbActual::default(),
    })
}

/// One HTTP step to TS — with parity headers.
async fn send_step_ts(
    client: &reqwest::Client,
    base: &str,
    step: &dropagentx_tooling::parity::Step,
) -> anyhow::Result<(u16, String, Vec<u8>)> {
    let method = reqwest::Method::from_bytes(step.method.as_bytes())?;
    let mut req = client.request(method, format!("{base}/api/v1{}", step.path));
    if step.admin {
        req = req.header("x-kia-key", dropagentx_tooling::parity::PARITY_KEY);
    }
    if step.bot_secret_header {
        req = req
            .header("x-telegram-bot-api-secret-token", dropagentx_tooling::parity::PARITY_SECRET);
    }
    if let Some(body) = &step.body {
        req = req.header("content-type", "application/json").body(body.to_string());
    }
    let res = req.send().await?;
    let status = res.status().as_u16();
    let ctype = res
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_owned();
    let bytes = res.bytes().await?.to_vec();
    Ok((status, ctype, bytes))
}

/// Rust↔TS cross-comparison — no goldens; the contract itself is the reference.
fn compare_cross(rust: &ScenarioResult, ts: &ScenarioResult) -> ScenarioCheck {
    let mut diffs = Vec::new();
    if rust.status != ts.status {
        diffs.push(format!("status rust {} ≠ ts {}", rust.status, ts.status));
    }
    if rust.content_type != ts.content_type {
        diffs.push(format!("content-type rust {:?} ≠ ts {:?}", rust.content_type, ts.content_type));
    }
    if rust.body != ts.body {
        let first = rust
            .body
            .iter()
            .zip(ts.body.iter())
            .position(|(a, b)| a != b)
            .unwrap_or(rust.body.len().min(ts.body.len()));
        diffs.push(format!(
            "body diff at byte {first} (rust {} B vs ts {} B)",
            rust.body.len(),
            ts.body.len()
        ));
    }
    ScenarioCheck { name: rust.name.clone(), ok: diffs.is_empty(), diffs }
}

/// Usage check — `compare` (golden-based) is used in the library verify path.
#[allow(dead_code)]
fn _uses_compare() {
    let _ = compare as fn(&ScenarioResult, &Golden) -> ScenarioCheck;
}
