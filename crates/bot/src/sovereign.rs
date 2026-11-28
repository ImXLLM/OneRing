//! The MAIRON sovereign command suite — deterministic formatters for the six
//! flagship commands (`/mairon`, `/trinity`, `/quant`, `/treasury`, `/warroom`,
//! `/hud`).
//!
//! Rules (the same repo-wide orchestrator contract):
//! * **Zero I/O** — every formatter is a pure function of compile-time constants.
//! * **Zero secrets** — no token, key or private URL ever enters a formatted text
//!   (locked by the orchestrator's secret-leak tests plus this module's own tests).
//! * **International English only** — the MAIRON brand is served worldwide; every
//!   string is plain English (no RTL, no locale-dependent digits).
//! * Every fact printed here mirrors the kernel constants: Trinity IQ tiers
//!   (136 / 139 / 150), the REA ≥ 85 quality gate, the 8 keyless CCXT venues,
//!   the Sharpe ≥ 1.20 live-deployment gate, Base-8453 treasury constants
//!   (chain id 8453, USDC `0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913`, `$0xKIA`,
//!   XTR Telegram Stars), the 4-stream revenue split, the capital-reserve
//!   partition, the 24-tick founder war-room and the 180-node neural HUD.

/// The Base-8453 chain id — the sovereign settlement chain.
pub const BASE_CHAIN_ID: u64 = 8453;

/// The canonical USDC contract on Base Mainnet.
pub const BASE_USDC_CONTRACT: &str = "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913";

/// The sovereign token ticker.
pub const KIA_TOKEN_SYMBOL: &str = "$0xKIA";

/// Telegram Stars currency code (in-app settlement rail).
pub const TELEGRAM_STARS_CURRENCY: &str = "XTR";

/// Trinity IQ tiers — the three-avatar calibration.
pub const TRINITY_IQ_ALPHA: u16 = 136;
pub const TRINITY_IQ_COMMAND_CENTER: u16 = 139;
pub const TRINITY_IQ_MAIRON: u16 = 150;

/// The REA review gate — an avatar build must score at least this to pass.
pub const REA_MIN_SCORE: u32 = 85;

/// The quant live-deployment gate — a strategy must beat this Sharpe.
pub const MAIRON_MIN_SHARPE: f64 = 1.20;

/// The neural-HUD topology — Fibonacci-sphere nodes across the dual shell.
pub const NEURAL_HUD_NODE_COUNT: usize = 180;

/// The founder war-room hourly causal tick count.
pub const WAR_ROOM_TICKS: usize = 24;

/// The war-room day-1 revenue delta (micros of USDC) — $9.55.
pub const WAR_ROOM_DAY1_DELTA_MICROS: i64 = 9_550_000;

/// The war-room closing wallet total (micros of USDC) — $42.55.
pub const WAR_ROOM_FINAL_TOTAL_MICROS: i64 = 42_550_000;

/// Capital reserve — operational crypto seed ($33.00).
pub const CAPITAL_RESERVE_CRYPTO_SEED_CENTS: i64 = 3_300;

/// Capital reserve — cash float ($15.00).
pub const CAPITAL_RESERVE_CASH_CENTS: i64 = 1_500;

/// Capital reserve — locked cold reserve ($350.00).
pub const CAPITAL_RESERVE_LOCKED_CENTS: i64 = 35_000;

/// The 8 keyless CCXT venues (zero API keys, zero KYC) — mirrors
/// `kian_wallet::ccxt_core::keyless_public_market_matrix`.
pub const KEYLESS_VENUES: [&str; 8] = [
    "binance-spot",
    "binance-linear-usdtm",
    "bybit-v5",
    "okx-v5",
    "kraken-spot",
    "coinbase-exchange",
    "hyperliquid-dex",
    "base-mainnet-dex",
];

/// The four Grok voice modes (wire ids mirror `GrokPersonaMode::wire_id`).
pub const GROK_VOICES: [&str; 4] =
    ["fun_witty", "unhinged_roast", "regular_balanced", "mairon_sovereign"];

/// The war-room co-founder council.
pub const WAR_ROOM_COUNCIL: [&str; 3] = ["@elonmusk", "@cz_binance", "@JeffBezos"];

/// The chairman handle whose directives steer the simulation.
pub const CHAIRMAN_HANDLE: &str = "0xKIA";

/// `/mairon` — the sovereign overview: brand, laws and the world-state surface.
#[must_use]
pub fn format_mairon() -> String {
    String::from(
        "MAIRON — Sovereign Founder OS\n\
         \n\
         One repository, one kernel, one authoritative world state.\n\
         \n\
         IMPERIUM laws:\n\
         1. Single world state — every perspective reads and writes the same\n\
            kernel-backed store; no orphan mockups.\n\
         2. Lossless perspective switching — ToySpark 64px, Creator Studio,\n\
            Pro Terminal, Quant Desk, Treasury, Neural-HUD and the Founder\n\
            War-Room share one state; switching never resets work.\n\
         3. Zero placeholders — every control is wired to a real /api/v1 route.\n\
         \n\
         Flagship commands:\n\
         /trinity — 3-avatar build pipeline\n\
         /quant — causal CCXT desk\n\
         /treasury — Base-8453 settlement\n\
         /warroom — 24-tick founder simulation\n\
         /hud — neural topology + Grok voices",
    )
}

/// `/trinity` — the 3-avatar build pipeline with the REA gate.
#[must_use]
pub fn format_trinity() -> String {
    format!(
        "TRINITY — 3-Avatar Sovereign Build Pipeline\n\
         \n\
         Stage 1  Alpha IQ {TRINITY_IQ_ALPHA}      — master prompt + project scaffold.\n\
         Stage 2  CommandCenter IQ {TRINITY_IQ_COMMAND_CENTER} — orchestrator, judicial strictness on.\n\
         Stage 3  Mairon IQ {TRINITY_IQ_MAIRON}      — overclocked judge, final calibration.\n\
         \n\
         Quality gate: every build passes the REA review with score >= {REA_MIN_SCORE}/100\n\
         before it may ship; two-key publish is enforced for store release.\n\
         \n\
         Try it live: Pro Terminal -> Trinity pipeline (Ctrl+K), or the\n\
         ToySpark tab of the MAIRON mini-app."
    )
}

/// `/quant` — the causal CCXT desk and the live-deployment gate.
#[must_use]
pub fn format_quant() -> String {
    let numbered: Vec<String> = KEYLESS_VENUES
        .iter()
        .enumerate()
        .map(|(i, v)| format!("  {}. {v}", i + 1))
        .collect();
    let venues = numbered.join("\n");
    format!(
        "QUANT — Causal Vibe-Trading & CCXT Desk\n\
         \n\
         {venue_count} keyless venues (no API keys, no KYC):\n\
         {venues}\
         \n\
         Backtests are strictly causal — candles[0..=t] only, open(t+1) fills.\n\
         Live-deployment gate: Sharpe >= {sharpe}, drawdown <= 20%, >= 5 trades.\n\
         Every published claim carries a SHA-256 claim hash anyone can verify.\n\
         \n\
         Desk: Pro Terminal -> Quant Desk; mini-app -> Vibe-Trading tab.",
        venue_count = KEYLESS_VENUES.len(),
        venues = venues,
        sharpe = MAIRON_MIN_SHARPE,
    )
}

/// `/treasury` — Base-8453 settlement, the 4-stream split and the reserve.
#[must_use]
pub fn format_treasury() -> String {
    format!(
        "TREASURY — Base-8453 Sovereign Settlement\n\
         \n\
         Chain: Base Mainnet (chain id {chain})\n\
         USDC: {usdc}\n\
         Token: {token}   In-app rail: Telegram Stars ({stars})\n\
         \n\
         4-Stream Revenue Split:\n\
         - Unreferred sale:  20% platform / 80% creator\n\
         - Referred sale:    10% referrer / 80% creator / 10% platform\n\
         - Instant Boosts:   100% MAIRON Sovereign Treasury / 0% Creator\n\
         - Swap spread:      1.5% treasury fee\n\
         \n\
         Capital Reserve Partition:\n\
         - Operational crypto seed: ${crypto_seed}\n\
         - Cash float:              ${cash}\n\
         - Locked cold reserve:     ${locked}\n\
         \n\
         Console: Founder War-Room -> Treasury view (Base-8453).",
        chain = BASE_CHAIN_ID,
        usdc = BASE_USDC_CONTRACT,
        token = KIA_TOKEN_SYMBOL,
        stars = TELEGRAM_STARS_CURRENCY,
        crypto_seed = format_cents(CAPITAL_RESERVE_CRYPTO_SEED_CENTS),
        cash = format_cents(CAPITAL_RESERVE_CASH_CENTS),
        locked = format_cents(CAPITAL_RESERVE_LOCKED_CENTS),
    )
}

/// `/warroom` — the 24-tick founder simulation and the chairman channel.
#[must_use]
pub fn format_warroom() -> String {
    let council = WAR_ROOM_COUNCIL.join(", ");
    format!(
        "WAR-ROOM — Founder Command Simulation\n\
         \n\
         24 hourly causal ticks (0..23) drive one live scenario: revenue\n\
         climbs from a ${delta} day-1 gain to a ${total} wallet total across\n\
         Base USDC and Telegram Stars.\n\
         \n\
         Co-founder council: {council}\n\
         Chairman channel: directives from {chairman} steer the simulation\n\
         through /api/v1/war-room/directive; /war-room/step advances one tick.\n\
         \n\
         Dashboard: the kernel serves the full war-room at /war-room.",
        delta = format_micros(WAR_ROOM_DAY1_DELTA_MICROS),
        total = format_micros(WAR_ROOM_FINAL_TOTAL_MICROS),
        council = council,
        chairman = CHAIRMAN_HANDLE,
    )
}

/// `/hud` — the neural topology and the Grok voice modes.
#[must_use]
pub fn format_hud() -> String {
    let voices = GROK_VOICES.join(", ");
    format!(
        "HUD — Neural Topology & Grok Voice Lab\n\
         \n\
         Fibonacci-sphere topology: {NEURAL_HUD_NODE_COUNT} nodes on a dual shell (inner\n\
         cognition ring, outer telemetry ring), rendered live by the kernel\n\
         at /neural-hud.\n\
         \n\
         10-DNA telemetry: every lab turn is logged per DNA cohort with REA\n\
         score, latency and training usability; exports emit SFT/DPO JSONL.\n\
         \n\
         Grok voice modes: {voices}\n\
         \n\
         Mini-app: Founder War-Room tab -> Grok Voice HUD."
    )
}

/// Map one `/cmd` (normalized, lowercase, no leading slash) to its formatter.
#[must_use]
pub fn format_sovereign_command(cmd: &str) -> Option<String> {
    match cmd {
        "mairon" => Some(format_mairon()),
        "trinity" => Some(format_trinity()),
        "quant" => Some(format_quant()),
        "treasury" => Some(format_treasury()),
        "warroom" => Some(format_warroom()),
        "hud" => Some(format_hud()),
        _ => None,
    }
}

/// True when this normalized command is served by the sovereign suite.
#[must_use]
pub fn is_sovereign_command(cmd: &str) -> bool {
    matches!(
        cmd,
        "mairon" | "trinity" | "quant" | "treasury" | "warroom" | "hud"
    )
}

/// Format a dollar amount from cents — e.g. `3_300` -> `33.00`.
fn format_cents(cents: i64) -> String {
    let whole = cents / 100;
    let frac = cents % 100;
    format!("{whole}.{frac:02}")
}

/// Format a dollar amount from USDC micros — e.g. `9_550_000` -> `9.55`.
fn format_micros(micros: i64) -> String {
    let cents = micros / 10_000;
    format_cents(cents)
}

// ── tests — pure, offline, deterministic ──────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// The USDC address constant matches the canonical Base Mainnet USDC.
    #[test]
    fn usdc_address_is_canonical() {
        assert_eq!(BASE_USDC_CONTRACT.len(), 42);
        assert!(BASE_USDC_CONTRACT.starts_with("0x8335"));
        assert!(BASE_USDC_CONTRACT.ends_with("2913"));
        assert!(BASE_USDC_CONTRACT.chars().skip(2).all(|c| c.is_ascii_hexdigit()));
    }

    /// Trinity tiers and gates match the kernel calibration.
    #[test]
    fn trinity_constants_are_locked() {
        assert_eq!(TRINITY_IQ_ALPHA, 136);
        assert_eq!(TRINITY_IQ_COMMAND_CENTER, 139);
        assert_eq!(TRINITY_IQ_MAIRON, 150);
        assert_eq!(REA_MIN_SCORE, 85);
    }

    /// Exactly 8 keyless venues, all unique lowercase ids.
    #[test]
    fn keyless_venue_matrix_is_exact() {
        assert_eq!(KEYLESS_VENUES.len(), 8);
        let unique: std::collections::HashSet<&str> = KEYLESS_VENUES.iter().copied().collect();
        assert_eq!(unique.len(), 8);
        for v in KEYLESS_VENUES {
            assert!(v.chars().all(|c| c.is_ascii_lowercase() || c == '-' || c.is_ascii_digit()), "{v}");
        }
    }

    /// The four Grok voices mirror the kernel wire ids.
    #[test]
    fn grok_voices_mirror_kernel_wire_ids() {
        assert_eq!(GROK_VOICES.len(), 4);
        assert!(GROK_VOICES.contains(&"mairon_sovereign"));
        assert!(GROK_VOICES.contains(&"fun_witty"));
        assert!(GROK_VOICES.contains(&"unhinged_roast"));
        assert!(GROK_VOICES.contains(&"regular_balanced"));
    }

    /// Every sovereign command routes to a formatter.
    #[test]
    fn all_sovereign_commands_route() {
        for cmd in ["mairon", "trinity", "quant", "treasury", "warroom", "hud"] {
            assert!(is_sovereign_command(cmd), "{cmd} must be sovereign");
            let text = format_sovereign_command(cmd).expect("formatter exists");
            assert!(!text.trim().is_empty(), "{cmd} formatter must be non-empty");
        }
        assert!(!is_sovereign_command("nope"));
        assert_eq!(format_sovereign_command("nope"), None);
        assert_eq!(format_sovereign_command(""), None);
    }

    /// `/mairon` states the three IMPERIUM laws in English.
    #[test]
    fn mairon_states_imperium_laws() {
        let t = format_mairon();
        assert!(t.contains("MAIRON"));
        assert!(t.contains("IMPERIUM"));
        assert!(t.contains("Single world state"));
        assert!(t.contains("Lossless perspective switching"));
        assert!(t.contains("Zero placeholders"));
    }

    /// `/trinity` prints the three IQ tiers and the REA gate.
    #[test]
    fn trinity_prints_tiers_and_gate() {
        let t = format_trinity();
        assert!(t.contains("136"), "alpha tier");
        assert!(t.contains("139"), "command-center tier");
        assert!(t.contains("150"), "mairon tier");
        assert!(t.contains("85"), "REA gate");
        assert!(t.contains("two-key publish"));
    }

    /// `/quant` lists all 8 venues and the Sharpe gate.
    #[test]
    fn quant_lists_venues_and_sharpe_gate() {
        let t = format_quant();
        for v in KEYLESS_VENUES {
            assert!(t.contains(v), "missing venue {v}");
        }
        assert!(t.contains("1.2"), "Sharpe gate threshold");
        assert!(t.contains("candles[0..=t]"), "causal discipline");
        assert!(t.contains("SHA-256"), "claim hashes");
    }

    /// `/treasury` carries the Base-8453 constants, the 4-stream split and the reserve.
    #[test]
    fn treasury_prints_split_and_reserve() {
        let t = format_treasury();
        assert!(t.contains(BASE_USDC_CONTRACT));
        assert!(t.contains("8453"));
        assert!(t.contains(KIA_TOKEN_SYMBOL));
        assert!(t.contains("XTR"));
        assert!(t.contains("20% platform / 80% creator"));
        assert!(t.contains("10% referrer / 80% creator / 10% platform"));
        assert!(t.contains("100% MAIRON Sovereign Treasury / 0% Creator"));
        assert!(t.contains("1.5% treasury fee"));
        assert!(t.contains("33.00"));
        assert!(t.contains("15.00"));
        assert!(t.contains("350.00"));
    }

    /// `/warroom` prints the tick count, council, chairman and revenue arc.
    #[test]
    fn warroom_prints_simulation_facts() {
        let t = format_warroom();
        assert!(t.contains("24 hourly causal ticks"));
        assert!(t.contains("9.55"));
        assert!(t.contains("42.55"));
        for m in WAR_ROOM_COUNCIL {
            assert!(t.contains(m), "missing council member {m}");
        }
        assert!(t.contains("0xKIA"));
        assert!(t.contains("/api/v1/war-room/directive"));
    }

    /// `/hud` prints the node count and every voice mode.
    #[test]
    fn hud_prints_topology_and_voices() {
        let t = format_hud();
        assert!(t.contains("180"));
        assert!(t.contains("10-DNA"));
        assert!(t.contains("/neural-hud"));
        for v in GROK_VOICES {
            assert!(t.contains(v), "missing voice {v}");
        }
    }

    /// Dollar formatting from cents and micros is exact and locale-neutral.
    #[test]
    fn money_formatting_is_exact() {
        assert_eq!(format_cents(3_300), "33.00");
        assert_eq!(format_cents(1_500), "15.00");
        assert_eq!(format_cents(35_000), "350.00");
        assert_eq!(format_cents(0), "0.00");
        assert_eq!(format_micros(9_550_000), "9.55");
        assert_eq!(format_micros(42_550_000), "42.55");
    }

    /// No formatter ever emits a secret-looking token (`ABCTOKEN` sentinel).
    #[test]
    fn formatters_never_leak_secrets() {
        const SENTINEL: &str = "123456:ABC-TOKEN-SHOULD-NEVER-LEAK";
        for cmd in ["mairon", "trinity", "quant", "treasury", "warroom", "hud"] {
            let t = format_sovereign_command(cmd).unwrap_or_default();
            assert!(!t.contains(SENTINEL));
            assert!(!t.contains("ABC-TOKEN"));
        }
    }

    /// Every formatted reply fits the Telegram hard cap (4096 chars).
    #[test]
    fn formatters_fit_telegram_cap() {
        for cmd in ["mairon", "trinity", "quant", "treasury", "warroom", "hud"] {
            let t = format_sovereign_command(cmd).unwrap_or_default();
            assert!(t.chars().count() <= 4096, "{cmd} exceeds the cap");
        }
    }

    /// War-room revenue math is internally consistent: 33.00 + 9.55 == 42.55.
    #[test]
    fn warroom_revenue_math_is_consistent() {
        let seed_cents = CAPITAL_RESERVE_CRYPTO_SEED_CENTS;
        let delta_cents = WAR_ROOM_DAY1_DELTA_MICROS / 10_000;
        let final_cents = WAR_ROOM_FINAL_TOTAL_MICROS / 10_000;
        assert_eq!(seed_cents + delta_cents, final_cents);
    }

    /// Pure English: no formatter output contains Arabic-script codepoints.
    #[test]
    fn formatters_are_pure_english() {
        for cmd in ["mairon", "trinity", "quant", "treasury", "warroom", "hud"] {
            let t = format_sovereign_command(cmd).unwrap_or_default();
            assert!(
                !t.chars().any(|c| ('\u{0600}'..='\u{06FF}').contains(&c)),
                "{cmd} must be pure English"
            );
        }
    }
}
