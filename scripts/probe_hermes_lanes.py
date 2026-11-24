#!/usr/bin/env python3
"""probe_hermes_lanes.py — live probe of the extra free lanes discovered in
KiaAgentX/ShopifyHermes (brain.py v1.2 + models_report.csv evidence).

Lanes under test (NO OpenRouter traffic — daily quota is never burned):
  1. our 9router   (https://9router-production-6f2225a3.up.railway.app/v1, key from .env.local)
       × oc/deepseek-v4.1-flash-free, oc/glm-5.3-flash-free, oc/longcat-2.5-free
  2. sibling 9router from ShopifyHermes (https://9router-production-af2f3af2.up.railway.app/v1)
       × same 3 models, first WITHOUT key, then WITH our 9router key
  3. credential discovery: any COARENA_API_KEY / NUMES_API_KEYS in the process
       environment or under /home/z/my-project (filenames only for matches; values
       are never printed in full).

Kernel-faithful probe: NO max_tokens cap (reasoning models starve content under
small caps — see OPENROUTER_BENCHMARK_REPORT_FA.md), short Persian ping,
90s timeout, one attempt per lane.

Output: JSON summary to stdout + scripts/hermes_lane_probe.json
"""
import json
import os
import re
import sys
import time
from pathlib import Path

import httpx

REPO = Path("/home/z/my-project/kian-launch/kian")
OUT = REPO / "scripts" / "hermes_lane_probe.json"

OUR_R9 = "https://9router-production-6f2225a3.up.railway.app/v1"
SIBLING_R9 = "https://9router-production-af2f3af2.up.railway.app/v1"
OC_MODELS = [
    "oc/deepseek-v4.1-flash-free",
    "oc/glm-5.3-flash-free",
    "oc/longcat-2.5-free",
]
PING = "فقط کلمهٔ «سلام» را بنویس."
TIMEOUT = 90.0


def redact(key: str) -> str:
    """Full redaction — only the key length is reported, never any fragment."""
    if not key:
        return ""
    return f"(set, {len(key)} chars)"


def load_env_local() -> dict:
    env = {}
    path = REPO / ".env.local"
    if path.exists():
        for line in path.read_text(encoding="utf-8").splitlines():
            line = line.strip()
            if line and not line.startswith("#") and "=" in line:
                k, v = line.split("=", 1)
                env[k.strip()] = v.strip()
    return env


def probe(base: str, key: str, model: str) -> dict:
    """One chat-completions ping. Returns a lane record (never echoes the key)."""
    headers = {"Content-Type": "application/json",
               "HTTP-Referer": "https://kian.app", "X-Title": "KIAN Studio"}
    if key:
        headers["Authorization"] = f"Bearer {key}"
    body = {"model": model,
            "messages": [{"role": "user", "content": PING}]}
    t0 = time.perf_counter()
    rec = {"base": base, "model": model,
           "key": redact(key) if key else "(none)"}
    try:
        with httpx.Client(timeout=TIMEOUT) as c:
            r = c.post(f"{base.rstrip('/')}/chat/completions",
                       headers=headers, json=body)
        rec["ms"] = int((time.perf_counter() - t0) * 1000)
        rec["status"] = r.status_code
        if r.status_code == 200:
            data = r.json()
            content = (data.get("choices") or [{}])[0].get("message", {}).get("content") or ""
            rec["ok"] = bool(content.strip())
            rec["content_head"] = content.strip()[:60].replace("\n", " ")
            rec["usage"] = data.get("usage", {})
        else:
            rec["ok"] = False
            try:
                err = r.json().get("error", {})
                rec["err"] = str(err.get("code") or err.get("message") or r.text[:80])[:80]
            except Exception:
                rec["err"] = r.text[:80].replace("\n", " ")
    except Exception as e:
        rec["ms"] = int((time.perf_counter() - t0) * 1000)
        rec["ok"] = False
        rec["err"] = type(e).__name__
    return rec


def find_creds() -> dict:
    """Look for Coarena/numes credentials in env and under /home/z/my-project."""
    found = {"env": {}, "files": []}
    for var in ("COARENA_API_KEY", "NUMES_API_KEYS", "OPENAI_API_KEY"):
        v = os.environ.get(var)
        if v:
            found["env"][var] = redact(v)
    env_local = load_env_local()
    for var in ("COARENA_API_KEY", "COARENA_KEYS", "NUMES_API_KEYS"):
        if env_local.get(var):
            found["env"][f".env.local:{var}"] = redact(env_local[var])
    # filename-level sweep (never print matched values)
    pat = re.compile(r"^\s*(COARENA_API_KEY|COARENA_KEYS|NUMES_API_KEYS)\s*=", re.M)
    for root in ("/home/z/my-project",):
        for dirpath, dirnames, filenames in os.walk(root):
            dirnames[:] = [d for d in dirnames if d not in
                           (".git", "node_modules", "target", "__pycache__", ".next", ".venv")]
            for fn in filenames:
                if not fn.startswith(".env") and not fn.endswith(".env"):
                    continue
                p = Path(dirpath) / fn
                try:
                    if pat.search(p.read_text(encoding="utf-8", errors="ignore")):
                        found["files"].append(str(p))
                except Exception:
                    pass
    return found


def main() -> None:
    envl = load_env_local()
    r9_key = envl.get("KIAN_9ROUTER_API_KEY") or envl.get("KIAN_AI_TERTIARY_KEY") or ""
    lanes: list[dict] = []

    print(f"== lane probe: oc/* family on two 9router instances (no OpenRouter traffic) ==")
    # 1) our 9router with our key
    for m in OC_MODELS:
        rec = probe(OUR_R9, r9_key, m)
        lanes.append({**rec, "group": "our-r9+our-key"})
        print(f"  our-r9  {m:<34} {rec.get('status', rec.get('err'))}  {rec.get('ms', '?')}ms  ok={rec.get('ok')}")

    # 2) sibling 9router — without key, then with our key
    for m in OC_MODELS:
        rec = probe(SIBLING_R9, "", m)
        lanes.append({**rec, "group": "sibling-r9+no-key"})
        print(f"  sib-nk  {m:<34} {rec.get('status', rec.get('err'))}  {rec.get('ms', '?')}ms  ok={rec.get('ok')}")
    for m in OC_MODELS:
        rec = probe(SIBLING_R9, r9_key, m)
        lanes.append({**rec, "group": "sibling-r9+our-key"})
        print(f"  sib-k   {m:<34} {rec.get('status', rec.get('err'))}  {rec.get('ms', '?')}ms  ok={rec.get('ok')}")

    creds = find_creds()
    live = [l for l in lanes if l.get("ok")]
    summary = {
        "probed_at": time.strftime("%Y-%m-%dT%H:%M:%S%z"),
        "our_r9": OUR_R9,
        "sibling_r9": SIBLING_R9,
        "lanes": lanes,
        "live_lanes": [{"group": l["group"], "model": l["model"], "ms": l.get("ms")} for l in live],
        "coarena_numes_creds": creds,
    }
    OUT.write_text(json.dumps(summary, ensure_ascii=False, indent=2), encoding="utf-8")
    print(f"\nlive lanes: {len(live)}/{len(lanes)}")
    for l in live:
        print(f"  ✓ {l['group']}  {l['model']}  {l.get('ms')}ms")
    print(f"coarena/numes creds: env={list(creds['env'])} files={creds['files']}")
    print(f"saved: {OUT}")


if __name__ == "__main__":
    sys.exit(main())
