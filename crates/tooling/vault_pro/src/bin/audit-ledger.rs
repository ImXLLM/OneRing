//! `audit-ledger` — final audit of the phase 8 cut-over (ADR-009 §7 phase 8).
//!
//! Acceptance criterion: «final audit: Σ wallets = 187,672 Đ and trialBalance.balanced
//! == true — the same number as previous generations, now with the Rust core».
//!
//! This binary runs against the target database file (read-only, raw SQL like
//! `genesis-proof`) and prints a go/no-go verdict at the end:
//! * `AUDIT PASS` → exit 0 — allowed to tag `rust-core-1.0.0` and shut down Node
//! * `AUDIT FAILED` → exit 1 — never cut over with a financial parity failure
//!
//! Usage: `audit-ledger <db-path>`

use dropagentx_db::genesis_proof;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(db_path) = args.first() else {
        eprintln!("usage: audit-ledger <db-path>");
        std::process::exit(2);
    };

    println!("══════════ cut-over final audit — ADR-009 phase 8 ══════════");
    println!("database : {db_path}");
    println!();

    let report = match genesis_proof(db_path).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("AUDIT FAILED — cannot open/audit database: {e}");
            std::process::exit(1);
        }
    };
    for line in &report.lines {
        println!("{line}");
    }

    // Final verdict — the same two acceptance criteria as ADR-009
    println!();
    if report.ok {
        println!("AUDIT PASS — Σ wallets = 187,672 Đ · trialBalance.balanced = true");
        println!("→ \u{645}\u{62c}\u{627}\u{632} \u{628}\u{647} \u{62a}\u{6af} rust-core-1.0.0 \u{648} \u{62e}\u{627}\u{645}\u{648}\u{634}\u{6cc} Node (\u{67e}\u{633} \u{627}\u{632} \u{6f2} \u{647}\u{641}\u{62a}\u{647} \u{6f1}\u{6f0}\u{6f0}\u{66a} \u{633}\u{628}\u{632}).");
        Ok(())
    } else {
        println!("AUDIT FAILED — {}", report.problems.join(" · "));
        println!("→ \u{628}\u{631}\u{6af}\u{634}\u{62a}: RUST_TRAFFIC_PCT=0 — \u{647}\u{631}\u{6af}\u{632} \u{628}\u{627} \u{645}\u{63a}\u{627}\u{6cc}\u{631}\u{62a} \u{645}\u{627}\u{644}\u{6cc} cut-over \u{646}\u{6a9}\u{646}\u{6cc}\u{62f}.");
        std::process::exit(1);
    }
}
