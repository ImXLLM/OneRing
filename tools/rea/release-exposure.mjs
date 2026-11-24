#!/usr/bin/env node
/** دروازه REA 10/10 — هر ردیف exposed = شکست بیلد. */
import { spawnSync } from "node:child_process";

const target = process.argv[2];
if (!target) {
  console.error("usage: node tools/rea/release-exposure.mjs <artifact-path>");
  process.exit(2);
}

const res = spawnSync("rea", ["audit-release-exposure", target, "--fail_on", "low"], { stdio: "inherit" });
if (res.status !== 0) {
  console.error(`REA gate FAILED (exit ${res.status}) — exposure found. 10/10 violated.`);
  process.exit(res.status ?? 1);
}
console.log("REA gate PASSED — zero exposure. 10/10");
