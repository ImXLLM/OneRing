//! `kian-ai` — گیت‌وی AI کیان — مغز M4.
//!
//! سه لایهٔ مستقل و تست‌پذیر:
//! ۱. **redaction** — اسکن قطعی ورودی: کلید خصوصی هگز، توکن بات، PEM، JWT،
//!    عبارت حافظه (mnemonic)، هگز بلند — هیچ رازی به ارائه‌دهندهٔ بیرونی
//!    نمی‌رسد؛ فقط دستهٔ یافته گزارش می‌شود، نه خودِ راز.
//! ۲. **budget** — پاکت سخت بودجه: مصرف امروز + برآورد مهارت ≤ سقف روزانه،
//!    وگرنه ۴۲۹. عدد صحیح، بدون float، بدون سمتِ «حدس».
//! ۳. **fallback** — سه اسلات مدل به ترتیب؛ اسلاتِ بدون کانفیگ حذف می‌شود و
//!    شکست هر اسلات تایپ‌شده ثبت می‌شود. پاسخ پاکتِ خراب = شکست اسلات.
//!
//! این کرِیت هیچ چیز از دیتابیس و HTTP سرور نمی‌داند — بودجه‌ای که خوانده
//! می‌شود عددی است که سرور از دفتر می‌آورد؛ گیت‌وی فقط ریاضی می‌کند.
#![forbid(unsafe_code)]

pub mod budget;
pub mod client;
pub mod gateway;
pub mod redaction;

/// شناسه کرِیت — برای smoke test و لاگ‌ها.
pub const NAME: &str = "kian-ai";

/// نسخه کرِیت — از `Cargo.toml` خوانده می‌شود.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub use budget::{BudgetDecision, BudgetEnvelope};
pub use gateway::{
    AiConfig, AiGateway, AttemptFailure, ChatPacket, ChatResult, CompletionPacket, GatewayError,
    ModelClient, PacketMessage, SlotConfig, BUILDER_MAX_TOKENS, SLOT_NAMES,
};
pub use redaction::{redact, RedactionFinding, RedactionReport};

#[cfg(test)]
mod tests {
    use super::*;

    /// ترکیب سه لایه: راز redact می‌شود، بودجه سخت است، فال‌بک ترتیبی است —
    /// کل قرارداد گیت‌وی در یک سناریوی سرتاسری.
    #[tokio::test]
    async fn gateway_end_to_end_redacts_then_falls_back() {
        // اسلات اول خراب، اسلات دوم سالم — متن حاوی کلید خصوصی
        let scripted = client::ScriptedClient::new(vec![
            Err("slot1 is down".into()), // تلاش ۱ → شکست اسلات primary
            Ok(CompletionPacket {
                content: "پاسخ مغز کیان".into(),
                prompt_tokens: 10,
                completion_tokens: 20,
            }),
        ]);
        let cfg = AiConfig {
            slots: vec![
                SlotConfig {
                    name: "primary".into(),
                    url: "http://slot1".into(),
                    key: "k1".into(),
                    model: "m1".into(),
                },
                SlotConfig {
                    name: "secondary".into(),
                    url: "http://slot2".into(),
                    key: "k2".into(),
                    model: "m2".into(),
                },
            ],
            daily_cap_tokens: 1_000,
        };
        let gw = AiGateway::new(cfg, std::sync::Arc::new(scripted));
        // متن خام (حاوی کلید خصوصی) مستقیم به گیت‌وی می‌رود — redaction کار خودِ
        // گیت‌وی است و یافته‌ها در پاسخ گزارش می‌شوند، نه راز.
        let raw = "کلید من 0xaaaa1111bbbb2222cccc3333dddd4444eeee5555ffff6666aaaabbbbccccdddd است";
        let standalone = redact(raw);
        assert_eq!(standalone.findings.len(), 1, "private key hex must be found");
        assert_eq!(standalone.findings[0].kind, "private_key_hex");
        let packet = vec![PacketMessage { role: "user".into(), content: raw.to_owned() }];
        let res = gw.chat(packet).await.expect("slot 2 must serve");
        assert_eq!(res.provider, "secondary");
        assert_eq!(res.redactions, vec!["private_key_hex".to_owned()]);
        assert_eq!(res.total_tokens, 30);
        assert_ne!(res.content, "", "content must not be empty");
    }

    #[tokio::test]
    async fn multi_turn_chat_with_max_tokens_preserves_html_and_redacts_user() {
        let html_reply = "<!DOCTYPE html><html><head><style>body{color:#0ff}</style></head><body><script>const x=1;</script></body></html>";
        let scripted = client::ScriptedClient::new(vec![Ok(CompletionPacket {
            content: html_reply.into(),
            prompt_tokens: 120,
            completion_tokens: 380,
        })]);
        let cfg = AiConfig {
            slots: vec![SlotConfig {
                name: "primary".into(),
                url: "http://slot1".into(),
                key: "k1".into(),
                model: "m1".into(),
            }],
            daily_cap_tokens: 10_000,
        };
        let gw = AiGateway::new(cfg, std::sync::Arc::new(scripted));
        let msgs = vec![
            PacketMessage {
                role: "system".into(),
                content: "build an interactive single page html web application with clean dark neon layout".into(),
            },
            PacketMessage {
                role: "assistant".into(),
                content: html_reply.into(),
            },
            PacketMessage {
                role: "user".into(),
                content: "رنگ دکمه را سبز کن و این توکن 123456789:AAHdqTcvbCHB1WUqptGffZfjHlXgMbGfRcE را نگذار".into(),
            },
        ];
        let res = gw
            .chat_with_max_tokens(msgs, Some(BUILDER_MAX_TOKENS))
            .await
            .expect("builder turn");
        assert_eq!(res.content, html_reply, "HTML/JS/CSS output must remain intact");
        assert_eq!(res.redactions, vec!["bot_token".to_owned()]);
        assert_eq!(res.total_tokens, 500);
    }
}
