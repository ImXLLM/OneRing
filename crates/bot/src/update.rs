//! Pure Telegram update parsing — from `serde_json::Value` to [`BotUpdate`].
//!
//! One parsing path for both shells (teloxide and the kernel webhook): the teloxide shell
//! converts every `Update` to a `Value` via `serde_json::to_value` and parsing happens
//! right here — one meaning, one test. Anything non-message/non-text → `None` (fail-closed;
//! it never crashes).

use serde_json::Value;

/// Minimal Telegram update — only what the orchestrator needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BotUpdate {
    /// Telegram update id — for trace/log.
    pub update_id: i64,
    /// Id of the chat to reply to (private chat).
    pub chat_id: i64,
    /// Human sender id — identity from a trusted channel, never from the user body.
    pub from_id: i64,
    /// Telegram username — may be absent (explicit Option).
    pub username: Option<String>,
    /// Command without `/` and without the `@BotBot` suffix — lower-case.
    pub command: Option<String>,
    /// Arguments after the command.
    pub args: Vec<String>,
    /// Raw message text (may be empty).
    pub text: String,
}

impl BotUpdate {
    /// Build from raw parts — the single command-parsing point; the shells use it
    /// too (the teloxide Update type is not Serialize).
    #[must_use]
    pub fn from_parts(
        update_id: i64,
        chat_id: i64,
        from_id: i64,
        username: Option<&str>,
        text: &str,
    ) -> Self {
        let (command, args) = parse_command(text);
        Self {
            update_id,
            chat_id,
            from_id,
            username: username.filter(|u| !u.is_empty()).map(str::to_owned),
            command,
            args,
            text: text.to_owned(),
        }
    }

    /// Pure parsing — only `message` **with text** (edited messages/captions/callbacks
    /// are ignored for now — additive in later phases). A textless message (photo etc.) →
    /// `None` — fail-closed; the bot never replies to what it does not understand.
    #[must_use]
    pub fn from_json(v: &Value) -> Option<Self> {
        let msg = v.get("message")?;
        let chat_id = msg.get("chat")?.get("id")?.as_i64()?;
        let from = msg.get("from")?;
        let from_id = from.get("id")?.as_i64()?;
        if from_id <= 0 {
            return None; // channel/bot — we do not reply
        }
        let username = from.get("username").and_then(Value::as_str);
        let text = msg.get("text").and_then(Value::as_str)?;
        let update_id = v.get("update_id").and_then(Value::as_i64).unwrap_or(0);
        Some(Self::from_parts(update_id, chat_id, from_id, username, text))
    }

    /// Command with a bot-name prefix (`/start@KiaBot`) → normalized to `start`.
    ///
    /// Matching host → command base; mismatched → `None` (addressed to another bot).
    /// No known bot name → the suffix is optimistically dropped.
    #[must_use]
    pub fn normalized_command(&self, bot_username: Option<&str>) -> Option<String> {
        let raw = self.command.as_deref()?;
        if let Some(bot) = bot_username {
            match raw.split_once('@') {
                Some((base, host)) if host.eq_ignore_ascii_case(bot) => {
                    return Some(base.to_ascii_lowercase());
                }
                Some(_) => return None, // addressed to another bot — we do not reply
                None => {}
            }
        }
        let base = raw.split('@').next().unwrap_or(raw);
        Some(base.to_ascii_lowercase())
    }
}

/// Parse the command from text — `/cmd arg1 arg2` → `("cmd", ["arg1","arg2"])`.
fn parse_command(text: &str) -> (Option<String>, Vec<String>) {
    let mut words = text.split_whitespace();
    let first = words.next().unwrap_or_default();
    if !first.starts_with('/') {
        return (None, Vec::new());
    }
    let cmd = first[1..].to_ascii_lowercase();
    if cmd.is_empty() {
        return (None, Vec::new());
    }
    (Some(cmd), words.map(str::to_owned).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn update(text: &str) -> Value {
        json!({
            "update_id": 42,
            "message": {
                "message_id": 1,
                "text": text,
                "chat": { "id": 100, "type": "private" },
                "from": { "id": 777, "username": "kia", "is_bot": false }
            }
        })
    }

    #[test]
    fn parses_private_text_message_with_command() {
        let u = BotUpdate::from_json(&update("/start hello")).expect("parses");
        assert_eq!(u.update_id, 42);
        assert_eq!(u.chat_id, 100);
        assert_eq!(u.from_id, 777);
        assert_eq!(u.username.as_deref(), Some("kia"));
        assert_eq!(u.command.as_deref(), Some("start"));
        assert_eq!(u.args, vec!["hello"]);
        assert_eq!(u.text, "/start hello");
    }

    #[test]
    fn plain_text_has_no_command() {
        let u = BotUpdate::from_json(&update("\u{633}\u{644}\u{627}\u{645}")).expect("parses");
        assert!(u.command.is_none());
        assert_eq!(u.args, [] as [String; 0]);
    }

    #[test]
    fn bot_suffix_is_normalized() {
        let u = BotUpdate::from_json(&update("/Balance@KiaBot")).expect("parses");
        assert_eq!(u.normalized_command(Some("kiabot")).as_deref(), Some("balance"));
        // host mismatch → addressed to another bot — None
        assert_eq!(u.normalized_command(Some("other")), None);
        // unknown bot name → the suffix is optimistically dropped
        assert_eq!(u.normalized_command(None).as_deref(), Some("balance"));
    }

    #[test]
    fn non_message_and_hostile_shapes_fail_closed() {
        assert!(BotUpdate::from_json(&json!({ "update_id": 1 })).is_none());
        assert!(BotUpdate::from_json(&json!({ "edited_message": { "chat": { "id": 1 } } })).is_none());
        // from without an id or a channel → reject
        let hostile = json!({
            "message": { "chat": { "id": 1 }, "from": { "id": -5 }, "text": "x" }
        });
        assert!(BotUpdate::from_json(&hostile).is_none());
        // textless message (photo etc.) → None — fail-closed, no reply
        let photo = json!({
            "update_id": 3,
            "message": { "chat": { "id": 100 }, "from": { "id": 777 }, "photo": [{}] }
        });
        assert!(BotUpdate::from_json(&photo).is_none(), "non-text must be ignored");
    }
}
