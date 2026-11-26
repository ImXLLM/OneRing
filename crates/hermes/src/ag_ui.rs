//! `ag_ui` — the AG-UI 1.0 event protocol for streaming agent-to-UI events (Wave 2 port).
//!
//! Ported from the official reference crate `ag-ui-core` (vault `03_ag_ui_protocol_and_schemas/rust/`):
//! Event type names are `SCREAMING_SNAKE_CASE` and field names are `camelCase` so the
//! generative-UI renderer on web/mini-app can consume these events without a translator.
//!
//! Kian extends the official set with three extension events (Activity/ActivityDelta/SubagentSpawn)
//! so that studio progress and subagent spawns also stream over the same standard
//! pipeline — the same pattern `OpenBot` uses for its subnodes.
//!
//! This module is structure and constructors only; delivery over `SSE`/`WebSocket` belongs to the server layer.
//! All constructors are deterministic and tested.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// AG-UI 1.0 event type — names match the official `ag-ui-core` crate (`SCREAMING_SNAKE_CASE`).
///
/// The last three events (`ActivitySnapshot`, `ActivityDelta`, `SubagentSpawn`) are Kian's
/// extension for the studio activity channel and subagents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AgUiEventType {
    /// Start of a model text message.
    TextMessageStart,
    /// Text chunk of a model message (stream delta).
    TextMessageContent,
    /// End of a model text message.
    TextMessageEnd,
    /// Start of a tool call.
    ToolCallStart,
    /// Tool call argument chunk.
    ToolCallArgs,
    /// End of tool call arguments.
    ToolCallEnd,
    /// Snapshot of the entire shared state.
    StateSnapshot,
    /// Incremental state patch (JSON Patch format).
    StateDelta,
    /// Activity channel snapshot (Kian extension).
    ActivitySnapshot,
    /// Activity channel delta (Kian extension).
    ActivityDelta,
    /// Subagent spawn with a deterministic thread (Kian extension).
    SubagentSpawn,
    /// Agent run start.
    RunStarted,
    /// Successful run end.
    RunFinished,
    /// Run error.
    RunError,
}

/// An AG-UI event — flat envelope with optional fields; an unset field is not serialized
/// so the renderer sees exactly the official contract (`messageId`, `toolCallId`,
/// `delta`, `state`, …).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgUiEvent {
    /// Event type — the envelope discriminator key.
    #[serde(rename = "type")]
    pub event_type: AgUiEventType,
    /// Seconds since epoch (port of the official `BaseEvent` `timestamp`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<f64>,
    /// Text message ID (`TextMessage*`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
    /// Text/argument chunk (`TextMessageContent` / `ToolCallArgs`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta: Option<String>,
    /// Tool call ID (`ToolCall*`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    /// Tool name (`ToolCallStart`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_name: Option<String>,
    /// Parent message of the tool call (`ToolCallStart`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_message_id: Option<String>,
    /// State snapshot (`StateSnapshot`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<JsonValue>,
    /// State patch (`StateDelta`) or activity (`ActivityDelta`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch: Option<JsonValue>,
    /// Activity channel (`ActivitySnapshot` / `ActivityDelta`) — e.g. `a2ui_recovery`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activity_type: Option<String>,
    /// Parent thread of the subagent (`SubagentSpawn`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_thread_id: Option<String>,
    /// Deterministic thread of the subagent (`SubagentSpawn`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subagent_thread_id: Option<String>,
    /// Thread ID of the run (`RunStarted` / `RunFinished`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
    /// Error message (`RunError`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

/// Epoch seconds for the `timestamp` field — the common touchpoint of all constructors.
fn now_epoch_secs() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0.0, |d| d.as_secs_f64())
}

impl AgUiEvent {
    /// Base envelope with type and timestamp — shared constructor for the rest.
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

    /// Start of a model text message — studio/Hermes response stream.
    #[must_use]
    pub fn text_message_start(message_id: impl Into<String>) -> Self {
        Self { event_type: AgUiEventType::TextMessageStart, message_id: Some(message_id.into()), ..Self::base(AgUiEventType::TextMessageStart) }
    }

    /// Text chunk of a model message.
    #[must_use]
    pub fn text_message_content(message_id: impl Into<String>, delta: impl Into<String>) -> Self {
        Self {
            event_type: AgUiEventType::TextMessageContent,
            message_id: Some(message_id.into()),
            delta: Some(delta.into()),
            ..Self::base(AgUiEventType::TextMessageContent)
        }
    }

    /// End of a model text message.
    #[must_use]
    pub fn text_message_end(message_id: impl Into<String>) -> Self {
        Self { event_type: AgUiEventType::TextMessageEnd, message_id: Some(message_id.into()), ..Self::base(AgUiEventType::TextMessageEnd) }
    }

    /// Start of a tool call (e.g. `render_a2ui` or `kb.search`).
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

    /// Tool call argument chunk (partial JSON in flight).
    #[must_use]
    pub fn tool_call_args(tool_call_id: impl Into<String>, delta: impl Into<String>) -> Self {
        Self {
            event_type: AgUiEventType::ToolCallArgs,
            tool_call_id: Some(tool_call_id.into()),
            delta: Some(delta.into()),
            ..Self::base(AgUiEventType::ToolCallArgs)
        }
    }

    /// End of tool call arguments.
    #[must_use]
    pub fn tool_call_end(tool_call_id: impl Into<String>) -> Self {
        Self { event_type: AgUiEventType::ToolCallEnd, tool_call_id: Some(tool_call_id.into()), ..Self::base(AgUiEventType::ToolCallEnd) }
    }

    /// Snapshot of the shared state.
    #[must_use]
    pub fn state_snapshot(state: JsonValue) -> Self {
        Self { event_type: AgUiEventType::StateSnapshot, state: Some(state), ..Self::base(AgUiEventType::StateSnapshot) }
    }

    /// Incremental state patch (JSON Patch).
    #[must_use]
    pub fn state_delta(patch: JsonValue) -> Self {
        Self { event_type: AgUiEventType::StateDelta, patch: Some(patch), ..Self::base(AgUiEventType::StateDelta) }
    }

    /// Studio activity channel snapshot (Kian extension) — e.g. A2UI self-heal status.
    #[must_use]
    pub fn activity_snapshot(activity_type: impl Into<String>, snapshot: JsonValue) -> Self {
        Self {
            event_type: AgUiEventType::ActivitySnapshot,
            activity_type: Some(activity_type.into()),
            state: Some(snapshot),
            ..Self::base(AgUiEventType::ActivitySnapshot)
        }
    }

    /// Activity channel delta (Kian extension).
    #[must_use]
    pub fn activity_delta(activity_type: impl Into<String>, patch: JsonValue) -> Self {
        Self {
            event_type: AgUiEventType::ActivityDelta,
            activity_type: Some(activity_type.into()),
            patch: Some(patch),
            ..Self::base(AgUiEventType::ActivityDelta)
        }
    }

    /// Subagent spawn with a deterministic thread (Kian extension) — the same derivation
    /// that `approvals::deterministic_subagent_thread_id` builds, so the client can locate
    /// the subagent transcript before the result arrives.
    #[must_use]
    pub fn subagent_spawn(parent_thread_id: impl Into<String>, subagent_thread_id: impl Into<String>) -> Self {
        Self {
            event_type: AgUiEventType::SubagentSpawn,
            parent_thread_id: Some(parent_thread_id.into()),
            subagent_thread_id: Some(subagent_thread_id.into()),
            ..Self::base(AgUiEventType::SubagentSpawn)
        }
    }

    /// Agent run start on a thread.
    #[must_use]
    pub fn run_started(thread_id: impl Into<String>) -> Self {
        Self { event_type: AgUiEventType::RunStarted, thread_id: Some(thread_id.into()), ..Self::base(AgUiEventType::RunStarted) }
    }

    /// Successful run end.
    #[must_use]
    pub fn run_finished(thread_id: impl Into<String>) -> Self {
        Self { event_type: AgUiEventType::RunFinished, thread_id: Some(thread_id.into()), ..Self::base(AgUiEventType::RunFinished) }
    }

    /// Run error.
    #[must_use]
    pub fn run_error(error_message: impl Into<String>) -> Self {
        Self { event_type: AgUiEventType::RunError, error_message: Some(error_message.into()), ..Self::base(AgUiEventType::RunError) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Serialization follows the official AG-UI contract: `type` uses `SCREAMING_SNAKE`
    /// and the remaining fields are camelCase; an unset field is not serialized.
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

        let text = AgUiEvent::text_message_content("m1", "\u{633}\u{644}\u{627}\u{645}");
        let json = serde_json::to_value(&text).expect("serialize");
        assert_eq!(json["type"], "TEXT_MESSAGE_CONTENT");
        assert_eq!(json["messageId"], "m1");
        assert_eq!(json["delta"], "\u{633}\u{644}\u{627}\u{645}");
    }

    /// State, activity and subagent events serialize/deserialize with a full JSON payload.
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

    /// Full studio stream cycle: a run starts, text arrives, a tool is invoked,
    /// state is snapshotted, and the run closes with an error or success.
    #[test]
    fn full_stream_lifecycle_builds_all_fourteen_event_types() {
        let stream = vec![
            AgUiEvent::run_started("th-1"),
            AgUiEvent::text_message_start("m1"),
            AgUiEvent::text_message_content("m1", "\u{62f}\u{631} \u{62d}\u{627}\u{644} \u{633}\u{627}\u{62e}\u{62a}…"),
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
            AgUiEvent::run_error("\u{67e}\u{627}\u{6a9}\u{62a} \u{62e}\u{631}\u{627}\u{628}"),
        ];
        assert_eq!(stream.len(), 14);
        let types: Vec<AgUiEventType> = stream.iter().map(|e| e.event_type).collect();
        let distinct: std::collections::BTreeSet<_> = types.iter().collect();
        assert_eq!(distinct.len(), 14, "every event type exercised exactly once");
        // all serialize and round-trip back to the same type.
        for ev in &stream {
            let json = serde_json::to_value(ev).expect("serialize");
            let back: AgUiEvent = serde_json::from_value(json).expect("deserialize");
            assert_eq!(back.event_type, ev.event_type);
        }
    }
}
