//! Kian studio design contract — port of `ui-ux-pro-max-skill` v2.15 +
//! `awesome-gpt-6-astra` (`website/design-qa.md`) + `androoAGI/starnet`
//! (`sidecar/compaction-fidelity.js`).
//!
//! three deterministic, tested pieces:
//!
//! 1. [`DesignDials`] — the three design dials `--variance` / `--motion` / `--density`
//!    from UI/UX Pro Max (range 1..=10 with automatic clamping) + automatic RTL detection
//!    for Persian and the `Vazirmatn` font stack.
//! 2. [`AstraQualityGate`] — Google's "first frame interactive" rule, adapted from Astra:
//!    canvas/form on the first frame (no dead marketing splash), a complete single-file
//!    document without broken local imports, animation stops on `visibilitychange`,
//!    and zero horizontal overflow at 390×844 mobile.
//! 3. [`build_verbatim_user_messages_block`] — a 5% window budget (floor 2,000 /
//!    cap 24,000 characters, the Hermes cap) with a fair water-fill share; compression
//!    never loses the user's original specifications.
#![forbid(unsafe_code)]

// ── 1) UI/UX Pro Max v2.15 design dials ───────────────────────────────────

/// The three design dials — exactly `--variance <1-10> --motion <1-10> --density <1-10>`
/// from `templates/base/skill-content.md` (line 167).
///
/// * variance: centered/minimal (1) ← balanced/modern (5) ← bold/asymmetric (10)
/// * motion:  subtle micro-interactions (1) ← standard scroll (5) ← intricate choreography (10)
/// * density: spacious 24-96px (1) ← standard 16-64px (5) ← dashboard 8-32px (10)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DesignDials {
    /// Visual variance (1..=10).
    pub variance: u8,
    /// Motion intensity (1..=10).
    pub motion: u8,
    /// Information density (1..=10).
    pub density: u8,
}

impl DesignDials {
    /// Lower bound of each dial's range.
    pub const MIN: u8 = 1;
    /// Upper bound of each dial's range.
    pub const MAX: u8 = 10;

    /// Built with automatic clamping to the 1..=10 range — raw input never escapes.
    #[must_use]
    pub fn new(variance: u8, motion: u8, density: u8) -> Self {
        Self {
            variance: variance.clamp(Self::MIN, Self::MAX),
            motion: motion.clamp(Self::MIN, Self::MAX),
            density: density.clamp(Self::MIN, Self::MAX),
        }
    }

    /// Variance layer — the mapping at line 172 of `skill-content.md`.
    #[must_use]
    pub fn variance_tier(&self) -> &'static str {
        match self.variance {
            1..=3 => "centered/minimal",
            4..=7 => "balanced/modern",
            _ => "bold/asymmetric",
        }
    }

    /// Motion layer — the mapping at line 173 of `skill-content.md`.
    #[must_use]
    pub fn motion_tier(&self) -> &'static str {
        match self.motion {
            1..=3 => "subtle",
            4..=7 => "standard",
            _ => "complex",
        }
    }

    /// Density layer — the mapping at line 174 of `skill-content.md`.
    #[must_use]
    pub fn density_tier(&self) -> &'static str {
        match self.density {
            1..=3 => "spacious",
            4..=7 => "standard",
            _ => "dense/dashboard",
        }
    }

    /// RTL font stack recommendation — line 9 of the Kian contract (`Vazirmatn` for Persian/Arabic).
    #[must_use]
    pub fn font_stack(&self, rtl: bool) -> &'static str {
        if rtl {
            RTL_FONT_STACK
        } else {
            LTR_FONT_STACK
        }
    }
}

/// Locked Persian RTL font stack (`dir="rtl"` + Vazirmatn).
pub const RTL_FONT_STACK: &str = "Vazirmatn, 'Noto Sans Arabic', system-ui, sans-serif";

/// Default LTR font stack (tech/data).
pub const LTR_FONT_STACK: &str = "'Space Grotesk', 'DM Sans', system-ui, sans-serif";

/// `<html>` attribute block for a Persian document — the line-9 contract requirement.
pub const RTL_HTML_ATTRS: &str = "dir=\"rtl\" lang=\"fa\"";

/// Is the prompt Persian/Arabic? — the presence of any character in the
/// `U+0600..=U+06FF` block (base Arabic/Persian) means the document must get `dir="rtl"` and
/// the [`RTL_FONT_STACK`] font stack.
#[must_use]
pub fn detect_persian_rtl(prompt: &str) -> bool {
    prompt
        .chars()
        .any(|c| ('\u{0600}'..='\u{06FF}').contains(&c))
}

/// Deterministic injection of `dir="rtl" lang="fa"` onto the generated document's `<html>` tag.
///
/// If the document already has `dir=`, it is returned untouched (double
/// marking is forbidden); if it has no `<html>` tag, the document is returned untouched (we only
/// recognize complete documents).
#[must_use]
pub fn enforce_rtl(html: &str) -> String {
    let lower = html.to_lowercase();
    if lower.contains("dir=") || !lower.contains("<html") {
        return html.to_owned();
    }
    // Find the first <html …> tag and inject the attributes into it.
    match html.find('<') {
        Some(start) if html[start..].to_lowercase().starts_with("<html") => {
            let tag_end = html[start..].find('>').map_or(start, |i| start + i);
            let (tag, rest) = html.split_at(tag_end);
            format!("{tag} {RTL_HTML_ATTRS}{rest}")
        }
        _ => html.to_owned(),
    }
}

// ── 2) Astra quality gate (first interactive frame) ────────────────────────────────

/// The [`AstraQualityGate::audit`] static audit report — the five boolean flags are deliberate:
/// each flag is an independent Astra gate rule and must not be collapsed into another enum/field.
#[allow(clippy::struct_excessive_bools)] // Audit report — each boolean is an independent rule
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AstraAudit {
    /// Interactive surface on the first frame (canvas/input/button/…) with no dead splash.
    pub first_frame_interactive: bool,
    /// Single-file document: doctype + complete closing + zero broken local imports.
    pub self_contained_single_file: bool,
    /// Animation/timer stops on `visibilitychange`.
    pub pauses_on_visibility_change: bool,
    /// Mobile viewport meta tag with `width=device-width`.
    pub mobile_viewport_meta: bool,
    /// Static zero horizontal overflow: no fixed pixel width ≥ 390px.
    pub mobile_zero_overflow_static: bool,
}

impl AstraAudit {
    /// List of issues in Persian — for direct feedback to the studio LLM.
    #[must_use]
    pub fn failures(&self) -> Vec<&'static str> {
        let mut out = Vec::new();
        if !self.first_frame_interactive {
            out.push("\u{641}\u{631}\u{6cc}\u{645} \u{627}\u{648}\u{644} \u{62a}\u{639}\u{627}\u{645}\u{644}\u{6cc} \u{646}\u{6cc}\u{633}\u{62a} — \u{633}\u{637}\u{62d} \u{6a9}\u{627}\u{631}/canvas/\u{641}\u{631}\u{645} \u{628}\u{627}\u{6cc}\u{62f} \u{628}\u{6cc}‌\u{62f}\u{631}\u{646}\u{6af} \u{62f}\u{631} viewport \u{628}\u{627}\u{634}\u{62f}\u{61b} \u{647}\u{6cc}\u{686} \u{627}\u{633}\u{67e}\u{644}\u{634} \u{628}\u{627}\u{632}\u{627}\u{631}\u{6cc}\u{627}\u{628}\u{6cc} \u{645}\u{631}\u{62f}\u{647}‌\u{627}\u{6cc} \u{642}\u{628}\u{644} \u{627}\u{632} \u{627}\u{67e} \u{645}\u{62c}\u{627}\u{632} \u{646}\u{6cc}\u{633}\u{62a}.");
        }
        if !self.self_contained_single_file {
            out.push("\u{633}\u{646}\u{62f} \u{62a}\u{6a9}‌\u{641}\u{627}\u{6cc}\u{644}\u{6cc} \u{6a9}\u{627}\u{645}\u{644} \u{646}\u{6cc}\u{633}\u{62a} — <!DOCTYPE html> \u{648} </html> \u{644}\u{627}\u{632}\u{645} \u{627}\u{633}\u{62a} \u{648} \u{647}\u{6cc}\u{686} \u{627}\u{6cc}\u{645}\u{67e}\u{648}\u{631}\u{62a} \u{645}\u{62d}\u{644}\u{6cc} (src/href \u{646}\u{633}\u{628}\u{6cc}) \u{634}\u{6a9}\u{633}\u{62a}\u{647}‌\u{627}\u{6cc} \u{645}\u{62c}\u{627}\u{632} \u{646}\u{6cc}\u{633}\u{62a}.");
        }
        if !self.pauses_on_visibility_change {
            out.push("\u{627}\u{646}\u{6cc}\u{645}\u{6cc}\u{634}\u{646}/\u{62a}\u{627}\u{6cc}\u{645}\u{631} \u{631}\u{648}\u{6cc} visibilitychange \u{645}\u{62a}\u{648}\u{642}\u{641} \u{646}\u{645}\u{6cc}‌\u{634}\u{648}\u{62f}.");
        }
        if !self.mobile_viewport_meta {
            out.push("\u{645}\u{62a}\u{627}‌\u{62a}\u{6af} viewport \u{628}\u{627} width=device-width \u{67e}\u{6cc}\u{62f}\u{627} \u{646}\u{634}\u{62f}.");
        }
        if !self.mobile_zero_overflow_static {
            out.push("\u{639}\u{631}\u{636} \u{67e}\u{6cc}\u{6a9}\u{633}\u{644}\u{6cc} \u{62b}\u{627}\u{628}\u{62a} ≥ \u{6f3}\u{6f9}\u{6f0}px \u{6cc}\u{627}\u{641}\u{62a} \u{634}\u{62f} — \u{62f}\u{631} \u{645}\u{648}\u{628}\u{627}\u{6cc}\u{644} \u{6f3}\u{6f9}\u{6f0}×\u{6f8}\u{6f4}\u{6f4} \u{633}\u{631}\u{631}\u{6cc}\u{632} \u{627}\u{641}\u{642}\u{6cc} \u{645}\u{6cc}‌\u{633}\u{627}\u{632}\u{62f}.");
        }
        out
    }

    /// Full pass through the Astra gate?
    #[must_use]
    pub fn passed(&self) -> bool {
        self.first_frame_interactive
            && self.self_contained_single_file
            && self.pauses_on_visibility_change
            && self.mobile_viewport_meta
            && self.mobile_zero_overflow_static
    }
}

/// Studio quality gate — Google-derived rules from Astra's `design-qa.md`.
#[derive(Debug, Clone, Copy, Default)]
pub struct AstraQualityGate {
    _private: (),
}

impl AstraQualityGate {
    /// Reference mobile width (iPhone 14 Pro Max ≈ 390).
    pub const MOBILE_WIDTH: u32 = 390;
    /// Reference mobile height.
    pub const MOBILE_HEIGHT: u32 = 844;
    /// Any fixed pixel width above this value creates horizontal overflow at 390px
    /// (exactly 390 is by definition not overflow — `max-width:390px` is the standard pattern).
    pub const OVERFLOW_WIDTH_PX: u32 = 390;

    /// Static audit of the generated HTML document — deterministic and network-free.
    ///
    /// * First-frame interactivity: presence of `canvas`/`input`/`button`/`textarea`/
    ///   `select`/`contenteditable` **and** the absence of a splash gate
    ///   (`id="splash"`/`class="splash…"`).
    /// * Single-file: `<!doctype html` + `</html>` + no relative `src=`/`href=`
    ///   (only `https:`/`data:`/`blob:`/`#` are allowed).
    /// * Static overflow: scanning `width:`/`min-width:`/`max-width:` and
    ///   `width="…"` for a pixel value > [`Self::OVERFLOW_WIDTH_PX`].
    #[must_use]
    pub fn audit(&self, html: &str) -> AstraAudit {
        let lower = html.to_lowercase();

        let has_surface = ["<canvas", "<input", "<button", "<textarea", "<select", "contenteditable"]
            .iter()
            .any(|m| lower.contains(m));
        let splash_gate = ["id=\"splash", "id='splash", "class=\"splash", "class='splash"]
            .iter()
            .any(|m| lower.contains(m));

        let doctype = lower.starts_with("<!doctype html") || lower.starts_with("<!doctype html ");
        let closed = lower.contains("</html>");
        let no_local_imports = !scan_local_imports(&lower);

        let viewport = lower.contains("<meta name=\"viewport\"")
            || lower.contains("<meta name='viewport'")
            || lower.contains("<meta  name=\"viewport\"");
        let device_width = lower.contains("width=device-width");

        AstraAudit {
            first_frame_interactive: has_surface && !splash_gate,
            self_contained_single_file: doctype && closed && no_local_imports,
            pauses_on_visibility_change: lower.contains("visibilitychange"),
            mobile_viewport_meta: viewport && device_width,
            mobile_zero_overflow_static: !scan_fixed_width_overflow(&lower),
        }
    }
}

/// Broken local import scan — any `src=`/`href=` with a relative value (not
/// `http`/`https`/`data:`/`blob:`/`#`/`mailto:`) means the document depends on a side file
/// and does not leave the studio intact.
fn scan_local_imports(lower_html: &str) -> bool {
    for attr in ["src=", "href="] {
        let mut from = 0;
        while let Some((next, value)) = find_attr_value(lower_html, attr, from) {
            from = next;
            let v = value.trim_start();
            let external = v.starts_with("http://")
                || v.starts_with("https://")
                || v.starts_with("data:")
                || v.starts_with("blob:")
                || v.starts_with('#')
                || v.starts_with("mailto:");
            if !external && !v.is_empty() {
                return true;
            }
        }
    }
    false
}

/// The next attribute value for `attr` from point `from` — returns (resume scan point, value).
fn find_attr_value<'a>(lower_html: &'a str, attr: &str, from: usize) -> Option<(usize, &'a str)> {
    let pos = lower_html[from..].find(attr)? + from;
    let after = pos + attr.len();
    let quote = lower_html.as_bytes().get(after).copied()?;
    if quote != b'"' && quote != b'\'' {
        return None; // No quotes → strict holding-back is safe
    }
    let close = char::from(quote);
    let start = after + 1;
    let end = lower_html[start..].find(close)? + start;
    Some((end + 1, &lower_html[start..end]))
}

/// Scan for fixed widths > 390px in CSS and `width="…"` attributes.
fn scan_fixed_width_overflow(lower_html: &str) -> bool {
    // CSS: width:/min-width:/max-width: <digits>px
    for prop in ["width:", "min-width:", "max-width:"] {
        let mut from = 0;
        while let Some(pos) = lower_html[from..].find(prop) {
            let abs = from + pos + prop.len();
            let tail = &lower_html[abs..];
            let digits: String = tail
                .trim_start()
                .chars()
                .take_while(char::is_ascii_digit)
                .collect();
            if let Ok(px) = digits.parse::<u32>() {
                // The leading offset trimmed from the tail — not the length of the cut section!
                let trimmed = tail.trim_start();
                let leading_ws = tail.len() - trimmed.len();
                let after_digits = abs + leading_ws + digits.len();
                let unit = lower_html[after_digits.min(lower_html.len())..]
                    .trim_start()
                    .to_lowercase();
                let is_px = unit.starts_with("px");
                let is_vw = unit.starts_with("vw");
                if is_px && px > AstraQualityGate::OVERFLOW_WIDTH_PX {
                    return true;
                }
                // vw > 100% also creates horizontal overflow (e.g. width: 120vw).
                if is_vw && px > 100 {
                    return true;
                }
            }
            from = abs;
        }
    }
    // HTML attribute: width="450" / width=450
    let attr = "width=\"";
    let mut from = 0;
    while let Some(pos) = lower_html[from..].find(attr) {
        let abs = from + pos + attr.len();
        let digits: String = lower_html[abs..]
            .chars()
            .take_while(char::is_ascii_digit)
            .collect();
        if digits
            .parse::<u32>()
            .is_ok_and(|px| px > AstraQualityGate::OVERFLOW_WIDTH_PX)
        {
            return true;
        }
        from = abs;
    }
    false
}

// ── 3) The starnet verbatim block (faithful compression) ─────────────────────────────

/// Verbatim messages block open tag — identical to `compaction-fidelity.js` line 30.
pub const VERBATIM_OPEN: &str = "<user_messages_verbatim>";
/// Verbatim messages block close tag.
pub const VERBATIM_CLOSE: &str = "</user_messages_verbatim>";

/// Verbatim block budget: ~5% of the window, floor 2,000 and cap 24,000 characters
/// (the Hermes cap — line 40 of `compaction-fidelity.js`).
#[must_use]
pub fn verbatim_budget_chars(window_chars: usize) -> usize {
    (window_chars / 20).clamp(2_000, 24_000)
}

/// The `<user_messages_verbatim>` block with a fair water-fill share.
///
/// The starnet rule: "short messages complete, long messages cut to their share and marked" —
/// budget left over from short messages is redistributed to hungry messages (two
/// water-fill passes). The user's original specifications are never lost in compression.
#[must_use]
pub fn build_verbatim_user_messages_block(messages: &[&str], window_chars: usize) -> String {
    use std::fmt::Write as _;

    if messages.is_empty() {
        return String::new();
    }
    let budget = verbatim_budget_chars(window_chars);
    let n = messages.len();

    let wants: Vec<usize> = messages.iter().map(|m| m.chars().count()).collect();
    let share = budget / n;
    let mut given: Vec<usize> = wants.iter().map(|w| (*w).min(share)).collect();

    // Redistribute the leftover share of complete messages to truncated messages.
    let mut leftover = budget.saturating_sub(given.iter().sum::<usize>());
    let hungry: Vec<usize> = (0..n)
        .filter(|&i| given[i] < wants[i])
        .collect();
    if !hungry.is_empty() {
        let extra = leftover / hungry.len();
        for &i in &hungry {
            let take = (wants[i] - given[i]).min(extra);
            given[i] += take;
            leftover -= take;
        }
        // Second pass: any remaining leftover goes to the first hungry ones.
        for &i in &hungry {
            if leftover == 0 {
                break;
            }
            let take = (wants[i] - given[i]).min(leftover);
            given[i] += take;
            leftover -= take;
        }
    }

    let mut out = String::from(VERBATIM_OPEN);
    out.push('\n');
    for (i, msg) in messages.iter().enumerate() {
        let kept: String = msg.chars().take(given[i]).collect();
        let _ = writeln!(out, "[#{} | {} chars]", i + 1, wants[i]);
        out.push_str(&kept);
        if given[i] < wants[i] {
            let _ = writeln!(out, "\n[+{} chars trimmed]", wants[i] - given[i]);
        } else {
            out.push('\n');
        }
    }
    out.push_str(VERBATIM_CLOSE);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── dials ──

    #[test]
    fn dials_clamp_to_1_10() {
        let d = DesignDials::new(0, 11, 255);
        assert_eq!(d.variance, 1);
        assert_eq!(d.motion, 10);
        assert_eq!(d.density, 10);
        let d = DesignDials::new(3, 7, 5);
        assert_eq!((d.variance, d.motion, d.density), (3, 7, 5));
    }

    #[test]
    fn dial_tiers_match_skill_content_md() {
        let lo = DesignDials::new(1, 2, 3);
        assert_eq!(lo.variance_tier(), "centered/minimal");
        assert_eq!(lo.motion_tier(), "subtle");
        assert_eq!(lo.density_tier(), "spacious");
        let mid = DesignDials::new(5, 5, 5);
        assert_eq!(mid.variance_tier(), "balanced/modern");
        assert_eq!(mid.motion_tier(), "standard");
        assert_eq!(mid.density_tier(), "standard");
        let hi = DesignDials::new(8, 9, 10);
        assert_eq!(hi.variance_tier(), "bold/asymmetric");
        assert_eq!(hi.motion_tier(), "complex");
        assert_eq!(hi.density_tier(), "dense/dashboard");
        // RTL font only for Persian — line 9 of the contract.
        assert!(hi.font_stack(true).starts_with("Vazirmatn"));
        assert!(hi.font_stack(false).starts_with("'Space Grotesk'"));
    }

    // ── RTL detection ──

    #[test]
    fn persian_rtl_detection_boundaries() {
        assert!(detect_persian_rtl("\u{633}\u{644}\u{627}\u{645} \u{62f}\u{646}\u{6cc}\u{627}"));
        assert!(detect_persian_rtl("\u{645}\u{627}\u{634}\u{6cc}\u{646}‌\u{62d}\u{633}\u{627}\u{628} \u{631}\u{6cc}\u{633}\u{6a9} \u{62a}\u{631}\u{6cc}\u{62f} \u{628}\u{627} \u{62f}\u{627}\u{631}\u{6a9} \u{646}\u{626}\u{648}\u{646}"));
        assert!(!detect_persian_rtl("hello world dashboard"));
        assert!(!detect_persian_rtl(""));
        // Lower boundary of the U+0600 block (ARABIC NUMBER SIGN)
        assert!(detect_persian_rtl("\u{0600}"));
        // Upper boundary of the U+06FF block
        assert!(detect_persian_rtl("\u{06FF}"));
        // Outside the block — NFKC is not the criterion either, only this range
        assert!(!detect_persian_rtl("\u{0750}"));
        // Mixed: a single Persian character inside English is enough
        assert!(detect_persian_rtl("dark neon dashboard \u{641}\u{627}\u{631}\u{633}\u{6cc}"));
    }

    #[test]
    fn enforce_rtl_injects_once_and_never_twice() {
        let doc = "<html><head><title>x</title></head><body>\u{633}\u{644}\u{627}\u{645}</body></html>";
        let enforced = enforce_rtl(doc);
        assert!(enforced.starts_with("<html dir=\"rtl\" lang=\"fa\">"));
        assert_eq!(enforced.matches("dir=\"rtl\"").count(), 1);

        // Already RTL — left untouched.
        let already = "<html dir=\"rtl\" lang=\"fa\"><body>\u{633}\u{644}\u{627}\u{645}</body></html>";
        assert_eq!(enforce_rtl(already), already);
        // Has no html tag — left untouched.
        assert_eq!(enforce_rtl("<div>\u{633}\u{644}\u{627}\u{645}</div>"), "<div>\u{633}\u{644}\u{627}\u{645}</div>");
    }

    // ── Astra gate ──

    const PASSING_DOC: &str = "<!DOCTYPE html><html dir=\"rtl\" lang=\"fa\"><head>\
<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
<style>body{width:100%;max-width:390px}</style></head>\
<body><canvas id=\"app\"></canvas><button>\u{634}\u{631}\u{648}\u{639}</button>\
<script>document.addEventListener('visibilitychange',()=>{/*pause*/});</script>\
</body></html>";

    #[test]
    fn astra_gate_passes_good_document() {
        let a = AstraQualityGate::default().audit(PASSING_DOC);
        assert_eq!(a, AstraAudit {
            first_frame_interactive: true,
            self_contained_single_file: true,
            pauses_on_visibility_change: true,
            mobile_viewport_meta: true,
            mobile_zero_overflow_static: true,
        });
        assert!(a.passed());
        assert_eq!(a.failures(), [] as [&str; 0]);
    }

    #[test]
    fn astra_gate_flags_dead_marketing_splash() {
        let doc = "<!DOCTYPE html><html><head><meta name=\"viewport\" content=\"width=device-width\">\
</head><body><div id=\"splash\" class=\"splash-screen\">\u{628}\u{647} \u{633}\u{627}\u{6cc}\u{62a} \u{645}\u{627} \u{62e}\u{648}\u{634} \u{622}\u{645}\u{62f}\u{6cc}\u{62f}…</div>\
<script>document.addEventListener('visibilitychange',()=>{});</script></body></html>";
        let a = AstraQualityGate::default().audit(doc);
        assert!(!a.first_frame_interactive, "splash gate must fail frame-1 rule");
        assert_eq!(a.failures().len(), 1);
    }

    #[test]
    fn astra_gate_flags_broken_local_import() {
        let doc = PASSING_DOC.replace("<canvas id=\"app\"></canvas>", "");
        let doc = doc.replace("</head>", "<script src=\"app.js\"></script></head>");
        let a = AstraQualityGate::default().audit(&doc);
        assert!(!a.self_contained_single_file, "relative src= must fail");
        // https CDN is allowed — a "broken local import" is only relative.
        let cdn = PASSING_DOC.replace("</head>", "<script src=\"https://cdn.example/x.js\"></script></head>");
        assert!(AstraQualityGate::default().audit(&cdn).self_contained_single_file);
    }

    #[test]
    fn astra_gate_flags_missing_visibility_pause_and_wide_layout() {
        let no_pause = PASSING_DOC.replace("visibilitychange", "focuschange");
        let a = AstraQualityGate::default().audit(&no_pause);
        assert!(!a.pauses_on_visibility_change);

        let wide = PASSING_DOC.replace("max-width:390px", "width:720px");
        let a = AstraQualityGate::default().audit(&wide);
        assert!(!a.mobile_zero_overflow_static, "720px fixed width must fail 390 rule");

        // The width="450" attribute is overflow too.
        let attr_wide = PASSING_DOC.replace("<canvas id=\"app\"></canvas>", "<canvas id=\"app\" width=\"450\"></canvas>");
        assert!(!AstraQualityGate::default().audit(&attr_wide).mobile_zero_overflow_static);
        // But width:100% and vw ≤ 100 are fine.
        assert!(AstraQualityGate::default().audit(PASSING_DOC).mobile_zero_overflow_static);
    }

    #[test]
    fn astra_gate_flags_missing_viewport_meta() {
        let no_vp = PASSING_DOC.replace("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">", "");
        let a = AstraQualityGate::default().audit(&no_vp);
        assert!(!a.mobile_viewport_meta);
    }

    // ── verbatim block ──

    #[test]
    fn verbatim_budget_clamps_2000_to_24000() {
        assert_eq!(verbatim_budget_chars(10_000), 2_000, "500 → \u{6a9}\u{641} \u{6f2}\u{66c}\u{6f0}\u{6f0}\u{6f0}");
        assert_eq!(verbatim_budget_chars(100_000), 5_000, "\u{6f5}\u{66a} \u{62f}\u{642}\u{6cc}\u{642}");
        assert_eq!(verbatim_budget_chars(1_000_000), 24_000, "\u{6f5}\u{6f0}\u{66c}\u{6f0}\u{6f0}\u{6f0} → \u{633}\u{642}\u{641} \u{6f2}\u{6f4}\u{66c}\u{6f0}\u{6f0}\u{6f0}");
        assert_eq!(verbatim_budget_chars(0), 2_000);
    }

    #[test]
    fn verbatim_keeps_short_messages_whole() {
        let msgs = ["\u{633}\u{644}\u{627}\u{645}", "\u{62f}\u{648}\u{645}\u{6cc} \u{647}\u{645} \u{6a9}\u{648}\u{62a}\u{627}\u{647} \u{627}\u{633}\u{62a}", "third is short"];
        let block = build_verbatim_user_messages_block(&msgs, 400_000);
        assert!(block.starts_with(VERBATIM_OPEN));
        assert!(block.ends_with(VERBATIM_CLOSE));
        for m in msgs {
            assert!(block.contains(m), "short message must survive whole: {m}");
        }
        assert!(!block.contains("trimmed"), "no message was cut");
    }

    #[test]
    fn verbatim_cuts_long_messages_to_fair_share() {
        let long = "x".repeat(50_000);
        let msgs = [long.as_str()];
        let block = build_verbatim_user_messages_block(&msgs, 100_000);
        let budget = verbatim_budget_chars(100_000); // 5,000
        let content = block
            .lines()
            .filter(|l| !l.starts_with('[') && *l != VERBATIM_OPEN && *l != VERBATIM_CLOSE)
            .fold(0usize, |acc, l| acc + l.chars().count());
        assert!(content <= budget, "content {content} must fit budget {budget}");
        assert!(block.contains(&format!("[+{} chars trimmed]", 50_000 - content)));
    }

    #[test]
    fn verbatim_water_fill_gives_slack_to_long_messages() {
        let short = "s".repeat(200);
        let long = "L".repeat(20_000);
        let msgs = [short.as_str(), long.as_str()];
        let block = build_verbatim_user_messages_block(&msgs, 200_000);
        let budget = verbatim_budget_chars(200_000); // 10,000
        // The short message stayed complete (its leftover was redistributed to the long message)
        let short_line = block
            .lines()
            .find(|l| l.starts_with('s') && l.len() == 200)
            .expect("short message whole");
        assert_eq!(short_line.chars().count(), 200);
        // The long message got more than its raw share (5,000) — the short message's 200 leftover reached it
        let long_kept = block
            .lines()
            .find(|l| l.starts_with('L'))
            .expect("long message line")
            .chars()
            .count();
        assert!(long_kept > budget / 2, "long message must exceed raw share: {long_kept}");
        let total = block
            .lines()
            .filter(|l| !l.starts_with('[') && *l != VERBATIM_OPEN && *l != VERBATIM_CLOSE)
            .fold(0usize, |acc, l| acc + l.chars().count());
        assert!(total <= budget, "water-fill must respect budget: {total} > {budget}");
    }

    #[test]
    fn verbatim_empty_input_is_empty_string() {
        assert_eq!(build_verbatim_user_messages_block(&[], 100_000), "");
    }
}
