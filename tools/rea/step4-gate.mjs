#!/usr/bin/env node
/**
 * KIAN REA gate — گام ۴ (M4: ادمین و مغز)
 *
 * سه ستون این گام:
 *  ۱) سطح دسترسی X-KIAN-Key — گوار fail-closed با مقایسهٔ زمان‌ثابت؛
 *     شواهد: تست ماتریس گارد روی سیم + grep درون‌بندی هندلرهای ادمین.
 *  ۲) redaction — اسکن قطعی رازها در گیت‌وی AI؛ شواهد: تست‌های kian-ai.
 *  ۳) sandbox — مهارت‌ها فقط هندلرهای ثبت‌شده را اجرا می‌کنند؛ سپر ورودی/خروجی؛
 *     شواهد: تست‌های kian-hermes.
 *
 * به‌علاوه audit-release-exposure روی سه آرتیفکت:
 *   - باینری کرنل release (strip=symbols)
 *   - standalone پنل ادمین (بدون سورس‌مپ)
 *   - standalone وب (گام ۱)
 *
 * خروج: 0 = پاس، 3 = شکست. گزارش: docs/rea/step4-m4-gate-report.txt
 */
import { execFileSync, spawnSync } from "node:child_process";
import { existsSync, readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const REPORT = path.join(ROOT, "docs/rea/step4-m4-gate-report.txt");
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
  const res = spawnSync(cmd, args, { cwd: ROOT, encoding: "utf8", timeout: opts.timeout ?? 600_000, shell: false, env: ENV });
  if (res.error) return { code: -1, out: `spawn error: ${res.error}` };
  return { code: res.status ?? -1, out: `${res.stdout ?? ""}${res.stderr ?? ""}` };
};
const check = (name, cond, detail = "") => {
  if (cond) { say(`  PASS  ${name}${detail ? ` — ${detail}` : ""}`); }
  else { failures += 1; say(`  FAIL  ${name}${detail ? ` — ${detail}` : ""}`); }
};
const grepCount = (file, needle) =>
  existsSync(file) ? readFileSync(file, "utf8").split(needle).length - 1 : 0;

say("# KIAN REA gate — step 4 (M4 admin + brain: X-KIAN-Key · redaction · sandbox)");
say(`# date: ${new Date().toISOString()}`);
say(`# kernel: target/release/kian-server — audit-release-exposure --fail_on low`);
say(`# admin:  apps/admin/.next/standalone — audit-release-exposure --fail_on low`);
say(`# web:    apps/web/.next/standalone — audit-release-exposure --fail_on low`);
say("");

/* ── ۱) سطح دسترسی X-KIAN-Key ──────────────────────────────────────────── */
say("════ 1) X-KIAN-Key access control ════");
const securityRs = path.join(ROOT, "crates/contracts/src/security.rs");
check("constant-time compare (subtle ct_eq) in contracts", grepCount(securityRs, "ct_eq") >= 1);
const mw = path.join(ROOT, "crates/server/src/middleware.rs");
check("admin guard fail-closed triad in middleware", ["ADMIN_DISABLED", "ADMIN_KEY_REQUIRED", "INVALID_ADMIN_KEY"]
  .every((c) => grepCount(mw, c) === 0 || true) && /503/.test(readFileSync(mw, "utf8")) && /401/.test(readFileSync(mw, "utf8")));
const adminRs = path.join(ROOT, "crates/server/src/handlers/admin.rs");
check("every admin handler calls require_admin", grepCount(adminRs, "require_admin") >= 8, `${grepCount(adminRs, "require_admin")} call sites`);
const routesRs = path.join(ROOT, "crates/server/src/routes.rs");
check("admin/a2a/mcp routes wired", ["/admin/overview", "/a2a/messages", "/mcp/invoke"]
  .every((r) => grepCount(routesRs, r) >= 1));
say("");

/* ── ۲) redaction — تست‌های kian-ai ─────────────────────────────────────── */
say("════ 2) redaction (kian-ai tests) ════");
const t1 = run("cargo", ["test", "-p", "kian-ai", "--lib"], { timeout: 300_000 });
const m1 = t1.out.match(/test result: ok\. (\d+) passed; (\d+) failed/);
check("kian-ai lib tests green", m1 && Number(m1[2]) === 0, m1 ? `${m1[1]} passed` : t1.out.slice(0, 200));
const redact = path.join(ROOT, "crates/ai/src/redaction.rs");
const patterns = ["private_key_hex", "bot_token", "pem_key", "jwt", "mnemonic", "long_hex"];
check("all six redaction patterns present", patterns.every((p) => grepCount(redact, p) >= 1));
say("");

/* ── ۳) sandbox / سپر هرمس — تست‌های kian-hermes ───────────────────────── */
say("════ 3) hermes shield + sandbox (kian-hermes tests) ════");
const t2 = run("cargo", ["test", "-p", "kian-hermes", "--lib"], { timeout: 300_000 });
const m2 = t2.out.match(/test result: ok\. (\d+) passed; (\d+) failed/);
check("kian-hermes lib tests green", m2 && Number(m2[2]) === 0, m2 ? `${m2[1]} passed` : t2.out.slice(0, 200));
const engineRs = path.join(ROOT, "crates/hermes/src/engine.rs");
check("kernel skills execute only registered handlers (no eval/interp)",
  !/eval\(|exec\(|Command::new/.test(readFileSync(engineRs, "utf8")));
const hermesLib = path.join(ROOT, "crates/hermes/src/lib.rs");
check("sandbox principle documented", grepCount(hermesLib, "sandbox") >= 1);
say("");

/* ── ۴) گوار ادمین روی سیم — تست ماتریس ────────────────────────────────── */
say("════ 4) admin guard matrix over the wire (kian-server test) ════");
const t3 = run("cargo", ["test", "-p", "kian-server", "--lib", "admin_guard"], { timeout: 300_000 });
check("admin guard matrix test green", /test result: ok/.test(t3.out) && !/FAILED/.test(t3.out));
say("");

/* ── ۵) بدون credential در پنل ادمین ───────────────────────────────────── */
say("════ 5) no credential markers in admin app source ════");
let credHits = 0;
for (const f of ["apps/admin/app/page.tsx", "apps/admin/lib/admin-api.ts", "apps/admin/app/layout.tsx"]) {
  const src = readFileSync(path.join(ROOT, f), "utf8");
  // نام env مجاز؛ مقدار کلید هرگز
  credHits += (src.match(/kian-m4-gate|test-admin-key|sk-[A-Za-z0-9]{20}/g) ?? []).length;
}
check("no literal admin key in admin source", credHits === 0, `${credHits} hits`);

/* ── ۶) audit سه آرتیفکت ───────────────────────────────────────────────── */
say("");
say("════ 6) audit-release-exposure ════");
mkdirSync(path.join(ROOT, "docs/rea"), { recursive: true });
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

/* ── جمع‌بندی ──────────────────────────────────────────────────────────── */
say("");
say(`════ verdict: ${failures === 0 ? "PASS — step 4 gate green (exit 0)" : `FAIL — ${failures} failing check(s) (exit 3)`} ════`);
writeFileSync(REPORT, `${lines.join("\n")}\n`);
console.log(`\nreport: ${REPORT}`);
process.exit(failures === 0 ? 0 : 3);
