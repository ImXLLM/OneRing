//! کلاینت‌های مدل — HTTP تولیدی سازگار با `OpenAI` + دابل تست اسکریپت‌شو.
//!
//! [`HttpModelClient`] فقط به نقطهٔ `{url}/chat/completions` با Bearer زنگ
//! می‌زند و پاسخ را سخت‌گیرانه پارس می‌کند (`choices[0].message.content` +
//! `usage.*`). جزئیات خطا رشتهٔ کوتاه بی‌راز است — کلید و متن درخواست هرگز
//! به detail نمی‌رود.

use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::Duration;

use crate::gateway::{ChatPacket, CompletionOutcome, CompletionPacket, ModelClient, SlotConfig};

/// سرصفحهٔ `HTTP-Referer` — شناسهٔ اپ روی روترهای attribution-محور (`OpenRouter`).
pub const SITE_URL: &str = "https://kian.app";

/// سرصفحهٔ `X-Title` — نام اپ روی روترهای attribution-محور (`OpenRouter`).
pub const SITE_TITLE: &str = "KIAN Studio";

/// کلاینت HTTP تولیدی — rustls + webpki-roots، بدون openssl سیستمی.
#[derive(Debug, Clone)]
pub struct HttpModelClient {
    /// کلاینت reqwest با تایم‌اوت ثابت
    http: reqwest::Client,
}

impl HttpModelClient {
    /// ساخت با تایم‌اوت مشخص (پیشنهاد: ۳۰ ثانیه).
    ///
    /// # Errors
    /// ساخت کلاینت reqwest ممکن است شکست بخورد (TLS backend).
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
            // فقط کد وضعیت — بدنهٔ آپ‌استریم ممکن است محتوای کاربر را برگرداند
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


/// کلاینت همیشه-رد — وقتی کلاینت HTTP بوت نشد؛ گیت‌وی صادقانه Disabled می‌دهد.
#[derive(Debug, Default)]
pub struct UnavailableClient;

#[async_trait::async_trait]
impl ModelClient for UnavailableClient {
    async fn complete(&self, _slot: &SlotConfig, _packet: &ChatPacket) -> CompletionOutcome {
        Err("http client unavailable at boot".into())
    }
}

/// دابل تست — صف پاسخ‌های آماده؛ هر `complete` یک آیتم مصرف می‌کند و
/// ترتیب تلاش‌ها را ضبط می‌کند (برای اثبات فال‌بک).
#[derive(Debug, Default)]
pub struct ScriptedClient {
    script: Mutex<VecDeque<Result<CompletionPacket, String>>>,
    calls: Mutex<Vec<String>>,
}

impl ScriptedClient {
    /// ساخت با صف پاسخ‌ها به ترتیب.
    #[must_use]
    pub fn new(script: Vec<Result<CompletionPacket, String>>) -> Self {
        Self { script: Mutex::new(script.into()), calls: Mutex::new(Vec::new()) }
    }

    /// نام اسلات‌هایی که تا حالا صدا زده شده‌اند — برای اثبات ترتیب فال‌بک.
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

    /// سرصفحه‌های attribution و Bearer باید روی سیم حاضر باشند — با سرور لوپ‌بک
    /// محلی اثبات می‌شود (بدون شبکهٔ بیرونی، بدون راز واقعی).
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
                    break; // پایان سرصفحه‌ها — بدنهٔ JSON مهم نیست
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
                "choices": [{"message": {"content": "سلام"}}],
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
            messages: vec![PacketMessage { role: "user".into(), content: "سلام".into() }],
            max_tokens: None,
        };
        let out = client.complete(&slot, &packet).await.expect("loopback must answer");
        assert_eq!(out.content, "سلام");
        assert_eq!((out.prompt_tokens, out.completion_tokens), (1, 2));
        server.await.expect("server task");
    }

    /// کلاینت HTTP فقط با URL بد — خطای کوتاه، بدون راز.
    #[tokio::test]
    async fn http_client_error_is_short_and_secret_free() {
        let client = HttpModelClient::new(Duration::from_secs(2)).expect("client");
        let slot = SlotConfig {
            name: "primary".into(),
            url: "http://127.0.0.1:1".into(), // پورت بسته — اتصال فوری رد می‌شود
            key: "super-secret-key".into(),
            model: "m".into(),
        };
        let packet = ChatPacket {
            model: "m".into(),
            messages: vec![PacketMessage { role: "user".into(), content: "سلام".into() }],
            max_tokens: None,
        };
        let err = client.complete(&slot, &packet).await.unwrap_err();
        assert!(!err.contains("super-secret"), "key must never leak into detail: {err}");
        assert!(!err.contains("سلام"), "user content must never leak: {err}");
        assert!(err.len() < 200, "detail must stay short");
    }
}
