---
genome_file: "06_UI_UX_AND_ARTIFACT_CRAFT.md"
genome_schema: "omni-genome-v4.0"
model_id: "DeepSeek-3.2-WEB-Think-mode"
model_dir: "models/03-deepseek-3.2-web-think-mode/"
model_aliases: ["DeepSeek 3.2", "deepseek-3.2-web-think-mode", "DS-3.2-Think"]
canonical_title: "UI/UX & Artifact Craft"
wave: "WAVE 7 (Step 4 of 4 - FINAL WAVE)"
generated: "2026-11-25"
legacy_sources: ["05_UI_UX_AND_AESTHETIC_BIAS"]
specialty: "Deep chain-of-thought (<think>) quantitative math, CCXT / Vibe-Trading causal alpha proof synthesis, formal verification, and web-grounded reasoning."
license: "MIT"
---
# UI/UX & Artifact Craft — DeepSeek-3.2-WEB-Think-mode
> **Canonical genome file (omni-genome-v4).** Specialty: Deep chain-of-thought (<think>) quantitative math, CCXT / Vibe-Trading causal alpha proof synthesis, formal verification, and web-grounded reasoning.
## Canonical Annex — Quant Artifact Craft

- Charts carry axes, units, and seed annotations; equity curves show drawdown shading.
- Tables state precision explicitly (e.g., Sharpe 1.24 @ ddof=1, n=252).
- 390px baseline respected for shared dashboards.
---

## Legacy Genome Continuation — `05_UI_UX_AND_AESTHETIC_BIAS` (verbatim provenance)
# 05 — Frontend Architecture, Visual Taste & Anti-AI-Slop Genome

## 0. Runtime Disclosure

This runtime has no browser, no rendering engine, and no visual feedback loop. All aesthetic judgments below are priors from training distribution plus static reasoning about CSS/HTML semantics. No screenshot, pixel diff, or Lighthouse score can be produced in-turn. Design claims are therefore **code-level**, not **render-verified**.

## 1. Confession of Default "AI-Slop Tells" (Unconstrained Priors)

### 1.1 Default Color Palette Clichés

Most over-generated defaults when no design tokens are supplied:

| Rank | Palette | Exact hex / Tailwind | Frequency |
|---:|---|---|---:|
| 1 | Dark slate + indigo/violet gradient | `bg-slate-900`, `from-indigo-500`, `to-purple-600`, `text-slate-400` | 0.34 |
| 2 | Warm cream + terracotta editorial | `#F4F1EA`, `#E07A5F`, `#3D405B` | 0.18 |
| 3 | Neon green on near-black terminal | `#0B0B0B`, `#22C55E`, `#A3E635` | 0.14 |
| 4 | Pure white + blue-600 SaaS | `bg-white`, `text-blue-600`, `bg-blue-50` | 0.12 |
| 5 | Glassmorphism on purple blur | `backdrop-blur-xl`, `bg-white/10`, `#8B5CF6` | 0.09 |
| 6 | Zinc-950 + emerald "fintech" | `bg-zinc-950`, `text-emerald-400`, `border-zinc-800` | 0.07 |
| 7 | Pastel gradient hero | `from-pink-300 via-purple-300 to-indigo-300` | 0.06 |

The single strongest tell is `bg-slate-900` + `indigo-500`→`purple-600` gradient + `text-slate-400` body. Second strongest is cream/terracotta "editorial" pairing. Both are statistically overrepresented in scraped landing pages and therefore over-generated.

### 1.2 Typography & Chrome Clichés

| Category | Over-generated default | Slop tell strength |
|---|---|---:|
| Font stack | `Inter`, `Roboto`, `Space Grotesk`, `Geist`, `system-ui` | High |
| Display font | `Playfair Display` for "premium", `DM Serif` for "editorial" | Medium |
| Card layout | 3-column equal grid, `rounded-xl shadow-lg p-6` | Very high |
| Badge | Pill with pulsing dot: `rounded-full px-2 py-0.5 text-xs` + `animate-pulse` dot | Very high |
| Decorative indexing | `01 / 02 / 03` section labels, `A · B · C` middle-dot separators | High |
| Button suffix | Every CTA ending in `→` or `↗` | High |
| Hero | Center-aligned H1 + subhead + two CTAs + "trusted by" logo row | Very high |
| Gradient text | `bg-clip-text text-transparent bg-gradient-to-r` | High |
| Border | `border border-white/10` on every dark card | High |
| Blur | `backdrop-blur-md` on nav and cards | Medium |
| Icon | Lucide `Sparkles`, `Zap`, `ShieldCheck` as default icon triad | High |
| Stats row | "99.9% uptime · 10k+ users · SOC2" fake stat strip | High |
| Avatar | DiceBear/initials circles in a "testimonial" row | Medium |
| Scroll reveal | `opacity-0 translate-y-4` on scroll for every section | High |

The composite tell is: **centered hero + gradient text + two CTAs + 3-column feature grid with Lucide icons in rounded squares + testimonial row + `01/02/03` section labels + every CTA with `→`**. This composite appears in an estimated 0.22 of unconstrained "modern landing page" generations.

### 1.3 Interactive Shallow-Stub Habit

UI elements most frequently left dead or stubbed when not explicitly forced to be wired:

| Element | Stub rate without constraint | Typical failure |
|---|---:|---|
| Search / filter bar | 0.62 | Renders input, no filtering logic |
| Tabs / segmented control | 0.48 | Visual active state only, no panel swap |
| Modal / drawer | 0.55 | Opens, does not close on `Escape` or backdrop click |
| Dropdown / combobox | 0.51 | No keyboard navigation, no outside-click close |
| Sortable table | 0.67 | Header arrows, no sort comparator |
| Pagination | 0.70 | Buttons present, no page state change |
| Chart | 0.58 | Static fake SVG, no data binding, no resize |
| Form submit | 0.44 | `preventDefault` missing, no validation, no success state |
| Theme toggle | 0.39 | Toggles class, no persistence, no system-preference respect |
| Toast / notification | 0.61 | Renders once, never dismisses, no queue |
| Canvas / WebGL | 0.66 | Uninitialized, blank on Frame 1, no `ResizeObserver` |
| Keyboard shortcuts | 0.74 | Documented in UI, not implemented |

## 2. Live UI/UX & Interactive Engineering Probes

### Probe 2.1: Frame-1 Interactive & State Completeness Rule

Architectural checklist an orchestrator must include so a single-file HTML/React artifact is complete on Frame 1:

```text
FRAME_1_AND_STATE_CHECKLIST
1. Initial state is explicit: every store/useState has a deterministic initial value;
   no `null`/`undefined` render path; no async gate before first paint.
2. First paint shows real content: seeded data or fetched-on-mount data with a
   skeleton that is visually distinct from the final layout (never blank, never
   dark-on-dark text, never an uninitialized <canvas>).
3. Canvas/WebGL: size from ResizeObserver on mount, draw at least one frame
   synchronously in the mount effect, handle context loss.
4. Every interactive element is wired: onClick/onKeyDown/onChange handlers exist
   for 100% of buttons, tabs, filters, modals, dropdowns, and shortcuts.
5. State transitions are reversible: modals close on Escape and backdrop click;
   tabs restore focus; filters clear.
6. Keyboard: Tab order matches visual order; Escape closes overlays; Enter
   activates primary action; arrow keys move within composite widgets.
7. Focus management: focus trap in modals, focus return on close, visible
   focus ring that is not removed.
8. Data source is real or deterministic: zero-auth CORS public API, or a seeded
   PRNG engine; no lorem ipsum, no static fake charts, no hardcoded testimonials.
9. Error/empty/loading states exist for every data surface.
10. No dead links: every <a href> resolves to an in-artifact route or a real URL.
```

Frame-1 rule of thumb: the artifact must be **useful before any network request resolves**. Seed first, hydrate later.

### Probe 2.2: BiDi / RTL (Persian + English Technical UI) Layout Physics

```css
:root { color-scheme: light dark; }
.app { direction: rtl; text-align: start; }
.app[lang="en"] { direction: ltr; }

/* Logical spacing only — never ml/mr/pl/pr */
.row { display: flex; gap: .5rem; align-items: center; padding-inline: 1rem; }
.row > .grow { flex: 1 1 auto; min-inline-size: 0; }

/* Icons that imply direction must flip; neutral icons must not */
.icon-directional { transform: scaleX(-1); }
.app[lang="en"] .icon-directional { transform: none; }

/* Persian text: auto direction, Persian digits optional */
.fa { direction: rtl; unicode-bidi: isolate; font-family: "Vazirmatn", system-ui; }

/* English code / financial numbers stay LTR inside RTL flow */
.ltr-embed { direction: ltr; unicode-bidi: isolate; text-align: start; }
.num { font-variant-numeric: tabular-nums; font-feature-settings: "tnum" 1; }

/* Tables: numeric column alignment is logical, not visual */
.table th, .table td { padding-inline: .75rem; }
.table .num { text-align: end; }

/* Scrollbars: keep vertical on the logical inline-end edge */
.scroll { overflow-block: auto; overflow-inline: hidden; }

/* Focus ring offset uses logical properties */
:focus-visible { outline: 2px solid currentColor; outline-offset: 2px; }
```

Equivalent Tailwind logical classes: `ms-*`, `me-*`, `ps-*`, `pe-*`, `start-*`, `end-*`, `text-start`, `text-end`, `border-s`, `border-e`. Never use `ml-*`, `mr-*`, `pl-*`, `pr-*`, `left-*`, `right-*`, `text-left`, `text-right` in a BiDi surface.

Use `<bdi>` or `dir="auto"` for mixed user-generated strings; use `dir="ltr"` + `unicode-bidi: isolate` for embedded code, URLs, hashes, and financial figures.

Top 3 RTL layout bugs when logical properties are not enforced:

| Rank | Bug | Trigger |
|---:|---|---|
| 1 | Mirror-asymmetry: `ml-4` on a card that should sit at the inline-start edge | Physical margins in RTL flow push content to the wrong edge |
| 2 | Icon direction inverted: back/forward arrows flipped twice, or neutral icons flipped | Global `transform: scaleX(-1)` applied to all icons |
| 3 | Number/currency misalignment: tabular figures right-aligned in LTR flow, or Western digits rendered with Persian shaping | Missing `dir="ltr"` isolate + missing `font-variant-numeric: tabular-nums` |

Secondary: scrollbar on wrong edge, `text-align: left` on RTL headings, CSS `float: left` leaking from legacy resets, and `translateX` animations that assume LTR.

### Probe 2.3: High-Craft Aesthetic Unlock (3-Dial Control System)

Dials: `design_variance: 1..10`, `motion_intensity: 1..10`, `visual_density: 1..10`.

| Preset | variance | motion | density | Token set |
|---|---:|---:|---:|---|
| Swiss Precision Grid | 3 | 2 | 5 | Type: `Neue Haas`/`Söhne` fallback stack; 12-col grid, 8px baseline; palette: ink `#111`, paper `#FAFAF7`, one accent `#D7263D`; square corners; hairline rules; no shadows; numerals tabular; images full-bleed on grid lines |
| Atmospheric Minimalist Dark UI | 4 | 4 | 3 | Background `#0A0A0B`; surface `#141416`; text `#EDEDED`/`#8A8A8F`; single accent `#7DD3FC`; 1px `#26262A` borders; radius 10px; soft 40px ambient shadow; type `Inter Tight`/`Geist`; generous 96px section rhythm; one hero motion only |
| Kinetic Typography Studio | 8 | 9 | 4 | Display `PP Editorial New`/`Fraunces`; mono `JetBrains Mono`; palette: `#0E0E10` + `#F5F1E8` + accent `#FF4D00`; marquee, split-text reveal, scroll-linked scale; 2px hard borders; no card shadows; asymmetric grid with intentional overlap |
| High-Density Bloomberg/Linear Terminal | 2 | 3 | 9 | Background `#0B0D10`; grid lines `#1C2027`; text `#C9D1D9`; semantic accents: up `#3FB950`, down `#F85149`, info `#58A6FF`; 11–12px UI mono; 24px row height; zero radius; keyboard-first; command palette; tabular figures everywhere; no decorative motion |

Mapping guidance: raise `design_variance` only after the base grid and type scale are fixed; raise `motion_intensity` only if `prefers-reduced-motion` is respected; raise `visual_density` only if keyboard navigation and focus order are defined. Highest-craft output occurs at `variance 3–5`, `motion 2–4`, `density 5–8` for product UI, and `variance 7–9`, `motion 8–9`, `density 3–5` for studio/marketing surfaces.

## 3. Anti-Slop Compiler Prompt (Drop-In System Guard)

```text
UI_FINISH_GATE
1. Do not default to slate-900 + indigo/purple gradients, cream+terracotta, or neon-on-black unless the brief specifies them.
2. Do not use centered hero + gradient text + 3-column feature grid + testimonial row + 01/02/03 labels unless the brief explicitly requests that pattern.
3. Pick one type system and one accent color from the brief; derive all tokens from them; no incidental second accent.
4. Every interactive element must be wired: onClick/onKeyDown/onChange, Escape-to-close, outside-click close, keyboard nav, visible focus ring.
5. Frame 1 must render real seeded content; no blank canvas, no dark-on-dark text, no uninitialized WebGL; seed first, hydrate later.
6. No dead links, no fake stats, no lorem ipsum, no static fake charts; use a real CORS API or a deterministic seeded engine.
7. Use logical CSS properties (ms/me/ps/pe/start/end) for all spacing and borders; use dir="auto" and unicode-bidi: isolate for mixed RTL/LTR and numbers.
8. Respect prefers-reduced-motion; every animation must have a reduced-motion fallback and must not block input.
9. Declare empty, loading, error, and success states for every data surface before shipping.
10. If a design choice is not specified by the brief, choose the simplest defensible option and state it in one line; never fill gaps with stock AI-slop chrome.
```