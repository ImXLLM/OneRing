//! قراردادهای وایرِ انحصارِ حاکم — ژنومِ چهارمدلی، منشورِ روزِ اول و خزانهٔ
//! Base (Wave 7 — Step 4/4).
//!
//! سه DTO روی وایر:
//! - [`MonopolyGenomeMatrixResponse`] — ماتریسِ ۴مدل × ۹فایل با وضعیتِ
//!   تأییدِ ۳۶فایلی و قواعدِ مسیریابی؛
//! - [`MonopolyCharterAndBaseResponse`] — منشورِ چهارجریانیِ روزِ اول +
//!   پیکربندیِ Base Mainnet (`8453`) + خلاصهٔ ممیزیِ هر دو قراردادِ Solidity؛
//! - [`MonopolySimulateSplitRequest`] / [`MonopolySimulateSplitResponse`] —
//!   اجرایِ `compute_day_one_atomic_split` (فروشِ معرف‌دار، فروشِ مستقیمِ
//!   بی‌معرف با قانونِ `2000 bps`، یا بوستِ ۱۰۰٪) + اثباتِ صفرجمع +
//!   URI پرداختِ EIP-681 روی Base.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// ── ماتریسِ ژنوم ─────────────────────────────────────────────────────────────

/// وضعیتِ تأییدِ یک مدل در ماتریسِ ژنوم.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MonopolyGenomeModelView {
    /// شناسهٔ دقیقِ مدل (`GPT 6-Web`, …).
    pub model_id: String,
    /// شناسه‌های alias — همه حل می‌شوند.
    pub aliases: Vec<String>,
    /// پوشهٔ ژنوم زیر `OMNI-MODEL-GENOME/models/`.
    pub genome_dir: String,
    /// تخصصِ حاکم.
    pub specialty: String,
    /// نامِ ۹ فایلِ canonical.
    pub canonical_files: Vec<String>,
    /// تعدادِ فایل‌هایِ تأییدشده (۰ تا ۹).
    pub files_verified: usize,
    /// مدل کامل وقتی `9/9`.
    pub is_complete: bool,
    /// آواتارِ ترامینیتیِ پیش‌فرض (`Alpha` | `Logic-Prime` | `Mairon`).
    pub trinity_avatar: String,
    /// منبعِ تأیید — مسیرِ دیسکی یا `builtin-registry`.
    pub verified_against: String,
}

/// قاعدهٔ مسیریابیِ دامنه → قهرمان.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MonopolyRoutingRuleView {
    /// دامنهٔ کار (`web-synthesis`, …).
    pub domain: String,
    /// مدلِ قهرمانِ مطلق.
    pub champion_model_id: String,
    /// آواتارِ ترامینیتیِ مقصد.
    pub trinity_avatar: String,
}

/// پاسخِ `GET /api/v1/monopoly/genome-matrix`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MonopolyGenomeMatrixResponse {
    /// امضایِ موتورِ انحصار.
    pub signature: String,
    /// چهار مدل با وضعیتِ تأییدِ ۹فایلی.
    pub models: Vec<MonopolyGenomeModelView>,
    /// قواعدِ مسیریابیِ دامنه → قهرمان.
    pub routing_rules: Vec<MonopolyRoutingRuleView>,
    /// کلِ فایل‌هایِ تأییدشده (هدف: ۳۶).
    pub total_files_verified: usize,
    /// هدفِ قراردادِ سخت.
    pub expected_file_count: usize,
}

// ── منشور + Base ─────────────────────────────────────────────────────────────

/// نمایِ یک جریانِ منشور.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MonopolyCharterStreamView {
    /// شمارهٔ جریان (۱ تا ۴).
    pub stream: u8,
    /// نامِ جریان.
    pub name: String,
    /// نرخِ bps اعلامی.
    pub bps: i64,
    /// مقصدِ حسابِ حاکم.
    pub destination_account: String,
    /// توضیحِ یک‌خطی.
    pub note: String,
}

/// خلاصهٔ ممیزیِ یک قراردادِ Solidity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MonopolyContractAuditView {
    /// نامِ قرارداد.
    pub contract_name: String,
    /// مسیرِ منبع در مخزن.
    pub source_path: String,
    /// نسخهٔ Solidity.
    pub solidity_version: String,
    /// ویژگی‌هایِ معماری — جفتِ کلید/مقدار.
    pub audit_points: Vec<MonopolyAuditPoint>,
}

/// یک بندِ ممیزی.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MonopolyAuditPoint {
    /// کلید (`total_supply`, `post_constructor_mint`, …).
    pub key: String,
    /// مقدارِ اعلامی.
    pub value: String,
}

/// پاسخِ `GET /api/v1/monopoly/charter-and-base`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MonopolyCharterAndBaseResponse {
    /// امضایِ موتورِ خزانه.
    pub signature: String,
    /// شعارِ منشور — «جنگ اول به از صلح آخر».
    pub charter_motto: String,
    /// چهار جریانِ درآمدِ روزِ اول.
    pub streams: Vec<MonopolyCharterStreamView>,
    /// پیکربندیِ Base Mainnet.
    pub base_chain_id: u64,
    /// RPC عمومیِ Base.
    pub base_rpc_url: String,
    /// مرورگرِ زنجیره.
    pub base_block_explorer: String,
    /// قراردادِ USDC بومیِ Base.
    pub base_usdc_contract: String,
    /// خلاصهٔ ممیزیِ `ZeroXKiaToken.sol`.
    pub token_audit: MonopolyContractAuditView,
    /// خلاصهٔ ممیزیِ `KiaBaseEscrowTreasury.sol`.
    pub escrow_audit: MonopolyContractAuditView,
}

// ── شبیه‌سازیِ تقسیم ─────────────────────────────────────────────────────────

/// درخواستِ `POST /api/v1/monopoly/simulate-split`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MonopolySimulateSplitRequest {
    /// سناریو — `marketplace_sale` یا `instant_boost`.
    pub scenario: String,
    /// داراییِ تسویه — `telegram-stars-xtr` | `base-native-eth` |
    /// `base-native-usdc` | `zero-x-kia-base-token` | `internal-micro-kian`.
    pub settlement_asset: String,
    /// مبلغِ ناخالصِ میکرویی (۱۰^-۶ واحدِ نمایش) — برای بوست نادیده گرفته
    /// می‌شود و قیمتِ کانونیکِ لایه اعمال می‌شود.
    pub gross_amount_micros: i64,
    /// شناسهٔ خریدار.
    pub buyer_user_id: String,
    /// شناسهٔ سازنده.
    pub creator_user_id: String,
    /// شناسهٔ معرف — خالی/حذف‌شده یعنی فروشِ مستقیمِ بی‌معرف (قانونِ ۲۰۰۰ bps).
    pub referrer_user_id: Option<String>,
    /// پایشِ وابسته — `1000..=5000`؛ پیش‌فرضِ منشور `1000`.
    pub affiliate_bps: i64,
    /// لایهٔ بوست برای سناریویِ `instant_boost` — `featured-slot-24h` |
    /// `mairon-fast-track-audit` | `viral-swarm-amplifier`.
    pub boost_tier: Option<String>,
    /// آدرسِ گیرندهٔ EIP-681 (اختیاری) — وقتی موجود باشد URI پرداختِ Base
    /// ساخته می‌شود.
    pub recipient_address: Option<String>,
}

/// یک پایِ دفتری روی وایر.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MonopolySplitLegView {
    /// گونهٔ پای (`buyer_debit` | `kia_treasury_credit` | …).
    pub leg_kind: String,
    /// حسابِ دفتری.
    pub account_id: String,
    /// مبلغِ علامت‌دارِ میکرویی.
    pub amount_micros: i64,
    /// نرخِ bps اعلامی.
    pub bps: i64,
    /// یادداشتِ منشور.
    pub note: String,
}

/// پاسخِ شبیه‌سازیِ تقسیم — اثباتِ صفرجمع + ماندگاری + EIP-681.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MonopolySimulateSplitResponse {
    /// شناسهٔ ردیفِ ماندگار (`split:<hash>`).
    pub split_id: String,
    /// سناریوی اجراشده.
    pub scenario: String,
    /// داراییِ تسویه.
    pub settlement_asset: String,
    /// مبلغِ ناخالصِ میکرویی.
    pub gross_amount_micros: i64,
    /// میکروهایِ خزانهٔ حاکم.
    pub kia_treasury_micros: i64,
    /// میکروهایِ معرف (۰ وقتی بی‌معرف).
    pub referrer_micros: i64,
    /// خالصِ سازنده.
    pub creator_net_micros: i64,
    /// سهمِ مؤثرِ پلتفرم بر حسب bps (۲۰۰۰ روی فروشِ مستقیم).
    pub effective_platform_bps: i64,
    /// پاهای دفتریِ دوطرفه.
    pub legs: Vec<MonopolySplitLegView>,
    /// اثباتِ صفرجمع — متنِ قابلِ ممیزی.
    pub zero_sum_proof: String,
    /// آیا ردیف در `sovereign_treasury_splits` ماندگار شد.
    pub persisted: bool,
    /// URI پرداختِ EIP-681 روی Base — فقط برای دارایی‌های on-chain و وقتی
    /// `recipient_address` داده شده باشد.
    pub eip681_payment_uri: Option<String>,
    /// خلاصهٔ وضعیتِ اورکستراتور.
    pub orchestrator_status: String,
}
