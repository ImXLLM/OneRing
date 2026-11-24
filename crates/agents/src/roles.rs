//! نقش‌های ۷گانهٔ ناوگان — منبع حقیقت ردیف‌های بذر جدول `Agent`.
//!
//! فعال‌سازی کامل ناوگان ۷عاملی (گنجینهٔ strategic-5repos — پوشهٔ ۰۵
//! `Agent-Reach` + `starnet`): پنج نقش متخصص (بازرگان، خزانه‌دار، پشتیبان،
//! تحلیل‌گر، کاوشگر) به‌همراه مغز هرمس و **نگهبان** فعال‌اند؛ نگهبان با
//! هندلر امنیت/راستی‌آزمایی واقعی ([`crate::agent_reach::GuardianSecurityHandler`])
//! سیم شده — دیوار چالش antِibot را رد می‌کند، مدارشکن توقف ساب‌ایجنت را
//! اعمال می‌کند و صدور قابلیت فرزند را همیشه تضعیف یکنواخت نگه می‌دارد.
//! وضعیت هر نقش همچنان صادقانه است: `active` فقط برای نقش‌هایی که کرنل
//! واقعاً از آن‌ها استفاده می‌کند — و اینک هر هفت نقش استفادهٔ واقعی دارند.

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

/// ۷ نقش ناوگان — همگی فعال (ناوگان ۷/۷).
///
/// توصیف‌ها از پورتال `AgentX` (پوشهٔ ۰۵ — `src/agents/base.ts`) غنی‌تر شده‌اند:
/// ارکستراتور = مسیریابِ اصلی؛ بازرگان = فروش مشاوره‌ای؛ خزانه‌دار = ولت و
/// آشتی دفتر؛ پشتیبان = میز کمک و داوری؛ تحلیل‌گر = BI فقط-کارمند؛ کاوشگر =
/// رشد و کوئست؛ نگهبان = امنیت و راستی‌آزمایی (پورت `Agent-Reach` + `starnet`).
pub const ROLES: [AgentRole; 7] = [
    AgentRole {
        id: "agent:orchestrator",
        role: "orchestrator",
        name: "هرمس",
        status: "active",
        description: "مغز ناوگان — طبقه‌بندی هدف و واگذاری به متخصص؛ مهارت‌ها را خودش هم اجرا می‌کند",
    },
    AgentRole {
        id: "agent:merchant",
        role: "merchant",
        name: "بازرگان",
        status: "active",
        description: "ایجنت بازار — فروش مشاوره‌ای، ساخت لیسینگ محصول و مذاکرهٔ قیمت (پورت AgentX sales)",
    },
    AgentRole {
        id: "agent:finance",
        role: "finance",
        name: "خزانه‌دار",
        status: "active",
        description: "ایجنت خزانه — واریز/برداشت، اسکرو و آشتی دفتر با ترازِ صفر (پورت AgentX finance)",
    },
    AgentRole {
        id: "agent:support",
        role: "support",
        name: "پشتیبان",
        status: "active",
        description: "ایجنت پشتیبانی — پاسخ شهروندان از پایگاه دانش و پذیرش پروندهٔ داوری (پورت AgentX support)",
    },
    AgentRole {
        id: "agent:analyst",
        role: "analyst",
        name: "تحلیل‌گر",
        status: "active",
        description: "ایجنت تحلیل — گزارش اقتصاد، رتبه‌بندی XP و روندها؛ فقط برای کارمندان (پورت AgentX analyst)",
    },
    AgentRole {
        id: "agent:scout",
        role: "scout",
        name: "کاوشگر",
        status: "active",
        description: "ایجنت رشد — کوئست، معرفی و کشف فرصت‌های فروش در کاتالوگ (پورت AgentX receptionist)",
    },
    AgentRole {
        id: "agent:guardian",
        role: "guardian",
        name: "نگهبان",
        status: "active",
        description: "ایجنت امنیت — هندلر راستی‌آزمایی فعال: رد دیوار چالش antِibot، مدارشکن توقف ساب‌ایجنت و تضعیف یکنواخت قابلیت‌ها (پورت Agent-Reach + starnet)",
    },
];

/// شمار نقش‌های فعال — ۷/۷ (مأموریت ناوگان ۷عاملی).
pub fn active_roles() -> impl Iterator<Item = &'static AgentRole> {
    ROLES.iter().filter(|r| r.status == "active")
}

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

    /// دقیقاً ۷ نقش، شناسه و نقش یکتا، وضعیت صادقانه — ناوگان ۷/۷ فعال.
    #[test]
    fn roles_are_seven_unique_and_honest() {
        assert_eq!(ROLES.len(), 7);
        let mut ids: Vec<&str> = ROLES.iter().map(|r| r.id).collect();
        ids.sort_unstable();
        assert!(ids.windows(2).all(|w| w[0] != w[1]));
        // هفت نقش فعال — مغز + پنج متخصص + نگهبانِ هندلر امنیت.
        assert_eq!(active_roles().count(), 7, "ناوگان ۷/۷ باید فعال باشد");
        for r in &ROLES {
            assert_eq!(r.status, "active", "role {} must be active", r.role);
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

    /// توصیف‌های غنی‌شده از پورتال AgentX — هر نقش فعال توضیح پورت‌شده دارد.
    #[test]
    fn active_roles_carry_agentx_enriched_descriptions() {
        let expectations: &[(&str, &str)] = &[
            ("merchant", "فروش مشاوره‌ای"),
            ("finance", "آشتی دفتر"),
            ("support", "داوری"),
            ("analyst", "رتبه‌بندی XP"),
            ("scout", "کوئست"),
            ("orchestrator", "واگذاری"),
            ("guardian", "راستی‌آزمایی"),
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
