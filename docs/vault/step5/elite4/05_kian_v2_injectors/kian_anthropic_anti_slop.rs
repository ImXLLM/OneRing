//! KIAN v2 Anthropic Anti-Slop & Web-Artifacts Validator (`crates/hermes`)
//!
//! Distilled from:
//! - `anthropics/skills/skills/frontend-design/SKILL.md`
//! - `anthropics/skills/skills/web-artifacts-builder/SKILL.md`
//! - `msitarzewski/agency-agents/design/design-ui-finish-gate-reviewer.md`

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AntiSlopSeverity {
    Warn,
    FailGate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AntiSlopFinding {
    pub code: &'static str,
    pub severity: AntiSlopSeverity,
    pub message: &'static str,
}

/// System prompt directive enforcing Anthropic's `frontend-design` + `web-artifacts-builder` standards.
pub const ANTHROPIC_ANTI_SLOP_DIRECTIVE: &str = "\
FRONTEND DESIGN & ARTIFACT STANDARDS (Anthropic Official):
1. Two-Pass Design Plan: First define 4-6 hex tokens, intentional font pairing (no default Inter-only SaaS kit), ASCII layout wireframe, and differentiating principles. Reject generic defaults before coding.
2. Anti-AI-Slop Palette & Chrome:
   - Never default to warm-cream (#F4F1EA) + serif + terracotta (#D97757) unless explicitly requested.
   - Never default to generic purple gradients, uniform rounded-card SaaS grids with identical rgba(0,0,0,.1) shadows, or near-black + neon acid-green clichés.
   - Avoid ALL-CAPS eyebrow labels above every heading, middle-dot meta strings ('A · B · C'), '01 / 02 / 03' markers on non-sequential content, and trailing '→' on every button/link.
3. Motion & Typography: Keep body line length < 80ch. Use one orchestrated load moment or purposeful interactive feedback instead of scattered section fade-ups.
4. Copy & Self-Contained Bundle: Active voice CTAs ('Save changes', not 'Submit'). All styles/scripts self-contained with visible keyboard focus and reduced-motion support.";

/// Inspects generated HTML/CSS/TSX artifact source for Anthropic anti-slop violations and finish-gate defects.
pub fn validate_artifact_html(html: &str) -> Vec<AntiSlopFinding> {
    let lower = html.to_ascii_lowercase();
    let mut findings = Vec::new();

    // 1. Anthropic Claude-tell palette (#f4f1ea + #d97757)
    if lower.contains("#f4f1ea") && lower.contains("#d97757") {
        findings.push(AntiSlopFinding {
            code: "SLOP_CLAUDE_TELL_PALETTE",
            severity: AntiSlopSeverity::FailGate,
            message: "Detected default Claude-interaction palette (#F4F1EA + #D97757); choose a brief-specific palette.",
        });
    }

    // 2. Generic Inter-only + purple gradient SaaS-kit tell
    let has_inter = lower.contains("font-family: 'inter'") || lower.contains("font-family: \"inter\"") || lower.contains("font-sans");
    let has_purple_grad = lower.contains("from-purple-") || lower.contains("to-indigo-") || lower.contains("#8b5cf6");
    if has_inter && has_purple_grad {
        findings.push(AntiSlopFinding {
            code: "SLOP_PURPLE_INTER_KIT",
            severity: AntiSlopSeverity::FailGate,
            message: "Detected generic Inter + purple/indigo gradient SaaS default (`web-artifacts-builder` anti-slop rule).",
        });
    }

    // 3. Overused template chrome (' · ' middle dots or excessive '→' arrows)
    let middle_dot_count = html.matches(" · ").count();
    let arrow_count = html.matches("→").count();
    if middle_dot_count >= 4 || arrow_count >= 5 {
        findings.push(AntiSlopFinding {
            code: "SLOP_TEMPLATE_CHROME",
            severity: AntiSlopSeverity::Warn,
            message: "Excessive middle-dot metadata strings (' · ') or trailing '→' button arrows detected.",
        });
    }

    // 4. Accessibility & self-contained artifact baseline
    if !lower.contains("prefers-reduced-motion") && (lower.contains("@keyframes") || lower.contains("transition:")) {
        findings.push(AntiSlopFinding {
            code: "A11Y_REDUCED_MOTION",
            severity: AntiSlopSeverity::Warn,
            message: "Animations detected without `@media (prefers-reduced-motion: reduce)` safeguard.",
        });
    }

    // 5. External CDN reliance warning for sandboxed single-file HTML artifacts
    if lower.contains("https://cdn.") || lower.contains("https://unpkg.com") {
        findings.push(AntiSlopFinding {
            code: "ARTIFACT_EXTERNAL_CDN",
            severity: AntiSlopSeverity::Warn,
            message: "External CDN link found; bundle assets inline for offline/sandboxed artifact reliability.",
        });
    }

    findings
}

pub fn passes_finish_gate(html: &str) -> bool {
    !validate_artifact_html(html)
        .iter()
        .any(|f| f.severity == AntiSlopSeverity::FailGate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_claude_tell_and_purple_inter_slop() {
        let bad = r#"<style>body{background:#F4F1EA;color:#D97757;font-family:'Inter';}</style><div class="from-purple-500"></div>"#;
        let findings = validate_artifact_html(bad);
        assert!(!passes_finish_gate(bad));
        assert!(findings.iter().any(|f| f.code == "SLOP_CLAUDE_TELL_PALETTE"));
        assert!(findings.iter().any(|f| f.code == "SLOP_PURPLE_INTER_KIT"));
    }

    #[test]
    fn passes_intentional_custom_design() {
        let good = r#"<!doctype html><html><head><style>:root{--bg:#0E1715;--ink:#EBF2EE;--accent:#E09F3E;}@media(prefers-reduced-motion:reduce){*{animation:none!important;}}</style></head><body><button>Publish</button></body></html>"#;
        assert!(passes_finish_gate(good));
        assert!(validate_artifact_html(good).is_empty());
    }
}
