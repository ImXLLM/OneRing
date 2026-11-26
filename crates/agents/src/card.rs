//! A2A discovery card — Kian's agent public face for external agents.
//!
//! Contract: no keys, tokens, or sessions in the card — only name, description, url,
//! versions and the skill list. The protocol version is honestly `a2a/1.0-kian` (Kian's own
//! implementation of the A2A concept, not a claim of official compatibility).

/// One skill in the card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardSkill {
    /// Skill ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Description
    pub description: String,
    /// Tags
    pub tags: Vec<String>,
}

/// Agent card — public JSON.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentCard {
    /// Agent name
    pub name: String,
    /// Description
    pub description: String,
    /// Service address (the kernel API base)
    pub url: String,
    /// Kian version
    pub version: String,
    /// Protocol version
    pub protocol_version: &'static str,
    /// Organization
    pub organization: String,
    /// Skills
    pub skills: Vec<CardSkill>,
}

impl AgentCard {
    /// Publishable JSON — key order stays deterministic via `BTreeMap`.
    #[must_use]
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.name,
            "description": self.description,
            "url": self.url,
            "version": self.version,
            "protocolVersion": self.protocol_version,
            "provider": { "organization": self.organization, "url": self.url },
            "skills": self.skills.iter().map(|s| serde_json::json!({
                "id": s.id, "name": s.name, "description": s.description, "tags": s.tags,
            })).collect::<Vec<_>>(),
        })
    }
}

/// Protocol version — the Kian implementation.
pub const PROTOCOL_VERSION: &str = "a2a/1.0-kian";

/// Build the card from boot parameters — skills come from Hermes.
#[must_use]
pub fn build_agent_card(
    name: &str,
    description: &str,
    url: &str,
    version: &str,
    skills: Vec<CardSkill>,
) -> AgentCard {
    AgentCard {
        name: name.to_owned(),
        description: description.to_owned(),
        url: url.to_owned(),
        version: version.to_owned(),
        protocol_version: PROTOCOL_VERSION,
        organization: "KiaAgentX".to_owned(),
        skills,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The card — correct shape, no credential of any kind.
    #[test]
    fn card_is_honest_and_secret_free() {
        let card = build_agent_card(
            "KIAN Kernel",
            "Money kernel + Hermes brain",
            "http://127.0.0.1:8088/api/v1",
            "0.1.0",
            vec![CardSkill {
                id: "wallet.balance".into(),
                name: "\u{62a}\u{631}\u{627}\u{632} \u{645}\u{646}".into(),
                description: "\u{62e}\u{648}\u{627}\u{646}\u{634} \u{642}\u{637}\u{639}\u{6cc} \u{62f}\u{641}\u{62a}\u{631}".into(),
                tags: vec!["kernel".into()],
            }],
        );
        let json = card.to_json();
        assert_eq!(json["protocolVersion"], PROTOCOL_VERSION);
        assert_eq!(json["provider"]["organization"], "KiaAgentX");
        let wire = json.to_string().to_lowercase();
        for marker in ["x-kian-key", "bearer", "password", "token="] {
            assert!(!wire.contains(marker), "card must not contain {marker}");
        }
        assert_eq!(card.skills.len(), 1);
    }
}
