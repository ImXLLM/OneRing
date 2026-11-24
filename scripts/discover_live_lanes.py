#!/usr/bin/env python3
"""Discover which 9router provider lanes have active credentials for this key.

One tiny chat request per lane (first owner segment). Output: /tmp/live_lanes.json
"""
from __future__ import annotations

import asyncio
import json
import re
import time
from collections import defaultdict
from pathlib import Path

import httpx

REPO = Path("/home/z/my-project/kian-launch/kian")
env = {}
for line in (REPO / ".env.local").read_text().splitlines():
    line = line.strip()
    if line and not line.startswith("#") and "=" in line:
        k, v = line.split("=", 1)
        env[k.strip()] = v.strip()
BASE = env["KIAN_AI_BASE_URL"].rstrip("/")
KEY = env["KIAN_AI_API_KEY"]

data = json.load(open("/tmp/models.json"))["data"]
by_owner = defaultdict(list)
for m in data:
    by_owner[m["id"].split("/", 1)[0]].append(m["id"])

BRAND = re.compile(r"(glm|qwen|kimi|claude|gpt|gemini|deepseek|grok|minimax|kat)", re.I)
PICKS = []
for owner, ids in sorted(by_owner.items()):
    branded = [i for i in ids if BRAND.search(i.split("/", 1)[1])]
    PICKS.append((owner, sorted(branded or ids, key=len)[0]))


async def probe(client: httpx.AsyncClient, sem: asyncio.Semaphore, owner: str, model: str):
    async with sem:
        t0 = time.perf_counter()
        try:
            r = await client.post(
                f"{BASE}/chat/completions",
                headers={"Authorization": f"Bearer {KEY}"},
                json={"model": model, "messages": [{"role": "user", "content": "سلام"}], "max_tokens": 8},
                timeout=40,
            )
            ms = int((time.perf_counter() - t0) * 1000)
            if r.status_code == 200:
                j = r.json()
                content = ((j.get("choices") or [{}])[0].get("message") or {}).get("content") or ""
                ok = content.strip() != ""
                err = "" if ok else "empty content"
            else:
                ok = False
                try:
                    err = (r.json().get("error") or {}).get("message", r.text[:100])
                except Exception:
                    err = r.text[:100]
            return {"owner": owner, "probe_model": model, "status": r.status_code,
                    "ms": ms, "live": ok, "err": err[:120]}
        except Exception as e:
            return {"owner": owner, "probe_model": model, "status": 0,
                    "ms": int((time.perf_counter() - t0) * 1000), "live": False,
                    "err": type(e).__name__}


async def main():
    sem = asyncio.Semaphore(8)
    async with httpx.AsyncClient() as client:
        t0 = time.perf_counter()
        results = await asyncio.gather(*[probe(client, sem, o, m) for o, m in PICKS])
    dt = time.perf_counter() - t0
    live = [r for r in results if r["live"]]
    print(f"probed {len(results)} lanes in {dt:.1f}s — LIVE: {len(live)}")
    for r in sorted(results, key=lambda x: (not x["live"], x["owner"])):
        mark = "LIVE" if r["live"] else "dead"
        print(f"{mark:<4} {r['owner']:<20} {r['status']:>3} {r['ms']:>6}ms  {r['probe_model']}"
              + (f"  | {r['err']}" if (not r["live"] and r["err"]) else ""))
    Path("/tmp/live_lanes.json").write_text(json.dumps(results, ensure_ascii=False, indent=1))
    print("\nsaved → /tmp/live_lanes.json")


if __name__ == "__main__":
    asyncio.run(main())
