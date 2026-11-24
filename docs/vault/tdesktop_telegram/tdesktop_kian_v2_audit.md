# KIAN v2 — sparse forensic audit of `telegramdesktop/tdesktop`

**Scope:** reusable Telegram Mini App / bot / payment implementation patterns for `crates/bot` and `apps/miniapp`.

## Executive readout

1. **The Mini App implementation is outside the originally listed folders.** The key client code is in `Telegram/SourceFiles/inline_bots/bot_attach_web_view.*` and `Telegram/SourceFiles/ui/chat/attach/attach_bot_webview.*`. I added those paths to the sparse checkout. `iv` is Instant View code, not the Mini App host.
2. **TDesktop is a Telegram client, not a Bot API server.** Its launch and checkout paths use internal MTProto RPCs such as `messages.requestAppWebView` and `messages.sendWebViewData`. KIAN's Rust bot should implement the public Bot API update and payment flow; these C++ RPC call sites are architecture evidence, not Bot API endpoints to copy.
3. **The strongest security gem is server-side `initData` validation.** The desktop host returns a WebView URL and runs the app; it does not authenticate KIAN's backend user for you. Send raw `Telegram.WebApp.initData` to Rust, validate its HMAC there, enforce freshness, and only then derive identity from its signed `user` field. Do not trust `initDataUnsafe` or a browser-supplied Telegram ID. [1](https://core.telegram.org/bots/webapps)
4. **Treat Mini App actions and payment callbacks as untrusted client input.** TDesktop limits `sendData` to a specific launch mode and one send per local instance, but that is client UX logic—not durable replay protection. Telegram also warns that `web_app_data.data` may be arbitrary; `sendData` is limited to 4096 bytes and is only available for keyboard-button launches. [1](https://core.telegram.org/bots/api) · [1](https://core.telegram.org/bots/webapps)
5. **No KIAN code or `seed/kian-production.db` was present in the workspace.** This is therefore an evidence-backed extraction and integration plan, not a patch or database audit. No KIAN files or schema were changed.

## Snapshot and speed/memory constraint

- Repository: `telegramdesktop/tdesktop`
- Audited HEAD: `070dbc645c543ce8b9bbbb7b72ef842ffd7671da` (`2026-11-23T23:40:02+04:00`)
- Used `git clone --depth 1 --filter=blob:none --sparse`, then sparse checkout. No full clone or recursive submodule checkout.
- Started with the requested `iv`, `payments`, `chat_helpers`, `data`, and `api` paths; added `inline_bots` and `ui/chat/attach` because they contain the actual Mini App launch and bridge code.
- Initial clone + first sparse checkout returned in about **1.9 s**; added paths took under a further second. Final sparse working tree, including `.git`, was about **15 MB** with 685 files in the selected source areas.
- Static source audit only; no C++ build or test suite was run. `Telegram/lib_webview` and other submodules were not fetched.

## Findings from TDesktop

### 1. Button behavior is a typed protocol, not one generic “bot button” action

`Api::ActivateBotButton` keeps ordinary text commands, callback buttons, Buy buttons, URL buttons, switch-inline buttons, WebView buttons, and **SimpleWebView** buttons as separate cases. Regular and Simple WebViews are opened differently; callback data is sent through a separate callback-answer request. [api_bot.cpp, button dispatch](https://github.com/telegramdesktop/tdesktop/blob/070dbc645c543ce8b9bbbb7b72ef842ffd7671da/Telegram/SourceFiles/api/api_bot.cpp#L313-L540) · [callback request lifecycle](https://github.com/telegramdesktop/tdesktop/blob/070dbc645c543ce8b9bbbb7b72ef842ffd7671da/Telegram/SourceFiles/api/api_bot.cpp#L54-L194)

**KIAN extraction:** represent incoming Bot API updates as distinct typed cases—e.g. callback query, ordinary message/command, `message.web_app_data`, `pre_checkout_query`, and `successful_payment`. Do not flatten them into one “button click” event; they have different trust, reply, and acknowledgement semantics.

### 2. Launch provenance and `start_param` are explicit

TDesktop models launch sources with a `WebViewSource` variant: keyboard button, switch, app link, attach-menu link, bot profile, main menu, attach menu, bot menu, game, and join-chat flow. The descriptor separately carries bot, context, button, and source. [bot_attach_web_view.h, source and descriptor types](https://github.com/telegramdesktop/tdesktop/blob/070dbc645c543ce8b9bbbb7b72ef842ffd7671da/Telegram/SourceFiles/inline_bots/bot_attach_web_view.h#L59-L239)

For app links, it resolves bot-app metadata, retains the start parameter, and may ask for open/write consent. The resulting `requestApp` sends the app ID/access hash, start parameter, theme, platform string (`tdesktop`), and flags for fullscreen, start parameter, and write permission over MTProto. [resolve / consent](https://github.com/telegramdesktop/tdesktop/blob/070dbc645c543ce8b9bbbb7b72ef842ffd7671da/Telegram/SourceFiles/inline_bots/bot_attach_web_view.cpp#L1096-L1237) · [requestApp](https://github.com/telegramdesktop/tdesktop/blob/070dbc645c543ce8b9bbbb7b72ef842ffd7671da/Telegram/SourceFiles/inline_bots/bot_attach_web_view.cpp#L1336-L1367) · [button/simple/main WebView requests](https://github.com/telegramdesktop/tdesktop/blob/070dbc645c543ce8b9bbbb7b72ef842ffd7671da/Telegram/SourceFiles/inline_bots/bot_attach_web_view.cpp#L1239-L1334)

The Mini App docs describe `start_param` and direct-link launch context as optional launch data. In KIAN, treat these as **routing/context hints**, not authorization. If a start parameter grants access to an order, invitation, or account, validate that grant server-side and bind it to the authenticated user. [1](https://core.telegram.org/bots/webapps)

### 3. Client launch URL/query ID is a lease/lifecycle detail—not user authentication

TDesktop parses a `WebViewResult` into URL, query ID, fullscreen/fullsize, and a `sameOrigin` flag. When a query ID exists, it listens for the matching WebView-result-sent update and periodically prolongs the WebView request; the instance is closed when the matching result event arrives. [result parsing](https://github.com/telegramdesktop/tdesktop/blob/070dbc645c543ce8b9bbbb7b72ef842ffd7671da/Telegram/SourceFiles/inline_bots/bot_attach_web_view.cpp#L862-L871) · [lifecycle / prolong](https://github.com/telegramdesktop/tdesktop/blob/070dbc645c543ce8b9bbbb7b72ef842ffd7671da/Telegram/SourceFiles/inline_bots/bot_attach_web_view.cpp#L1552-L1599)

**KIAN extraction:** model app launch source/context and server-issued action/query IDs explicitly. A `query_id` is useful for Telegram's answer flow; it is not a substitute for validating `initData` or authenticating arbitrary backend requests.

### 4. Native bridge defenses are useful, but do not copy its limits as app limits

The native host parses bridge messages as JSON, rejects malformed payloads, distinguishes legacy and external-shell envelopes, validates the external shell's source/origin/token, and checks WebView message origin when the server-provided `sameOrigin` flag is enabled. The bridge also rejects messages over `64 MiB` and sanitizes command names written to logs. [message parser and shell envelope](https://github.com/telegramdesktop/tdesktop/blob/070dbc645c543ce8b9bbbb7b72ef842ffd7671da/Telegram/SourceFiles/ui/chat/attach/attach_bot_webview.cpp#L73-L388) · [message size/origin/dispatch](https://github.com/telegramdesktop/tdesktop/blob/070dbc645c543ce8b9bbbb7b72ef842ffd7671da/Telegram/SourceFiles/ui/chat/attach/attach_bot_webview.cpp#L2346-L2535) · [bridge shim](https://github.com/telegramdesktop/tdesktop/blob/070dbc645c543ce8b9bbbb7b72ef842ffd7671da/Telegram/SourceFiles/ui/chat/attach/attach_bot_webview.cpp#L2622-L2630) · [same-origin outbound event guard](https://github.com/telegramdesktop/tdesktop/blob/070dbc645c543ce8b9bbbb7b72ef842ffd7671da/Telegram/SourceFiles/ui/chat/attach/attach_bot_webview.cpp#L3913-L3927)

The bridge maps `web_app_*` commands into explicit handlers for close, buttons, theme/viewport, link opening, invoice opening, write access, phone requests, storage, and more. Some platform capabilities deliberately report unsupported instead of pretending success. TDesktop's JavaScript shim is an internal native transport; `apps/miniapp` should use Telegram's documented WebApp SDK/API, not depend on `TelegramWebviewProxy` or `TelegramGameProxy` internals.

**Do not copy the `64 MiB` bridge cap as a product/API limit.** It is a host-side message guard. The public `sendData` limit is 4096 bytes and applies only to keyboard-button launches. Larger application payloads should go through authenticated KIAN API requests. [1](https://core.telegram.org/bots/webapps)

### 5. Consent and storage are scoped and explicit

For app launches, TDesktop can show a consent prompt, including a separate checkbox for write access. A Web App's later write-access request checks current permission, asks the user, and only then calls the allow-send-message operation. [app consent](https://github.com/telegramdesktop/tdesktop/blob/070dbc645c543ce8b9bbbb7b72ef842ffd7671da/Telegram/SourceFiles/inline_bots/bot_attach_web_view.cpp#L1183-L1237) · [write-access prompt](https://github.com/telegramdesktop/tdesktop/blob/070dbc645c543ce8b9bbbb7b72ef842ffd7671da/Telegram/SourceFiles/ui/chat/attach/attach_bot_webview.cpp#L3093-L3144) · [permission RPC](https://github.com/telegramdesktop/tdesktop/blob/070dbc645c543ce8b9bbbb7b72ef842ffd7671da/Telegram/SourceFiles/inline_bots/bot_attach_web_view.cpp#L1891-L1909)

Device storage is keyed by bot and caps a value and aggregate key/value `QString::size()` budget at `5 * 2^20` code units (not literal UTF-8 wire bytes). This is a useful isolation/quota pattern, not a KIAN server-side storage guarantee. [inline_bot_storage.cpp](https://github.com/telegramdesktop/tdesktop/blob/070dbc645c543ce8b9bbbb7b72ef842ffd7671da/Telegram/SourceFiles/inline_bots/inline_bot_storage.cpp#L18-L85)

**KIAN extraction:** keep permissions explicit and auditable. A client prompt or TDesktop's local “trusted bot” state cannot grant a backend authorization. Check KIAN's own user/session permissions on every sensitive endpoint.

### 6. `sendData` is narrow and client-side single-use

The panel checks that data is a non-empty string. The delegate sends it only for a SimpleWebView launched from the bot's own chat, sets a per-instance `_dataSent` guard, sends the bytes, applies returned updates, and closes the app. [panel validation](https://github.com/telegramdesktop/tdesktop/blob/070dbc645c543ce8b9bbbb7b72ef842ffd7671da/Telegram/SourceFiles/ui/chat/attach/attach_bot_webview.cpp#L2722-L2734) · [source/single-send guard](https://github.com/telegramdesktop/tdesktop/blob/070dbc645c543ce8b9bbbb7b72ef842ffd7671da/Telegram/SourceFiles/inline_bots/bot_attach_web_view.cpp#L1838-L1858)

The public Bot API says `web_app_data.data` may be arbitrary, and documents `sendData` as keyboard-button-only and up to 4096 bytes. Therefore, on KIAN: parse it as untrusted input, validate schema/size, authorize against the verified sender and expected bot/chat context, and add durable deduplication if processing it can mutate state. A client boolean is not replay protection. [1](https://core.telegram.org/bots/api) · [1](https://core.telegram.org/bots/webapps)

### 7. Mini App invoice UI is not the payment source of truth

`openInvoice` only validates that a slug exists before delegating to the payment flow. `WebViewInstance` starts Telegram's checkout process and maps its UI outcome to `paid`, `failed`, `pending`, or `cancelled` for the app panel. [openInvoice dispatch](https://github.com/telegramdesktop/tdesktop/blob/070dbc645c543ce8b9bbbb7b72ef842ffd7671da/Telegram/SourceFiles/ui/chat/attach/attach_bot_webview.cpp#L2986-L2998) · [checkout/result mapping](https://github.com/telegramdesktop/tdesktop/blob/070dbc645c543ce8b9bbbb7b72ef842ffd7671da/Telegram/SourceFiles/inline_bots/bot_attach_web_view.cpp#L1676-L1729)

That result is **client UI state**, not authoritative proof for KIAN to fulfill an order. The public Bot API payment flow requires answering `pre_checkout_query` within 10 seconds and reports completion through `successful_payment`; Telegram documents currency/total amount as integer fields, with `XTR` for Stars. KIAN should grant goods/credits only after its bot update handler records and validates the corresponding successful payment, idempotently. [1](https://core.telegram.org/bots/api)

## KIAN v2 implementation plan

### `crates/bot` — proposed extraction (not applied)

- **`miniapp::validate_init_data(raw, bot_token, now, max_age)`**: parse the raw query string; build the sorted newline-separated `key=value` check string from decoded values; compute `secret = HMAC-SHA256(key="WebAppData", message=bot_token)` and then the expected HMAC over the check string; compare hashes in constant time; validate numeric/fresh `auth_date`; decode `user` only after successful verification. With modern data, exclude `hash` from the bot-token HMAC input but keep any `signature` field that was received. The third-party Ed25519 verification has a different check string (excludes both `hash` and `signature`); do not conflate the two flows. [1](https://core.telegram.org/bots/webapps)
- Return a server-side verified principal/context (Telegram user ID, auth time, and optional signed launch fields). Issue KIAN's own short-lived session/token if the Mini App needs multiple API calls. Keep the bot token only in server configuration; never send it to Next.js/browser code.
- Route update types separately: `message.web_app_data`, callback query, command/message, `pre_checkout_query`, and `successful_payment`. Answer pre-checkout promptly; persist/dedupe updates and payment charge IDs so retried updates cannot double-credit.
- Generate order identity, payload, price, currency, and authorization on the server. Store monetary values as integer minor units / integer Stars, not floating-point values. Never accept a browser-submitted amount or user ID as authoritative.

### `apps/miniapp` — proposed extraction (not applied)

- Use Telegram's documented WebApp SDK and pass the **raw** `initData` string to the Rust backend for validation. Use parsed/unsafe fields only for display until the backend authenticates them. Do not build a custom dependency on TDesktop's private native bridge.
- Call `ready()` after first render; make theme and viewport/safe-area styling resilient to missing/partial platform features. Treat `start_param` as navigation state, not a credential.
- For checkout, ask the backend to create/resolve an invoice and then use Telegram's invoice flow. Display the Mini App's invoice-close status as UX only; await the Bot API `successful_payment` update before changing durable entitlement/balance state.
- Use `sendData` only for a small keyboard-button handoff (≤4096 bytes). For normal app actions or larger data, use authenticated KIAN endpoints with server validation and idempotency keys.

### `seed/kian-production.db` — follow-up once available

The DB was not available to inspect, so I did not assume table names or alter data. During integration, check for durable uniqueness/idempotency around Bot API `update_id`, invoice/order payload, and Telegram payment charge IDs; store amounts as integers with currency; and avoid retaining raw `initData` longer than needed. The verified user identity should be associated with each sensitive action/order.

## Do-not-copy list

- Do not treat TDesktop MTProto requests as public Bot API methods.
- Do not trust `initDataUnsafe`, `start_param`, WebView JS events, `web_app_data.data`, a `paid` event from JavaScript, or a client-side `_dataSent`-style flag as authorization/payment proof.
- Do not copy the 64 MiB native bridge ceiling as the `sendData` limit; the public limit is 4096 bytes.
- Do not reuse TDesktop's UI trust/consent state as KIAN's backend permission system.
- Do not strip both `hash` and `signature` from the Bot-token HMAC input; only the Ed25519 third-party signature check excludes both.

## Source map

All code links above are pinned to audit HEAD `070dbc645c543ce8b9bbbb7b72ef842ffd7671da`; repository paths are case-sensitive. Primary areas: `api/api_bot.cpp`, `inline_bots/bot_attach_web_view.*`, `ui/chat/attach/attach_bot_webview.*`, `inline_bots/inline_bot_storage.cpp`, and `payments/payments_checkout_process.cpp`.
