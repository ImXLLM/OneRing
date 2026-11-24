#!/usr/bin/env bash
set -euo pipefail
ROOT="${1:-.}"
python3 "$(dirname "$0")/run_rea_gate.py" --root "$ROOT" --cargo
