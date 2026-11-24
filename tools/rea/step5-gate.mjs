#!/usr/bin/env node
/**
 * KIAN REA gate — گام ۵ (M5: حاکمیت + کیف پول چهارزنجیره) — گیت نهایی ۱۰/۱۰
 *
 * ستون‌های این گام:
 *  ۱) حاکمیت — پارلمان HITL گره‌خورده با مادهٔ ۷ منشور (آستانه از kian-domain،
 *     رأی انسانی اجباری برای مبالغ ≥ ۵۰,۰۰۰ Đ)؛ اجرا فقط از مسیر دامنهٔ خالص.
 *  ۲) کیف پول چهارزنجیره — اعتبارسنجی آدرس با چک‌سام‌های رسمی هر زنجیره
 *     (Base58Check / 32B / CRC16-XMODEM / EIP-55) + قراردادهای کانونیک USDT.
 *  ۳) سلام کل workspace — cargo test همه سبز.
 *  ۴) clippy -D warnings صفر خطا.
 *  ۵) بدون credential در سورس.
 *  ۶) audit-release-exposure روی سه آرتیفکت (kernel/admin/web) با fail_on low.
 *
 * خروج: 0 = پاس (۱۰/۱۰)، 3 = شکست. گزارش: docs/rea/step5-m5-final-gate-report.txt
 */
import { spawnSync } from "node:child_process";
import { existsSync, readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const REPORT = path.join(ROOT, "docs/rea/step5-m5-final-gate-report.txt");
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
  const res = spawnSync(cmd, args, { cwd: ROOT, encoding: "utf8", timeout: opts.timeout ?? 900_000, shell: false, env: ENV });
  if (res.error) return { code: -1, out: `spawn error: ${res.error}` };
  return { code: res.status ?? -1, out: `${res.stdout ?? ""}${res.stderr ?? ""}` };
};
const check = (name, cond, detail = "") => {
  if (cond) { say(`  PASS  ${name}${detail ? ` — ${detail}` : ""}`); }
  else { failures += 1; say(`  FAIL  ${name}${detail ? ` — ${detail}` : ""}`); }
};
const grepCount = (file, needle) =>
  existsSync(file) ? readFileSync(file, file.endsWith(".rs") ? "utf8" : "utf8").split(needle).length - 1 : 0;
const testResult = (out) => {
  const m = out.match(/test result: ok\. (\d+) passed; (\d+) failed/);
  return m ? { passed: Number(m[1]), failed: Number(m[2]) } : null;
};

say("# KIAN REA gate — step 5 FINAL (M5 governance + four-chain wallet — 10/10)");
say(`# date: ${new Date().toISOString()}`);
say("# kernel: target/release/kian-server — audit-release-exposure --fail_on low");
say("# admin:  apps/admin/.next/standalone — audit-release-exposure --fail_on low");
say("# web:    apps/web/.next/standalone — audit-release-exposure --fail_on low");
say("");

/* ── ۱) حاکمیت — پارلمان HITL گره‌خورده با منشور ───────────────────────── */
say("════ 1) governance — parliament HITL bound to the charter ════");
{
  const nationRs = readFileSync(path.join(ROOT, "crates/nation/src/lib.rs"), "utf8");
  const domainHitl = grepCount(path.join(ROOT, "crates/domain/src/charter.rs"), "HITL_THRESHOLD_DIN");
  check("charter is the single source of the 50,000 Đ gate", domainHitl >= 1);
  const execRs = readFileSync(path.join(ROOT, "crates/server/src/handlers/nation.rs"), "utf8");
  check(
    "nation reuses the charter gate (no duplicated threshold math)",
    /requires_hitl/.test(nationRs) &&
      /hitl_threshold_din: NationPolicy::charter_default\(\)\.hitl_threshold_din/.test(execRs),
  );
  check(
    "human vote required — one-voter-one-vote + HITL reason encoded",
    ["AlreadyVoted", "HumanApprovalMissing", "QuorumNotMet", "MajorityNo"].every((k) => nationRs.includes(k)),
  );
  check(
    "execution only from PASSED (one-way transition)",
    /NotPassed/.test(nationRs) && /status != BillStatus::Passed/.test(nationRs),
  );
  check(
    "server execution goes through the pure M2 domain path",
    /release_escrow|refund_escrow/.test(execRs) &&
      /approved_by_admin: true/.test(execRs) &&
      /TREASURY_ACCOUNT_ID/.test(execRs),
  );
  const routesRs = readFileSync(path.join(ROOT, "crates/server/src/routes.rs"), "utf8");
  check(
    "nation routes wired (open/list/get/vote/close/execute)",
    ["/nation/bills", "/nation/bills/{id}/votes", "/nation/bills/{id}/close", "/nation/bills/{id}/execute"]
      .every((r) => routesRs.includes(r)),
  );
  const t = run("cargo", ["test", "-p", "kian-nation"], { timeout: 300_000 });
  const r = testResult(t.out);
  check("kian-nation tests green", r && r.failed === 0, r ? `${r.passed} passed` : t.out.slice(0, 200));
}
say("");

/* ── ۲) کیف پول چهارزنجیره — چک‌سام‌های رسمی ────────────────────────────── */
say("════ 2) four-chain wallet — official checksums per chain ════");
{
  const walletRs = readFileSync(path.join(ROOT, "crates/wallet/src/lib.rs"), "utf8");
  const canonical = [
    "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t",
    "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",
    "EQCxE6mUtQJKFnGfaROTKOt1lZbDiiX1kCixRv7Nw2Id_sDs",
    "0x55d398326f99059fF775485246999027B3197955",
  ];
  check(
    "canonical USDT contracts on TRX/SOL/TON/BSC verified in constants",
    canonical.every((c) => walletRs.includes(c)),
  );
  check(
    "official checksum validators per chain",
    ["b58_decode", "crc16_xmodem", "eip55_checksum", "keccak256", "validate_address"].every((f) => walletRs.includes(f)),
  );
  check(
    "keccak vectors present (empty/abc/multi-block)",
    [
      "c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470",
      "4e03657aea45a94fc7d47ba826c8d667c0d1e6e33a64a036ec44f58fa12d6c45",
      "96ea54061def936c4be90b518992fdc6f12f535068a256229aca54267b4d084d",
    ].every((v) => walletRs.includes(v)),
  );
  check(
    "custody: treasury movement requires parliament bill",
    /CustodyViolation/.test(walletRs) && /bill:/.test(walletRs),
  );
  const routesRs = readFileSync(path.join(ROOT, "crates/server/src/routes.rs"), "utf8");
  check(
    "wallet chain routes wired (chains/treasury/addresses/usdt-deposits)",
    ["/wallet/chains", "/wallet/treasury", "/wallet/addresses", "/wallet/usdt-deposits"]
      .every((r) => routesRs.includes(r)),
  );
  const mig = readFileSync(path.join(ROOT, "crates/db/migrations/0004_m5_nation_wallet.sql"), "utf8");
  check("USDT deposit replay-proof (UNIQUE chain+txHash)", mig.includes("UsdtDeposit_chain_txHash_key"));
  const t = run("cargo", ["test", "-p", "kian-wallet"], { timeout: 300_000 });
  const r = testResult(t.out);
  check("kian-wallet tests green", r && r.failed === 0, r ? `${r.passed} passed` : t.out.slice(0, 200));
}
say("");

/* ── ۳) سلام کل workspace — همهٔ تست‌ها ─────────────────────────────────── */
say("════ 3) whole-workspace tests green ════");
{
  const t = run("cargo", ["test", "--workspace"], { timeout: 900_000 });
  const passed = [...t.out.matchAll(/test result: ok\. (\d+) passed/g)].reduce((a, m) => a + Number(m[1]), 0);
  const failed = [...t.out.matchAll(/test result: FAILED/g)].length;
  const err = /error\[E|error: could not compile/.test(t.out);
  check("cargo test --workspace", !err && failed === 0, `${passed} tests passed, ${failed} failing suites`);
  check("server integration surface covers M5 flows", /43 passed/.test(t.out), "kian-server lib = 43 tests");
}
say("");

/* ── ۴) clippy -D warnings صفر ─────────────────────────────────────────── */
say("════ 4) clippy -D warnings = 0 errors ════");
{
  const t = run("cargo", ["clippy", "--workspace", "--all-targets", "--", "-D", "warnings"], { timeout: 900_000 });
  const errCount = (t.out.match(/^error/gm) ?? []).length;
  check("clippy clean", t.code === 0 && errCount === 0, `${errCount} error(s)`);
}
say("");

/* ── ۵) بدون credential در سورس ────────────────────────────────────────── */
say("════ 5) no credential markers in source ════");
{
  let hits = 0;
  const scan = [
    "crates/nation/src/lib.rs",
    "crates/wallet/src/lib.rs",
    "crates/server/src/handlers/nation.rs",
    "crates/server/src/handlers/chains.rs",
    "crates/contracts/src/nation.rs",
    "crates/contracts/src/chains.rs",
    "apps/web/src",
    "apps/admin/app",
  ];
  for (const rel of scan) {
    const abs = path.join(ROOT, rel);
    if (!existsSync(abs)) continue;
    const res = spawnSync("grep", ["-rEnI", "(sk-[A-Za-z0-9]{16}|bot[0-9]{6}:[A-Za-z0-9_-]{30}|BEGIN (RSA |EC )?PRIVATE KEY|KIAN_ADMIN_KEY=)", abs], { encoding: "utf8" });
    const out = `${res.stdout ?? ""}`;
    if (out.trim()) { hits += out.trim().split("\n").length; say(`    HIT in ${rel}: ${out.trim().split("\n")[0]}`); }
  }
  check("no literal keys/tokens in M5 sources and frontends", hits === 0, `${hits} hits`);
}
say("");

/* ── ۶) audit-release-exposure روی سه آرتیفکت ──────────────────────────── */
say("════ 6) audit-release-exposure (fail_on low) ════");
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

/* ── جمع‌بندی ──────────────────────────────────────────────────────────── */
say(`════ verdict: ${failures === 0 ? "PASS — FINAL gate 10/10 green (exit 0)" : `FAIL — ${failures} failing check(s) (exit 3)`} ════`);
writeFileSync(REPORT, `${lines.join("\n")}\n`);
console.log(`\nreport: ${REPORT}`);
process.exit(failures === 0 ? 0 : 3);
