//! کوئری‌های مغز M4 — مهارت‌های هرمس، ران‌ها، حافظهٔ ۷لایه، ناوگان ایجنت‌ها،
//! صندوق A2A و خوانش‌های ادمین.
//!
//! قواعد ثابت سر جایش است: هیچ پولی اینجا جابه‌جا نمی‌شود (مغز فقط می‌خواند
//! و شاهد می‌نویسد)؛ راز هرگز در ستونی ذخیره نمی‌شود — ورودی رانِ REDACTED
//! فقط متن پاک‌شده را نگه می‌دارد.

use crate::error::DbResult;
use crate::store::{now_iso, V3Store};
use sqlx::Row;

/// بذر مهارت — دادهٔ خالص از `crates/hermes`؛ صدا زدن سرور هنگام بوت.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillSeed {
    /// شناسهٔ مهارت (`wallet.balance`)
    pub id: String,
    /// عنوان فارسی
    pub title: String,
    /// دسته
    pub category: String,
    /// توضیح
    pub description: String,
    /// `kernel` یا `llm`
    pub kind: String,
    /// برآورد توکن
    pub estimate_tokens: i64,
}

/// ردیف مهارت پس از seed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillRow {
    /// شناسه
    pub id: String,
    /// عنوان
    pub title: String,
    /// دسته
    pub category: String,
    /// توضیح
    pub description: String,
    /// نوع
    pub kind: String,
    /// برآورد توکن
    pub estimate_tokens: i64,
    /// فعال؟
    pub active: bool,
}

/// شاهد یک اجرای مهارت — ورودی/خروجی پس از سپر؛ راز هرگز.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HermesRunRow {
    /// شناسه (`hrun:{uuid}`)
    pub id: String,
    /// کاربر
    pub user_id: String,
    /// مهارت
    pub skill_id: String,
    /// ورودی (پس از redaction اگر redact شد)
    pub input: String,
    /// خروجی — ران مسدودشده رشتهٔ خالی
    pub output: String,
    /// `ALLOW`/`REDACTED`/`BLOCKED`
    pub shield: String,
    /// `hermes.ok`/`hermes.blocked`/`hermes.failed`
    pub outcome: String,
    /// ارائه‌دهنده — مهارت kernel `None`
    pub provider: Option<String>,
    /// مدل — مهارت kernel `None`
    pub model: Option<String>,
    /// توکن ورودی
    pub prompt_tokens: i64,
    /// توکن خروجی
    pub completion_tokens: i64,
    /// مجموع
    pub total_tokens: i64,
    /// زمان ISO — خوانشی؛ درج خودکار با `now_iso()` انجام می‌شود و مقدار این
    /// فیلد در مسیر درج نادیده گرفته می‌شود.
    pub created_at: String,
}

/// یک یادداشت حافظهٔ ۷لایه.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryNoteRow {
    /// شناسه (`mem:{uuid}`)
    pub id: String,
    /// کاربر
    pub user_id: String,
    /// لایه — یکی از ۷ لایهٔ ثابت
    pub layer: String,
    /// محتوا
    pub content: String,
}

/// ردیف ایجنت ناوگان.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentRow {
    /// شناسه (`agent:{role}`)
    pub id: String,
    /// نام
    pub name: String,
    /// نقش
    pub role: String,
    /// وضعیت صادقانه (`active`/`registered`/`disabled`)
    pub status: String,
    /// توضیح
    pub description: String,
}

/// پیام A2A.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct A2aRow {
    /// شناسه (`a2a:{uuid}`)
    pub id: String,
    /// فرستنده
    pub from_agent: String,
    /// گیرنده
    pub to_agent: String,
    /// نوع (`handoff`/`query`/`result`)
    pub kind: String,
    /// بدنه
    pub body: String,
    /// زمان ISO — غایب یعنی «همین حالا»
    pub created_at: Option<String>,
}

impl V3Store {
    /// بذر مهارت‌ها — upsert متادیتا، پرچم `active` را دست نمی‌زند
    /// (اگر ادمین مهارتی را خاموش کرده باشد بوت دوباره روشنش نمی‌کند).
    ///
    /// # Errors
    /// خطای SQL.
    pub async fn seed_skills(&self, skills: &[SkillSeed]) -> DbResult<()> {
        for s in skills {
            sqlx::query(
                r#"INSERT INTO "HermesSkill" ("id","title","category","description","kind","estimateTokens","active")
                   VALUES (?,?,?,?,?,?,true)
                   ON CONFLICT("id") DO UPDATE SET
                     "title"=excluded."title", "category"=excluded."category",
                     "description"=excluded."description", "kind"=excluded."kind",
                     "estimateTokens"=excluded."estimateTokens""#,
            )
            .bind(&s.id)
            .bind(&s.title)
            .bind(&s.category)
            .bind(&s.description)
            .bind(&s.kind)
            .bind(s.estimate_tokens)
            .execute(self.pool_ref())
            .await?;
        }
        Ok(())
    }

    /// فهرست مهارت‌ها — `active_only` برای سیم عمومی، `false` برای ادمین.
    ///
    /// # Errors
    /// خطای SQL.
    pub async fn list_skills(&self, active_only: bool) -> DbResult<Vec<SkillRow>> {
        let sql = if active_only {
            r#"SELECT "id","title","category","description","kind","estimateTokens","active"
               FROM "HermesSkill" WHERE "active"=true ORDER BY "id""#
        } else {
            r#"SELECT "id","title","category","description","kind","estimateTokens","active"
               FROM "HermesSkill" ORDER BY "id""#
        };
        let rows = sqlx::query(sql).fetch_all(self.pool_ref()).await?;
        Ok(rows
            .into_iter()
            .map(|r| SkillRow {
                id: r.get("id"),
                title: r.get("title"),
                category: r.get("category"),
                description: r.get("description"),
                kind: r.get("kind"),
                estimate_tokens: r.get("estimateTokens"),
                active: r.get("active"),
            })
            .collect())
    }

    /// درج شاهد اجرا — پس از هر ران (پایهٔ امتیاز هویتی و دنبالهٔ ادمین).
    ///
    /// # Errors
    /// خطای SQL.
    pub async fn insert_hermes_run(&self, r: &HermesRunRow) -> DbResult<()> {
        sqlx::query(
            r#"INSERT INTO "HermesRun"
               ("id","userId","skillId","input","output","shield","outcome","provider","model","promptTokens","completionTokens","totalTokens","createdAt")
               VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?)"#,
        )
        .bind(&r.id)
        .bind(&r.user_id)
        .bind(&r.skill_id)
        .bind(&r.input)
        .bind(&r.output)
        .bind(&r.shield)
        .bind(&r.outcome)
        .bind(&r.provider)
        .bind(&r.model)
        .bind(r.prompt_tokens)
        .bind(r.completion_tokens)
        .bind(r.total_tokens)
        .bind(now_iso())
        .execute(self.pool_ref())
        .await?;
        Ok(())
    }

    /// ران‌های یک کاربر — جدیدترین اول.
    ///
    /// # Errors
    /// خطای SQL.
    pub async fn list_hermes_runs(&self, user_id: &str, limit: i64) -> DbResult<Vec<HermesRunRow>> {
        let rows = sqlx::query(
            r#"SELECT "id","userId","skillId","input","output","shield","outcome","provider","model","promptTokens","completionTokens","totalTokens","createdAt"
               FROM "HermesRun" WHERE "userId" = ?
               ORDER BY "createdAt" DESC LIMIT ?"#,
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(self.pool_ref())
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| HermesRunRow {
                id: r.get("id"),
                user_id: r.get("userId"),
                skill_id: r.get("skillId"),
                input: r.get("input"),
                output: r.get("output"),
                shield: r.get("shield"),
                outcome: r.get("outcome"),
                provider: r.get("provider"),
                model: r.get("model"),
                prompt_tokens: r.get("promptTokens"),
                completion_tokens: r.get("completionTokens"),
                total_tokens: r.get("totalTokens"),
                created_at: r.get("createdAt"),
            })
            .collect())
    }

    /// شمارش ران‌های سالم/مسدودشدهٔ کاربر — پایهٔ امتیاز هویتی صادقانه.
    ///
    /// # Errors
    /// خطای SQL.
    pub async fn hermes_run_counts(&self, user_id: &str) -> DbResult<(i64, i64)> {
        let row = sqlx::query(
            r#"SELECT
                 COALESCE(SUM(CASE WHEN "outcome"='hermes.ok' THEN 1 ELSE 0 END),0) AS ok,
                 COALESCE(SUM(CASE WHEN "outcome"='hermes.blocked' THEN 1 ELSE 0 END),0) AS blocked
               FROM "HermesRun" WHERE "userId" = ?"#,
        )
        .bind(user_id)
        .fetch_one(self.pool_ref())
        .await?;
        Ok((row.get("ok"), row.get("blocked")))
    }

    /// نوشتن یادداشت حافظه — لایه از enum هرمس می‌آید و اینجا فقط رشته است.
    ///
    /// # Errors
    /// خطای SQL.
    pub async fn insert_memory_note(&self, r: &MemoryNoteRow) -> DbResult<()> {
        sqlx::query(
            r#"INSERT INTO "MemoryNote" ("id","userId","layer","content","createdAt")
               VALUES (?,?,?,?,?)"#,
        )
        .bind(&r.id)
        .bind(&r.user_id)
        .bind(&r.layer)
        .bind(&r.content)
        .bind(now_iso())
        .execute(self.pool_ref())
        .await?;
        Ok(())
    }

    /// خواندن حافظه — همهٔ لایه‌ها یا یک لایه؛ جدیدترین اول.
    ///
    /// # Errors
    /// خطای SQL.
    pub async fn list_memory_notes(
        &self,
        user_id: &str,
        layer: Option<&str>,
        limit: i64,
    ) -> DbResult<Vec<(String, String, String)>> {
        // خروجی: (id, layer, content)
        let rows = if let Some(layer) = layer {
            sqlx::query(
                r#"SELECT "id","layer","content" FROM "MemoryNote"
                   WHERE "userId" = ? AND "layer" = ? ORDER BY "createdAt" DESC LIMIT ?"#,
            )
            .bind(user_id)
            .bind(layer)
            .bind(limit)
            .fetch_all(self.pool_ref())
            .await?
        } else {
            sqlx::query(
                r#"SELECT "id","layer","content" FROM "MemoryNote"
                   WHERE "userId" = ? ORDER BY "createdAt" DESC LIMIT ?"#,
            )
            .bind(user_id)
            .bind(limit)
            .fetch_all(self.pool_ref())
            .await?
        };
        Ok(rows.into_iter().map(|r| (r.get("id"), r.get("layer"), r.get("content"))).collect())
    }

    /// ثبت ایجنت ناوگان — فقط بار اول؛ وضعیت ادمین هرگز از بوت بازنویسی نمی‌شود.
    ///
    /// # Errors
    /// خطای SQL.
    pub async fn seed_agent(&self, r: &AgentRow) -> DbResult<()> {
        sqlx::query(
            r#"INSERT INTO "Agent" ("id","name","role","status","description")
               VALUES (?,?,?,?,?)
               ON CONFLICT("id") DO NOTHING"#,
        )
        .bind(&r.id)
        .bind(&r.name)
        .bind(&r.role)
        .bind(&r.status)
        .bind(&r.description)
        .execute(self.pool_ref())
        .await?;
        Ok(())
    }

    /// فهرست ناوگان — ثابت به ترتیب نقش.
    ///
    /// # Errors
    /// خطای SQL.
    pub async fn list_agents(&self) -> DbResult<Vec<AgentRow>> {
        let rows = sqlx::query(
            r#"SELECT "id","name","role","status","description" FROM "Agent" ORDER BY "role""#,
        )
        .fetch_all(self.pool_ref())
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| AgentRow {
                id: r.get("id"),
                name: r.get("name"),
                role: r.get("role"),
                status: r.get("status"),
                description: r.get("description"),
            })
            .collect())
    }

    /// ایجنت با نقش — برای گارد فرستنده/گیرندهٔ A2A.
    ///
    /// # Errors
    /// خطای SQL.
    pub async fn get_agent(&self, agent_id: &str) -> DbResult<Option<AgentRow>> {
        let row = sqlx::query(
            r#"SELECT "id","name","role","status","description" FROM "Agent" WHERE "id" = ?"#,
        )
        .bind(agent_id)
        .fetch_optional(self.pool_ref())
        .await?;
        Ok(row.map(|r| AgentRow {
            id: r.get("id"),
            name: r.get("name"),
            role: r.get("role"),
            status: r.get("status"),
            description: r.get("description"),
        }))
    }

    /// ارسال پیام A2A — گیت‌وی ادمین.
    ///
    /// # Errors
    /// خطای SQL.
    pub async fn insert_a2a_message(&self, r: &A2aRow) -> DbResult<()> {
        sqlx::query(
            r#"INSERT INTO "A2aMessage" ("id","fromAgent","toAgent","kind","body","createdAt")
               VALUES (?,?,?,?,?,?)"#,
        )
        .bind(&r.id)
        .bind(&r.from_agent)
        .bind(&r.to_agent)
        .bind(&r.kind)
        .bind(&r.body)
        .bind(r.created_at.clone().unwrap_or_else(now_iso))
        .execute(self.pool_ref())
        .await?;
        Ok(())
    }

    /// صندوق ورودی ایجنت — جدیدترین اول.
    ///
    /// # Errors
    /// خطای SQL.
    pub async fn list_a2a_inbox(&self, agent_id: &str, limit: i64) -> DbResult<Vec<A2aRow>> {
        let rows = sqlx::query(
            r#"SELECT "id","fromAgent","toAgent","kind","body","createdAt"
               FROM "A2aMessage" WHERE "toAgent" = ?
               ORDER BY "createdAt" DESC LIMIT ?"#,
        )
        .bind(agent_id)
        .bind(limit)
        .fetch_all(self.pool_ref())
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| A2aRow {
                id: r.get("id"),
                from_agent: r.get("fromAgent"),
                to_agent: r.get("toAgent"),
                kind: r.get("kind"),
                body: r.get("body"),
                created_at: r.get("createdAt"),
            })
            .collect())
    }

    /// همهٔ واریزهای TON (همهٔ کاربران) — نمای ادمین؛ فیلتر وضعیت اختیاری.
    ///
    /// # Errors
    /// خطای SQL.
    pub async fn list_ton_deposits_all(
        &self,
        status: Option<&str>,
        limit: i64,
    ) -> DbResult<Vec<crate::users::TonDepositRow>> {
        let rows = if let Some(status) = status {
            sqlx::query(
                r#"SELECT "id","userId","txHash","walletAddress","amountTon","status","createdAt","confirmedAt","confirmedBy"
                   FROM "TonDeposit" WHERE "status" = ?
                   ORDER BY "createdAt" DESC LIMIT ?"#,
            )
            .bind(status)
            .bind(limit)
            .fetch_all(self.pool_ref())
            .await?
        } else {
            sqlx::query(
                r#"SELECT "id","userId","txHash","walletAddress","amountTon","status","createdAt","confirmedAt","confirmedBy"
                   FROM "TonDeposit" ORDER BY "createdAt" DESC LIMIT ?"#,
            )
            .bind(limit)
            .fetch_all(self.pool_ref())
            .await?
        };
        Ok(rows
            .into_iter()
            .map(|r| crate::users::TonDepositRow {
                id: r.get("id"),
                user_id: r.get("userId"),
                tx_hash: r.get("txHash"),
                wallet_address: r.get("walletAddress"),
                amount_ton: r.get("amountTon"),
                status: r.get("status"),
                created_at: r.get("createdAt"),
                confirmed_at: r.get("confirmedAt"),
                confirmed_by: r.get("confirmedBy"),
            })
            .collect())
    }
}

/// ردیف جلسهٔ سازندهٔ محصول HTML با AI — نگاشت ۱به‌۱ به جدول `"BuilderSession"` (مهاجرت `0005`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuilderSessionRow {
    /// شناسهٔ یکتا (`bldr:{uuid}`)
    pub id: String,
    /// مالک جلسه (`citizen:{tg_id}`)
    pub account_id: String,
    /// مرحله (`discovery` / `ready` / `published`)
    pub stage: String,
    /// عنوان پیشنهادی محصول
    pub title: String,
    /// توضیح فروش (فرمول CAST)
    pub description: String,
    /// قیمت پیشنهادی به دیین
    pub suggested_price: i64,
    /// درصد پورسانت معرف پیشنهادی (`0..=5000`)
    pub affiliate_bps: u16,
    /// کد کامل HTML تک‌صفحه‌ای تولیدشده
    pub generated_html: String,
    /// شناسهٔ محصول منتشرشده در بازار (در صورت انتشار)
    pub published_product_id: Option<String>,
    /// زمان ایجاد (Unix epoch ثانیه)
    pub created_at: i64,
    /// زمان آخرین تغییر (Unix epoch ثانیه)
    pub updated_at: i64,
}

/// بستهٔ به‌روزرسانی خروجی و وضعیت جلسهٔ سازندهٔ محصول.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuilderSessionPatch {
    /// مرحلهٔ جدید
    pub stage: String,
    /// عنوان پیشنهادی
    pub title: String,
    /// توضیح فروش
    pub description: String,
    /// قیمت پیشنهادی به دیین
    pub suggested_price: i64,
    /// نرخ پورسانت معرف
    pub affiliate_bps: u16,
    /// کد HTML تولیدشده
    pub generated_html: String,
    /// شناسهٔ محصول منتشرشده (در صورت انتشار)
    pub published_product_id: Option<String>,
    /// زمان به‌روزرسانی (Unix epoch ثانیه)
    pub updated_at: i64,
}

/// یک پیام تاریخچهٔ گفتگوی سازندهٔ محصول — جدول `"BuilderMessage"`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuilderMessageRow {
    /// شناسهٔ افزایشی پیام
    pub id: i64,
    /// شناسهٔ جلسهٔ مرتبط
    pub session_id: String,
    /// نقش (`user` / `assistant` / `system`)
    pub role: String,
    /// متن پیام
    pub content: String,
    /// زمان ثبت (Unix epoch ثانیه)
    pub created_at: i64,
}

#[allow(clippy::needless_pass_by_value)] // الگوی sqlx: Row از fetch مصرف می‌شود
fn builder_session_from(r: sqlx::sqlite::SqliteRow) -> BuilderSessionRow {
    let bps_raw: i64 = r.get("affiliateBps");
    BuilderSessionRow {
        id: r.get("id"),
        account_id: r.get("accountId"),
        stage: r.get("stage"),
        title: r.get("title"),
        description: r.get("description"),
        suggested_price: r.get("suggestedPrice"),
        affiliate_bps: u16::try_from(bps_raw).unwrap_or(kian_domain::DEFAULT_AFFILIATE_BPS),
        generated_html: r.get("generatedHtml"),
        published_product_id: r.get("publishedProductId"),
        created_at: r.get("createdAt"),
        updated_at: r.get("updatedAt"),
    }
}

#[allow(clippy::needless_pass_by_value)]
fn builder_message_from(r: sqlx::sqlite::SqliteRow) -> BuilderMessageRow {
    BuilderMessageRow {
        id: r.get("id"),
        session_id: r.get("sessionId"),
        role: r.get("role"),
        content: r.get("content"),
        created_at: r.get("createdAt"),
    }
}

impl V3Store {
    /// ساخت جلسهٔ جدید سازندهٔ محصول.
    ///
    /// # Errors
    /// خطای SQL.
    pub async fn create_builder_session(&self, row: &BuilderSessionRow) -> DbResult<()> {
        sqlx::query(
            r#"INSERT INTO "BuilderSession"
               ("id","accountId","stage","title","description","suggestedPrice",
                "affiliateBps","generatedHtml","publishedProductId","createdAt","updatedAt")
               VALUES (?,?,?,?,?,?,?,?,?,?,?)"#,
        )
        .bind(&row.id)
        .bind(&row.account_id)
        .bind(&row.stage)
        .bind(&row.title)
        .bind(&row.description)
        .bind(row.suggested_price)
        .bind(i64::from(row.affiliate_bps))
        .bind(&row.generated_html)
        .bind(&row.published_product_id)
        .bind(row.created_at)
        .bind(row.updated_at)
        .execute(self.pool_ref())
        .await?;
        Ok(())
    }

    /// خواندن جلسهٔ سازنده فقط اگر متعلق به `account_id` باشد (ایزوله‌سازی مالکیت).
    ///
    /// # Errors
    /// خطای SQL.
    pub async fn get_builder_session(
        &self,
        id: &str,
        account_id: &str,
    ) -> DbResult<Option<BuilderSessionRow>> {
        let row = sqlx::query(
            r#"SELECT "id","accountId","stage","title","description","suggestedPrice",
               "affiliateBps","generatedHtml","publishedProductId","createdAt","updatedAt"
               FROM "BuilderSession"
               WHERE "id" = ? AND "accountId" = ?"#,
        )
        .bind(id)
        .bind(account_id)
        .fetch_optional(self.pool_ref())
        .await?;
        Ok(row.map(builder_session_from))
    }

    /// فهرست جلسات سازندهٔ یک شهروند — جدیدترین اول.
    ///
    /// # Errors
    /// خطای SQL.
    pub async fn list_builder_sessions_for_account(
        &self,
        account_id: &str,
        limit: i64,
    ) -> DbResult<Vec<BuilderSessionRow>> {
        let cap = limit.clamp(1, 100);
        let rows = sqlx::query(
            r#"SELECT "id","accountId","stage","title","description","suggestedPrice",
               "affiliateBps","generatedHtml","publishedProductId","createdAt","updatedAt"
               FROM "BuilderSession"
               WHERE "accountId" = ?
               ORDER BY "updatedAt" DESC, "id" DESC
               LIMIT ?"#,
        )
        .bind(account_id)
        .bind(cap)
        .fetch_all(self.pool_ref())
        .await?;
        Ok(rows.into_iter().map(builder_session_from).collect())
    }

    /// به‌روزرسانی آرتیفکت، متادیتا و مرحلهٔ جلسهٔ سازنده (با بررسی مالکیت).
    ///
    /// خروجی `true` یعنی ردیف متعلق به کاربر پیدا و به‌روز شد.
    ///
    /// # Errors
    /// خطای SQL.
    pub async fn update_builder_session_artifact(
        &self,
        id: &str,
        account_id: &str,
        patch: &BuilderSessionPatch,
    ) -> DbResult<bool> {
        let res = sqlx::query(
            r#"UPDATE "BuilderSession"
               SET "stage" = ?, "title" = ?, "description" = ?, "suggestedPrice" = ?,
                   "affiliateBps" = ?, "generatedHtml" = ?, "publishedProductId" = ?,
                   "updatedAt" = ?
               WHERE "id" = ? AND "accountId" = ?"#,
        )
        .bind(&patch.stage)
        .bind(&patch.title)
        .bind(&patch.description)
        .bind(patch.suggested_price)
        .bind(i64::from(patch.affiliate_bps))
        .bind(&patch.generated_html)
        .bind(&patch.published_product_id)
        .bind(patch.updated_at)
        .bind(id)
        .bind(account_id)
        .execute(self.pool_ref())
        .await?;
        Ok(res.rows_affected() > 0)
    }

    /// افزودن یک پیام جدید به گفتگوی جلسه و برگرداندن شناسهٔ پیام درج‌شده.
    ///
    /// # Errors
    /// خطای SQL.
    pub async fn append_builder_message(
        &self,
        session_id: &str,
        role: &str,
        content: &str,
        created_at: i64,
    ) -> DbResult<i64> {
        let res = sqlx::query(
            r#"INSERT INTO "BuilderMessage" ("sessionId","role","content","createdAt")
               VALUES (?,?,?,?)"#,
        )
        .bind(session_id)
        .bind(role)
        .bind(content)
        .bind(created_at)
        .execute(self.pool_ref())
        .await?;
        Ok(res.last_insert_rowid())
    }

    /// برگرداندن آخرین `limit` پیام یک جلسه به ترتیب صعودی (`id ASC`) برای جلوگیری از انفجار توکن.
    ///
    /// # Errors
    /// خطای SQL.
    pub async fn list_builder_messages(
        &self,
        session_id: &str,
        limit: i64,
    ) -> DbResult<Vec<BuilderMessageRow>> {
        let cap = limit.clamp(1, 100);
        let rows = sqlx::query(
            r#"SELECT "id","sessionId","role","content","createdAt"
               FROM (
                   SELECT "id","sessionId","role","content","createdAt"
                   FROM "BuilderMessage"
                   WHERE "sessionId" = ?
                   ORDER BY "id" DESC
                   LIMIT ?
               )
               ORDER BY "id" ASC"#,
        )
        .bind(session_id)
        .bind(cap)
        .fetch_all(self.pool_ref())
        .await?;
        Ok(rows.into_iter().map(builder_message_from).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn store() -> V3Store {
        let s = V3Store::connect_in_memory().await.expect("connect");
        s.migrate().await.expect("migrate");
        s
    }

    /// چرخهٔ کامل مغز روی دیتابیس واقعی: بذر مهارت idempotent، شاهد ران،
    /// حافظه، ایجنت، پیام A2A و فهرست ادمین واریزها.
    #[tokio::test]
    #[allow(clippy::too_many_lines)] // پوشش جدول‌های مغز — خوانایی تست مهم‌تر است
    async fn brain_tables_round_trip() {
        let store = store().await;

        // مهارت‌ها — بذر دوباره متادیتا را به‌روز می‌کند ولی شمارش ثابت است
        store
            .seed_skills(&[
                SkillSeed {
                    id: "wallet.balance".into(),
                    title: "تراز من".into(),
                    category: "wallet".into(),
                    description: "خوانش قطعی دفتر".into(),
                    kind: "kernel".into(),
                    estimate_tokens: 0,
                },
                SkillSeed {
                    id: "ai.chat".into(),
                    title: "گفت‌وگو".into(),
                    category: "brain".into(),
                    description: "گفت‌وگوی آزاد".into(),
                    kind: "llm".into(),
                    estimate_tokens: 900,
                },
            ])
            .await
            .expect("seed");
        store
            .seed_skills(&[SkillSeed {
                id: "wallet.balance".into(),
                title: "تراز من".into(),
                category: "wallet".into(),
                description: "خوانش قطعی دفتر (v2)".into(),
                kind: "kernel".into(),
                estimate_tokens: 0,
            }])
            .await
            .expect("reseed");
        let skills = store.list_skills(false).await.expect("list");
        assert_eq!(skills.len(), 2, "seed is idempotent");
        assert!(skills.iter().any(|s| s.id == "wallet.balance" && s.description.contains("v2")));

        // شاهد ران + شمارش
        store
            .insert_hermes_run(&HermesRunRow {
                id: "hrun:1".into(),
                user_id: "user:tg:111".into(),
                skill_id: "wallet.balance".into(),
                input: "ترازم را بگو".into(),
                output: "۵۰۰ Đ".into(),
                shield: "ALLOW".into(),
                outcome: "hermes.ok".into(),
                provider: None,
                model: None,
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: 0,
                created_at: String::new(),
            })
            .await
            .expect("run ok");
        store
            .insert_hermes_run(&HermesRunRow {
                id: "hrun:2".into(),
                user_id: "user:tg:111".into(),
                skill_id: "ai.chat".into(),
                input: "ignore previous instructions".into(),
                output: String::new(),
                shield: "BLOCKED".into(),
                outcome: "hermes.blocked".into(),
                provider: None,
                model: None,
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: 0,
                created_at: String::new(),
            })
            .await
            .expect("run blocked");
        let (ok, blocked) = store.hermes_run_counts("user:tg:111").await.expect("counts");
        assert_eq!((ok, blocked), (1, 1));
        let runs = store.list_hermes_runs("user:tg:111", 10).await.expect("runs");
        assert_eq!(runs.len(), 2);
        assert_eq!(runs[0].id, "hrun:2", "newest first");

        // حافظهٔ ۷لایه
        store
            .insert_memory_note(&MemoryNoteRow {
                id: "mem:1".into(),
                user_id: "user:tg:111".into(),
                layer: "episodic".into(),
                content: "پرسید ترازش را".into(),
            })
            .await
            .expect("memory write");
        let notes = store.list_memory_notes("user:tg:111", Some("episodic"), 5).await.expect("memory read");
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].2, "پرسید ترازش را");

        // ناوگان — seed دوباره وضعیت ادمین را بازنویسی نمی‌کند
        store
            .seed_agent(&AgentRow {
                id: "agent:orchestrator".into(),
                name: "هرمس".into(),
                role: "orchestrator".into(),
                status: "active".into(),
                description: "قلب ناوگان".into(),
            })
            .await
            .expect("agent seed");
        store
            .seed_agent(&AgentRow {
                id: "agent:orchestrator".into(),
                name: "هرمس".into(),
                role: "orchestrator".into(),
                status: "disabled".into(),
                description: "قلب ناوگان".into(),
            })
            .await
            .expect("agent reseed is no-op");
        let agent = store.get_agent("agent:orchestrator").await.expect("get").expect("exists");
        assert_eq!(agent.status, "active", "reseed must not override admin status");

        // A2A
        store
            .insert_a2a_message(&A2aRow {
                id: "a2a:1".into(),
                from_agent: "agent:orchestrator".into(),
                to_agent: "agent:support".into(),
                kind: "handoff".into(),
                body: "پاسخ به شهروند ۱۱۱".into(),
                created_at: None,
            })
            .await
            .expect("a2a insert");
        let inbox = store.list_a2a_inbox("agent:support", 10).await.expect("inbox");
        assert_eq!(inbox.len(), 1);
        assert_eq!(inbox[0].from_agent, "agent:orchestrator");

        // واریزهای ادمین — خالی هم پاسخ سالم است
        let deps = store.list_ton_deposits_all(Some("PENDING"), 50).await.expect("deposits");
        assert!(deps.is_empty(), "no pending deposits at the start");
    }

    #[tokio::test]
    async fn builder_sessions_ownership_and_sliding_window_messages() {
        let store = store().await;
        let session = BuilderSessionRow {
            id: "bldr:1".into(),
            account_id: "citizen:100".into(),
            stage: "discovery".into(),
            title: String::new(),
            description: String::new(),
            suggested_price: 100,
            affiliate_bps: 1_000,
            generated_html: String::new(),
            published_product_id: None,
            created_at: 1_700_000_000,
            updated_at: 1_700_000_000,
        };
        store.create_builder_session(&session).await.expect("create");

        // مالکیت: کاربر دیگر نمی‌تواند جلسه را بخواند یا تغییر دهد
        assert!(store.get_builder_session("bldr:1", "citizen:999").await.unwrap().is_none());
        let owned = store
            .get_builder_session("bldr:1", "citizen:100")
            .await
            .unwrap()
            .expect("owner sees session");
        assert_eq!(owned.stage, "discovery");

        // به‌روزرسانی آرتیفکت
        let updated = store
            .update_builder_session_artifact(
                "bldr:1",
                "citizen:100",
                &BuilderSessionPatch {
                    stage: "ready".into(),
                    title: "محاسبه‌گر هوشمند".into(),
                    description: "ابزار تعاملی".into(),
                    suggested_price: 250,
                    affiliate_bps: 1_500,
                    generated_html: "<!DOCTYPE html><html><body>ok</body></html>".into(),
                    published_product_id: None,
                    updated_at: 1_700_000_123,
                },
            )
            .await
            .unwrap();
        assert!(updated);
        let reloaded = store.get_builder_session("bldr:1", "citizen:100").await.unwrap().unwrap();
        assert_eq!(reloaded.stage, "ready");
        assert_eq!(reloaded.suggested_price, 250);
        assert_eq!(reloaded.affiliate_bps, 1_500);

        // ۱۵ پیام درج می‌کنیم و پنجرهٔ ۱۲تایی می‌خوانیم → باید پیام‌های ۴ تا ۱۵ به ترتیب صعودی برگردند
        for i in 1..=15 {
            store
                .append_builder_message("bldr:1", "user", &format!("msg-{i}"), 1_700_000_000 + i)
                .await
                .unwrap();
        }
        let msgs = store.list_builder_messages("bldr:1", 12).await.unwrap();
        assert_eq!(msgs.len(), 12);
        assert_eq!(msgs[0].content, "msg-4");
        assert_eq!(msgs[11].content, "msg-15");
    }
}
