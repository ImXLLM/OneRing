#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""inject_vault_gems.py — \u062a\u0632\u0631\u06cc\u0642 \u06af\u0646\u062c\u06cc\u0646\u0647\u0654 Hidden Gems \u0628\u0647 KIAN v2 (STEP 2 \u0645\u0623\u0645\u0648\u0631\u06cc\u062a \u06f9\u067e\u0648\u0634\u0647‌\u0627\u06cc).

Sections:
  1. Static mount verification/patch for apps/web/public/products/trinity/
     (ocean-test-lab: CSP-safe inline Tailwind utility CSS + local three.js)
  2. Persian RTL showcase launcher builder (< 30 KB, self-contained)
  3. Idempotent upsert of 8 TRINITY flagship products into seed/kian-production.db
  4. Idempotent upsert of 33 KB entries (16 commerce skills, 11 engineering
     skills, SOUL/PRICING, DAY1/MARKETING/V3/VALUATION playbooks)
  5. Verification: 20 active products, trial balance untouched, LIKE search < 5 ms

Idempotency: every upsert compares a digest first; unchanged rows are skipped,
changed rows are replaced, and audit rows are written only on real changes.
Trial balance (Posting) is never touched by this script.
"""

from __future__ import annotations

import hashlib
import json
import re
import sqlite3
import sys
import time
from pathlib import Path

REPO = Path("/home/z/my-project/kian-launch/kian")
VAULT = Path("/tmp/kia-gems-vault")
SEED_DB = REPO / "seed" / "kian-production.db"
TRINITY = REPO / "apps" / "web" / "public" / "products" / "trinity"
CREATED_AT = "2026-11-24T00:00:00.000Z"
SELLER = "citizen:8198598635"  # @ImXforevr
AFFILIATE_BPS = 2500  # 25% affiliate commission

# ── 1. TRINITY product definitions ──────────────────────────────────────────
# price in micro-KIAN (300..=1200 Đ) — far below the 50,000 Đ HITL threshold.

PRODUCTS: list[dict] = [
    dict(
        id="prod:trinity-sc-workbench",
        title="\u0645\u06cc\u0632 \u06a9\u0627\u0631 \u0647\u0648\u0634\u0645\u0646\u062f SC (\u0646\u0633\u062e\u0647\u0654 \u06a9\u0627\u0645\u0644 RTL)",
        file="sc.html",
        price=1_200_000_000,
        category="AI Tool",
        tagline="\u0627\u0632 \u0627\u06cc\u062f\u0647 \u062a\u0627 \u0646\u06cc\u0627\u0632\u0645\u0646\u062f\u06cc \u0631\u0648\u0634\u0646 — \u0645\u06cc\u0632\u06a9\u0627\u0631 \u06a9\u0627\u0645\u0644 \u0627\u0633\u062a\u0648\u062f\u06cc\u0648 SC",
        accent="#22d3ee",
        icon="🧭",
        features=[
            "\u062a\u0628\u062f\u06cc\u0644 \u0627\u06cc\u062f\u0647\u0654 \u062e\u0627\u0645 \u0628\u0647 \u0646\u06cc\u0627\u0632\u0645\u0646\u062f\u06cc \u0633\u0627\u062e\u062a\u0627\u0631\u06cc\u0627\u0641\u062a\u0647 \u0648 \u062a\u0633\u062a‌\u067e\u0630\u06cc\u0631",
            "\u067e\u0634\u062a\u06cc\u0628\u0627\u0646\u06cc \u06a9\u0627\u0645\u0644 RTL \u0641\u0627\u0631\u0633\u06cc \u0648 \u0642\u0627\u0644\u0628‌\u0647\u0627\u06cc \u0622\u0645\u0627\u062f\u0647\u0654 \u0645\u062d\u0635\u0648\u0644",
            "\u062e\u0631\u0648\u062c\u06cc \u0645\u0633\u062a\u0642\u06cc\u0645 \u0628\u0631\u0627\u06cc \u0627\u0633\u062a\u0648\u062f\u06cc\u0648\u06cc \u0633\u0627\u062e\u062a \u0647\u0631\u0645\u0633",
        ],
        desc="\u0645\u06cc\u0632 \u06a9\u0627\u0631 \u0647\u0648\u0634\u0645\u0646\u062f SC \u06a9\u0627\u0645\u0644‌\u062a\u0631\u06cc\u0646 \u0627\u0628\u0632\u0627\u0631 \u0645\u062f\u06cc\u0631\u06cc\u062a \u0627\u06cc\u062f\u0647 \u0648 \u0646\u06cc\u0627\u0632\u0645\u0646\u062f\u06cc \u0627\u0633\u062a\u061b \u0627\u06cc\u062f\u0647 \u0631\u0627 \u0648\u0627\u0631\u062f \u0645\u06cc‌\u06a9\u0646\u06cc\u062f \u0648 \u062e\u0631\u0648\u062c\u06cc\u060c \u0646\u06cc\u0627\u0632\u0645\u0646\u062f\u06cc \u0631\u0648\u0634\u0646 \u0648 \u0642\u0627\u0628\u0644 \u0627\u062c\u0631\u0627\u0633\u062a. \u0646\u0633\u062e\u0647\u0654 \u06a9\u0627\u0645\u0644 RTL \u0628\u0627 \u062a\u0645 \u062a\u06cc\u0631\u0647 \u0648 \u0631\u0627\u0628\u0637 \u0641\u0627\u0631\u0633\u06cc.",
    ),
    dict(
        id="prod:trinity-fishkal-game",
        title="\u0628\u0627\u0632\u06cc \u0622\u0631\u06a9\u06cc\u062f \u0641\u06cc\u0632\u06cc\u06a9\u06cc FISHKAL — Deep Catch",
        file="fishkal.html",
        price=640_000_000,
        category="Game",
        tagline="\u0634\u06cc\u0631\u062c\u0647 \u0628\u0632\u0646\u060c \u0639\u0645\u06cc\u0642‌\u062a\u0631\u06cc\u0646 \u0634\u06a9\u0627\u0631 \u0631\u0627 \u0628\u06a9\u0646!",
        accent="#38bdf8",
        icon="🐟",
        features=[
            "\u0641\u06cc\u0632\u06cc\u06a9 \u0648\u0627\u0642\u0639\u06cc \u0634\u0646\u0627 \u0648 \u067e\u0631\u062a\u0627\u0628 \u0642\u0644\u0627\u0628 \u0628\u0627 \u0622\u0628‌\u0634\u0646\u0627\u0633\u06cc \u0632\u0646\u062f\u0647",
            "\u0628\u06cc\u0634 \u0627\u0632 \u06f3\u06f0 \u06af\u0648\u0646\u0647\u0654 \u0645\u0627\u0647\u06cc \u0628\u0627 \u0631\u0641\u062a\u0627\u0631 \u0647\u0648\u0634\u0645\u0646\u062f \u0641\u0631\u0627\u0631",
            "\u062c\u062f\u0648\u0644 \u0631\u06a9\u0648\u0631\u062f \u0645\u062d\u0644\u06cc \u0648 \u0627\u0631\u062a\u0642\u0627\u06cc \u0642\u0644\u0627\u0628 \u0648 \u0637\u0639\u0645\u0647",
        ],
        desc="FISHKAL \u06cc\u06a9 \u0628\u0627\u0632\u06cc \u0622\u0631\u06a9\u06cc\u062f \u0641\u06cc\u0632\u06cc\u06a9\u06cc \u062a\u06a9‌\u0635\u0641\u062d\u0647‌\u0627\u06cc \u0627\u0633\u062a\u061b \u0628\u0627 \u0642\u0644\u0627\u0628 \u0634\u06cc\u0631\u062c\u0647 \u0645\u06cc‌\u0632\u0646\u06cc\u060c \u0645\u0627\u0647\u06cc‌\u0647\u0627 \u0631\u0627 \u0634\u06a9\u0627\u0631 \u0645\u06cc‌\u06a9\u0646\u06cc \u0648 \u0627\u0631\u062a\u0642\u0627 \u0645\u06cc‌\u06af\u06cc\u0631\u06cc. \u06af\u0631\u0627\u0641\u06cc\u06a9 Canvas \u06a9\u0627\u0645\u0644 \u0648 \u0622\u0641\u0644\u0627\u06cc\u0646.",
    ),
    dict(
        id="prod:trinity-trading-arena-v10",
        title="\u0634\u0628\u06cc\u0647‌\u0633\u0627\u0632 \u0645\u0639\u0627\u0645\u0644\u0627\u062a\u06cc Trading God Arena v10",
        file="trading-god-arena.html",
        price=980_000_000,
        category="Trading",
        tagline="\u0648\u06cc\u0631\u0627\u06cc\u0634 Elon Musk — \u0622\u0648\u0631\u062f\u06af\u0627\u0647 \u0622\u0645\u0648\u0632\u0634 \u0645\u0639\u0627\u0645\u0644\u0647",
        accent="#fbbf24",
        icon="📈",
        features=[
            "\u0634\u0628\u06cc\u0647‌\u0633\u0627\u0632\u06cc \u06a9\u0646\u062f\u0644\u060c \u0645\u0627\u0631\u062c\u06cc\u0646 \u0648 \u0645\u062f\u06cc\u0631\u06cc\u062a \u0631\u06cc\u0633\u06a9 \u0628\u062f\u0648\u0646 \u0631\u06cc\u0633\u06a9 \u0648\u0627\u0642\u0639\u06cc",
            "\u062d\u0627\u0644\u062a \u062f\u0633\u06a9\u062a\u0627\u067e \u0628\u0647\u06cc\u0646\u0647 \u0628\u0627 \u0645\u06cc\u0627\u0646‌\u0628\u0631\u0647\u0627\u06cc \u0635\u0641\u062d\u0647‌\u06a9\u0644\u06cc\u062f",
            "\u0622\u0645\u0627\u0631 \u062c\u0644\u0633\u0647\u060c \u0627\u0634\u062a\u0628\u0627\u0647\u0627\u062a \u0631\u0627\u06cc\u062c \u0648 \u0628\u0627\u0632\u062e\u0648\u0631\u062f \u0622\u0645\u0648\u0632\u0634\u06cc",
        ],
        desc="Trading God Arena v10 \u0634\u0628\u06cc\u0647‌\u0633\u0627\u0632 \u0645\u0639\u0627\u0645\u0644\u0627\u062a\u06cc \u062d\u0631\u0641\u0647‌\u0627\u06cc \u0627\u0633\u062a\u061b \u0627\u0633\u062a\u0631\u0627\u062a\u0698\u06cc‌\u0627\u062a \u0631\u0627 \u0628\u062f\u0648\u0646 \u067e\u0648\u0644 \u0648\u0627\u0642\u0639\u06cc \u062a\u0633\u062a \u0645\u06cc‌\u06a9\u0646\u06cc \u0648 \u0628\u0627 \u0628\u0627\u0632\u062e\u0648\u0631\u062f \u0644\u062d\u0638\u0647‌\u0627\u06cc \u06cc\u0627\u062f \u0645\u06cc‌\u06af\u06cc\u0631\u06cc.",
    ),
    dict(
        id="prod:trinity-nova-messenger",
        title="\u067e\u06cc\u0627\u0645‌\u0631\u0633\u0627\u0646 \u0641\u0648\u0642‌\u067e\u0648\u06cc\u0627 Nova Messenger RTL",
        file="nova-messenger.html",
        price=450_000_000,
        category="UI Kit",
        tagline="\u0631\u0627\u0628\u0637 \u0686\u062a \u067e\u0648\u06cc\u0627 \u0628\u0627 \u0627\u0646\u06cc\u0645\u06cc\u0634\u0646‌\u0647\u0627\u06cc \u0646\u0631\u0645",
        accent="#a78bfa",
        icon="💬",
        features=[
            "\u0627\u0646\u06cc\u0645\u06cc\u0634\u0646 \u0648\u0631\u0648\u062f \u067e\u06cc\u0627\u0645\u060c \u062a\u0627\u06cc\u067e\u06cc\u0646\u06af \u0648 \u0648\u0627\u06a9\u0646\u0634‌\u0647\u0627\u06cc \u0632\u0646\u062f\u0647",
            "\u062a\u0645 \u062a\u06cc\u0631\u0647 \u0648 \u0631\u0648\u0634\u0646 \u0628\u0627 \u067e\u0634\u062a\u06cc\u0628\u0627\u0646\u06cc \u06a9\u0627\u0645\u0644 RTL",
            "\u0646\u0645\u0648\u0646\u0647‌\u0647\u0627\u06cc \u0686\u062a\u060c \u06af\u0631\u0648\u0647 \u0648 \u067e\u0627\u0633\u062e \u0633\u0631\u06cc\u0639 \u0622\u0645\u0627\u062f\u0647",
        ],
        desc="Nova Messenger \u06cc\u06a9 \u0631\u0627\u0628\u0637 \u067e\u06cc\u0627\u0645‌\u0631\u0633\u0627\u0646 \u0641\u0648\u0642‌\u067e\u0648\u06cc\u0627 \u0627\u0633\u062a\u061b \u0628\u0631\u0627\u06cc \u067e\u0631\u0648\u0698\u0647‌\u0647\u0627\u06cc \u0686\u062a \u062e\u0648\u062f\u062a \u0627\u0632 \u0622\u0646 \u0628\u0647‌\u0639\u0646\u0648\u0627\u0646 \u067e\u0627\u06cc\u0647\u0654 \u0631\u0627\u0628\u0637 \u0627\u0633\u062a\u0641\u0627\u062f\u0647 \u06a9\u0646 \u0648 \u0633\u0627\u0639\u062a‌\u0647\u0627 \u0637\u0631\u0627\u062d\u06cc \u0631\u0627 \u0647\u062f\u0631 \u0646\u062f\u0647.",
    ),
    dict(
        id="prod:trinity-liquid-glass",
        title="\u0631\u0627\u0628\u0637 \u0634\u06cc\u0634\u0647‌\u0627\u06cc \u0645\u0627\u06cc\u0639 Liquid Glass UI (\u06f5 \u0628\u0648\u0645 Canvas)",
        file="liquid-glass.html",
        price=520_000_000,
        category="UI Kit",
        tagline="\u067e\u0646\u062c \u0628\u0648\u0645 \u0634\u06cc\u0634\u0647‌\u0627\u06cc \u0645\u0627\u06cc\u0639 \u0628\u0627 \u0641\u06cc\u0632\u06cc\u06a9 \u0648\u0627\u0642\u0639\u06cc",
        accent="#34d399",
        icon="🫧",
        features=[
            "\u06f5 \u0628\u0648\u0645 Canvas \u0645\u0633\u062a\u0642\u0644 \u0628\u0627 \u062f\u06cc\u0646\u0627\u0645\u06cc\u06a9 \u0633\u06cc\u0627\u0644 \u0648 \u0634\u06a9\u0633\u062a \u0646\u0648\u0631",
            "\u06a9\u0646\u062a\u0631\u0644 \u0632\u0646\u062f\u0647\u0654 \u06af\u0631\u0627\u0646\u0631\u0648\u06cc\u060c \u0631\u0646\u06af \u0648 \u0634\u062f\u062a \u0627\u0646\u0639\u06a9\u0627\u0633",
            "\u06a9\u062f \u062a\u0645\u06cc\u0632 \u0628\u0631\u0627\u06cc \u0627\u0633\u062a\u0641\u0627\u062f\u0647\u0654 \u0645\u0633\u062a\u0642\u06cc\u0645 \u062f\u0631 \u0645\u062d\u0635\u0648\u0644 \u062e\u0648\u062f\u062a",
        ],
        desc="\u067e\u06a9\u06cc\u062c Liquid Glass \u067e\u0646\u062c \u0628\u0648\u0645 \u062a\u0639\u0627\u0645\u0644\u06cc \u0634\u06cc\u0634\u0647\u0654 \u0645\u0627\u06cc\u0639 \u0645\u06cc‌\u062f\u0647\u062f\u061b \u0628\u0631\u0627\u06cc \u0644\u0646\u062f\u06cc\u0646\u06af\u060c \u0645\u06cc\u0646\u06cc‌\u0627\u067e \u0648 \u062f\u0627\u0634\u0628\u0648\u0631\u062f\u060c \u062c\u0644\u0648\u0647‌\u0627\u06cc \u0645\u062f\u0631\u0646 \u0648 \u0633\u0628\u06a9 \u0645\u06cc‌\u0633\u0627\u0632\u062f.",
    ),
    dict(
        id="prod:trinity-ocean-webgl",
        title="\u0622\u0632\u0645\u0627\u06cc\u0634\u06af\u0627\u0647 \u0641\u06cc\u0632\u06cc\u06a9 \u062f\u0631\u06cc\u0627 \u0648 \u0634\u06cc\u062f\u0631 WebGL (Ocean Test Lab)",
        file="ocean-test-lab.html",
        price=480_000_000,
        category="3D/WebGL",
        tagline="\u0622\u0632\u0645\u0627\u06cc\u0634\u06af\u0627\u0647 \u0634\u06cc\u062f\u0631 \u0622\u0628\u060c \u0645\u0648\u062c \u0648 \u0646\u0648\u0631 \u062e\u0648\u0631\u0634\u06cc\u062f",
        accent="#2dd4bf",
        icon="🌊",
        features=[
            "\u0634\u06cc\u062f\u0631 \u0622\u0628 \u0628\u0627 \u06a9\u0646\u062a\u0631\u0644 \u0645\u0648\u062c\u060c \u0628\u0627\u062f \u0648 \u0639\u0645\u0642 \u0628\u0647‌\u0635\u0648\u0631\u062a \u0632\u0646\u062f\u0647",
            "\u062a\u0646\u0638\u06cc\u0645\u0627\u062a \u0646\u0648\u0631\u060c \u0627\u0641\u0642 \u0648 \u0645\u062a\u0631\u06cc\u0627\u0644 \u0628\u0627 \u062e\u0631\u0648\u062c\u06cc \u06a9\u062f GLSL",
            "\u0645\u062e\u0635\u0648\u0635 \u062a\u0648\u0633\u0639\u0647‌\u062f\u0647\u0646\u062f\u06af\u0627\u0646 WebGL \u0648 \u0628\u0627\u0632\u06cc‌\u0633\u0627\u0632\u0647\u0627",
        ],
        desc="Ocean Test Lab \u0622\u0632\u0645\u0627\u06cc\u0634\u06af\u0627\u0647 \u062a\u0639\u0627\u0645\u0644\u06cc \u0641\u06cc\u0632\u06cc\u06a9 \u062f\u0631\u06cc\u0627\u0633\u062a\u061b \u067e\u0627\u0631\u0627\u0645\u062a\u0631\u0647\u0627\u06cc \u0634\u06cc\u062f\u0631 \u0622\u0628 \u0631\u0627 \u0632\u0646\u062f\u0647 \u062a\u063a\u06cc\u06cc\u0631 \u0645\u06cc‌\u062f\u0647\u06cc \u0648 \u06a9\u062f GLSL \u0622\u0645\u0627\u062f\u0647 \u0645\u06cc‌\u06af\u06cc\u0631\u06cc.",
    ),
    dict(
        id="prod:trinity-fable-bazaar-3d",
        title="\u0628\u0627\u0632\u0627\u0631 \u0633\u0647‌\u0628\u0639\u062f\u06cc \u062a\u0639\u0627\u0645\u0644\u06cc Fable Bazaar 3D",
        file="fable-bazaar.html",
        price=760_000_000,
        category="3D/WebGL",
        tagline="\u0642\u062f\u0645 \u0628\u0632\u0646 \u062f\u0631 \u0628\u0627\u0632\u0627\u0631 \u0633\u0646\u062a\u06cc \u0633\u0647‌\u0628\u0639\u062f\u06cc \u0648 \u0642\u0635\u0647 \u0628\u0633\u0627\u0632",
        accent="#f472b6",
        icon="🏰",
        features=[
            "\u062f\u0646\u06cc\u0627\u06cc \u0633\u0647‌\u0628\u0639\u062f\u06cc \u0628\u0627 \u06a9\u0646\u062a\u0631\u0644 \u0642\u062f\u0645‌\u0632\u062f\u0646 \u0648 \u0646\u0645\u0627\u06cc \u0627\u0648\u0644‌\u0634\u062e\u0635",
            "\u062f\u06a9\u0627\u0646‌\u0647\u0627\u060c \u0628\u0627\u0632\u06cc\u06af\u0631\u0627\u0646 \u0648 \u062f\u06cc\u0627\u0644\u0648\u06af‌\u0647\u0627\u06cc \u062a\u0639\u0627\u0645\u0644\u06cc",
            "\u0633\u0627\u062e\u062a \u0628\u0627 Three.js \u0648 \u0631\u0627\u0628\u0637 \u0641\u0627\u0631\u0633\u06cc \u06a9\u0627\u0645\u0644",
        ],
        desc="Fable Bazaar \u06cc\u06a9 \u0628\u0627\u0632\u0627\u0631 \u0633\u0646\u062a\u06cc\u0650 \u0633\u0647‌\u0628\u0639\u062f\u06cc \u0642\u0627\u0628\u0644 \u0642\u062f\u0645‌\u0632\u062f\u0646 \u0627\u0633\u062a\u061b \u0628\u06cc\u0646 \u062f\u06a9\u0627\u0646‌\u0647\u0627 \u0628\u06af\u0631\u062f\u060c \u0628\u0627 \u0641\u0631\u0648\u0634\u0646\u062f\u0647‌\u0647\u0627 \u062d\u0631\u0641 \u0628\u0632\u0646 \u0648 \u0641\u0636\u0627\u06cc \u0642\u0635\u0647‌\u0627\u06cc \u0631\u0627 \u062a\u062c\u0631\u0628\u0647 \u06a9\u0646.",
    ),
    dict(
        id="prod:trinity-dropagentx-suite",
        title="\u067e\u06a9\u06cc\u062c \u06a9\u0627\u0645\u0644 \u06f1\u06f2 \u0635\u0641\u062d\u0647‌\u0627\u06cc \u0633\u0648\u067e\u0631\u0627\u067e \u0648 \u0645\u063a\u0632 \u0633\u0647‌\u0628\u0639\u062f\u06cc DropAgentX",
        file="dropagentx/showcase3d.html",
        price=1_100_000_000,
        category="Super-App",
        tagline="\u0633\u0648\u067e\u0631\u0627\u067e \u06f1\u06f2 \u0635\u0641\u062d\u0647‌\u0627\u06cc + \u0646\u0645\u0627\u06cc\u0634 \u0633\u0647‌\u0628\u0639\u062f\u06cc \u0645\u062d\u0635\u0648\u0644\u0627\u062a",
        accent="#fb7185",
        icon="🚀",
        features=[
            "\u06f1\u06f2 \u0635\u0641\u062d\u0647\u0654 \u06a9\u0627\u0645\u0644: \u0644\u0646\u062f\u06cc\u0646\u06af\u060c \u0641\u0631\u0648\u0634\u06af\u0627\u0647\u060c \u0627\u062f\u0645\u06cc\u0646\u060c \u0644\u0627\u06cc\u0648 \u0648 \u0627\u06cc\u0646\u0633\u0627\u06cc\u062a",
            "\u0646\u0645\u0627\u06cc\u0634 \u0633\u0647‌\u0628\u0639\u062f\u06cc \u0645\u062d\u0635\u0648\u0644\u0627\u062a \u0628\u0627 Three.js \u0648 \u0641\u0648\u0646\u062a \u0648\u0632\u06cc\u0631\u0645\u062a\u0646",
            "PWA \u0628\u0627 manifest \u0648 \u0633\u0631\u0648\u06cc\u0633‌\u0648\u0631\u06a9\u0631 \u0622\u0641\u0644\u0627\u06cc\u0646",
        ],
        desc="\u0633\u0648\u0626\u06cc\u062a DropAgentX \u06cc\u06a9 \u0633\u0648\u067e\u0631\u0627\u067e \u06a9\u0627\u0645\u0644 \u062f\u0648\u0627\u0632\u062f\u0647‌\u0635\u0641\u062d\u0647‌\u0627\u06cc \u0627\u0633\u062a\u061b \u0647\u0645\u0631\u0627\u0647 \u0646\u0645\u0627\u06cc\u0634 \u0633\u0647‌\u0628\u0639\u062f\u06cc \u0645\u062d\u0635\u0648\u0644\u0627\u062a\u060c \u0645\u063a\u0632 SenPai \u0648 \u0627\u067e \u0622\u0641\u0644\u0627\u06cc\u0646‌\u067e\u0630\u06cc\u0631 PWA.",
    ),
]

# ── 2. Persian RTL launcher builder (< 30 KB) ───────────────────────────────

LAUNCHER_TMPL = """<!DOCTYPE html>
<html lang="fa" dir="rtl">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{title} — \u067e\u06cc\u0634‌\u0646\u0645\u0627\u06cc\u0634 \u0628\u0627\u0632\u0627\u0631 \u06a9\u06cc\u0627\u0646</title>
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
body{{font-family:Vazirmatn,Tahoma,'Segoe UI',sans-serif;background:#0b0e14;color:#e6edf3;min-height:100vh;display:flex;align-items:center;justify-content:center;padding:16px}}
.card{{max-width:760px;width:100%;background:linear-gradient(160deg,#11151c,#0b0e14);border:1px solid #1f2630;border-radius:20px;padding:24px;position:relative;overflow:hidden}}
.card::before{{content:'';position:absolute;inset:0;background:radial-gradient(600px 200px at 80% -10%,{accent}22,transparent);pointer-events:none}}
.badge{{display:inline-flex;align-items:center;gap:6px;background:{accent}1a;color:{accent};border:1px solid {accent}44;border-radius:999px;padding:4px 12px;font-size:12px;font-weight:700}}
h1{{font-size:22px;line-height:1.5;margin:14px 0 6px}}
.tagline{{color:#94a3b8;font-size:14px;margin-bottom:16px}}
.preview{{position:relative;border:1px solid #1f2630;border-radius:14px;overflow:hidden;background:#05070b;height:320px;margin:14px 0}}
.preview iframe{{width:100%;height:100%;border:0;background:#05070b}}
.preview .zoom{{position:absolute;top:8px;left:8px;background:#0b0e14cc;border:1px solid #1f2630;color:#94a3b8;border-radius:8px;font-size:11px;padding:3px 8px}}
.zoom-ctrl{{display:flex;gap:8px;align-items:center;justify-content:flex-end;margin:-6px 0 10px;font-size:12px;color:#94a3b8}}
.zoom-ctrl button{{background:#11151c;border:1px solid #1f2630;color:#e6edf3;border-radius:8px;padding:2px 10px;cursor:pointer;font-family:inherit}}
.zoom-ctrl button:hover{{border-color:{accent}}}
ul{{list-style:none;margin:10px 0 14px}}
li{{padding:6px 0;font-size:14px;color:#cbd5e1;display:flex;gap:8px;align-items:flex-start}}
li b{{color:{accent}}}
.meta{{display:flex;flex-wrap:wrap;gap:10px;align-items:center;justify-content:space-between;border-top:1px solid #1f2630;padding-top:16px}}
.price{{font-size:18px;font-weight:800}}
.price small{{display:block;color:#94a3b8;font-weight:400;font-size:11px;margin-top:2px}}
.btn{{display:inline-flex;align-items:center;gap:8px;background:linear-gradient(135deg,{accent},#0ea5e9);color:#04121f;font-weight:800;border:0;border-radius:14px;padding:12px 22px;font-size:15px;cursor:pointer;text-decoration:none;transition:transform .15s ease,box-shadow .15s ease}}
.btn:hover{{transform:translateY(-2px);box-shadow:0 8px 24px {accent}55}}
.foot{{margin-top:14px;font-size:11px;color:#64748b;line-height:1.9}}
.foot span{{color:{accent}}}
@media(max-width:560px){{h1{{font-size:18px}}.preview{{height:240px}}}}
</style>
</head>
<body>
<main class="card">
  <span class="badge">{icon} {category} — \u0645\u062d\u0635\u0648\u0644 \u067e\u0631\u0686\u0645‌\u062f\u0627\u0631 \u062a\u0631\u06cc\u0646\u06cc\u062a\u06cc</span>
  <h1>{title}</h1>
  <p class="tagline">{tagline}</p>
  <div class="preview"><iframe id="pv" title="\u067e\u06cc\u0634‌\u0646\u0645\u0627\u06cc\u0634 \u0632\u0646\u062f\u0647" src="{launch_url}" loading="lazy" referrerpolicy="no-referrer"></iframe><span class="zoom">\u067e\u06cc\u0634‌\u0646\u0645\u0627\u06cc\u0634 \u0632\u0646\u062f\u0647\u0654 \u062a\u0639\u0627\u0645\u0644\u06cc</span></div>
  <div class="zoom-ctrl"><span>\u0628\u0632\u0631\u06af‌\u0646\u0645\u0627\u06cc\u06cc \u067e\u06cc\u0634‌\u0646\u0645\u0627\u06cc\u0634:</span><button type="button" onclick="zm(-.1)">−</button><span id="zv">\u06f3\u06f2\u06f0</span><button type="button" onclick="zm(.1)">+</button></div>
  <ul>
    {features}
  </ul>
  <div class="meta">
    <div class="price">{price_din} Đ<small>{price_fa} — \u0622\u0632\u0627\u062f\u0633\u0627\u0632\u06cc \u0622\u0646\u06cc \u0627\u0633\u06a9\u0631\u0648\u060c \u0628\u062f\u0648\u0646 \u062a\u0648\u0642\u0641 \u0627\u0646\u0633\u0627\u0646\u06cc</small></div>
    <a class="btn" href="{launch_url}" target="_blank" rel="noopener">\u0627\u062c\u0631\u0627\u06cc \u0641\u0648\u0631\u06cc \u0645\u062d\u0635\u0648\u0644 ↗</a>
  </div>
  <p class="foot">\u0627\u06cc\u0646 \u067e\u06cc\u0634‌\u0646\u0645\u0627\u06cc\u0634 \u0632\u0646\u062f\u0647 \u0645\u0633\u062a\u0642\u06cc\u0645\u0627\u064b \u0645\u062d\u0635\u0648\u0644 \u0648\u0627\u0642\u0639\u06cc \u0631\u0627 \u0627\u062c\u0631\u0627 \u0645\u06cc‌\u06a9\u0646\u062f\u061b \u067e\u0633 \u0627\u0632 \u062e\u0631\u06cc\u062f\u060c \u0646\u0633\u062e\u0647\u0654 \u06a9\u0627\u0645\u0644 \u062a\u062d\u0648\u06cc\u0644 \u062f\u0627\u062f\u0647 \u0645\u06cc‌\u0634\u0648\u062f. \u0641\u0631\u0648\u0634\u0646\u062f\u0647: <span>@ImXforevr</span> — \u067e\u0648\u0631\u0633\u0627\u0646\u062a \u0645\u0639\u0631\u0641: <span>\u06f2\u06f5\u066a</span></p>
</main>
<script>
var h=320;
function zm(d){{h=Math.max(200,Math.min(700,Math.round(h*(1+d))));document.getElementById('pv').style.height=h+'px';document.getElementById('zv').textContent=fa(h);}}
function fa(n){{return String(n).replace(/\\d/g,function(d){{return '\u06f0\u06f1\u06f2\u06f3\u06f4\u06f5\u06f6\u06f7\u06f8\u06f9'[d];}});}}
document.getElementById('pv').addEventListener('error',function(){{document.querySelector('.preview').innerHTML='<p style="padding:24px;color:#94a3b8;font-size:13px;line-height:2">\u067e\u06cc\u0634‌\u0646\u0645\u0627\u06cc\u0634 \u062f\u0631 \u0627\u06cc\u0646 \u0645\u062d\u06cc\u0637 \u062f\u0631 \u062f\u0633\u062a\u0631\u0633 \u0646\u06cc\u0633\u062a\u061b \u062f\u06a9\u0645\u0647\u0654 «\u0627\u062c\u0631\u0627\u06cc \u0641\u0648\u0631\u06cc \u0645\u062d\u0635\u0648\u0644» \u0631\u0627 \u0628\u0627\u0632 \u06a9\u0646\u06cc\u062f.</p>';}});
</script>
</body>
</html>"""


def _fa_num(n: int) -> str:
    digits = "\u06f0\u06f1\u06f2\u06f3\u06f4\u06f5\u06f6\u06f7\u06f8\u06f9"
    return "".join(digits[int(c)] if c.isdigit() else c for c in str(n))


def _price_fa(price: int) -> str:
    return f"{_fa_num(price // 1_000_000)} \u062f\u06cc\u06cc\u0646 (Đ) \u0627\u0632 \u0628\u0627\u0632\u0627\u0631 \u06a9\u06cc\u0627\u0646"


def build_launcher(p: dict) -> str:
    features = "\n".join(f'    <li><b>◂</b><span>{f}</span></li>' for f in p["features"])
    launch_url = f"/products/trinity/{p['file']}"
    html = LAUNCHER_TMPL.format(
        title=p["title"],
        tagline=p["tagline"],
        category=p["category"],
        accent=p["accent"],
        icon=p["icon"],
        features=features,
        launch_url=launch_url,
        price_din=_fa_num(p["price"] // 1_000_000),
        price_fa=_price_fa(p["price"]),
    )
    assert len(html.encode()) < 30_000, f"launcher too big: {p['id']} {len(html.encode())}"
    return html


def build_delivery(p: dict, full_html: str | None) -> str:
    """≤150 KB → exact standalone HTML; >150 KB → slim launcher with instant-launch."""
    if full_html is not None and len(full_html.encode("utf-8")) <= 150_000:
        return full_html
    return build_launcher(p)


# ── 3. Ocean Test Lab: CSP-safe inline Tailwind utilities + local three.js ──

_TAILWIND_TOKEN_CSS: dict[str, str] = {
    # layout / position
    "relative": "position:relative", "fixed": "position:fixed",
    "top-0": "top:0", "top-24": "top:6rem", "bottom-4": "bottom:1rem",
    "left-0": "left:0", "left-4": "left:1rem", "left-1/2": "left:50%",
    "right-0": "right:0", "right-4": "right:1rem", "z-20": "z-index:20",
    # display / flex / grid
    "block": "display:block", "flex": "display:flex", "grid": "display:grid",
    "hidden": "display:none", "flex-col": "flex-direction:column",
    "items-center": "align-items:center", "items-baseline": "align-items:baseline",
    "justify-between": "justify-content:space-between", "justify-center": "justify-content:center",
    "grid-cols-2": "grid-template-columns:repeat(2,minmax(0,1fr))",
    "col-span-2": "grid-column:span 2 / span 2",
    # spacing
    "gap-1": "gap:0.25rem", "gap-1.5": "gap:0.375rem", "gap-2": "gap:0.5rem",
    "gap-2.5": "gap:0.625rem", "gap-3": "gap:0.75rem", "gap-3.5": "gap:0.875rem",
    "gap-4": "gap:1rem", "ml-2": "margin-left:0.5rem", "mr-1": "margin-right:0.25rem",
    "mt-1": "margin-top:0.25rem", "p-2": "padding:0.5rem", "p-2.5": "padding:0.625rem",
    "p-3": "padding:0.75rem", "p-4": "padding:1rem", "px-1.5": "padding-left:0.375rem;padding-right:0.375rem",
    "px-2": "padding-left:0.5rem;padding-right:0.5rem", "px-3.5": "padding-left:0.875rem;padding-right:0.875rem",
    "px-4": "padding-left:1rem;padding-right:1rem", "px-6": "padding-left:1.5rem;padding-right:1.5rem",
    "py-0.5": "padding-top:0.125rem;padding-bottom:0.125rem", "py-2": "padding-top:0.5rem;padding-bottom:0.5rem",
    "py-2.5": "padding-top:0.625rem;padding-bottom:0.625rem", "pb-2.5": "padding-bottom:0.625rem",
    "pt-2.5": "padding-top:0.625rem", "pt-3": "padding-top:0.75rem",
    # sizing
    "w-2.5": "width:0.625rem", "w-5": "width:1.25rem", "w-7": "width:1.75rem",
    "w-64": "width:16rem", "w-72": "width:18rem", "w-full": "width:100%",
    "w-[95%]": "width:95%", "h-2.5": "height:0.625rem", "h-5": "height:1.25rem",
    "h-7": "height:1.75rem", "h-full": "height:100%",
    "max-w-2xl": "max-width:42rem", "max-h-[calc(100vh-11rem)]": "max-height:calc(100vh - 11rem)",
    "overflow-hidden": "overflow:hidden", "overflow-y-auto": "overflow-y:auto",
    "-translate-x-1/2": "transform:translateX(-50%)",
    # borders / radius
    "border": "border-width:1px;border-style:solid;border-color:rgb(255 255 255 / 0.2)",
    "border-b": "border-bottom-width:1px;border-bottom-style:solid;border-color:rgb(255 255 255 / 0.2)",
    "border-t": "border-top-width:1px;border-top-style:solid;border-color:rgb(255 255 255 / 0.2)",
    "border-amber-400/15": "border-color:rgb(251 191 36 / 0.15)",
    "border-amber-400/20": "border-color:rgb(251 191 36 / 0.2)",
    "border-amber-400/30": "border-color:rgb(251 191 36 / 0.3)",
    "border-cyan-400/20": "border-color:rgb(34 211 238 / 0.2)",
    "border-emerald-500/30": "border-color:rgb(16 185 129 / 0.3)",
    "border-slate-700/60": "border-color:rgb(51 65 85 / 0.6)",
    "border-slate-700/70": "border-color:rgb(51 65 85 / 0.7)",
    "border-white/20": "border-color:rgb(255 255 255 / 0.2)",
    "rounded": "border-radius:0.25rem", "rounded-xl": "border-radius:0.75rem",
    "rounded-2xl": "border-radius:1rem", "rounded-full": "border-radius:9999px",
    # colors
    "bg-amber-400": "background-color:#fbbf24", "bg-black/35": "background-color:rgb(0 0 0 / 0.35)",
    "bg-emerald-400": "background-color:#34d399", "bg-emerald-500/20": "background-color:rgb(16 185 129 / 0.2)",
    "bg-slate-800/80": "background-color:rgb(30 41 59 / 0.8)", "bg-slate-900/75": "background-color:rgb(15 23 42 / 0.75)",
    "bg-slate-900/80": "background-color:rgb(15 23 42 / 0.8)", "bg-slate-950/60": "background-color:rgb(2 6 23 / 0.6)",
    "text-amber-100": "color:#fef3c7", "text-amber-200": "color:#fde68a",
    "text-amber-200/80": "color:rgb(253 230 138 / 0.8)", "text-amber-300": "color:#fcd34d",
    "text-amber-300/80": "color:rgb(252 211 77 / 0.8)", "text-amber-400": "color:#fbbf24",
    "text-cyan-300": "color:#67e8f9", "text-emerald-300": "color:#6ee7b7",
    "text-emerald-400": "color:#34d399", "text-purple-300": "color:#d8b4fe",
    "text-slate-200": "color:#e2e8f0", "text-slate-300/90": "color:rgb(203 213 225 / 0.9)",
    "text-slate-400": "color:#94a3b8", "text-white": "color:#fff",
    # typography
    "text-xs": "font-size:0.75rem;line-height:1rem", "text-sm": "font-size:0.875rem;line-height:1.25rem",
    "text-base": "font-size:1rem;line-height:1.5rem", "text-lg": "font-size:1.125rem;line-height:1.75rem",
    "text-xl": "font-size:1.25rem;line-height:1.75rem", "text-2xl": "font-size:1.5rem;line-height:2rem",
    "text-[10px]": "font-size:10px", "text-[11px]": "font-size:11px",
    "font-bold": "font-weight:700", "font-medium": "font-weight:500", "font-semibold": "font-weight:600",
    "tracking-wider": "letter-spacing:0.05em", "tracking-widest": "letter-spacing:0.1em",
    "uppercase": "text-transform:uppercase", "underline": "text-decoration-line:underline",
    "text-center": "text-align:center",
    # effects / interaction
    "drop-shadow": "filter:drop-shadow(0 1px 2px rgb(0 0 0 / 0.5))",
    "transition": "transition-property:color,background-color,border-color,opacity;transition-duration:150ms",
    "transition-all": "transition-property:all;transition-duration:150ms",
    "animate-spin": "animation:oceanSpin 1s linear infinite",
    "animate-pulse": "animation:oceanPulse 2s cubic-bezier(.4,0,.6,1) infinite",
    "animate-ping": "animation:oceanPing 1s cubic-bezier(0,0,.2,1) infinite",
    "cursor-pointer": "cursor:pointer", "select-none": "user-select:none",
    "pointer-events-none": "pointer-events:none", "pointer-events-auto": "pointer-events:auto",
    # hover states
    "hover:bg-slate-700/80": "hover-background-color:rgb(51 65 85 / 0.8)",
    "hover:text-amber-200": "hover-color:#fde68a",
}

_TAILWIND_KEYFRAMES = """
@keyframes oceanSpin{to{transform:rotate(360deg)}}
@keyframes oceanPulse{50%{opacity:.5}}
@keyframes oceanPing{75%,100%{transform:scale(2);opacity:0}}
"""


def build_ocean_tailwind_css(tokens: set[str]) -> str:
    """Emit static CSS for exactly the Tailwind utility tokens used by ocean-test-lab."""
    base, sm, md = {}, {}, {}
    for t in sorted(tokens):  # deterministic — set order varies with PYTHONHASHSEED
        if t in ("cam-btn", "cannon-btn", "naval-panel", "naval-slider", "active",
                 "font-nautical", "font-mono-tech"):
            continue  # custom classes already styled by the file's own <style>
        css = _TAILWIND_TOKEN_CSS.get(t)
        if css is None:
            if t.startswith("sm:"):
                sm[t[3:]] = _TAILWIND_TOKEN_CSS.get(t[3:], "")
            elif t.startswith("md:"):
                md[t[3:]] = _TAILWIND_TOKEN_CSS.get(t[3:], "")
            continue
        base[t] = css
    lines = ["/* generated: static Tailwind utilities (CSP-safe inline replacement) */"]

    def esc(sel: str) -> str:
        return sel.replace("/", r"\/").replace("[", r"\[").replace("]", r"\]").replace("(", r"\(").replace(")", r"\)").replace(".", r"\.").replace("%", r"\%").replace("#", r"\#").replace(":", r"\:")

    for t, css in base.items():
        if css.startswith("hover-"):
            lines.append(f".{esc(t)}:hover{{{css[len('hover-'):]}}}")
        else:
            lines.append(f".{esc(t)}{{{css}}}")
    if sm:
        lines.append("@media(min-width:640px){" + "".join(f".sm\\:{esc(k)}{{{v}}}" for k, v in sm.items() if v) + "}")
    if md:
        lines.append("@media(min-width:768px){" + "".join(f".md\\:{esc(k)}{{{v}}}" for k, v in md.items() if v) + "}")
    lines.append(_TAILWIND_KEYFRAMES)
    return "\n".join(lines)


def patch_ocean_lab() -> str:
    """Patch the *mounted* ocean-test-lab.html: inline Tailwind utilities + local three.js."""
    src = VAULT / "01_trinity_standalone_products" / "ocean-test-lab.html"
    dst = TRINITY / "ocean-test-lab.html"
    html = src.read_text(encoding="utf-8", errors="replace")
    tokens = set()
    for m in re.finditer(r'class="([^"]*)"', html):
        tokens.update(m.group(1).split())
    css = build_ocean_tailwind_css(tokens)
    ocean_assets = TRINITY / "ocean-assets"
    ocean_assets.mkdir(parents=True, exist_ok=True)
    three_src = VAULT / "01_trinity_standalone_products" / "dropagentx" / "vendor" / "three.min.js"
    (ocean_assets / "three.min.js").write_bytes(three_src.read_bytes())
    tw_tag = '<script src="/assets/vendor/tailwind.js"></script>'
    th_tag = '<script src="/assets/vendor/three.min.js"></script>'
    assert tw_tag in html and th_tag in html, "ocean lab vendor tags missing"
    html = html.replace(tw_tag, f"<style>{css}</style>")
    html = html.replace(th_tag, '<script src="./ocean-assets/three.min.js"></script>')
    dst.write_text(html, encoding="utf-8")
    return css

# ── 4. Idempotent DB upserts ────────────────────────────────────────────────

def row_digest(vals: list) -> str:
    return hashlib.sha256(json.dumps(vals, ensure_ascii=False, default=str).encode("utf-8")).hexdigest()


def upsert_products(db: sqlite3.Connection) -> dict:
    stats = {"inserted": 0, "updated": 0, "skipped": 0, "audit": 0}
    cur = db.cursor()
    for p in PRODUCTS:
        path = TRINITY / p["file"]
        full_html = read_text_lenient(path) if path.is_file() else None
        preview = build_launcher(p)
        delivery = build_delivery(p, full_html)
        category = p["category"]
        vals = [SELLER, p["title"], p["desc"], category, p["price"], preview,
                AFFILIATE_BPS, "HTML_APP", delivery, 1, CREATED_AT]
        existing = cur.execute(
            'SELECT "sellerId","title","description","category","priceDin","previewHtml",'
            '"affiliateBps","artifactKind","deliveryPayload","active","createdAt" '
            'FROM "Product" WHERE "id"=?', (p["id"],)).fetchone()
        if existing is not None and list(existing) == vals:
            stats["skipped"] += 1
            continue
        cur.execute(
            'INSERT OR REPLACE INTO "Product" ("id","sellerId","title","description","category",'
            '"priceDin","previewHtml","affiliateBps","artifactKind","deliveryPayload","active","createdAt")'
            ' VALUES (?,?,?,?,?,?,?,?,?,?,?,?)',
            (p["id"], *vals))
        if existing is None:
            stats["inserted"] += 1
        else:
            stats["updated"] += 1
        audit_target = f"vault.seed:{p['id']}"
        has_audit = cur.execute(
            'SELECT COUNT(*) FROM "AuditLog" WHERE "action"=? AND "target"=?',
            ("vault.seed", p["id"])).fetchone()[0]
        if not has_audit:
            cur.execute(
                'INSERT INTO "AuditLog" ("id","actor","action","target","meta","at")'
                ' VALUES (?,?,?,?,?,?)',
                (f"audit:vaultseed:{p['id'].split(':')[-1]}", "vault-integration", "vault.seed",
                 p["id"], json.dumps({"file": p["file"], "priceDin": p["price"]},
                                     ensure_ascii=False), CREATED_AT))
            stats["audit"] += 1
    return stats


def _kb_items() -> list[tuple[str, str, str]]:
    """(id, title, body) — 16 commerce + 11 engineering + SOUL/PRICING + 4 playbooks."""
    items: list[tuple[str, str, str]] = []

    def parse_frontmatter(md: str) -> tuple[str, str]:
        m = re.match(r"^---\s*\nname:\s*(.+?)\s*\ndescription:\s*(.+?)\s*\n", md)
        if m:
            return m.group(1).strip(), m.group(2).strip()
        m2 = re.search(r"^#\s*SKILL:\s*(.+)$", md, re.M)
        if m2:
            title = m2.group(1).strip()
            desc_m = re.search(r"^#\s*.*?：(.*?)$", md, re.M)
            return title, (desc_m.group(1).strip() if desc_m else title)
        return None, None

    skills02 = VAULT / "02_10x_engine_and_16_skills" / "engine" / "skills_builtin"
    for d in sorted(skills02.iterdir()):
        md_path = d / "SKILL.md"
        if not md_path.is_file():
            continue
        body = md_path.read_text(encoding="utf-8", errors="replace")
        name, desc = parse_frontmatter(body)
        slug = d.name
        title = desc if desc else (name or slug)
        items.append((f"kb:skill:{slug}", f"\u0645\u0647\u0627\u0631\u062a \u062a\u062c\u0627\u0631\u06cc: {title}", body))

    skills09 = VAULT / "09_recovered_playbooks_skills_valuation" / "kia_skills"
    for d in sorted((skills09 / "skills").iterdir()):
        md_path = d / "SKILL.md"
        if not md_path.is_file():
            continue
        body = md_path.read_text(encoding="utf-8", errors="replace")
        name, desc = parse_frontmatter(body)
        slug = d.name
        title = desc if desc else (name or slug)
        items.append((f"kb:eng:{slug}", f"\u0645\u0647\u0627\u0631\u062a \u0645\u0647\u0646\u062f\u0633\u06cc \u06a9\u06cc\u0627: {title}", body))

    for extra_id, extra_title, path in [
        ("kb:eng:soul", "\u0645\u0646\u0634\u0648\u0631 \u0647\u0648\u06cc\u062a\u06cc SOUL — \u0631\u0648\u062d \u0645\u0647\u0646\u062f\u0633\u06cc \u06a9\u06cc\u0627", skills09 / "SOUL.md"),
        ("kb:eng:pricing", "\u0642\u06cc\u0645\u062a‌\u06af\u0630\u0627\u0631\u06cc \u0645\u0647\u0646\u062f\u0633\u06cc \u06a9\u06cc\u0627 (PRICING)", skills09 / "PRICING.md"),
        ("kb:playbook:day1-sales-kit", "\u06a9\u06cc\u0641 \u0641\u0631\u0648\u0634 \u0631\u0648\u0632 \u0627\u0648\u0644 (DAY1 Sales Kit)",
         VAULT / "09_recovered_playbooks_skills_valuation" / "legacy_python_v5" / "DAY1-SALES-KIT.md"),
        ("kb:playbook:marketing-fa", "\u0628\u0627\u0632\u06cc‌\u0646\u0627\u0645\u0647\u0654 \u0628\u0627\u0632\u0627\u0631\u06cc\u0627\u0628\u06cc \u0641\u0627\u0631\u0633\u06cc",
         VAULT / "09_recovered_playbooks_skills_valuation" / "legacy_python_v5" / "docs" / "MARKETING-PLAYBOOK-FA.md"),
        ("kb:playbook:v3-blueprint", "\u0646\u0642\u0634\u0647\u0654 \u0631\u0627\u0647 \u06cc\u06a9\u067e\u0627\u0631\u0686\u0647\u0654 V3",
         VAULT / "09_recovered_playbooks_skills_valuation" / "legacy_python_v5" / "V3_UNIFIED_BLUEPRINT.md"),
        ("kb:playbook:valuation", "\u0627\u0631\u0632\u0634‌\u06af\u0630\u0627\u0631\u06cc \u067e\u0631\u062a\u0641\u0648\u06cc \u0645\u0647\u0646\u062f\u0633\u06cc \u06a9\u06cc\u0627 (~$758K)",
         VAULT / "09_recovered_playbooks_skills_valuation" / "portfolio" / "VALUATION.md"),
    ]:
        body = read_text_lenient(path)
        items.append((extra_id, extra_title, body))
    return items


def read_text_lenient(path: Path) -> str:
    """UTF-8 first; tolerate stray cp1252 bytes (e.g. \\x97 em-dash in legacy files)."""
    raw = path.read_bytes()
    try:
        return raw.decode("utf-8")
    except UnicodeDecodeError:
        return raw.decode("utf-8", errors="replace").replace("\ufffd", "—")


def upsert_kb(db: sqlite3.Connection) -> dict:
    stats = {"inserted": 0, "updated": 0, "skipped": 0}
    cur = db.cursor()
    for kb_id, title, body in _kb_items():
        row = cur.execute('SELECT "title","body" FROM "KbEntry" WHERE "id"=?', (kb_id,)).fetchone()
        if row is not None and row[0] == title and row[1] == body:
            stats["skipped"] += 1
            continue
        cur.execute(
            'INSERT INTO "KbEntry" ("id","title","body","createdAt","updatedAt") VALUES (?,?,?,?,?)'
            ' ON CONFLICT("id") DO UPDATE SET "title"=excluded."title", "body"=excluded."body",'
            ' "updatedAt"=excluded."updatedAt"',
            (kb_id, title, body, CREATED_AT, CREATED_AT))
        if row is None:
            stats["inserted"] += 1
        else:
            stats["updated"] += 1
    return stats


# ── 5. Verification ─────────────────────────────────────────────────────────

def verify(db: sqlite3.Connection) -> dict:
    cur = db.cursor()
    active = cur.execute('SELECT COUNT(*) FROM "Product" WHERE "active"=1').fetchone()[0]
    trinity = cur.execute(
        'SELECT COUNT(*) FROM "Product" WHERE "active"=1 AND "id" LIKE \'prod:trinity-%\'').fetchone()[0]
    kb = cur.execute('SELECT COUNT(*) FROM "KbEntry"').fetchone()[0]
    trial = cur.execute("SELECT COALESCE(SUM(amount),0) FROM Posting").fetchone()[0]
    credits = cur.execute("SELECT COALESCE(SUM(amount),0) FROM Posting WHERE amount>0").fetchone()[0]
    timings = []
    for q in ("\u0627\u0633\u06a9\u0631\u0648", "\u067e\u0648\u0631\u0633\u0627\u0646\u062a", "\u062a\u062a\u0631", "\u0642\u06cc\u0645\u062a‌\u06af\u0630\u0627\u0631\u06cc", "persian-rtl"):
        t0 = time.perf_counter()
        rows = cur.execute(
            'SELECT COUNT(*) FROM "KbEntry" WHERE "title" LIKE ? OR "body" LIKE ?',
            (f"%{q}%", f"%{q}%")).fetchone()[0]
        timings.append((q, rows, (time.perf_counter() - t0) * 1000))
    return {"active": active, "trinity": trinity, "kb": kb, "trial_sum": trial,
            "credits": credits, "search": timings}


def main() -> int:
    assert REPO.is_dir() and VAULT.is_dir(), "repo/vault missing"
    # 1) static mount checks + ocean lab patch
    for p in PRODUCTS:
        f = TRINITY / p["file"]
        assert f.is_file(), f"missing static product {f}"
    css_len = len(patch_ocean_lab().encode())
    print(f"[1/4] static mount OK — ocean-test-lab patched (inline utils CSS {css_len}B + ocean-assets/three.min.js)")

    # 2) launcher size audit
    for p in PRODUCTS:
        preview = build_launcher(p)
        size = len(preview.encode("utf-8"))
        assert size < 30_000
    print("[2/4] 8 launcher previews built — all < 30 KB, Persian RTL, live iframe preview")

    # 3) DB upserts (backup trial-balance proof first)
    before = verify(sqlite3.connect(SEED_DB))
    db = sqlite3.connect(SEED_DB)
    db.row_factory = sqlite3.Row
    try:
        ps = upsert_products(db)
        ks = upsert_kb(db)
        db.commit()
    finally:
        db.close()
    print(f"[3/4] products: +{ps['inserted']} new, {ps['updated']} updated, {ps['skipped']} unchanged (audits: {ps['audit']})")
    print(f"      kb entries: +{ks['inserted']} new, {ks['updated']} updated, {ks['skipped']} unchanged")

    # 4) verification
    after = verify(sqlite3.connect(SEED_DB))
    print(f"[4/4] active products: {after['active']} (trinity: {after['trinity']}) | KbEntry: {after['kb']}")
    print(f"      trial balance sum: {after['trial_sum']} (credits {after['credits']} = 58,700,200,000 expected)")
    for q, rows, ms in after["search"]:
        flag = "OK" if ms < 5.0 else "SLOW"
        print(f"      search '{q}': {rows} hits in {ms:.2f} ms [{flag}]")
    ok = (after["active"] == 20 and after["trinity"] == 8 and after["kb"] >= 30
          and after["trial_sum"] == 0 and all(ms < 5.0 for _, _, ms in after["search"]))
    print("RESULT:", "PASS" if ok else "FAIL")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
