//! نقطهٔ ورود سرور — بوت fail-closed: دیتابیس ناسالم یا مهاجرت شکسته، پروسه را
//! با کد خروج غیرصفر و پیام صادقانه می‌بندد. لاگ‌ها JSON — trace layer آکسوم.

use kian_server::{build_router, AppState, ServerConfig};

#[tokio::main]
async fn main() {
    // لاگ JSON — env-filter با RUST_LOG (پیش‌فرض info)
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let config = ServerConfig::from_env();
    let admin_enabled = config.admin_key.is_some();

    // پوشهٔ والد دیتابیس فایل‌محور — sqlx فایل را می‌سازد اما پوشه را نه
    // let-chains (edition 2024) — گارد پوشهٔ دیتابیس در یک شرط
    if let Some(parent) = config.db_file_path().and_then(|p| p.parent().map(std::path::Path::to_path_buf))
        && let Err(e) = std::fs::create_dir_all(&parent)
    {
        eprintln!("boot refused: cannot create data dir {}: {e}", parent.display());
        std::process::exit(3);
    }

    let store = match kian_db::V3Store::connect(&config.database_url).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("boot refused: cannot open database {} — {e}", config.database_url);
            std::process::exit(3);
        }
    };
    if let Err(e) = store.migrate().await {
        eprintln!("boot refused: migrations failed — {e}");
        std::process::exit(3);
    }
    if let Err(e) = store.ensure_system_accounts().await {
        eprintln!("boot refused: system accounts failed — {e}");
        std::process::exit(3);
    }
    // مغز M4 — ۱۶ مهارت هرمس + ناوگان ۷ نقش (idempotent؛ وضعیت ادمین حفظ می‌شود)
    if let Err(e) = kian_server::handlers::hermes::seed_skills(&store).await {
        eprintln!("boot refused: hermes skill seed failed — {e}");
        std::process::exit(3);
    }
    if let Err(e) = kian_server::handlers::agents::seed_agents(&store).await {
        eprintln!("boot refused: agent fleet seed failed — {e}");
        std::process::exit(3);
    }

    let addr = format!("{}:{}", config.host, config.port);
    let state = AppState::new(store, config).shared();
    let router = build_router(state).into_make_service_with_connect_info::<std::net::SocketAddr>();

    tracing::info!(
        admin_enabled,
        "kian-server listening on http://{addr} — kernel surface under /api/v1"
    );

    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("boot refused: cannot bind {addr} — {e}");
            std::process::exit(4);
        }
    };
    if let Err(e) = axum::serve(listener, router).await {
        eprintln!("server error: {e}");
        std::process::exit(5);
    }
}
