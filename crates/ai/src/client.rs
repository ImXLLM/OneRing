//! Model clients — production `OpenAI`-compatible HTTP + a scripted test double.
//!
//! [`HttpModelClient`] only calls `{url}/chat/completions` with a Bearer
//! and parses the response strictly (`choices[0].message.content` +
//! `usage.*`). Error details are a short secret-free string — the key and request text never
//! reach the detail.

use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::Duration;

use crate::gateway::{ChatPacket, CompletionOutcome, CompletionPacket, ModelClient, SlotConfig};

/// `HTTP-Referer` header — app identity on attribution-centric routers (`OpenRouter`).
pub const SITE_URL: &str = "https://kian.app";

/// `X-Title` header — app name on attribution-centric routers (`OpenRouter`).
pub const SITE_TITLE: &str = "KIAN Studio";

/// Production HTTP client — rustls + webpki-roots, no system openssl.
#[derive(Debug, Clone)]
pub struct HttpModelClient {
    /// reqwest client with a fixed timeout
    http: reqwest::Client,
}

impl HttpModelClient {
    /// Build with an explicit timeout (suggested: 30 seconds).
    ///
    /// # Errors
    /// reqwest client construction can fail (TLS backend).
    pub fn new(timeout: Duration) -> Result<Self, String> {
        let http = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| format!("http client build: {e}"))?;
        Ok(Self { http })
    }
}

#[async_trait::async_trait]
impl ModelClient for HttpModelClient {
    async fn complete(&self, slot: &SlotConfig, packet: &ChatPacket) -> CompletionOutcome {
        let url = format!("{}/chat/completions", slot.url.trim_end_matches('/'));
        let mut body = serde_json::json!({
            "model": packet.model,
            "messages": packet
                .messages
                .iter()
                .map(|m| serde_json::json!({"role": m.role, "content": m.content}))
                .collect::<Vec<_>>(),
        });
        if let Some(max_tokens) = packet.max_tokens
            && let Some(obj) = body.as_object_mut()
        {
            obj.insert("max_tokens".into(), serde_json::json!(max_tokens));
        }
        let resp = self
            .http
            .post(url)
            .bearer_auth(&slot.key)
            .header("HTTP-Referer", SITE_URL)
            .header("X-Title", SITE_TITLE)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("http: {e}"))?;
        let status = resp.status();
        if !status.is_success() {
            // status code only — the upstream body may contain user content
            return Err(format!("upstream status {status}"));
        }
        let json: serde_json::Value =
            resp.json().await.map_err(|e| format!("body parse: {e}"))?;
        let content = json
            .pointer("/choices/0/message/content")
            .and_then(serde_json::Value::as_str)
            .ok_or("malformed: missing choices[0].message.content")?;
        let prompt_tokens = json
            .pointer("/usage/prompt_tokens")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(0);
        let completion_tokens = json
            .pointer("/usage/completion_tokens")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(0);
        Ok(CompletionPacket {
            content: content.to_owned(),
            prompt_tokens,
            completion_tokens,
        })
    }
}


/// Always-rejecting client — when the HTTP client failed to boot; the gateway honestly reports Disabled.
#[derive(Debug, Default)]
pub struct UnavailableClient;

#[async_trait::async_trait]
impl ModelClient for UnavailableClient {
    async fn complete(&self, _slot: &SlotConfig, _packet: &ChatPacket) -> CompletionOutcome {
        Err("http client unavailable at boot".into())
    }
}

/// Test double — a queue of canned responses; each `complete` consumes one item and
/// records the attempt order (to prove fallback).
#[derive(Debug, Default)]
pub struct ScriptedClient {
    script: Mutex<VecDeque<Result<CompletionPacket, String>>>,
    calls: Mutex<Vec<String>>,
}

impl ScriptedClient {
    /// Build with the response queue in order.
    #[must_use]
    pub fn new(script: Vec<Result<CompletionPacket, String>>) -> Self {
        Self { script: Mutex::new(script.into()), calls: Mutex::new(Vec::new()) }
    }

    /// Slot names called so far — to prove the fallback order.
    #[must_use]
    pub fn called_slots(&self) -> Vec<String> {
        self.calls.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone()
    }
}

#[async_trait::async_trait]
impl ModelClient for ScriptedClient {
    async fn complete(
        &self,
        slot: &SlotConfig,
        _packet: &ChatPacket,
    ) -> Result<CompletionPacket, String> {
        self.calls
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(slot.name.clone());
        self.script
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .pop_front()
            .unwrap_or_else(|| Err("script exhausted".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gateway::PacketMessage;

    /// The attribution and Bearer headers must be present on the wire — proven with a local
    /// loopback server (no external network, no real secrets).
    #[tokio::test]
    async fn http_client_sends_attribution_headers() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("loopback bind");
        let addr = listener.local_addr().expect("local addr");
        let server = tokio::spawn(async move {
            let (mut sock, _) = listener.accept().await.expect("accept");
            let mut buf = vec![0u8; 8192];
            let mut read = 0usize;
            loop {
                let n = sock.read(&mut buf[read..]).await.expect("read");
                if n == 0 {
                    break;
                }
                read += n;
                if buf[..read].windows(4).any(|w| w == b"\r\n\r\n") {
                    break; // end of headers — the JSON body does not matter
                }
            }
            let head = String::from_utf8_lossy(&buf[..read]).to_lowercase();
            assert!(
                head.contains("http-referer: https://kian.app"),
                "HTTP-Referer must be sent: {head}"
            );
            assert!(head.contains("x-title: kian studio"), "X-Title must be sent: {head}");
            assert!(
                head.contains("authorization: bearer test-key"),
                "Bearer key must be sent: {head}"
            );
            let payload = serde_json::json!({
                "choices": [{"message": {"content": "\u{633}\u{644}\u{627}\u{645}"}}],
                "usage": {"prompt_tokens": 1, "completion_tokens": 2}
            })
            .to_string();
            let resp = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                payload.len(),
                payload
            );
            sock.write_all(resp.as_bytes()).await.expect("write");
            sock.shutdown().await.expect("shutdown");
        });

        let client = HttpModelClient::new(Duration::from_secs(5)).expect("client");
        let slot = SlotConfig {
            name: "primary".into(),
            url: format!("http://{addr}"),
            key: "test-key".into(),
            model: "m".into(),
        };
        let packet = ChatPacket {
            model: "m".into(),
            messages: vec![PacketMessage { role: "user".into(), content: "\u{633}\u{644}\u{627}\u{645}".into() }],
            max_tokens: None,
        };
        let out = client.complete(&slot, &packet).await.expect("loopback must answer");
        assert_eq!(out.content, "\u{633}\u{644}\u{627}\u{645}");
        assert_eq!((out.prompt_tokens, out.completion_tokens), (1, 2));
        server.await.expect("server task");
    }

    /// HTTP client with a bad URL only — short error, no secrets.
    #[tokio::test]
    async fn http_client_error_is_short_and_secret_free() {
        let client = HttpModelClient::new(Duration::from_secs(2)).expect("client");
        let slot = SlotConfig {
            name: "primary".into(),
            url: "http://127.0.0.1:1".into(), // closed port — the connection is refused immediately
            key: "super-secret-key".into(),
            model: "m".into(),
        };
        let packet = ChatPacket {
            model: "m".into(),
            messages: vec![PacketMessage { role: "user".into(), content: "\u{633}\u{644}\u{627}\u{645}".into() }],
            max_tokens: None,
        };
        let err = client.complete(&slot, &packet).await.unwrap_err();
        assert!(!err.contains("super-secret"), "key must never leak into detail: {err}");
        assert!(!err.contains("\u{633}\u{644}\u{627}\u{645}"), "user content must never leak: {err}");
        assert!(err.len() < 200, "detail must stay short");
    }
}
