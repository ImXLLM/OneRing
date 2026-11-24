#!/usr/bin/env python3
"""
Idempotent KIAN v2 Knowledge Base Seeder
Sources:
  - tashfeenahmed/freellmapi (Free LLM Provider Registry)
  - public-apis/public-apis (Zero-Auth HTTPS+CORS Public APIs for KIAN Studio /builder)

Target: seed/kian-production.db ("KbEntry" table)
Usage:
  python3 inject_freellm_and_public_apis.py --db seed/kian-production.db
"""

import argparse
import json
import sqlite3
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

ZERO_AUTH_STUDIO_SKILL = """# KIAN Studio Skill: Zero-Auth Live Public APIs (`Auth: No` + `HTTPS: Yes` + `CORS: Yes`)

When generating single-file `<!DOCTYPE html>` web applications in **KIAN Studio (`/builder`)**, NEVER hardcode mock numbers when a zero-auth, HTTPS, CORS-enabled public API exists, and NEVER prompt the end-user for an API key unless they explicitly ask for a paid service.

## Golden Rules for Browser-Side `fetch()`
1. **Always use `HTTPS`** and wrap every `fetch()` in `try { ... } catch (err) { ... }` with a clean UI fallback banner + cached `localStorage` fallback so rate limits never break the preview.
2. **Debounce user inputs (`300ms`)** on search/filter boxes (Wikipedia, Dictionary, Iconify, REST Countries) to respect public rate limits.
3. **Prefer `CORS: Yes` endpoints** so direct browser `fetch()` works inside standalone `<!DOCTYPE html>` files or sandboxed iframes.

## Verified Zero-Key Endpoints & Ready Snippets

### 1. Weather & Climate (`Open-Meteo` — CORS: Yes, No Key)
```js
const url = `https://api.open-meteo.com/v1/forecast?latitude=${lat}&longitude=${lon}&current=temperature_2m,relative_humidity_2m,wind_speed_10m,weather_code&daily=temperature_2m_max,temperature_2m_min&timezone=auto`;
const weather = await fetch(url).then(r => r.json());
```

### 2. Currency Exchange Rates (`Frankfurter` & `Currency-api` — CORS: Yes, No Key)
```js
// ECB official rates + historical time series
const fx = await fetch('https://api.frankfurter.app/latest?from=USD&to=EUR,GBP,JPY,TRY').then(r => r.json());
// 150+ currencies including IRR/AED/crypto via jsDelivr CDN (zero rate limit)
const allFx = await fetch('https://cdn.jsdelivr.net/npm/@fawazahmed0/currency-api@latest/v1/currencies/usd.json').then(r => r.json());
```

### 3. Live Cryptocurrency Prices (`CoinGecko` — CORS: Yes, No Key)
```js
const coins = await fetch('https://api.coingecko.com/api/v3/simple/price?ids=bitcoin,ethereum,solana,tether&vs_currencies=usd&include_24hr_change=true').then(r => r.json());
```

### 4. GeoIP & Country Profiles (`Country.is`, `GeoJS`, `REST Countries` — CORS: Yes, No Key)
```js
const geo = await fetch('https://get.geojs.io/v1/ip/geo.json').then(r => r.json());
const countryInfo = await fetch(`https://restcountries.com/v3.1/alpha/${geo.country_code}?fields=name,capital,population,flags,currencies,languages`).then(r => r.json());
```

### 5. Tech News, Spaceflight & Encyclopedia (`HackerNews`, `Spaceflight News`, `Wikipedia`)
```js
const topIds = await fetch('https://hacker-news.firebaseio.com/v0/topstories.json').then(r => r.json());
const stories = await Promise.all(topIds.slice(0, 8).map(id => fetch(`https://hacker-news.firebaseio.com/v0/item/${id}.json`).then(r => r.json())));
const space = await fetch('https://api.spaceflightnewsapi.net/v4/articles/?limit=6').then(r => r.json());
const wiki = await fetch('https://en.wikipedia.org/api/rest_v1/page/summary/Large_language_model').then(r => r.json());
```

### 6. Dictionaries, Poetry, Trivia, Icons & Test Data
- **Dictionary:** `https://api.dictionaryapi.dev/api/v2/entries/en/<word>`
- **Trivia Quiz:** `https://opentdb.com/api.php?amount=10&type=multiple`
- **Programming Jokes:** `https://v2.jokeapi.dev/joke/Programming?safe-mode`
- **Icons Search:** `https://api.iconify.design/search?query=chart&limit=16`
- **Mock Store/Users:** `https://dummyjson.com/products?limit=12`
- **Public Holidays:** `https://date.nager.at/api/v3/PublicHolidays/2026/US`
"""


def build_free_llm_kb_content() -> str:
    reg_path = ROOT / "01_freellmapi_complete_catalog" / "providers_registry.json"
    if reg_path.exists():
        data = json.loads(reg_path.read_text(encoding="utf-8"))
        providers = data.get("providers", [])
    else:
        providers = []

    lines = [
        "# KIAN v2 Free LLM Multi-Provider Registry (No-Credit-Card & Keyless)",
        f"- Catalog source: `tashfeenahmed/freellmapi` (`{data.get('live_catalog_version', '2026.11')}` if loaded)",
        f"- Total cataloged platforms: **{len(providers)}**",
        "",
        "## Priority OpenAI-Compatible Free Endpoints",
        "| Platform | Base URL | Auth / Card | Rate Limits | Sample Free Models |",
        "|---|---|---|---|---|",
    ]
    for p in providers:
        if p.get("category") != "llm_chat" or not p.get("base_url"):
            continue
        models_preview = ", ".join(m["model_id"] for m in p.get("models", [])[:3]) or "catalog-synced"
        lines.append(
            f"| `{p['platform']}` | `{p['base_url']}` | {p['signup_requirement']} | {p['rate_limits']} | `{models_preview}` |"
        )
    return "\n".join(lines)


def ensure_schema(conn: sqlite3.Connection) -> None:
    conn.execute(
        """
        CREATE TABLE IF NOT EXISTS "KbEntry" (
            "id" TEXT PRIMARY KEY,
            "slug" TEXT UNIQUE NOT NULL,
            "title" TEXT NOT NULL,
            "category" TEXT NOT NULL,
            "tags" TEXT NOT NULL,
            "content" TEXT NOT NULL,
            "source" TEXT NOT NULL,
            "updatedAt" TEXT NOT NULL
        )
        """
    )


def upsert_kb_entry(
    conn: sqlite3.Connection,
    entry_id: str,
    slug: str,
    title: str,
    category: str,
    tags: list[str],
    content: str,
    source: str,
) -> str:
    now = datetime.now(timezone.utc).isoformat()
    cur = conn.execute('SELECT id FROM "KbEntry" WHERE id = ? OR slug = ?', (entry_id, slug))
    exists = cur.fetchone() is not None
    conn.execute(
        """
        INSERT INTO "KbEntry" ("id", "slug", "title", "category", "tags", "content", "source", "updatedAt")
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT("id") DO UPDATE SET
            "slug" = excluded."slug",
            "title" = excluded."title",
            "category" = excluded."category",
            "tags" = excluded."tags",
            "content" = excluded."content",
            "source" = excluded."source",
            "updatedAt" = excluded."updatedAt"
        """,
        (entry_id, slug, title, category, json.dumps(tags), content, source, now),
    )
    return "updated" if exists else "inserted"


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--db", default="seed/kian-production.db", help="Path to SQLite DB")
    args = parser.parse_args()

    db_path = Path(args.db)
    db_path.parent.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(str(db_path))
    try:
        ensure_schema(conn)
        studio_top_path = ROOT / "02_zero_auth_public_apis_for_studio" / "top_120_studio_browser_apis.json"
        studio_extra = ""
        if studio_top_path.exists():
            top_data = json.loads(studio_top_path.read_text(encoding="utf-8"))
            studio_extra = "\n\n## Top 120 Curated Zero-Auth APIs Summary\n" + "\n".join(
                f"- **{a['category']} · {a['name']}** (`CORS: {a['cors']}`): {a['url']} — {a['description']}"
                for a in top_data.get("apis", [])
            )

        r1 = upsert_kb_entry(
            conn,
            entry_id="kb:skill-zero-auth-live-apis",
            slug="skill-zero-auth-live-apis",
            title="Zero-Auth Live Public APIs for KIAN Studio (/builder)",
            category="studio_skill",
            tags=["studio", "public-apis", "zero-auth", "cors", "https", "fetch"],
            content=ZERO_AUTH_STUDIO_SKILL + studio_extra,
            source="public-apis/public-apis",
        )
        r2 = upsert_kb_entry(
            conn,
            entry_id="kb:registry-free-llm-providers",
            slug="registry-free-llm-providers",
            title="Free LLM API Provider Registry (No-Credit-Card & Keyless)",
            category="ai_infrastructure",
            tags=["freellmapi", "llm", "free-tier", "openai-compat", "rate-limits"],
            content=build_free_llm_kb_content(),
            source="tashfeenahmed/freellmapi",
        )
        conn.commit()
        print(json.dumps({"db": str(db_path), "results": {"kb:skill-zero-auth-live-apis": r1, "kb:registry-free-llm-providers": r2}}))
    finally:
        conn.close()


if __name__ == "__main__":
    main()
