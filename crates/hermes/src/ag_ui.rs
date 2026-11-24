//! `ag_ui` — پروتکل رخداد AG-UI 1.0 برای استریم ایجنت‌به‌UI (پورت Wave 2).
//!
//! پورت از کریت مرجع رسمی `ag-ui-core` (گنجینهٔ `03_ag_ui_protocol_and_schemas/rust/`):
//! نام تایپ رخدادها `SCREAMING_SNAKE_CASE` و نام فیلدها `camelCase` است تا رندر
//! جنریتیو-UI روی وب/مینی‌اپ بدون ترجمه‌گر، این رخدادها را مصرف کند.
//!
//! کیان مجموعهٔ رسمی را با سه رخداد پسوندی (Activity/ActivityDelta/SubagentSpawn)
//! گسترش می‌دهد تا پیشرفت استودیو و تولد ساب‌ایجنت هم روی همین لولهٔ استاندارد
//! استریم شود — همان الگویی که `OpenBot` برای زیرگره‌هایش استفاده می‌کند.
//!
//! این ماژول فقط ساختار و سازنده است؛ ارسال روی `SSE`/`WebSocket` لایهٔ سرور است.
//! همهٔ سازنده‌ها قطعی‌اند و تست‌شده.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// تایپ رخداد AG-UI 1.0 — نام‌ها مطابق کریت رسمی `ag-ui-core` (`SCREAMING_SNAKE_CASE`).
///
/// سه رخداد آخر (`ActivitySnapshot`, `ActivityDelta`, `SubagentSpawn`) پسوندِ
/// کیان برای کانال فعالیت استودیو و ساب‌ایجنت‌هاست.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AgUiEventType {
    /// آغاز پیام متنی مدل.
    TextMessageStart,
    /// تکهٔ متن پیام مدل (دلتای استریم).
    TextMessageContent,
    /// پایان پیام متنی مدل.
    TextMessageEnd,
    /// آغاز فراخوانی ابزار.
    ToolCallStart,
    /// تکهٔ آرگومان‌های فراخوانی ابزار.
    ToolCallArgs,
    /// پایان آرگومان‌های فراخوانی ابزار.
    ToolCallEnd,
    /// عکس فوری کل state اشتراکی.
    StateSnapshot,
    /// پچ افزایشی state (قالب JSON Patch).
    StateDelta,
    /// عکس فوری کانال فعالیت (پسوند کیان).
    ActivitySnapshot,
    /// دلتای کانال فعالیت (پسوند کیان).
    ActivityDelta,
    /// تولد ساب‌ایجنت با thread قطعی (پسوند کیان).
    SubagentSpawn,
    /// آغاز ران ایجنت.
    RunStarted,
    /// پایان موفق ران.
    RunFinished,
    /// خطای ران.
    RunError,
}

/// یک رخداد AG-UI — پاکت تخت با فیلدهای اختیاری؛ فیلد بی‌مقدار سری نمی‌شود
/// تا رندرگر دقیقاً همان قرارداد رسمی را ببیند (`messageId`، `toolCallId`،
/// `delta`، `state`، …).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgUiEvent {
    /// تایپ رخداد — کلید متمایزکنندهٔ پاکت.
    #[serde(rename = "type")]
    pub event_type: AgUiEventType,
    /// ثانیه از epoch (پورت `timestamp` در `BaseEvent` رسمی).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<f64>,
    /// شناسهٔ پیام متنی (`TextMessage*`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
    /// تکهٔ متن/آرگومان (`TextMessageContent` / `ToolCallArgs`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta: Option<String>,
    /// شناسهٔ فراخوانی ابزار (`ToolCall*`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    /// نام ابزار (`ToolCallStart`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_name: Option<String>,
    /// پیام والد فراخوانی ابزار (`ToolCallStart`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_message_id: Option<String>,
    /// عکس فوری state (`StateSnapshot`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<JsonValue>,
    /// پچ state (`StateDelta`) یا فعالیت (`ActivityDelta`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch: Option<JsonValue>,
    /// کانال فعالیت (`ActivitySnapshot` / `ActivityDelta`) — مثلاً `a2ui_recovery`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activity_type: Option<String>,
    /// thread والد ساب‌ایجنت (`SubagentSpawn`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_thread_id: Option<String>,
    /// thread قطعی ساب‌ایجنت (`SubagentSpawn`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subagent_thread_id: Option<String>,
    /// شناسهٔ thread ران (`RunStarted` / `RunFinished`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
    /// پیام خطا (`RunError`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

/// ثانیهٔ epoch برای فیلد `timestamp` — نقطهٔ مشترک همهٔ سازنده‌ها.
fn now_epoch_secs() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0.0, |d| d.as_secs_f64())
}

impl AgUiEvent {
    /// پاکت پایه با تایپ و timestamp — سازندهٔ مشترک بقیه.
    fn base(event_type: AgUiEventType) -> Self {
        Self {
            event_type,
            timestamp: Some(now_epoch_secs()),
            message_id: None,
            delta: None,
            tool_call_id: None,
            tool_call_name: None,
            parent_message_id: None,
            state: None,
            patch: None,
            activity_type: None,
            parent_thread_id: None,
            subagent_thread_id: None,
            thread_id: None,
            error_message: None,
        }
    }

    /// آغاز پیام متنی مدل — استریم پاسخ استودیو/هرمس.
    #[must_use]
    pub fn text_message_start(message_id: impl Into<String>) -> Self {
        Self { event_type: AgUiEventType::TextMessageStart, message_id: Some(message_id.into()), ..Self::base(AgUiEventType::TextMessageStart) }
    }

    /// تکهٔ متن پیام مدل.
    #[must_use]
    pub fn text_message_content(message_id: impl Into<String>, delta: impl Into<String>) -> Self {
        Self {
            event_type: AgUiEventType::TextMessageContent,
            message_id: Some(message_id.into()),
            delta: Some(delta.into()),
            ..Self::base(AgUiEventType::TextMessageContent)
        }
    }

    /// پایان پیام متنی مدل.
    #[must_use]
    pub fn text_message_end(message_id: impl Into<String>) -> Self {
        Self { event_type: AgUiEventType::TextMessageEnd, message_id: Some(message_id.into()), ..Self::base(AgUiEventType::TextMessageEnd) }
    }

    /// آغاز فراخوانی ابزار (مثلاً `render_a2ui` یا `kb.search`).
    #[must_use]
    pub fn tool_call_start(
        tool_call_id: impl Into<String>,
        tool_call_name: impl Into<String>,
        parent_message_id: Option<String>,
    ) -> Self {
        Self {
            event_type: AgUiEventType::ToolCallStart,
            tool_call_id: Some(tool_call_id.into()),
            tool_call_name: Some(tool_call_name.into()),
            parent_message_id,
            ..Self::base(AgUiEventType::ToolCallStart)
        }
    }

    /// تکهٔ آرگومان‌های فراخوانی ابزار (JSON ناقصِ در راه).
    #[must_use]
    pub fn tool_call_args(tool_call_id: impl Into<String>, delta: impl Into<String>) -> Self {
        Self {
            event_type: AgUiEventType::ToolCallArgs,
            tool_call_id: Some(tool_call_id.into()),
            delta: Some(delta.into()),
            ..Self::base(AgUiEventType::ToolCallArgs)
        }
    }

    /// پایان آرگومان‌های فراخوانی ابزار.
    #[must_use]
    pub fn tool_call_end(tool_call_id: impl Into<String>) -> Self {
        Self { event_type: AgUiEventType::ToolCallEnd, tool_call_id: Some(tool_call_id.into()), ..Self::base(AgUiEventType::ToolCallEnd) }
    }

    /// عکس فوری state اشتراکی.
    #[must_use]
    pub fn state_snapshot(state: JsonValue) -> Self {
        Self { event_type: AgUiEventType::StateSnapshot, state: Some(state), ..Self::base(AgUiEventType::StateSnapshot) }
    }

    /// پچ افزایشی state (JSON Patch).
    #[must_use]
    pub fn state_delta(patch: JsonValue) -> Self {
        Self { event_type: AgUiEventType::StateDelta, patch: Some(patch), ..Self::base(AgUiEventType::StateDelta) }
    }

    /// عکس فوری کانال فعالیت استودیو (پسوند کیان) — مثلاً وضعیت خودترمیم A2UI.
    #[must_use]
    pub fn activity_snapshot(activity_type: impl Into<String>, snapshot: JsonValue) -> Self {
        Self {
            event_type: AgUiEventType::ActivitySnapshot,
            activity_type: Some(activity_type.into()),
            state: Some(snapshot),
            ..Self::base(AgUiEventType::ActivitySnapshot)
        }
    }

    /// دلتای کانال فعالیت (پسوند کیان).
    #[must_use]
    pub fn activity_delta(activity_type: impl Into<String>, patch: JsonValue) -> Self {
        Self {
            event_type: AgUiEventType::ActivityDelta,
            activity_type: Some(activity_type.into()),
            patch: Some(patch),
            ..Self::base(AgUiEventType::ActivityDelta)
        }
    }

    /// تولد ساب‌ایجنت با thread قطعی (پسوند کیان) — همان derivation که
    /// `approvals::deterministic_subagent_thread_id` می‌سازد تا کلاینت قبل از
    /// رسیدن نتیجه، رونوشت ساب‌ایجنت را پیدا کند.
    #[must_use]
    pub fn subagent_spawn(parent_thread_id: impl Into<String>, subagent_thread_id: impl Into<String>) -> Self {
        Self {
            event_type: AgUiEventType::SubagentSpawn,
            parent_thread_id: Some(parent_thread_id.into()),
            subagent_thread_id: Some(subagent_thread_id.into()),
            ..Self::base(AgUiEventType::SubagentSpawn)
        }
    }

    /// آغاز ران ایجنت روی یک thread.
    #[must_use]
    pub fn run_started(thread_id: impl Into<String>) -> Self {
        Self { event_type: AgUiEventType::RunStarted, thread_id: Some(thread_id.into()), ..Self::base(AgUiEventType::RunStarted) }
    }

    /// پایان موفق ران.
    #[must_use]
    pub fn run_finished(thread_id: impl Into<String>) -> Self {
        Self { event_type: AgUiEventType::RunFinished, thread_id: Some(thread_id.into()), ..Self::base(AgUiEventType::RunFinished) }
    }

    /// خطای ران.
    #[must_use]
    pub fn run_error(error_message: impl Into<String>) -> Self {
        Self { event_type: AgUiEventType::RunError, error_message: Some(error_message.into()), ..Self::base(AgUiEventType::RunError) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// سری رخدادها قرارداد رسمی AG-UI را رعایت می‌کند: `type` با `SCREAMING_SNAKE`
    /// و بقیهٔ فیلدها camelCase؛ فیلد بی‌مقدار سری نمی‌شود.
    #[test]
    fn events_serialize_with_official_protocol_field_names() {
        let ev = AgUiEvent::tool_call_start("call-1", "render_a2ui", Some("msg-9".into()));
        let json = serde_json::to_value(&ev).expect("serialize");
        assert_eq!(json["type"], "TOOL_CALL_START");
        assert_eq!(json["toolCallId"], "call-1");
        assert_eq!(json["toolCallName"], "render_a2ui");
        assert_eq!(json["parentMessageId"], "msg-9");
        assert!(json.get("delta").is_none(), "absent fields must not serialize");
        assert!(json.get("state").is_none());

        let text = AgUiEvent::text_message_content("m1", "سلام");
        let json = serde_json::to_value(&text).expect("serialize");
        assert_eq!(json["type"], "TEXT_MESSAGE_CONTENT");
        assert_eq!(json["messageId"], "m1");
        assert_eq!(json["delta"], "سلام");
    }

    /// رخدادهای state و فعالیت و ساب‌ایجنت با بار JSON کامل سری/دسری می‌شوند.
    #[test]
    fn state_activity_and_subagent_events_round_trip() {
        let snap = AgUiEvent::state_snapshot(json!({"progress": 0.42, "lang": "fa"}));
        let json = serde_json::to_value(&snap).expect("serialize");
        assert_eq!(json["type"], "STATE_SNAPSHOT");
        assert_eq!(json["state"]["progress"], 0.42);
        let back: AgUiEvent = serde_json::from_value(json).expect("deserialize");
        assert_eq!(back, snap);

        let delta = AgUiEvent::state_delta(json!([{"op": "replace", "path": "/progress", "value": 1.0}]));
        let json = serde_json::to_value(&delta).expect("serialize");
        assert_eq!(json["type"], "STATE_DELTA");
        assert_eq!(json["patch"][0]["op"], "replace");

        let act = AgUiEvent::activity_delta("a2ui_recovery", json!({"attempt": 2}));
        let json = serde_json::to_value(&act).expect("serialize");
        assert_eq!(json["type"], "ACTIVITY_DELTA");
        assert_eq!(json["activityType"], "a2ui_recovery");

        let spawn = AgUiEvent::subagent_spawn("11111111-2222-8333-4444-555555555555",
            "aaaaaaaa-2222-8333-4444-555555555555");
        let json = serde_json::to_value(&spawn).expect("serialize");
        assert_eq!(json["type"], "SUBAGENT_SPAWN");
        assert_eq!(json["parentThreadId"], "11111111-2222-8333-4444-555555555555");
        assert_eq!(json["subagentThreadId"], "aaaaaaaa-2222-8333-4444-555555555555");
    }

    /// چرخهٔ کامل استریم استودیو: ران شروع می‌شود، متن می‌آید، ابزار صدا زده
    /// می‌شود، state عکس گرفته می‌شود و ران با خطا یا موفقیت می‌بندد.
    #[test]
    fn full_stream_lifecycle_builds_all_fourteen_event_types() {
        let stream = vec![
            AgUiEvent::run_started("th-1"),
            AgUiEvent::text_message_start("m1"),
            AgUiEvent::text_message_content("m1", "در حال ساخت…"),
            AgUiEvent::text_message_end("m1"),
            AgUiEvent::tool_call_start("c1", "kb.search", Some("m1".into())),
            AgUiEvent::tool_call_args("c1", "{\"q\":"),
            AgUiEvent::tool_call_end("c1"),
            AgUiEvent::activity_snapshot("a2ui_recovery", json!({"attempt": 1})),
            AgUiEvent::activity_delta("a2ui_recovery", json!({"attempt": 2})),
            AgUiEvent::state_snapshot(json!({})),
            AgUiEvent::state_delta(json!([])),
            AgUiEvent::subagent_spawn("th-1", "th-sub"),
            AgUiEvent::run_finished("th-1"),
            AgUiEvent::run_error("پاکت خراب"),
        ];
        assert_eq!(stream.len(), 14);
        let types: Vec<AgUiEventType> = stream.iter().map(|e| e.event_type).collect();
        let distinct: std::collections::BTreeSet<_> = types.iter().collect();
        assert_eq!(distinct.len(), 14, "every event type exercised exactly once");
        // همه سری می‌شوند و دوباره به همان تایپ برمی‌گردند.
        for ev in &stream {
            let json = serde_json::to_value(ev).expect("serialize");
            let back: AgUiEvent = serde_json::from_value(json).expect("deserialize");
            assert_eq!(back.event_type, ev.event_type);
        }
    }
}
