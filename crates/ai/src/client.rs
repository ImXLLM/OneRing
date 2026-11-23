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
