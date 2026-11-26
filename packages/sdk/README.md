# @kian/sdk — The MAIRON Sovereign Client

The entry gate for external builders into the ecosystem. Two layers:

1. `MaironSdkClient` (`src/mairon-client.ts`) — the canonical client with fully
   implemented domain namespaces wrapping real `/api/v1/*` routes:
   - `sdk.trinity.{alpha, build, orchestrate, status, publishTwoKey, submitReview}`
   - `sdk.arda.{listLocales, ecosystemMatrix, switchLocale, localize, exportSkillMd}`
   - `sdk.superStudio.{morphMode, executeModality, portfolio}`
   - `sdk.supremacy.{applySurgicalDiff, listCheckpoints, restoreCheckpoint, executeWorkflow, invokeMcpTool, verifySeal}`
   - `sdk.quant.{listVenues, runCausalBacktest, listStrategies, verifyClaimHash, deployLiveStrategy}`
   - `sdk.treasury.{getCharterState, calculateSplit, listGenomeModels, setGenomeRouting}`
   - `sdk.neuralHud.{getGraphState, renderPreset, dashboardUrl}` and
     `sdk.grokLab.{uiBundle, runSimulation, exportSftDpo}`
   - `sdk.warRoom.{getState, stepTick, sendDirective, reset, explainSimple, dashboardUrl}`
   - `sdk.admin.*` (control plane) and **`sdk.syncWorldState()`** — the unified
     authoritative snapshot across active subsystems (degraded subsystems keep
     their last known payload; nothing is dropped).

2. The legacy flat client (`createKianClient` + `api`) — unchanged, still
   exported, so existing imports never break. `KianSdkClient` and
   `createKianSdk` are stable aliases of `MaironSdkClient` / `createMaironSdk`.

Identity is never constructed here: the raw Telegram initData string goes to
the kernel where the official HMAC is verified (fail-closed).
