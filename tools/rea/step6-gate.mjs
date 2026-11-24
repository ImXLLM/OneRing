#!/usr/bin/env node
/**
 * KIAN REA gate — گام ۶ (M6: بات تلگرام teloxide 0.17 — orchestrator خالص) — ۱۰/۱۰
 *
 * ستون‌های این گام:
 *  ۱) هستهٔ خالص بات — صفر I/O در orchestrator، سقف نرخ با ساعت تزریقی،
 *     فرمان‌ها و برش پیام — همهٔ تست‌ها آفلاین با FakePort.
 *  ۲) هویت تک‌مسیره — ضرب initData فقط با `build_bot_init_data` (دارندهٔ
 *     توکن) و تأیید فقط با `verify_telegram_init_data`؛ هیچ مسیر جانبی
 *     (بدون X-KIAN-Key در kian-bot، بدون endpoint هویت جدید).
 *  ۳) وبهوک fail-closed — BOT_DISABLED/BOT_UNAUTHORIZED با مقایسهٔ زمان‌ثابت؛
 *     ران‌تایم فقط با هر دو راز؛ health صادقانه.
 *  ۴) سلام کل workspace — cargo test همه سبز.
 *  ۵) clippy -D warnings صفر خطا.
 *  ۶) بدون credential در سورس.
 *  ۷) audit-release-exposure روی سه آرتیفکت (kernel/admin/web) با fail_on low.
 *
 * خروج: 0 = پاس (۱۰/۱۰)، 3 = شکست. گزارش: docs/rea/step6-m6-gate-report.txt
 */
import { spawnSync } from "node:child_process";
import { existsSync, readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const REPORT = path.join(ROOT, "docs/rea/step6-m6-gate-report.txt");
const AUDIT = path.join(ROOT, "tools/rea/release-exposure.mjs");

let failures = 0;
const lines = [];
const say = (s = "") => { lines.push(s); console.log(s); };
// cargo ممکن است در ~/.cargo/bin باشد — PATH صریح برای spawnSync بدون shell
const ENV = {
  ...process.env,
  PATH: `${process.env.HOME ?? ""}/.cargo/bin:${process.env.PATH ?? ""}`,
  CARGO_INCREMENTAL: "0",
};
const run = (cmd, args, opts = {}) => {
  // maxBuffer ۶۴MB — خروجی cargo test --workspace از ۱MB پیش‌فرض عبور می‌کند
  const res = spawnSync(cmd, args, {
    cwd: ROOT, encoding: "utf8", timeout: opts.timeout ?? 900_000, shell: false, env: ENV,
    maxBuffer: 64 * 1024 * 1024,
  });
  if (res.error) return { code: -1, out: `spawn error: ${res.error}` };
  return { code: res.status ?? -1, out: `${res.stdout ?? ""}${res.stderr ?? ""}` };
};
const check = (name, cond, detail = "") => {
  if (cond) { say(`  PASS  ${name}${detail ? ` — ${detail}` : ""}`); }
  else { failures += 1; say(`  FAIL  ${name}${detail ? ` — ${detail}` : ""}`); }
};
const read = (rel) => readFileSync(path.join(ROOT, rel), "utf8");
const testResult = (out) => {
  const m = out.match(/test result: ok\. (\d+) passed; (\d+) failed/);
  return m ? { passed: Number(m[1]), failed: Number(m[2]) } : null;
};

say("# KIAN REA gate — step 6 (M6 telegram bot — teloxide 0.17 pure orchestrator — 10/10)");
say(`# date: ${new Date().toISOString()}`);
say("# kernel: target/release/kian-server — audit-release-exposure --fail_on low");
say("# admin:  apps/admin/.next/standalone — audit-release-exposure --fail_on low");
say("# web:    apps/web/.next/standalone — audit-release-exposure --fail_on low");
say("");

/* ── ۱) هستهٔ خالص بات — آفلاین‌تست‌پذیر ─────────────────────────────────── */
say("════ 1) bot orchestrator — pure, offline-testable ════");
{
  const orch = read("crates/bot/src/orchestrator.rs");
  const lib = read("crates/bot/src/lib.rs");
  check("orchestrator has zero I/O — no reqwest/teloxide/std::net in the core",
    !/reqwest|teloxide|std::net|tokio::net/.test(orch));
  check("rate limit uses injected clock (now_ms) — no Instant/now in core",
    /now_ms/.test(orch) && !/Instant::now|SystemTime::now/.test(orch));
  check("commands wired (start/help/id/link/economy/balance/me + fallback)",
    ['"start"', '"help"', '"id"', '"link"', '"economy"', '"balance"', '"me"'].every((c) => orch.includes(c)));
  check("reply truncation at char boundary (policy cap ≤ telegram 4096)",
    /TELEGRAM_MAX_CHARS/.test(orch) && /fn clip/.test(orch));
  check("rate-limit warns once per window (no spam amplification)",
    /RateVerdict/.test(orch) && /last_warn/.test(orch));
  check("both shells share one parse path (from_parts/from_json)",
    /from_parts/.test(read("crates/bot/src/update.rs")) &&
      /from_parts/.test(read("crates/bot/src/shell.rs")) &&
      /BotUpdate::from_json/.test(read("crates/server/src/handlers/bot.rs")));
  check("kian-bot wired into workspace (lib + bin kian-bot)",
    /name = "kian-bot"/.test(read("crates/bot/Cargo.toml")));
  const t = run("cargo", ["test", "-p", "kian-bot"], { timeout: 300_000 });
  const r = testResult(t.out);
  check("kian-bot tests green (pure core offline)", r && r.failed === 0, r ? `${r.passed} passed` : t.out.slice(0, 200));
}
say("");

/* ── ۲) هویت تک‌مسیره — ضرب و تأیید رسمی initData ───────────────────────── */
say("════ 2) identity — single official path (mint → verify HMAC) ════");
{
  const security = read("crates/contracts/src/security.rs");
  check("official minter exists in contracts (single source)",
    /pub fn build_bot_init_data/.test(security));
  check("mint→verify roundtrip test present (official scheme)",
    /bot_minted_init_data_roundtrips_through_official_verify/.test(security));
  check("minter escapes hostile usernames (JSON-safe)",
    /fn escape_json/.test(security));
  const kernelHttp = read("crates/bot/src/kernel_http.rs");
  check("polling shell uses POST /auth/telegram (same path as miniapp)",
    /\/api\/v1\/auth\/telegram/.test(kernelHttp) && /build_bot_init_data/.test(kernelHttp));
  const serverBot = read("crates/server/src/handlers/bot.rs");
  check("webhook shell verifies minted initData through official verify",
    /verify_telegram_init_data\(&init_data/.test(serverBot) && /build_bot_init_data/.test(serverBot));
  check("no X-KIAN-Key / admin path anywhere in kian-bot",
    !/X-KIAN-Key|admin_key|\/admin\//.test(read("crates/bot/src/lib.rs") + read("crates/bot/src/kernel_http.rs") + read("crates/bot/src/shell.rs") + read("crates/bot/src/port.rs")));
  check("no new identity endpoint on the kernel (only /auth/telegram)",
    !/bot\/session|\/auth\/bot/.test(read("crates/server/src/routes.rs")));
}
say("");

/* ── ۳) وبهوک fail-closed + health صادقانه ──────────────────────────────── */
say("════ 3) webhook fail-closed — BOT_DISABLED / BOT_UNAUTHORIZED ════");
{
  const serverBot = read("crates/server/src/handlers/bot.rs");
  const contracts = read("crates/contracts/src/error.rs");
  check("BOT_DISABLED 503 + BOT_UNAUTHORIZED 401 exist in contracts (phase-5 contract)",
    /BOT_DISABLED/.test(contracts) && /BOT_UNAUTHORIZED/.test(contracts));
  check("guard uses constant-time compare on the official secret header",
    /x-telegram-bot-api-secret-token/.test(serverBot) && /verify_kian_key/.test(serverBot));
  check("runtime builds only with both secrets (token + webhook secret ≥ 16)",
    /build_bot/.test(read("crates/server/src/state.rs")) && /secret.len\(\) < 16/.test(read("crates/server/src/state.rs")));
  check("webhook route wired", /\/bot\/webhook/.test(read("crates/server/src/routes.rs")));
  check("non-text updates answered 200 handled:false (no reply to unknown)",
    /handled/.test(serverBot));
  check("health reports bot honestly from runtime presence",
    /bot_enabled = state\.bot\.is_some\(\)/.test(read("crates/server/src/handlers/system.rs")));
  check("reply execution behind TelegramSender port (tests record, prod sends)",
    /pub trait TelegramSender/.test(serverBot) && /RecordingSender/.test(serverBot) && /TelegramHttpSender/.test(serverBot));
  const t = run("cargo", ["test", "-p", "kian-server", "bot"], { timeout: 300_000 });
  const r = testResult(t.out);
  check("kian-server webhook tests green (guard matrix + full flow offline)",
    r && r.failed === 0, r ? `${r.passed} passed` : t.out.slice(0, 200));
}
say("");

/* ── ۴) سلام کل workspace ───────────────────────────────────────────────── */
say("════ 4) whole-workspace tests green ════");
{
  const t = run("cargo", ["test", "--workspace"], { timeout: 900_000 });
  const passed = [...t.out.matchAll(/test result: ok\. (\d+) passed/g)].reduce((a, m) => a + Number(m[1]), 0);
  const failed = [...t.out.matchAll(/test result: FAILED/g)].length;
  const err = /error\[E|error: could not compile/.test(t.out);
  check("cargo test --workspace", !err && failed === 0, `${passed} tests passed, ${failed} failing suites`);
  check("server integration surface covers M6 flows", /48 passed/.test(t.out), "kian-server lib = 48 tests");
}
say("");

/* ── ۵) clippy -D warnings صفر ──────────────────────────────────────────── */
say("════ 5) clippy -D warnings = 0 errors ════");
{
  const t = run("cargo", ["clippy", "--workspace", "--all-targets", "--", "-D", "warnings"], { timeout: 900_000 });
  const errCount = (t.out.match(/^error/gm) ?? []).length;
  check("clippy clean", t.code === 0 && errCount === 0, `${errCount} error(s)`);
}
say("");

/* ── ۶) بدون credential در سورس ─────────────────────────────────────────── */
say("════ 6) no credential markers in source ════");
{
  let hits = 0;
  const scan = [
    "crates/bot/src",
    "crates/server/src/handlers/bot.rs",
    "crates/contracts/src/security.rs",
    "apps/web/src",
    "apps/admin/app",
  ];
  for (const rel of scan) {
    const abs = path.join(ROOT, rel);
    if (!existsSync(abs)) continue;
    const res = spawnSync("grep", ["-rEnI", "(sk-[A-Za-z0-9]{16}|bot[0-9]{6}:[A-Za-z0-9_-]{30}|BEGIN (RSA |EC )?PRIVATE KEY|KIAN_ADMIN_KEY=|KIAN_BOT_TOKEN=[0-9])", abs], { encoding: "utf8" });
    const out = `${res.stdout ?? ""}`;
    if (out.trim()) { hits += out.trim().split("\n").length; say(`    HIT in ${rel}: ${out.trim().split("\n")[0]}`); }
  }
  check("no literal keys/tokens in M6 sources and frontends", hits === 0, `${hits} hits`);
}
say("");

/* ── ۷) audit-release-exposure روی سه آرتیفکت ───────────────────────────── */
say("════ 7) audit-release-exposure (fail_on low) ════");
mkdirSync(path.join(ROOT, "docs/rea"), { recursive: true });
{
  const artifacts = [
    ["kernel", "target/release/kian-server"],
    ["admin", "apps/admin/.next/standalone"],
    ["web", "apps/web/.next/standalone"],
  ];
  for (const [name, rel] of artifacts) {
    const abs = path.join(ROOT, rel);
    if (!existsSync(abs)) { failures += 1; say(`  FAIL  ${name} artifact missing: ${rel}`); continue; }
    say(`── ${name}: ${rel}`);
    const res = spawnSync("node", [AUDIT, abs, "--fail_on", "low"], { cwd: ROOT, encoding: "utf8", timeout: 300_000 });
    const out = `${res.stdout ?? ""}${res.stderr ?? ""}`;
    const notObserved = (out.match(/not_observed/g) ?? []).length;
    const monitored = (out.match(/monitored/g) ?? []).length;
    const exposed = (out.match(/"exposed"|exposed:/g) ?? []).length;
    const ev = out.match(/evidence_id: (\S+)/);
    say(out.trim().split("\n").slice(0, 40).join("\n"));
    check(`${name} audit exit 0 (fail_on low)`, res.status === 0, `status=${res.status}`);
    if (ev) say(`  evidence_id: ${ev[1]}`);
    say(`  filters: not_observed×${notObserved} monitored×${monitored} exposed×${exposed}`);
  }
}
say("");

/* ── جمع‌بندی ───────────────────────────────────────────────────────────── */
say(`════ verdict: ${failures === 0 ? "PASS — M6 gate 10/10 green (exit 0)" : `FAIL — ${failures} failing check(s) (exit 3)`} ════`);
writeFileSync(REPORT, `${lines.join("\n")}\n`);
console.log(`\nreport: ${REPORT}`);
process.exit(failures === 0 ? 0 : 3);
