#!/usr/bin/env node
/**
 * KIAN M3 — builds valid Telegram initData for mini-app testing/development.
 *
 * Exactly the official WebApp scheme: sorted params, a data_check_string joined with
 * `\n`, secret = SHA256(bot_token), HMAC-SHA256 → hex hash.
 *
 * Usage:
 *   node tools/telegram/make-init-data.mjs --bot "123:ABC" --user 100 [--username kia] [--json]
 *
 * Warning: never run it with a real bot token in logs/CI.
 */

import { createHash, createHmac } from "node:crypto";

function parseArgs(argv) {
  const out = {};
  for (let i = 2; i < argv.length; i++) {
    const a = argv[i];
    if (a === "--json") out.json = true;
    else if (a.startsWith("--")) out[a.slice(2)] = argv[i + 1];
  }
  return out;
}

const args = parseArgs(process.argv);
const bot = args.bot;
const user = args.user;

if (!bot || !user) {
  console.error("usage: node tools/telegram/make-init-data.mjs --bot <token> --user <id> [--username <u>] [--json]");
  process.exit(2);
}

const userJson = JSON.stringify({
  id: Number(user),
  ...(args.username ? { username: args.username } : {}),
  first_name: args.username ?? String(user),
});

/** Mirrors the real URLSearchParams — only raw unreserved characters survive. */
function urlencode(s) {
  return encodeURIComponent(s).replace(/[!'()*]/g, (c) => `%${c.charCodeAt(0).toString(16).toUpperCase()}`);
}

const params = [
  ["auth_date", String(Math.floor(Date.now() / 1000))],
  ["user", userJson],
];
params.sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0));

const dataCheckString = params.map(([k, v]) => `${k}=${v}`).join("\n");
const secret = createHash("sha256").update(bot).digest();
const hash = createHmac("sha256", secret).update(dataCheckString).digest("hex");

const initData = `${params.map(([k, v]) => `${k}=${urlencode(v)}`).join("&")}&hash=${hash}`;

if (args.json) {
  console.log(JSON.stringify({ initData }));
} else {
  console.log(initData);
}
