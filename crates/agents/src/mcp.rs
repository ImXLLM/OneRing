//! ابزارهای MCP — allowlist بسته؛ ابزار ناشناخته هرگز invoke نمی‌شود.
//!
//! هر ابزار: نام، توضیح، آرگومان‌های لازم به ترتیب، و `admin_only`. اجرا سمت
//! سرور با کوئری‌های خواندنِ ثبت‌شده انجام می‌شود — MCP هیچ SQL آزادی به
//! فراخواننده نمی‌دهد (اصل sqlsafe لگاسی).

/// تعریف یک ابزار MCP.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct McpToolDef {
    /// نام ابزار (`kernel.economy`)
    pub name: &'static str,
    /// توضیح
    pub description: &'static str,
    /// آرگومان‌های لازم به ترتیب
    pub args: &'static [&'static str],
    /// فقط ادمین؟ (خوانش عمومی = false)
    pub admin_only: bool,
}

/// allowlist ابزارها — ۸ ابزار خواندنی؛ نوشتن روی MCP ممنوع (فقط API کرنل).
pub const TOOLS: [McpToolDef; 8] = [
    McpToolDef {
        name: "kernel.economy",
        description: "خلاصهٔ اقتصاد ملی (مادهٔ ۶) — عرضه، خزانه، قفل امانت",
        args: &[],
        admin_only: false,
    },
    McpToolDef {
        name: "kernel.health",
        description: "liveness عمیق — دفتر پاسخ می‌دهد؟",
        args: &[],
        admin_only: false,
    },
    McpToolDef {
        name: "market.products",
        description: "محصولات فعال بازار",
        args: &["limit"],
        admin_only: false,
    },
    McpToolDef {
        name: "escrow.get",
        description: "وضعیت یک امانت با شناسه",
        args: &["id"],
        admin_only: false,
    },
    McpToolDef {
        name: "kb.search",
        description: "جست‌وجوی پایگاه دانش",
        args: &["q"],
        admin_only: false,
    },
    McpToolDef {
        name: "audit.tail",
        description: "آخرین ردیف‌های ممیزی — فقط ادمین",
        args: &["limit"],
        admin_only: true,
    },
    McpToolDef {
        name: "wallet.balance_of",
        description: "تراز یک حساب دفتر — فقط ادمین",
        args: &["account_id"],
        admin_only: true,
    },
    McpToolDef {
        name: "deposits.list",
        description: "فهرست واریزهای TON با فیلتر وضعیت — فقط ادمین",
        args: &["status"],
        admin_only: true,
    },
];

/// ابزار با نام.
#[must_use]
pub fn tool_def(name: &str) -> Option<&'static McpToolDef> {
    TOOLS.iter().find(|t| t.name == name)
}

/// اعتبارسنجی آرگومان‌ها — همهٔ کلیدهای لازم باید رشتهٔ غیرخالی باشند.
/// خروجی: دلیل نقص یا `None` برای درست.
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

    /// allowlist — ۸ ابزار، نام یکتا، ادمین‌ها دقیقاً مشخص.
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

    /// اعتبارسنجی آرگومان — نقص دقیق گزارش می‌شود.
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
