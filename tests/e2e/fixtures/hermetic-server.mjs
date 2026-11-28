#!/usr/bin/env node
/**
 * Pillar 2 fixture — hermetic zero-dependency HTTP server for the MAIRON
 * Playwright suite. Serves the real kernel artifacts byte-exact from the repo
 * tree plus a deterministic `/api/v1` contract surface and the IMPERIUM
 * tab-lifecycle reference implementation of the mini-app. No network, no
 * kernel, no randomness — the same bytes on every run.
 */
import http from "node:http";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const PORT = Number(process.env.MAIRON_FIXTURE_PORT ?? 8517);

const read = (rel) => readFileSync(path.join(ROOT, rel), "utf8");
const WAR_ROOM_HTML = read("tools/war-room/war_room.html");
const NEURAL_HUD_HTML = read("tools/neural-hud/neural_hud.html");
const FA_PATCH_JSON = read("locales/fa.patch.json");

/** Deterministic contract stub — the exact wire shapes the real kernel emits. */
const API = {
  "/api/v1/health": { status: "ok", service: "MAIRON Sovereign Kernel", fixture: true },
  "/api/v1/trinity/alpha": {
    reply: "Alpha draft assembled from the deterministic fixture.",
    artifacts: { planMd: "# plan", html: "<!DOCTYPE html>" },
    provider: "fixture",
    usedFallback: false,
  },
  "/api/v1/trinity/orchestrate": {
    reply: "Trinity pipeline sealed deterministically.",
    maironScore: 97,
    provider: "fixture",
    usedFallback: false,
  },
  "/api/v1/grok-lab/telemetry": { accepted: true, persistedId: "lab-fixture-1" },
  "/api/v1/grok-lab/summary": { sessions: 10, dna: 10, reaAvg: 91.4, fixture: true },
  "/api/v1/grok-lab/export-sft-dpo": {
    sftRows: [{ messages: [{ role: "user", content: "hi" }, { role: "assistant", content: "hello" }] }],
    dpoRows: [{ prompt: "hi", chosen: "hello", rejected: "..." }],
    format: { sft: "chatml", dpo: "prompt/chosen/rejected" },
  },
  "/api/v1/supremacy/mcp": {
    jsonrpc: "2.0",
    id: 1,
    result: { tools: ["trinity", "arda", "studio", "vibe_quant", "base_treasury"] },
    error: null,
  },
  "/api/v1/war-room/state": {
    signature: "MAIRON-WAR-ROOM",
    githubRepo: "ImXLLM/OneRing",
    tickCursor: 0,
    progressPercent: 0,
    elapsedSeconds: 0,
    scenario: [],
  },
  "/api/v1/war-room/step": {
    executed: { tickIndex: 0, simulationSecond: 28_800 },
    tickCursor: 1,
    progressPercent: 4,
    scenarioExhausted: false,
  },
  "/api/v1/treasury/base/boost": {
    ok: true,
    boostTier: "featured-slot-24h",
    creatorBps: 0,
    treasuryBps: 10_000,
  },
};

/**
 * IMPERIUM tab-lifecycle reference implementation of the Telegram mini-app:
 * every tab stays mounted in the DOM; switching toggles `hidden` + display —
 * zero state loss, the same contract `apps/miniapp` ships.
 */
const MINIAPP_HTML = `<!DOCTYPE html>
<html lang="en" dir="ltr">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0, viewport-fit=cover">
<title>MAIRON Mini-App (IMPERIUM tab fixture)</title>
<style>
  html,body{margin:0;padding:0;background:#050508;color:#e8ecf4;font:14px system-ui;width:100%;}
  *{box-sizing:border-box;max-width:100%;}
  nav{display:flex;gap:4px;padding:6px;position:sticky;top:0;background:#050508;}
  button{flex:1;padding:8px 4px;font-size:12px;background:#101020;color:#e8ecf4;border:1px solid #22d3ee;}
  section{padding:10px;}
  input{width:100%;max-width:100%;padding:10px;font-size:16px;background:#0a0a12;color:#e8ecf4;border:1px solid #22d3ee;}
</style>
</head>
<body>
<nav id="tabs" role="tablist">
  <button id="tab-btn-wallet"   role="tab" aria-selected="true">Wallet</button>
  <button id="tab-btn-builder"  role="tab" aria-selected="false">Builder</button>
  <button id="tab-btn-market"  role="tab" aria-selected="false">Market</button>
  <button id="tab-btn-grok"    role="tab" aria-selected="false">Grok Lab</button>
  <button id="locale-btn" aria-label="Toggle locale direction">FA/EN</button>
</nav>
<section id="tab-wallet"  role="tabpanel" aria-label="Sovereign wallet"><h1>Sovereign Wallet</h1><label>Boost memo <input id="wallet-input" value="day-one"/></label><p id="wallet-note">100% of Instant Boosts route to the MAIRON Sovereign Treasury</p></section>
<section id="tab-builder" role="tabpanel" aria-label="AI builder" hidden><h1>Builder</h1><label>Prompt <input id="builder-input" value=""/></label></section>
<section id="tab-market"  role="tabpanel" aria-label="Marketplace" hidden><h1>Market</h1><label>Search <input id="market-input" value=""/></label></section>
<section id="tab-grok"    role="tabpanel" aria-label="Grok lab" hidden><h1>Grok Lab</h1><label>Session <input id="grok-input" value=""/></label></section>
<script>
  const TABS = ["wallet", "builder", "market", "grok"];
  // Shared store — survives every switch (IMPERIUM law 2).
  const store = { activeTab: "wallet", dir: "ltr" };
  function render() {
    for (const t of TABS) {
      const el = document.getElementById("tab-" + t);
      const active = store.activeTab === t;
      el.hidden = !active;                                       // stays mounted
      el.style.display = active ? "block" : "none";              // zero layout leaks
      document.getElementById("tab-btn-" + t).setAttribute("aria-selected", String(active));
    }
    document.documentElement.setAttribute("dir", store.dir);
    document.documentElement.setAttribute("lang", store.dir === "rtl" ? "fa" : "en");
  }
  window.__maironFixture = { store, render };
  for (const t of TABS) {
    document.getElementById("tab-btn-" + t).addEventListener("click", () => {
      store.activeTab = t; render();
    });
  }
  document.getElementById("locale-btn").addEventListener("click", () => {
    store.dir = store.dir === "ltr" ? "rtl" : "ltr"; render();
  });
  render();
</script>
</body>
</html>
`;

/** Deterministic HMAC echo — the same data-check string always yields the same hash. */
function telegramAuthStub(body) {
  const expected = (body && body.expectedHash) || "";
  return {
    ok: expected.length === 64 && /^[0-9a-f]{64}$/.test(expected),
    session: "fixture-session",
    user: { tgUserId: body?.tgUserId ?? 6398636666 },
    algorithm: "HMAC-SHA256(WebAppData)",
  };
}

const server = http.createServer((req, res) => {
  const url = (req.url ?? "/").split("?")[0];
  const send = (code, body, type) => {
    res.writeHead(code, { "content-type": type, "cache-control": "no-store" });
    res.end(body);
  };
  if (url === "/healthz") return send(200, "ok", "text/plain");
  if (url === "/war-room") return send(200, WAR_ROOM_HTML, "text/html; charset=utf-8");
  if (url === "/war-room/locales/fa.patch.json") return send(200, FA_PATCH_JSON, "application/json; charset=utf-8");
  if (url === "/locales/fa.patch.json") return send(200, FA_PATCH_JSON, "application/json; charset=utf-8");
  if (url === "/neural-hud") return send(200, NEURAL_HUD_HTML, "text/html; charset=utf-8");
  if (url === "/miniapp") return send(200, MINIAPP_HTML, "text/html; charset=utf-8");
  if (url.startsWith("/api/v1/")) {
    if (url === "/api/v1/auth/telegram") {
      let raw = "";
      req.on("data", (c) => (raw += c));
      req.on("end", () => {
        let body = {};
        try { body = JSON.parse(raw || "{}"); } catch { body = {}; }
        return send(200, JSON.stringify(telegramAuthStub(body)), "application/json");
      });
      return;
    }
    if (url === "/api/v1/rate-limit-probe") {
      return send(429, JSON.stringify({ error: { code: "RATE_LIMITED" } }), "application/json");
    }
    const hit = API[url];
    if (hit) return send(200, JSON.stringify(hit), "application/json");
    return send(404, JSON.stringify({ error: { code: "NOT_FOUND" } }), "application/json");
  }
  return send(404, "not found", "text/plain");
});

server.listen(PORT, "127.0.0.1", () => {
  process.stdout.write(`MAIRON hermetic fixture on http://127.0.0.1:${PORT}\n`);
});
