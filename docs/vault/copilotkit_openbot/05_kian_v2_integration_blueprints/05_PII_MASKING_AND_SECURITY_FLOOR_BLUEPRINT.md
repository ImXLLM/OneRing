# Zero-Dep PII Masking, Vault Separation & Ed25519 Security Floor Blueprint for KIAN v2

## Source Gems
- `01_openbot_core/shared/recorder-masking.ts`
- `01_openbot_core/shared/security-floor.ts` & `01_openbot_core/desktop_tauri_rust/security_floor.rs`
- `01_openbot_core/desktop_tauri_rust/vault.rs`
- `04_generative_ui_and_canvas_components/shadify_runtime/resilience.ts`

## Where to Inject in KIAN v2
1. **Linear-Time PII/Secret Scrubber (`crates/security/src/masking.rs` + browser extension)**:
   - Port `recorder-masking.ts` patterns: atomic-group-style linear-time JWT regex (immune to ReDoS), bounded RFC-5321 emails, ISO/IEC 7812-1 card digit grouping, 4–10 digit OTP/2FA key detection, and `CREDENTIAL_HEADERS` scrubbing before any telemetry or memory leaves the client.
2. **Ed25519 Signed Security Floor (`crates/security/src/security_floor.rs`)**:
   - Implement monotonic `serial` + Ed25519 signature verification over raw payload bytes (`security-floor.json`) before parsing JSON, enabling safe emergency version lockouts offline/online.
3. **Process Resilience & OOM Drain (`apps/runtime/resilience.ts`)**:
   - Adopt `isIgnorable` stream disconnect guard + proactive heap-threshold graceful drain (`isShuttingDown()`) for Node/Bun BFF proxies.
