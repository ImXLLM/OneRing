# HITL Governance, Action Boundary & What-If Simulator Blueprint for KIAN v2

## Source Gems
- `01_openbot_core/docs/approvals.md` & `01_openbot_core/shared/approval-outcomes.ts`
- `01_openbot_core/shared/what-if.ts` (`WhatIfAction`, `WhatIfLayerId`, `trial` mode)
- `01_openbot_core/app_copilot/gallery-tools.tsx` (`useDecisionTool` multi-call `Map<toolCallId, resolver>`)
- `01_openbot_core/app_copilot/approval-tools.tsx` & `01_openbot_core/components_approvals/inline-approval.tsx`
- `04_generative_ui_and_canvas_components/open_mcp_client/PreviewSpreadsheetChanges.tsx`
- `04_generative_ui_and_canvas_components/research_ana_canvas/structure-proposal-viewer.tsx`

## Where to Inject in KIAN v2
1. **Server Action Boundary (`crates/security` / `crates/hermes`)**:
   - Implement OpenBot's 6-category action classifier (Payments, Credentials/2FA, Destructive/Delete/Archive, Send/Reply, Share/Permissions, Post/Publish) checking first verb + accessible button label + MCP `destructiveHint` (never raw document text).
   - Return dual-voice outcomes from `approval-outcomes.ts`: LLM-facing deterministic strings (`DECLINED_BY_PERSON`, `NOT_RUN_WHILE_WAITING`, `HANDLED_BY_PERSON`) vs clean human UI strings via `outcomeForPerson`.
   - Add **Handoff ("I did it myself")** and **Masked Secret Input (`computer_request_secret`)** for passwords/OTP/cards so the LLM never sees user credentials.
2. **Dry-Run Policy Simulator (`crates/security/what_if.rs` + Admin UI)**:
   - Port `WhatIfAction` across layers (`grant`, `app_allowlist`, `capabilities`, `network`, `boundaries`, `safety`, `approvals`) with `trial` mode support.
3. **Frontend Multi-Call HITL Hook (`apps/web/src/hooks/use-decision-tool.ts`)**:
   - Replace single-slot HITL hooks with `Map<toolCallId, { resolve, stop }>` registered via `useFrontendTool` so concurrent tasks/subagents never steal each other's approval resolvers.
