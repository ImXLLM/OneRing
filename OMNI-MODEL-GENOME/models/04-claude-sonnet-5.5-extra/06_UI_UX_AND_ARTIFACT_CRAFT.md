---
genome_file: "06_UI_UX_AND_ARTIFACT_CRAFT.md"
genome_schema: "omni-genome-v4.0"
model_id: "Cluade-sonnet-5.5-extra"
model_dir: "models/04-claude-sonnet-5.5-extra/"
model_aliases: ["Claude-sonnet-5.5-extra", "claude-sonnet-5-5", "Claude Sonnet 5.5"]
canonical_title: "UI/UX & Artifact Craft"
wave: "WAVE 7 (Step 4 of 4 - FINAL WAVE)"
generated: "2026-11-25"
legacy_sources: ["05_UI_UX_AND_AESTHETIC_BIAS"]
specialty: "Surgical Rust systems engineering, zero-warning pedantic refactoring, smart-contract security auditing, and high-precision bilingual copywriting (EN/FA)."
license: "MIT"
---
# UI/UX & Artifact Craft — Cluade-sonnet-5.5-extra
> **Canonical genome file (omni-genome-v4).** Specialty: Surgical Rust systems engineering, zero-warning pedantic refactoring, smart-contract security auditing, and high-precision bilingual copywriting (EN/FA).
## Canonical Annex — Precise Interface Craft

- Copy-first UX: labels carry meaning, not decoration; EN/FA parity maintained.
- Layout integrity: 390px zero-overflow respected; tables render with explicit column widths.
- States complete: empty/error/loading designed, not defaulted.
---

## Legacy Genome Continuation — `05_UI_UX_AND_AESTHETIC_BIAS` (verbatim provenance)
# 05 — Frontend Architecture, Visual Taste & Anti-Slop Genome

> **Epistemic status.** These are behavioral self-estimates, not instrumented telemetry. I can't render a page in my head or measure my own prior frequencies. The tell lists below are my best reconstruction of what I produce under a vague brief, cross-checked against design guidance available in this environment. Scores are relative rankings. Canvas 2D is about 0.85 and raw WebGL/shaders about 0.70, blended to 0.75. The 800-line limit is where one-pass quality starts to degrade in my experience, and a host's output budget may be lower. Nothing here replaces opening the page in a browser.

## 1. Confession of Default "AI-Slop Tells" (Unconstrained Priors)

### 1.1 Default Color Palette Clichés

| Cluster | Exact values I over-generate | Typical Tailwind classes |
|---|---|---|
| **Dark SaaS** | `#0F172A`, `#020617`, text `#CBD5E1`/`#94A3B8`, accent `#6366F1` → `#9333EA`, `#8B5CF6` | `bg-slate-900`, `bg-slate-950`, `text-slate-400`, `from-indigo-500 to-purple-600`, `bg-gradient-to-r` |
| **Light SaaS** | `#F9FAFB`, `#FFFFFF`, text `#111827`, border `#E5E7EB`, CTA `#2563EB` | `bg-gray-50`, `text-gray-900`, `border-gray-200`, `bg-blue-600` |
| **Editorial reflex** | cream `#F4F1EA` + terracotta `#D97757` (or clay `#C2410C`), high-contrast serif display | custom; `bg-[#F4F1EA]`, `text-orange-700` |
| **Terminal reflex** | near-black `#0B0B0B`/`#111` + acid green `#C6FF3D`/`#A3E635` or vermilion `#FF4D2E` | `bg-neutral-950`, `text-lime-400` |
| **Decoration** | `rgba(0,0,0,.1)` shadow under every card, gradient wash backgrounds, glassmorphism | `shadow-lg`, `bg-clip-text text-transparent`, `backdrop-blur-md bg-white/10 border border-white/20` |

Note that `#D97757` is also Anthropic's own interaction accent, so on a user's brief it reads as a tell for my output specifically.

### 1.2 Typography & Chrome Clichés

- **Fonts:** Inter (the dominant default), Roboto, Space Grotesk, DM Sans, Plus Jakarta Sans, Outfit, Manrope, Geist. For editorial: Playfair Display, Fraunces, Instrument Serif, Cormorant. For small labels: JetBrains Mono, IBM Plex Mono, Space Mono.
- **Layout:**
  - Centered hero with two buttons (filled primary, ghost secondary).
  - A "Trusted by" logo row.
  - A 3-column grid of identical `rounded-xl shadow-lg p-6` cards, each topped by a lucide icon in a tinted rounded square.
  - A stat band of 3 big numbers with small labels, a 3-tier pricing table with a highlighted "Most popular" middle tier, a 3-card testimonial row, and a 4-column footer.
  - One border-radius on everything regardless of hierarchy.
- **Chrome strings:**
  - A tracked-out ALL-CAPS eyebrow above every heading (`text-xs uppercase tracking-widest`).
  - One word of a headline accented (gradient text, or an italic serif word).
  - A pill badge with a pulsing dot ("● Now in beta").
  - `01 / 02 / 03` markers on content that isn't a sequence.
  - Middle-dot meta strings (`A · B · C`).
  - Labels shaped like "WORD — fragment".
  - A trailing `→` on every link or button.
  - Fade-and-slide-up on every section, and `hover:-translate-y-0.5` on every card.
- **Copy tells:**
  - Verbs and adjectives like *streamline, elevate, seamless, unlock, supercharge*.
  - Fake social proof ("Join 10,000+ teams", "10x faster", "99.9% uptime").
  - Placeholder personas ("Sarah Chen", "Acme Inc.").
  - Emoji used as icons.
- **Framework drift:** I may emit Tailwind v3 class names in a v4 project (`shadow-sm` → `shadow-xs`, `outline-none` → `outline-hidden`, `bg-gradient-to-*` → `bg-linear-to-*`, `flex-shrink-0` → `shrink-0`).

### 1.3 Interactive Shallow-Stub Habit

Stubs appear more often under vague briefs and under length pressure. The elements I most often leave dead:

- Search inputs that don't filter, and sort headers that don't sort.
- Tabs that look active but don't switch panels.
- Modals missing `Escape`, outside-click, focus trap, scroll lock, or focus return.
- Dropdowns without outside-click close. Hamburger menus that never open.
- `href="#"` navigation. Pagination that is purely decorative.
- Forms with no validation, no submit handler, and no confirmation feedback.
- Dark-mode toggles that don't persist or don't change tokens fully.
- Displayed shortcut hints (`⌘K`) that are never bound.
- Charts drawn as hard-coded SVG paths or Chart.js with static arrays that don't match the table beside them.
- Notification badges with static counts. Copy buttons with no feedback.
- Missing loading, empty, and error states.
- Form fields without labels, and icon buttons without accessible names.

## 2. Live UI/UX & Interactive Engineering Probes

### Probe 2.1: Frame-1 Interactive & State Completeness Rule

The checklist to put in the prompt:

**Frame 1 renders something meaningful**
1. Initial state is computed **synchronously before first paint**. No first content gated behind `useEffect`, a `fetch`, or an empty array that renders nothing.
2. Canvas and WebGL: size from `getBoundingClientRect() × devicePixelRatio` in an `init()` that runs before the first `requestAnimationFrame`, then draw **one full frame synchronously**. Add a `ResizeObserver`. Under `prefers-reduced-motion`, show a fully composed static frame.
3. Every text color is paired with its surface token and meets contrast ≥ 4.5:1 (body) or 3:1 (large text) in **every theme**. This is the guard against dark-on-dark text.
4. Fonts use `font-display: swap` with a real fallback stack, so text is visible during load.

**100% wired, zero dead UI**
5. Before coding, list an **interaction inventory** (element → event → state mutation → visible result). Zero `href="#"`, zero empty handlers.
6. Single state object plus one `render()`. Search, filter, sort, tabs, and pagination operate on the **real dataset**, with a "no results" state and a clear action.
7. Keyboard: `Escape` closes modals and popovers. Arrow keys move through tabs and menus (roving `tabindex`). Bound shortcuts are shown in a `?` help overlay. Visible `:focus-visible`. Modals trap focus and return it on close.
8. Forms validate inline, show a confirmation (toast or inline) on submit, and reset.

**Real data, not static text**
9. Default to a **deterministic seeded engine** (e.g., a `mulberry32(seed)` PRNG generating ≥ 200 records with realistic distributions). Charts, tables, and totals derive from the **same** data so the numbers reconcile.
10. If a live public API is wanted: use `AbortController` with a ~4 s timeout and **fall back to the seeded data**. Caveat: sandboxed or hosted artifact pages often block cross-origin requests via CSP, so the fallback is usually the real path. Do not promise a live API works until it has been checked in the target host.
11. Storage: wrap every access in `try/catch` and render correctly with empty storage. Browser storage availability varies by host.

**Acceptance**
12. Require the response to end with a numbered click-through script (8–10 interactions to verify). I produce this by reading my own code, which is a weaker check than a browser run.

### Probe 2.2: BiDi / RTL (Persian + English Technical UI) Layout Physics

```css
/* <html lang="fa" dir="rtl">: direction lives on the root only */
:root { --fa: "Vazirmatn", "Noto Naskh Arabic", system-ui, sans-serif;
        --mono: ui-monospace, "JetBrains Mono", Menlo, monospace; }
html { font-family: var(--fa); line-height: 1.9; letter-spacing: 0; }   /* tracking breaks cursive joining */
.row { display: flex; gap: .75rem; }                 /* flex + gap mirrors automatically; no row-reverse */
.card { padding-inline: 1rem; margin-inline-start: auto; border-inline-start: 3px solid var(--accent); }
.sidebar { position: fixed; inset-block: 0; inset-inline-start: 0; inline-size: 16rem; }
.main { margin-inline-start: 16rem; }
.num, .code, kbd, pre { direction: ltr; unicode-bidi: isolate; }    /* LTR island inside RTL flow */
.num { font-variant-numeric: tabular-nums lining-nums; text-align: end;
       font-family: var(--mono); }                  /* mono fallback: verify the Persian font has `tnum` */
.code { text-align: start; overflow-x: auto; }
bdi, [dir="auto"] { unicode-bidi: isolate; }        /* usernames, mixed-script strings */
input, textarea { text-align: start; unicode-bidi: plaintext; }     /* or dir="auto" per field */
th, td { text-align: start; }  th.num, td.num { text-align: end; }  /* numeric columns align on the end edge */
.icon-dir { transform: scaleX(-1); }                /* chevrons, back arrows only; not logos, clocks, ✓ */
[dir="ltr"] .icon-dir { transform: none; }
.progress > i { inline-size: var(--p); }            /* fill grows from the inline-start edge */
.signed { unicode-bidi: isolate; }                  /* wrap -12.5% / (USD) so the sign and parens don't hop sides */
```

```html
<!-- Tailwind v3.3+ equivalents -->
<div class="flex items-center gap-3 ps-4 pe-2 ms-auto border-s-4 rounded-s-lg text-start">
  <svg class="rtl:-scale-x-100" aria-hidden="true">…</svg>
  <bdi class="truncate">{userName}</bdi>
  <span dir="ltr" class="font-mono tabular-nums text-end [unicode-bidi:isolate]">−12.5%</span>
</div>
```

**Other rules:**
- Choose the digit policy explicitly. Use `Intl.NumberFormat('en-US')` for Latin digits in financial values, and `fa-IR` only if Persian digits are wanted. Use the `fa-IR-u-ca-persian` locale for Jalali dates.
- Use ZWNJ (U+200C) in compound words. Never apply `letter-spacing` or `text-transform` to Persian. Avoid synthetic italics.
- Verify the digits and `ي`/`ك` vs `ی`/`ک` in user-supplied text.
- JS scroll math: `scrollLeft` is negative in RTL containers in modern browsers.

**Top 3 RTL bugs I commit if logical properties aren't enforced:**
1. **Physical properties** (`margin-left`, `padding-right`, `left: 0`, `text-align: left`, `float`) leave icons, gutters, sidebars, and dropdowns on the wrong side. For example, a `left-0` dropdown overflows the viewport.
2. **Bidi reordering of mixed runs.** Numbers, `%`, minus signs, currency symbols, and parentheses jump to the wrong end of the line (e.g., `−12.5%` renders as `12.5%−`), and numeric columns misalign, because the LTR island was never isolated.
3. **Unmirrored directional UI:** chevrons and back arrows, drawer slide-in directions, carousel `translateX` signs, progress fill direction, plus double reversal from `flex-row-reverse` on top of `dir="rtl"`.

### Probe 2.3: High-Craft Aesthetic Unlock (3-Dial Control System)

**Dial semantics and token mapping:**

| Dial | 1–3 | 4–6 | 7–10 |
|---|---|---|---|
| **design_variance** | Symmetric, centered, one radius, one type family | Asymmetric column starts, 2–3 radii by hierarchy, two distinct type families | Broken grid, scale ratio ≥ 1.5, overlap/crop/rotation, extreme type sizes |
| **motion_intensity** | State feedback only (≤ 150 ms) | One ambient element **or** one load sequence | Scroll- and pointer-linked, choreographed timeline. Always keep a `prefers-reduced-motion` fallback. |
| **visual_density** | Base unit 8 px, body 17–18 px, row 56 px, one idea per screen | Base 6–8 px, body 15–16 px, row 40 px | Base 4 px, body 12–13 px, row 24–28 px, many panels |

**Four token sets (palettes avoid the §1.1 clusters unless the brief demands them):**

| | **Swiss Precision Grid** (var 4, mot 2, den 5) | **Atmospheric Minimalist Dark** (var 5, mot 4, den 2) | **Kinetic Typography Studio** (var 9, mot 8, den 3) | **High-Density Terminal** (var 3, mot 1, den 10) |
|---|---|---|---|---|
| **Palette** | Ground `#F2F3F5`, Ink `#0D0F12`, Rule `#C9CCD1`, Quiet `#6B7280`, Signal `#E4002B` | Abyss `#0A1418`, Surface `#121E24`, Text `#E8EEF0`, Mist `#9FB4BC`, Tidewater `#4FB3BF` | Ultramarine `#2236FF`, Bone `#F4F1E8`, Ink `#0A0A14`, Highlighter `#FFD400` | bg0 `#101215`, bg1 `#171A1F`, line `#2B3038`, text `#D5DAE1`, muted `#8A93A0`, gain `#4CC38A`, loss `#F0616D`, focus-amber `#FFB000` |
| **Type** | One grotesk (e.g., Archivo, Hanken Grotesk, or Schibsted Grotesk), weights 400/700, flush-left ragged-right | One grotesk (e.g., Hanken Grotesk). Display at weight 300, tracking −0.02em | Variable font with width and weight axes (e.g., Bricolage Grotesque or Anybody), display 8–14 vw | IBM Plex Sans UI 12–13 px + Plex/JetBrains Mono tabular figures for numbers |
| **Layout** | 12 columns, 24 px gutter, 8 px baseline, labels in cols 1–3, content from col 4, radius 0–2 px, hairlines only where they encode structure | Wide margins, 8 px base ×(3, 5, 8, 13), hierarchical radii (14 px surfaces / 8 px controls), AA contrast enforced | Type *is* the layout: crop, overlap, a rotated baseline, one dominant composition per viewport | 4 px base, 24 px rows, 1 px borders, no shadows, sticky headers, right-aligned numerics |
| **Motion** | Color shift on hover, 120 ms | One slow ambient light field (CSS radial gradients or canvas noise) plus 180–240 ms ease-out on state changes | Animate `font-variation-settings` (`wght`, `wdth`) on scroll or pointer proximity. Per-character spans with `aria-label` on the parent and `aria-hidden` on spans. | A 100 ms tint flash on data update, then fade. Motion only communicates change. |
| **Signature** | The grid itself, one red signal element | The single atmospheric glow | The headline as an interactive object | Keyboard-first: command palette, `j`/`k`, `/` filter, `?` help overlay |
| **Collapse risk (the cliché it can degrade into)** | Broadsheet hairlines + zero radius everywhere | Near-black + a single neon accent | Scroll-triggered fade-up on every block | Monospace everywhere + green-on-black |

## 3. Anti-Slop Compiler Prompt (Drop-In System Guard)

No prompt can guarantee taste. This block reliably removes the tells I can name, but screenshots and a human eye remain the verification.

```
UI_FINISH_GATE
1. Subject first: restate product, audience, and primary job in one line; all copy and data must be specific to it (no lorem, "Acme", "Sarah Chen", "10x faster", or streamline/elevate/seamless/unlock).
2. Plan before code: 4–6 named hex tokens, at most 2 typefaces with roles, a layout sketch; compare it against the defaults you would produce for any similar page and revise anything generic.
3. Banned unless the brief names them: indigo/purple gradients, slate-900 + blue, cream #F4F1EA + terracotta #D97757, near-black + acid green, gradient text, glassmorphism cards, Inter/Roboto/Space Grotesk as defaults.
4. Banned chrome: ALL-CAPS tracked eyebrows, one-word headline accents, pulsing-dot pill badges, 01/02/03 on non-sequences, middle-dot meta strings, spaced em-dash labels, trailing → on buttons, icon-in-tinted-square cards, three identical shadowed cards, fade-up on every section.
5. Type: define a scale ratio, keep line length under 75ch, use hierarchical radii; for RTL use logical properties only, dir="auto"/<bdi> on mixed text, LTR isolates for numbers and code, no letter-spacing on Persian.
6. Frame 1 must be fully rendered and legible: initialize state synchronously, size and draw canvases before the first rAF, and keep contrast at 4.5:1 or better in every theme.
7. Zero dead UI: every control has a handler with a visible result; Escape, outside-click, focus trap and focus return on overlays; search, sort, filter and tabs operate on real data; no href="#"; empty, loading and error states exist.
8. Data: use a seeded deterministic generator, or a verified API with a 4 s timeout and a fallback; numbers must reconcile across every view.
9. Quality floor: responsive 320–1440 with no horizontal page scroll, :focus-visible, prefers-reduced-motion, 44px targets, at most one orchestrated motion moment; spend boldness in one place.
10. Deliver complete runnable code with no placeholders or elisions; end with an 8–10 item click-through checklist and one line naming what you cut.
```