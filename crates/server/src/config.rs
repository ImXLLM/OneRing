//! Server boot configuration — env-only; no secrets in code (REA filter: `text.credential_markers`).
//!
//! fail-closed: an unset admin key means admin endpoints are **disabled** (never open) —
//! the decision happens in `verify_kian_key` + `require_admin`; this module only reads it.
//! The same contract holds for `KIAN_BOT_TOKEN` (M3 Telegram identity) and the AI slots (M4).

use kian_ai::AiConfig;

/// Boot configuration — immutable after startup.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Admin key — from `KIAN_ADMIN_KEY`; `None` = admin disabled (fail-closed).
    /// Minimum 16 characters enforced by the contractual guard.
    pub admin_key: Option<String>,
    /// Telegram bot token — from `KIAN_BOT_TOKEN`; `None` = Telegram identity disabled
    /// (fail-closed; same contract as admin). Never logged or echoed into responses.
    pub bot_token: Option<String>,
    /// Telegram webhook secret — from `KIAN_BOT_WEBHOOK_SECRET`; `None` = the bot webhook
    /// is disabled (fail-closed, Phase 5). Telegram sends the header
    /// `X-Telegram-Bot-Api-Secret-Token` on every update; compared in constant time.
    pub bot_webhook_secret: Option<String>,
    /// Mini-app URL — from `KIAN_MINIAPP_URL`; default localhost:3000. The
    /// «Open Mini App» button in bot replies comes from this.
    pub miniapp_url: String,
    /// Bearer session TTL in hours — from `KIAN_SESSION_TTL_HOURS`; default 168 (7 days).
    pub session_ttl_hours: i64,
    /// TON treasury address for deposits — from `KIAN_TON_TREASURY`; default empty string
    /// (the front end consumes no `/health`-like endpoint; env is client-side only).
    pub ton_treasury: String,
    /// Din-to-TON conversion rate — from `KIAN_TON_TO_DIN`; default 1000 Din per TON.
    /// Mint calculation: `micro_ton * rate / 1_000_000` — floored integer division (minimum 1).
    pub ton_to_din_rate: i64,
    /// Din-to-USDT conversion rate — from `KIAN_USDT_TO_DIN`; default 1000 Đ to USDT
    /// (same scale as the TON rate). Calculation: `micro_usdt * rate / 1_000_000`.
    pub usdt_to_din_rate: i64,
    /// Parliament quorum — from `KIAN_NATION_QUORUM`; default 3.
    pub nation_quorum: i64,
    /// Per-chain treasury address — from `KIAN_TREASURY_TRX/SOL/TON/BSC`; empty means
    /// unset (transparent in `/wallet/treasury`, not hidden).
    pub treasury_trx: String,
    /// Solana treasury
    pub treasury_sol: String,
    /// TON treasury
    pub treasury_ton: String,
    /// Binance Chain treasury
    pub treasury_bsc: String,
    /// AI gateway (M4) — three fallback slots + a daily cap; loaded from env via [`AiConfig::from_env`].
    /// Slot keys are never logged or put on the wire.
    pub ai: AiConfig,
    /// sqlite database — from `KIAN_DATABASE_URL`; default local file. Format:
    /// raw path (`data/kian.sqlite`), `:memory:`, or `sqlite::memory:` —
    /// `V3Store::connect` takes the whole string as a filename (no scheme/query).
    pub database_url: String,
    /// Bind host — from `KIAN_HOST`; default 127.0.0.1.
    pub host: String,
    /// Bind port — from `KIAN_PORT`; default 8088.
    pub port: u16,
    /// Request cap per 60-second window — from `KIAN_RATE_LIMIT`; default 120.
    pub rate_limit: usize,
    /// `X-Forwarded-For` trust — from `KIAN_TRUST_PROXY`; default true
    /// (deployment behind a trusted proxy such as Railway that writes XFF). false =
    /// only the socket IP counts — direct deployment: any client with forged XFF could
    /// slip past the rate cap (security fix from the 2025-11 load test).
    pub trust_proxy: bool,
    /// `VACUUM INTO` backups folder — from `KIAN_BACKUP_DIR`; default data/backups.
    pub backup_dir: String,
    /// Backups retained — from `KIAN_KEEP_BACKUPS`; default 7.
    pub keep_backups: usize,
    /// Allowed CORS origins (web mini app) — from `KIAN_CORS_ORIGIN`, comma-separated;
    /// default localhost:3000 only. fail-closed: unknown origins never receive a
    /// preflight response.
    pub cors_origins: Vec<String>,
    /// Development/demo mode — enables the `/auth/dev-login` and `/wallet/faucet`
    /// endpoints. Strict fail-closed contract (D1 forensic review — Commit #27):
    /// only the explicit values `KIAN_DEV_MODE=1`/`true`/`yes` (case-insensitive)
    /// turn it on; unset or any other value (`0`, `false`, `no`, `on`,
    /// empty string) means off — even with the bot token absent too. A missing bot token
    /// never turns `dev_mode` on; tests set `config.dev_mode = true` explicitly
    /// ([`ServerConfig::base`] is the test base, not production).
    pub dev_mode: bool,
}

impl ServerConfig {
    /// fail-closed base — the foundation of `from_env` and the test constructor: everything off,
    /// local bind, no admin, no AI slots.
    #[must_use]
    pub fn base() -> Self {
        Self {
            admin_key: None,
            bot_token: None,
            bot_webhook_secret: None,
            miniapp_url: "http://localhost:3000".into(),
            session_ttl_hours: 168,
            ton_treasury: String::new(),
            ton_to_din_rate: 1000,
            usdt_to_din_rate: 1000,
            nation_quorum: 3,
            treasury_trx: String::new(),
            treasury_sol: String::new(),
            treasury_ton: String::new(),
            treasury_bsc: String::new(),
            ai: AiConfig {
                slots: Vec::new(),
                daily_cap_tokens: 50_000,
            },
            database_url: "sqlite::memory:".into(),
            host: "127.0.0.1".into(),
            port: 0,
            rate_limit: 10_000,
            trust_proxy: true,
            backup_dir: "/tmp/kian-test-backups".into(),
            keep_backups: 1,
            cors_origins: Vec::new(),
            // test base — demo endpoints on so the builder->purchase loop opens without Telegram
            // opens. Production never passes through here: `from_env` is strictly
            // fail-closed (only `KIAN_DEV_MODE=1|true|yes` turns it on) and
            // the demo tests set this field explicitly.
            dev_mode: true,
        }
    }

    /// File path for file-backed databases — `None` for in-memory.
    /// Same normalization as `V3Store::connect`: strips the `file:`/`sqlite:` prefix and
    /// detects `:memory:`; the `?query` part is ignored.
    #[must_use]
    pub fn db_file_path(&self) -> Option<std::path::PathBuf> {
        let without_file = self
            .database_url
            .strip_prefix("file:")
            .unwrap_or(&self.database_url);
        let without_scheme = without_file
            .strip_prefix("sqlite:")
            .filter(|&rest| rest != ":memory:")
            .unwrap_or(without_file);
        let normalized = without_scheme.split('?').next().unwrap_or(without_scheme);
        if normalized == ":memory:" || normalized == "sqlite::memory:" || normalized.trim().is_empty() {
            return None;
        }
        Some(std::path::PathBuf::from(normalized))
    }

    /// Read from the environment — nothing fails; a missing admin just means «disabled».
    #[must_use]
    pub fn from_env() -> Self {
        let env = |k: &str| std::env::var(k).ok().filter(|v| !v.trim().is_empty());
        let bot_token = env("KIAN_BOT_TOKEN");
        // D1 (forensic commission #27): dev mode is strictly fail-closed — the flag's absence
        // never turns it on, even without a bot token (the old "missing bot token means dev on"
        // behavior was removed; the fail-open backdoor is closed).
        let dev_mode = dev_mode_from_env(std::env::var("KIAN_DEV_MODE").ok());
        Self {
            admin_key: env("KIAN_ADMIN_KEY"),
            bot_token,
            bot_webhook_secret: env("KIAN_BOT_WEBHOOK_SECRET"),
            miniapp_url: env("KIAN_MINIAPP_URL").unwrap_or_else(|| "http://localhost:3000".into()),
            session_ttl_hours: env("KIAN_SESSION_TTL_HOURS")
                .and_then(|p| p.parse().ok())
                .unwrap_or(168),
            ton_treasury: env("KIAN_TON_TREASURY").unwrap_or_default(),
            ton_to_din_rate: env("KIAN_TON_TO_DIN")
                .and_then(|p| p.parse().ok())
                .unwrap_or(1000),
            usdt_to_din_rate: env("KIAN_USDT_TO_DIN")
                .and_then(|p| p.parse().ok())
                .unwrap_or(1000),
            nation_quorum: env("KIAN_NATION_QUORUM")
                .and_then(|p| p.parse().ok())
                .unwrap_or(3),
            treasury_trx: env("KIAN_TREASURY_TRX").unwrap_or_default(),
            treasury_sol: env("KIAN_TREASURY_SOL").unwrap_or_default(),
            treasury_ton: env("KIAN_TREASURY_TON").unwrap_or_default(),
            treasury_bsc: env("KIAN_TREASURY_BSC").unwrap_or_default(),
            ai: AiConfig::from_env(),
            database_url: env("KIAN_DATABASE_URL").unwrap_or_else(|| "data/kian.sqlite".into()),
            host: env("KIAN_HOST").unwrap_or_else(|| "127.0.0.1".into()),
            port: env("KIAN_PORT")
                .and_then(|p| p.parse().ok())
                .unwrap_or(8088),
            rate_limit: env("KIAN_RATE_LIMIT")
                .and_then(|p| p.parse().ok())
                .unwrap_or(120),
            trust_proxy: env("KIAN_TRUST_PROXY").is_none_or(
                |v| !matches!(v.to_ascii_lowercase().as_str(), "0" | "false" | "no" | "off"),
            ),
            backup_dir: env("KIAN_BACKUP_DIR").unwrap_or_else(|| "data/backups".into()),
            keep_backups: env("KIAN_KEEP_BACKUPS")
                .and_then(|p| p.parse().ok())
                .unwrap_or(7),
            cors_origins: env("KIAN_CORS_ORIGIN").map_or_else(
                || {
                    vec![
                        "http://localhost:3000".into(),
                        "http://127.0.0.1:3000".into(),
                    ]
                },
                |raw| {
                    raw.split(',')
                        .map(str::trim)
                        .filter(|o| !o.is_empty())
                        .map(str::to_owned)
                        .collect()
                },
            ),
            dev_mode,
        }
    }
}

/// D1 — the pure fail-closed core of dev mode: only `1`/`true`/`yes`
/// (case-insensitive, trimmed) turn it on; `None` (unset) and
/// any other value — even without a bot token — means off. Pure function so tests can
/// run without env races (env is process-wide and parallel tests would interfere).
fn dev_mode_from_env(raw: Option<String>) -> bool {
    raw.is_some_and(|v| {
        matches!(
            v.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes"
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Safe defaults: admin disabled, local bind, a reasonable rate cap.
    #[test]
    fn defaults_are_fail_closed() {
        // env variables may be set in the test environment — we only check the structure
        let cfg = ServerConfig::from_env();
        if std::env::var("KIAN_ADMIN_KEY").is_err() {
            assert!(
                cfg.admin_key.is_none(),
                "unset admin key must stay disabled"
            );
        }
        assert!(cfg.rate_limit > 0);
        assert!(cfg.keep_backups > 0);
    }

    /// Dev mode — the explicit flag trumps everything; tests set the field explicitly.
    #[test]
    fn dev_mode_is_explicit_field_on_test_base() {
        // the explicit flag trumps a present bot token
        let mut cfg = ServerConfig::base();
        cfg.bot_token = Some("123:tok".into());
        cfg.dev_mode = false;
        assert!(!cfg.dev_mode);
        cfg.dev_mode = true;
        assert!(cfg.dev_mode);
    }

    /// D1 — forensic commission #27: an unset `KIAN_DEV_MODE` is strictly fail-closed;
    /// a missing bot token never turns `dev_mode` on again.
    #[test]
    fn dev_mode_is_strictly_fail_closed_when_unset_even_without_bot_token() {
        // "unset" = None — the output is false regardless of bot-token state
        assert!(!dev_mode_from_env(None));
        assert!(!dev_mode_from_env(Some(String::new())));
        assert!(!dev_mode_from_env(Some("   ".into())));
        // partial near-on values also stay off (strict fail-closed)
        for off in ["0", "false", "no", "off", "on", "enabled", "dev", "1,0", "yes please"] {
            assert!(!dev_mode_from_env(Some(off.into())), "{off} must stay off");
        }
    }

    /// D1 — only the three explicit values turn it on; every other value is off.
    #[test]
    fn dev_mode_only_explicit_truthy_values_enable_it() {
        for on in ["1", "true", "TRUE", "True", "yes", "YES", " 1 ", "true "] {
            assert!(dev_mode_from_env(Some(on.into())), "{on} must enable");
        }
        for off in ["0", "false", "no", "off", "on", "enabled", "dev", "1,0"] {
            assert!(!dev_mode_from_env(Some(off.into())), "{off} must stay off");
        }
    }

    // ── D6 — deterministic 69-key parity of P0..P5 across all three env files (Commit #28) ──

    /// Workspace root from `CARGO_MANIFEST_DIR` (`crates/server` → root).
    fn workspace_root() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .expect("workspace root is two levels above crates/server")
            .to_path_buf()
    }

    /// `KEY=VALUE` keys of an env file in order of appearance — comments/blank lines ignored.
    fn parse_env_keys(path: &std::path::Path) -> Vec<String> {
        std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
            .lines()
            .filter_map(|line| {
                let t = line.trim();
                if t.is_empty() || t.starts_with('#') {
                    return None;
                }
                let key = t.split('=').next()?.trim();
                let is_key = !key.is_empty()
                    && key
                        .chars()
                        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_');
                is_key.then(|| key.to_owned())
            })
            .collect()
    }

    /// All three env files must carry exactly the same 69 canonical keys in the same
    /// P0 → P5 order — the deterministic Gate 2 lock (D6).
    #[test]
    fn env_files_have_exact_69_key_p0_to_p5_parity() {
        let root = workspace_root();
        let env_path = root.join(".env.example");
        let railway_path = root.join("railway.env.example");
        let json_path = root.join("railway-env-v1.0.0.json");

        let canonical = parse_env_keys(&env_path);
        assert_eq!(canonical.len(), 69, ".env.example must carry exactly 69 keys");
        let p0 = ["KIAN_DATABASE_URL", "KIAN_HOST", "KIAN_PORT", "KIAN_SEED_DATABASE_PATH", "KIAN_BACKUP_DIR", "KIAN_KEEP_BACKUPS"];
        for (idx, key) in p0.iter().enumerate() {
            assert_eq!(canonical[idx], *key, "P0 block must open .env.example in order");
        }

        // railway.env.example — the same byte-for-byte order
        let railway = parse_env_keys(&railway_path);
        assert_eq!(railway.len(), 69, "railway.env.example must carry exactly 69 keys");
        assert_eq!(railway, canonical, "railway.env.example key order must mirror .env.example exactly");

        // railway-env-v1.0.0.json — P0..P5 groups with the same key set
        let raw = std::fs::read_to_string(&json_path)
            .unwrap_or_else(|e| panic!("read {}: {e}", json_path.display()));
        let parsed: serde_json::Value = serde_json::from_str(&raw)
            .unwrap_or_else(|e| panic!("railway-env json must parse: {e}"));
        let groups = [
            "P0_critical_boot_network_storage",
            "P1_sovereign_security_fail_closed",
            "P2_ai_inference_mesh",
            "P3_telegram_bridge",
            "P4_non_custodial_treasury_rails",
            "P5_runtime_governance_build_vars",
        ];
        let mut json_keys = Vec::new();
        for g in groups {
            let obj = parsed
                .get(g)
                .unwrap_or_else(|| panic!("missing priority group {g}"))
                .as_object()
                .unwrap_or_else(|| panic!("group {g} must be an object"));
            json_keys.extend(obj.keys().cloned());
        }
        assert_eq!(json_keys.len(), 69, "json must carry exactly 69 env keys");
        let mut sorted_json = json_keys.clone();
        sorted_json.sort();
        let mut sorted_canonical = canonical.clone();
        sorted_canonical.sort();
        assert_eq!(sorted_json, sorted_canonical, "json key set must equal the 69 canonical keys");
        // KIAN_DEV_MODE=0 and a >=16-character admin key on the JSON wire
        assert_eq!(parsed["P1_sovereign_security_fail_closed"]["KIAN_DEV_MODE"], "0");
        assert_eq!(
            parsed["P1_sovereign_security_fail_closed"]["KIAN_ADMIN_KEY"],
            "replace-with-32-char-sovereign-admin-key-min16"
        );

        // physical order of the JSON file — the first occurrence of each key must follow the same
        // P0 → P5 order (groups come before _notes).
        let mut last: Option<usize> = None;
        for key in &canonical {
            let needle = format!("\"{key}\"");
            let pos = raw.find(&needle).unwrap_or_else(|| panic!("key {key} absent from json"));
            if let Some(prev) = last {
                assert!(pos > prev, "json key {key} appears out of P0→P5 order");
            }
            last = Some(pos);
        }
    }

}
