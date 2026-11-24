#!/usr/bin/env node
/**
 * پاکسازی REA خروجی standalone — حذف همه سورس‌مپ‌ها از آرتیفکت انتشار.
 * فیلتر REA: javascript.source_maps → "Do not ship source maps beside the release."
 * استفاده: node tools/rea/clean-standalone.mjs <standalone-dir>
 */
import { readdirSync, rmSync, statSync } from "node:fs";
import { join } from "node:path";

const root = process.argv[2];
if (!root) {
  console.error("usage: node tools/rea/clean-standalone.mjs <standalone-dir>");
  process.exit(2);
}

let removed = 0;

function walk(dir) {
  for (const entry of readdirSync(dir)) {
    const full = join(dir, entry);
    const st = statSync(full);
    if (st.isDirectory()) walk(full);
    else if (entry.endsWith(".map")) {
      rmSync(full);
      removed += 1;
    }
  }
}

walk(root);
console.log(`REA clean-standalone: removed ${removed} source map file(s) from ${root}`);
