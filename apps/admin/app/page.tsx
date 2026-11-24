"use client";

import { useCallback, useEffect, useState } from "react";
import Image from "next/image";
import {
  Alert, Badge, Button, Input, Label, toast,
} from "@kian/ui";
import {
  adminDelete, adminGet, adminPost, AdminApiError, getKey, KERNEL_URL, publicGet, setKey,
} from "@/lib/admin-api";

/* ── انواع وایر — قرینهٔ قراردادهای کرنل (camelCase) ───────────────────── */

type Health = {
  status: string; version: string; llmConfigured: boolean; llmModel: string | null;
  ai: string | null; bot: string | null; uptime: number | null;
};
type Overview = {
  users: number; accounts: number; escrows: number; hitlPending: number;
  mintedAllTime: number; burnedAllTime: number; aiCalls: number;
  revenueSeries: { day: string; din: number }[];
  salesBySeller: { sellerId: string; din: number }[];
  salesByProduct: { productId: string; title: string | null; din: number }[];
};
type Hitl = {
  id: string; kind: string; escrowId: string; amount: number; requesterId: string;
  note?: string; status: string; createdAt: string; expiresAt: string;
  decidedBy?: string; decidedAt?: string; decisionNote?: string;
};
type Deposit = {
  id: string; userId: string; txHash: string; walletAddress: string; amountTon: string;
  status: string; createdAt: string; decidedAt?: string; decidedBy?: string;
};
type Product = {
  id: string; sellerId: string; title: string; description: string; priceDin: number;
  category: string | null; active: boolean; createdAt: string;
};
type AiSlot = { slot: number; name: string; model: string; enabled: boolean };
type AiStatus = { enabled: boolean; dailyCap: number; slots: AiSlot[] };
type Skill = { id: string; title: string; category: string; description: string; kind: string; estimateTokens: number };
type Agent = { id: string; name: string; role: string; status: string; description: string };
type McpTool = { name: string; description: string; args: string[]; adminOnly: boolean };
type Fleet = { agents: Agent[]; a2aEnabled: boolean; mcpTools: McpTool[] };
type A2aMsg = { id: string; fromAgent: string; toAgent: string; kind: string; body: string; at: string };
type AuditEntry = { id: string; actor: string; action: string; target?: string; meta?: string; at: string };
/** درخواست برداشت کریپتو — قرینهٔ WithdrawalRequestView در crates/contracts. */
type Wdr = {
  id: string; accountId: string; chain: string; asset: string; destinationAddress: string;
  amountDin: number; txHash: string | null; status: string;
  createdAt: number; resolvedAt: number | null;
};
/** ردیف امانت برای داوری ادمین — قرینهٔ EscrowView. */
type EscrowRow = {
  id: string; buyerId: string; sellerId: string; amount: number; productRef: string | null;
  status: string; createdAt: string; disputeReason: string | null;
};
/** خلاصهٔ اقتصاد ملی — GET /economy (عمومی، مادهٔ ۶). */
type Economy = {
  currency: string; symbol: string;
  circulatingSupply: number; treasuryBalance: number; escrowLocked: number;
  mintedAllTime: number; burnedAllTime: number; mintedThisYear: number;
  yearlyMintCap: number; genesisSupply: number;
  trialBalanceSum: number; balanced: boolean;
  accountCount: number; openEscrows: number;
};
type Bill = {
  id: string; kind: "escrowRelease" | "treasuryMove"; subject: string; amountDin: number;
  resolution?: "release" | "refund"; title: string; status: string;
  decisionReason?: string; openedAt: string; closedAt?: string | null; executedAt?: string | null;
  votesYes: number; votesNo: number; humanVotesYes: number; hitlRequired: boolean;
};

/* ── پوشش صادقانهٔ ۷۱ اندپوینت legacy (web_admin.py) ───────────────────── */
const LEGACY_COVERAGE: { group: string; count: number; kian: string; status: "done" | "partial" | "next" }[] = [
  { group: "احراز و نشست عملیاتی", count: 6, kian: "X-KIAN-Key fail-closed + مقایسهٔ زمان‌ثابت", status: "done" },
  { group: "داشبورد اقتصاد / overview", count: 8, kian: "GET /admin/overview از دفتر واقعی", status: "done" },
  { group: "صف HITL پارلمان", count: 5, kian: "GET/POST /admin/hitl — تأیید = تسویهٔ اتمی", status: "done" },
  { group: "امانت‌ها (بازرس ادمین)", count: 9, kian: "مسیرهای public /escrows + MCP escrow.get", status: "done" },
  { group: "خزانه و واریز TON", count: 7, kian: "GET /admin/deposits + confirm/reject مادهٔ ۳", status: "done" },
  { group: "مدیریت بازار/محصولات", count: 8, kian: "GET /admin/products + DELETE + مسیرهای M3", status: "done" },
  { group: "مغز — گیت‌وی AI و مهارت‌ها", count: 11, kian: "/ai/status + /hermes/* + MCP", status: "done" },
  { group: "ناوگان ایجنت‌ها (A2A/MCP)", count: 9, kian: "/agents/fleet + /a2a/* + /mcp/invoke", status: "done" },
  { group: "ممیزی و بکاپ", count: 5, kian: "GET /admin/audit + POST /system/backup", status: "done" },
  { group: "کاربران و پشتیبانی", count: 8, kian: "فهرست شهروندان — سیم M5 (پارلمان کامل)", status: "next" },
  { group: "عملیات خزانهٔ USDT چهارزنجیره", count: 5, kian: "crates/wallet — M5", status: "next" },
];

const din = (n: number) => new Intl.NumberFormat("fa-IR").format(n);
const TABS = [
  "overview", "withdrawals", "arbitration", "hitl", "deposits",
  "parliament", "market", "mint", "brain", "fleet", "audit", "legacy",
] as const;
type Tab = (typeof TABS)[number];
const TAB_LABELS: Record<Tab, string> = {
  overview: "آمار اقتصاد",
  withdrawals: "برداشت‌های کریپتو",
  arbitration: "داوری اسکرو",
  hitl: "صف HITL",
  deposits: "واریزهای TON",
  parliament: "پارلمان",
  market: "بازار",
  mint: "شارژ حساب",
  brain: "مغز",
  fleet: "ناوگان",
  audit: "ممیزی",
  legacy: "پوشش legacy",
};

export default function AdminPage() {
  // پیش‌فرضِ دمو — کلید رسمی از env کرنل (KIAN_ADMIN_KEY) می‌آید؛ در تستِ امشب
  // «dev-admin-key» از پیش پر می‌شود تا بنیان‌گذار بدون اصطکاک وارد شود.
  const [keyInput, setKeyInput] = useState("dev-admin-key");
  const [unlocked, setUnlocked] = useState(false);
  const [tab, setTab] = useState<Tab>("overview");
  const [health, setHealth] = useState<Health | null>(null);
  const [loading, setLoading] = useState(false);
  // تیکِ بروزرسانی — با هر کلیک، تبِ فعال remount می‌شود و همهٔ داده‌ها تازه می‌شوند
  const [refreshTick, setRefreshTick] = useState(0);

  const refreshHealth = useCallback(() => {
    publicGet<Health>("/health").then(setHealth).catch(() => setHealth(null));
  }, []);

  useEffect(() => {
    refreshHealth();
    if (getKey()) setUnlocked(true);
  }, [refreshHealth]);

  const login = () => {
    setKey(keyInput.trim());
    setKeyInput("");
    setUnlocked(true);
    toast({ title: "کلید نشست ثبت شد — با بستن تب می‌میرد" });
  };
  const logout = () => {
    setKey(null);
    setUnlocked(false);
    toast({ title: "کلید نشست پاک شد" });
  };

  // پایدار — وابسته به هیچ‌چیز؛ اگر هر رندر نو شود، useCallback فرزندها
  // را تغییر می‌دهد و حلقهٔ بی‌نهایت fetch/rerender می‌سازد (OOM رندرر).
  const safe = useCallback(async <T,>(fn: () => Promise<T>): Promise<T | null> => {
    setLoading(true);
    try {
      return await fn();
    } catch (e) {
      const msg = e instanceof AdminApiError ? `${e.code}: ${e.message}` : "خطای شبکه — کرنل در دسترس نیست";
      toast({ title: msg, variant: "destructive" });
      return null;
    } finally {
      setLoading(false);
    }
  }, []);

  return (
    <main className="mx-auto max-w-6xl px-4 py-8 flex flex-col gap-6">
      {/* سربرگ سلطنتی */}
      <header className="flex flex-wrap items-center justify-between gap-4">
        <div>
          <h1 className="text-2xl font-extrabold tracking-tight">
            اتاق فرمان بنیان‌گذار <span className="text-gold-400">KIAN</span>
          </h1>
          <p className="text-sm text-[var(--color-muted-foreground)] mt-1">
            اقتصاد، برداشت‌های کریپتو، داوری اسکرو و شارژ حساب — همه در یک پنل
          </p>
        </div>
        <div className="flex items-center gap-3">
          <Image
            src="/logo.png"
            alt=""
            width={40}
            height={40}
            priority
            className="size-10 rounded-xl ring-1 ring-gold-400/40 shadow-lg shadow-royal-600/30"
          />
          {health && (
            <div className="flex items-center gap-2 text-xs">
              <Badge variant={health.status === "ok" ? "success" : "destructive"}>
                کرنل {health.status === "ok" ? "سالم" : "ناسالم"}
              </Badge>
              <Badge variant="secondary">مغز: {health.ai === "enabled" ? "فعال" : "خاموش"}</Badge>
              <Badge variant="secondary">بات: {health.bot === "enabled" ? "فعال" : "خاموش"}</Badge>
            </div>
          )}
          {unlocked && (
            <Button variant="outline" size="sm" onClick={logout}>
              خروج
            </Button>
          )}
        </div>
      </header>

      {!unlocked ? (
        /* ── دروازهٔ ورود ── */
        <section className="rounded-xl border border-[var(--color-border)] bg-[var(--color-card)] p-6 max-w-lg mx-auto w-full flex flex-col gap-4">
          <Alert variant="gold">
            کلید ادمین فقط در sessionStorage همین تب زندگی می‌کند — با بستن تب پاک می‌شود.
            کلید از env کرنل (`KIAN_ADMIN_KEY`) می‌آید و هرگز در این پنل ذخیرهٔ پایدار ندارد.
          </Alert>
          <div className="flex flex-col gap-2">
            <Label htmlFor="k">X-KIAN-Key</Label>
            <Input
              id="k"
              type="password"
              dir="ltr"
              value={keyInput}
              onChange={(e) => setKeyInput(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && keyInput.trim() && login()}
              placeholder="کلید ادمین سرور"
            />
          </div>
          <Button onClick={login} disabled={!keyInput.trim()}>
            بازکردن پنل
          </Button>
          <p className="text-xs text-[var(--color-muted-foreground)]">
            کرنل: <code dir="ltr" className="text-[var(--color-royal-400)]">{KERNEL_URL}</code>
            {health ? ` — نسخهٔ ${health.version}` : " — پاسخ نمی‌دهد"}
          </p>
        </section>
      ) : (
        <>
          {/* نوار تب‌ها + بروزرسانی سراسری */}
          <nav className="flex flex-wrap items-center gap-2">
            {TABS.map((t) => (
              <Button key={t} size="sm" variant={tab === t ? "default" : "ghost"} onClick={() => setTab(t)}>
                {TAB_LABELS[t]}
              </Button>
            ))}
            <Button
              size="sm"
              variant="outline"
              className="mr-auto"
              onClick={() => {
                setRefreshTick((t) => t + 1);
                refreshHealth();
              }}
            >
              🔄 بروزرسانی داده‌ها
            </Button>
          </nav>
          {loading && <p className="text-xs text-[var(--color-muted-foreground)]">در حال بارگذاری…</p>}
          {/* key={refreshTick} — remount تب فعال با هر «بروزرسانی» = بارگیری مجدد داده */}
          {tab === "overview" && <OverviewTab key={refreshTick} safe={safe} refreshHealth={refreshHealth} />}
          {tab === "withdrawals" && <WithdrawalsTab key={refreshTick} safe={safe} />}
          {tab === "arbitration" && <EscrowArbitrationTab key={refreshTick} safe={safe} />}
          {tab === "hitl" && <HitlTab key={refreshTick} safe={safe} />}
          {tab === "deposits" && <DepositsTab key={refreshTick} safe={safe} />}
          {tab === "parliament" && <ParliamentTab key={refreshTick} safe={safe} />}
          {tab === "market" && <MarketTab key={refreshTick} safe={safe} />}
          {tab === "mint" && <MintTab safe={safe} />}
          {tab === "brain" && <BrainTab key={refreshTick} safe={safe} />}
          {tab === "fleet" && <FleetTab key={refreshTick} safe={safe} />}
          {tab === "audit" && <AuditTab key={refreshTick} safe={safe} />}
          {tab === "legacy" && <LegacyTab />}
        </>
      )}
    </main>
  );
}

/* ── کمکی‌های رندر ─────────────────────────────────────────────────────── */

function Card({ title, value, hint }: { title: string; value: string; hint?: string }) {
  return (
    <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-card)] p-4">
      <p className="text-xs text-[var(--color-muted-foreground)]">{title}</p>
      <p className="text-xl font-bold mt-1" dir="ltr">{value}</p>
      {hint && <p className="text-[11px] text-[var(--color-muted-foreground)] mt-1">{hint}</p>}
    </div>
  );
}

function StatusBadge({ status }: { status: string }) {
  const map: Record<string, "success" | "destructive" | "secondary" | "warning"> = {
    PENDING: "warning", APPROVED: "success", REJECTED: "destructive", EXPIRED: "secondary",
    CONFIRMED: "success", active: "success", registered: "secondary", disabled: "destructive",
    OPEN: "warning", PASSED: "success", EXECUTED: "success",
    RELEASED: "success", DISPUTED: "warning", REFUNDED: "secondary", FUNDED: "warning",
    DELIVERED: "warning", CREATED: "secondary", CANCELLED: "secondary",
  };
  return <Badge variant={map[status] ?? "secondary"}>{status}</Badge>;
}

type SafeFn = <T,>(fn: () => Promise<T>) => Promise<T | null>;

/* ── نمای کلی ──────────────────────────────────────────────────────────── */

function OverviewTab({ safe, refreshHealth }: { safe: SafeFn; refreshHealth: () => void }) {
  const [data, setData] = useState<Overview | null>(null);
  const [eco, setEco] = useState<Economy | null>(null);

  const load = useCallback(() => {
    safe(async () => {
      setData(await adminGet<Overview>("/admin/overview"));
      return true;
    });
    // اقتصاد ملی عمومی است — حتی با کلیدِ نادرست کار می‌کند (مادهٔ ۶)
    publicGet<Economy>("/economy")
      .then(setEco)
      .catch(() => setEco(null));
  }, [safe]);

  useEffect(() => {
    load();
    refreshHealth();
  }, [load, refreshHealth]);

  if (!data) return <p className="text-sm text-[var(--color-muted-foreground)]">داده‌ای نیست — کلید را بررسی کنید.</p>;
  const maxRev = Math.max(1, ...data.revenueSeries.map((r) => r.din));

  return (
    <section className="flex flex-col gap-6">
      {/* بخش ۱ — آمار اقتصاد و خزانه (مادهٔ ۶) */}
      {eco && (
        <div className="flex flex-col gap-2">
          <h3 className="text-sm font-bold text-[var(--color-royal-400)]">
            آمار اقتصاد و خزانه (Economy &amp; Overview)
          </h3>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
            <Card title="عرضه در گردش" value={`${din(eco.circulatingSupply)} Đ`} hint="مادهٔ ۶" />
            <Card
              title="خزانهٔ ملی (مالیات انباشته)"
              value={`${din(eco.treasuryBalance)} Đ`}
              hint="۱۰٪ هر تسویه (مادهٔ ۲)"
            />
            <Card title="قفل‌شده در اسکرو" value={`${din(eco.escrowLocked)} Đ`} />
            <Card title="اسکروهای باز" value={din(eco.openEscrows)} />
            <Card title="ضرب امسال / سقف (مادهٔ ۳)" value={`${din(eco.mintedThisYear)} / ${din(eco.yearlyMintCap)} Đ`} />
            <Card title="حساب‌های دفتر" value={din(eco.accountCount)} />
            <Card
              title="توازن دفتر دوطرفه"
              value={eco.balanced ? "متوازن ✓" : "نامتوازن!"}
              hint={`Σ=${din(eco.trialBalanceSum)} Đ`}
            />
            <Card title="عرضهٔ آغازین (genesis)" value={`${din(eco.genesisSupply)} Đ`} />
          </div>
        </div>
      )}

      <div className="flex flex-col gap-2">
        <h3 className="text-sm font-bold text-[var(--color-royal-400)]">آمار عملیات (داخل کرنل)</h3>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
          <Card title="شهروندان تلگرام" value={din(data.users)} />
          <Card title="حساب‌های دفتر" value={din(data.accounts)} />
          <Card title="امانت‌ها" value={din(data.escrows)} />
          <Card title="HITL در انتظار" value={din(data.hitlPending)} hint={data.hitlPending > 0 ? "نیاز به تصمیم" : undefined} />
          <Card title="کل ضرب (مادهٔ ۳)" value={`${din(data.mintedAllTime)} Đ`} />
          <Card title="کل سوزانده" value={`${din(data.burnedAllTime)} Đ`} />
          <Card title="فراخوانی‌های مغز" value={din(data.aiCalls)} />
          <Card title="درآمد بازار ۱۴روزه" value={`${din(data.revenueSeries.reduce((a, r) => a + r.din, 0))} Đ`} />
        </div>
      </div>

      <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-card)] p-4">
        <h3 className="text-sm font-bold mb-3">سری درآمد ۱۴ روزه (مادهٔ ۶)</h3>
        {data.revenueSeries.length === 0 ? (
          <p className="text-xs text-[var(--color-muted-foreground)]">هنوز فروشی ثبت نشده.</p>
        ) : (
          <div className="flex items-end gap-1 h-28" dir="ltr">
            {data.revenueSeries.map((r) => (
              <div key={r.day} className="flex-1 flex flex-col items-center gap-1 group">
                <div
                  className="w-full rounded-t bg-[var(--color-royal-500)] min-h-1"
                  style={{ height: `${Math.max(4, (r.din / maxRev) * 100)}%` }}
                  title={`${r.day}: ${r.din} Đ`}
                />
                <span className="text-[9px] text-[var(--color-muted-foreground)]">{r.day.slice(5)}</span>
              </div>
            ))}
          </div>
        )}
      </div>

      <div className="grid md:grid-cols-2 gap-4">
        <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-card)] p-4">
          <h3 className="text-sm font-bold mb-2">فروش به ازای فروشنده</h3>
          {data.salesBySeller.length === 0 && <p className="text-xs text-[var(--color-muted-foreground)]">—</p>}
          <ul className="text-xs flex flex-col gap-1">
            {data.salesBySeller.map((s) => (
              <li key={s.sellerId} className="flex justify-between" dir="ltr">
                <span className="truncate max-w-[60%]">{s.sellerId}</span>
                <span className="font-bold">{din(s.din)} Đ</span>
              </li>
            ))}
          </ul>
        </div>
        <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-card)] p-4">
          <h3 className="text-sm font-bold mb-2">فروش به ازای محصول</h3>
          {data.salesByProduct.length === 0 && <p className="text-xs text-[var(--color-muted-foreground)]">—</p>}
          <ul className="text-xs flex flex-col gap-1">
            {data.salesByProduct.map((s) => (
              <li key={s.productId} className="flex justify-between">
                <span className="truncate max-w-[60%]">{s.title ?? s.productId}</span>
                <span className="font-bold" dir="ltr">{din(s.din)} Đ</span>
              </li>
            ))}
          </ul>
        </div>
      </div>
    </section>
  );
}

/* ── بخش ۲ — مدیریت درخواست‌های برداشت کریپتو ─────────────────────────── */

function WithdrawalsTab({ safe }: { safe: SafeFn }) {
  const [items, setItems] = useState<Wdr[] | null>(null);
  const [filter, setFilter] = useState("ALL");
  const [txHash, setTxHash] = useState<Record<string, string>>({});
  const [busyId, setBusyId] = useState<string | null>(null);

  const load = useCallback(() => {
    safe(async () => {
      const res = await adminGet<{ items: Wdr[] }>(
        filter === "ALL"
          ? "/admin/wallet/withdrawals"
          : `/admin/wallet/withdrawals?status=${filter}`,
      );
      setItems(res.items);
      return true;
    });
  }, [safe, filter]);

  useEffect(() => {
    load();
  }, [load]);

  const confirm = async (id: string) => {
    const hash = (txHash[id] ?? "").trim();
    if (hash.length < 8) {
      toast({ title: "TxHash را وارد کنید (حداقل ۸ کاراکتر)", variant: "destructive" });
      return;
    }
    setBusyId(id);
    try {
      await adminPost(`/wallet/withdrawals/${id}/confirm`, { txHash: hash });
      toast({ title: "برداشت تأیید شد — پرداخت روی زنجیره ثبت شد", variant: "success" });
      load();
    } catch (e) {
      toast({
        title: "تأیید ناموفق",
        description: e instanceof Error ? e.message : "خطای شبکه",
        variant: "destructive",
      });
    } finally {
      setBusyId(null);
    }
  };

  const reject = async (id: string) => {
    setBusyId(id);
    try {
      await adminPost(`/wallet/withdrawals/${id}/reject`, {});
      toast({ title: "درخواست رد شد — وجه به کیف شهروند برگشت خورد", variant: "success" });
      load();
    } catch (e) {
      toast({
        title: "رد ناموفق",
        description: e instanceof Error ? e.message : "خطای شبکه",
        variant: "destructive",
      });
    } finally {
      setBusyId(null);
    }
  };

  return (
    <section className="flex flex-col gap-4">
      <div className="flex flex-wrap items-center gap-2">
        {["ALL", "PENDING", "CONFIRMED", "REJECTED"].map((f) => (
          <Button key={f} size="sm" variant={filter === f ? "default" : "ghost"} onClick={() => setFilter(f)}>
            {f === "ALL" ? "همه" : f}
          </Button>
        ))}
        <Button size="sm" variant="outline" className="mr-auto" onClick={() => load()}>
          🔄 بروزرسانی
        </Button>
      </div>

      {!items ? (
        <p className="text-sm text-[var(--color-muted-foreground)]">در حال بارگذاری…</p>
      ) : items.length === 0 ? (
        <p className="text-sm text-[var(--color-muted-foreground)]">هیچ درخواست برداشتی در این وضعیت نیست.</p>
      ) : (
        <ul className="flex flex-col gap-3">
          {items.map((w) => (
            <li key={w.id} className="rounded-lg border border-[var(--color-border)] bg-[var(--color-card)] p-4 flex flex-col gap-2">
              <div className="flex flex-wrap items-center gap-2 text-xs">
                <StatusBadge status={w.status} />
                <span className="font-bold" dir="ltr">{din(w.amountDin)} Đ</span>
                <span className="text-[var(--color-muted-foreground)]" dir="ltr">
                  {w.asset} · {w.chain}
                </span>
                <span className="text-[var(--color-muted-foreground)]" dir="ltr">{w.id}</span>
                <span className="text-[var(--color-muted-foreground)]" dir="ltr">{w.accountId}</span>
              </div>
              <p className="text-xs text-[var(--color-muted-foreground)] truncate" dir="ltr">
                → {w.destinationAddress}
              </p>
              {w.txHash && (
                <p className="text-xs text-emerald-400 truncate" dir="ltr">tx: {w.txHash}</p>
              )}
              {w.status === "PENDING" && (
                <div className="flex flex-wrap items-center gap-2 pt-1">
                  <Input
                    dir="ltr"
                    placeholder="TxHash پرداخت روی زنجیره (0x…)"
                    value={txHash[w.id] ?? ""}
                    onChange={(e) => setTxHash((prev) => ({ ...prev, [w.id]: e.target.value }))}
                    className="max-w-xs"
                  />
                  <Button size="sm" disabled={busyId === w.id} onClick={() => void confirm(w.id)}>
                    ✅ تایید و ثبت TxHash
                  </Button>
                  <Button size="sm" variant="destructive" disabled={busyId === w.id} onClick={() => void reject(w.id)}>
                    ❌ رد درخواست و بازگشت وجه به کیف‌پول
                  </Button>
                </div>
              )}
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}

/* ── بخش ۳ — داوری اسکرو (آزادسازی/بازپرداخت ادمین) ───────────────────── */

function EscrowArbitrationTab({ safe }: { safe: SafeFn }) {
  const [items, setItems] = useState<EscrowRow[] | null>(null);
  const [busyId, setBusyId] = useState<string | null>(null);

  const load = useCallback(() => {
    safe(async () => {
      const res = await adminGet<{ items: EscrowRow[] }>("/escrows?status=DISPUTED");
      setItems(res.items);
      return true;
    });
  }, [safe]);

  useEffect(() => {
    load();
  }, [load]);

  const act = async (id: string, kind: "release" | "refund") => {
    setBusyId(id);
    try {
      await adminPost(`/escrows/${id}/${kind}`, {});
      toast({
        title: kind === "release" ? "وجه برای فروشنده آزاد شد" : "وجه به خریدار بازگشت خورد",
        variant: "success",
      });
      load();
    } catch (e) {
      toast({
        title: "اقدام داوری ناموفق",
        description: e instanceof Error ? e.message : "خطای شبکه",
        variant: "destructive",
      });
    } finally {
      setBusyId(null);
    }
  };

  return (
    <section className="flex flex-col gap-4">
      <p className="text-xs leading-5 text-[var(--color-muted-foreground)]">
        اسکروهای در وضعیت DISPUTED — وجه تا حل اختلاف قفل است. «آزادسازی» =
        تسویهٔ مادهٔ ۲ (مالیات + پورسانت معرف + سهم فروشنده)؛ «بازپرداخت» =
        برگشت کامل به خریدار. صف تأیید انسانی مادهٔ ۷ در تب «صف HITL» است.
      </p>
      <div className="flex justify-end">
        <Button size="sm" variant="outline" onClick={() => load()}>🔄 بروزرسانی</Button>
      </div>
      {!items ? (
        <p className="text-sm text-[var(--color-muted-foreground)]">در حال بارگذاری…</p>
      ) : items.length === 0 ? (
        <p className="text-sm text-[var(--color-muted-foreground)]">هیچ اختلاف بازی در صف داوری نیست. ✅</p>
      ) : (
        <ul className="flex flex-col gap-3">
          {items.map((e) => (
            <li key={e.id} className="rounded-lg border border-[var(--color-border)] bg-[var(--color-card)] p-4 flex flex-col gap-2">
              <div className="flex flex-wrap items-center gap-2 text-xs">
                <StatusBadge status={e.status} />
                <span className="font-bold" dir="ltr">{din(e.amount)} Đ</span>
                <span className="text-[var(--color-muted-foreground)]" dir="ltr">{e.id}</span>
                <span className="text-[var(--color-muted-foreground)]">خریدار: {e.buyerId}</span>
                <span className="text-[var(--color-muted-foreground)]">فروشنده: {e.sellerId}</span>
              </div>
              {e.disputeReason && (
                <p className="text-xs text-amber-400 leading-5">دلیل اختلاف: {e.disputeReason}</p>
              )}
              <div className="flex flex-wrap gap-2 pt-1">
                <Button size="sm" disabled={busyId === e.id} onClick={() => void act(e.id, "release")}>
                  آزادسازی وجه برای فروشنده (RELEASE_TO_SELLER)
                </Button>
                <Button size="sm" variant="destructive" disabled={busyId === e.id} onClick={() => void act(e.id, "refund")}>
                  بازپرداخت به خریدار (REFUND_TO_BUYER)
                </Button>
              </div>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}

/* ── بخش ۴ — شارژ دستی حساب (Direct Admin Mint) ───────────────────────── */

function MintTab({ safe }: { safe: SafeFn }) {
  const [accountId, setAccountId] = useState("citizen:1001");
  const [amount, setAmount] = useState("");
  const [memo, setMemo] = useState("");
  const [busy, setBusy] = useState(false);

  const submit = async (ev: React.FormEvent) => {
    ev.preventDefault();
    const amt = Number.parseInt(amount.trim(), 10);
    if (!Number.isFinite(amt) || amt < 1) {
      toast({ title: "مبلغ نامعتبر", description: "مبلغ باید عدد صحیح بزرگ‌تر از صفر باشد (Đ).", variant: "destructive" });
      return;
    }
    setBusy(true);
    try {
      const res = await adminPost<{ accountId: string; balance: number; txId: string }>("/wallet/mint", {
        accountId: accountId.trim(),
        amount: amt,
        ...(memo.trim() ? { memo: memo.trim() } : {}),
      });
      toast({
        title: "حساب شارژ شد",
        description: `تراز جدید ${res.accountId}: ${din(res.balance)} Đ`,
        variant: "success",
      });
      setAmount("");
      setMemo("");
    } catch (e) {
      toast({
        title: "ضرب ناموفق",
        description: e instanceof Error ? e.message : "خطای شبکه",
        variant: "destructive",
      });
    } finally {
      setBusy(false);
    }
  };

  return (
    <section className="max-w-xl flex flex-col gap-4">
      <p className="text-xs leading-5 text-[var(--color-muted-foreground)]">
        ضرب مستقیم دیین به یک حساب دفتر (مادهٔ ۳ — سقف سالانه اعمال می‌شود).
        حساب شهروند به شکل <code dir="ltr">citizen:1001</code> است؛ خروجی تراکنش
        در ممیزی ثبت می‌شود.
      </p>
      <form onSubmit={(e) => void submit(e)} className="flex flex-col gap-3 rounded-lg border border-[var(--color-border)] bg-[var(--color-card)] p-4">
        <div className="flex flex-col gap-1">
          <Label htmlFor="m-acct">شناسهٔ حساب</Label>
          <Input id="m-acct" dir="ltr" value={accountId} onChange={(e) => setAccountId(e.target.value)} required placeholder="citizen:1001" />
        </div>
        <div className="flex flex-col gap-1">
          <Label htmlFor="m-amt">مبلغ (Đ / micro-KIAN)</Label>
          <Input id="m-amt" dir="ltr" inputMode="numeric" value={amount} onChange={(e) => setAmount(e.target.value)} required placeholder="5000" />
        </div>
        <div className="flex flex-col gap-1">
          <Label htmlFor="m-memo">یادداشت (اختیاری)</Label>
          <Input id="m-memo" value={memo} onChange={(e) => setMemo(e.target.value)} placeholder="شارژ دستی بنیان‌گذار" />
        </div>
        <Button type="submit" disabled={busy}>
          {busy ? "در حال ضرب…" : "ضرب و شارژ حساب"}
        </Button>
      </form>
    </section>
  );
}

/* ── صف HITL ───────────────────────────────────────────────────────────── */

function HitlTab({ safe }: { safe: SafeFn }) {
  const [items, setItems] = useState<Hitl[] | null>(null);
  const [note, setNote] = useState("");

  const load = useCallback(() => {
    safe(async () => {
      const res = await adminGet<{ items: Hitl[] }>("/admin/hitl");
      setItems(res.items);
      return true;
    });
  }, [safe]);

  useEffect(() => { load(); }, [load]);

  const decide = async (id: string, action: "approve" | "reject") => {
    await safe(async () => {
      const res = await adminPost<{ id: string; status: string; settled: boolean; replayed: boolean }>(
        `/admin/hitl/${id}/${action}`,
        note.trim() ? { note: note.trim() } : {},
      );
      toast({
        title:
          action === "approve"
            ? `تأیید شد — ${res.settled ? (res.replayed ? "تسویه replay شد" : "تسویه ۹۰/۱۰ انجام شد") : "بدون تسویه"}`
            : "رد شد — هیچ پولی جابه‌جا نشده",
        variant: res.settled || action === "reject" ? "success" : "default",
      });
      setNote("");
      load();
      return true;
    });
  };

  if (!items) return <p className="text-sm text-[var(--color-muted-foreground)]">در حال بارگذاری صف…</p>;
  return (
    <section className="flex flex-col gap-4">
      <Input placeholder="یادداشت تصمیم (اختیاری)" value={note} onChange={(e) => setNote(e.target.value)} />
      {items.length === 0 && <p className="text-sm text-[var(--color-muted-foreground)]">صف خالی است — درخواست HITL در انتظار نیست.</p>}
      {items.map((h) => (
        <div key={h.id} className="rounded-lg border border-[var(--color-border)] bg-[var(--color-card)] p-4 flex flex-col gap-2">
          <div className="flex flex-wrap items-center justify-between gap-2">
            <div className="flex items-center gap-2">
              <StatusBadge status={h.status} />
              <span className="text-sm font-bold">{h.kind}</span>
              <code dir="ltr" className="text-xs text-[var(--color-muted-foreground)]">{h.id}</code>
            </div>
            <span className="text-sm font-bold text-gold-400" dir="ltr">{din(h.amount)} Đ</span>
          </div>
          <p className="text-xs text-[var(--color-muted-foreground)]" dir="ltr">
            escrow: {h.escrowId} · requester: {h.requesterId} · {h.createdAt}
          </p>
          {h.note && <p className="text-xs">یادداشت: {h.note}</p>}
          {h.status === "PENDING" && (
            <div className="flex gap-2">
              <Button size="sm" onClick={() => decide(h.id, "approve")}>تأیید + تسویهٔ اتمی</Button>
              <Button size="sm" variant="destructive" onClick={() => decide(h.id, "reject")}>رد</Button>
            </div>
          )}
          {h.decidedAt && (
            <p className="text-[11px] text-[var(--color-muted-foreground)]" dir="ltr">
              decided by {h.decidedBy} at {h.decidedAt}{h.decisionNote ? ` — ${h.decisionNote}` : ""}
            </p>
          )}
        </div>
      ))}
    </section>
  );
}

/* ── پارلمان (M5 — /nation/*) ─────────────────────────────────────────── */

const BILL_KIND_LABEL: Record<Bill["kind"], string> = {
  escrowRelease: "حل اختلاف اسکرو",
  treasuryMove: "جابه‌جایی خزانه",
};
const BILL_RESOLUTION_LABEL: Record<NonNullable<Bill["resolution"]>, string> = {
  release: "تسویه ۹۰/۱۰ به نفع فروشنده",
  refund: "بازگشت کامل به خریدار",
};
const DECISION_REASON_LABEL: Record<string, string> = {
  passed: "تصویب — نصاب + اکثریت + مادهٔ ۷",
  quorum_not_met: "نصاب تأمین نشد",
  majority_no: "اکثریت رأی مخالف",
  human_approval_missing: "رأی انسانی مادهٔ ۷ کافی نبود",
};

function ParliamentTab({ safe }: { safe: SafeFn }) {
  const [items, setItems] = useState<Bill[] | null>(null);
  const [filter, setFilter] = useState("ALL");

  // خواندن لایحه‌ها عمومی است (شفافیت مادهٔ ۶) — بدون کلید.
  const load = useCallback(() => {
    safe(async () => {
      const res = await publicGet<{ items: Bill[] }>("/nation/bills");
      setItems(res.items);
      return true;
    });
  }, [safe]);

  useEffect(() => { load(); }, [load]);

  // بستن/اجرا — فقط ادمین؛ پول همیشه از مسیر دامنهٔ خالص M2 می‌گذرد.
  const act = async (id: string, action: "close" | "execute") => {
    await safe(async () => {
      const res = await adminPost<Bill>(`/nation/bills/${id}/${action}`);
      toast({
        title:
          action === "close"
            ? `شمارش قطعی شد — ${res.status}${res.decisionReason ? ` (${DECISION_REASON_LABEL[res.decisionReason] ?? res.decisionReason})` : ""}`
            : "لایحه روی کرنل اجرا شد — هیچ مسیر جانبی برای پول نیست",
        variant: action === "execute" || res.status === "PASSED" ? "success" : "default",
      });
      load();
      return true;
    });
  };

  const shown = items?.filter((b) => filter === "ALL" || b.status === filter) ?? [];

  return (
    <section className="flex flex-col gap-4">
      <Alert variant="royal">
        پارلمان HITL (مادهٔ ۷): باز کردن لایحه و رأی، کنش شهروند با نشست مینی‌اپ است — اینجا فقط
        <b> بستن قطعی </b>و<b> اجرا </b>با X-KIAN-Key انجام می‌شود. لایحه‌های ≥ ۵۰,۰۰۰ Đ رأی انسانی
        می‌خواهند و بدون آن هرگز PASSED نمی‌شوند.
      </Alert>

      <div className="flex gap-2 items-center">
        {["ALL", "OPEN", "PASSED", "REJECTED", "EXECUTED"].map((s) => (
          <Button key={s} size="sm" variant={filter === s ? "default" : "ghost"} onClick={() => setFilter(s)}>
            {s === "ALL" ? "همه" : s}
          </Button>
        ))}
      </div>

      {!items && <p className="text-sm text-[var(--color-muted-foreground)]">در حال بارگذاری لایحه‌ها…</p>}
      {items && shown.length === 0 && (
        <p className="text-sm text-[var(--color-muted-foreground)]">لایحه‌ای با این وضعیت نیست — پارلمان خاموش اما آماده است.</p>
      )}

      {shown.map((b) => (
        <div key={b.id} className="rounded-lg border border-[var(--color-border)] bg-[var(--color-card)] p-4 flex flex-col gap-2">
          <div className="flex flex-wrap items-center justify-between gap-2">
            <div className="flex flex-wrap items-center gap-2">
              <StatusBadge status={b.status} />
              <span className="text-sm font-bold">{b.title}</span>
              <Badge variant="secondary">{BILL_KIND_LABEL[b.kind]}</Badge>
              {b.hitlRequired && <Badge variant="warning">مادهٔ ۷ — رأی انسانی لازم</Badge>}
            </div>
            <span className="text-sm font-bold text-gold-400" dir="ltr">{din(b.amountDin)} Đ</span>
          </div>

          <p className="text-xs text-[var(--color-muted-foreground)] break-all" dir="ltr">
            {b.id} · subject: {b.subject}
            {b.resolution ? ` · ${BILL_RESOLUTION_LABEL[b.resolution]}` : ""} · opened: {b.openedAt}
          </p>

          <p className="text-xs" dir="ltr">
            <span className="text-emerald-400 font-bold">{din(b.votesYes)} yes</span>
            {" / "}
            <span className="text-red-400 font-bold">{din(b.votesNo)} no</span>
            {b.hitlRequired && <> · human-yes: {din(b.humanVotesYes)}</>}
          </p>

          {b.decisionReason && (
            <p className="text-xs text-[var(--color-muted-foreground)]">
              دلیل قطعی: {DECISION_REASON_LABEL[b.decisionReason] ?? b.decisionReason}
            </p>
          )}
          {b.closedAt && (
            <p className="text-[11px] text-[var(--color-muted-foreground)]" dir="ltr">closed: {b.closedAt}</p>
          )}
          {b.executedAt && (
            <p className="text-[11px] text-[var(--color-muted-foreground)]" dir="ltr">executed: {b.executedAt}</p>
          )}

          {b.status === "OPEN" && (
            <div className="flex gap-2">
              <Button size="sm" onClick={() => act(b.id, "close")}>بستن + شمارش قطعی</Button>
            </div>
          )}
          {b.status === "PASSED" && (
            <div className="flex gap-2">
              <Button size="sm" onClick={() => act(b.id, "execute")}>اجرا روی کرنل</Button>
            </div>
          )}
        </div>
      ))}
    </section>
  );
}

/* ── واریزهای TON ──────────────────────────────────────────────────────── */

function DepositsTab({ safe }: { safe: SafeFn }) {
  const [items, setItems] = useState<Deposit[] | null>(null);
  const [status, setStatus] = useState("PENDING");

  const load = useCallback(() => {
    safe(async () => {
      const res = await adminGet<{ items: Deposit[] }>(`/admin/deposits?status=${status}`);
      setItems(res.items);
      return true;
    });
  }, [safe, status]);

  useEffect(() => { load(); }, [load]);

  const decide = async (id: string, action: "confirm" | "reject") => {
    await safe(async () => {
      await adminPost(`/wallet/ton-deposits/${id}/${action}`);
      toast({
        title: action === "confirm" ? "تأیید شد — ضرب مادهٔ ۳ اجرا شد" : "رد شد",
        variant: action === "confirm" ? "success" : "default",
      });
      load();
      return true;
    });
  };

  return (
    <section className="flex flex-col gap-4">
      <div className="flex gap-2 items-center">
        {["PENDING", "CONFIRMED", "REJECTED"].map((s) => (
          <Button key={s} size="sm" variant={status === s ? "default" : "ghost"} onClick={() => setStatus(s)}>
            {s}
          </Button>
        ))}
      </div>
      {items?.length === 0 && <p className="text-sm text-[var(--color-muted-foreground)]">واریزی با این وضعیت نیست.</p>}
      {items?.map((d) => (
        <div key={d.id} className="rounded-lg border border-[var(--color-border)] bg-[var(--color-card)] p-4 flex flex-col gap-2">
          <div className="flex flex-wrap items-center justify-between gap-2">
            <div className="flex items-center gap-2">
              <StatusBadge status={d.status} />
              <code dir="ltr" className="text-xs text-[var(--color-muted-foreground)]">{d.id}</code>
            </div>
            <span className="text-sm font-bold text-gold-400" dir="ltr">{d.amountTon} TON</span>
          </div>
          <p className="text-xs text-[var(--color-muted-foreground)] break-all" dir="ltr">
            user: {d.userId} · tx: {d.txHash.slice(0, 24)}… · wallet: {d.walletAddress.slice(0, 16)}…
          </p>
          {d.status === "PENDING" && (
            <div className="flex gap-2">
              <Button size="sm" onClick={() => decide(d.id, "confirm")}>تأیید + ضرب مادهٔ ۳</Button>
              <Button size="sm" variant="destructive" onClick={() => decide(d.id, "reject")}>رد</Button>
            </div>
          )}
        </div>
      ))}
    </section>
  );
}

/* ── بازار ─────────────────────────────────────────────────────────────── */

function MarketTab({ safe }: { safe: SafeFn }) {
  const [items, setItems] = useState<Product[] | null>(null);

  const load = useCallback(() => {
    safe(async () => {
      const res = await adminGet<{ items: Product[] }>("/admin/products");
      setItems(res.items);
      return true;
    });
  }, [safe]);

  useEffect(() => { load(); }, [load]);

  const remove = async (id: string) => {
    await safe(async () => {
      await adminDelete(`/admin/products/${id}`);
      toast({ title: "محصول حذف شد", variant: "success" });
      load();
      return true;
    });
  };

  if (!items) return <p className="text-sm text-[var(--color-muted-foreground)]">در حال بارگذاری…</p>;
  return (
    <section className="flex flex-col gap-3">
      {items.length === 0 && <p className="text-sm text-[var(--color-muted-foreground)]">محصولی ثبت نشده.</p>}
      {items.map((p) => (
        <div key={p.id} className="rounded-lg border border-[var(--color-border)] bg-[var(--color-card)] p-4 flex flex-wrap items-center justify-between gap-3">
          <div className="min-w-0">
            <div className="flex items-center gap-2">
              <span className="font-bold">{p.title}</span>
              <Badge variant={p.active ? "success" : "secondary"}>{p.active ? "فعال" : "غیرفعال"}</Badge>
            </div>
            <p className="text-xs text-[var(--color-muted-foreground)] truncate max-w-xl" dir="ltr">{p.sellerId}</p>
          </div>
          <div className="flex items-center gap-3">
            <span className="font-bold text-gold-400" dir="ltr">{din(p.priceDin)} Đ</span>
            <Button size="sm" variant="destructive" onClick={() => remove(p.id)}>حذف</Button>
          </div>
        </div>
      ))}
    </section>
  );
}

/* ── مغز ───────────────────────────────────────────────────────────────── */

function BrainTab({ safe }: { safe: SafeFn }) {
  const [ai, setAi] = useState<AiStatus | null>(null);
  const [skills, setSkills] = useState<Skill[] | null>(null);

  useEffect(() => {
    safe(async () => { setAi(await publicGet<AiStatus>("/ai/status")); return true; });
    safe(async () => { setSkills((await publicGet<{ items: Skill[] }>("/hermes/skills")).items); return true; });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <section className="flex flex-col gap-5">
      <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-card)] p-4">
        <h3 className="text-sm font-bold mb-2">گیت‌وی AI — فال‌بک سه‌اسلات</h3>
        {ai && (
          <div className="flex flex-col gap-2 text-xs">
            <p>سقف روزانهٔ هر کاربر: <b dir="ltr">{din(ai.dailyCap)}</b> توکن — گیت سخت قبل از هر فراخوانی</p>
            <div className="grid grid-cols-3 gap-2">
              {ai.slots.length === 0 && <p className="text-[var(--color-muted-foreground)]">اسلاتی کانفیگ نشده — صادقانه خاموش (envهای KIAN_AI_*_URL/KEY/MODEL).</p>}
              {ai.slots.map((s) => (
                <div key={s.slot} className="rounded border border-[var(--color-border)] p-2">
                  <div className="flex items-center justify-between">
                    <b>{s.name}</b>
                    <Badge variant={s.enabled ? "success" : "secondary"}>{s.enabled ? "فعال" : "خاموش"}</Badge>
                  </div>
                  <p className="text-[var(--color-muted-foreground)] mt-1" dir="ltr">{s.model || "—"}</p>
                </div>
              ))}
            </div>
            <Alert variant="royal">کلیدهای اسلات فقط از env کرنل می‌آیند و هرگز روی سیم نمی‌آیند — این پاسخ بدون راز است.</Alert>
          </div>
        )}
      </div>

      <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-card)] p-4">
        <h3 className="text-sm font-bold mb-2">۱۶ مهارت هرمس</h3>
        <div className="grid sm:grid-cols-2 gap-2 text-xs">
          {skills?.map((s) => (
            <div key={s.id} className="rounded border border-[var(--color-border)] p-2">
              <div className="flex items-center justify-between">
                <b dir="ltr">{s.id}</b>
                <Badge variant={s.kind === "kernel" ? "success" : "secondary"}>{s.kind}</Badge>
              </div>
              <p className="mt-1">{s.title} — {s.description}</p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}

/* ── ناوگان ────────────────────────────────────────────────────────────── */

function FleetTab({ safe }: { safe: SafeFn }) {
  const [fleet, setFleet] = useState<Fleet | null>(null);
  const [inbox, setInbox] = useState<A2aMsg[] | null>(null);
  const [selected, setSelected] = useState("support");

  const loadFleet = useCallback(() => {
    safe(async () => { setFleet(await publicGet<Fleet>("/agents/fleet")); return true; });
  }, [safe]);

  useEffect(() => { loadFleet(); }, [loadFleet]);

  const loadInbox = async (agent: string) => {
    setSelected(agent);
    await safe(async () => {
      const res = await adminGet<{ items: A2aMsg[] }>(`/a2a/messages/${agent}`);
      setInbox(res.items);
      return true;
    });
  };

  return (
    <section className="flex flex-col gap-5">
      <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-card)] p-4">
        <h3 className="text-sm font-bold mb-2">ناوگان ۷ نقش — وضعیت صادقانه</h3>
        <div className="grid sm:grid-cols-2 gap-2 text-xs">
          {fleet?.agents.map((a) => (
            <div key={a.id} className="rounded border border-[var(--color-border)] p-2">
              <div className="flex items-center justify-between">
                <b>{a.name}</b>
                <StatusBadge status={a.status} />
              </div>
              <p className="text-[var(--color-muted-foreground)] mt-1">{a.description}</p>
            </div>
          ))}
        </div>
      </div>

      <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-card)] p-4">
        <h3 className="text-sm font-bold mb-2">ابزارهای MCP — allowlist بسته</h3>
        <div className="flex flex-wrap gap-2 text-xs">
          {fleet?.mcpTools.map((t) => (
            <div key={t.name} className="rounded border border-[var(--color-border)] p-2" title={t.description}>
              <b dir="ltr">{t.name}</b>
              {t.adminOnly && <Badge variant="warning" className="ms-1">ادمین</Badge>}
              <p className="text-[var(--color-muted-foreground)] mt-1" dir="ltr">args: {t.args.join(", ") || "—"}</p>
            </div>
          ))}
        </div>
      </div>

      <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-card)] p-4">
        <h3 className="text-sm font-bold mb-2">صندوق A2A — گیت‌وی ادمین</h3>
        <div className="flex flex-wrap gap-2 mb-3">
          {["orchestrator", "merchant", "finance", "support", "analyst", "scout", "guardian"].map((a) => (
            <Button key={a} size="sm" variant={selected === a ? "default" : "ghost"} onClick={() => loadInbox(a)}>
              {a}
            </Button>
          ))}
        </div>
        {inbox?.length === 0 && <p className="text-xs text-[var(--color-muted-foreground)]">صندوق خالی است.</p>}
        <div className="flex flex-col gap-2 text-xs">
          {inbox?.map((m) => (
            <div key={m.id} className="rounded border border-[var(--color-border)] p-2">
              <div className="flex items-center justify-between" dir="ltr">
                <span>{m.fromAgent} → {m.toAgent}</span>
                <Badge variant="secondary">{m.kind}</Badge>
              </div>
              <p className="mt-1">{m.body}</p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}

/* ── ممیزی ─────────────────────────────────────────────────────────────── */

function AuditTab({ safe }: { safe: SafeFn }) {
  const [items, setItems] = useState<AuditEntry[] | null>(null);

  const load = useCallback(() => {
    safe(async () => {
      const res = await adminGet<{ items: AuditEntry[] }>("/admin/audit?limit=80");
      setItems(res.items);
      return true;
    });
  }, [safe]);

  useEffect(() => { load(); }, [load]);

  return (
    <section className="flex flex-col gap-2">
      {items?.length === 0 && <p className="text-sm text-[var(--color-muted-foreground)]">دنبالهٔ ممیزی خالی است.</p>}
      {items?.map((e) => (
        <div key={e.id} className="rounded border border-[var(--color-border)] bg-[var(--color-card)] p-3 text-xs flex flex-wrap items-center justify-between gap-2">
          <div className="flex items-center gap-2">
            <code dir="ltr" className="text-[var(--color-royal-400)]">{e.action}</code>
            <span className="text-[var(--color-muted-foreground)]" dir="ltr">{e.actor}</span>
            {e.target && <code dir="ltr" className="text-[var(--color-muted-foreground)]">{e.target}</code>}
          </div>
          <span className="text-[var(--color-muted-foreground)]" dir="ltr">{e.at}</span>
        </div>
      ))}
    </section>
  );
}

/* ── پوشش legacy ───────────────────────────────────────────────────────── */

function LegacyTab() {
  const done = LEGACY_COVERAGE.filter((c) => c.status === "done").reduce((a, c) => a + c.count, 0);
  const total = LEGACY_COVERAGE.reduce((a, c) => a + c.count, 0);
  return (
    <section className="flex flex-col gap-4">
      <Alert variant="royal">
        جانشین صادقانهٔ <b>web_admin.py</b>: {done} از {total} اندپوینت legacy اکنون روی کرنل KIAN پوشش
        داده شده‌اند؛ باقی‌مانده (کاربران/پشتیبانی کامل و خزانهٔ USDT چهارزنجیره) با M5 وایر می‌شود —
        هیچ عددی اینجا پمپاژ نشده است.
      </Alert>
      <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-card)] p-4 flex flex-col gap-2 text-sm">
        {LEGACY_COVERAGE.map((c) => (
          <div key={c.group} className="flex flex-wrap items-center justify-between gap-2 py-1 border-b border-[var(--color-border)] last:border-0">
            <div>
              <b>{c.group}</b>
              <span className="text-xs text-[var(--color-muted-foreground)] ms-2">({c.count} اندپوینت legacy)</span>
            </div>
            <div className="flex items-center gap-2">
              <span className="text-xs text-[var(--color-muted-foreground)]" dir="ltr">{c.kian}</span>
              <Badge variant={c.status === "done" ? "success" : "secondary"}>
                {c.status === "done" ? "پوشش داده شد" : "M5"}
              </Badge>
            </div>
          </div>
        ))}
      </div>
    </section>
  );
}
