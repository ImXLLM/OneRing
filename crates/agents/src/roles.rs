//! نقش‌های ۷گانهٔ ناوگان — منبع حقیقت ردیف‌های بذر جدول `Agent`.

/// یک نقش ناوگان.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AgentRole {
    /// شناسه (`agent:{role}`)
    pub id: &'static str,
    /// نقش (کلید یکتا)
    pub role: &'static str,
    /// نام نمایشی
    pub name: &'static str,
    /// وضعیت صادقانه — `active` فقط برای نقش‌هایی که کرنل واقعاً استفاده می‌کند
    pub status: &'static str,
    /// توضیح نقش
    pub description: &'static str,
}

/// ۷ نقش ناوگان — orchestrator تنها نقش فعال (مغز همین کرنل است).
pub const ROLES: [AgentRole; 7] = [
    AgentRole {
        id: "agent:orchestrator",
        role: "orchestrator",
        name: "هرمس",
        status: "active",
        description: "قلب ناوگان — مغز همین کرنل؛ مهارت‌ها را اجرا می‌کند",
    },
    AgentRole {
        id: "agent:merchant",
        role: "merchant",
        name: "بازرگان",
        status: "registered",
        description: "ایجنت بازار — فروش، تحویل و مذاکره (وایر در M5)",
    },
    AgentRole {
        id: "agent:finance",
        role: "finance",
        name: "خزانه‌دار",
        status: "registered",
        description: "ایجنت خزانه — واریز/برداشت و آشتی دفتر (وایر در M5)",
    },
    AgentRole {
        id: "agent:support",
        role: "support",
        name: "پشتیبان",
        status: "registered",
        description: "ایجنت پشتیبانی — پاسخ شهروندان از پایگاه دانش (وایر در M5)",
    },
    AgentRole {
        id: "agent:analyst",
        role: "analyst",
        name: "تحلیل‌گر",
        status: "registered",
        description: "ایجنت تحلیل — گزارش اقتصاد و روندها (وایر در M5)",
    },
    AgentRole {
        id: "agent:scout",
        role: "scout",
        name: "کاوشگر",
        status: "registered",
        description: "ایجنت رشد — کوئست، معرفی و کشف فرصت (وایر در M5)",
    },
    AgentRole {
        id: "agent:guardian",
        role: "guardian",
        name: "نگهبان",
        status: "registered",
        description: "ایجنت امنیت — ممیزی، سپر و پاسخ به تهدید (وایر در M5)",
    },
];

/// نقش با شناسه.
#[must_use]
pub fn role_of(id: &str) -> Option<&'static AgentRole> {
    ROLES.iter().find(|r| r.id == id)
}

/// ردیف‌های بذر برای جدول `Agent` — خروجی به‌صورت تاپل ساده (بدون وابستگی db).
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

    /// دقیقاً ۷ نقش، شناسه و نقش یکتا، وضعیت صادقانه.
    #[test]
    fn roles_are_seven_unique_and_honest() {
        assert_eq!(ROLES.len(), 7);
        let mut ids: Vec<&str> = ROLES.iter().map(|r| r.id).collect();
        ids.sort_unstable();
        assert!(ids.windows(2).all(|w| w[0] != w[1]));
        // فقط orchestrator فعال — بقیه صادقانه registered
        for r in &ROLES {
            if r.role == "orchestrator" {
                assert_eq!(r.status, "active");
            } else {
                assert_eq!(r.status, "registered", "role {} must be honest", r.role);
            }
        }
        assert!(role_of("agent:orchestrator").is_some());
        assert!(role_of("agent:nope").is_none());
    }

    /// بذر — ۷ ردیف هم‌تراز با ROLES.
    #[test]
    fn seed_matches_roles() {
        let seed = seed_agents();
        assert_eq!(seed.len(), ROLES.len());
        for (row, role) in seed.iter().zip(ROLES.iter()) {
            assert_eq!(row.0, role.id);
            assert_eq!(row.3, role.status);
        }
    }
}
