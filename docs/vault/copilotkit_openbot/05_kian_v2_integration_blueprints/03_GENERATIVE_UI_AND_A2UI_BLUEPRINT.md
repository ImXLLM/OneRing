# Sandboxed Generative UI, Idiomorph Streaming & A2UI Recovery Blueprint for KIAN v2

## Source Gems
- `04_generative_ui_and_canvas_components/OpenIntelligentUI_generative_ui/open-generative-ui/` (`process-partial-html.ts`, `frame-content.ts`, `idiomorph-inline.ts`)
- `03_ag_ui_protocol_and_schemas/typescript/a2ui-toolkit/src/recovery.ts` & `validate.ts`
- `03_ag_ui_protocol_and_schemas/openbot_a2ui_catalog.ts` & `02_openbot_prompts_and_skills/generative-ui-design-skill.ts`
- `01_openbot_core/app_copilot/sandboxed-tools.tsx` & `01_openbot_core/components_gallery/charts.tsx` (`provenance`)

## Where to Inject in KIAN v2
1. **Live Zero-Flicker Artifact Preview (`apps/web/app/builder` & `apps/miniapp`)**:
   - Use `processPartialHtml` + `Idiomorph.morph` inside a CSP-locked `<iframe sandbox="allow-scripts">` (allowlist: `cdnjs.cloudflare.com`, `esm.sh`, `cdn.jsdelivr.net`, `unpkg.com`, zero same-origin access) for zero-flicker streaming HTML/SVG/JS previews.
   - Inject `GENERATIVE_UI_DESIGN_SKILL` and `master-agent-playbook.txt` into KIAN's builder system prompt so generated widgets automatically adapt to dark/light mode and narrow mobile/miniapp columns.
2. **A2UI Validation & Self-Healing Retry Loop (`crates/ai`)**:
   - Port `runA2UIRecovery` (`validateA2UIComponents` -> feed `[code] path: message` back into prompt up to 3 attempts) so malformed declarative UI JSON self-heals before reaching the user.
3. **Mandatory Provenance on Charts (`components/gallery/charts.tsx`)**:
   - Require `basis: "read" | "given" | "own-knowledge"` + `sources` + `asOf` on generated charts/KPIs so users immediately distinguish verified tool data from LLM estimates.
