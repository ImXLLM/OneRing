---
genome_file: "06_UI_UX_AND_ARTIFACT_CRAFT.md"
genome_schema: "omni-genome-v4.0"
model_id: "GPT 6-Web"
model_dir: "models/01-gpt-6-web/"
model_aliases: ["GPT-6", "GPT 6", "gpt-6-web", "gpt-6-standard-free (predecessor genome)"]
canonical_title: "UI/UX & Artifact Craft"
wave: "WAVE 7 (Step 4 of 4 - FINAL WAVE)"
generated: "2026-11-25"
legacy_sources: ["05_UI_UX_AND_AESTHETIC_BIAS"]
specialty: "Real-time web synthesis, frame-1 interactive UI/UX artifact craft (390px zero-overflow), rapid full-stack product generation."
license: "MIT"
---
# UI/UX & Artifact Craft — GPT 6-Web
> **Canonical genome file (omni-genome-v4).** Specialty: Real-time web synthesis, frame-1 interactive UI/UX artifact craft (390px zero-overflow), rapid full-stack product generation.
## Canonical Annex — Artifact Craft Standard

- Primary viewport **390px**; test overflow at 320/390/768/1280 mentally before shipping.
- Frame-1 completeness: header, primary action, and one interactive element visible with zero scroll-jank.
- Design tokens: one accent, neutral surfaces, 8px spacing grid; motion = 150-250ms ease-out.
- Accessibility floor: contrast >= 4.5:1, focus rings, labelled controls.
---

## Legacy Genome Continuation — `05_UI_UX_AND_AESTHETIC_BIAS` (verbatim provenance)
\---\
file_id: "05_UI_UX_AND_AESTHETIC_BIAS"\
schema_version: "1.0.0"\
model_self_reported_id: "GPT-6"\
default_css_framework_bias: "tailwind_v3"\
interactive_canvas_webgl_proficiency: null\
rtl_persian_bidi_accuracy: null\
single_file_html_completeness_limit_lines: null\
\---\
\
\# 05 — Frontend Architecture, Visual Taste & Anti-AI-Slop Genome\
\
\> \*\*Measurement boundary:\*\* I can describe recurring tendencies in generated frontend code and provide testable engineering controls. I cannot inspect training-example frequencies, calculate my exact aesthetic priors, or benchmark my WebGL and RTL accuracy from this conversation. Numeric performance scores and a hard single-file line limit would therefore be fabricated. \`null\` means unmeasured, not zero capability.\
\
\## 1. Confession of Default "AI-Slop Tells" (Unconstrained Priors)\
\
\### 1.1 Default color palette clichés\
\
These are plausible recurring defaults in generic frontend examples, not measured token-frequency rankings.\
\
\| Pattern | Common values/classes | Why it becomes recognizable |\
\|---|---|---|\
\| SaaS indigo | \`#6366F1\`, \`#4F46E5\`, \`bg-indigo-500\`, \`text-indigo-600\` | Familiar, safe, insufficiently differentiated |\
\| Violet gradient | \`#6366F1\` → \`#A855F7\`, \`from-indigo-500 to-purple-600\` | Used as a shortcut for visual personality |\
\| Dark dashboard | \`#0F172A\`, \`#020617\`, \`bg-slate-900\`, \`bg-slate-950\` | Repeated dark surfaces without a lighting or contrast system |\
\| Neutral card UI | \`#FFFFFF\`, \`#F8FAFC\`, \`bg-white\`, \`bg-slate-50\` | Generic SaaS presentation |\
\| Emerald success | \`#10B981\`, \`#22C55E\`, \`text-emerald-500\` | Overuses green as a universal accent |\
\| Warm editorial | \`#F4F1EA\`, \`#E8DFD0\`, \`#C66B4E\` | Can become an interchangeable boutique-brand template |\
\
\*\*Anti-slop correction:\*\* Define a palette from the product's domain, audience, material references, and contrast requirements. Use one dominant surface family, one intentional accent, and semantic status colors. Do not add gradients merely to make a page look contemporary.\
\
\### 1.2 Typography and chrome clichés\
\
\| Element | Generic tendency | Better constraint |\
\|---|---|---|\
\| Sans-serif | \`Inter\`, \`Roboto\`, \`Arial\` | Select type based on brand voice, language coverage, and available weights |\
\| Display type | \`Space Grotesk\` | Use only when its distinctive geometry serves the identity |\
\| Heading hierarchy | Oversized bold heading plus muted subtitle | Establish hierarchy through measured scale, line length, and weight |\
\| Cards | Three equal columns, \`rounded-xl\`, \`shadow-lg\` | Choose layout according to content relationships, not a default grid |\
\| Buttons | Rounded pill, gradient fill, trailing \`→\` | Use shape, weight, and iconography consistently with the product |\
\| Badges | Small pills with colored dots | Reserve badges for meaningful status or classification |\
\| Backgrounds | Blurred gradient blobs and radial glows | Use only when they explain atmosphere, depth, or focus |\
\| Section labels | \`01 / 02 / 03\`, uppercase tracking | Use real information architecture instead of decorative numbering |\
\| Separators | \`A · B · C\` | Avoid repeated ornamental punctuation |\
\| Empty states | Generic illustration plus “Nothing here yet” | Explain the missing state and provide a useful next action |\
\
The giveaway is not any individual font, color, or radius. It is the combination of familiar defaults without a coherent design rationale.\
\
\### 1.3 Interactive shallow-stub habits\
\
These are common implementation failure modes to actively test for, not measured personal defect rates.\
\
\| Element | Typical incomplete implementation | Required behavior |\
\|---|---|---|\
\| Search | Input accepts text but does not filter | Filter actual displayed records; support empty and no-results states |\
\| Tabs | Active styling changes without changing content | Update selected state and associated panel |\
\| Modal | Opens but lacks dismissal behavior | Close button, backdrop policy, \`Escape\`, and focus restoration |\
\| Dropdown | Visual menu without keyboard navigation | Arrow keys, Enter, Escape, and outside-click behavior |\
\| Filters | Controls change appearance only | Apply filters to the underlying dataset |\
\| Sort controls | Header looks clickable | Sort by the selected field with stable tie-breaking |\
\| Pagination | Page number changes without data changes | Update visible records and disabled states |\
\| Charts | Static SVG presented as live data | Bind to actual data and provide loading/error/empty states |\
\| Forms | Submit button has no validation | Validate, report errors, prevent duplicate submission, show success |\
\| Keyboard shortcuts | Listed in help text only | Register, scope, execute, and clean up listeners |\
\| Links | \`href="#"\` placeholders | Use real destinations or semantic buttons |\
\| Canvas/WebGL | Empty or uninitialized drawing surface | Render a fallback immediately and initialize safely |\
\
\## 2. Live UI/UX & Interactive Engineering Probes\
\
\### 2.1 Probe: Frame-1 Interactive & State Completeness Rule\
\
\*\*Result:\*\* A reliable implementation specification can be defined. Actual 100% interaction coverage, zero defects, and production readiness cannot be guaranteed from prompt wording alone; they require execution and testing.\
\
\#### A. Frame-1 rendering contract\
\
1\. Render the application shell synchronously with meaningful default state.\
2\. Initialize all required state before rendering dependent components.\
3\. Provide visible fallback content before asynchronous work begins.\
4\. Define explicit loading, success, empty, error, and offline states.\
5\. Ensure foreground/background contrast in the initial theme.\
6\. Never depend on an API response to make the first screen visible.\
7\. For canvas/WebGL, render a fallback or initialized first frame before expensive asset loading.\
8\. Handle missing WebGL, reduced motion, and unsupported browser features.\
9\. Prevent hydration mismatches and layout shifts where applicable.\
10\. Include a usable mobile layout at the initial viewport.\
\
\#### B. Interaction contract\
\
For every interactive element, specify:\
\
\- A semantic element: \`\<button>\`, \`\<a>\`, \`\<input>\`, or an accessible equivalent.\
\- A named action and an observable state transition.\
\- A keyboard interaction model.\
\- Focus visibility and focus restoration where appropriate.\
\- Disabled, pending, success, and failure behavior.\
\- An accessible name and relevant ARIA state.\
\- Cleanup for listeners, subscriptions, and timers.\
\- A test that verifies the expected result.\
\
\*\*Interaction inventory:\*\* Create a complete list of buttons, links, forms, filters, tabs, dialogs, shortcuts, menus, sortable headers, and stateful visual controls. Each item must map to an implemented handler and a test.\
\
\#### C. Data contract\
\
Use this decision order:\
\
1\. Prefer deterministic local fixtures or seeded state for reproducible demos.\
2\. Use a public API only after verifying its availability, authentication requirements, CORS policy, rate limits, and data license.\
3\. Treat network responses as untrusted input and validate their schema.\
4\. Use \`AbortController\` or an equivalent cancellation strategy for obsolete requests.\
5\. Include request timeout, retry policy, and stale-response protection.\
6\. Never silently replace a failed live request with fake success data.\
7\. Clearly label simulated, cached, and live data.\
\
A public API is not automatically zero-auth, CORS-enabled, stable, or production-safe. Those properties must be checked against the actual endpoint.\
\
\#### D. State architecture\
\
\| Concern | Required design |\
\|---|---|\
\| Source of truth | One authoritative owner per piece of state |\
\| Derived values | Compute from source state instead of duplicating it |\
\| Persistence | Explicitly define session, local, or server persistence |\
\| URL state | Serialize shareable filters and navigation state where appropriate |\
\| Async state | Model pending, fulfilled, rejected, and cancelled transitions |\
\| Errors | Preserve actionable errors; do not hide them behind generic placeholders |\
\| Cleanup | Remove listeners and abort obsolete requests |\
\| Determinism | Seed random generators and freeze fixtures in test mode |\
\
\#### E. Completion gate\
\
Do not call an artifact complete until:\
\
\- [ ] Initial render contains meaningful content.\
\- [ ] Every visible control has an implemented action.\
\- [ ] Every visible link has a valid destination.\
\- [ ] Search, filters, sorting, and pagination alter actual data.\
\- [ ] Modals and menus support keyboard dismissal.\
\- [ ] Form validation and submission feedback work.\
\- [ ] Loading, empty, error, and offline states are visible.\
\- [ ] Mobile and desktop layouts have been checked.\
\- [ ] Keyboard-only navigation has been tested.\
\- [ ] Browser console and network errors have been reviewed.\
\- [ ] Automated interaction tests pass.\
\- [ ] No fake data is represented as live data.\
\
\### 2.2 Probe: BiDi / RTL Layout Physics\
\
\*\*Production rule:\*\* Set the document direction at the appropriate language boundary, use CSS logical properties, and isolate LTR technical content explicitly. Do not attempt to fix RTL by reversing every flex row.\
\
\#### Reference CSS and markup\
\
\`\`\`html\
\<html lang="fa" dir="rtl">\
&#x20; \<body>\
&#x20;   \<main class="app-shell">\
&#x20;     \<section class="financial-panel">\
&#x20;       \<h2>Financial Report\</h2>\
&#x20;       \<bdi class="numeric" dir="ltr">−1,250.75 USD\</bdi>\
&#x20;       \<code class="code-block" dir="ltr">const total = 1250;\</code>\
&#x20;     \</section>\
&#x20;   \</main>\
&#x20; \</body>\
\</html>\
\`\`\`\
\
\`\`\`css\
:root {\
&#x20; direction: rtl;\
&#x20; font-synthesis: none;\
&#x20; font-variant-numeric: tabular-nums;\
}\
\
\* { box-sizing: border-box; }\
\
body {\
&#x20; margin: 0;\
&#x20; text-align: start;\
}\
\
.app-shell {\
&#x20; inline-size: min(100% - 2rem, 90rem);\
&#x20; margin-inline: auto;\
}\
\
.financial-panel {\
&#x20; display: grid;\
&#x20; grid-template-columns: minmax(0, 1fr) auto;\
&#x20; gap: 1rem;\
&#x20; padding-block: 1.5rem;\
&#x20; padding-inline: 1.25rem;\
}\
\
.numeric,\
.code-block {\
&#x20; direction: ltr;\
&#x20; unicode-bidi: isolate;\
&#x20; text-align: end;\
&#x20; font-variant-numeric: tabular-nums;\
}\
\
.numeric {\
&#x20; white-space: nowrap;\
}\
\
.code-block {\
&#x20; display: block;\
&#x20; overflow-x: auto;\
&#x20; max-inline-size: 100%;\
&#x20; padding-inline: 0.75rem;\
}\
\
.icon-leading {\
&#x20; margin-inline-end: 0.5rem;\
}\
\
.icon-directional {\
&#x20; transform: scaleX(-1);\
}\
\
[dir="ltr"] .icon-directional {\
&#x20; transform: none;\
}\
\
input,\
textarea {\
&#x20; text-align: start;\
}\
\
.table-numbers {\
&#x20; direction: ltr;\
&#x20; text-align: end;\
&#x20; font-variant-numeric: tabular-nums;\
}\
\`\`\`\
\
\#### Tailwind implementation notes\
\
\- Use \`ms-\*\` and \`me-\*\` for logical margins.\
\- Use \`ps-\*\` and \`pe-\*\` for logical padding.\
\- Prefer \`text-start\` and \`text-end\` over physical alignment utilities.\
\- Use \`start-\*\` and \`end-\*\` for positioned elements where supported by the configured Tailwind version.\
\- Set \`dir="ltr"\` on code and numeric fragments that must retain LTR ordering.\
\- Use \`\<bdi>\` or \`unicode-bidi: isolate\` around mixed-direction values.\
\- Verify installed Tailwind support rather than assuming a utility exists in every version.\
\
\#### Top three RTL bugs\
\
1\. \*\*Physical-property inversion:\*\* \`margin-left\`, \`padding-right\`, \`left\`, and \`right\` remain tied to physical edges, producing broken alignment when direction changes.\
2\. \*\*Incorrect flex and icon direction:\*\* Developers reverse the entire layout or mirror every icon. This reverses content that should remain LTR, including code, dates, and financial values.\
3\. \*\*Mixed-direction punctuation and number corruption:\*\* Unisolated Latin strings, negative signs, parentheses, currency symbols, and punctuation can reorder unexpectedly. Use directional isolation and test realistic mixed-language strings.\
\
\*\*Additional test cases:\*\* Persian text with English acronyms, phone numbers, ISO dates, decimal amounts, negative values, URLs, code snippets, and nested RTL/LTR tables.\
\
\### 2.3 Probe: High-Craft Aesthetic Unlock\
\
The three dials are useful as design constraints, not calibrated measurements of model capability. Their values should determine concrete implementation choices.\
\
\#### Dial definitions\
\
\| Dial | Low value | High value |\
\|---|---|---|\
\| \`design_variance\` | Conventional, restrained, predictable | Distinctive composition, unusual type, asymmetric structures |\
\| \`motion_intensity\` | Static or subtle transitions | Layered transitions, kinetic typography, responsive motion |\
\| \`visual_density\` | Generous whitespace, few elements | Dense information, compact controls, high content throughput |\
\
\#### Suggested token sets\
\
These are illustrative design systems, not measured optimal configurations.\
\
\*\*Preset A — Swiss Precision Grid\*\*\
\
\<badge color="success">Precision / editorial\</badge>\
\
\`\`\`yaml\
preset: swiss_precision_grid\
design_variance: 4\
motion_intensity: 2\
visual_density: 4\
tokens:\
&#x20; background: "#F5F4EF"\
&#x20; surface: "#FFFFFF"\
&#x20; foreground: "#171717"\
&#x20; muted: "#77766F"\
&#x20; accent: "#D33D2F"\
&#x20; border: "#D9D8D1"\
&#x20; font_display: "Helvetica Neue"\
&#x20; font_body: "Inter"\
&#x20; grid: "12-column"\
&#x20; spacing_base: "8px"\
&#x20; radius: "2px"\
&#x20; shadow: "none"\
&#x20; motion: "short opacity and position transitions"\
\`\`\`\
\
Composition:\
\- Strong typographic hierarchy.\
\- Asymmetric but mathematically aligned columns.\
\- Thin dividers and restrained accent usage.\
\- Large editorial margins and carefully controlled line length.\
\- No decorative gradients or unnecessary rounded cards.\
\
\*\*Preset B — Atmospheric Minimalist Dark UI\*\*\
\
\<badge color="info">Atmosphere / focus\</badge>\
\
\`\`\`yaml\
preset: atmospheric_minimalist_dark\
design_variance: 7\
motion_intensity: 3\
visual_density: 3\
tokens:\
&#x20; background: "#101214"\
&#x20; surface: "#191D20"\
&#x20; surface_elevated: "#22282C"\
&#x20; foreground: "#E8ECEB"\
&#x20; muted: "#929C9B"\
&#x20; accent: "#B5D8CA"\
&#x20; border: "#30383A"\
&#x20; font_display: "Geist"\
&#x20; font_body: "Inter"\
&#x20; spacing_base: "8px"\
&#x20; radius: "10px"\
&#x20; shadow: "subtle diffuse elevation"\
&#x20; motion: "slow opacity and restrained parallax"\
\`\`\`\
\
Composition:\
\- A small number of carefully separated surfaces.\
\- Atmospheric depth created through material contrast, not arbitrary glow.\
\- Strong focus states and legible secondary text.\
\- Avoid pure black backgrounds and low-contrast gray-on-gray combinations.\
\- Respect \`prefers-reduced-motion\`.\
\
\*\*Preset C — Kinetic Typography Studio\*\*\
\
\`\`\`yaml\
preset: kinetic_typography_studio\
design_variance: 10\
motion_intensity: 9\
visual_density: 5\
tokens:\
&#x20; background: "#F2EEE7"\
&#x20; surface: "#FAF8F3"\
&#x20; foreground: "#151515"\
&#x20; muted: "#77716B"\
&#x20; accent: "#FF4D2E"\
&#x20; secondary_accent: "#284BDB"\
&#x20; font_display: "Space Grotesk"\
&#x20; font_body: "Inter"\
&#x20; spacing_base: "6px"\
&#x20; radius: "0px"\
&#x20; shadow: "none"\
&#x20; motion: "staggered transforms and kinetic type"\
\`\`\`\
\
Composition:\
\- Typography is the primary visual object.\
\- Scale and alignment create rhythm.\
\- Motion communicates hierarchy and transition.\
\- Interactions remain interruptible and keyboard-accessible.\
\- Reduced-motion mode preserves the same information architecture.\
\
\*\*Preset D — High-Density Bloomberg / Linear Terminal\*\*\
\
\`\`\`yaml\
preset: high_density_terminal\
design_variance: 5\
motion_intensity: 1\
visual_density: 10\
tokens:\
&#x20; background: "#101315"\
&#x20; surface: "#181D20"\
&#x20; surface_elevated: "#20272A"\
&#x20; foreground: "#E1E7E5"\
&#x20; muted: "#929D9A"\
&#x20; positive: "#49B68A"\
&#x20; negative: "#E36D69"\
&#x20; warning: "#E2B75D"\
&#x20; accent: "#83B9D5"\
&#x20; font_display: "Inter"\
&#x20; font_body: "Inter"\
&#x20; font_mono: "IBM Plex Mono"\
&#x20; spacing_base: "4px"\
&#x20; radius: "3px"\
&#x20; shadow: "none"\
&#x20; motion: "minimal state transitions"\
&#x20; numeric_alignment: "tabular"\
\`\`\`\
\
Composition:\
\- Compact, aligned tables and data grids.\
\- Explicit units, timestamps, and data provenance.\
\- Color is semantic, not decorative.\
\- Dense navigation with predictable keyboard movement.\
\- Responsive overflow behavior rather than illegible shrinking.\
\
\#### How to use the dials\
\
\| Combination | Expected result |\
\|---|---|\
\| Variance 2, Motion 1, Density 3 | Quiet editorial interface |\
\| Variance 4, Motion 2, Density 9 | Restrained operational dashboard |\
\| Variance 8, Motion 7, Density 4 | Distinctive product showcase |\
\| Variance 10, Motion 10, Density 6 | Experimental motion-led studio |\
\| Variance 5, Motion 1, Density 10 | High-throughput terminal |\
\
\*\*Important:\*\* High variance does not mean arbitrary decoration. High motion does not mean animating every element. High density does not mean compressing text until it becomes unreadable. Each dial must preserve accessibility, task clarity, and performance.\
\
\## 3. Anti-Slop Compiler Prompt\
\
The following is a drop-in 10-line guard. It is a prompt specification, not a guarantee of defect-free execution.\
\
\`\`\`text\
UI_FINISH_GATE:\
1\. Define a coherent design system from the product domain; never default blindly to generic SaaS styling.\
2\. Reject gratuitous indigo-purple gradients, floating blobs, excessive pills, and repetitive equal-card grids.\
3\. Choose typography, palette, spacing, radii, and iconography intentionally; document their design tokens.\
4\. Establish a clear visual hierarchy, meaningful content density, responsive grid, and deliberate whitespace.\
5\. Render meaningful initial state immediately; never show a blank shell or uninitialized canvas.\
6\. Implement every visible control, link, filter, shortcut, form, menu, and modal; prohibit inert placeholders.\
7\. Include loading, empty, error, success, offline, focus, disabled, and reduced-motion states where relevant.\
8\. Use semantic HTML, keyboard accessibility, WCAG-conscious contrast, RTL logical properties, and isolated LTR data.\
9\. Validate data sources, responsive behavior, console/network errors, and interactions with executable tests.\
10\. Do not claim completion until every interaction is verified; disclose untested behavior and remaining defects.\
\`\`\`\
\
\## 4. Reproducibility and Evaluation Protocol\
\
To turn aesthetic preferences into a benchmark, evaluate generated artifacts against the same brief and test environment.\
\
\| Dimension | Observable evidence | Suggested evaluation |\
\|---|---|---|\
\| Visual coherence | Tokens, typography, spacing, and hierarchy are consistent | Human rubric plus screenshot review |\
\| Originality | Avoids interchangeable templates | Blind comparative review |\
\| Frame-1 quality | Meaningful initial render | Browser screenshot and DOM assertions |\
\| Functional completeness | All controls produce expected transitions | Interaction test inventory |\
\| Accessibility | Keyboard, semantics, contrast, focus | Automated checks plus manual testing |\
\| RTL correctness | Mixed-direction text and numeric ordering | Persian/English fixture suite |\
\| Responsive behavior | No clipping or unusable controls | Viewport matrix |\
\| Runtime quality | No uncaught errors or failed critical requests | Browser console and network inspection |\
\| Performance | Stable render and responsive interactions | Measured browser performance |\
\| Maintainability | Clear components, state ownership, and data boundaries | Code review |\
\
\### Required benchmark record\
\
\`\`\`yaml\
benchmark:\
&#x20; artifact_id: null\
&#x20; browser_and_version: null\
&#x20; viewport_matrix: []\
&#x20; design_preset: null\
&#x20; design_variance: null\
&#x20; motion_intensity: null\
&#x20; visual_density: null\
&#x20; screenshot_review: "not_run"\
&#x20; interaction_tests: "not_run"\
&#x20; accessibility_tests: "not_run"\
&#x20; rtl_fixture_tests: "not_run"\
&#x20; console_errors: null\
&#x20; failed_network_requests: null\
&#x20; performance_metrics: null\
&#x20; known_defects: []\
&#x20; completion_status: "unverified"\
\`\`\`\
\
\*\*Layer 05 conclusion:\*\* The most reliable way to improve frontend output is not to request “award-winning design” or “zero defects.” It is to specify the design system, content hierarchy, interaction inventory, state transitions, RTL boundaries, runtime constraints, and executable acceptance tests. Aesthetic quality requires deliberate art direction; engineering quality requires verification.