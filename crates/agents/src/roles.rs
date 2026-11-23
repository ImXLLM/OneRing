//! نقش‌های ۷گانهٔ ناوگان — منبع حقیقت ردیف‌های بذر جدول `Agent`.
//!
//! فعال‌سازی ناوگان ۶عاملی (پوشهٔ ۰۵ گنجینه — `AgentX`): پنج نقش متخصص
//! (بازرگان، خزانه‌دار، پشتیبان، تحلیل‌گر، کاوشگر) به‌همراه مغز هرمس
//! **فعال**اند؛ نگهبان هنوز صادقانه `registered` است تا کرنل واقعاً از آن
//! استفاده نکرده، خودش را فعال وانمود نکند (اصل صادق‌بودن).

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

/// ۷ نقش ناوگان — ۶ فعال + نگهبان ثبت‌شده (صداقت وضعیت).
///
/// توصیف‌ها از پورتال `AgentX` (پوشهٔ ۰۵ — `src/agents/base.ts`) غنی‌تر شده‌اند:
/// ارکستراتور = مسیریابِ اصلی؛ بازرگان = فروش مشاوره‌ای؛ خزانه‌دار = ولت و
/// آشتی دفتر؛ پشتیبان = میز کمک و داوری؛ تحلیل‌گر = BI فقط-کارمند؛ کاوشگر =
/// رشد و کوئست.
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
        status: "registered",
        description: "ایجنت امنیت — ممیزی، سپر و پاسخ به تهدید (وایر در M5+؛ هنوز صادقانه ثبت‌شده)",
    },
];

/// شمار نقش‌های فعال — ۶/۷ (مأموریت ناوگان ۶عاملی).
#[must_use]
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

    /// دقیقاً ۷ نقش، شناسه و نقش یکتا، وضعیت صادقانه — ناوگان ۶+۱.
    #[test]
    fn roles_are_seven_unique_and_honest() {
        assert_eq!(ROLES.len(), 7);
        let mut ids: Vec<&str> = ROLES.iter().map(|r| r.id).collect();
        ids.sort_unstable();
        assert!(ids.windows(2).all(|w| w[0] != w[1]));
        // شش نقش فعال — مغز + پنج متخصص؛ نگهبان صادقانه registered
        assert_eq!(active_roles().count(), 6, "ناوگان ۶عاملی باید فعال باشد");
        for r in &ROLES {
            if r.role == "guardian" {
                assert_eq!(r.status, "registered", "guardian must stay honest");
            } else {
                assert_eq!(r.status, "active", "role {} must be active", r.role);
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
