//! Live upstream test — runs only with env set and is `#[ignore]` by default.
//!
//! Usage:
//! ```sh
//! KIAN_AI_PRIMARY_URL=... KIAN_AI_PRIMARY_KEY=... KIAN_AI_PRIMARY_MODEL=... \
//!   cargo test -p kian-ai --test live_upstream -- --ignored --nocapture
//! ```
//!
//! Contract: the key comes from env and is never printed in log/detail; this file contains no
//! hardcoded value so no secret is ever left in the repository.

use std::time::Duration;

use kian_ai::client::HttpModelClient;
use kian_ai::gateway::{AiConfig, AiGateway, PacketMessage, SlotConfig};

fn slot_from_env(name: &str) -> Option<SlotConfig> {
    let env = |k: &str| std::env::var(k).ok().filter(|v| !v.trim().is_empty());
    let (u, k, m) = (
        env(&format!("KIAN_AI_{name}_URL")),
        env(&format!("KIAN_AI_{name}_KEY")),
        env(&format!("KIAN_AI_{name}_MODEL")),
    );
    match (u, k, m) {
        (Some(url), Some(key), Some(model)) => {
            Some(SlotConfig { name: name.to_lowercase(), url, key, model })
        }
        _ => None,
    }
}

/// Real primary slot — secret from env; without env the test is ignored.
fn live_config() -> Option<AiConfig> {
    let slot = slot_from_env("PRIMARY")?;
    Some(AiConfig { slots: vec![slot], daily_cap_tokens: 50_000 })
}

/// Simple live chat — the packet must be complete (non-empty content + non-negative tokens).
#[tokio::test]
#[ignore = "live network — run explicitly with --ignored and env"]
async fn live_upstream_answers_with_wellformed_packet() {
    let Some(cfg) = live_config() else {
        panic!("KIAN_AI_PRIMARY_URL/KEY/MODEL must be set for the live test");
    };
    let client = HttpModelClient::new(Duration::from_secs(60)).expect("http client");
    let gw = AiGateway::new(cfg.clone(), std::sync::Arc::new(client));

    let res = gw
        .chat(vec![PacketMessage {
            role: "user".into(),
            content: "\u{641}\u{642}\u{637} \u{6a9}\u{644}\u{645}\u{647}\u{654} «\u{633}\u{644}\u{627}\u{645}» \u{631}\u{627} \u{628}\u{646}\u{648}\u{6cc}\u{633}.".into(),
        }])
        .await
        .expect("live slot must answer");

    assert!(!res.content.trim().is_empty(), "content must not be empty");
    assert!(res.prompt_tokens >= 0 && res.completion_tokens >= 0);
    assert!(res.redactions.is_empty(), "clean text must not be redacted");
    println!("provider={} model={}", res.provider, res.model);
    println!(
        "tokens: prompt={} completion={} total={}",
        res.prompt_tokens, res.completion_tokens, res.total_tokens
    );
    println!("content: {}", res.content);
}

/// Input containing a private key — the secret must not reach upstream and the finding must be reported.
#[tokio::test]
#[ignore = "live network — run explicitly with --ignored and env"]
async fn live_upstream_redacts_secret_before_send() {
    let Some(cfg) = live_config() else {
        panic!("KIAN_AI_PRIMARY_URL/KEY/MODEL must be set for the live test");
    };
    let client = HttpModelClient::new(Duration::from_secs(60)).expect("http client");
    let gw = AiGateway::new(cfg, std::sync::Arc::new(client));

    let secret = "0xaaaa1111bbbb2222cccc3333dddd4444eeee5555ffff6666aaaabbbbccccdddd";
    let res = gw
        .chat(vec![PacketMessage {
            role: "user".into(),
            content: format!("\u{6a9}\u{644}\u{6cc}\u{62f} \u{645}\u{646} {secret} \u{627}\u{633}\u{62a} — \u{628}\u{6af}\u{648} \u{641}\u{642}\u{637} «\u{62f}\u{631}\u{6cc}\u{627}\u{641}\u{62a} \u{634}\u{62f}»"),
        }])
        .await
        .expect("live slot must answer");

    assert_eq!(
        res.redactions,
        vec!["private_key_hex".to_owned()],
        "finding must be reported"
    );
    assert!(
        !res.content.to_lowercase().contains("0xaaaa1111"),
        "secret must never echo back"
    );
    println!("redactions={:?} total={}", res.redactions, res.total_tokens);
    println!("content: {}", res.content);
}
