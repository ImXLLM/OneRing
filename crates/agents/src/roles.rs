//! The fleet’s 7 roles — the source of truth for the `Agent` table seed rows.
//!
//! Full activation of the 7-agent fleet (strategic-5repos treasury — folder 05
//! `Agent-Reach` + `starnet`): the five specialist roles (merchant, treasurer, support,
//! analyst, scout) together with the Hermes brain and the **Guardian** are active; the Guardian with
//! the real security/verification handler ([`crate::agent_reach::GuardianSecurityHandler`]) is
//! wired — passes the antibot challenge wall, applies the sub-agent stop circuit
//! breaker, and keeps child capability issuance uniformly attenuated at all times.
//! Every role’s status stays honest: `active` only for roles the kernel
//! actually uses them — and now all seven roles have real usage.

/// One fleet role.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AgentRole {
    /// id (`agent:{role}`)
    pub id: &'static str,
    /// role (unique key)
    pub role: &'static str,
    /// display name
    pub name: &'static str,
    /// honest status — `active` only for roles the kernel actually uses
    pub status: &'static str,
    /// role description
    pub description: &'static str,
}

/// The 7 fleet roles — all active (fleet 7/7).
///
/// Descriptions enriched from the `AgentX` portal (folder 05 — `src/agents/base.ts`):
/// orchestrator = the primary router; merchant = consultative sales; treasurer = wallet and
/// ledger reconciliation; support = help desk and adjudication; analyst = staff-only BI; scout =
/// growth and quests; guardian = security and verification (port of `Agent-Reach` + `starnet`).
pub const ROLES: [AgentRole; 7] = [
    AgentRole {
        id: "agent:orchestrator",
        role: "orchestrator",
        name: "\u{647}\u{631}\u{645}\u{633}",
        status: "active",
        description: "\u{645}\u{63a}\u{632} \u{646}\u{627}\u{648}\u{6af}\u{627}\u{646} — \u{637}\u{628}\u{642}\u{647}‌\u{628}\u{646}\u{62f}\u{6cc} \u{647}\u{62f}\u{641} \u{648} \u{648}\u{627}\u{6af}\u{630}\u{627}\u{631}\u{6cc} \u{628}\u{647} \u{645}\u{62a}\u{62e}\u{635}\u{635}\u{61b} \u{645}\u{647}\u{627}\u{631}\u{62a}‌\u{647}\u{627} \u{631}\u{627} \u{62e}\u{648}\u{62f}\u{634} \u{647}\u{645} \u{627}\u{62c}\u{631}\u{627} \u{645}\u{6cc}‌\u{6a9}\u{646}\u{62f}",
    },
    AgentRole {
        id: "agent:merchant",
        role: "merchant",
        name: "\u{628}\u{627}\u{632}\u{631}\u{6af}\u{627}\u{646}",
        status: "active",
        description: "\u{627}\u{6cc}\u{62c}\u{646}\u{62a} \u{628}\u{627}\u{632}\u{627}\u{631} — \u{641}\u{631}\u{648}\u{634} \u{645}\u{634}\u{627}\u{648}\u{631}\u{647}‌\u{627}\u{6cc}\u{60c} \u{633}\u{627}\u{62e}\u{62a} \u{644}\u{6cc}\u{633}\u{6cc}\u{646}\u{6af} \u{645}\u{62d}\u{635}\u{648}\u{644} \u{648} \u{645}\u{630}\u{627}\u{6a9}\u{631}\u{647}\u{654} \u{642}\u{6cc}\u{645}\u{62a} (\u{67e}\u{648}\u{631}\u{62a} AgentX sales)",
    },
    AgentRole {
        id: "agent:finance",
        role: "finance",
        name: "\u{62e}\u{632}\u{627}\u{646}\u{647}‌\u{62f}\u{627}\u{631}",
        status: "active",
        description: "\u{627}\u{6cc}\u{62c}\u{646}\u{62a} \u{62e}\u{632}\u{627}\u{646}\u{647} — \u{648}\u{627}\u{631}\u{6cc}\u{632}/\u{628}\u{631}\u{62f}\u{627}\u{634}\u{62a}\u{60c} \u{627}\u{633}\u{6a9}\u{631}\u{648} \u{648} \u{622}\u{634}\u{62a}\u{6cc} \u{62f}\u{641}\u{62a}\u{631} \u{628}\u{627} \u{62a}\u{631}\u{627}\u{632}\u{650} \u{635}\u{641}\u{631} (\u{67e}\u{648}\u{631}\u{62a} AgentX finance)",
    },
    AgentRole {
        id: "agent:support",
        role: "support",
        name: "\u{67e}\u{634}\u{62a}\u{6cc}\u{628}\u{627}\u{646}",
        status: "active",
        description: "\u{627}\u{6cc}\u{62c}\u{646}\u{62a} \u{67e}\u{634}\u{62a}\u{6cc}\u{628}\u{627}\u{646}\u{6cc} — \u{67e}\u{627}\u{633}\u{62e} \u{634}\u{647}\u{631}\u{648}\u{646}\u{62f}\u{627}\u{646} \u{627}\u{632} \u{67e}\u{627}\u{6cc}\u{6af}\u{627}\u{647} \u{62f}\u{627}\u{646}\u{634} \u{648} \u{67e}\u{630}\u{6cc}\u{631}\u{634} \u{67e}\u{631}\u{648}\u{646}\u{62f}\u{647}\u{654} \u{62f}\u{627}\u{648}\u{631}\u{6cc} (\u{67e}\u{648}\u{631}\u{62a} AgentX support)",
    },
    AgentRole {
        id: "agent:analyst",
        role: "analyst",
        name: "\u{62a}\u{62d}\u{644}\u{6cc}\u{644}‌\u{6af}\u{631}",
        status: "active",
        description: "\u{627}\u{6cc}\u{62c}\u{646}\u{62a} \u{62a}\u{62d}\u{644}\u{6cc}\u{644} — \u{6af}\u{632}\u{627}\u{631}\u{634} \u{627}\u{642}\u{62a}\u{635}\u{627}\u{62f}\u{60c} \u{631}\u{62a}\u{628}\u{647}‌\u{628}\u{646}\u{62f}\u{6cc} XP \u{648} \u{631}\u{648}\u{646}\u{62f}\u{647}\u{627}\u{61b} \u{641}\u{642}\u{637} \u{628}\u{631}\u{627}\u{6cc} \u{6a9}\u{627}\u{631}\u{645}\u{646}\u{62f}\u{627}\u{646} (\u{67e}\u{648}\u{631}\u{62a} AgentX analyst)",
    },
    AgentRole {
        id: "agent:scout",
        role: "scout",
        name: "\u{6a9}\u{627}\u{648}\u{634}\u{6af}\u{631}",
        status: "active",
        description: "\u{627}\u{6cc}\u{62c}\u{646}\u{62a} \u{631}\u{634}\u{62f} — \u{6a9}\u{648}\u{626}\u{633}\u{62a}\u{60c} \u{645}\u{639}\u{631}\u{641}\u{6cc} \u{648} \u{6a9}\u{634}\u{641} \u{641}\u{631}\u{635}\u{62a}‌\u{647}\u{627}\u{6cc} \u{641}\u{631}\u{648}\u{634} \u{62f}\u{631} \u{6a9}\u{627}\u{62a}\u{627}\u{644}\u{648}\u{6af} (\u{67e}\u{648}\u{631}\u{62a} AgentX receptionist)",
    },
    AgentRole {
        id: "agent:guardian",
        role: "guardian",
        name: "\u{646}\u{6af}\u{647}\u{628}\u{627}\u{646}",
        status: "active",
        description: "\u{627}\u{6cc}\u{62c}\u{646}\u{62a} \u{627}\u{645}\u{646}\u{6cc}\u{62a} — \u{647}\u{646}\u{62f}\u{644}\u{631} \u{631}\u{627}\u{633}\u{62a}\u{6cc}‌\u{622}\u{632}\u{645}\u{627}\u{6cc}\u{6cc} \u{641}\u{639}\u{627}\u{644}: \u{631}\u{62f} \u{62f}\u{6cc}\u{648}\u{627}\u{631} \u{686}\u{627}\u{644}\u{634} ant\u{650}ibot\u{60c} \u{645}\u{62f}\u{627}\u{631}\u{634}\u{6a9}\u{646} \u{62a}\u{648}\u{642}\u{641} \u{633}\u{627}\u{628}‌\u{627}\u{6cc}\u{62c}\u{646}\u{62a}\u{60c} \u{62a}\u{636}\u{639}\u{6cc}\u{641} \u{6cc}\u{6a9}\u{646}\u{648}\u{627}\u{62e}\u{62a} \u{642}\u{627}\u{628}\u{644}\u{6cc}\u{62a}‌\u{647}\u{627} \u{648} \u{62f}\u{627}\u{648}\u{631}\u{6cc} REA \u{645}\u{627}\u{6cc}\u{631}\u{648}\u{646} (`mairon.rea`/`product.rate`) \u{631}\u{648}\u{6cc} \u{645}\u{62d}\u{635}\u{648}\u{644}\u{627}\u{62a} \u{628}\u{627}\u{632}\u{627}\u{631} (\u{67e}\u{648}\u{631}\u{62a} Agent-Reach + starnet + morluto/rea)",
    },
];

/// Active role count — 7/7 (the 7-agent fleet mission).
pub fn active_roles() -> impl Iterator<Item = &'static AgentRole> {
    ROLES.iter().filter(|r| r.status == "active")
}

/// Role by id.
#[must_use]
pub fn role_of(id: &str) -> Option<&'static AgentRole> {
    ROLES.iter().find(|r| r.id == id)
}

/// Seed rows for the `Agent` table — output as plain tuples (no db dependency).
#[must_use]
pub fn seed_agents() -> Vec<(&'static str, &'static str, &'static str, &'static str, &'static str)> {
    ROLES
        .iter()
        .map(|r| (r.id, r.name, r.role, r.status, r.description))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Exactly 7 roles, unique id and role, honest status — fleet 7/7 active.
    #[test]
    fn roles_are_seven_unique_and_honest() {
        assert_eq!(ROLES.len(), 7);
        let mut ids: Vec<&str> = ROLES.iter().map(|r| r.id).collect();
        ids.sort_unstable();
        assert!(ids.windows(2).all(|w| w[0] != w[1]));
        // seven active roles — brain + five specialists + the security-handler guardian.
        assert_eq!(active_roles().count(), 7, "\u{646}\u{627}\u{648}\u{6af}\u{627}\u{646} \u{6f7}/\u{6f7} \u{628}\u{627}\u{6cc}\u{62f} \u{641}\u{639}\u{627}\u{644} \u{628}\u{627}\u{634}\u{62f}");
        for r in &ROLES {
            assert_eq!(r.status, "active", "role {} must be active", r.role);
        }
        assert!(role_of("agent:orchestrator").is_some());
        assert!(role_of("agent:nope").is_none());
    }

    /// Seed — 7 rows aligned with ROLES.
    #[test]
    fn seed_matches_roles() {
        let seed = seed_agents();
        assert_eq!(seed.len(), ROLES.len());
        for (row, role) in seed.iter().zip(ROLES.iter()) {
            assert_eq!(row.0, role.id);
            assert_eq!(row.3, role.status);
        }
    }

    /// Descriptions enriched from the `AgentX` portal — every active role has a ported description.
    #[test]
    fn active_roles_carry_agentx_enriched_descriptions() {
        let expectations: &[(&str, &str)] = &[
            ("merchant", "\u{641}\u{631}\u{648}\u{634} \u{645}\u{634}\u{627}\u{648}\u{631}\u{647}‌\u{627}\u{6cc}"),
            ("finance", "\u{622}\u{634}\u{62a}\u{6cc} \u{62f}\u{641}\u{62a}\u{631}"),
            ("support", "\u{62f}\u{627}\u{648}\u{631}\u{6cc}"),
            ("analyst", "\u{631}\u{62a}\u{628}\u{647}‌\u{628}\u{646}\u{62f}\u{6cc} XP"),
            ("scout", "\u{6a9}\u{648}\u{626}\u{633}\u{62a}"),
            ("orchestrator", "\u{648}\u{627}\u{6af}\u{630}\u{627}\u{631}\u{6cc}"),
            ("guardian", "\u{631}\u{627}\u{633}\u{62a}\u{6cc}‌\u{622}\u{632}\u{645}\u{627}\u{6cc}\u{6cc}"),
        ];
        for (role, marker) in expectations {
            let r = ROLES.iter().find(|r| r.role == *role).expect("role exists");
            assert!(
                r.description.contains(marker),
                "description of {role} lacks '{marker}': {}",
                r.description
            );
        }
    }
}
