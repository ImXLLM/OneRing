#!/usr/bin/env python3
"""
probe_9router_models.py — Live health & active-model probe for KIAN v2 / decolua/9router.

Targets:
  1. Your 9router gateway (`https://9router-production-6f2225a3.up.railway.app/v1` by default,
     authenticated via `--api-key` or `NINE_ROUTER_API_KEY` env var).
  2. Public upstream zero-cost catalogs (OpenCode Zen, OpenRouter `:free`, Nous `:free`)
     so it also validates active free IDs even when Railway returns 401 without an API key.

Usage:
  NINE_ROUTER_API_KEY="sk-..." python3 probe_9router_models.py --smoke-chat
"""

import argparse
import json
import os
import sys
import urllib.error
import urllib.request
from typing import Any, Dict, List, Optional, Tuple

DEFAULT_GATEWAY = "https://9router-production-6f2225a3.up.railway.app/v1"

# Active verified OpenCode Free + fallback lanes (2026-11-23) + known retired ones to flag.
CANDIDATE_MODELS = [
    "oc/deepseek-v4.1-flash-free",
    "oc/mimo-v2.6-flash-free",
    "oc/big-pickle",
    "oc/beam-free",
    "oc/iceman-free",
    "oc/flint-40b-free",
    "oc/nemotron-3.5-super-vl-free",
    "oc/nemotron-3.5-lightning-free",
    "oc/hy4-flash-free",
    "oc/tidepool-alpha-free",
    # Retired / broken IDs kept only to verify 410/400 detection:
    "oc/longcat-2.5-free",
    "oc/glm-5.3-flash-free",
]


def http_json(
    url: str,
    method: str = "GET",
    headers: Optional[Dict[str, str]] = None,
    payload: Optional[Dict[str, Any]] = None,
    timeout: int = 20,
) -> Tuple[int, Any, str]:
    hdrs = {"User-Agent": "kian-v2-9router-probe/1.0"}
    if headers:
        hdrs.update(headers)
    data = None
    if payload is not None:
        hdrs.setdefault("Content-Type", "application/json")
        data = json.dumps(payload).encode("utf-8")
    req = urllib.request.Request(url, data=data, headers=hdrs, method=method)
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            raw = resp.read().decode("utf-8", errors="replace")
            try:
                return resp.status, json.loads(raw), raw
            except json.JSONDecodeError:
                return resp.status, None, raw
    except urllib.error.HTTPError as e:
        raw = e.read().decode("utf-8", errors="replace")
        try:
            return e.code, json.loads(raw), raw
        except json.JSONDecodeError:
            return e.code, None, raw
    except Exception as e:
        return 0, None, str(e)


def probe_opencode_upstream() -> Dict[str, Any]:
    status, data, raw = http_json(
        "https://opencode.ai/zen/v1/models",
        headers={"User-Agent": "opencode/latest/2.0.26/cli", "x-opencode-client": "cli"},
    )
    if status != 200 or not isinstance(data, dict):
        return {"status": status, "error": raw[:300], "active_free_chat": []}
    ids = [m["id"] for m in data.get("data", []) if isinstance(m, dict) and "id" in m]
    free_chat = [
        f"oc/{mid}"
        for mid in ids
        if (mid.endswith("-free") or mid == "big-pickle") and mid != "jev-1.13-free"
    ]
    return {
        "status": status,
        "total_zen_models": len(ids),
        "active_free_chat": free_chat,
        "has_retired_longcat_2_5_free": "longcat-2.5-free" in ids,
        "has_retired_glm_5_3_flash_free": "glm-5.3-flash-free" in ids,
    }


def probe_openrouter_free() -> List[str]:
    status, data, _ = http_json("https://openrouter.ai/api/v1/models")
    if status != 200 or not isinstance(data, dict):
        return []
    free = [
        m for m in data.get("data", [])
        if (m.get("id", "").endswith(":free") or m.get("id") == "openrouter/free")
        and str(m.get("pricing", {}).get("prompt")) == "0"
        and str(m.get("pricing", {}).get("completion")) == "0"
    ]
    free.sort(key=lambda x: x.get("context_length") or 0, reverse=True)
    return [f"openrouter/{m['id']} (ctx={m.get('context_length')})" for m in free[:12]]


def probe_nous_free() -> List[str]:
    status, data, _ = http_json("https://inference-api.nousresearch.com/v1/models")
    if status != 200 or not isinstance(data, dict):
        return []
    return [
        f"nous/{m['id']} (ctx={m.get('context_length')})"
        for m in data.get("data", [])
        if str(m.get("pricing", {}).get("prompt")) == "0"
        and str(m.get("pricing", {}).get("completion")) == "0"
    ]


def smoke_chat(gateway: str, api_key: str, model: str) -> Dict[str, Any]:
    # Includes non-empty reasoning_content on assistant history to mirror 9router's injector.
    payload = {
        "model": model,
        "messages": [
            {"role": "user", "content": "Reply with OK."},
        ],
        "max_tokens": 16,
        "stream": False,
    }
    status, data, raw = http_json(
        f"{gateway}/chat/completions",
        method="POST",
        headers={"Authorization": f"Bearer {api_key}"},
        payload=payload,
        timeout=35,
    )
    verdict = "OK" if 200 <= status < 300 else (
        "RETIRED_410" if status == 410 else (
            "BAD_REQ_400" if status == 400 else (
                "RATE_LIMIT_429" if status == 429 else f"HTTP_{status}"
            )
        )
    )
    snippet = ""
    if isinstance(data, dict):
        choices = data.get("choices") or []
        if choices:
            snippet = (choices[0].get("message", {}).get("content") or "").strip()[:60]
        elif "error" in data:
            snippet = str(data["error"])[:100]
    else:
        snippet = raw[:100]
    return {"model": model, "status": status, "verdict": verdict, "detail": snippet}


def main() -> int:
    parser = argparse.ArgumentParser(description="Probe 9router & live upstream free catalogs")
    parser.add_argument("--base-url", default=os.environ.get("NINE_ROUTER_BASE_URL", DEFAULT_GATEWAY))
    parser.add_argument("--api-key", default=os.environ.get("NINE_ROUTER_API_KEY", ""))
    parser.add_argument("--smoke-chat", action="store_true", help="Run 1-token chat probes (requires --api-key)")
    parser.add_argument("--json-out", default="", help="Optional path to write full JSON report")
    args = parser.parse_args()

    gateway = args.base_url.rstrip("/")
    headers = {"Authorization": f"Bearer {args.api_key}"} if args.api_key else {}

    gw_status, gw_data, gw_raw = http_json(f"{gateway}/models", headers=headers)
    gw_models: List[str] = []
    if gw_status == 200 and isinstance(gw_data, dict):
        gw_models = [m.get("id", "") for m in gw_data.get("data", []) if isinstance(m, dict)]

    report: Dict[str, Any] = {
        "gateway_url": gateway,
        "gateway_models_status": gw_status,
        "gateway_models_count": len(gw_models),
        "gateway_models_sample": gw_models[:25],
        "opencode_upstream": probe_opencode_upstream(),
        "openrouter_free_top": probe_openrouter_free(),
        "nous_free": probe_nous_free(),
        "smoke_chat_results": [],
    }

    print(f"[Gateway] {gateway}/models -> HTTP {gw_status} (models={len(gw_models)})")
    if gw_status == 401:
        print("  -> Gateway requires Bearer token; pass --api-key or set NINE_ROUTER_API_KEY for /models & --smoke-chat.")

    oc = report["opencode_upstream"]
    print(f"[OpenCode Live] HTTP {oc.get('status')} | Active chat -free models ({len(oc.get('active_free_chat', []))}):")
    for mid in oc.get("active_free_chat", []):
        print(f"  + {mid}")
    print(f"  - Retired check: longcat-2.5-free present? {oc.get('has_retired_longcat_2_5_free')}, glm-5.3-flash-free present? {oc.get('has_retired_glm_5_3_flash_free')}")

    print(f"[OpenRouter Free Live] Top {len(report['openrouter_free_top'])} zero-cost models:")
    for item in report["openrouter_free_top"][:8]:
        print(f"  + {item}")

    print(f"[Nous Free Live] ({len(report['nous_free'])}):")
    for item in report["nous_free"]:
        print(f"  + {item}")

    if args.smoke_chat:
        if not args.api_key:
            print("[Smoke Chat] Skipped: --api-key / NINE_ROUTER_API_KEY is required.", file=sys.stderr)
        else:
            print("\n[Smoke Chat Probes]")
            for m in CANDIDATE_MODELS:
                res = smoke_chat(gateway, args.api_key, m)
                report["smoke_chat_results"].append(res)
                print(f"  {res['model']:34s} | {res['verdict']:15s} | {res['detail']}")

    if args.json_out:
        with open(args.json_out, "w", encoding="utf-8") as f:
            json.dump(report, f, indent=2, ensure_ascii=False)
        print(f"\nSaved JSON report to {args.json_out}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
