# KIAN `scout` & `analyst` Agents — Zero-Cost Internet Reach Blueprint (`Agent-Reach` Port)

## Tier 1: Zero-Key HTTP Backends (Pure Rust `reqwest` in `crates/agents` / `crates/hermes`)
1. **Exa Free Hosted MCP (`https://mcp.exa.ai/mcp`)**:
   - JSON-RPC `POST`: `{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"web_search_exa","arguments":{"query":"...","numResults":5}}}`
   - Fallback page read: `web_fetch_exa` with `{"urls":["..."],"maxCharacters":20000}`.
   - Handles both `application/json` and SSE `data:` frames.
2. **Jina Reader (`https://r.jina.ai/{url}`)**:
   - Send `Accept: text/plain` + desktop `User-Agent`, cap body at `5 MB`.
   - Run `_is_antibot_page` check on first `4096` bytes (Cloudflare `Just a moment...`, `Ray ID`, `requiring captcha`) and auto-fallback to `web_fetch_exa`.
3. **Strict SSRF Guard (`utils/url.py::normalize_public_http_url`)**:
   - Reject userinfo (`user:pass@host`), backslashes, non-HTTP(S) schemes, `.local`/`.internal`/`localhost`/`metadata.google.internal`, and non-global IP literals (`!ip.is_global()`).

## Tier 2: Public-First Social Routing (`runtime/public_web.py`)
- Before launching a headless browser for `x.com/` or `reddit.com/`, query Exa with `(query) (site:x.com/ OR site:twitter.com/)` and filter out login-wall shells via `_readable()`.
- Use `yt-dlp --skip-download --write-subs --write-auto-subs --sub-format vtt` for zero-cost YouTube transcript extraction.
