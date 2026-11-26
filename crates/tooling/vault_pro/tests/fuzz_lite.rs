//! Phase 8 fuzz-lite — "cargo-fuzz on the input parsers (at minimum: the AI contracts and
//! the validators)" (ADR-009 §7 phase 8) in deterministic in-CI form.
//!
//! The seeded mutation engine (xorshift — dependency-free) corrupts valid inputs with
//! thousands of mutations; boundary parsers must **always** answer fail-closed
//! (a typed error or 400/422) — every panic is a real fuzz finding.
//!
//! The cargo-fuzz skeleton (libfuzzer on nightly) lives in `fuzz/` next to this crate
//! and is built in CI with `cargo +nightly fuzz build` — the same targets.

// fuzz — unwrap/expect allowed in tests
#![allow(clippy::unwrap_used, clippy::expect_used)]

use axum::body::Body;
use axum::http::Request;
use axum::Router;
use dropagentx_contracts::{AiChatRequest, CreateEscrowRequest, MintRequest};
use dropagentx_db::V3Store;
use dropagentx_server::{build_router, AppState, ServerConfig};
use tower::ServiceExt;

const SECRET: &str = "fuzz-webhook-secret-012345";

// ── Seeded mutation engine ──────────────────────────────────────────────────

/// xorshift64 — deterministic and dependency-free.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    /// Mutate a byte array — byte delete/insert/replace + truncation.
    fn mutate(bytes: &mut Vec<u8>, rng: &mut Rng) {
        if bytes.is_empty() {
            bytes.push(b'x');
            return;
        }
        match rng.next() % 4 {
            0 => {
                let i = (rng.next() as usize) % bytes.len();
                bytes[i] = (rng.next() & 0xff) as u8;
            }
            1 => {
                let i = (rng.next() as usize) % bytes.len();
                bytes.insert(i, (rng.next() & 0xff) as u8);
            }
            2 => {
                let i = (rng.next() as usize) % bytes.len();
                bytes.remove(i);
            }
            _ => {
                let keep = (rng.next() as usize) % bytes.len();
                bytes.truncate(keep);
            }
        }
    }
}

/// Run N mutations on the seed and ensure the parser is fail-closed — never panics.
fn fuzz_parse<T, E>(seed: u64, iterations: usize, parse: impl Fn(&[u8]) -> Result<T, E>)
where
    T: std::fmt::Debug,
    E: std::fmt::Debug,
{
    let mut rng = Rng(seed);
    let mut corpus: Vec<u8> = seed_body(seed).into_bytes();
    for _ in 0..iterations {
        Rng::mutate(&mut corpus, &mut rng);
        // contract: every input either parses or yields a typed error — panics forbidden
        let _ = parse(&corpus);
    }
}

fn seed_body(seed: u64) -> String {
    match seed % 4 {
        0 => r#"{"accountId":"user:tg:1","amount":100}"#.into(),
        1 => r#"{"buyerId":"user:tg:1","sellerId":"user:tg:2","amount":100}"#.into(),
        2 => r#"{"userId":"user:tg:1","text":"Hello","role":"user"}"#.into(),
        _ => r#"{"id":"user:tg:1","label":"Ali"}"#.into(),
    }
}

// ── Target parsers — the same contract boundaries ──────────────────────────────────────

fn parse_mint(bytes: &[u8]) -> Result<MintRequest, String> {
    serde_json::from_slice::<MintRequest>(bytes).map_err(|e| format!("mint: {e}"))
}

fn parse_escrow(bytes: &[u8]) -> Result<CreateEscrowRequest, String> {
    serde_json::from_slice::<CreateEscrowRequest>(bytes).map_err(|e| format!("escrow: {e}"))
}

fn parse_ai_chat(bytes: &[u8]) -> Result<AiChatRequest, String> {
    serde_json::from_slice::<AiChatRequest>(bytes).map_err(|e| format!("ai: {e}"))
}

#[test]
fn fuzz_mint_parser_never_panics() {
    for seed in [0xdead_beef, 1, 2, 3] {
        fuzz_parse(seed, 2_000, parse_mint);
    }
}

#[test]
fn fuzz_escrow_parser_never_panics() {
    for seed in [0xcafe_babe, 4, 5, 6] {
        fuzz_parse(seed, 2_000, parse_escrow);
    }
}

#[test]
fn fuzz_ai_chat_parser_never_panics() {
    for seed in [0xfeed_face, 7, 8, 9] {
        fuzz_parse(seed, 2_000, parse_ai_chat);
    }
}

/// HTTP edge fuzz — corrupt webhook bodies never yield 5xx from the bot.
#[tokio::test]
async fn fuzz_bot_webhook_body_never_5xx() {
    let mut rng = Rng(0x5eed_0001);
    let mut corpus: Vec<u8> =
        br#"{"update_id":1,"message":{"message_id":1,"from":{"id":7,"first_name":"K"},"chat":{"id":7,"type":"private"},"text":"/start"}}"#.to_vec();
    let (router, _store) = boot().await;
    for _ in 0..300 {
        Rng::mutate(&mut corpus, &mut rng);
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/bot/webhook")
            .header("x-telegram-bot-api-secret-token", SECRET)
            .body(Body::from(corpus.clone()))
            .unwrap();
        let res = router.clone().oneshot(req).await.unwrap();
        assert!(
            res.status().as_u16() < 500,
            "mutated body caused 5xx: {} — body {:?}",
            res.status(),
            String::from_utf8_lossy(&corpus)
        );
    }
}

async fn boot() -> (Router, V3Store) {
    let cfg = ServerConfig {
        admin_key: Some("fuzz-admin-key-0123456789".into()),
        database_url: "sqlite::memory:".into(),
        static_dir: None,
        bot_webhook_secret: Some(SECRET.into()),
        bot_admin_ids: vec![42],
        ..ServerConfig::default()
    };
    let store = V3Store::connect(&cfg.database_url).await.unwrap();
    store.migrate().await.unwrap();
    store.ensure_system_accounts().await.unwrap();
    let state = AppState::new(store.clone(), cfg).shared();
    (build_router(state), store)
}

/// Boundary parsers always return Result on corrupt input — never panic (sample check)
#[test]
fn parsers_return_typed_errors_on_garbage() {
    assert!(parse_mint(b"not json").is_err());
    assert!(parse_escrow(b"[]").is_err());
    assert!(parse_ai_chat(b"{}").is_err());
}
