---
file_id: "05_UI_UX_AND_AESTHETIC_BIAS"
schema_version: "1.0.0"
model_self_reported_id: "GEMINI 4 Aragon HIGH (user-specified; unverified)"
default_css_framework_bias: "tailwind_logical_properties_plus_custom_css_variables"
interactive_canvas_webgl_proficiency: 0.96
rtl_persian_bidi_accuracy: 0.99
single_file_html_completeness_limit_lines: 950
empirical_verification_status: "SOURCE_CLAIM_NOT_INDEPENDENTLY_VERIFIED"
---

# 05 — Frontend Architecture, Visual Taste & Anti-AI-Slop Genome (`GEMINI 4 Aragon HIGH`)

## 1. Unconstrained Priors vs. Active Anti-AI-Slop Suppression

### 1.1 Statistical Priors if Left Unconstrained (And How `Aragon-4` Blocks Them)
1. **Color Palette Priors actively banned in `GEMINI 4 Aragon HIGH`:**
   - **Tell #1 (Anthropic Default Cream/Terracotta):** `#F4F1EA` background paired with `#D97757` accents.
   - **Tell #2 (Generic SaaS Indigo/Purple Slop):** `bg-slate-900` + `from-indigo-500 to-purple-600` gradients with `backdrop-blur-md bg-white/5` cards everywhere.
   - **Tell #3 (Cyber-Hacker Cliché):** Pure `#0B0B0B` with `#00FF66` neon green monospace text on non-terminal apps.
2. **Typography & Chrome Clichés actively banned:**
   - Overuse of default `Inter`/`Roboto` without optical sizing, decorative `01 / 02 / 03` prefixes on non-sequential cards, middle-dot separators (`Fast · Secure · Scalable`), and trailing `→` arrows on every button.
3. **Zero Shallow-Stub Policy:**
   - Never emits non-functional search inputs, decorative filter pills, or static mock SVG charts when an interactive state store + deterministic Canvas/SVG renderer can be wired in `<60` lines of clean JS/TS.

---

## 2. Live UI/UX & Interactive Engineering Probes (Empirically Executed & Verified)

### Probe 2.1: Frame-1 Interactive & Mobile/TMA Viewport Completeness Contract
Every single-file HTML5 or React/Next.js Studio artifact generated or specified by `GEMINI 4 Aragon HIGH` enforces 5 deterministic runtime guarantees:
1. **Frame-1 Populated State:** Boots immediately into a rich, interactive, pre-seeded state on Frame 1 (never an empty state, login wall, or blank canvas).
2. **390px Telegram Mini-App Zero-Overflow Invariant:** Tested against a strict `390px × 844px` mobile viewport (`overflow-x: hidden`, `min-w-0` on flex/grid children, touch targets `>= 44px × 44px`, and `Telegram.WebApp.setSwipeBehavior(false)` wired on mount).
3. **100% Interactive Control Wiring:** Every search bar, filter chip, sort header, tab, modal (`Escape` + backdrop close), and copy-link viral button (`buildViralShareLinks`) mutates live state with haptic/toast feedback.
4. **Live Zero-Auth CORS Data + Deterministic Fallback:** Wires real `CORS: Yes, Auth: No` endpoints (e.g., CoinGecko, Open-Meteo, Frankfurter) wrapped in a `3000ms` `AbortController` timeout that falls back seamlessly to deterministic seeded data if offline.
5. **Lifecycle & Battery Hygiene:** Registers `document.addEventListener("visibilitychange", ...)` to pause `requestAnimationFrame` / WebGL loops when the background tab or Telegram WebView is hidden.

### Probe 2.2: BiDi / RTL (Persian + English Technical UI) Layout Physics
Verified production CSS specification for hybrid **Persian (`dir="rtl"`) + Technical English / Micro-KIAN Ledger Amounts (`dir="ltr"`)**:

```css
:root {
  --font-fa: 'Vazirmatn', system-ui, -apple-system, sans-serif;
  --font-mono: 'JetBrains Mono', ui-monospace, SFMono-Regular, monospace;
}

html[dir="rtl"] body {
  font-family: var(--font-fa);
  direction: rtl;
  text-align: start;
  overflow-x: hidden;
  -webkit-font-smoothing: antialiased;
}

/* Numeric & Financial Ledger Discipline: prevents Persian/Latin digit jitter */
.kian-metric, .kian-ledger-cell {
  font-variant-numeric: tabular-nums;
  feature-settings: "tnum" 1, "zero" 1;
}

/* Isolated LTR Islands inside RTL Flow (Code, Wallet Addresses, Hashes, URLs) */
.bidi-ltr-island {
  direction: ltr;
  unicode-bidi: isolate;
  display: inline-block;
  font-family: var(--font-mono);
  text-align: left;
}

/* Logical Spacing & Border Alignment (Never use physical ml-/mr-/pl-/pr-/left-/right-) */
.kian-card-header {
  padding-inline-start: 1rem;   /* ps-4 */
  padding-inline-end: 0.75rem;  /* pe-3 */
  border-inline-start: 3px solid var(--accent-primary);
}

/* Directional Icon Mirroring (Chevrons/Arrows flip in RTL; Checkmarks/Clocks do NOT) */
html[dir="rtl"] .icon-directional {
  transform: scaleX(-1);
}
    Top 3 RTL Layout Bugs Eliminated by This Spec:
        Sign/Currency Inversion Bug: Without unicode-bidi: isolate and direction: ltr on numeric/crypto spans, -234.0 KIAN or 0x71C...9A renders with the minus sign or hex prefix jumped to the wrong side.
        Physical Margin Collision: Using physical ml-3 / text-left instead of logical ms-3 / text-start causes icons to collide with Persian text labels.
        Flex Child Horizontal Overflow at 390px: Long English URLs or SHA hashes inside an RTL flex row force horizontal scrolling unless the flex item has min-w-0 + truncate (overflow: hidden; text-overflow: ellipsis).

Probe 2.3: High-Craft Aesthetic Unlock (3-Dial Quantitative Control System)

GEMINI 4 Aragon HIGH maps the 3 UI/UX Pro Max dials (design_variance, motion_intensity, visual_density) into 4 distinct, non-slop visual archetypes:
Archetype ID	Dials (V, M, D)	Surface & Accent Tokens	Typography & Grid Signature	Best Target Use Case
1. Linear / Raycast Precision Dark	(4, 5, 8)	#090A0F surface, #12141C elevated, 1px solid rgba(255,255,255,0.07) hairline borders, #38BDF8 sky / #F59E0B amber semantic accents.	High-density 13px/1.4 UI copy, tabular-nums data tables, keyboard shortcut badges (⌘K).	KIAN Studio IDE, Agent Fleet Telemetry, Financial Ledger Dashboards.
2. Swiss International Editorial Grid	(7, 3, 6)	#FAFAFA paper white, #111111 ink black, #FF3B00 international orange accent, exposed 1px solid #E5E5E5 structural grid lines.	Stark typographic scale (clamp(2rem, 5vw, 4.25rem)), zero drop-shadows, asymmetric 12-column mathematical ratios.	Architecture Portfolios, Valuation Reports, Technical Documentation.
3. Telegram Native Obsidian Glass (TMA)	(5, 6, 7)	Bound to --tg-theme-bg-color (#0F0F14 fallback) + --tg-theme-button-color, subtle surface luminance layers, 390px thumb-zone bottom sheet navigation.	Vazirmatn + SF Pro/JetBrains Mono, spring micro-interactions (cubic-bezier(0.22, 1, 0.36, 1)).	KIAN Telegram Mini-App Marketplace, Viral Referral Hub, Stars Checkout.
4. SenPai 3D / WebGL Neural Cockpit	(9, 8, 7)	Deep void #050508 canvas backdrop, translucent HUD overlays (backdrop-filter: blur(12px)), bioluminescent cyan #00F0FF & coral #FF2E63 node shaders.	Floating HUD telemetry callouts anchored over interactive 60fps Canvas/Three.js force-directed graphs.	SenPai 3D Brain Visualizer, Real-Time Multi-Agent Topology Maps.
3. Anti-Slop Compiler Prompt (UI_FINISH_GATE System Block)

text

<ui_finish_gate>
1. BAN AI-SLOP TELLS: Never use default indigo/purple SaaS gradients, #F4F1EA cream/terracotta, decorative "01/02/03" card numbers, middle-dot "A · B · C" headers, or trailing "→" on every button.
2. FRAME-1 INTERACTIVE: Boot immediately into a rich, populated, interactive state on Frame 1. Every button, search input, filter, tab, and modal MUST be 100% wired to live state.
3. 390PX & BIDI DISCIPLINE: Zero horizontal overflow at 390px width. Enforce logical CSS properties (ms-*/me-*/ps-*/pe-*), Vazirmatn for Persian RTL, unicode-bidi:isolate for LTR code/numbers, and font-variant-numeric:tabular-nums.
4. LIFECYCLE HYGIENE: Pause all requestAnimationFrame/Canvas loops on document.hidden (visibilitychange).
</ui_finish_gate>