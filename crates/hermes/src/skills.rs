//! فهرست ۱۶ مهارت هرمس — منبع حقیقت دادهٔ مهارت‌ها (seed دیتابیس از همین‌جا).
//!
//! دو جنس مهارت، هر دو قطعی در رفتار:
//! * [`SkillKind::Kernel`] — اجرای مستقیم روی کرنل پول (بدون LLM؛ توکن صفر).
//! * [`SkillKind::Llm`] — قالب پرامپت ثابت + گیت‌وی AI با پاکت بودجه.
//!
//! مهارت‌ها هرگز کد کاربر را اجرا نمی‌کنند — فقط هندلرهای ثبت‌شدهٔ این فایل
//! (اصل sandbox مغز؛ گیت REA گام ۴).

/// جنس مهارت.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillKind {
    /// اجرای قطعی روی کرنل — بدون LLM
    Kernel,
    /// قالب پرامپت + گیت‌وی AI
    Llm,
}

impl SkillKind {
    /// رشتهٔ سیم (`kernel`/`llm`).
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            SkillKind::Kernel => "kernel",
            SkillKind::Llm => "llm",
        }
    }
}

/// تعریف یک مهارت.
#[derive(Debug, Clone, Copy)]
pub struct SkillDef {
    /// شناسهٔ مهارت (`wallet.balance`)
    pub id: &'static str,
    /// عنوان فارسی
    pub title: &'static str,
    /// دسته
    pub category: &'static str,
    /// توضیح یک‌خطی
    pub description: &'static str,
    /// جنس اجرا
    pub kind: SkillKind,
    /// برآورد توکن برای پاکت بودجه — kernel صفر
    pub estimate_tokens: i64,
}

/// ۱۶ مهارت تجاری هرمس — ترتیب، ترتیبِ نمایش عمومی هم هست.
pub const SKILLS: [SkillDef; 16] = [
    SkillDef {
        id: "wallet.balance",
        title: "تراز من",
        category: "wallet",
        description: "خوانش قطعی تراز و قفل امانتی از دفتر دوطرفه",
        kind: SkillKind::Kernel,
        estimate_tokens: 0,
    },
    SkillDef {
        id: "wallet.txs",
        title: "تراکنش‌های من",
        category: "wallet",
        description: "۱۰ ردیف اخیر دفتر حساب من",
        kind: SkillKind::Kernel,
        estimate_tokens: 0,
    },
    SkillDef {
        id: "market.list",
        title: "بازار",
        category: "market",
        description: "محصولات فعال بازار با قیمت دیین",
        kind: SkillKind::Kernel,
        estimate_tokens: 0,
    },
    SkillDef {
        id: "market.my",
        title: "سفارش‌های من",
        category: "market",
        description: "محصولات و امانت‌هایی که طرفشان هستم",
        kind: SkillKind::Kernel,
        estimate_tokens: 0,
    },
    SkillDef {
        id: "escrow.status",
        title: "وضعیت امانت",
        category: "market",
        description: "وضعیت یک امانت با شناسه escrow:…",
        kind: SkillKind::Kernel,
        estimate_tokens: 0,
    },
    SkillDef {
        id: "economy.report",
        title: "گزارش اقتصاد",
        category: "nation",
        description: "شفافیت مادهٔ ۶: عرضه، خزانه، قفل امانت",
        kind: SkillKind::Kernel,
        estimate_tokens: 0,
    },
    SkillDef {
        id: "kb.search",
        title: "جست‌وجوی دانش",
        category: "brain",
        description: "جست‌وجوی پایگاه دانش کیان",
        kind: SkillKind::Kernel,
        estimate_tokens: 0,
    },
    SkillDef {
        id: "deposits.mine",
        title: "واریزهای من",
        category: "wallet",
        description: "وضعیت واریزهای TON من",
        kind: SkillKind::Kernel,
        estimate_tokens: 0,
    },
    SkillDef {
        id: "hermes.help",
        title: "راهنمای هرمس",
        category: "brain",
        description: "فهرست مهارت‌های در دسترس",
        kind: SkillKind::Kernel,
        estimate_tokens: 0,
    },
    SkillDef {
        id: "identity.report",
        title: "اعتبار من",
        category: "brain",
        description: "امتیاز هویتی من از شمارش ران‌های سالم/مسدود",
        kind: SkillKind::Kernel,
        estimate_tokens: 0,
    },
    SkillDef {
        id: "ai.chat",
        title: "گفت‌وگو با مغز",
        category: "brain",
        description: "گفت‌وگوی آزاد با مغز کیان روی گیت‌وی فال‌بک",
        kind: SkillKind::Llm,
        estimate_tokens: 900,
    },
    SkillDef {
        id: "market.describe",
        title: "توصیف محصول",
        category: "market",
        description: "تولید توضیح فروش‌پذیر برای محصول من",
        kind: SkillKind::Llm,
        estimate_tokens: 600,
    },
    SkillDef {
        id: "support.faq",
        title: "پشتیبانی",
        category: "support",
        description: "پاسخ پشتیبانی بر پایهٔ متن سؤال من",
        kind: SkillKind::Llm,
        estimate_tokens: 500,
    },
    SkillDef {
        id: "nation.explain",
        title: "حاکمیت",
        category: "nation",
        description: "توضیح پارلمان و آستانهٔ ۵۰٬۰۰۰ دیین",
        kind: SkillKind::Llm,
        estimate_tokens: 400,
    },
    SkillDef {
        id: "charter.explain",
        title: "منشور",
        category: "nation",
        description: "توضیح مواد منشور کیان",
        kind: SkillKind::Llm,
        estimate_tokens: 400,
    },
    SkillDef {
        id: "growth.quest",
        title: "کوئست رشد",
        category: "growth",
        description: "پیشنهاد کوئست رشد متناسب با من",
        kind: SkillKind::Llm,
        estimate_tokens: 500,
    },
];

/// مهارت با شناسه.
#[must_use]
pub fn find_skill(id: &str) -> Option<&'static SkillDef> {
    SKILLS.iter().find(|s| s.id == id)
}

/// مهارت‌های فعال برای بذر دیتابیس — ترتیب ثابت.
#[must_use]
pub fn all_skills() -> &'static [SkillDef; 16] {
    &SKILLS
}

#[cfg(test)]
mod tests {
    use super::*;

    /// دقیقاً ۱۶ مهارت، شناسه‌ها یکتا، هر مهارت metadata کامل.
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

    /// شناسه‌ها فرمت دامنه‌ای دارند (نقطهٔ دسته‌بندی).
    #[test]
    fn ids_are_dotted_lowercase() {
        for s in &SKILLS {
            assert!(s.id.split('.').count() == 2, "id must be category.name: {}", s.id);
            assert!(s.id.chars().all(|c| c.is_ascii_lowercase() || c == '.' || c.is_ascii_digit()));
        }
    }
}
