//! تجزیهٔ خالص آپدیت تلگرام — از `serde_json::Value` به [`BotUpdate`].
//!
//! مسیر واحد تجزیه برای هر دو شِل (teloxide و وبهوک کرنل): شِل teloxide هر
//! `Update` را با `serde_json::to_value` به Value می‌بَرد و همین‌جا تجزیه
//! می‌شود — یک معنا، یک تست. هر چیز غیرپیام/غیرمتنی → `None` (fail-closed؛
//! هرگز crash نمی‌کند).

use serde_json::Value;

/// آپدیت مینیمال تلگرام — فقط چیزی که orchestrator لازم دارد.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BotUpdate {
    /// شناسهٔ آپدیت تلگرام — برای trace/log.
    pub update_id: i64,
    /// شناسهٔ چت مقصد پاسخ (private chat).
    pub chat_id: i64,
    /// شناسهٔ فرستندهٔ انسانی — هویت از کانال معتمد، هرگز از بدنهٔ کاربر.
    pub from_id: i64,
    /// username تلگرام — ممکن است غایب باشد (Option صریح).
    pub username: Option<String>,
    /// فرمان بدون `/` و بدون پسوند `@BotBot` — حروف کوچک.
    pub command: Option<String>,
    /// آرگومان‌های بعد از فرمان.
    pub args: Vec<String>,
    /// متن خام پیام (می‌تواند خالی باشد).
    pub text: String,
}

impl BotUpdate {
    /// ساخت از اجزای خام — نقطهٔ واحد تجزیهٔ فرمان؛ از آن شِل‌ها هم استفاده
    /// می‌کنند (teloxide Update نوع Serialize ندارد).
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

    /// تجزیهٔ خالص — فقط `message` **با متن** (پیام ویرایش‌شده/کپشن/کال‌بک
    /// فعلاً نادیده — additive در فازهای بعد). پیام بدون متن (عکس و…) →
    /// `None` — fail-closed؛ بات به چیزی که نمی‌فهمد پاسخ نمی‌دهد.
    #[must_use]
    pub fn from_json(v: &Value) -> Option<Self> {
        let msg = v.get("message")?;
        let chat_id = msg.get("chat")?.get("id")?.as_i64()?;
        let from = msg.get("from")?;
        let from_id = from.get("id")?.as_i64()?;
        if from_id <= 0 {
            return None; // کانال/بات — پاسخ نمی‌دهیم
        }
        let username = from.get("username").and_then(Value::as_str);
        let text = msg.get("text").and_then(Value::as_str)?;
        let update_id = v.get("update_id").and_then(Value::as_i64).unwrap_or(0);
        Some(Self::from_parts(update_id, chat_id, from_id, username, text))
    }

    /// فرمان با پیشوند نام بات (`/start@KiaBot`) → نرمال به `start`.
    ///
    /// هاست همسان → پایهٔ فرمان؛ ناهمسان → `None` (خطاب به بات دیگر است).
    /// بدون نام باتِ شناخته‌شده → پسوند به‌صورت خوش‌بینانه حذف می‌شود.
    #[must_use]
    pub fn normalized_command(&self, bot_username: Option<&str>) -> Option<String> {
        let raw = self.command.as_deref()?;
        if let Some(bot) = bot_username {
            match raw.split_once('@') {
                Some((base, host)) if host.eq_ignore_ascii_case(bot) => {
                    return Some(base.to_ascii_lowercase());
                }
                Some(_) => return None, // خطاب به بات دیگر — پاسخ نمی‌دهیم
                None => {}
            }
        }
        let base = raw.split('@').next().unwrap_or(raw);
        Some(base.to_ascii_lowercase())
    }
}

/// تجزیهٔ فرمان از متن — `/cmd arg1 arg2` → `("cmd", ["arg1","arg2"])`.
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
        let u = BotUpdate::from_json(&update("سلام")).expect("parses");
        assert!(u.command.is_none());
        assert_eq!(u.args, [] as [String; 0]);
    }

    #[test]
    fn bot_suffix_is_normalized() {
        let u = BotUpdate::from_json(&update("/Balance@KiaBot")).expect("parses");
        assert_eq!(u.normalized_command(Some("kiabot")).as_deref(), Some("balance"));
        // هاست ناهمسان → خطاب به بات دیگر است — None
        assert_eq!(u.normalized_command(Some("other")), None);
        // نام بات ناشناخته → پسوند خوش‌بینانه حذف می‌شود
        assert_eq!(u.normalized_command(None).as_deref(), Some("balance"));
    }

    #[test]
    fn non_message_and_hostile_shapes_fail_closed() {
        assert!(BotUpdate::from_json(&json!({ "update_id": 1 })).is_none());
        assert!(BotUpdate::from_json(&json!({ "edited_message": { "chat": { "id": 1 } } })).is_none());
        // from بی‌شناسه یا کانال → رد
        let hostile = json!({
            "message": { "chat": { "id": 1 }, "from": { "id": -5 }, "text": "x" }
        });
        assert!(BotUpdate::from_json(&hostile).is_none());
        // پیام بدون متن (عکس و…) → None — fail-closed، بدون پاسخ
        let photo = json!({
            "update_id": 3,
            "message": { "chat": { "id": 100 }, "from": { "id": 777 }, "photo": [{}] }
        });
        assert!(BotUpdate::from_json(&photo).is_none(), "non-text must be ignored");
    }
}
