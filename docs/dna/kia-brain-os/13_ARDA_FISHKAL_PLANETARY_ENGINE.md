# 13_ARDA_FISHKAL_PLANETARY_ENGINE — `ARDA` — the 24-Hour Planetary Localization Engine on authentic `Fishkal` architecture
> **Layer**: `KIA-BRAIN-OS` Kernel Extension #13
> **Subject**: Kia (`@KiaAgentX` / `0xKIA`) — Sovereign Architect (`Ni-Ti` / `Mairon` Archetype)
> **Mandate**: Industrialize Kia's zero-dependency HTML5 localization engine **`Fishkal`** (`3,616` lines, `667,241` bytes) into `ARDA` — the 24-hour planetary localization agent of `0xKIA` — with **all 25 forensic-audit defects cured**, the 20-locale circular vector flag system, the `sha256` delta-guarded cron loop, and Mairon's autonomous `SKILL.md` + `skill.json` forge. (Wave 4 — Step 1 of 4.)

## The `Fishkal` forensic base
- **Source**: line-by-line forensic audit of Kia's own zero-dependency HTML5 engine `Fishkal` — the reference implementation Kia ships inside every self-built product (`viewBox="0 0 20 14"` vector flag dictionary at `L2690–2704`, circular `.lf` badge at `L353–358`, atomic root flip at `L2085`, per-script typography at `L299/L301–304/L389–390`, cached-`Intl` gap at `L2201/L2670`, WAI-ARIA listbox switcher at `L2113–2145` with defects `D-1..D-8`, canvas bidi procedure at `L2898–2933`, `L2683`, `L3082`).
- **Verbatim heritage**: the original 13 locales (`en`, `ar`, `fa`, `tr`, `fr`, `es`, `pt`, `nl`, `ja`, `hi`, `zh`, `id`, `ko`) keep their exact SVG bytes (`FISHKAL_VECTOR_FLAGS` is test-pinned: `original_13_svgs_are_verbatim_fishkal`); 7 planetary locales (`de`, `ru`, `it`, `he`, `ur`, `vi`, `th`) are added in the same vector grammar → **exactly 20 locales**.
- **Native-script switcher glyphs preserved** (`STR.code`): `'\u{0641}\u{0627}'` (fa), `'\u{0639}'` (ar), `'\u{0627}\u{0631}\u{062F}\u{0648}'` (ur), `'עב'` (he), uppercase ISO for LTR locales — alongside canonical `native_name` (`'\u{0641}\u{0627}\u{0631}\u{0633}\u{06CC}'`, `'\u{0627}\u{0644}\u{0639}\u{0631}\u{0628}\u{064A}\u{0629}'`, `'English'`, `'Bahasa Indonesia'`, …) and `TextDirection` (`Ltr` | `Rtl`, exactly 4 RTL locales).

## The 25 cured defects — implemented in `crates/hermes/src/arda_fishkal_engine.rs`
| # | Defect class | Cure (test-pinned) |
| :-- | :--- | :--- |
| 1 | Non-atomic root flip | `ARDA.applyLocale` sets `root.lang`, `root.dir`, `root.dataset.lang` in one synchronous tick (`L2085` cured) |
| 2–3 | Global-only font / broken RTL tracking | per-locale `:root[data-lang="…"]` blocks set `--font`, `--track-body`, `--track-display`; RTL (`ar`,`fa`,`ur`,`he`) uses `'Vazirmatn'`, drops `.34em → .16em` (body) and `.07em` + `text-indent:.07em` (display) so cursive letter-joining never breaks (`L299/L301–304/L389–390` upgraded) |
| 4–9 | 6 fragmented dictionaries (`I18N`,`BIZSTR`,`LOGSTR`,`CARDSTR`,`OPTSTR`,`SPECIES`) | one unified `ArdaLocaleCatalog` over 6 scopes (`core`, `business`, `records`, `share`, `tools`, `content`); `validate_catalog_parity` mechanically proves 100% key parity (kills the missing-`unitFt` bug in `tr..ko` and the 3-locale-only `OPTSTR` defect), detects duplicate keys, and emits exact `coverage_score` (`0.0..=1.0`) |
| 10–11 | `Intl` constructor churn (`L2201`, `L2670`) | `Intl.NumberFormat` / `Intl.PluralRules` / `Intl.DateTimeFormat` cached per locale with region subtags (`fa-IR`, `ar-AE`, `hi-IN`, `de-DE`, `fr-FR`, `ja-JP`, `zh-CN`, `tr-TR`, `ru-RU`, …) + currency formatting (`XTR`, `USD`, `AED`, `IRR`, `USDC`) |
| 12–14 | DOM bidi breaks on composite strings / latin tickers | `dir="auto"` + `unicode-bidi:plaintext` on composites, forced `dir="ltr"` on `[data-dir="latin"]`, and `ARDA.hydrate` applying both at runtime (`L2683`, `L3082`) |
| 15 | Canvas RTL text drawn in logical order | `ARDA.canvas.drawBidiText(ctx, text, x, y, opts)` implementing the exact 4-step procedure: `g.direction` → `g.textAlign` → mirrored `nx` → forced LTR for Latin substrings (`L2898–2933`) |
| 16 (`D-1`) | switcher positioned with physical `left` | logical `inset-inline-start` everywhere — zero physical `left:` in the bundle |
| 17 (`D-2`) | hardcoded English `aria-label` | localized `aria-label` from `core.ui_language` in all 20 locales |
| 18 (`D-3`) | duplicate menu builds on re-mount | single idempotent builder guarded by `dataset.ardaSwitcher === "on"` |
| 19 (`D-6`) | color-only active-locale marker (WCAG 1.4.1 fail) | visible `✓` glyph colored `--gold` alongside `aria-selected` |
| 20 | lost roving focus | `ArrowDown/ArrowUp/Home/End/Escape` with `focus({preventScroll:true})` |
| 21 | scroll-jump on open | `preventScroll:true` on every focus transfer |
| 22 | menu never dismissed | outside `pointerdown` dismissal |
| 23 | touch targets too small | 44px coarse-pointer targets kept while the trigger keeps Fishkal's 40px glass-pill geometry |
| 24 | flag badge rotation leaks | defensive caret-rotation override preserved verbatim: `transform:none !important; transition:none; stroke:none;` |
| 25 | double injection | `data-arda-runtime="on"` idempotency gate — `inject_arda_runtime` is a no-op on already-armed pages |

## Circular Mini-Flag Badges
- `render_circular_flag_badge(code)`: upgrades Fishkal's `.lf` badge to a **circular** badge — `--flag-badge-radius: 50%`; `width/height: 20px`; `border-radius: 50%`; `overflow: hidden`; inline-flex centering; `box-shadow: 0 0 0 1px rgba(239,230,214,.28)` (`--cream` ring on the dark marketplace).
- Product flow: `detect_origin_locale` reads the origin language from the artifact (root `lang` attribute → conclusive script glyphs: Urdu-specific `\u{0679} \u{0688} \u{0691} \u{06BA} \u{06D2} \u{06BE} \u{06C1}` before Persian-specific `\u{067E} \u{0686} \u{0698} \u{06AF} \u{06A9}` → Arabic block → Hebrew → Devanagari → Thai → Kana → Hangul → Cyrillic → CJK → Vietnamese markers → `en`) and stamps the **Origin Circular Flag Badge** (e.g. a Persian accounting calculator ships with the circular Iran flag + `'\u{0641}\u{0627}'`).
- `render_product_flag_strip(origin, translated)`: renders the origin badge + one circular mini-flag per translated locale for the marketplace card and the in-app language switcher (origin dedup, unknown codes dropped, deterministic order).

## The 24-hour cron loop — `ArdaPlanetaryCronDaemon`
- Interval locked at `ARDA_CRON_INTERVAL_SECS = 86_400` (exactly 24h).
- Every tick scans the product queue (products + `0xKIA` platform views) with a `sha256_content_delta` guard (`arda_content_sha256`): unchanged content is **skipped idempotently**; new or updated content is localized across the 20 planetary locales, injected with the Fishkal runtime, and stamped with circular flag badges (`ArdaCronTickReport { scanned, skipped_unchanged, localized, localized_ids }`).
- Test-pinned: second tick over identical content reports `localized == 0, skipped_unchanged == n`.

## Mairon's Autonomous `SKILL.md` Forge — `MaironSkillForge`
- Wired to the REA forge gate: whenever a product clears Mairon's gate (`rea_score >= MARKETPLACE_APPROVAL_THRESHOLD = 85`), `MaironSkillForge::forge_skill_package` deterministically emits:
  1. **`SKILL.md`** — YAML frontmatter (`name`, `slug`, `version`, `rea_score`, `mairon_certified: true`, `origin_locale`, `translated_locales`) + Purpose + Invocation Contract (the three live endpoints) + Input/Output Schema + Verification Gates (REA bar, parity `1.00`, `25/25 cured defects`, content `sha256`).
  2. **`skill.json`** — structured manifest compatible with the Hermes / MCP / OpenCode skill registries (`registry.compatible`, `invocation`, `input_schema`, `output_schema`, `verification_gates`, `circular_flag_strip_html`).
- Below the gate the forge refuses (`SkillForgeError::BelowReaGate`) — a sub-threshold artifact can never become a skill. The marketplace `GET /skill` endpoint judges **live** (`MaironSovereignJudge::judge_market_product`) and auto-forges + stamps on the fly (`forged_on_the_fly = true`); the stamped package is then served byte-identical forever.

## Live HTTP surface (Wave 4)
| Endpoint | Auth | Contract |
| :--- | :--- | :--- |
| `GET /api/v1/arda/flags` | public | 20-locale Fishkal vector flag & circular badge catalog (`ArdaFlagsCatalogResponse`) |
| `POST /api/v1/arda/localize-now` | session | ARDA localization + Fishkal runtime injection on raw HTML or `product_id` (owner-only stamp), returns localized bundle + circular flag strip + parity report + content `sha256` |
| `GET /api/v1/market/products/{id}/skill` | public | product's `SKILL.md`, `skill.json`, circular flag strip — auto-forged on the fly when the live Mairon judgment clears `≥ 85`, `422` below the gate |

## Persistence — migration `0008_arda_locales_and_skill_forge.sql`
- `"Product"` gains `originLocale TEXT NOT NULL DEFAULT 'en'`, `supportedLocalesJson TEXT NOT NULL DEFAULT '["en","fa","ar"]'`, `skillMd TEXT`, `skillJson TEXT`, `ardaLastLocalizedAt TEXT` — all 20 existing product rows preserved (safe defaults, same contract as `0005`/`0007`).
- `stamp_product_arda` writes the locale stamp + skill package (`COALESCE` preserves an existing forge); `crates/db/build.rs` pins `cargo:rerun-if-changed=migrations` so `sqlx::migrate!` can never expand stale schema.

## Kernel engineering contract
- Zero `unsafe` (`#![forbid(unsafe_code)]` across the workspace), zero Clippy warnings (`rustc 1.100.0`, `-D warnings`).
- Everything deterministic: origin detection, catalog parity, bundle generation (bit-identical for identical input — `content_sha256`-pinned), injection, cron ticks, skill forging — no hidden clocks, no IO in the engine core.
- The generated runtime JS is node-syntax-checked and carries every cure marker; the 20 SVGs keep `viewBox="0 0 20 14"`.
- Tests: 13 engine tests (verbatim SVGs, parity validator, bundle markers, injection idempotency, cron delta guard, skill forge gate) + 7 live endpoint tests (catalog, injection, owner-only stamping, forge-on-the-fly, 422 below gate, 404) ride on top of the 560-test kernel (583+ total, 0 failures).

## Planetary flow
```text
product HTML (e.g. Persian accounting calculator)
   │
   ▼
detect_origin_locale ──► origin = fa ──► ⭕ circular Iran badge + '\u{0641}\u{0627}'
   │
   ▼
ArdaFishkalBundleGenerator.generate(fa, [19 targets])
   │  unified 6-scope catalog · parity 1.00 · cached Intl · bidi 4-step
   │  WAI-ARIA listbox (D-1..D-8 cured) · atomic 3-attr root flip
   ▼
inject_arda_runtime ──► idempotent <style>+<script> before </head>
   │
   ▼
render_product_flag_strip ──► ⭕fa ⭕en ⭕ar … on the marketplace card
   │
   ▼
ArdaPlanetaryCronDaemon (24h) ──► sha256 delta guard ──► re-localize only deltas
   │
   ▼
Mairon REA gate ≥ 85 ──► MaironSkillForge ──► SKILL.md + skill.json
   │                                          (Hermes / MCP / OpenCode registries)
   ▼
⚒️ sovereign product, planetary by default
```
