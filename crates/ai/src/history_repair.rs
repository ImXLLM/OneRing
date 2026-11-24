//! `history_repair` — ترمیم تاریخچهٔ گفتگو، پایدارسازی آرگومان استریم و حلقهٔ خودترمیم استودیو.
//!
//! پورت خط‌به‌خط از گنجینهٔ CopilotKit/OpenBot (Wave 2):
//! - `openbot/app/src/lib/copilot/repair-history.ts` → [`repair_unanswered_tool_calls`]:
//!   هر فراخوانی ابزارِ بی‌پاسخ (قطع استریم، ریلود صفحه، جابه‌جایی نتیجه) بلافاصله
//!   بعد از نوبت دستیار با نتیجهٔ مصنوعی «UNANSWERED» پر می‌شود تا ارائه‌دهندهٔ
//!   بالادستی هرگز با `MissingToolResultsError` / HTTP ۴۰۰ کل گفتگو را نکشد.
//! - `openbot/settled-args.ts` → [`is_settled_stream_arg`]: هنگام استریم، مقدارهای
//!   نیمه‌راه (رنگ هگز ناقص، URL ناتمام، عددِ با «.» یا «-» پایانی) نگه داشته
//!   می‌شوند تا کامپوننت‌های UI هنگام تایپ نمی‌پرند (zero-jitter).
//! - `a2ui-toolkit/recovery.ts` → [`validate_and_build_self_healing_feedback`]:
//!   اعتبارسنجی پاکت خروجی استودیو و ساخت پرامپت ترمیمِ دقیق برای نوبت بعدی مدل.
//!
//! همهٔ توابع قطعی‌اند (بدون شبکه، بدون شانس) و بر ساختارهای سبک سری‌شونده کار
//! می‌کنند — این کرِیت هیچ‌چیز از دیتابیس نمی‌داند.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// نتیجهٔ مصنوعی برای فراخوانی‌ای که پاسخی نیامد — صدای یکدست «اجرا نشد»
/// (پورت `notRun` در `shared/approval-outcomes.ts`): چه شد + دو راهِ بعد.
pub const UNANSWERED: &str = "Not run: no result came back for this call before the conversation moved on. Run it again if you still need it, or say in your answer that you could not get it.";

/// سقف پیش‌فرض تلاش‌های حلقهٔ خودترمیم استودیو (تلاش اول + دوباره‌ها).
pub const MAX_A2UI_ATTEMPTS: u8 = 3;

/// اعلان یک فراخوانی ابزار داخل پیام دستیار.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolCallDecl {
    /// شناسهٔ فراخوانی — نتیجه با همین شناسه به این اعلان گره می‌خورد.
    pub id: String,
    /// نام ابزار (برای لاگ و صفحات ممیزی).
    pub name: String,
}

/// یک نوبت سبک و سری‌شونده از ترننسکریپت رو‌به‌روی ارائه‌دهنده — نقش‌های
/// AG-UI/OpenBot: `system` / `user` / `assistant` / `tool` (نتیجهٔ ابزار نقشِ
/// مستقل خودش را دارد). هیچ وابستگی به ارائه‌دهنده یا دیتابیس ندارد؛ فیلدها
/// camelCase سری می‌شوند (قرارداد AG-UI: `toolCalls`، `toolCallId`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "role", rename_all = "lowercase", rename_all_fields = "camelCase")]
pub enum ToolCallTurn {
    System { content: String },
    User { content: String },
    Assistant {
        content: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        tool_calls: Vec<ToolCallDecl>,
    },
    /// نتیجهٔ ابزار — با `toolCallId` به اعلانِ بالای خودش گره می‌خورد.
    Tool { tool_call_id: String, content: String },
}

/// خروجی ترمیم: پیام اصلی دست‌نخورده یا نتیجهٔ مصنوعی تزریق‌شده.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepairedMessage {
    /// خود پیام (اصلی یا مصنوعی).
    pub turn: ToolCallTurn,
    /// `true` یعنی این نتیجه توسط ترمیم ساخته شده، نه توسط ابزار واقعی.
    pub synthesized: bool,
}

/// نتیجهٔ ابزار با `toolCallId` آن.
fn tool_call_id_of(turn: &ToolCallTurn) -> Option<&str> {
    match turn {
        ToolCallTurn::Tool { tool_call_id, .. } => Some(tool_call_id),
        _ => None,
    }
}

/// فراخوانی‌های ابزاری که هیچ نتیجه‌ای در کل آرایه ندارند — بدون توجه به ترتیب
/// (پورت `unansweredToolCalls`). شناسه‌های تکراری یک‌بار برمی‌گردند.
///
/// # Errors
/// ندارد.
#[must_use]
pub fn unanswered_tool_calls(turns: &[ToolCallTurn]) -> BTreeSet<String> {
    let answered: BTreeSet<&str> = turns.iter().filter_map(tool_call_id_of).collect();
    let mut unanswered = BTreeSet::new();
    for turn in turns {
        if let ToolCallTurn::Assistant { tool_calls, .. } = turn {
            for call in tool_calls {
                if !answered.contains(call.id.as_str()) {
                    unanswered.insert(call.id.clone());
                }
            }
        }
    }
    unanswered
}

/// همان پیام‌ها، با این تفاوت که هر فراخوانی ابزار نتیجه‌ای دارد که «بعد از
/// آن» می‌آید (پورت `repairUnansweredToolCalls`).
///
/// موقعیت همهٔ ماجراست: ارائه‌دهنده نتیجه را به اعلانِ بالای خودش گره می‌زند، پس
/// نتیجه‌ای که «قبل از» فراخوانی خودش ذخیره شده (بعد از هند‌آف/ AskPerson/MCP
/// همین‌طور ذخیره می‌شود) در نظر ارائه‌دهنده یعنی فراخوانی بی‌پاسخ و مرگ کل
/// گفتگو. نتیجهٔ زودرس «جابه‌جا» می‌شود نه جایگزین — واقعی است و حرف بیشتری
/// برای گفتن دارد؛ فقط فراخوانی‌ای که هیچ‌جا نتیجه ندارد جملهٔ مصنوعیِ
/// [`UNANSWERED`] را می‌گیرد. نتیجه‌ای که پاسخ هیچ فراخوانی‌ای نیست حذف می‌شود.
///
/// پاسخ‌های شناخته‌شدهٔ بیرون از ترننسکریپت (مثلاً تصمیم HITL که روی درخواست
/// مانده) از `known` می‌آیند و بر جملهٔ مصنوعی مقدم‌اند.
///
/// ورودی خالی یا سالم، بدون تغییر برمی‌گردد؛ دو بار اجرا idempotent است.
#[must_use]
pub fn repair_unanswered_tool_calls(
    given: &[ToolCallTurn],
    known: &BTreeMap<String, String>,
) -> Vec<RepairedMessage> {
    // شناسه‌های اعلان‌شده به ترتیب وقوع.
    let mut called: BTreeSet<&str> = BTreeSet::new();
    // پاسخ‌داده از دید ارائه‌دهنده: نتیجه‌ای بعد از فراخوانی خودش.
    let mut answered: BTreeSet<&str> = BTreeSet::new();
    // نتیجهٔ واقعی که قبل از فراخوانی خودش آمد — برای جای‌گذاری درست نگه داشته می‌شود.
    let mut early: BTreeMap<&str, &ToolCallTurn> = BTreeMap::new();
    // اندیس نتیجه‌هایی که همان‌جا که هستند پاسخ هیچ‌چیز نیستند.
    let mut misplaced: BTreeSet<usize> = BTreeSet::new();

    for (index, turn) in given.iter().enumerate() {
        match turn {
            ToolCallTurn::Assistant { tool_calls, .. } => {
                for call in tool_calls {
                    called.insert(call.id.as_str());
                }
            }
            ToolCallTurn::Tool { tool_call_id, .. } => {
                if called.contains(tool_call_id.as_str()) && !answered.contains(tool_call_id.as_str())
                {
                    answered.insert(tool_call_id.as_str());
                } else {
                    // قبل از فراخوانی خودش، یا نتیجهٔ دومِ فراخوانیِ پاسخ‌خورده،
                    // یا نتیجه‌ای که پاسخ هیچ فراخوانی‌ای نیست.
                    early.entry(tool_call_id.as_str()).or_insert(turn);
                    misplaced.insert(index);
                }
            }
            ToolCallTurn::System { .. } | ToolCallTurn::User { .. } => {}
        }
    }

    let mut missing_any = false;
    for turn in given {
        if let ToolCallTurn::Assistant { tool_calls, .. } = turn
            && tool_calls.iter().any(|c| !answered.contains(c.id.as_str()))
        {
            missing_any = true;
            break;
        }
    }
    if !missing_any && misplaced.is_empty() {
        return given
            .iter()
            .cloned()
            .map(|turn| RepairedMessage { turn, synthesized: false })
            .collect();
    }

    let mut repaired: Vec<RepairedMessage> = Vec::with_capacity(given.len() + 4);
    let mut filled: BTreeSet<&str> = answered.clone();
    for (index, turn) in given.iter().enumerate() {
        if misplaced.contains(&index) {
            continue;
        }
        let is_assistant = matches!(turn, ToolCallTurn::Assistant { .. });
        repaired.push(RepairedMessage { turn: turn.clone(), synthesized: false });
        if !is_assistant {
            continue;
        }
        if let ToolCallTurn::Assistant { tool_calls, .. } = turn {
            for call in tool_calls {
                if filled.contains(call.id.as_str()) {
                    continue;
                }
                // بلافاصله بعد از پیام دستیار و قبل از هر پیام بعدی: ارائه‌دهنده
                // نتیجه را باید بعد از اعلان ببیند و ترتیب، ترتیب خود آرایهٔ
                // `tool_calls` باشد.
                let (synthesized_turn, synthesized) = match early.get(call.id.as_str()) {
                    Some(moved) => ((*moved).clone(), false),
                    None => (
                        ToolCallTurn::Tool {
                            tool_call_id: call.id.clone(),
                            content: known
                                .get(&call.id)
                                .cloned()
                                .unwrap_or_else(|| UNANSWERED.to_owned()),
                        },
                        true,
                    ),
                };
                repaired.push(RepairedMessage { turn: synthesized_turn, synthesized });
                // فراخوانی تکراری فقط یک نتیجهٔ ترمیم می‌گیرد.
                filled.insert(call.id.as_str());
            }
        }
    }
    repaired
}

/// آیا مقدارِ در حال استریم «settled» است و اجازهٔ نمایش در UI زنده را دارد؟
///
/// تا وقتی فراخوانی کامل نشده (`is_stream_complete == false`) مقدارهای نیمه‌راه
/// نگه داشته می‌شوند تا کامپوننت‌ها نمی‌پرند:
/// - رنگ هگز ناقص: با `#` شروع می‌شود و طولش ۴/۷/۹ نیست (`#abc` کامل، `#abcde`
///   هنوز در راه است)؛
/// - URL ناتمام: با `http://` یا `https://` شروع می‌شود و به `/` یا `.` ختم
///   می‌شود یا کوتاه‌تر از ۱۲ نویسه است؛
/// - عددِ در حال تایپ که با «.» یا «-» ختم می‌شود («۱۲» در راهِ «۱۲.۵»).
///
/// با `is_stream_complete == true` همه‌چیز بلافاصله settled است.
#[must_use]
pub fn is_settled_stream_arg(key: &str, raw_val: &str, is_stream_complete: bool) -> bool {
    let _ = key; // نام کلید در قرارداد فعلی بی‌اثر است؛ مقدار حرف آخر را می‌زند.
    if is_stream_complete {
        return true;
    }
    // رنگ هگز — فقط طول‌های کامل (۳/۶/۸ رقم + `#`).
    if raw_val.starts_with('#') {
        return matches!(raw_val.chars().count(), 4 | 7 | 9);
    }
    // URL ناتمام.
    if raw_val.starts_with("http://") || raw_val.starts_with("https://") {
        return !(raw_val.ends_with('/')
            || raw_val.ends_with('.')
            || raw_val.chars().count() < 12);
    }
    // عددِ با «.» یا «-» پایانی — هنوز عدد بعدی نمی‌داند چند رقم می‌آید.
    if (raw_val.ends_with('.') || raw_val.ends_with('-')) && is_all_digits(&raw_val[..raw_val.len() - 1])
    {
        return false;
    }
    true
}

/// آیا رشتهٔ خالی‌نشده فقط رقم ASCII است؟
fn is_all_digits(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

/// شمارش تگ بازِ `<tag …>` با مرز واژه (پورت `<(style|script)\b` در TS).
fn count_open_tags(lower_html: &str, tag: &str) -> usize {
    let pattern = format!("<{tag}");
    let mut count = 0;
    let mut cursor = 0;
    while let Some(found) = lower_html[cursor..].find(&pattern) {
        let abs = cursor + found;
        let after = &lower_html[abs + pattern.len()..];
        let boundary_ok = after.is_empty()
            || after
                .chars()
                .next()
                .is_some_and(|c| !(c.is_ascii_alphanumeric() || c == '-'));
        if boundary_ok {
            count += 1;
        }
        cursor = abs + pattern.len();
    }
    count
}

/// عیب‌های پاکت خروجی استودیو — هر عیب با کد و توضیح دقیق برای پرامپت ترمیم.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioDefect {
    /// کد عیب — همان‌که در پرامپت ترمیم مدل می‌بیند (`[missing_doctype]` …).
    pub code: String,
    /// توضیح یک‌جمله‌ای دقیق: چه تگ/بخشی کم یا اضافه است.
    pub message: String,
}

/// اعتبارسنجی کامل پاکت استودیو + ساخت بازخورد خودترمیم (پورت `recovery.ts`).
///
/// خروجی کامل استودیو باید پاکت کامل HTML داشته باشد: `<!DOCTYPE html>` آغازین و
/// `</html>` پایانی، تگ‌های `<script>` و `<style>` بالانس، و بدون نشت حصار
/// markdown (` ``` `). اگر سالم است `Ok(())`؛ وگرنه:
/// - `attempt < max_attempts` → `Err` با پرامپت ساخت‌یافتهٔ ترمیم که دقیقاً
///   می‌گوید کدام تگ/عیب کم است (مدل نوبت بعد دقیقاً همان را می‌سازد)؛
/// - تلاش‌ها تمام شده → `Err` با پاکت `A2UI_RECOVERY_EXHAUSTED` برای رندر
///   فال‌بک امن.
///
/// همین اعتبارسنجی باید هم تصمیم retry ابزار و هم تصمیم suppress رندر را بگیرد
/// تا هیچ‌وقت با هم در تضاد نباشند (اصل «ONE validator» در recovery.ts).
///
/// # Errors
/// هر خروجی نامعتبر یک `Err` ساخت‌یافته است — خودِ این بازخورد ورودیِ نوبت
/// بعدی مدل می‌شود.
pub fn validate_and_build_self_healing_feedback(
    html_or_json: &str,
    attempt: u8,
    max_attempts: u8,
) -> Result<(), String> {
    let defects = studio_defects(html_or_json);
    if defects.is_empty() {
        return Ok(());
    }
    if attempt >= max_attempts {
        return Err(format!(
            "A2UI_RECOVERY_EXHAUSTED: Failed to generate valid Studio output after {max_attempts} attempt(s). \
             Do not retry silently; render the safe fallback envelope instead."
        ));
    }
    let listed = defects
        .iter()
        .map(|d| format!("- [{}] {}", d.code, d.message))
        .collect::<Vec<_>>()
        .join("\n");
    Err(format!(
        "A2UI_REPAIR_FEEDBACK: attempt {attempt}/{max_attempts} produced an invalid Studio document — \
         fix exactly these defects and regenerate the complete document:\n{listed}"
    ))
}

/// فهرست عیب‌های ساختاری خروجی استودیو (بدون قضاوت محتوایی).
#[must_use]
pub fn studio_defects(html_or_json: &str) -> Vec<StudioDefect> {
    let mut defects = Vec::new();
    let lower = html_or_json.to_ascii_lowercase();

    if !lower.contains("<!doctype html") {
        defects.push(StudioDefect {
            code: "missing_doctype".into(),
            message: "the document must start with a complete <!DOCTYPE html> envelope".into(),
        });
    }
    if !lower.contains("</html>") {
        defects.push(StudioDefect {
            code: "missing_closing_html".into(),
            message: "the document must end with a closing </html> tag".into(),
        });
    }
    for (tag, open_code, close_code) in
        [("script", "unbalanced_script", "unclosed_script"), ("style", "unbalanced_style", "unclosed_style")]
    {
        let open = count_open_tags(&lower, tag);
        let close = lower.matches(&format!("</{tag}>")).count();
        if open > close {
            defects.push(StudioDefect {
                code: close_code.into(),
                message: format!("{open} <{tag}> block(s) opened but only {close} closed — every <{tag}> needs its closing </{tag}>"),
            });
        } else if close > open {
            defects.push(StudioDefect {
                code: open_code.into(),
                message: format!("{close} </{tag}> closer(s) but only {open} <{tag}> opener(s) — remove the extra closing tag"),
            });
        }
    }
    if lower.contains("```") {
        defects.push(StudioDefect {
            code: "markdown_fence_leak".into(),
            message: "raw markdown fence (```) leaked into the output — emit the bare HTML document with no code fences".into(),
        });
    }
    defects
}

/// میان‌بر اعتبارسنجی بدون پرامپت ترمیم — سالم یا نه.
#[must_use]
pub fn is_valid_studio_output(html_or_json: &str) -> bool {
    studio_defects(html_or_json).is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assistant(calls: &[(&str, &str)]) -> ToolCallTurn {
        ToolCallTurn::Assistant {
            content: String::new(),
            tool_calls: calls
                .iter()
                .map(|(id, name)| ToolCallDecl { id: (*id).into(), name: (*name).into() })
                .collect(),
        }
    }

    fn tool(id: &str, content: &str) -> ToolCallTurn {
        ToolCallTurn::Tool { tool_call_id: id.into(), content: content.into() }
    }

    /// فراخوانی بی‌پاسخ → نتیجهٔ مصنوعی UNANSWERED بلافاصله بعد از نوبت دستیار.
    #[test]
    fn orphaned_tool_call_gets_synthetic_result_right_after_assistant() {
        let turns = vec![
            ToolCallTurn::User { content: "قیمت را بگو".into() },
            assistant(&[("c1", "market.price")]),
            ToolCallTurn::User { content: "خوب است".into() },
        ];
        let repaired = repair_unanswered_tool_calls(&turns, &BTreeMap::new());
        assert_eq!(repaired.len(), 4);
        assert!(!repaired[0].synthesized && !repaired[1].synthesized);
        assert!(repaired[2].synthesized, "synthetic result must sit at index 2");
        assert_eq!(
            repaired[2].turn,
            ToolCallTurn::Tool { tool_call_id: "c1".into(), content: UNANSWERED.into() }
        );
        assert_eq!(repaired[3].turn, turns[2], "later user turn stays after the repair");
    }

    /// نتیجه‌ای که قبل از فراخوانی ذخیره شده جابه‌جا می‌شود، نه جایگزین؛ نتیجهٔ
    /// بی‌صاحب حذف می‌شود؛ دو بار اجرا idempotent است.
    #[test]
    fn early_result_is_moved_orphans_dropped_and_repair_is_idempotent() {
        let turns = vec![
            tool("c9", "نتیجهٔ بی‌صاحب"),
            tool("c1", "Handed to Knowledge — پاسخ واقعی"),
            assistant(&[("c1", "kb.search"), ("c2", "market.price")]),
        ];
        let repaired = repair_unanswered_tool_calls(&turns, &BTreeMap::new());
        // c9 حذف؛ c1 جابه‌جا بعد از دستیار؛ c2 مصنوعی.
        assert_eq!(repaired.len(), 3, "orphan dropped; assistant + moved + synthetic");
        assert!(matches!(repaired[0].turn, ToolCallTurn::Assistant { .. }));
        assert!(!repaired[1].synthesized);
        assert_eq!(
            repaired[1].turn,
            tool("c1", "Handed to Knowledge — پاسخ واقعی"),
            "the real early result is moved, not replaced"
        );
        assert!(repaired[2].synthesized);
        assert_eq!(repaired[2].turn, tool("c2", UNANSWERED));

        // سالم از دید ارائه‌دهنده: هر اعلان بعد از خودش نتیجه دارد.
        let plain: Vec<ToolCallTurn> = repaired.iter().map(|m| m.turn.clone()).collect();
        let second = repair_unanswered_tool_calls(&plain, &BTreeMap::new());
        assert_eq!(
            plain,
            second.iter().map(|m| m.turn.clone()).collect::<Vec<_>>(),
            "second repair pass must be a no-op"
        );
        assert!(second.iter().all(|m| !m.synthesized));
    }

    /// پاسخ شناخته‌شدهٔ بیرونی بر جملهٔ مصنوعی مقدم است؛ نتیجهٔ دومِ تکراری حذف می‌شود.
    #[test]
    fn known_answer_wins_and_duplicate_results_are_dropped() {
        let turns = vec![
            assistant(&[("c1", "escrow.create")]),
            tool("c1", "نتیجهٔ اول"),
            tool("c1", "نتیجهٔ دوم تکراری"),
        ];
        let mut known = BTreeMap::new();
        known.insert("c1".to_owned(), "تأیید شد: پاکت‌ساز بسته شد".to_owned());
        let repaired = repair_unanswered_tool_calls(&turns, &known);
        // هر دو نتیجهٔ موجود بعد از فراخوانی‌اند: اولی answered، دومی misplaced → حذف.
        assert_eq!(repaired.len(), 2);
        assert!(!repaired[1].synthesized);
        assert_eq!(repaired[1].turn, tool("c1", "نتیجهٔ اول"));

        // و اگر هیچ نتیجه‌ای نبود، پاسخ known جای UNANSWERED می‌نشیند.
        let orphan = vec![assistant(&[("c1", "escrow.create")])];
        let repaired = repair_unanswered_tool_calls(&orphan, &known);
        assert_eq!(repaired.len(), 2);
        assert!(repaired[1].synthesized);
        assert_eq!(repaired[1].turn, tool("c1", "تأیید شد: پاکت‌ساز بسته شد"));
    }

    /// فارسی‌نویسی در ترننسکریپت نباید مسیر ترمیم را بشکند (ورودی واقعی کاربر).
    #[test]
    fn unanswered_scan_ignores_user_and_system_turns() {
        let turns = vec![
            ToolCallTurn::System { content: "تو مغز کیانی".into() },
            ToolCallTurn::User { content: "tool متن با کلمهٔ tool".into() },
            assistant(&[("c1", "kb.search"), ("c2", "kb.search")]),
        ];
        let unanswered = unanswered_tool_calls(&turns);
        assert_eq!(unanswered.iter().cloned().collect::<Vec<_>>(), vec!["c1", "c2"]);
        // فراخوانی تکراری فقط یک نتیجهٔ ترمیم می‌گیرد.
        let repaired = repair_unanswered_tool_calls(&turns, &BTreeMap::new());
        assert_eq!(repaired.len(), 5);
        assert!(repaired[3].synthesized && repaired[4].synthesized);
    }

    /// رنگ هگز: کامل‌ها (۴/۷/۹) settled، نیمه‌راه unsettled.
    #[test]
    fn hex_colors_settle_only_on_complete_lengths() {
        for (val, expect) in [
            ("#abc", true),      // ۴ نویسه — کامل
            ("#abcdef", true),   // ۷ نویسه — کامل
            ("#aabbccdd", true), // ۹ نویسه — کامل با آلفا
            ("#ab", false),      // نیمه‌راه
            ("#abcde", false),   // نیمه‌راه
            ("#", false),
        ] {
            assert_eq!(is_settled_stream_arg("color", val, false), expect, "val={val}");
        }
    }

    /// URL ناتمام (به «/» یا «.» ختم می‌شود یا کوتاه‌تر از ۱۲) نگه داشته می‌شود.
    #[test]
    fn incomplete_urls_are_held_back() {
        assert!(!is_settled_stream_arg("url", "http://", false));
        assert!(!is_settled_stream_arg("href", "https://a.", false));
        assert!(!is_settled_stream_arg("src", "https://a.io/", false));
        assert!(!is_settled_stream_arg("url", "http://a.io", false), "11 chars is incomplete");
        assert!(is_settled_stream_arg("url", "https://a.io", false), "12 chars is settled");
        assert!(is_settled_stream_arg("url", "https://kian.dev/market", false));
        assert!(is_settled_stream_arg("url", "https://a.io/", true), "complete stream always settles");
    }

    /// عددِ با «.» یا «-» پایانی unsettled است تا نوار UI نپرد.
    #[test]
    fn trailing_numeric_strings_are_held_back() {
        assert!(!is_settled_stream_arg("amount", "12.", false), "12 is on its way to 12.5");
        assert!(!is_settled_stream_arg("amount", "125-", false));
        assert!(is_settled_stream_arg("amount", "12.5", false));
        assert!(is_settled_stream_arg("amount", "125", false));
        assert!(is_settled_stream_arg("label", "سلام.", false), "non-numeric tail is not a number");
        assert!(is_settled_stream_arg("amount", "-", false), "empty digits before '-'");
    }

    /// سند کامل سالم است؛ عیب‌ها دقیقاً نام‌گذاری می‌شوند.
    #[test]
    fn self_healing_validates_complete_envelope_and_names_exact_defects() {
        let good = "<!DOCTYPE html><html><head><style>body{color:#0ff}</style></head><body><script>const x=1;</script>سلام</body></html>";
        assert_eq!(validate_and_build_self_healing_feedback(good, 0, MAX_A2UI_ATTEMPTS), Ok(()));
        assert!(is_valid_studio_output(good));

        // بدون doctype و بدون </html> — هر دو عیب با کد دقیق گزارش می‌شود.
        let feedback = validate_and_build_self_healing_feedback("<div>سلام</div>", 0, MAX_A2UI_ATTEMPTS)
            .expect_err("incomplete envelope must fail");
        assert!(feedback.contains("A2UI_REPAIR_FEEDBACK"), "feedback: {feedback}");
        assert!(feedback.contains("[missing_doctype]"));
        assert!(feedback.contains("[missing_closing_html]"));
        assert!(feedback.contains("attempt 0/3"));
    }

    /// اسکریپت/استایلِ باز مانده و حصار markdown عیبِ نام‌دار می‌گیرند.
    #[test]
    fn self_healing_flags_unclosed_script_and_markdown_fence() {
        let feedback = validate_and_build_self_healing_feedback(
            "<!DOCTYPE html><html><body><script>let x = 1;\n```html\n",
            1,
            MAX_A2UI_ATTEMPTS,
        )
        .expect_err("unclosed script + fence must fail");
        assert!(feedback.contains("[unclosed_script]"), "feedback: {feedback}");
        assert!(feedback.contains("[markdown_fence_leak]"));

        // تگ کست‌شدهٔ اضافی هم گزارش می‌شود (نشت پاکت).
        let feedback = validate_and_build_self_healing_feedback(
            "<!DOCTYPE html><html></html></style>",
            0,
            MAX_A2UI_ATTEMPTS,
        )
        .expect_err("extra closer must fail");
        assert!(feedback.contains("[unbalanced_style]"), "feedback: {feedback}");
    }

    /// قبل از سقف، پرامپت ترمیم؛ بعد از سقف، پاکت EXHAUSTED برای فال‌بک امن.
    #[test]
    fn self_healing_switches_to_exhausted_envelope_after_max_attempts() {
        let bad = "<div>ناقص</div>";
        let repair = validate_and_build_self_healing_feedback(bad, 2, MAX_A2UI_ATTEMPTS)
            .expect_err("attempt 2 of 3 still repairs");
        assert!(repair.starts_with("A2UI_REPAIR_FEEDBACK"));
        let exhausted = validate_and_build_self_healing_feedback(bad, 3, MAX_A2UI_ATTEMPTS)
            .expect_err("exhausted is still an Err");
        assert!(exhausted.starts_with("A2UI_RECOVERY_EXHAUSTED"), "got: {exhausted}");
        assert!(exhausted.contains("after 3 attempt(s)"));
    }

    /// ساختارهای سبک سری‌شونده‌اند و نقش‌ها با تگ `role` سری می‌شوند (قرارداد AG-UI).
    #[test]
    fn turns_serialize_with_role_tag_and_repaired_flags() {
        let turns = vec![assistant(&[("c1", "kb.search")])];
        let json = serde_json::to_string(&turns).expect("serialize");
        assert!(json.contains("\"role\":\"assistant\""), "json: {json}");
        assert!(json.contains("\"toolCalls\""), "camelCase toolCalls: {json}");

        let repaired = repair_unanswered_tool_calls(&turns, &BTreeMap::new());
        let json = serde_json::to_string(&repaired).expect("serialize repaired");
        assert!(json.contains("\"synthesized\":true"), "json: {json}");
        assert!(json.contains("\"role\":\"tool\""));
    }
}
