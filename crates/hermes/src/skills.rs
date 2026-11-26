//! The 16 Hermes skills list — the single source of truth for skill data (the database seed comes from here).
//!
//! Two kinds of skill, both deterministic in behavior:
//! * [`SkillKind::Kernel`] — direct execution on the money kernel (no LLM; zero tokens).
//! * [`SkillKind::Llm`] — fixed prompt template + AI gateway with a budget envelope.
//!
//! Skills never execute user code — only the registered handlers in this file
//! (the brain sandbox principle; REA gate step 4).

/// Skill kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillKind {
    /// Deterministic execution on the kernel — no LLM
    Kernel,
    /// Prompt template + AI gateway
    Llm,
}

impl SkillKind {
    /// Wire string (`kernel`/`llm`).
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            SkillKind::Kernel => "kernel",
            SkillKind::Llm => "llm",
        }
    }
}

/// Definition of one skill.
#[derive(Debug, Clone, Copy)]
pub struct SkillDef {
    /// Skill ID (`wallet.balance`)
    pub id: &'static str,
    /// Persian title
    pub title: &'static str,
    /// Category
    pub category: &'static str,
    /// One-line description
    pub description: &'static str,
    /// Execution kind
    pub kind: SkillKind,
    /// Token estimate for the budget envelope — kernel is zero
    pub estimate_tokens: i64,
}

/// The 16 Hermes business skills — this order is also the public display order.
pub const SKILLS: [SkillDef; 16] = [
    SkillDef {
        id: "wallet.balance",
        title: "\u{62a}\u{631}\u{627}\u{632} \u{645}\u{646}",
        category: "wallet",
        description: "\u{62e}\u{648}\u{627}\u{646}\u{634} \u{642}\u{637}\u{639}\u{6cc} \u{62a}\u{631}\u{627}\u{632} \u{648} \u{642}\u{641}\u{644} \u{627}\u{645}\u{627}\u{646}\u{62a}\u{6cc} \u{627}\u{632} \u{62f}\u{641}\u{62a}\u{631} \u{62f}\u{648}\u{637}\u{631}\u{641}\u{647}",
        kind: SkillKind::Kernel,
        estimate_tokens: 0,
    },
    SkillDef {
        id: "wallet.txs",
        title: "\u{62a}\u{631}\u{627}\u{6a9}\u{646}\u{634}‌\u{647}\u{627}\u{6cc} \u{645}\u{646}",
        category: "wallet",
        description: "\u{6f1}\u{6f0} \u{631}\u{62f}\u{6cc}\u{641} \u{627}\u{62e}\u{6cc}\u{631} \u{62f}\u{641}\u{62a}\u{631} \u{62d}\u{633}\u{627}\u{628} \u{645}\u{646}",
        kind: SkillKind::Kernel,
        estimate_tokens: 0,
    },
    SkillDef {
        id: "market.list",
        title: "\u{628}\u{627}\u{632}\u{627}\u{631}",
        category: "market",
        description: "\u{645}\u{62d}\u{635}\u{648}\u{644}\u{627}\u{62a} \u{641}\u{639}\u{627}\u{644} \u{628}\u{627}\u{632}\u{627}\u{631} \u{628}\u{627} \u{642}\u{6cc}\u{645}\u{62a} \u{62f}\u{6cc}\u{6cc}\u{646}",
        kind: SkillKind::Kernel,
        estimate_tokens: 0,
    },
    SkillDef {
        id: "market.my",
        title: "\u{633}\u{641}\u{627}\u{631}\u{634}‌\u{647}\u{627}\u{6cc} \u{645}\u{646}",
        category: "market",
        description: "\u{645}\u{62d}\u{635}\u{648}\u{644}\u{627}\u{62a} \u{648} \u{627}\u{645}\u{627}\u{646}\u{62a}‌\u{647}\u{627}\u{6cc}\u{6cc} \u{6a9}\u{647} \u{637}\u{631}\u{641}\u{634}\u{627}\u{646} \u{647}\u{633}\u{62a}\u{645}",
        kind: SkillKind::Kernel,
        estimate_tokens: 0,
    },
    SkillDef {
        id: "escrow.status",
        title: "\u{648}\u{636}\u{639}\u{6cc}\u{62a} \u{627}\u{645}\u{627}\u{646}\u{62a}",
        category: "market",
        description: "\u{648}\u{636}\u{639}\u{6cc}\u{62a} \u{6cc}\u{6a9} \u{627}\u{645}\u{627}\u{646}\u{62a} \u{628}\u{627} \u{634}\u{646}\u{627}\u{633}\u{647} escrow:…",
        kind: SkillKind::Kernel,
        estimate_tokens: 0,
    },
    SkillDef {
        id: "economy.report",
        title: "\u{6af}\u{632}\u{627}\u{631}\u{634} \u{627}\u{642}\u{62a}\u{635}\u{627}\u{62f}",
        category: "nation",
        description: "\u{634}\u{641}\u{627}\u{641}\u{6cc}\u{62a} \u{645}\u{627}\u{62f}\u{647}\u{654} \u{6f6}: \u{639}\u{631}\u{636}\u{647}\u{60c} \u{62e}\u{632}\u{627}\u{646}\u{647}\u{60c} \u{642}\u{641}\u{644} \u{627}\u{645}\u{627}\u{646}\u{62a}",
        kind: SkillKind::Kernel,
        estimate_tokens: 0,
    },
    SkillDef {
        id: "kb.search",
        title: "\u{62c}\u{633}\u{62a}‌\u{648}\u{62c}\u{648}\u{6cc} \u{62f}\u{627}\u{646}\u{634}",
        category: "brain",
        description: "\u{62c}\u{633}\u{62a}‌\u{648}\u{62c}\u{648}\u{6cc} \u{67e}\u{627}\u{6cc}\u{6af}\u{627}\u{647} \u{62f}\u{627}\u{646}\u{634} \u{6a9}\u{6cc}\u{627}\u{646}",
        kind: SkillKind::Kernel,
        estimate_tokens: 0,
    },
    SkillDef {
        id: "deposits.mine",
        title: "\u{648}\u{627}\u{631}\u{6cc}\u{632}\u{647}\u{627}\u{6cc} \u{645}\u{646}",
        category: "wallet",
        description: "\u{648}\u{636}\u{639}\u{6cc}\u{62a} \u{648}\u{627}\u{631}\u{6cc}\u{632}\u{647}\u{627}\u{6cc} TON \u{645}\u{646}",
        kind: SkillKind::Kernel,
        estimate_tokens: 0,
    },
    SkillDef {
        id: "hermes.help",
        title: "\u{631}\u{627}\u{647}\u{646}\u{645}\u{627}\u{6cc} \u{647}\u{631}\u{645}\u{633}",
        category: "brain",
        description: "\u{641}\u{647}\u{631}\u{633}\u{62a} \u{645}\u{647}\u{627}\u{631}\u{62a}‌\u{647}\u{627}\u{6cc} \u{62f}\u{631} \u{62f}\u{633}\u{62a}\u{631}\u{633}",
        kind: SkillKind::Kernel,
        estimate_tokens: 0,
    },
    SkillDef {
        id: "identity.report",
        title: "\u{627}\u{639}\u{62a}\u{628}\u{627}\u{631} \u{645}\u{646}",
        category: "brain",
        description: "\u{627}\u{645}\u{62a}\u{6cc}\u{627}\u{632} \u{647}\u{648}\u{6cc}\u{62a}\u{6cc} \u{645}\u{646} \u{627}\u{632} \u{634}\u{645}\u{627}\u{631}\u{634} \u{631}\u{627}\u{646}‌\u{647}\u{627}\u{6cc} \u{633}\u{627}\u{644}\u{645}/\u{645}\u{633}\u{62f}\u{648}\u{62f}",
        kind: SkillKind::Kernel,
        estimate_tokens: 0,
    },
    SkillDef {
        id: "ai.chat",
        title: "\u{6af}\u{641}\u{62a}‌\u{648}\u{6af}\u{648} \u{628}\u{627} \u{645}\u{63a}\u{632}",
        category: "brain",
        description: "\u{6af}\u{641}\u{62a}‌\u{648}\u{6af}\u{648}\u{6cc} \u{622}\u{632}\u{627}\u{62f} \u{628}\u{627} \u{645}\u{63a}\u{632} \u{6a9}\u{6cc}\u{627}\u{646} \u{631}\u{648}\u{6cc} \u{6af}\u{6cc}\u{62a}‌\u{648}\u{6cc} \u{641}\u{627}\u{644}‌\u{628}\u{6a9}",
        kind: SkillKind::Llm,
        estimate_tokens: 900,
    },
    SkillDef {
        id: "market.describe",
        title: "\u{62a}\u{648}\u{635}\u{6cc}\u{641} \u{645}\u{62d}\u{635}\u{648}\u{644}",
        category: "market",
        description: "\u{62a}\u{648}\u{644}\u{6cc}\u{62f} \u{62a}\u{648}\u{636}\u{6cc}\u{62d} \u{641}\u{631}\u{648}\u{634}‌\u{67e}\u{630}\u{6cc}\u{631} \u{628}\u{631}\u{627}\u{6cc} \u{645}\u{62d}\u{635}\u{648}\u{644} \u{645}\u{646}",
        kind: SkillKind::Llm,
        estimate_tokens: 600,
    },
    SkillDef {
        id: "support.faq",
        title: "\u{67e}\u{634}\u{62a}\u{6cc}\u{628}\u{627}\u{646}\u{6cc}",
        category: "support",
        description: "\u{67e}\u{627}\u{633}\u{62e} \u{67e}\u{634}\u{62a}\u{6cc}\u{628}\u{627}\u{646}\u{6cc} \u{628}\u{631} \u{67e}\u{627}\u{6cc}\u{647}\u{654} \u{645}\u{62a}\u{646} \u{633}\u{624}\u{627}\u{644} \u{645}\u{646}",
        kind: SkillKind::Llm,
        estimate_tokens: 500,
    },
    SkillDef {
        id: "nation.explain",
        title: "\u{62d}\u{627}\u{6a9}\u{645}\u{6cc}\u{62a}",
        category: "nation",
        description: "\u{62a}\u{648}\u{636}\u{6cc}\u{62d} \u{67e}\u{627}\u{631}\u{644}\u{645}\u{627}\u{646} \u{648} \u{622}\u{633}\u{62a}\u{627}\u{646}\u{647}\u{654} \u{6f5}\u{6f0}\u{66c}\u{6f0}\u{6f0}\u{6f0} \u{62f}\u{6cc}\u{6cc}\u{646}",
        kind: SkillKind::Llm,
        estimate_tokens: 400,
    },
    SkillDef {
        id: "charter.explain",
        title: "\u{645}\u{646}\u{634}\u{648}\u{631}",
        category: "nation",
        description: "\u{62a}\u{648}\u{636}\u{6cc}\u{62d} \u{645}\u{648}\u{627}\u{62f} \u{645}\u{646}\u{634}\u{648}\u{631} \u{6a9}\u{6cc}\u{627}\u{646}",
        kind: SkillKind::Llm,
        estimate_tokens: 400,
    },
    SkillDef {
        id: "growth.quest",
        title: "\u{6a9}\u{648}\u{626}\u{633}\u{62a} \u{631}\u{634}\u{62f}",
        category: "growth",
        description: "\u{67e}\u{6cc}\u{634}\u{646}\u{647}\u{627}\u{62f} \u{6a9}\u{648}\u{626}\u{633}\u{62a} \u{631}\u{634}\u{62f} \u{645}\u{62a}\u{646}\u{627}\u{633}\u{628} \u{628}\u{627} \u{645}\u{646}",
        kind: SkillKind::Llm,
        estimate_tokens: 500,
    },
];

/// Skill by ID.
#[must_use]
pub fn find_skill(id: &str) -> Option<&'static SkillDef> {
    SKILLS.iter().find(|s| s.id == id)
}

/// Active skills for the database seed — fixed order.
#[must_use]
pub fn all_skills() -> &'static [SkillDef; 16] {
    &SKILLS
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Exactly 16 skills, unique IDs, every skill with complete metadata.
    #[test]
    fn skills_are_sixteen_unique_and_complete() {
        assert_eq!(SKILLS.len(), 16);
        let mut ids: Vec<&str> = SKILLS.iter().map(|s| s.id).collect();
        ids.sort_unstable();
        let unique = ids.windows(2).all(|w| w[0] != w[1]);
        assert!(unique, "skill ids must be unique: {ids:?}");
        for s in &SKILLS {
            assert!(!s.title.is_empty() && !s.description.is_empty());
            match s.kind {
                SkillKind::Kernel => assert_eq!(s.estimate_tokens, 0, "kernel skill costs zero"),
                SkillKind::Llm => assert!(s.estimate_tokens > 0, "llm skill must estimate"),
            }
        }
    }

    /// IDs use the domain format (category-dot).
    #[test]
    fn ids_are_dotted_lowercase() {
        for s in &SKILLS {
            assert!(s.id.split('.').count() == 2, "id must be category.name: {}", s.id);
            assert!(s.id.chars().all(|c| c.is_ascii_lowercase() || c == '.' || c.is_ascii_digit()));
        }
    }
}
