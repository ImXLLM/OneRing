#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
migrate_dropagent_to_kian.py — مهاجرت قطعی و idempotent جامعهٔ زندهٔ dropagent به KIAN v2
=========================================================================================

ورودی:  پایگاه ۳۰روزهٔ لگاسی (marketplace-20260924-0400.db — فقط‌خواندنی)
خروجی:  kian-production.db با اسکیمای کامل KIAN v2 آمادهٔ استقرار Railway

تضمین‌های کرنل (crates/db/src/store.rs):
  * `"Posting"` تنها منبع حقیقت تراز است (ADR-003) — هر تراز با Tx (MINT) و
    دو پا (`mint:genesis` بدهکار −amount، `citizen:{tg}` بستانکار +amount) پشتیبان می‌شود.
  * `V3Store::load_ledger_state()` جدول‌های Account/Tx/Posting را می‌خواند و
    kinds باید lowercase باشد: user/escrow/treasury/mint/burn.
  * `load_escrow_registry()` همهٔ ردیف‌های Escrow را به وضعیت دامنه می‌خواند —
    status باید از ثابت‌های EscrowStatus باشد (اینجا: RELEASED).
  * Σ همهٔ Postingها باید دقیقاً ۰ باشد تا `trial_balance` و همهٔ `apply()`های بعدی سالم بمانند.

پاک‌سازی مالی (طبق دستور مهاجرت):
  * هر کاربر عادی: دقیقاً تراز مثبت لگاسی × ۱٬۰۰۰٬۰۰۰ micro، سقف ۱٬۰۰۰ KIAN
  * @ImXsi  (6398636666): ۱٬۰۰۰ KIAN — اصلاح واریز دوبار Deposit #3 (tx #806/#807)
  * @ImXforevr (8198598635): ۵٬۰۰۰ KIAN — تراز بنیان‌گذار
  * Σ کل genesis = ۵۸٬۷۰۰٬۲۰۰٬۰۰۰ micro (۵۸٬۷۰۰.۲ KIAN) — زیر سقف جدید منشور
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import sqlite3
import sys
from datetime import datetime, timezone

MICRO = 1_000_000                     # 1 credit = 1 KIAN = 1,000,000 micro
REGULAR_CAP_MICRO = 1_000_000_000     # سقف کاربر عادی: ۱٬۰۰۰ KIAN
PRODUCT_MIN_PRICE_MICRO = 10_000_000  # حداقل قیمت محصول: ۱۰ KIAN
AFFILIATE_BPS = 2_000                 # پورسانت وایرال ۲۰٪
TAX_BPS = 1_000                       # مادهٔ ۲ منشور: ۱۰٪ مالیات خزانه
PURCHASE_TTL_H = 72

INSIDER_FIXED_MICRO = {
    "8198598635": 5_000_000_000,  # @ImXforevr — بنیان‌گذار
    "6398636666": 1_000_000_000,  # @ImXsi — اصلاح دابل‌کردیت
}

SYSTEM_ACCOUNTS = [
    ("mint:genesis", "mint", "منبع عرضه (ضرب)"),
    ("treasury:nation", "treasury", "خزانه ملی"),
    ("burn:void", "burn", "مقصد سوزاندن"),
]

MEMORY_LAYER_MAP = {
    "interest": "semantic",
    "fact": "semantic",
    "goal": "identity",
    "skill": "procedural",
}

REPORT: dict = {}


# ── کمکی‌ها ───────────────────────────────────────────────────────────────────
def iso(ts: float | int | None) -> str | None:
    """Unix epoch (ثانیه، شاید float) → ISO-8601 UTC با millis و پسوند Z (قرارداد EPOCH_ISO کرنل)."""
    if ts is None:
        return None
    dt = datetime.fromtimestamp(float(ts), tz=timezone.utc)
    return dt.strftime("%Y-%m-%dT%H:%M:%S.") + f"{dt.microsecond // 1000:03d}Z"


def s(v) -> str | None:
    """None/رشتهٔ خالی → None؛ وگرنه رشتهٔ پاک‌شده."""
    if v is None:
        return None
    t = str(v).strip()
    return t if t else None


def sanitize_micro(tg_id: str, credits: float) -> int:
    if tg_id in INSIDER_FIXED_MICRO:
        return INSIDER_FIXED_MICRO[tg_id]
    return min(round(max(credits, 0.0) * MICRO), REGULAR_CAP_MICRO)


def split_statements(sql_text: str) -> list[str]:
    """حذف کامنت‌های خط کامل و شکست روی ';' — برای اجرای گام‌به‌گام با گارد ALTER."""
    lines = [ln for ln in sql_text.splitlines() if not ln.lstrip().startswith("--")]
    out = []
    for chunk in "\n".join(lines).split(";"):
        c = chunk.strip()
        if c:
            out.append(c)
    return out


def column_exists(con: sqlite3.Connection, table: str, column: str) -> bool:
    cur = con.execute(f'PRAGMA table_info("{table}")')
    return any(r[1] == column for r in cur.fetchall())


def apply_migration_file(con: sqlite3.Connection, version: int, path: str) -> None:
    """اجرای یک فایل مهاجرت با گارد ADD COLUMN (idempotent) + ثبت ردیف _sqlx_migrations."""
    raw = open(path, "r", encoding="utf-8").read()
    checksum = hashlib.sha384(raw.encode("utf-8")).digest()
    description = re.sub(r"^\d+_", "", os.path.splitext(os.path.basename(path))[0])
    for stmt in split_statements(raw):
        m = re.search(r'ALTER\s+TABLE\s+"([^"]+)"\s+ADD\s+COLUMN\s+"([^"]+)"', stmt, re.I)
        if m and column_exists(con, m.group(1), m.group(2)):
            continue  # ستون از قبل هست — رد شو
        con.execute(stmt)
    con.execute(
        'INSERT INTO _sqlx_migrations (version, description, installed_on, success, checksum, execution_time) '
        'VALUES (?, ?, CURRENT_TIMESTAMP, 1, ?, 0) '
        'ON CONFLICT(version) DO UPDATE SET checksum=excluded.checksum, success=1',
        (version, description, checksum),
    )


def wipe_migration_rows(con: sqlite3.Connection) -> dict:
    """حذف ردیف‌های مهاجرت قبلی با پیشوندهای قطعی — اجرای دوم = همان وضعیت (idempotent)."""
    wiped = {}
    def d(sql: str, args: tuple = ()) -> None:
        wiped[sql.split("FROM")[1].split("WHERE")[0].strip('" ')] = con.execute(sql, args).rowcount
    d('DELETE FROM "Posting" WHERE "txId" LIKE ?', ("tx-mig-%",))
    d('DELETE FROM "Tx" WHERE "id" LIKE ?', ("tx-mig-%",))
    d('DELETE FROM "AuditLog" WHERE "id" LIKE ?', ("audit-mig-%",))
    d('DELETE FROM "Escrow" WHERE "id" LIKE ?', ("escrow-mig-%",))
    d('DELETE FROM "BuilderMessage" WHERE "sessionId" LIKE ?', ("bldr-mig-%",))
    d('DELETE FROM "BuilderSession" WHERE "id" LIKE ?', ("bldr-mig-%",))
    d('DELETE FROM "MemoryNote" WHERE "id" LIKE ?', ("mem-mig-%",))
    d('DELETE FROM "Product" WHERE "id" LIKE ?', ("prod-mig-%",))
    d('DELETE FROM "ReferralLineage"', ())
    d('DELETE FROM "User" WHERE "id" LIKE ?', ("citizen:%",))
    d('DELETE FROM "Account" WHERE "id" LIKE ?', ("citizen:%",))
    return wiped


# ── آرتیفکت HTML محصولات ─────────────────────────────────────────────────────
def artifact_html(title: str, seller: str, category: str | None, description: str,
                  price_kian: str, full: bool) -> str:
    """آرتیفکت HTML تک‌صفحه‌ای، فارسی RTL، خودکفا و تعاملی — بدون وابستگی خارجی."""
    cat = category or "tool"
    paras = "".join(
        f"<p>{ln}</p>" for ln in description.splitlines() if ln.strip()
    ) or "<p>این محصول دیجیتال به‌همراه مهاجرت جامعهٔ dropagent به KIAN v2 منتشر شد.</p>"
    esc_title = (title or "محصول KIAN").replace("<", "&lt;")
    esc_seller = (seller or "unknown").replace("<", "&lt;")
    extra = (
        '<button onclick="dl()">⬇️ دانلود نسخهٔ کامل</button>'
        if full
        else '<button onclick="alert(\'برای دریافت نسخهٔ کامل، از طریق بازار KIAN خرید کنید.\')">🔒 دریافت نسخهٔ کامل</button>'
    )
    return f"""<!DOCTYPE html>
<html lang="fa" dir="rtl">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{esc_title} — KIAN v2</title>
<style>
  :root {{ --bg:#0b0f1a; --card:#121829; --ink:#e8ecf8; --sub:#93a0bd; --gold:#f5b942; --line:#232c45; }}
  * {{ box-sizing:border-box; margin:0; padding:0; }}
  body {{ background:var(--bg); color:var(--ink); font-family:Vazirmatn,Tahoma,system-ui,sans-serif; line-height:1.9; }}
  .wrap {{ max-width:760px; margin:0 auto; padding:32px 20px 64px; }}
  .badge {{ display:inline-block; background:var(--line); color:var(--gold); border:1px solid var(--gold);
           border-radius:999px; padding:4px 14px; font-size:13px; margin-bottom:14px; }}
  h1 {{ font-size:26px; line-height:1.5; margin-bottom:6px; }}
  .seller {{ color:var(--sub); font-size:14px; margin-bottom:22px; }}
  .price {{ background:linear-gradient(135deg,#1a2238,#121829); border:1px solid var(--line); border-radius:14px;
            padding:16px 20px; display:flex; align-items:center; justify-content:space-between; margin-bottom:22px; }}
  .price b {{ color:var(--gold); font-size:20px; }}
  .card {{ background:var(--card); border:1px solid var(--line); border-radius:14px; padding:20px 22px; margin-bottom:16px; }}
  .card h2 {{ font-size:17px; margin-bottom:10px; color:var(--gold); }}
  .card p {{ margin-bottom:8px; white-space:pre-wrap; }}
  button {{ background:var(--gold); color:#1a1305; border:0; border-radius:10px; padding:12px 26px;
            font-size:15px; font-weight:700; cursor:pointer; font-family:inherit; }}
  button:hover {{ filter:brightness(1.1); }}
  .foot {{ color:var(--sub); font-size:12px; margin-top:26px; text-align:center; }}
</style>
</head>
<body>
<div class="wrap">
  <span class="badge">📦 {cat} · KIAN v2 Marketplace</span>
  <h1>{esc_title}</h1>
  <div class="seller">فروشنده: @{esc_seller}</div>
  <div class="price"><span>قیمت</span><b>{price_kian} KIAN</b></div>
  <div class="card"><h2>معرفی محصول</h2>{paras}</div>
  <div class="card"><h2>دربارهٔ تحویل</h2>
    <p>این محصول به‌صورت آرتیفکت HTML تعاملی تحویل داده می‌شود — بلافاصله پس از آزادسازی اسکرو،
       نسخهٔ کامل از مسیر <code>GET /api/v1/market/escrows/&#123;id&#125;/delivery</code> در دسترس خریدار است.</p>
    <p>پورسانت معرف: <b>{AFFILIATE_BPS // 100}٪</b> از هر خرید از طریق لینک دعوت شما.</p>
  </div>
  {extra}
  <div class="foot">KIAN v2 · مهاجرت‌یافته از جامعهٔ ۳۰روزهٔ dropagent · دفتر دوطرفهٔ ممیزی‌پذیر</div>
</div>
<script>
function dl() {{
  const blob = new Blob([document.documentElement.outerHTML], {{type: 'text/html'}});
  const a = document.createElement('a');
  a.href = URL.createObjectURL(blob);
  a.download = '{esc_seller}-kian-artifact.html';
  a.click();
}}
</script>
</body>
</html>"""


# ── هستهٔ مهاجرت ──────────────────────────────────────────────────────────────
def migrate(legacy_path: str, target_path: str, repo_root: str) -> dict:
    mig_dir = os.path.join(repo_root, "crates", "db", "migrations")
    os.makedirs(os.path.dirname(os.path.abspath(target_path)), exist_ok=True)
    if os.path.exists(target_path):
        os.remove(target_path)  # خروجی قطعی — بازسازی کامل
    con = sqlite3.connect(target_path)
    con.execute("PRAGMA journal_mode=DELETE")
    con.execute("PRAGMA foreign_keys=OFF")

    # ۱) اسکیمای KIAN v2 — همهٔ مهاجرت‌ها + ثبت checksum برای sqlx::migrate!()
    con.execute(
        'CREATE TABLE IF NOT EXISTS _sqlx_migrations ('
        'version INTEGER PRIMARY KEY, description TEXT NOT NULL, '
        'installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP, '
        'success BOOLEAN NOT NULL DEFAULT 1, checksum BLOB NOT NULL, '
        'execution_time INTEGER NOT NULL DEFAULT 0)'
    )
    mig_files = sorted(
        f for f in os.listdir(mig_dir) if re.match(r"^\d{4}_.*\.sql$", f)
    )
    for f in mig_files:
        apply_migration_file(con, int(f[:4]), os.path.join(mig_dir, f))
    REPORT["migrations_applied"] = mig_files

    # ۲) حساب‌های سیستمی — همان ensure_system_accounts کرنل (مادهٔ ۴ منشور)
    for acc_id, kind, label in SYSTEM_ACCOUNTS:
        con.execute(
            'INSERT OR IGNORE INTO "Account" ("id","kind","label","createdAt") VALUES (?,?,?,?)',
            (acc_id, kind, label, iso(datetime.now(timezone.utc).timestamp())),
        )

    # ۳) خواندن لگاسی — فقط‌خواندنی
    src = sqlite3.connect(f"file:{legacy_path}?mode=ro", uri=True)
    src.row_factory = sqlite3.Row
    users = [dict(r) for r in src.execute(
        "SELECT user_id, username, first_name, credits, created_at, referred_by FROM users ORDER BY created_at ASC, user_id ASC")]
    products = [dict(r) for r in src.execute("SELECT * FROM products ORDER BY id ASC")]
    purchases = [dict(r) for r in src.execute("SELECT * FROM purchases ORDER BY id ASC")]
    chats = [dict(r) for r in src.execute("SELECT * FROM chat_messages ORDER BY created_at ASC, id ASC")]
    memories = [dict(r) for r in src.execute("SELECT * FROM user_memories ORDER BY id ASC")]

    user_ids = {str(u["user_id"]) for u in users}
    username_of = {str(u["user_id"]): s(u["username"]) for u in users}

    # ۴) کاربران + حساب‌های شهروندی + تراز پاک‌سازی‌شده
    total_minted = 0
    minted_accounts = 0
    for u in users:
        tg = str(u["user_id"])
        acc = f"citizen:{tg}"
        created = iso(u["created_at"])
        uname = s(u["username"])
        con.execute(
            'INSERT INTO "Account" ("id","kind","label","createdAt") VALUES (?,?,?,?)',
            (acc, "user", uname or s(u["first_name"]) or f"citizen {tg}", created),
        )
        referrer_tg = s(u["referred_by"])
        con.execute(
            'INSERT INTO "User" ("id","telegramId","username","tonAddress","accountId","createdAt","firstName","referredBy") '
            'VALUES (?,?,?,?,?,?,?,?)',
            (acc, tg, uname, None, acc, created, s(u["first_name"]),
             f"citizen:{referrer_tg}" if referrer_tg in user_ids else None),
        )
        micro = sanitize_micro(tg, u["credits"] or 0.0)
        if micro > 0:
            minted_accounts += 1
            total_minted += micro
            tx_id = f"tx-mig-mint-{tg}"
            memo = (f"migration genesis — legacy user {tg} "
                    f"(@{uname}) — {u['credits']} credits × {MICRO:,} micro — sanitized "
                    f"(cap {REGULAR_CAP_MICRO // MICRO:,} KIAN)"
                    if tg not in INSIDER_FIXED_MICRO else
                    f"migration genesis — legacy user {tg} (@{uname}) — sanitized fixed balance "
                    f"({micro // MICRO:,} KIAN); ImXsi deposit#3 double-credit fixed, founder grant cleaned")
            con.execute(
                'INSERT INTO "Tx" ("id","idempotencyKey","memo","at") VALUES (?,?,?,?)',
                (tx_id, f"mig:genesis:{tg}:v1", memo, created),
            )
            con.execute(
                'INSERT INTO "Posting" ("txId","accountId","amount") VALUES (?,?,?)',
                (tx_id, "mint:genesis", -micro),
            )
            con.execute(
                'INSERT INTO "Posting" ("txId","accountId","amount") VALUES (?,?,?)',
                (tx_id, acc, +micro),
            )
    REPORT["users_total"] = len(users)
    REPORT["minted_accounts"] = minted_accounts
    REPORT["genesis_micro"] = total_minted

    # ۵) نسب رفرال — آرشیو کامل ۲۴۷ لینک کمپین
    ref_rows = 0
    for u in users:
        rtg = s(u["referred_by"])
        if rtg:
            con.execute(
                'INSERT OR REPLACE INTO "ReferralLineage" ("tgUserId","referredByTgId","legacyCreatedAt") VALUES (?,?,?)',
                (str(u["user_id"]), rtg, iso(u["created_at"])),
            )
            ref_rows += 1
    REPORT["referral_links"] = ref_rows

    # ۶) محصولات — ۷ تأییدشده + pending یگانه + بهترین Amingh30
    def pick(cands: list[dict]) -> dict:
        return sorted(cands, key=lambda p: (-len(p["description"] or ""), p["id"]))[0]

    approved = [p for p in products if p["status"] == "approved"]
    pend_groups: dict[tuple, list[dict]] = {}
    for p in products:
        if p["status"] == "pending":
            pend_groups.setdefault((str(p["creator_id"]), (p["title"] or "").strip()), []).append(p)
    pending = [pick(g) for g in pend_groups.values()]
    rejected_by_creator: dict[str, list[dict]] = {}
    for p in products:
        if p["status"] == "rejected":
            rejected_by_creator.setdefault(str(p["creator_id"]), []).append(p)
    revived = [pick(g) for g in rejected_by_creator.values()]
    migrated_products = approved + pending + revived
    REPORT["products"] = {
        "approved": len(approved),
        "pending_unique": len(pending),
        "rejected_revived": len(revived),
        "total_active": len(migrated_products),
        "dropped_duplicates": len(products) - len(migrated_products),
    }
    legacy_product_map = {}
    for p in migrated_products:
        pid = f"prod-mig-{p['id']}"
        legacy_product_map[str(p["id"])] = pid
        seller_tg = str(p["creator_id"])
        seller_uname = username_of.get(seller_tg) or f"user{seller_tg}"
        price_micro = max(round((p["price_credits"] or 0) * MICRO), PRODUCT_MIN_PRICE_MICRO)
        html = artifact_html(p["title"], seller_uname, s(p["category"]), p["description"] or "",
                             f"{price_micro // MICRO:,}", full=True)
        con.execute(
            'INSERT INTO "Product" ("id","sellerId","title","description","category","priceDin",'
            '"previewHtml","affiliateBps","artifactKind","deliveryPayload","active","createdAt") '
            'VALUES (?,?,?,?,?,?,?,?,?,?,?,?)',
            (pid, f"citizen:{seller_tg}", p["title"], (p["description"] or "").strip(),
             s(p["category"]), price_micro, html, AFFILIATE_BPS, "HTML_APP", html, 1, iso(p["created_at"])),
        )

    # ۷) خریدهای تاریخی → اسکرو RELEASED (مدل خواندنی — بدون دست‌زدن به ترازها)
    esc_count = 0
    esc_volume = 0
    for pu in purchases:
        prod = next((x for x in products if x["id"] == pu["product_id"]), None)
        assert prod is not None, f"purchase {pu['id']} references unknown product {pu['product_id']}"
        pid = legacy_product_map[str(pu["product_id"])]
        buyer = f"citizen:{pu['buyer_id']}"
        seller = f"citizen:{prod['creator_id']}"
        amount = round((pu["price_credits"] or 0) * MICRO)
        tax = amount * TAX_BPS // 10_000
        esc_id = f"escrow-mig-{pu['id']}"
        at = iso(pu["purchased_at"])
        con.execute(
            'INSERT INTO "Escrow" ("id","buyerId","sellerId","amount","productRef","status","createdAt",'
            '"fundedTxId","settleTxId","disputeReason","deliveryRef","expiresAt","referrer","affiliateBps") '
            'VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?)',
            (esc_id, buyer, seller, amount, pid, "RELEASED", at, None, None, None,
             "legacy:migration", iso((float(pu["purchased_at"]) + PURCHASE_TTL_H * 3600)), None, 0),
        )
        con.execute(
            'INSERT INTO "AuditLog" ("id","actor","action","target","meta","at") VALUES (?,?,?,?,?,?)',
            (f"audit-mig-escrow-{pu['id']}", "migration", "legacy.purchase_migration", esc_id,
             json.dumps({"legacyPurchaseId": pu["id"], "legacyProductId": pu["product_id"],
                         "priceCredits": pu["price_credits"], "amountMicro": amount,
                         "taxMicro": tax, "sellerMicro": amount - tax,
                         "paymentMethod": pu["payment_method"],
                         "purchasedAt": iso(pu["purchased_at"]),
                         "legacyUpdatedAt": iso(pu["created_at"])}, ensure_ascii=False), at),
        )
        esc_count += 1
        esc_volume += amount
    REPORT["escrows"] = {"count": esc_count, "volume_micro": esc_volume}

    # ۸) حافظهٔ AI → MemoryNote + گفتگوهای هرمس → BuilderSession/BuilderMessage
    mem_count = 0
    for mrow in memories:
        con.execute(
            'INSERT INTO "MemoryNote" ("id","userId","layer","content","createdAt") VALUES (?,?,?,?,?)',
            (f"mem-mig-{mrow['id']}", f"citizen:{mrow['user_id']}",
             MEMORY_LAYER_MAP.get(str(mrow["kind"]).strip(), "semantic"),
             (mrow["content"] or "").strip(), iso(mrow["created_at"])),
        )
        mem_count += 1
    REPORT["memory_notes"] = mem_count

    by_user: dict[str, list[dict]] = {}
    for c in chats:
        by_user.setdefault(str(c["user_id"]), []).append(c)
    sess_count = msg_count = 0
    for tg, msgs in by_user.items():
        sid = f"bldr-mig-{tg}"
        first_user = next((m for m in msgs if m["role"] == "user"), msgs[0])
        con.execute(
            'INSERT INTO "BuilderSession" ("id","accountId","stage","title","description","suggestedPrice",'
            '"affiliateBps","generatedHtml","publishedProductId","createdAt","updatedAt") '
            'VALUES (?,?,?,?,?,?,?,?,?,?,?)',
            (sid, f"citizen:{tg}", "discovery",
             "گفتگوی مهاجرت‌یافته از هرمس (dropagent)",
             ((first_user["content"] or "").strip()[:200] or "گفتگوی تاریخی کاربر با هرمس"),
             50 * MICRO, AFFILIATE_BPS, "", None, int(float(msgs[0]["created_at"])),
             int(float(msgs[-1]["created_at"]))),
        )
        for mrow in msgs:
            con.execute(
                'INSERT INTO "BuilderMessage" ("sessionId","role","content","createdAt") VALUES (?,?,?,?)',
                (sid, str(mrow["role"]), mrow["content"] or "", int(float(mrow["created_at"]))),
            )
            msg_count += 1
        sess_count += 1
    REPORT["builder_sessions"] = sess_count
    REPORT["builder_messages"] = msg_count
    src.close()

    # ۹) راستی‌آزمایی دفتر — همان قواعد load_ledger_state/trial_balance کرنل
    trial = con.execute('SELECT COALESCE(SUM("amount"),0) FROM "Posting"').fetchone()[0]
    assert trial == 0, f"trial balance broken: {trial}"
    mint_bal = con.execute(
        'SELECT COALESCE(SUM(p."amount"),0) FROM "Posting" p JOIN "Account" a ON a."id"=p."accountId" '
        'WHERE a."kind"=\'mint\'').fetchone()[0]
    assert mint_bal == -total_minted, f"mint balance {mint_bal} != -{total_minted}"
    kinds = {r[0] for r in con.execute('SELECT DISTINCT "kind" FROM "Account"')}
    assert kinds <= {"user", "mint", "treasury", "burn", "escrow"}, f"bad account kinds: {kinds}"
    for row in con.execute('SELECT DISTINCT "status" FROM "Escrow"'):
        assert row[0] in {"CREATED", "FUNDED", "DELIVERED", "RELEASED", "DISPUTED", "REFUNDED", "CANCELLED", "EXPIRED"}
    for row in con.execute('SELECT "id","buyerId","sellerId","productRef" FROM "Escrow"'):
        assert con.execute('SELECT 1 FROM "Account" WHERE "id"=?', (row[1],)).fetchone(), row
        assert con.execute('SELECT 1 FROM "Account" WHERE "id"=?', (row[2],)).fetchone(), row
        assert con.execute('SELECT 1 FROM "Product" WHERE "id"=?', (row[3],)).fetchone(), row
    # هر تراز شهروند باید با sanitize لگاسی برابر باشد
    users_check = con.execute(
        'SELECT u."telegramId", COALESCE(SUM(p."amount"),0) FROM "User" u '
        'LEFT JOIN "Posting" p ON p."accountId"=u."accountId" GROUP BY u."telegramId"').fetchall()
    assert len(users_check) == len(users)
    for tg, bal in users_check:
        src_credits = next(u["credits"] or 0.0 for u in users if str(u["user_id"]) == tg)
        assert bal == sanitize_micro(tg, src_credits), f"balance mismatch {tg}: {bal}"
    counts = {
        "accounts_total": con.execute('SELECT COUNT(*) FROM "Account"').fetchone()[0],
        "accounts_citizen": con.execute('SELECT COUNT(*) FROM "Account" WHERE "kind"=\'user\'').fetchone()[0],
        "users": con.execute('SELECT COUNT(*) FROM "User"').fetchone()[0],
        "products_active": con.execute('SELECT COUNT(*) FROM "Product" WHERE "active"=1').fetchone()[0],
        "escrows": con.execute('SELECT COUNT(*) FROM "Escrow"').fetchone()[0],
        "escrows_released": con.execute('SELECT COUNT(*) FROM "Escrow" WHERE "status"=\'RELEASED\'').fetchone()[0],
        "referral_lineage": con.execute('SELECT COUNT(*) FROM "ReferralLineage"').fetchone()[0],
        "tx": con.execute('SELECT COUNT(*) FROM "Tx"').fetchone()[0],
        "postings": con.execute('SELECT COUNT(*) FROM "Posting"').fetchone()[0],
        "trial_sum": trial,
    }
    assert counts["accounts_total"] == len(users) + len(SYSTEM_ACCOUNTS)
    assert counts["users"] == len(users) == 466
    assert counts["escrows"] == 108 and counts["escrows_released"] == 108
    assert counts["referral_lineage"] == 247
    REPORT["verify"] = counts

    con.commit()
    con.execute("PRAGMA wal_checkpoint(TRUNCATE)")
    con.execute("VACUUM")
    con.commit()
    con.close()
    return REPORT


def main() -> int:
    ap = argparse.ArgumentParser(description="Deterministic dropagent → KIAN v2 migration")
    repo = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    ap.add_argument("--legacy", default="/home/z/my-project/upload/marketplace-20260924-0400.db")
    ap.add_argument("--target", default=os.path.join(repo, "seed", "kian-production.db"))
    ap.add_argument("--report", default=os.path.join(repo, "scripts", "migration_reconciliation.json"))
    args = ap.parse_args()
    assert os.path.exists(args.legacy), f"legacy db not found: {args.legacy}"
    assert os.path.isdir(os.path.join(repo, "crates", "db", "migrations")), "run inside kian repo"
    report = migrate(args.legacy, args.target, repo)
    os.makedirs(os.path.dirname(args.report), exist_ok=True)
    with open(args.report, "w", encoding="utf-8") as fh:
        json.dump(report, fh, ensure_ascii=False, indent=2)
    size = os.path.getsize(args.target)
    print("=== MIGRATION OK ===")
    print(json.dumps(report, ensure_ascii=False, indent=2))
    print(f"target: {args.target} ({size:,} bytes)")
    print(f"genesis supply: {report['genesis_micro']:,} micro = {report['genesis_micro'] / MICRO:,.1f} KIAN "
          f"(cap 500,000 KIAN — headroom {(500_000_000_000 - report['genesis_micro']) / MICRO:,.1f} KIAN)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
