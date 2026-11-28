# @kian/admin — MAIRON Institutional Governance & Audit Console

The administrative control plane of the MAIRON platform. Authentication is the
`X-KIAN-Key` header (sessionStorage-scoped, fail-closed: no key configured on
the kernel means the admin endpoints are off).

Sections:

1. **Institutional Overview** — `GET /api/v1/admin/overview`, `GET /api/v1/health`,
   `GET /api/v1/economy` with the 14-day revenue series.
2. **Ledger Auditor** — the live `SUM(Posting.amount) == 0` invariant monitor
   plus the escrow state machine and session-bound release/refund controls.
3. **Two-Key Publish Vault** — the HITL queue (`GET /api/v1/admin/hitl` with
   approve/reject) and the `MaironVerifiedSeal` gate inspector (REA >= 85,
   id = `mairon-seal:<sha256(artifact)[0..16]>`, recomputable client-side).
4. **Genome & Environment** — the 5 x 9 = 45 model override matrix
   (`GET /api/v1/monopoly/genome-matrix`, `POST /api/v1/monopoly/route-model`)
   and the 69-key `P0 -> P5` environment readiness monitor from `@kian/config`.
5. **10-DNA Lab Telemetry** — the seeded benchmark logs with a 1-click
   SFT/DPO JSONL dataset export (`GET /api/v1/grok-lab/export-sft-dpo`, admin-gated).
6. **War-Room Chairman** — directive broadcast and simulation reset through
   `POST /api/v1/war-room/{directive,reset}` with an operator session
   (`POST /api/v1/auth/dev-login`, dev mode only; production fails honestly).
