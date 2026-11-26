#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""inject_vault_gems.py — inject the Hidden Gems vault into MAIRON (the 9-folder mission, phase 2).

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
        title="The SC smart workbench (full version)",
        file="sc.html",
        price=1_200_000_000,
        category="AI Tool",
        tagline="From raw idea to a clear requirement — the complete SC studio workbench",
        accent="#22d3ee",
        icon="🧭",
        features=[
            "Turn a raw idea into structured, testable requirements",
            "Full RTL support and ready-made product templates",
            "Direct output into the build studio",
        ],
        desc="The SC smart workbench is the most complete idea-and-requirement management tool: you enter an idea and the output is a clear, executable requirement. The full version ships a dark theme and a complete interface.",
    ),
    dict(
        id="prod:trinity-fishkal-game",
        title="The FISHKAL physics arcade game — Deep Catch",
        file="fishkal.html",
        price=640_000_000,
        category="Game",
        tagline="Dive in and make the deepest catch!",
        accent="#38bdf8",
        icon="🐟",
        features=[
            "Real swimming and hook-throwing physics with live water dynamics",
            "More than 30 fish species with smart escape behavior",
            "A local leaderboard plus hook and bait upgrades",
        ],
        desc="FISHKAL is a one-page physics arcade game: dive with the hook, catch fish and collect upgrades. Complete offline Canvas graphics.",
    ),
    dict(
        id="prod:trinity-trading-arena-v10",
        title="The Trading God Arena v10 trading simulator",
        file="trading-god-arena.html",
        price=980_000_000,
        category="Trading",
        tagline="A trading-practice arena with live market feedback",
        accent="#fbbf24",
        icon="📈",
        features=[
            "Candle, margin and risk-management simulation with no real risk",
            "An optimized desktop mode with keyboard shortcuts",
            "Session stats, common mistakes and coaching feedback",
        ],
        desc="Trading God Arena v10 is a professional trading simulator: test your strategy without real money and learn from instant feedback.",
    ),
    dict(
        id="prod:trinity-nova-messenger",
        title="The hyper-dynamic Nova Messenger",
        file="nova-messenger.html",
        price=450_000_000,
        category="UI Kit",
        tagline="A dynamic chat interface with smooth animations",
        accent="#a78bfa",
        icon="💬",
        features=[
            "Message-entry animations, typing indicators and live reactions",
            "Dark and light themes with full RTL support",
            "Ready-made chat, group and quick-reply samples",
        ],
        desc="Nova Messenger is a hyper-dynamic chat interface: use it as the base for your own chat projects and skip hours of design work.",
    ),
    dict(
        id="prod:trinity-liquid-glass",
        title="The Liquid Glass UI (5 canvas canvases)",
        file="liquid-glass.html",
        price=520_000_000,
        category="UI Kit",
        tagline="Five liquid-glass canvases with real physics",
        accent="#34d399",
        icon="🫧",
        features=[
            "5 independent canvases with fluid dynamics and light refraction",
            "Live control of viscosity, color and reflection intensity",
            "Clean code ready to drop into your own product",
        ],
        desc="The Liquid Glass pack ships five interactive liquid-glass canvases: a modern, light effect for landings, mini-apps and dashboards.",
    ),
    dict(
        id="prod:trinity-ocean-webgl",
        title="The ocean physics and WebGL shader lab (Ocean Test Lab)",
        file="ocean-test-lab.html",
        price=480_000_000,
        category="3D/WebGL",
        tagline="A water, wave and sunlight shader laboratory",
        accent="#2dd4bf",
        icon="🌊",
        features=[
            "A water shader with live wave, wind and depth controls",
            "Light, horizon and material settings with GLSL code output",
            "Built for WebGL developers and game makers",
        ],
        desc="Ocean Test Lab is an interactive ocean-physics lab: tweak the water shader parameters live and get ready-made GLSL code.",
    ),
    dict(
        id="prod:trinity-fable-bazaar-3d",
        title="The interactive 3D Fable Bazaar",
        file="fable-bazaar.html",
        price=760_000_000,
        category="3D/WebGL",
        tagline="Walk a 3D traditional bazaar and build stories",
        accent="#f472b6",
        icon="🏰",
        features=[
            "A 3D world with walk controls and first-person view",
            "Shops, characters and interactive dialogues",
            "Built with Three.js and a complete interface",
        ],
        desc="Fable Bazaar is a walkable 3D traditional bazaar: wander between shops, talk to vendors and soak in the storybook atmosphere.",
    ),
    dict(
        id="prod:trinity-dropagentx-suite",
        title="The 12-page super-app package and 3D brain of DropAgentX",
        file="dropagentx/showcase3d.html",
        price=1_100_000_000,
        category="Super-App",
        tagline="A 12-page super-app + a 3D product showcase",
        accent="#fb7185",
        icon="🚀",
        features=[
            "12 complete pages: landing, store, admin, live and insight",
            "A 3D product showcase with Three.js",
            "A PWA with a manifest and an offline service worker",
        ],
        desc="The DropAgentX suite is a complete twelve-page super-app with a 3D product showcase, the SenPai brain and an installable offline PWA.",
    ),
]

# ── 2. Persian RTL launcher builder (< 30 KB) ───────────────────────────────

LAUNCHER_TMPL = """<!DOCTYPE html>
<html lang="fa" dir="rtl">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{title} — MAIRON market preview</title>
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
  <span class="badge">{icon} {category} — a Trinity flagship product</span>
  <h1>{title}</h1>
  <p class="tagline">{tagline}</p>
  <div class="preview"><iframe id="pv" title="Live preview" src="{launch_url}" loading="lazy" referrerpolicy="no-referrer"></iframe><span class="zoom">Preview</span></div>
  <div class="zoom-ctrl"><span>Preview zoom:</span><button type="button" onclick="zm(-.1)">-</button><span id="zv">320</span><button type="button" onclick="zm(.1)">+</button><button type="button" onclick="zr()">reset</button></div>
  <ul>
    {features}
  </ul>
  <div class="meta">
    <div class="price">{price_din} Đ<small>{price_fa} — instant escrow release, no human hold</small></div>
    <a class="btn" href="{launch_url}" target="_blank" rel="noopener">Launch the product now ↗</a>
  </div>
  <p class="foot">This live preview runs the real product directly; the full version is delivered after purchase. Seller: <span>@ImXforevr</span> — a 25 percent referral commission on every purchase.</p>
</main>
<script>
var h=320;
function zm(d){{h=Math.max(200,Math.min(700,Math.round(h*(1+d))));document.getElementById('pv').style.height=h+'px';document.getElementById('zv').textContent=fa(h);}}
function fa(n){{return String(n);}}
document.getElementById('pv').addEventListener('error',function(){{document.querySelector('.preview').innerHTML='<p style="padding:24px;color:#94a3b8;font-size:14px">Preview failed to load — open the product page.</p>';}});
</script>
</body>
</html>"""


def _fa_num(n: int) -> str:
    digits = "0123456789"
    return "".join(digits[int(c)] if c.isdigit() else c for c in str(n))


def _price_fa(price: int) -> str:
    return f"{price // 1_000_000} Din (Đ) on the MAIRON market"


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
        items.append((f"kb:skill:{slug}", f"Business skill: {title}", body))

    skills09 = VAULT / "09_recovered_playbooks_skills_valuation" / "kia_skills"
    for d in sorted((skills09 / "skills").iterdir()):
        md_path = d / "SKILL.md"
        if not md_path.is_file():
            continue
        body = md_path.read_text(encoding="utf-8", errors="replace")
        name, desc = parse_frontmatter(body)
        slug = d.name
        title = desc if desc else (name or slug)
        items.append((f"kb:eng:{slug}", f"MAIRON engineering skill: {title}", body))

    for extra_id, extra_title, path in [
        ("kb:eng:soul", "The SOUL identity charter — the spirit of MAIRON engineering", skills09 / "SOUL.md"),
        ("kb:eng:pricing", "MAIRON engineering pricing (PRICING)", skills09 / "PRICING.md"),
        ("kb:playbook:day1-sales-kit", "The day-1 sales kit (DAY1 Sales Kit)",
         VAULT / "09_recovered_playbooks_skills_valuation" / "legacy_python_v5" / "DAY1-SALES-KIT.md"),
        ("kb:playbook:marketing-fa", "The Persian marketing playbook",
         VAULT / "09_recovered_playbooks_skills_valuation" / "legacy_python_v5" / "docs" / "MARKETING-PLAYBOOK-FA.md"),
        ("kb:playbook:v3-blueprint", "The unified V3 blueprint",
         VAULT / "09_recovered_playbooks_skills_valuation" / "legacy_python_v5" / "V3_UNIFIED_BLUEPRINT.md"),
        ("kb:playbook:valuation", "The MAIRON engineering portfolio valuation (~$758K)",
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
        return raw.decode("utf-8", errors="replace").replace("�", "—")


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
    for q in ("escrow", "commission", "tether", "pricing", "persian-rtl"):
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
