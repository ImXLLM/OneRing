//! کارت کشف A2A — نمای عمومی ایجنت کیان برای ایجنت‌های بیرونی.
//!
//! قرارداد: هیچ کلید، توکن یا نشستی در کارت نیست — فقط نام، توضیح، url،
//! نسخه‌ها و فهرست مهارت‌ها. نسخهٔ پروتکل صادقانه `a2a/1.0-kian` است (پیاده‌سازی
//! خودِ کیان از مفهوم A2A، نه ادعای سازگاری رسمی).

/// یک مهارت در کارت.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardSkill {
    /// شناسه مهارت
    pub id: String,
    /// نام نمایشی
    pub name: String,
    /// توضیح
    pub description: String,
    /// برچسب‌ها
    pub tags: Vec<String>,
}

/// کارت ایجنت — JSON عمومی.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentCard {
    /// نام ایجنت
    pub name: String,
    /// توضیح
    pub description: String,
    /// نشانی سرویس (پایهٔ API کرنل)
    pub url: String,
    /// نسخهٔ کیان
    pub version: String,
    /// نسخهٔ پروتکل
    pub protocol_version: &'static str,
    /// سازمان
    pub organization: String,
    /// مهارت‌ها
    pub skills: Vec<CardSkill>,
}

impl AgentCard {
    /// JSON قابل انتشار — ترتیب کلیدها با `BTreeMap` قطعی می‌ماند.
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

/// نسخهٔ پروتکل — پیاده‌سازی کیان.
pub const PROTOCOL_VERSION: &str = "a2a/1.0-kian";

/// ساخت کارت از پارامترهای بوت — مهارت‌ها از هرمس می‌آیند.
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

    /// کارت — شکل درست، بدون هر نشانهٔ اعتبارنامه.
    #[test]
    fn card_is_honest_and_secret_free() {
        let card = build_agent_card(
            "KIAN Kernel",
            "Money kernel + Hermes brain",
            "http://127.0.0.1:8088/api/v1",
            "0.1.0",
            vec![CardSkill {
                id: "wallet.balance".into(),
                name: "تراز من".into(),
                description: "خوانش قطعی دفتر".into(),
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
