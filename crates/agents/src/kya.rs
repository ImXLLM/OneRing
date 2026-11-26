//! KYA metadata and x402 pricing — port of `04_0xkiabot_security_x402_kya`
//! (vault folder 04).
//!
//! * [`KyaMetadata`] — the Know-Your-Agent identity card (ERC-8004 style): every
//!   active agent is described on the wire with an ES256 capability document and its allowed roles;
//!   this structure underlies `/.well-known/agent-card.json` and the ERC-8004 registry.
//! * [`X402Pricing`] — the HTTP-402 payment gate (version 2): pricing in
//!   SBC on the Radius testnet, never reading a private key — only the
//!   seller's public address (`payTo`).
//!
//! Both structures are data-only (serde) — signing and counting on real paths
//! go on the wire later; here is the deterministic, tested contract.
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Accepted signature algorithm — per the KYA spec, ES256 only (the P-256 curve).
pub const KYA_SUPPORTED_JWA: &str = "ES256";

/// x402 network ID — Radius Testnet (the Python `NETWORK`).
pub const X402_NETWORK: &str = "eip155:72344";

/// x402 protocol version (the Python `X402_VERSION`).
pub const X402_VERSION: u32 = 2;

/// The Know-Your-Agent identity card of a fleet agent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KyaMetadata {
    /// Agent ID (`agent:merchant` …)
    pub agent_id: String,
    /// Fleet role
    pub role: String,
    /// ERC-8004 capability model
    pub registry: String,
    /// Signature algorithm — ES256 only
    pub algorithm: String,
    /// The agent's public JWKS path (published on the wire; never a private key)
    pub jwks_path: String,
    /// Allowed agent domains (operation scopes)
    pub scopes: Vec<String>,
    /// Identity-card version
    pub version: u32,
}

impl KyaMetadata {
    /// The active agent identity card from a role — scopes derive from the role.
    ///
    /// # Errors
    /// The `registered` role (not active) — an inactive agent gets no KYA identity card.
    pub fn for_role(role: &crate::roles::AgentRole) -> Result<Self, String> {
        if role.status != "active" {
            return Err(format!("role {} is not active", role.role));
        }
        let scopes: Vec<String> = match role.role {
            "orchestrator" => vec!["hermes.run", "agents.dispatch", "skills.execute"]
                .into_iter().map(str::to_owned).collect(),
            "merchant" => vec!["market.list", "market.describe", "market.negotiate"]
                .into_iter().map(str::to_owned).collect(),
            "finance" => vec!["wallet.read", "escrow.watch", "ledger.reconcile"]
                .into_iter().map(str::to_owned).collect(),
            "support" => vec!["kb.search", "tickets.answer", "parliament.arbitrate"]
                .into_iter().map(str::to_owned).collect(),
            "analyst" => vec!["economy.report", "leaderboard.read"]
                .into_iter().map(str::to_owned).collect(),
            "scout" => vec!["quests.run", "referrals.track", "catalog.discover"]
                .into_iter().map(str::to_owned).collect(),
            "guardian" => vec![
                "security.audit",
                "surface.verify",
                "stall.enforce",
                "mairon.rea",
                "product.rate",
            ]
            .into_iter().map(str::to_owned).collect(),
            other => vec![format!("{other}.pending")],
        };
        Ok(Self {
            agent_id: role.id.to_owned(),
            role: role.role.to_owned(),
            registry: "erc8004".to_owned(),
            algorithm: KYA_SUPPORTED_JWA.to_owned(),
            jwks_path: format!("/.well-known/agents/{}/jwks.json", role.role),
            scopes,
            version: 1,
        })
    }

    /// Identity cards of all active fleet agents — the stable [`crate::roles::ROLES`] order
    /// (`registered` roles get no identity card — fail-closed).
    ///
    /// # Errors
    /// Never with the current order (all inputs are active) — but signing stays `Result`
    /// so adding a semi-active role does not break compilation.
    pub fn for_fleet() -> Result<Vec<Self>, String> {
        crate::roles::active_roles().map(Self::for_role).collect()
    }
}

/// The x402 v2 payment gate for monetized A2A calls.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct X402Pricing {
    /// Protocol version (always 2)
    pub version: u32,
    /// Recipient public address (`payTo`) — a private key is never here
    pub pay_to: String,
    /// EIP-155 chain ID
    pub network: String,
    /// Asset contract (SBC on the Radius testnet)
    pub asset: String,
    /// Default price in asset units (6 decimals — exactly like Python)
    pub price_sbc: String,
    /// The price cap, to keep the gateway from breaking
    pub max_price_sbc: String,
    /// EIP-3009 transfer method (`transferWithAuthorization`)
    pub asset_transfer_method: String,
}

impl Default for X402Pricing {
    fn default() -> Self {
        Self {
            version: X402_VERSION,
            pay_to: String::new(), // From env at boot; never a private key
            network: X402_NETWORK.to_owned(),
            asset: "0x33ad9e4BD16B69B5BFdED37D8B5D9fF9aba014Fb".to_owned(),
            price_sbc: "0.001".to_owned(),
            max_price_sbc: "1".to_owned(),
            asset_transfer_method: "eip3009".to_owned(),
        }
    }
}

impl X402Pricing {
    /// Pricing with the seller address — the only input needed from the wallet.
    #[must_use]
    pub fn new(pay_to: &str) -> Self {
        Self { pay_to: pay_to.to_owned(), ..Self::default() }
    }

    /// Price allowed? — between zero and the cap (a simple, deterministic string-decimal comparison).
    #[must_use]
    pub fn accepts(&self, price_sbc: &str) -> bool {
        let p = parse_decimal(price_sbc);
        let max = parse_decimal(&self.max_price_sbc);
        p > 0 && p <= max
    }
}

/// Deterministic decimal parsing into 6-decimal units (no float) — like the Python `Decimal`.
fn parse_decimal(s: &str) -> u128 {
    let (int_part, frac_part) = s.trim().split_once('.').unwrap_or((s.trim(), ""));
    let int: u128 = int_part.parse().unwrap_or(0);
    let mut frac: u128 = 0;
    for (i, c) in frac_part.chars().take(6).enumerate() {
        if let Some(d) = c.to_digit(10) {
            frac += u128::from(d) * 10u128.pow(5 - u32::try_from(i).unwrap_or(0));
        }
    }
    int * 1_000_000 + frac
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::roles::{ROLES};

    /// KYA identity cards are built for all 7 active agents — a 7/7 active fleet.
    #[test]
    fn kya_metadata_for_active_fleet_only() {
        let fleet = KyaMetadata::for_fleet().expect("fleet");
        assert_eq!(fleet.len(), 7, "\u{647}\u{645}\u{647}\u{654} \u{627}\u{6cc}\u{62c}\u{646}\u{62a}‌\u{647}\u{627}\u{6cc} \u{641}\u{639}\u{627}\u{644} \u{634}\u{646}\u{627}\u{633}\u{646}\u{627}\u{645}\u{647} \u{645}\u{6cc}‌\u{6af}\u{6cc}\u{631}\u{646}\u{62f}");
        for m in &fleet {
            assert_eq!(m.algorithm, "ES256");
            assert_eq!(m.registry, "erc8004");
            assert!(m.jwks_path.starts_with("/.well-known/agents/"));
            assert_ne!(m.scopes, [] as [std::string::String; 0]);
        }
        // The guardian is now active — it carries the security/verification scopes.
        let guardian = ROLES.iter().find(|r| r.role == "guardian").expect("guardian");
        let g = KyaMetadata::for_role(guardian).expect("guardian is now active");
        assert!(g.scopes.contains(&"security.audit".to_owned()));
        assert!(g.scopes.contains(&"surface.verify".to_owned()));
    }

    /// Uniqueness and separation of Mairon capabilities — `mairon.rea` / `product.rate` live
    /// only on `guardian` and the 7/7 fleet stays intact (Wave 2 Step 6).
    #[test]
    fn guardian_dispatches_mairon_rea_capabilities_exclusively() {
        let fleet = KyaMetadata::for_fleet().expect("fleet");
        assert_eq!(fleet.len(), 7, "\u{646}\u{627}\u{648}\u{6af}\u{627}\u{646} \u{6f7}/\u{6f7} \u{628}\u{627}\u{6cc}\u{62f} \u{62d}\u{641}\u{638} \u{634}\u{648}\u{62f}");
        let carriers = fleet
            .iter()
            .filter(|m| m.scopes.contains(&"mairon.rea".to_owned()))
            .map(|m| m.role.as_str())
            .collect::<Vec<_>>();
        assert_eq!(carriers, vec!["guardian"], "\u{62f}\u{627}\u{648}\u{631}\u{6cc} \u{645}\u{627}\u{6cc}\u{631}\u{648}\u{646} \u{641}\u{642}\u{637} \u{627}\u{632} \u{646}\u{6af}\u{647}\u{628}\u{627}\u{646} \u{6af}\u{633}\u{633}\u{62a}\u{647} \u{645}\u{6cc}‌\u{634}\u{648}\u{62f}");
        let g = fleet.iter().find(|m| m.role == "guardian").expect("guardian kya");
        assert!(g.scopes.contains(&"mairon.rea".to_owned()));
        assert!(g.scopes.contains(&"product.rate".to_owned()));
        // No Mairon scope exists anywhere else — uniquely exclusive.
        for m in &fleet {
            if m.role != "guardian" {
                assert!(!m.scopes.contains(&"mairon.rea".to_owned()));
                assert!(!m.scopes.contains(&"product.rate".to_owned()));
            }
        }
    }

    /// Scopes derive from roles and carry no private keys.
    #[test]
    fn scopes_are_role_derived_and_secret_free() {
        let merchant = ROLES.iter().find(|r| r.role == "merchant").expect("merchant");
        let m = KyaMetadata::for_role(merchant).expect("merchant kya");
        assert!(m.scopes.contains(&"market.describe".to_owned()));
        let fin = ROLES.iter().find(|r| r.role == "finance").expect("finance");
        let f = KyaMetadata::for_role(fin).expect("finance kya");
        assert!(f.scopes.contains(&"ledger.reconcile".to_owned()));
        // No scope contains "key" or "secret" — the key-centric contract
        for m in KyaMetadata::for_fleet().expect("fleet") {
            for s in &m.scopes {
                assert!(!s.contains("key"), "scope \u{628}\u{627} \u{6a9}\u{644}\u{6cc}\u{62f} \u{645}\u{645}\u{646}\u{648}\u{639}: {s}");
            }
        }
    }

    /// The x402 gate — Python defaults and capped acceptance.
    #[test]
    fn x402_pricing_defaults_and_gate() {
        let p = X402Pricing::new("0xVENDOR");
        assert_eq!(p.version, 2);
        assert_eq!(p.network, "eip155:72344");
        assert_eq!(p.price_sbc, "0.001");
        assert_eq!(p.max_price_sbc, "1");
        assert_eq!(p.asset_transfer_method, "eip3009");
        assert!(p.accepts("0.001"));
        assert!(p.accepts("1"));
        assert!(p.accepts("0.999999"));
        assert!(!p.accepts("0"), "\u{635}\u{641}\u{631} \u{645}\u{62c}\u{627}\u{632} \u{646}\u{6cc}\u{633}\u{62a}");
        assert!(!p.accepts("1.000001"), "\u{628}\u{627}\u{644}\u{627}\u{6cc} \u{633}\u{642}\u{641} \u{631}\u{62f} \u{645}\u{6cc}‌\u{634}\u{648}\u{62f}");
        assert!(!p.accepts("2"));
        assert!(!p.accepts("abc"), "\u{648}\u{631}\u{648}\u{62f}\u{6cc} \u{62e}\u{631}\u{627}\u{628} \u{631}\u{62f} \u{645}\u{6cc}‌\u{634}\u{648}\u{62f}");
    }

    /// Wire serialization — camelCase and free of any private keys.
    #[test]
    fn wire_format_is_camel_case_and_secret_free() {
        let kya = KyaMetadata::for_fleet().expect("fleet").swap_remove(0);
        let j = serde_json::to_string(&kya).expect("kya json");
        assert!(j.contains("\"agentId\""));
        assert!(j.contains("\"jwksPath\""));
        assert!(j.contains("\"algorithm\":\"ES256\""));
        let p = X402Pricing::new("0xABC");
        let jp = serde_json::to_string(&p).expect("x402 json");
        assert!(jp.contains("\"payTo\":\"0xABC\""));
        assert!(jp.contains("\"network\":\"eip155:72344\""));
        let all = format!("{j}{jp}");
        assert!(!all.to_lowercase().contains("private"), "\u{628}\u{62f}\u{648}\u{646} \u{6a9}\u{644}\u{6cc}\u{62f} \u{62e}\u{635}\u{648}\u{635}\u{6cc}");
        assert!(!all.to_lowercase().contains("secret"), "\u{628}\u{62f}\u{648}\u{646} \u{631}\u{627}\u{632}");
    }
}
