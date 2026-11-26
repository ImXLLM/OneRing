//! MCP tools — a closed allowlist; an unknown tool is never invoked.
//!
//! Each tool: name, description, required arguments in order, and `admin_only`. Execution happens
//! server-side with registered read queries — MCP grants no free-form SQL
//! to the caller (the legacy sqlsafe principle).

/// Definition of one MCP tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct McpToolDef {
    /// Tool name (`kernel.economy`)
    pub name: &'static str,
    /// Description
    pub description: &'static str,
    /// Required arguments, in order
    pub args: &'static [&'static str],
    /// Admin only? (public read = false)
    pub admin_only: bool,
}

/// Tool allowlist — 8 readable tools; writes over MCP are forbidden (kernel API only).
pub const TOOLS: [McpToolDef; 8] = [
    McpToolDef {
        name: "kernel.economy",
        description: "\u{62e}\u{644}\u{627}\u{635}\u{647}\u{654} \u{627}\u{642}\u{62a}\u{635}\u{627}\u{62f} \u{645}\u{644}\u{6cc} (\u{645}\u{627}\u{62f}\u{647}\u{654} \u{6f6}) — \u{639}\u{631}\u{636}\u{647}\u{60c} \u{62e}\u{632}\u{627}\u{646}\u{647}\u{60c} \u{642}\u{641}\u{644} \u{627}\u{645}\u{627}\u{646}\u{62a}",
        args: &[],
        admin_only: false,
    },
    McpToolDef {
        name: "kernel.health",
        description: "liveness \u{639}\u{645}\u{6cc}\u{642} — \u{62f}\u{641}\u{62a}\u{631} \u{67e}\u{627}\u{633}\u{62e} \u{645}\u{6cc}‌\u{62f}\u{647}\u{62f}\u{61f}",
        args: &[],
        admin_only: false,
    },
    McpToolDef {
        name: "market.products",
        description: "\u{645}\u{62d}\u{635}\u{648}\u{644}\u{627}\u{62a} \u{641}\u{639}\u{627}\u{644} \u{628}\u{627}\u{632}\u{627}\u{631}",
        args: &["limit"],
        admin_only: false,
    },
    McpToolDef {
        name: "escrow.get",
        description: "\u{648}\u{636}\u{639}\u{6cc}\u{62a} \u{6cc}\u{6a9} \u{627}\u{645}\u{627}\u{646}\u{62a} \u{628}\u{627} \u{634}\u{646}\u{627}\u{633}\u{647}",
        args: &["id"],
        admin_only: false,
    },
    McpToolDef {
        name: "kb.search",
        description: "\u{62c}\u{633}\u{62a}‌\u{648}\u{62c}\u{648}\u{6cc} \u{67e}\u{627}\u{6cc}\u{6af}\u{627}\u{647} \u{62f}\u{627}\u{646}\u{634}",
        args: &["q"],
        admin_only: false,
    },
    McpToolDef {
        name: "audit.tail",
        description: "\u{622}\u{62e}\u{631}\u{6cc}\u{646} \u{631}\u{62f}\u{6cc}\u{641}‌\u{647}\u{627}\u{6cc} \u{645}\u{645}\u{6cc}\u{632}\u{6cc} — \u{641}\u{642}\u{637} \u{627}\u{62f}\u{645}\u{6cc}\u{646}",
        args: &["limit"],
        admin_only: true,
    },
    McpToolDef {
        name: "wallet.balance_of",
        description: "\u{62a}\u{631}\u{627}\u{632} \u{6cc}\u{6a9} \u{62d}\u{633}\u{627}\u{628} \u{62f}\u{641}\u{62a}\u{631} — \u{641}\u{642}\u{637} \u{627}\u{62f}\u{645}\u{6cc}\u{646}",
        args: &["account_id"],
        admin_only: true,
    },
    McpToolDef {
        name: "deposits.list",
        description: "\u{641}\u{647}\u{631}\u{633}\u{62a} \u{648}\u{627}\u{631}\u{6cc}\u{632}\u{647}\u{627}\u{6cc} TON \u{628}\u{627} \u{641}\u{6cc}\u{644}\u{62a}\u{631} \u{648}\u{636}\u{639}\u{6cc}\u{62a} — \u{641}\u{642}\u{637} \u{627}\u{62f}\u{645}\u{6cc}\u{646}",
        args: &["status"],
        admin_only: true,
    },
];

/// Tool by name.
#[must_use]
pub fn tool_def(name: &str) -> Option<&'static McpToolDef> {
    TOOLS.iter().find(|t| t.name == name)
}

/// Argument validation — every required key must be a non-empty string.
/// Output: the defect reason or `None` when valid.
#[must_use]
pub fn validate_args(tool: &McpToolDef, arguments: &serde_json::Value) -> Option<String> {
    let Some(obj) = arguments.as_object() else {
        return Some(format!("arguments must be a JSON object for `{}`", tool.name));
    };
    for arg in tool.args {
        match obj.get(*arg) {
            Some(serde_json::Value::String(s)) if !s.trim().is_empty() => {}
            Some(serde_json::Value::Number(n)) if n.as_i64().unwrap_or(0) > 0 => {}
            _ => return Some(format!("missing or empty argument `{arg}`")),
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Allowlist — 8 tools, unique names, admins exactly specified.
    #[test]
    fn tools_are_eight_unique_with_admin_flags() {
        assert_eq!(TOOLS.len(), 8);
        let mut names: Vec<&str> = TOOLS.iter().map(|t| t.name).collect();
        names.sort_unstable();
        assert!(names.windows(2).all(|w| w[0] != w[1]));
        let admin_count = TOOLS.iter().filter(|t| t.admin_only).count();
        assert_eq!(admin_count, 3, "audit.tail + wallet.balance_of + deposits.list");
        assert!(tool_def("kernel.economy").is_some());
        assert!(tool_def("drop tables").is_none(), "no free-form SQL tool");
    }

    /// Argument validation — the exact defect is reported.
    #[test]
    fn args_validation_reports_exact_missing() {
        let t = tool_def("escrow.get").expect("tool");
        assert_eq!(
            validate_args(t, &serde_json::json!({})),
            Some("missing or empty argument `id`".into())
        );
        assert_eq!(validate_args(t, &serde_json::json!({"id":"esc:1"})), None);
        assert!(validate_args(t, &serde_json::json!("string")).is_some());
        let limit_tool = tool_def("market.products").expect("tool");
        assert_eq!(validate_args(limit_tool, &serde_json::json!({"limit": 10})), None);
    }
}
