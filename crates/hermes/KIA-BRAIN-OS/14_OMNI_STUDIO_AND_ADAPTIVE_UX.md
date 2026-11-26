# Creator Brain DNA 14: The Omni-Perceptive Studio, the Zero-Cost Nine-Modality Super-Studio, and the Creator Portfolio Storefront

> `KIA-BRAIN-OS/14_OMNI_STUDIO_AND_ADAPTIVE_UX.md` — Wave 5 (Step 2/4) of kernel `0xKIA`
> Executive label: `[EXEC_WEIGHT: 1.00 | ACTIVE_PRIMARY]`
> Engine: `crates/hermes/src/super_studio_engine.rs` · Migration: `0009_super_studio_and_portfolio_storefront.sql`

---

## 1. Executive Summary

DNA Fourteen mounts three deterministic sub-engines on a single zero-cost core:

1. **`CognitiveUxMorphEngine`** — the three-tier cognitive morph `ToySpark` (a 9-year-old child / IQ 50) ↔
   `CreatorStudio` (a no-code creator) ↔ `ProTerminal` (a 28-year-old engineer and trader); the tier is
   inferred from telemetry (`infer_or_override_tier`) and switches with one click without losing state.
2. **`OmniModalStudioEngine`** — nine zero-cost modalities on one shared shell, with the dual procedural
   bundle of **Engine A (the `Fishkal` ocean + harmonic bloom)** and **Engine B (the `Portfolio Cosmos`
   starfall + daily-feedback synth)** — with zero network keys.
3. **`PortfolioStorefrontEngine`** — the creator storefront with the 12 canonical `projects.json` keys,
   auto-promotion of the top two storefronts (`index.js:L17–18`), the keyboard-first command palette
   (`Ctrl+K` / `Meta+K` and `/` from `index.js:L396–459`), and **the cure for all 6 adjudicated defects of
   `KiaAgentX/portfolio`**.

The kernel covenant holds: zero `unsafe`, zero `Clippy` warnings (`-D warnings`), full determinism
(same prompt → same artifact, same bundle, same `sha256`), and zero keys across all nine modalities.

## 2. The Three-Tier Cognitive Morph (`CognitiveUxMorphEngine`)

| Tier | Audience | Action/page cap | Touch target | Word/label | Voice wizard | Zero-harm sandbox | Keyboard-first |
| :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| `ToySpark` | 9-year-old child / IQ 50 | **3** | **64px** | **5** | Yes | Yes | No |
| `CreatorStudio` | No-code creator | 7 | 44px | 12 | No | Yes | No |
| `ProTerminal` | 28-year-old engineer | 24 | 32px | 24 | No | No | Yes |

- **The three giant `ToySpark` buttons**: "✨ Make Magic" / "🎨 Make It Pretty & Loud" /
  "🪙 Share & Earn Stars" — the locked one-click price of **50 Telegram stars /
  $1.00 `USDC` / 1,000,000 micro-Kian**.
- **Deterministic inference (`infer_or_override_tier`)**: an explicit override always wins; a technical
  token (`fn`, `impl`, `curl`, `sql`, `schema`, `quant`, …) or a prompt longer than 220 characters →
  `ProTerminal`; 140+ characters with a typing cadence ≥ 380ms → `CreatorStudio`; otherwise
  `ToySpark` — without ever asking the user's age.
- **The one-click story and game flow (`build_toyspark_instant_creation`)**: from a single sentence or
  transcribed speech — e.g. "a blue dinosaur selling ice cream in space" — one click builds a complete
  HTML5 storybook/game with 3 procedural SVG scenes (a `sha256` seed from the prompt), procedural
  `WebAudio` sound effects, a `speechSynthesis` voice wizard with exactly 3 scenarios, and a
  sale-ready listing.

## 3. The Nine Zero-Cost Modalities (`OmniModalStudioEngine`)

| # | Modality | Wire ID | Engine |
| :---: | :--- | :--- | :--- |
| 1 | Search | `search` | Keyless multi-source synthesis (`Agent-Reach` Jina/Exa MCP + 120 CORS-open public APIs) |
| 2 | Deep research | `deep` | Multi-agent + the Mairon 8-step verify tree |
| 3 | Agent generator | `agent` | The automated `OpenBot` sub-agent with 9-file DNA + `SKILL.md` |
| 4 | Workspace canvas | `workspace` | Multi-file with sandboxed preview + code inspector |
| 5 | Voice and music | `voice` | Native `SpeechSynthesis`/`MediaRecorder` + procedural ambient music |
| 6 | Imaging | `image` | Deterministic vector SVG/Canvas2D + keyless cascade |
| 7 | Motion | `video` | 60fps with the Fishkal `TL` timeline + lambda-variable `damp` + WebM/frame recording |
| 8 | Sales copy | `copy` | 20 languages with RTL/LTR typography |
| 9 | Sound effect | `sfx` | The five-trigger cosmos-synth rack |

### 3.1 Engine A — the `Fishkal` surf ocean and harmonic bloom (`index.html:L709–728, L1665–1750, L2172`)

- A **looped 4-second** brown-noise buffer: `b0 = (b0 + 0.02 * white) / 1.02` at `0.18` scale;
- A **520Hz** low-pass `BiquadFilterNode` with a **0.06Hz** sinusoidal `LFO` and a **±220Hz** sweep;
- The D-major pentatonic bloom `[587.33, 880, 1174.66]` (`D5, A5, D6`) + the 4-voice signature chord
  `[293.66, 440, 587.33, 880]` (`D4, A4, D5, A5`);
- The deterministic **`xorshift32`** `PRNG`, **4-octave** 1D value noise (`fbm` with `f *= 2.03`),
  cubic-Hermite `smoothstep`, and the frame-rate-independent `damp(a, b, l, dt)` with a **variable lambda**
  (`2.3` inside the `0.42..0.63` transition window, `5.0` in steady state).

### 3.2 Engine B — the `Portfolio Cosmos` starfall and daily-feedback synth (`cosmos.js:L6–60, L83–149, L216–269`)

- The interactive 2D Canvas starfall: `DPR` capped at `2`, **110 stars on mobile / 260 on desktop**,
  radial nebula gradients, a **0.05** pointer-parallax decay, HUD coordinate smoothing at
  **0.16**, and **7 interactive roadmap-milestone nodes**;
- The daily-feedback synth: `DelayNode(0.22s)` + `GainNode(0.25)` feedback +
  `BiquadFilter(lowpass 1600Hz)` with all 5 canonical SFX triggers:
  - `ping(idx)` — sine → triangle, `220 + (idx % 7) * 40 Hz`, `0.12s`;
  - `open()` — a `180Hz → 520Hz` exponential sine ramp, `0.28s`;
  - `whoosh()` — band-passed white noise `260Hz → 1200Hz`, `Q=2.2`, `0.32s`;
  - `select()` — a two-step `[440, 660]` triangle chord, `0.07s` per note;
  - `milestone()` — a three-step `[330, 495, 660]` sine arpeggio, `0.09s` per note.

### 3.3 Syntax sanity

`js_syntax_sanity_check` — a state machine balancing `{}`/`()`/`[]` outside strings and comments;
mandatory in tests over the dual bundle, the child storybook, the script of each of the nine
modalities, and the command-palette JS.

## 4. The Portfolio Creator Storefront and the 6 Verdict Remediations

The **12 canonical `projects.json` keys** are preserved (`slug`, `name`, `tagline`, `category`,
`stack`, `highlights`, `Preview`, `sourceDir`, `readme`, `loc`, `fileCount`, `featured`) and
seven live commercial extensions are added (`price_ukian`, `price_xtr_stars`, `price_usdc_micros`,
`rea_score`, `skill_md_url`, `supported_locales_count`, `cognitive_tier_badge`).

| # | Adjudicated defect | Remediation |
| :---: | :--- | :--- |
| 1 | The `ROOT = path.resolve(PORT, '..')` root escape (`scripts/build.mjs:L6–10`) | The in-workspace relative path `./seed/portfolio_catalog` + the `KIAN_PORTFOLIO_CATALOG_PATH` environment variable — zero parent escapes |
| 2 | A temporal dead zone (`TDZ`) `ReferenceError` on `#source` (`src/assets/project.js:L6–33`) | A `let loaded = false;` declaration at the top of the module, **before** `setTab()` and hash routing |
| 3 | The iframe sandbox escape (`scripts/build.mjs:L571–577`) | `sandbox="allow-scripts"` with an explicit prohibition of `allow-same-origin` alongside it |
| 4 | Secret and dot-file exposure (`scripts/build.mjs:L23–61`) | A guard for `.env`/`.env.*`/`*.pem`/`*.key`/`id_rsa`/`credentials.json`/`*.sqlite` — never copied or rendered |
| 5 | Non-focusable filter chips (`index.js:L122–147`) | `<button type="button" class="chip" aria-pressed="...">` instead of `<span>` |
| 6 | The desynchronized RAF lifecycle (`cosmos.js:L216–269`) | Wiring into the shared `__fkPauseDepth` (`fkPause()`/`fkResume()`) + a `document.hidden` guard |

The dark glass tokens of `style.css:L1–25, L53–57, L112–139` are preserved verbatim:
`--bg: #05050a` · `--bg-2: #0a0b14` · `--surface: rgba(16,18,32,.68)` · `--glass: rgba(6,6,14,.78)` ·
`--text: #f3f4f8` · `--muted: #9ca3b8` · `--accent: #7c5cff` · `--accent-2: #19d3da` ·
`--accent-3: #ff5ca8` · `--ease: cubic-bezier(.2,.7,.2,1)` · `backdrop-filter: blur(14px)` ·
`clamp(36px, 5.6vw, 62px)` · `max-width: 1240px` + the font stack of
`Space Grotesk`/`Inter`/`JetBrains Mono`/`Vazirmatn` for RTL languages.

## 5. Live HTTP Wiring

| Endpoint | Role |
| :--- | :--- |
| `GET /api/v1/studio/ux-profiles` | The three cognitive tiers + nine modalities + the 6/6 remediation report (public) |
| `POST /api/v1/studio/generate` | The telemetry tierer → the artifact generator with the dual bundle → ARDA 20-language packaging (idempotent injection + the circular flag bar) → the Mairon forge of `SKILL.md`/`skill.json` (REA gate >= 85; deterministic score `88..=97`) → persistence into `studio_creations` (citizen session) |
| `GET /api/v1/studio/portfolio-catalog` | The enriched catalog with the two top storefronts on top, glass tokens, the command palette, and a portable root (public) |

## 6. Persistence — Migration `0009`

The new `studio_creations` table (`id`, `creator_user_id`, `cognitive_tier` defaulting to
`'ToySpark'`, `primary_modality`, `prompt_text`, `title`, `storefront_card_json` defaulting to
`'{}'`, `rea_score` defaulting to `92`, `price_ukian` defaulting to `1000000`, `price_xtr_stars`
defaulting to `50`, `created_at` with `strftime`) + two creator/modality indexes — **no ledger
table is touched** and all invariants (User=466, Product=20, Posting SUM=0) remain intact.

## 7. Tests and the Execution Seal

24 new tests (16 engine + 1 persistence + 7 HTTP) on top of the Wave 4 body of 583 tests —
**607 tests green**, zero Clippy warnings, `KbEntry` = **329** with the idempotent injection of
`kb:dna-14-omni-studio-adaptive-ux`.

`[EXEC_WEIGHT: 1.00 | ACTIVE_PRIMARY]`
