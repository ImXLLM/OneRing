#!/usr/bin/env python3
"""Gate 8 parity check — the 69 env keys must be index-for-index identical
(index 0..68) across .env.example, railway.env.example, and the
MAIRON_ENV_SPEC in packages/config/src/index.ts. Exit 0 = parity."""
from __future__ import annotations

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]

def env_keys(path: pathlib.Path) -> list[str]:
    keys: list[str] = []
    for line in path.read_text(encoding="utf-8").splitlines():
        s = line.strip()
        if s.startswith("#") or not s:
            continue
        m = re.match(r"([A-Z0-9_]+)\s*=", s)
        if m:
            keys.append(m.group(1))
    return keys

keys_env = env_keys(ROOT / ".env.example")
keys_railway = env_keys(ROOT / "railway.env.example")
cfg_text = (ROOT / "packages/config/src/index.ts").read_text(encoding="utf-8")
keys_config = re.findall(r'\{ key: "([A-Z0-9_]+)", tier: "P\d"', cfg_text)

ok = True
for name, keys in ((".env.example", keys_env), ("railway.env.example", keys_railway), ("packages/config/src/index.ts", keys_config)):
    print(f"{name}: {len(keys)} keys")
    if len(keys) != 69:
        ok = False
if keys_env != keys_railway:
    print("FAIL: .env.example and railway.env.example diverge")
    ok = False
if keys_env != keys_config:
    for i, (a, b) in enumerate(zip(keys_env, keys_config)):
        if a != b:
            print(f"FAIL: index {i} — env {a} != config {b}")
            break
    ok = False
if ok:
    print("OK: keys_env == keys_railway == keys_config (69 keys, index 0..68)")
sys.exit(0 if ok else 1)
