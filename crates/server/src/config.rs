//! پیکربندی بوت سرور — فقط از env؛ هیچ secret در کد (فیلتر REA: `text.credential_markers`).
//!
//! fail-closed: کلید ادمین تنظیم‌نشده یعنی اندپوینت‌های ادمین **غیرفعال** (هرگز باز) —
//! تصمیم در `verify_kian_key` + `require_admin` گرفته می‌شود، اینجا فقط خوانده می‌شود.
//! همین قرارداد برای `KIAN_BOT_TOKEN` (هویت تلگرام M3) و اسلات‌های AI (M4) برقرار است.

use kian_ai::AiConfig;

/// پیکربندی بوت — تغییرناپذیر پس از شروع.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// کلید ادمین — از `KIAN_ADMIN_KEY`؛ `None` = ادمین غیرفعال (fail-closed).
    /// حداقل ۱۶ کاراکتر توسط گارد قراردادی تضمین می‌شود.
    pub admin_key: Option<String>,
    /// توکن بات تلگرام — از `KIAN_BOT_TOKEN`؛ `None` = هویت تلگرام غیرفعال
    /// (fail-closed؛ همان قرارداد ادمین). هرگز لاگ یا در پاسخ نمی‌رود.
    pub bot_token: Option<String>,
    /// راز وبهوک تلگرام — از `KIAN_BOT_WEBHOOK_SECRET`؛ `None` = وبهوک بات
    /// غیرفعال (fail-closed فاز ۵). تلگرام در هر آپدیت هدر
    /// `X-Telegram-Bot-Api-Secret-Token` می‌فرستد؛ مقایسهٔ زمان‌ثابت.
    pub bot_webhook_secret: Option<String>,
    /// URL مینی‌اپ — از `KIAN_MINIAPP_URL`؛ پیش‌فرض localhost:3000. دکمهٔ
    /// «ورود به مینی‌اپ» در پاسخ‌های بات از همین می‌آید.
    pub miniapp_url: String,
    /// عمر نشست bearer به ساعت — از `KIAN_SESSION_TTL_HOURS`؛ پیش‌فرض ۱۶۸ (۷ روز).
    pub session_ttl_hours: i64,
    /// آدرس خزانهٔ TON برای واریزها — از `KIAN_TON_TREASURY`؛ پیش‌فرض رشتهٔ خالی
    /// (فرانت از `/health`-مانند endpoint استفاده نمی‌کند؛ فقط سمت کلاینت env دارد).
    pub ton_treasury: String,
    /// نرخ تبدیل دیین به تون — از `KIAN_TON_TO_DIN`؛ پیش‌فرض ۱۰۰۰ دیین به تون.
    /// محاسبهٔ ضرب: `micro_ton * rate / 1_000_000` — حذف اعشار به کف (حداقل ۱).
    pub ton_to_din_rate: i64,
    /// نرخ تبدیل دیین به USDT — از `KIAN_USDT_TO_DIN`؛ پیش‌فرض ۱۰۰۰ Đ به USDT
    /// (هم‌مقیاس با نرخ TON). ضرب: `micro_usdt * rate / 1_000_000`.
    pub usdt_to_din_rate: i64,
    /// نصاب پارلمان — از `KIAN_NATION_QUORUM`؛ پیش‌فرض ۳.
    pub nation_quorum: i64,
    /// آدرس خزانهٔ هر زنجیره — از `KIAN_TREASURY_TRX/SOL/TON/BSC`؛ خالی یعنی
    /// تنظیم‌نشده (شفاف در `/wallet/treasury`، نه مخفی).
    pub treasury_trx: String,
    /// خزانه سولانا
    pub treasury_sol: String,
    /// خزانه تون
    pub treasury_ton: String,
    /// خزانه بایننس‌چین
    pub treasury_bsc: String,
    /// گیت‌وی AI (M4) — سه اسلات فال‌بک + سقف روزانه؛ از env با [`AiConfig::from_env`].
    /// کلیدهای اسلات هرگز لاگ یا روی سیم نمی‌روند.
    pub ai: AiConfig,
    /// دیتابیس sqlite — از `KIAN_DATABASE_URL`؛ پیش‌فرض فایل محلی. فرمت:
    /// مسیر خام (`data/kian.sqlite`)، `:memory:` یا `sqlite::memory:` —
    /// `V3Store::connect` کل رشته را filename می‌گیرد (بدون scheme/query).
    pub database_url: String,
    /// هاست bind — از `KIAN_HOST`؛ پیش‌فرض 127.0.0.1.
    pub host: String,
    /// پورت bind — از `KIAN_PORT`؛ پیش‌فرض 8088.
    pub port: u16,
    /// سقف درخواست در پنجرهٔ ۶۰ثانیه‌ای — از `KIAN_RATE_LIMIT`؛ پیش‌فرض ۱۲۰.
    pub rate_limit: usize,
    /// اعتماد به `X-Forwarded-For` — از `KIAN_TRUST_PROXY`؛ پیش‌فرض true
    /// (استقرار پشت پروکسی معتبر مثل Railway که XFF را می‌نویسد). false =
    /// فقط IP سوکت ملاک است — استقرار مستقیم: هر کلاینت با XFF جعلی می‌توانست
    /// سقف نرخ را دور بزند (اصلاح امنیتی تست بار ۲۰۲۵-۱۱).
    pub trust_proxy: bool,
    /// پوشهٔ بکاپ‌های `VACUUM INTO` — از `KIAN_BACKUP_DIR`؛ پیش‌فرض data/backups.
    pub backup_dir: String,
    /// تعداد بکاپ نگه‌داشتنی — از `KIAN_KEEP_BACKUPS`؛ پیش‌فرض ۷.
    pub keep_backups: usize,
    /// اوریجین‌های مجاز CORS (مینی‌اپ وب) — از `KIAN_CORS_ORIGIN` با ویرگول؛
    /// پیش‌فرض فقط localhost:3000. fail-closed: اوریجین ناشناخته هرگز پاسخ
    /// preflight نمی‌گیرد.
    pub cors_origins: Vec<String>,
}

impl ServerConfig {
    /// پایهٔ fail-closed — مبنای `from_env` و سازندهٔ تست‌ها: همه‌چیز خاموش،
    /// bind محلی، بدون ادمین، بدون اسلات AI.
    #[must_use]
    pub fn base() -> Self {
        Self {
            admin_key: None,
            bot_token: None,
            bot_webhook_secret: None,
            miniapp_url: "http://localhost:3000".into(),
            session_ttl_hours: 168,
            ton_treasury: String::new(),
            ton_to_din_rate: 1000,
            usdt_to_din_rate: 1000,
            nation_quorum: 3,
            treasury_trx: String::new(),
            treasury_sol: String::new(),
            treasury_ton: String::new(),
            treasury_bsc: String::new(),
            ai: AiConfig {
                slots: Vec::new(),
                daily_cap_tokens: 50_000,
            },
            database_url: "sqlite::memory:".into(),
            host: "127.0.0.1".into(),
            port: 0,
            rate_limit: 10_000,
            trust_proxy: true,
            backup_dir: "/tmp/kian-test-backups".into(),
            keep_backups: 1,
            cors_origins: Vec::new(),
        }
    }

    /// مسیر فایل دیتابیس‌های فایل‌محور — `None` برای حافظه.
    /// همان نرمال‌سازی `V3Store::connect`: حذف پیشوند `file:` و تشخیص `:memory:`.
    #[must_use]
    pub fn db_file_path(&self) -> Option<std::path::PathBuf> {
        let normalized = self
            .database_url
            .strip_prefix("file:")
            .unwrap_or(&self.database_url);
        if normalized == ":memory:" || normalized == "sqlite::memory:" {
            return None;
        }
        let file = normalized.split('?').next().unwrap_or(normalized);
        if file.trim().is_empty() {
            return None;
        }
        Some(std::path::PathBuf::from(file))
    }

    /// خواندن از محیط — هیچ چیز fail نمی‌شود؛ ادمین‌نبودن فقط «غیرفعال» است.
    #[must_use]
    pub fn from_env() -> Self {
        let env = |k: &str| std::env::var(k).ok().filter(|v| !v.trim().is_empty());
        Self {
            admin_key: env("KIAN_ADMIN_KEY"),
            bot_token: env("KIAN_BOT_TOKEN"),
            bot_webhook_secret: env("KIAN_BOT_WEBHOOK_SECRET"),
            miniapp_url: env("KIAN_MINIAPP_URL").unwrap_or_else(|| "http://localhost:3000".into()),
            session_ttl_hours: env("KIAN_SESSION_TTL_HOURS")
                .and_then(|p| p.parse().ok())
                .unwrap_or(168),
            ton_treasury: env("KIAN_TON_TREASURY").unwrap_or_default(),
            ton_to_din_rate: env("KIAN_TON_TO_DIN")
                .and_then(|p| p.parse().ok())
                .unwrap_or(1000),
            usdt_to_din_rate: env("KIAN_USDT_TO_DIN")
                .and_then(|p| p.parse().ok())
                .unwrap_or(1000),
            nation_quorum: env("KIAN_NATION_QUORUM")
                .and_then(|p| p.parse().ok())
                .unwrap_or(3),
            treasury_trx: env("KIAN_TREASURY_TRX").unwrap_or_default(),
            treasury_sol: env("KIAN_TREASURY_SOL").unwrap_or_default(),
            treasury_ton: env("KIAN_TREASURY_TON").unwrap_or_default(),
            treasury_bsc: env("KIAN_TREASURY_BSC").unwrap_or_default(),
            ai: AiConfig::from_env(),
            database_url: env("KIAN_DATABASE_URL").unwrap_or_else(|| "data/kian.sqlite".into()),
            host: env("KIAN_HOST").unwrap_or_else(|| "127.0.0.1".into()),
            port: env("KIAN_PORT")
                .and_then(|p| p.parse().ok())
                .unwrap_or(8088),
            rate_limit: env("KIAN_RATE_LIMIT")
                .and_then(|p| p.parse().ok())
                .unwrap_or(120),
            trust_proxy: env("KIAN_TRUST_PROXY").map_or(
                true,
                |v| !matches!(v.to_ascii_lowercase().as_str(), "0" | "false" | "no" | "off"),
            ),
            backup_dir: env("KIAN_BACKUP_DIR").unwrap_or_else(|| "data/backups".into()),
            keep_backups: env("KIAN_KEEP_BACKUPS")
                .and_then(|p| p.parse().ok())
                .unwrap_or(7),
            cors_origins: env("KIAN_CORS_ORIGIN").map_or_else(
                || {
                    vec![
                        "http://localhost:3000".into(),
                        "http://127.0.0.1:3000".into(),
                    ]
                },
                |raw| {
                    raw.split(',')
                        .map(str::trim)
                        .filter(|o| !o.is_empty())
                        .map(str::to_owned)
                        .collect()
                },
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// پیش‌فرض‌های امن: ادمین غیرفعال، bind محلی، سقف نرخ معقول.
    #[test]
    fn defaults_are_fail_closed() {
        // در محیط تست متغیرها ممکن است تنظیم شده باشند — فقط ساختار را چک می‌کنیم
        let cfg = ServerConfig::from_env();
        if std::env::var("KIAN_ADMIN_KEY").is_err() {
            assert!(
                cfg.admin_key.is_none(),
                "unset admin key must stay disabled"
            );
        }
        assert!(cfg.rate_limit > 0);
        assert!(cfg.keep_backups > 0);
    }
}
