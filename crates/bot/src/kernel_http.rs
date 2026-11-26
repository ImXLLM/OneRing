//! The real HTTP implementation of [`KernelPort`] — the kernel client for the polling shell.
//!
//! * `POST /api/v1/auth/telegram` with signed initData — **the same official
//!   mini-app identity path**; no side path exists.
//! * `GET /api/v1/me` with Bearer — balance/locks.
//! * `GET /api/v1/economy` — public (article 6).
//! * The bot token is used only to sign initData; it never appears in logs/URLs.

use std::time::Duration;

use async_trait::async_trait;
use kian_contracts::build_bot_init_data;
use serde::Deserialize;
use serde_json::json;

use crate::port::{BotEconomy, BotError, BotMe, KernelPort};

/// Kernel client — holds the token for signing initData.
pub struct KernelHttpClient {
    http: reqwest::Client,
    /// Kernel base — e.g. `http://127.0.0.1:8088`.
    base_url: String,
    /// Bot token — signs initData only; never logged.
    bot_token: String,
}

#[derive(Deserialize)]
struct AuthWire {
    token: String,
}

#[derive(Deserialize)]
struct MeWire {
    user: MeUserWire,
    #[serde(rename = "escrowLocked")]
    escrow_locked: i64,
    balance: i64,
}

#[derive(Deserialize)]
struct MeUserWire {
    username: Option<String>,
    #[serde(rename = "accountId")]
    account_id: String,
    #[serde(rename = "tonAddress")]
    ton_address: Option<String>,
}

#[derive(Deserialize)]
struct EconomyWire {
    #[serde(rename = "circulatingSupply")]
    circulating_supply: i64,
    #[serde(rename = "treasuryBalance")]
    treasury_balance: i64,
    #[serde(rename = "escrowLocked")]
    escrow_locked: i64,
    #[serde(rename = "mintedAllTime")]
    minted_all_time: i64,
    #[serde(rename = "burnedAllTime")]
    burned_all_time: i64,
    #[serde(rename = "accountCount")]
    account_count: i64,
    #[serde(rename = "openEscrows")]
    open_escrows: i64,
}

impl KernelHttpClient {
    /// Build — 15-second timeout; rustls (webpki-roots).
    ///
    /// # Errors
    /// TLS client construction can fail.
    pub fn new(base_url: &str, bot_token: &str) -> reqwest::Result<Self> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent("kian-bot/0.1 (+kernel client)")
            .build()?;
        Ok(Self { http, base_url: base_url.trim_end_matches('/').to_owned(), bot_token: bot_token.to_owned() })
    }

    /// Map a kernel response to [`BotError`] — the raw kernel message comes through honestly but
    /// never contains a token/key (the M2 error envelope contract).
    async fn map_err(res: reqwest::Response) -> BotError {
        let body: serde_json::Value = res.json().await.unwrap_or(serde_json::json!({}));
        let err = &body["error"];
        BotError::Kernel {
            code: err["code"].as_str().unwrap_or("HTTP_ERROR").to_owned(),
            message: err["message"].as_str().unwrap_or("the kernel returned an invalid response").to_owned(),
        }
    }
}

#[async_trait]
impl KernelPort for KernelHttpClient {
    async fn economy(&self) -> Result<BotEconomy, BotError> {
        let url = format!("{}/api/v1/economy", self.base_url);
        let res = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| BotError::Transport(e.to_string()))?;
        if !res.status().is_success() {
            return Err(Self::map_err(res).await);
        }
        let wire: EconomyWire =
            res.json().await.map_err(|e| BotError::Transport(e.to_string()))?;
        Ok(BotEconomy {
            circulating: wire.circulating_supply,
            treasury: wire.treasury_balance,
            escrow_locked: wire.escrow_locked,
            minted_all: wire.minted_all_time,
            burned_all: wire.burned_all_time,
            accounts: wire.account_count,
            open_escrows: wire.open_escrows,
        })
    }

    async fn me_via_session(
        &self,
        telegram_id: i64,
        username: Option<&str>,
        now_secs: i64,
    ) -> Result<BotMe, BotError> {
        // 1) Sign initData from the trusted channel (the bot holds the token)
        let init_data =
            build_bot_init_data(&self.bot_token, telegram_id, username, now_secs);
        // 2) Session via the same official mini-app path
        let auth_url = format!("{}/api/v1/auth/telegram", self.base_url);
        let res = self
            .http
            .post(&auth_url)
            .json(&json!({ "initData": init_data }))
            .send()
            .await
            .map_err(|e| BotError::Transport(e.to_string()))?;
        if !res.status().is_success() {
            return Err(Self::map_err(res).await);
        }
        let auth: AuthWire = res.json().await.map_err(|e| BotError::Transport(e.to_string()))?;
        // 3) The me view with Bearer
        let me_url = format!("{}/api/v1/me", self.base_url);
        let res = self
            .http
            .get(&me_url)
            .bearer_auth(&auth.token)
            .send()
            .await
            .map_err(|e| BotError::Transport(e.to_string()))?;
        if !res.status().is_success() {
            return Err(Self::map_err(res).await);
        }
        let wire: MeWire = res.json().await.map_err(|e| BotError::Transport(e.to_string()))?;
        Ok(BotMe {
            username: wire.user.username,
            account_id: wire.user.account_id,
            ton_address: wire.user.ton_address,
            balance: wire.balance,
            escrow_locked: wire.escrow_locked,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The client-signed initData is verified per the official algorithm —
    /// identity has one path, not two.
    #[test]
    fn minted_init_data_is_official_scheme() {
        let client = KernelHttpClient::new("http://127.0.0.1:8088/", "7000000000:TEST").expect("client");
        let data = build_bot_init_data(&client.bot_token, 4242, Some("kia"), 1_760_000_000);
        let identity =
            kian_contracts::verify_telegram_init_data_at(&data, "7000000000:TEST", 1_760_000_000)
                .expect("verifies");
        assert_eq!(identity.user_id, "4242");
        assert_eq!(identity.username.as_deref(), Some("kia"));
        // normalized base_url — no trailing slash
        assert_eq!(client.base_url, "http://127.0.0.1:8088");
    }

    /// Error mapping — the kernel code comes through; the user text is non-technical and secret-free.
    #[test]
    fn bot_error_user_text_is_english_and_secret_free() {
        let e = BotError::Kernel { code: "IDENTITY_DISABLED".to_owned(), message: "x".to_owned() };
        assert!(e.user_text().contains("disabled"));
        let t = BotError::Transport("boom".to_owned());
        assert!(t.user_text().contains("not reachable"));
        assert!(!t.user_text().contains("boom"));
        let s = BotError::Kernel { code: "SESSION_EXPIRED".to_owned(), message: "y".to_owned() };
        assert!(s.user_text().contains("expired"));
    }
}
