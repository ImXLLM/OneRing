//! متادیتای KYA و قیمت‌گذاری x402 — پورت `04_0xkiabot_security_x402_kya`
//! (پوشهٔ ۰۴ گنجینه).
//!
//! * [`KyaMetadata`] — شناسنامهٔ Know-Your-Agent (سبک ERC-8004): هر ایجنت
//!   فعال با توان‌نامهٔ امضای ES256 و نقش‌های مجازش روی سیم توصیف می‌شود؛
//!   همین ساختار پایهٔ `/.well-known/agent-card.json` و رجیستری ERC-8004 است.
//! * [`X402Pricing`] — دروازهٔ پرداخت HTTP-402 (نسخهٔ ۲): قیمت‌گذاری بر حسب
//!   SBC روی شبکهٔ Radius تست‌نت، بدون خواندن هرگز کلید خصوصی — فقط
//!   آدرس عمومی فروشنده (`payTo`).
//!
//! هر دو ساختار فقط داده‌اند (serde) — امضادانی و شمارش روی مسیرهای واقعی
//! بعدها سیم می‌شود؛ اینجا قراردادِ قطعی و تست‌شده.
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// الگوریتم امضای پذیرفته‌شده — طبق مشخصهٔ KYA فقط ES256 (منحنی P-256).
pub const KYA_SUPPORTED_JWA: &str = "ES256";

/// شناسهٔ شبکهٔ x402 — Radius Testnet (عین `NETWORK` پایتون).
pub const X402_NETWORK: &str = "eip155:72344";

/// نسخهٔ پروتکل x402 (عین `X402_VERSION` پایتون).
pub const X402_VERSION: u32 = 2;

/// شناسنامهٔ Know-Your-Agent یک ایجنت ناوگان.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KyaMetadata {
    /// شناسهٔ ایجنت (`agent:merchant` …)
    pub agent_id: String,
    /// نقش ناوگان
    pub role: String,
    /// مدل توان‌نامهٔ ERC-8004
    pub registry: String,
    /// الگوریتم امضا — فقط ES256
    pub algorithm: String,
    /// مسیر JWKS عمومی ایجنت (روی سیم منتشر می‌شود؛ هرگز کلید خصوصی نیست)
    pub jwks_path: String,
    /// دامنه‌های مجاز ایجنت (اسکوپ عملیات)
    pub scopes: Vec<String>,
    /// نسخهٔ شناسنامه
    pub version: u32,
}

impl KyaMetadata {
    /// شناسنامهٔ ایجنت فعال از نقش — اسکوپ‌ها از نقش مشتق می‌شوند.
    ///
    /// # Errors
    /// نقش `registered` (نه فعال) — ایجنت غیرفعال شناسنامهٔ KYA نمی‌گیرد.
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
            "guardian" => vec!["security.audit", "surface.verify", "stall.enforce"]
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

    /// شناسنامه‌های همهٔ ایجنت‌های فعال ناوگان — ترتیب پایدار [`crate::roles::ROLES`]
    /// (نقش‌های `registered` شناسنامه نمی‌گیرند — fail-closed).
    ///
    /// # Errors
    /// هرگز در ترتیب فعلی (همهٔ ورودی‌ها فعال‌اند) — ولی امضا `Result` می‌ماند
    /// تا افزودن نقشِ نیم‌فعال، کامپایل نشکند.
    pub fn for_fleet() -> Result<Vec<Self>, String> {
        crate::roles::active_roles().map(Self::for_role).collect()
    }
}

/// دروازهٔ پرداخت x402 v2 برای فراخوانی‌های A2A پول‌دار.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct X402Pricing {
    /// نسخهٔ پروتکل (همیشه ۲)
    pub version: u32,
    /// آدرس عمومی دریافت‌کننده (`payTo`) — کلید خصوصی هرگز اینجا نیست
    pub pay_to: String,
    /// شناسهٔ شبکهٔ EIP-155
    pub network: String,
    /// قرارداد دارایی (SBC روی Radius تست‌نت)
    pub asset: String,
    /// قیمت پیش‌فرض بر حسب واحدهای دارایی (۶ رقم اعشار — عین پایتون)
    pub price_sbc: String,
    /// سقف قیمت مجاز برای جلوگیری از خرابی درگاه
    pub max_price_sbc: String,
    /// روش انتقال EIP-3009 (`transferWithAuthorization`)
    pub asset_transfer_method: String,
}

impl Default for X402Pricing {
    fn default() -> Self {
        Self {
            version: X402_VERSION,
            pay_to: String::new(), // از env در بوت؛ هرگز کلید خصوصی
            network: X402_NETWORK.to_owned(),
            asset: "0x33ad9e4BD16B69B5BFdED37D8B5D9fF9aba014Fb".to_owned(),
            price_sbc: "0.001".to_owned(),
            max_price_sbc: "1".to_owned(),
            asset_transfer_method: "eip3009".to_owned(),
        }
    }
}

impl X402Pricing {
    /// قیمت‌گذاری با آدرس فروشنده — تنها ورودیِ لازم از ولت.
    #[must_use]
    pub fn new(pay_to: &str) -> Self {
        Self { pay_to: pay_to.to_owned(), ..Self::default() }
    }

    /// قیمت مجاز؟ — بین صفر و سقف (مقایسهٔ اعشاریِ رشته‌ای ساده و قطعی).
    #[must_use]
    pub fn accepts(&self, price_sbc: &str) -> bool {
        let p = parse_decimal(price_sbc);
        let max = parse_decimal(&self.max_price_sbc);
        p > 0 && p <= max
    }
}

/// تجزیهٔ اعشاری قطعی به واحدهای ۶رقمی (بدون float) — عین `Decimal` پایتون.
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

    /// شناسنامهٔ KYA برای همهٔ ۷ ایجنت فعال ساخته می‌شود — ناوگان ۷/۷ فعال.
    #[test]
    fn kya_metadata_for_active_fleet_only() {
        let fleet = KyaMetadata::for_fleet().expect("fleet");
        assert_eq!(fleet.len(), 7, "همهٔ ایجنت‌های فعال شناسنامه می‌گیرند");
        for m in &fleet {
            assert_eq!(m.algorithm, "ES256");
            assert_eq!(m.registry, "erc8004");
            assert!(m.jwks_path.starts_with("/.well-known/agents/"));
            assert!(!m.scopes.is_empty());
        }
        // نگهبان حالا فعال است — اسکوپ‌های امنیت/راستی‌آزمایی دارد.
        let guardian = ROLES.iter().find(|r| r.role == "guardian").expect("guardian");
        let g = KyaMetadata::for_role(guardian).expect("guardian is now active");
        assert!(g.scopes.contains(&"security.audit".to_owned()));
        assert!(g.scopes.contains(&"surface.verify".to_owned()));
    }

    /// اسکوپ‌ها از نقش مشتق می‌شوند و بدون کلید خصوصی‌اند.
    #[test]
    fn scopes_are_role_derived_and_secret_free() {
        let merchant = ROLES.iter().find(|r| r.role == "merchant").expect("merchant");
        let m = KyaMetadata::for_role(merchant).expect("merchant kya");
        assert!(m.scopes.contains(&"market.describe".to_owned()));
        let fin = ROLES.iter().find(|r| r.role == "finance").expect("finance");
        let f = KyaMetadata::for_role(fin).expect("finance kya");
        assert!(f.scopes.contains(&"ledger.reconcile".to_owned()));
        // هیچ اسکوپی «key» یا «secret» ندارد — قرارداد کلید-محوری
        for m in KyaMetadata::for_fleet().expect("fleet") {
            for s in &m.scopes {
                assert!(!s.contains("key"), "scope با کلید ممنوع: {s}");
            }
        }
    }

    /// دروازهٔ x402 — پیش‌فرض‌های پایتون و پذیرش سقف‌دار.
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
        assert!(!p.accepts("0"), "صفر مجاز نیست");
        assert!(!p.accepts("1.000001"), "بالای سقف رد می‌شود");
        assert!(!p.accepts("2"));
        assert!(!p.accepts("abc"), "ورودی خراب رد می‌شود");
    }

    /// سریال‌سازی سیم — camelCase و بدون هیچ کلید خصوصی.
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
        assert!(!all.to_lowercase().contains("private"), "بدون کلید خصوصی");
        assert!(!all.to_lowercase().contains("secret"), "بدون راز");
    }
}
