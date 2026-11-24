//! `compressor` — فشرده‌ساز دو مرحله‌ای کانتکست (پورت هرمس).
//!
//! پورت از گنجینهٔ `NousResearch` (Wave 2 - Step 2):
//! - `agent/context_compressor_prune.py` → فاز ۱ هرس قطعی خروجی ابزار
//!   ([`prune_old_tool_outputs`]): خروجی‌های حجیمِ نوبت‌های قدیمی به استابِ
//!   یک‌خطی تبدیل می‌شوند (`[tool:{name}] ran -> exit {code}, {lines} lines
//!   ({bytes}B) | last: {last_non_empty_line}`) — اما **هرگز** نوبتِ فعال/آخر
//!   (`keep_recent_turns >= 1`، پورت `_keep_running_turn`) و هرگز خروجیِ حاوی
//!   راز (گارد راز کیان: [`kian_ai::redaction::redact`]) دست نمی‌خورد.
//!   گیتِ دیسک سرد: [`should_trigger_cold_prune`] با کف `16_000` توکن —
//!   شکستن کشِ ارائه‌دهنده برای پس‌انداز ناچیز هرگز توجیه ندارد.
//! - `agent/micro_compaction.py` → فاز ۲ ([`partition_for_micro_compaction`]):
//!   مرزهای فolding خلاصهٔ ریز (micro-compaction) — فقط بلوک‌های تبادلِ
//!   کاملِ `(assistant + tool*)` از نوبت‌های قدیمی شایستگی جمع‌بندی دارند و
//!   **همهٔ پیام‌های `user` صددرصد دست‌نخورده در جای خودشان می‌مانند**.
//!
//! همهٔ توابع قطعی‌اند؛ ورودی ساختار سبک [`CompressorMessage`] است نه مدلِ
//! ارائه‌دهنده — این کرِیت هیچ‌چیز از دیتابیس نمی‌داند.
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// کف توکنِ پس‌انداز برای هرس روی کشِ سردِ ارائه‌دهنده
/// (`_COLD_CACHE_MIN_RECLAIM_TOKENS = 16_000` در منبع).
pub const COLD_CACHE_MIN_RECLAIM_TOKENS: usize = 16_000;

/// کف کاراکتر برای اینکه خروجی ابزار «حجیم» شمرده شود و هرس‌شدنی باشد.
pub const MIN_PRUNE_CHARS: usize = 400;

/// سقف طولِ خطِ «آخرین خروجی» داخل استاب.
const LAST_LINE_ROOM: usize = 120;

/// یک پیام سبک و سری‌شونده از تاریخچه — نقش‌های چت‌کامپلیشن:
/// `system` / `user` / `assistant` / `tool`. خروجی ابزار در `content` نقش
/// `tool` می‌آید و `tool_name` نام ابزارِ مولدش را نگه می‌دارد.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompressorMessage {
    pub role: String,
    pub content: String,
    /// نام ابزار برای نقش `tool` (برای استاب و تله‌متری)؛ رشتهٔ خالی = نداشته.
    #[serde(default)]
    pub tool_name: String,
}

impl CompressorMessage {
    #[must_use]
    pub fn new(role: &str, content: &str) -> Self {
        Self { role: role.to_owned(), content: content.to_owned(), tool_name: String::new() }
    }

    #[must_use]
    pub fn tool(role: &str, tool_name: &str, content: impl Into<String>) -> Self {
        Self { role: role.to_owned(), content: content.into(), tool_name: tool_name.to_owned() }
    }

    fn is_tool(&self) -> bool {
        self.role == "tool"
    }
}

/// نتیجهٔ هرس قطعی: تاریخچهٔ تازه + چند پیام کوتاه شده + چند بایت پس‌انداز.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PruneOutcome {
    pub messages: Vec<CompressorMessage>,
    pub pruned_count: usize,
    pub saved_bytes: usize,
}

/// گیتِ هرس روی کشِ سرد: فقط وقتی پس‌اندازِ تخمینی از کفِ ۱۶٬۰۰۰ توکن بگذرد
/// ارزشِ شکستن کشِ از پیش منقضی را دارد (پورت `_COLD_CACHE_MIN_RECLAIM_TOKENS`).
#[must_use]
pub fn should_trigger_cold_prune(estimated_saved_tokens: usize) -> bool {
    estimated_saved_tokens >= COLD_CACHE_MIN_RECLAIM_TOKENS
}

/// برآورد درشتِ توکن — حدود ۴ کاراکتر در توکن (خنثی زبانی؛ برای فارسی/RTL
/// همین درقتِ سخت‌گیرانه، نه کمتر، انتخاب می‌شود).
#[must_use]
pub fn estimate_tokens(text: &str) -> usize {
    text.chars().count().div_ceil(4)
}

/// خروجی ابزار حاوی راز است؟ — گارد راز کیان قطعی تصمیم می‌گیرد: یافتهٔ
/// اسکنرِ [`crate::redact`] یا پیشوندهای صریحِ کلیدِ API (`sk-`، `ghp_`،
/// `xox`، `Bearer `، JWT `eyJ…`) یعنی «این خروجی هرگز دست نمی‌خورد،
/// بایت‌به‌بایت می‌ماند» — دفاع در عمق روی اسکنرِ پایه.
fn contains_secret(text: &str) -> bool {
    if !crate::redact(text).findings.is_empty() {
        return true;
    }
    for marker in ["sk-", "ghp_", "gho_", "xoxb-", "Bearer ", "AKIA"] {
        if text.contains(marker) {
            return true;
        }
    }
    // JWT سه‌تکه‌ای: eyJ…eyJ…eyJ (بدون نیازمندی به regex)
    text.contains("eyJ")
}

/// کد خروجی را از پاکت JSON نتیجه می‌خواند — پورت regex
/// `"exit_code"\s*:\s*(-?\d+)` با تحملِ کوتیشنِ escape‌شده در پاکت‌های
/// تو‌در‌تو (`\"exit_code\":`). نبودِ پاکت = `0` (موفق).
fn extract_exit_code(content: &str) -> String {
    let mut search_from = 0usize;
    while let Some(rel) = content[search_from..].find("exit_code") {
        let i = search_from + rel;
        let after = content[i + "exit_code".len()..].trim_start();
        // کوتیشنِ بسته‌شدهٔ کلید (با یا بدون بک‌اسلش) و فاصله‌ها خورده می‌شوند
        let after = after.trim_start_matches(['\\', '"', '\'']).trim_start();
        let Some(after) = after.strip_prefix(':') else {
            search_from = i + "exit_code".len();
            continue;
        };
        let after = after.trim_start();
        let digits: String =
            after.chars().take_while(|c| c.is_ascii_digit() || *c == '-').collect();
        if digits.is_empty() || digits == "-" {
            search_from = i + "exit_code".len();
            continue;
        }
        return digits;
    }
    "0".to_owned()
}

/// استاب یک‌خطی برای خروجی هرس‌شده — پورت `terminal_stub` با قالبِ عمومیِ
/// گنجینه: `[tool:{name}] ran -> exit {code}, {lines} lines ({bytes}B) | last: …`
/// استاب همیشه زیر `MIN_PRUNE_CHARS` می‌ماند تا پاسِ بعدی آن را دوباره
/// «نتیجه برای خلاصه‌کردن» نگیرد.
fn tool_stub(tool_name: &str, content: &str) -> String {
    let bytes = content.len();
    let lines = content.lines().count();
    let exit_code = extract_exit_code(content);
    let last = content
        .lines()
        .rev()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("")
        .chars()
        .take(LAST_LINE_ROOM)
        .collect::<String>();
    let mut stub = format!(
        "[tool:{tool_name}] ran -> exit {exit_code}, {lines} lines ({bytes}B) | last: {last}"
    );
    if stub.len() > MIN_PRUNE_CHARS {
        stub = stub.chars().take(MIN_PRUNE_CHARS).collect();
    }
    stub
}

/// مرز حفاظت: آخرین `keep_recent_turns` نوبتِ کامل (`user … tool*` انتهایی)
/// هرگز هرس نمی‌شود — پورت `_keep_running_turn` با سخت‌گیری بیشتر: `keep_recent_turns`
/// کمتر از ۱ معنا ندارد و fail-closed به ۱ می‌رود.
fn protected_boundary(messages: &[CompressorMessage], keep_recent_turns: usize) -> usize {
    let keep = keep_recent_turns.max(1);
    // از انتها به عقب، شروعِ nاوبتِ نگه‌داشته‌شده را پیدا می‌کنیم: مرزِ شروعِ
    // هر نوبت، ردیفِ `user` آن است.
    let mut turns_seen = 0usize;
    let mut boundary = 0usize;
    for (i, msg) in messages.iter().enumerate().rev() {
        if msg.role == "user" {
            turns_seen += 1;
            if turns_seen == keep {
                boundary = i;
                break;
            }
        }
    }
    if turns_seen < keep {
        // کل تاریخچه کوتاه‌تر از حفاظ است — هیچ چیز هرس‌شدنی نیست
        boundary = 0;
    }
    boundary
}

/// فاز ۱ — هرس قطعی و بدون LLMِ خروجی ابزارهای قدیمی.
///
/// قواعد (پورت `prune_tool_results_only`):
/// ۱. مرز حفاظت از «شمارش پیام» نیست، از «شروع نوبتِ user» است تا هرگز وسطِ
///    نتایجِ موازیِ یک گام دستیار بریده نشود.
/// ۲. فقط خروجی‌های `tool` حجیم (≥ [`MIN_PRUNE_CHARS`]) از نوبت‌های **قدیمی**
///    استاب می‌شوند؛ خروجیِ حاوی راز (گارد [`kian_ai::redact`]) همیشه دست‌نخورده.
/// ۳. اگر هیچ چیز کوتاه نشد، ورودی عیناً برگردانده می‌شود (قرارداد no-op).
#[must_use]
pub fn prune_old_tool_outputs(
    messages: &[CompressorMessage],
    keep_recent_turns: usize,
) -> PruneOutcome {
    let boundary = protected_boundary(messages, keep_recent_turns);
    let mut out: Vec<CompressorMessage> = Vec::with_capacity(messages.len());
    let mut pruned_count = 0usize;
    let mut saved_bytes = 0usize;
    for (i, msg) in messages.iter().enumerate() {
        let eligible = i < boundary
            && msg.is_tool()
            && msg.content.chars().count() >= MIN_PRUNE_CHARS
            && !contains_secret(&msg.content);
        if eligible {
            let stub = tool_stub(&msg.tool_name, &msg.content);
            saved_bytes += msg.content.len().saturating_sub(stub.len());
            pruned_count += 1;
            out.push(CompressorMessage::tool("tool", &msg.tool_name, &stub));
        } else {
            out.push(msg.clone());
        }
    }
    if pruned_count == 0 {
        // قرارداد no-op: ورودی عیناً — تماس‌گیرنده روی «هیچ تغییری» گیت می‌زند
        return PruneOutcome { messages: messages.to_vec(), pruned_count: 0, saved_bytes: 0 };
    }
    PruneOutcome { messages: out, pruned_count, saved_bytes }
}

/// یک بلوک تبادلِ واجد شرطِ جمع‌بندی — از یک `assistant` (بدون فراخوانی ابزار
/// معلق) تا پیش از `user` بعدی؛ فقط پیام‌های `assistant` و `tool` دارد.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExchangeBlock {
    /// بازهٔ `[start, end)` روی تاریخچهٔ ورودی.
    pub start: usize,
    pub end: usize,
    pub tokens: usize,
}

/// خروجی تفکیک فاز ۲ — کدام بازه‌ها شایستگی جمع‌بندی دارند و چه چیزی محفوظ است.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MicroCompactionPlan {
    /// بازه‌های تبادلِ کاملِ قدیمی — هرگز شامل `user` نیستند.
    pub foldable: Vec<ExchangeBlock>,
    /// شاخص‌های محفوظ: همهٔ `user`ها + سرِ سیستم + نوبتِ فعال.
    pub keep: Vec<usize>,
    /// شاخص‌های جمع‌بندی‌شدنیِ کل (تجمیعِ foldable) — برای تله‌متری.
    pub foldable_indices: Vec<usize>,
}

/// فاز ۲ — تفکیک تاریخچه برای خلاصهٔ ریزِ تبادل‌محور
/// (پورت `micro_compaction.py`).
///
/// قواعد سخت:
/// ۱. **همهٔ پیام‌های `user` صددرصد دست‌نخورده در جای خود می‌مانند** — خلاصه
///    هرگز کلمه‌ای از درخواست واقعی انسان را نمی‌بلعد.
/// ۲. فقط بلوک‌های تبادلِ **کاملِ** `(assistant + tool*)` از نوبت‌های قدیمی
///    (قبل از مرزِ حفاظتِ نوبتِ فعال) شایستگی دارند: بلوک با `assistant` باز
///    می‌شود، با `user` بعدی (یا انتها) بسته می‌شود؛ `system` سرِ تاریخچه
///    هرگز جمع نمی‌شود.
/// ۳. بلوکِ ناقص (assistant بدون بسته‌شدن، یا ردیفِ tool یتیم) fail-closed
///    محفوظ می‌ماند نه جمع‌بندی.
#[must_use]
pub fn partition_for_micro_compaction(
    messages: &[CompressorMessage],
    keep_recent_turns: usize,
) -> MicroCompactionPlan {
    let boundary = protected_boundary(messages, keep_recent_turns);
    let mut plan = MicroCompactionPlan::default();
    // حلقهٔ دستی: بعد از بلوکِ جمع‌شدنی به انتهایش می‌پریم تا ردیف‌های tool
    // داخل بلوک، به‌اشتباه دوباره «محفوظ» نشمارند.
    let mut i = 0usize;
    while i < messages.len() {
        if messages[i].role == "assistant" {
            // بازهٔ بلوک را ببند: از اینجا تا اولین user/system بعدی
            let mut end = i + 1;
            while end < boundary
                && matches!(messages[end].role.as_str(), "assistant" | "tool")
            {
                end += 1;
            }
            let complete =
                end < messages.len() && messages[end].role == "user" && end <= boundary;
            if complete {
                let tokens: usize = messages[i..end]
                    .iter()
                    .map(|m| estimate_tokens(&m.content))
                    .sum();
                plan.foldable.push(ExchangeBlock { start: i, end, tokens });
                plan.foldable_indices.extend(i..end);
                i = end; // userِ بستن‌دهنده در دور بعد، محفوظ می‌شود
            } else {
                plan.keep.push(i);
                i += 1;
            }
        } else {
            // system / user / ردیفِ tool یتیم (assistant قبلی جمع‌نشدنی) و هر
            // نقشِ ناشناخته — همه محفوظ fail-closed؛ به‌ویژه همهٔ userها صددرصد
            // دست‌نخورده در جای خود می‌مانند
            plan.keep.push(i);
            i += 1;
        }
    }
    plan
}

#[cfg(test)]
mod tests {
    use super::*;

    fn history() -> Vec<CompressorMessage> {
        vec![
            CompressorMessage::new("system", "تو هرمس کیان هستی."),
            CompressorMessage::new("user", "ساختار پروژه را بررسی کن"),
            CompressorMessage::new("assistant", "دارم ساختار را می‌خوانم…"),
            CompressorMessage::tool("tool", "shell", "exit 0\n".repeat(200)), // حجیم
            CompressorMessage::tool("tool", "fs", "exit 0\n".repeat(150)),       // حجیم
            CompressorMessage::new("user", "حالا تست‌ها را بزن"),
            CompressorMessage::new("assistant", "در حال اجرای تست‌ها…"),
            CompressorMessage::tool("tool", "shell", "test ok\n".repeat(120)), // نوبتِ فعال
        ]
    }

    /// نوبتِ فعال/آخر هرگز هرس نمی‌شود — پورت `_keep_running_turn`.
    #[test]
    fn active_turn_is_never_pruned() {
        let out = prune_old_tool_outputs(&history(), 1);
        let active = &out.messages[out.messages.len() - 1];
        assert_eq!(active.content, "test ok\n".repeat(120), "خروجی نوبتِ فعال دست‌نخورده");
        assert!(out.pruned_count >= 2, "خروجی‌های قدیمی باید هرس شوند");
    }

    /// استاب یک‌خطی با قالبِ گنجینه ساخته می‌شود و بایت پس‌انداز می‌کند.
    #[test]
    fn stub_format_carries_exit_lines_bytes_last() {
        let content = "step a\nstep b\n\"exit_code\": 3\nlast non-empty line";
        let stub = tool_stub("shell", content);
        assert!(stub.starts_with("[tool:shell] ran -> exit 3, "), "استاب: {stub}");
        assert!(stub.contains("4 lines"), "تعداد خط: {stub}");
        assert!(stub.contains(&format!("({}B)", content.len())), "بایت: {stub}");
        assert!(stub.contains("last: last non-empty line"), "آخرین خط: {stub}");
        assert!(stub.chars().count() < MIN_PRUNE_CHARS, "استاب کوتاه می‌ماند");
        // پاکت escape-شدهٔ JSON هم کد خروجی را می‌خواند
        let escaped = r#"{"output": "{\"exit_code\": 42, \"out\": 1}"}"#;
        assert_eq!(extract_exit_code(escaped), "42");
        // نبودِ پاکت = موفق (0)
        assert_eq!(extract_exit_code("plain output"), "0");
        assert_eq!(extract_exit_code("\"exit_code\": -1"), "-1");
    }

    /// خروجیِ حاوی راز هرگز هرس نمی‌شود — گارد راز کیان بر فشرده‌سازی مقدم است.
    #[test]
    fn secret_bearing_output_is_never_pruned() {
        let secret_out = format!("{}\n", "sk-or-v1-DUMMY-TEST-FIXTURE-not-real-aaaaaaaa".repeat(30));
        let msgs = vec![
            CompressorMessage::new("user", "قدم اول"),
            CompressorMessage::new("assistant", "اجرا شد"),
            CompressorMessage::tool("tool", "shell", secret_out.clone()),
            CompressorMessage::new("user", "قدم دوم"),
        ];
        let out = prune_old_tool_outputs(&msgs, 1);
        assert_eq!(out.pruned_count, 0, "راز = دست‌نخورده");
        assert_eq!(out.messages[2].content, secret_out, "بایت‌به‌بایت");
    }

    /// خروجیِ کوتاه هرس نمی‌شود (کف حجیمی) — و no-op عیناً ورودی برمی‌گرداند.
    #[test]
    fn small_outputs_and_noop_return_input_unchanged() {
        let msgs = vec![
            CompressorMessage::new("user", "سلام"),
            CompressorMessage::new("assistant", "درود"),
            CompressorMessage::tool("tool", "tiny", "ok"),
            CompressorMessage::new("user", "ادامه"),
        ];
        let out = prune_old_tool_outputs(&msgs, 1);
        assert_eq!(out.messages, msgs, "no-op باید عیناً ورودی باشد");
        assert_eq!(out.pruned_count, 0);
        assert_eq!(out.saved_bytes, 0);
    }

    /// گیت کش سرد: کف ۱۶٬۰۰۰ توکن سخت است.
    #[test]
    fn cold_prune_gate_enforces_16k_floor() {
        assert!(!should_trigger_cold_prune(15_999));
        assert!(should_trigger_cold_prune(16_000));
        assert!(should_trigger_cold_prune(50_000));
        assert_eq!(COLD_CACHE_MIN_RECLAIM_TOKENS, 16_000);
    }

    /// برآورد توکن درشت — ۴ کاراکتر در توکن، صفر = صفر.
    #[test]
    fn token_estimation_is_rough_and_positive() {
        assert_eq!(estimate_tokens(""), 0);
        assert_eq!(estimate_tokens("abcdefgh"), 2);
        assert_eq!(estimate_tokens("abc"), 1);
    }

    /// تفکیک فاز ۲: فقط تبادل کامل قدیمی جمع‌شدنی است و همهٔ userها دست‌نخورده.
    #[test]
    fn micro_compaction_partitions_exchanges_and_keeps_users() {
        let msgs = history();
        let plan = partition_for_micro_compaction(&msgs, 1);
        // همهٔ userها + system محفوظ‌اند
        for (i, msg) in msgs.iter().enumerate() {
            if matches!(msg.role.as_str(), "user" | "system") {
                assert!(plan.keep.contains(&i), "نقش {} در شاخص {i} باید محفوظ باشد", msg.role);
            }
        }
        // هیچ بازهٔ جمع‌شدنی شامل user نیست و هیچ ردیفی دوبار شمرده نمی‌شود
        for block in &plan.foldable {
            for i in block.start..block.end {
                assert_ne!(msgs[i].role, "user", "user هرگز در بلوک جمع‌بندی نیست");
                assert!(!plan.keep.contains(&i), "ردیفِ جمع‌شدنی نباید در keep هم باشد: {i}");
            }
        }
        assert_eq!(
            plan.foldable_indices.len(),
            plan.foldable.iter().map(|b| b.end - b.start).sum::<usize>(),
            "شاخص‌های تجمیعی با جمع بلوک‌ها برابر است"
        );
        // تبادل قدیمی (assistant + 2 tool) جمع‌شدنی است
        assert_eq!(plan.foldable.len(), 1, "فقط بلوک قدیمی کامل");
        assert_eq!(plan.foldable[0].start, 2);
        assert_eq!(plan.foldable[0].end, 5);
        assert!(plan.foldable[0].tokens > 0);
        // ردیف‌های نوبتِ فعال (6..8) محفوظ fail-closed
        for i in 6..8 {
            assert!(plan.keep.contains(&i), "نوبتِ فعال شاخص {i} محفوظ");
        }
    }

    /// بلوک ناقص (بدون user بعدی = نوبت در جریان) جمع‌شدنی نیست.
    #[test]
    fn incomplete_exchange_is_kept_not_folded() {
        let msgs = vec![
            CompressorMessage::new("system", "سر"),
            CompressorMessage::new("user", "سوال قدیمی"),
            CompressorMessage::new("assistant", "دارم کار می‌کنم…"),
            CompressorMessage::tool("tool", "shell", "خروجی"),
        ];
        let plan = partition_for_micro_compaction(&msgs, 1);
        assert!(plan.foldable.is_empty(), "بلوکِ بدون بستنِ user جمع نمی‌شود");
        assert!(plan.keep.contains(&2) && plan.keep.contains(&3));
    }

    /// تاریخچهٔ کوتاه‌تر از حفاظ — هیچ چیز جمع‌شدنی نیست (fail-closed).
    #[test]
    fn tiny_history_is_fully_protected() {
        let msgs = vec![
            CompressorMessage::new("user", "تنها نوبت"),
            CompressorMessage::new("assistant", "پاسخ"),
        ];
        let plan = partition_for_micro_compaction(&msgs, 1);
        assert!(plan.foldable.is_empty());
        let out = prune_old_tool_outputs(&msgs, 1);
        assert_eq!(out.pruned_count, 0);
    }

    /// سری‌سازی گردش‌به‌گردان CompressorMessage — پایدار برای تاریخچهٔ ذخیره‌شده.
    #[test]
    fn compressor_message_roundtrips_serde() {
        let msg = CompressorMessage::tool("tool", "shell", "{\"exit_code\": 0}");
        let json = serde_json::to_string(&msg).unwrap();
        let back: CompressorMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(back, msg);
        let without_name: CompressorMessage =
            serde_json::from_str("{\"role\":\"user\",\"content\":\"سلام\"}").unwrap();
        assert_eq!(without_name.tool_name, "");
    }
}
