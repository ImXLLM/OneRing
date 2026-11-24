//! `tool_parser` — انباشتگر استریم فراخوانی ابزار و پارسر XML+JSON هرمس.
//!
//! پورت خط‌به‌خط از گنجینهٔ `NousResearch` (Wave 2 - Step 2):
//! - `agent/chat_completion_helpers_tool_calls.py` → [`ToolCallStreamAccumulator`]:
//!   انباشت دلتاهای استریمِ `tool_calls` با دو اصلاح سرنوشت‌ساز: (۱) نام تابع با
//!   **انتساب** (`=`) جایگزین می‌شود نه الحاق (`+=`) — بعضی گیت‌وی‌ها (`MiniMax` روی
//!   NVIDIA NIM، مدل‌های رایگان `OpenRouter`) نام کامل را در هر دلتا می‌فرستند و
//!   الحاق کورکورانه «`read_fileread_file`» می‌سازد؛ (۲) عمق براکت JSON ردیابی
//!   می‌شود تا اگر ارائه‌دهنده همان `index` را بعد از بستن شیء JSON دوباره مصرف
//!   کرد، اسلاتِ تازه و تمیزی باز شود نه ادامهٔ فراخوانی قبلی.
//! - `Hermes-Function-Calling/utils.py` → [`extract_xml_tool_calls`]: استخراج
//!   بلوک‌های `{"name": ..., "arguments": {...}}` از `<tool_call>` متنِ دستیار،
//!   با فال‌بکِ آسان‌گیر (ویرگول انتهایی، کلید‌های یک‌نقل‌قولی) — مدل‌های وزن‌باز
//!   `:free` بدون function-calling بومیِ ارائه‌دهنده هم می‌توانند ابزارهای کیان
//!   را صدا بزنند.
//!
//! همهٔ توابع قطعی‌اند؛ هیچ I/O و شانسی در کار نیست.
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, HashMap, HashSet};

/// یک دلتای ورودی استریم — شکل خنثیِ دلتای `tool_calls` در chat-completions
/// (هر ارائه‌دهنده‌ای با همین چهار فیلد ترجمه می‌شود).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ToolCallDelta {
    /// شاخص اسلات از ارائه‌دهنده؛ `None` مثل `0` حساب می‌شود (پورت `py`: `raw_idx`).
    pub index: Option<usize>,
    /// شناسهٔ فراخوانی — حضورِ شناسهٔ تازه در همان شاخص یعنی «فراخوانی بعدی».
    pub id: Option<String>,
    /// نام تابع اگر در این دلتا آمده (بعضی گیت‌وی‌ها هر دلتا می‌فرستند).
    pub name: Option<String>,
    /// تکهٔ بعدی متن آرگومان‌ها (الحاق تکه‌ها = متن کامل JSON).
    pub arguments: Option<String>,
}

/// یک فراخوانی ابزارِ کامل‌شده پس از انباشت.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AccumulatedToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

/// وضعیت اسکن افزایشی براکت‌ها برای یک اسلات — هرگز از سرِ متن شروع نمی‌کنیم:
/// اسکن از «آخرین تکه» خطی می‌ماند و برای ارائه‌دهنده‌هایی که هرگز به اینجا
/// نمی‌رسند رایگان است (پورت `_arg_scan`).
#[derive(Debug, Clone, Copy, Default)]
struct ArgScanState {
    parts_scanned: usize,
    depth: i64,
    in_string: bool,
    escaped: bool,
    started: bool,
}

/// انباشتگر استریم `tool_calls` — پورت `_ToolCallAccumulator`.
///
/// نکتهٔ کلیدی `Ollama`: نقطهٔ پایانی سازگار با Ollama برای هر فراخوانیِ دستهٔ
/// موازی، همه را روی `index=0` می‌فرستد و فقط با `id` از هم جدا می‌شوند؛ پس
/// شناسهٔ تازه روی شاخصِ دیده‌شده، به اسلاتِ تازه هدایت می‌شود — اما فقط وقتی
/// فراخوانیِ جاری از نظر ساختاری بسته شده باشد.
#[derive(Debug, Default)]
pub struct ToolCallStreamAccumulator {
    slots: BTreeMap<usize, AccumulatedToolCall>,
    notified: HashSet<usize>,
    last_id_at_idx: HashMap<usize, String>,
    active_slot_by_idx: HashMap<usize, usize>,
    argument_parts: HashMap<usize, Vec<String>>,
    arg_scan: HashMap<usize, ArgScanState>,
}

impl ToolCallStreamAccumulator {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// یک دلتا را ادغام می‌کند؛ بارِ اول که نامِ تابع کامل شد، همان نام برمی‌گردد.
    ///
    /// قواعد (پورت خط‌به‌خط از `feed` پایتونی):
    /// ۱. شاخص `None` = `0`؛ شناسهٔ عددی به رشته می‌آید (Poolside).
    /// ۲. شناسهٔ تازه روی شاخصِ دیده‌شده فقط وقتی اسلاتِ تازه باز می‌کند که
    ///    [`Self::starts_new_call`] بگوید «فراخوانی جاری بسته شده».
    /// ۳. نام تابع با **انتساب** جایگزین می‌شود، نه الحاق — وگرنه نامِ تکراری
    ///    گیت‌وی‌های `MiniMax`/NIM «`read_fileread_file`» می‌سازد.
    /// ۴. تکهٔ آرگومان‌ها جدا انبار می‌شود؛ [`Self::materialize`] یک‌بار به‌هم
    ///    می‌چسباند (الحاق دلتا به دلتا روی آرگومان‌های بزرگ درجهٔ ۲ است).
    pub fn feed(&mut self, delta: &ToolCallDelta) -> Option<String> {
        let raw_idx = delta.index.unwrap_or(0);
        let delta_id = delta.id.clone().unwrap_or_default();

        self.active_slot_by_idx.entry(raw_idx).or_insert(raw_idx);
        if !delta_id.is_empty()
            && self
                .last_id_at_idx
                .get(&raw_idx)
                .is_some_and(|prev| prev != &delta_id)
            && self.starts_new_call(
                delta.name.as_deref(),
                self.active_slot_by_idx[&raw_idx],
            )
        {
            let fresh = self.slots.keys().next_back().copied().unwrap_or(0) + 1;
            self.active_slot_by_idx.insert(raw_idx, fresh);
        }
        if !delta_id.is_empty() {
            self.last_id_at_idx.insert(raw_idx, delta_id);
        }
        let idx = self.active_slot_by_idx[&raw_idx];

        let entry = self.slots.entry(idx).or_default();
        if let Some(id) = delta.id.as_deref().filter(|id| !id.is_empty()) {
            entry.id = id.to_string();
        }
        if let Some(name) = delta.name.as_deref().filter(|name| !name.is_empty()) {
            // انتساب، نه الحاق — اصلاح MiniMax / NVIDIA NIM / OpenRouter
            // «read_fileread_file» اینجاست.
            entry.name = name.to_string();
        }
        if let Some(args) = delta.arguments.as_deref().filter(|args| !args.is_empty()) {
            self.argument_parts.entry(idx).or_default().push(args.to_owned());
        }

        let name = entry.name.clone();
        if !name.is_empty() && self.notified.insert(idx) {
            return Some(name);
        }
        None
    }

    /// شناسهٔ تازه روی شاخصِ مصرف‌شده، فراخوانیِ تازه باز می‌کند یا ادامهٔ فعلی؟
    ///
    /// فقط دلتای **نام‌دار** می‌تواند فراخوانی باز کند (LM Studio و Kimi روی هر
    /// تکهٔ آرگومان شناسهٔ تازه می‌فرستند بدون نام؛ بعضی مسیرهای `MiniMax` هم نام
    /// را هر بار می‌فرستند) و نامِ همان فقط وقتی باز می‌کند که آرگومان‌های
    /// فراخوانیِ جاری بسته شده باشند — دو فراخوانیِ موازیِ یک ابزار باید باز هم
    /// از هم جدا شوند (پورت `_starts_new_call`).
    fn starts_new_call(&mut self, incoming_name: Option<&str>, slot: usize) -> bool {
        let Some(name) = incoming_name.filter(|n| !n.is_empty()) else {
            return false;
        };
        if self.slots.get(&slot).map(|e| e.name.as_str()) != Some(name) {
            return true;
        }
        self.arguments_closed(slot)
    }

    /// اتمام ساختاری (بالانسِ براکت بیرونِ رشته) — نه JSON معتبرِ اکید، تا
    /// فراخوانیِ قابل‌ترمیم با ویرگول انتهایی هم اسلاتش را می‌بندد. فقط تکه‌های
    /// افزوده‌شده از آخرین اسکن را می‌بیند (پورت `_arguments_closed`).
    fn arguments_closed(&mut self, slot: usize) -> bool {
        let mut state = self.arg_scan.get(&slot).copied().unwrap_or_default();
        let parts = self.argument_parts.get(&slot).cloned().unwrap_or_default();
        for ch in parts.iter().flat_map(|p| p.chars()).skip(state.parts_scanned) {
            if state.in_string {
                if state.escaped {
                    state.escaped = false;
                } else if ch == '\\' {
                    state.escaped = true;
                } else if ch == '"' {
                    state.in_string = false;
                }
            } else if ch == '"' {
                state.in_string = true;
            } else if ch == '{' || ch == '[' {
                state.depth += 1;
                state.started = true;
            } else if ch == '}' || ch == ']' {
                state.depth -= 1;
            }
        }
        state.parts_scanned = parts.iter().map(|p| p.chars().count()).sum();
        self.arg_scan.insert(slot, state);
        state.started && state.depth == 0 && !state.in_string
    }

    /// تکه‌های آرگومان را در هر اسلات به `arguments` می‌چسباند؛ idempotent —
    /// و اسلات‌ها به ترتیب شاخص برمی‌گردند (پورت `materialize`).
    #[must_use]
    pub fn materialize(&self) -> Vec<AccumulatedToolCall> {
        self.slots
            .iter()
            .map(|(idx, entry)| AccumulatedToolCall {
                id: entry.id.clone(),
                name: entry.name.clone(),
                arguments: self
                    .argument_parts
                    .get(idx)
                    .map_or_else(|| entry.arguments.clone(), |parts| parts.concat()),
            })
            .collect()
    }

    /// تعداد اسلات‌های بازشده — برای تله‌متری و تست.
    #[must_use]
    pub fn slot_count(&self) -> usize {
        self.slots.len()
    }
}

/// یک فراخوانی ابزارِ استخراج‌شده از متنِ دستیار.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedToolCall {
    pub name: String,
    pub arguments: serde_json::Value,
}

/// استخراج بلوک‌های `<tool_call>{"name": ..., "arguments": ...}</tool_call>`
/// از پاسخ متنی دستیار (پورت `validate_and_extract_tool_calls` در
/// `Hermes-Function-Calling/utils.py`).
///
/// هر بلوک اول با `serde_json` اکید می‌خوانَد؛ اگر شکست، فال‌بکِ آسان‌گیر:
/// ویرگول‌های انتهایی و کلید/رشته‌های یک‌نقل‌قولی (معادل `ast.literal_eval`).
/// بلوک‌های خراب پرت می‌شوند و بقیه ادامه می‌یابند — خطای تجمیعی برنمی‌گردد.
/// متنِ بیرون از بلوک‌ها (تحلیل، خلاصهٔ هرمس) نادیده گرفته می‌شود.
#[must_use]
pub fn extract_xml_tool_calls(text: &str) -> Vec<ParsedToolCall> {
    let mut calls = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find("<tool_call>") {
        let after_tag = &rest[start + "<tool_call>".len()..];
        let Some(end) = after_tag.find("</tool_call>") else {
            break; // بلوکِ بسته‌نشده — استریمِ ناتمام؛ هیچ چیز استخراج نمی‌شود
        };
        let body = after_tag[..end].trim();
        if let Some(call) = parse_tool_call_body(body) {
            calls.push(call);
        }
        rest = &after_tag[end + "</tool_call>".len()..];
    }
    calls
}

/// بدنهٔ یک بلوک `tool_call` را می‌خواند: اکید، بعد آسان‌گیر.
fn parse_tool_call_body(body: &str) -> Option<ParsedToolCall> {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(body) {
        return value_to_call(&v);
    }
    let v = lenient_json_object(body)?;
    value_to_call(&v)
}

/// فال‌بک آسان‌گیر: ویرگول انتهایی را می‌خورَد و کلید/رشتهٔ یک‌نقل‌قولی را به
/// دوتایی می‌بَرد — معادل سبک `ast.literal_eval` بدون هرگز اجرای کد.
fn lenient_json_object(body: &str) -> Option<serde_json::Value> {
    let trimmed = body.trim();
    if !trimmed.starts_with('{') {
        return None;
    }
    // گام ۱: کلید/رشتهٔ یک‌نقل‌قولی → دوتایی (درست‌کار با escapeهای پایتونی)
    let chars: Vec<char> = trimmed.chars().collect();
    let mut converted = String::with_capacity(trimmed.len());
    let mut in_string: Option<char> = None;
    let mut i = 0usize;
    while i < chars.len() {
        let c = chars[i];
        match in_string {
            None => match c {
                '\'' | '"' => {
                    in_string = Some(c);
                    converted.push('"');
                }
                _ => converted.push(c),
            },
            Some(q) => {
                if c == '\\' && i + 1 < chars.len() {
                    let next = chars[i + 1];
                    match (q, next) {
                        // \' داخل یک‌نقل‌قولی = خودِ کوتیشن — در JSON دوتایی بدون escape
                        ('\'', '\'') => converted.push('\''),
                        // \" داخل یک‌نقل‌قولی = کوتیشنِ دوتایی — در JSON escape می‌خواهد
                        ('\'', '"') => converted.push_str("\\\""),
                        // \\ در هر دو = بک‌اسلش escape‌شده
                        (_, '\\') => converted.push_str("\\\\"),
                        // بقیهٔ escapeها (\n، \t، …) همان‌طور در JSON معتبرند
                        _ => {
                            converted.push('\\');
                            converted.push(next);
                        }
                    }
                    i += 1;
                } else if c == q {
                    in_string = None;
                    converted.push('"');
                } else {
                    converted.push(c);
                }
            }
        }
        i += 1;
    }
    // گام ۲: ویرگول انتهایی قبل از } یا ] حذف می‌شود (فقط بیرونِ رشته)
    let chars: Vec<char> = converted.chars().collect();
    let mut cleaned = String::with_capacity(converted.len());
    let mut in_string = false;
    let mut j = 0usize;
    while j < chars.len() {
        let c = chars[j];
        if in_string {
            if c == '\\' && j + 1 < chars.len() {
                cleaned.push(c);
                cleaned.push(chars[j + 1]);
                j += 1;
            } else if c == '"' {
                in_string = false;
            }
            cleaned.push(c);
        } else if c == '"' {
            in_string = true;
            cleaned.push(c);
        } else if c == ',' {
            // نگاه به اولین نویسهٔ غیرخالی بعدی — اگر بستنِ بلوک بود، کاما می‌خوریم
            let mut k = j + 1;
            while k < chars.len() && chars[k].is_whitespace() {
                k += 1;
            }
            if !(k < chars.len() && (chars[k] == '}' || chars[k] == ']')) {
                cleaned.push(c);
            }
        } else {
            cleaned.push(c);
        }
        j += 1;
    }
    serde_json::from_str(&cleaned).ok()
}

/// یک `Value` به [`ParsedToolCall`] فقط وقتی شکل `{"name": ..., "arguments": ...}`
/// دارد؛ `arguments` رشته‌ایِ JSON-encoded هم باز می‌شود (بعضی مدل‌ها دوتا
/// encode می‌کنند).
fn value_to_call(v: &serde_json::Value) -> Option<ParsedToolCall> {
    let obj = v.as_object()?;
    let name = obj.get("name")?.as_str()?.to_owned();
    let arguments = match obj.get("arguments") {
        None => serde_json::Value::Object(serde_json::Map::new()),
        Some(serde_json::Value::String(s)) => {
            serde_json::from_str(s).unwrap_or(serde_json::Value::Object(serde_json::Map::new()))
        }
        Some(other) => other.clone(),
    };
    if name.is_empty() {
        return None;
    }
    Some(ParsedToolCall { name, arguments })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn delta(index: usize, id: &str, name: &str, args: &str) -> ToolCallDelta {
        ToolCallDelta {
            index: Some(index),
            id: (!id.is_empty()).then(|| id.to_owned()),
            name: (!name.is_empty()).then(|| name.to_owned()),
            arguments: (!args.is_empty()).then(|| args.to_owned()),
        }
    }

    /// نام تکراری گیت‌وی (MiniMax/NIM) با انتساب جایگزین می‌شود نه الحاق.
    #[test]
    fn duplicate_name_overwrites_never_concatenates() {
        let mut acc = ToolCallStreamAccumulator::new();
        assert_eq!(
            acc.feed(&delta(0, "c1", "read_file", "")).as_deref(),
            Some("read_file")
        );
        // گیت‌وی بدنام: نام کامل را در هر دلتا می‌فرستد
        acc.feed(&delta(0, "c1", "read_file", "{\"path\":"));
        acc.feed(&delta(0, "c1", "read_file", "\"a.rs\"}"));
        let calls = acc.materialize();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "read_file", "هرگز read_fileread_file نشود");
        assert_eq!(calls[0].arguments, "{\"path\":\"a.rs\"}");
    }

    /// شاخصِ مصرف‌شدهٔ Ollama (همه روی 0) با شناسهٔ تازه به اسلاتِ تازه می‌رود
    /// — وقتی فراخوانیِ قبلی از نظر ساختاری بسته شده است.
    #[test]
    fn reused_index_after_closed_object_opens_fresh_slot() {
        let mut acc = ToolCallStreamAccumulator::new();
        acc.feed(&delta(0, "a", "tool_one", "{\"x\":1}"));
        acc.feed(&delta(0, "b", "tool_one", "{\"y\":2}"));
        let calls = acc.materialize();
        assert_eq!(calls.len(), 2, "دو فراخوانی موازیِ یک ابزار باید جدا شوند");
        assert_eq!(calls[0].id, "a");
        assert_eq!(calls[1].id, "b");
        assert_eq!(calls[0].arguments, "{\"x\":1}");
        assert_eq!(calls[1].arguments, "{\"y\":2}");
    }

    /// شناسهٔ تازه با شاخصِ تکراری وقتی آرگومان‌ها هنوز بازند، ادامهٔ همان
    /// فراخوانی است (LM Studio/Kimi روی هر تکه شناسهٔ تازه می‌فرستند).
    #[test]
    fn fresh_id_on_open_object_continues_current_call() {
        let mut acc = ToolCallStreamAccumulator::new();
        acc.feed(&delta(0, "id-1", "write", "{\"content\":"));
        acc.feed(&delta(0, "id-2", "", "\"سلام\"}"));
        let calls = acc.materialize();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].id, "id-2", "آخرین شناسه برنده است");
        assert_eq!(calls[0].arguments, "{\"content\":\"سلام\"}");
    }

    /// عمق براکت داخل رشته حساب نمی‌شود — `}` داخل مقدار رشته‌ای اسلات را نمی‌بندد.
    #[test]
    fn braces_inside_strings_do_not_close_slot() {
        let mut acc = ToolCallStreamAccumulator::new();
        // دلتای اول: آکولادِ پایانی JSON «داخل» رشتهٔ باز است — اسلات هنوز باز است
        acc.feed(&delta(0, "x", "echo", r#"{"text":"}"#));
        // دلتای دوم با نامِ همان ابزار: چون آرگومان‌ها هنوز بازند، ادامهٔ همان فراخوانی است
        acc.feed(&delta(0, "y", "echo", r#""}"#));
        let calls = acc.materialize();
        assert_eq!(calls.len(), 1, "آکولادِ داخل رشته نباید فراخوانی را ببندد");
        assert_eq!(calls[0].arguments, r#"{"text":"}"}"#);
    }

    /// ویرگول انتهایی (JSON قابل‌ترمیم) هم اسلات را می‌بندد.
    #[test]
    fn trailing_comma_still_closes_slot() {
        let mut acc = ToolCallStreamAccumulator::new();
        acc.feed(&delta(0, "a", "t", "{\"a\":1,}"));
        acc.feed(&delta(0, "b", "t", "{\"b\":2}"));
        assert_eq!(acc.slot_count(), 2);
    }

    /// دلتای بدون شاخص = شاخص ۰ و استریم نام‌فقط تکه‌تکه آرگومان می‌آید.
    #[test]
    fn unnamed_index_streams_argument_fragments() {
        let mut acc = ToolCallStreamAccumulator::new();
        acc.feed(&delta(0, "c", "search", "{\"q\": \""));
        acc.feed(&ToolCallDelta { index: None, id: None, name: None, arguments: Some("کیان".into()) });
        acc.feed(&ToolCallDelta { index: None, id: None, name: None, arguments: Some("\"}".into()) });
        let calls = acc.materialize();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].arguments, "{\"q\": \"کیان\"}");
    }

    /// شناسهٔ عددی (Poolside) به رشته می‌آید.
    #[test]
    fn integer_id_becomes_string() {
        let mut acc = ToolCallStreamAccumulator::new();
        acc.feed(&ToolCallDelta {
            index: Some(0),
            id: Some("42".into()),
            name: Some("t".into()),
            arguments: None,
        });
        let calls = acc.materialize();
        assert_eq!(calls[0].id, "42");
    }

    /// `extract_xml_tool_calls` — بلوک استاندارد هرمس با آرگومان‌های واقعی.
    #[test]
    fn extracts_standard_hermes_tool_call() {
        let text = "در حال جست‌وجو…\n<tool_call>\n{\"name\": \"web_search\", \"arguments\": {\"query\": \"KIAN\", \"limit\": 3}}\n</tool_call>\nخلاصهٔ نتیجه‌ها بعداً.";
        let calls = extract_xml_tool_calls(text);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "web_search");
        assert_eq!(calls[0].arguments, json!({"query": "KIAN", "limit": 3}));
    }

    /// چند بلوک موازی در یک پاسخ — همه استخراج می‌شوند به ترتیب.
    #[test]
    fn extracts_multiple_parallel_calls_in_order() {
        let text = "<tool_call>{\"name\": \"a\", \"arguments\": {\"i\": 1}}</tool_call>"
            .to_owned()
            + "میان‌متنِ تحلیل مدل"
            + "<tool_call>{\"name\": \"b\", \"arguments\": {\"i\": 2}}</tool_call>";
        let calls = extract_xml_tool_calls(&text);
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].name, "a");
        assert_eq!(calls[1].name, "b");
    }

    /// فال‌بک آسان‌گیر: ویرگول انتهایی + کلید یک‌نقل‌قولی.
    #[test]
    fn lenient_fallback_handles_trailing_comma_and_single_quotes() {
        let text = "<tool_call>{'name': 'wallet_balance', 'arguments': {'addr': 'x', }}</tool_call>";
        let calls = extract_xml_tool_calls(text);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "wallet_balance");
        assert_eq!(calls[0].arguments, json!({"addr": "x"}));
    }

    /// `arguments` به‌صورت رشتهٔ JSON-encoded (دو-انکد مدل) باز می‌شود.
    #[test]
    fn string_encoded_arguments_are_decoded() {
        let text = "<tool_call>{\"name\": \"t\", \"arguments\": \"{\\\"k\\\": 7}\"}</tool_call>";
        let calls = extract_xml_tool_calls(text);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].arguments, json!({"k": 7}));
    }

    /// بلوکِ خراب پرت می‌شود، بلوکِ بعدی سالم می‌ماند؛ بلوکِ بسته‌نشده هیچ نیست.
    #[test]
    fn broken_blocks_are_skipped_stream_safe() {
        let bad = "<tool_call>{\"name\": } NOPE</tool_call>";
        assert_eq!(extract_xml_tool_calls(bad), [] as [_; 0]);
        let mixed = format!("{bad}<tool_call>{{\"name\": \"ok\", \"arguments\": {{}}}}</tool_call>");
        let calls = extract_xml_tool_calls(&mixed);
        assert_eq!(calls.len(), 1, "بلوکِ خراب بلوکِ سالم را نمی‌کشد");
        assert_eq!(calls[0].name, "ok");
        // استریم ناتمام: تگ بسته نیست → هیچ
        assert_eq!(extract_xml_tool_calls("{\"name\": \"x\""), [] as [_; 0]);
        // متن معمولی بدون تگ → هیچ
        assert_eq!(extract_xml_tool_calls("پاسخ سادهٔ بدون ابزار"), [] as [_; 0]);
    }
}
