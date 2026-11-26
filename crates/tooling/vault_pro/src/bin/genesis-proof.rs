//! Port of `back/server/scripts/genesis-proof.ts` — external genesis audit.
//!
//! Raw read-only SQL — deliberately bypasses the domain and store so the numbers cannot
//! inherit bugs from the audited code. Exits 1 if any expectation is violated.
//!
//! ```text
//! cargo run -p dropagentx-tooling --bin genesis-proof -- --db file:./db/v3.db
//! ```

use std::process::ExitCode;

fn arg_of(name: &str) -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1).cloned())
}

#[tokio::main]
async fn main() -> ExitCode {
    let db_path = match arg_of("--db") {
        Some(p) => p,
        None => match std::env::var("DATABASE_URL") {
            Ok(u) => u,
            Err(_) => {
                eprintln!("Usage: genesis-proof --db file:./db/v3.db (or DATABASE_URL)");
                return ExitCode::from(2);
            }
        },
    };
    match dropagentx_db::genesis_proof(&db_path).await {
        Ok(report) => {
            for line in &report.lines {
                println!("{line}");
            }
            if report.ok {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(e) => {
            eprintln!("[proof] FAILED: {e:#}");
            ExitCode::FAILURE
        }
    }
}
