"use client";

import { useCallback, useEffect, useState } from "react";
import Image from "next/image";
import {
  Alert, Badge, Button, Input, Label, toast,
} from "@kian/ui";
import {
  adminDelete, adminGet, adminPost, AdminApiError, getKey, KERNEL_URL, publicGet, setKey,
} from "@/lib/admin-api";

/* ── Wire types — mirror of the kernel contracts (camelCase) ───────────────────── */

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
/** Crypto withdrawal request — mirror of WithdrawalRequestView in crates/contracts. */
type Wdr = {
  id: string; accountId: string; chain: string; asset: string; destinationAddress: string;
  amountDin: number; txHash: string | null; status: string;
  createdAt: number; resolvedAt: number | null;
};
/** Escrow row for admin adjudication — mirror of EscrowView. */
type EscrowRow = {
  id: string; buyerId: string; sellerId: string; amount: number; productRef: string | null;
  status: string; createdAt: string; disputeReason: string | null;
};
/** National economy summary — GET /economy (public, Article 6). */
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

/* ── Honest coverage of the 71 legacy endpoints (web_admin.py) ───────────────────── */
const LEGACY_COVERAGE: { group: string; count: number; kian: string; status: "done" | "partial" | "next" }[] = [
  { group: "\u0627\u062d\u0631\u0627\u0632 \u0648 \u0646\u0634\u0633\u062a \u0639\u0645\u0644\u06cc\u0627\u062a\u06cc", count: 6, kian: "X-KIAN-Key fail-closed + \u0645\u0642\u0627\u06cc\u0633\u0647\u0654 \u0632\u0645\u0627\u0646‌\u062b\u0627\u0628\u062a", status: "done" },
  { group: "\u062f\u0627\u0634\u0628\u0648\u0631\u062f \u0627\u0642\u062a\u0635\u0627\u062f / overview", count: 8, kian: "GET /admin/overview \u0627\u0632 \u062f\u0641\u062a\u0631 \u0648\u0627\u0642\u0639\u06cc", status: "done" },
  { group: "\u0635\u0641 HITL \u067e\u0627\u0631\u0644\u0645\u0627\u0646", count: 5, kian: "GET/POST /admin/hitl — \u062a\u0623\u06cc\u06cc\u062f = \u062a\u0633\u0648\u06cc\u0647\u0654 \u0627\u062a\u0645\u06cc", status: "done" },
  { group: "\u0627\u0645\u0627\u0646\u062a‌\u0647\u0627 (\u0628\u0627\u0632\u0631\u0633 \u0627\u062f\u0645\u06cc\u0646)", count: 9, kian: "\u0645\u0633\u06cc\u0631\u0647\u0627\u06cc public /escrows + MCP escrow.get", status: "done" },
  { group: "\u062e\u0632\u0627\u0646\u0647 \u0648 \u0648\u0627\u0631\u06cc\u0632 TON", count: 7, kian: "GET /admin/deposits + confirm/reject \u0645\u0627\u062f\u0647\u0654 \u06f3", status: "done" },
  { group: "\u0645\u062f\u06cc\u0631\u06cc\u062a \u0628\u0627\u0632\u0627\u0631/\u0645\u062d\u0635\u0648\u0644\u0627\u062a", count: 8, kian: "GET /admin/products + DELETE + \u0645\u0633\u06cc\u0631\u0647\u0627\u06cc M3", status: "done" },
  { group: "\u0645\u063a\u0632 — \u06af\u06cc\u062a‌\u0648\u06cc AI \u0648 \u0645\u0647\u0627\u0631\u062a‌\u0647\u0627", count: 11, kian: "/ai/status + /hermes/* + MCP", status: "done" },
  { group: "\u0646\u0627\u0648\u06af\u0627\u0646 \u0627\u06cc\u062c\u0646\u062a‌\u0647\u0627 (A2A/MCP)", count: 9, kian: "/agents/fleet + /a2a/* + /mcp/invoke", status: "done" },
  { group: "\u0645\u0645\u06cc\u0632\u06cc \u0648 \u0628\u06a9\u0627\u067e", count: 5, kian: "GET /admin/audit + POST /system/backup", status: "done" },
  { group: "\u06a9\u0627\u0631\u0628\u0631\u0627\u0646 \u0648 \u067e\u0634\u062a\u06cc\u0628\u0627\u0646\u06cc", count: 8, kian: "\u0641\u0647\u0631\u0633\u062a \u0634\u0647\u0631\u0648\u0646\u062f\u0627\u0646 — \u0633\u06cc\u0645 M5 (\u067e\u0627\u0631\u0644\u0645\u0627\u0646 \u06a9\u0627\u0645\u0644)", status: "next" },
  { group: "\u0639\u0645\u0644\u06cc\u0627\u062a \u062e\u0632\u0627\u0646\u0647\u0654 USDT \u0686\u0647\u0627\u0631\u0632\u0646\u062c\u06cc\u0631\u0647", count: 5, kian: "crates/wallet — M5", status: "next" },
];

const din = (n: number) => new Intl.NumberFormat("fa-IR").format(n);
const TABS = [
  "overview", "withdrawals", "arbitration", "hitl", "deposits",
  "parliament", "market", "mint", "brain", "fleet", "audit", "legacy",
] as const;
type Tab = (typeof TABS)[number];
const TAB_LABELS: Record<Tab, string> = {
  overview: "\u0622\u0645\u0627\u0631 \u0627\u0642\u062a\u0635\u0627\u062f",
  withdrawals: "\u0628\u0631\u062f\u0627\u0634\u062a‌\u0647\u0627\u06cc \u06a9\u0631\u06cc\u067e\u062a\u0648",
  arbitration: "\u062f\u0627\u0648\u0631\u06cc \u0627\u0633\u06a9\u0631\u0648",
  hitl: "\u0635\u0641 HITL",
  deposits: "\u0648\u0627\u0631\u06cc\u0632\u0647\u0627\u06cc TON",
  parliament: "\u067e\u0627\u0631\u0644\u0645\u0627\u0646",
  market: "\u0628\u0627\u0632\u0627\u0631",
  mint: "\u0634\u0627\u0631\u0698 \u062d\u0633\u0627\u0628",
  brain: "\u0645\u063a\u0632",
  fleet: "\u0646\u0627\u0648\u06af\u0627\u0646",
  audit: "\u0645\u0645\u06cc\u0632\u06cc",
  legacy: "\u067e\u0648\u0634\u0634 legacy",
};

export default function AdminPage() {
  // demo default — the official key comes from the kernel env (KIAN_ADMIN_KEY); in tonight’s test
  // "dev-admin-key" is pre-filled so the founder signs in without friction.
  const [keyInput, setKeyInput] = useState("dev-admin-key");
  const [unlocked, setUnlocked] = useState(false);
  const [tab, setTab] = useState<Tab>("overview");
  const [health, setHealth] = useState<Health | null>(null);
  const [loading, setLoading] = useState(false);
  // refresh tick — each click remounts the active tab and refetches all data
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
    toast({ title: "\u06a9\u0644\u06cc\u062f \u0646\u0634\u0633\u062a \u062b\u0628\u062a \u0634\u062f — \u0628\u0627 \u0628\u0633\u062a\u0646 \u062a\u0628 \u0645\u06cc‌\u0645\u06cc\u0631\u062f" });
  };
  const logout = () => {
    setKey(null);
    setUnlocked(false);
    toast({ title: "\u06a9\u0644\u06cc\u062f \u0646\u0634\u0633\u062a \u067e\u0627\u06a9 \u0634\u062f" });
  };

  // stable — depends on nothing; if it were new on every render, the children’s useCallback
  // would change, creating an infinite fetch/rerender loop (renderer OOM).
  const safe = useCallback(async <T,>(fn: () => Promise<T>): Promise<T | null> => {
    setLoading(true);
    try {
      return await fn();
    } catch (e) {
      const msg = e instanceof AdminApiError ? `${e.code}: ${e.message}` : "\u062e\u0637\u0627\u06cc \u0634\u0628\u06a9\u0647 — \u06a9\u0631\u0646\u0644 \u062f\u0631 \u062f\u0633\u062a\u0631\u0633 \u0646\u06cc\u0633\u062a";
      toast({ title: msg, variant: "destructive" });
      return null;
    } finally {
      setLoading(false);
    }
  }, []);

  return (
    <main className="mx-auto max-w-6xl px-4 py-8 flex flex-col gap-6">
      {/* Royal header */}
      <header className="flex flex-wrap items-center justify-between gap-4">
        <div>
          <h1 className="text-2xl font-extrabold tracking-tight">
            &#x627;&#x62a;&#x627;&#x642; &#x641;&#x631;&#x645;&#x627;&#x646; &#x628;&#x646;&#x6cc;&#x627;&#x646;‌&#x6af;&#x630;&#x627;&#x631; <span className="text-gold-400">KIAN</span>
          </h1>
          <p className="text-sm text-[var(--color-muted-foreground)] mt-1">
            &#x627;&#x642;&#x62a;&#x635;&#x627;&#x62f;&#x60c; &#x628;&#x631;&#x62f;&#x627;&#x634;&#x62a;‌&#x647;&#x627;&#x6cc; &#x6a9;&#x631;&#x6cc;&#x67e;&#x62a;&#x648;&#x60c; &#x62f;&#x627;&#x648;&#x631;&#x6cc; &#x627;&#x633;&#x6a9;&#x631;&#x648; &#x648; &#x634;&#x627;&#x631;&#x698; &#x62d;&#x633;&#x627;&#x628; — &#x647;&#x645;&#x647; &#x62f;&#x631; &#x6cc;&#x6a9; &#x67e;&#x646;&#x644;
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
                &#x6a9;&#x631;&#x646;&#x644; {health.status === "ok" ? "\u0633\u0627\u0644\u0645" : "\u0646\u0627\u0633\u0627\u0644\u0645"}
              </Badge>
              <Badge variant="secondary">&#x645;&#x63a;&#x632;: {health.ai === "enabled" ? "\u0641\u0639\u0627\u0644" : "\u062e\u0627\u0645\u0648\u0634"}</Badge>
              <Badge variant="secondary">&#x628;&#x627;&#x62a;: {health.bot === "enabled" ? "\u0641\u0639\u0627\u0644" : "\u062e\u0627\u0645\u0648\u0634"}</Badge>
            </div>
          )}
          {unlocked && (
            <Button variant="outline" size="sm" onClick={logout}>
              &#x62e;&#x631;&#x648;&#x62c;
            </Button>
          )}
        </div>
      </header>

      {!unlocked ? (
        /* ── Entry gate ── */
        <section className="rounded-xl border border-[var(--color-border)] bg-[var(--color-card)] p-6 max-w-lg mx-auto w-full flex flex-col gap-4">
          <Alert variant="gold">
            &#x6a9;&#x644;&#x6cc;&#x62f; &#x627;&#x62f;&#x645;&#x6cc;&#x646; &#x641;&#x642;&#x637; &#x62f;&#x631; sessionStorage &#x647;&#x645;&#x6cc;&#x646; &#x62a;&#x628; &#x632;&#x646;&#x62f;&#x6af;&#x6cc; &#x645;&#x6cc;‌&#x6a9;&#x646;&#x62f; — &#x628;&#x627; &#x628;&#x633;&#x62a;&#x646; &#x62a;&#x628; &#x67e;&#x627;&#x6a9; &#x645;&#x6cc;‌&#x634;&#x648;&#x62f;.
            &#x6a9;&#x644;&#x6cc;&#x62f; &#x627;&#x632; env &#x6a9;&#x631;&#x646;&#x644; (`KIAN_ADMIN_KEY`) &#x645;&#x6cc;‌&#x622;&#x6cc;&#x62f; &#x648; &#x647;&#x631;&#x6af;&#x632; &#x62f;&#x631; &#x627;&#x6cc;&#x646; &#x67e;&#x646;&#x644; &#x630;&#x62e;&#x6cc;&#x631;&#x647;&#x654; &#x67e;&#x627;&#x6cc;&#x62f;&#x627;&#x631; &#x646;&#x62f;&#x627;&#x631;&#x62f;.
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
              placeholder="\u06a9\u0644\u06cc\u062f \u0627\u062f\u0645\u06cc\u0646 \u0633\u0631\u0648\u0631"
            />
          </div>
          <Button onClick={login} disabled={!keyInput.trim()}>
            &#x628;&#x627;&#x632;&#x6a9;&#x631;&#x62f;&#x646; &#x67e;&#x646;&#x644;
          </Button>
          <p className="text-xs text-[var(--color-muted-foreground)]">
            &#x6a9;&#x631;&#x646;&#x644;: <code dir="ltr" className="text-[var(--color-royal-400)]">{KERNEL_URL}</code>
            {health ? ` — \u0646\u0633\u062e\u0647\u0654 ${health.version}` : " — \u067e\u0627\u0633\u062e \u0646\u0645\u06cc‌\u062f\u0647\u062f"}
          </p>
        </section>
      ) : (
        <>
          {/* Tab bar + global refresh */}
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
              🔄 &#x628;&#x631;&#x648;&#x632;&#x631;&#x633;&#x627;&#x646;&#x6cc; &#x62f;&#x627;&#x62f;&#x647;‌&#x647;&#x627;
            </Button>
          </nav>
          {loading && <p className="text-xs text-[var(--color-muted-foreground)]">&#x62f;&#x631; &#x62d;&#x627;&#x644; &#x628;&#x627;&#x631;&#x6af;&#x630;&#x627;&#x631;&#x6cc;…</p>}
          {/* key={refreshTick} — remount the active tab on every "refresh" = data reload */}
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

/* ── Render helpers ─────────────────────────────────────────────────────── */

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

/* ── Overview ──────────────────────────────────────────────────────────── */

function OverviewTab({ safe, refreshHealth }: { safe: SafeFn; refreshHealth: () => void }) {
  const [data, setData] = useState<Overview | null>(null);
  const [eco, setEco] = useState<Economy | null>(null);

  const load = useCallback(() => {
    safe(async () => {
      setData(await adminGet<Overview>("/admin/overview"));
      return true;
    });
    // the national economy is public — works even with a wrong key (Article 6)
    publicGet<Economy>("/economy")
      .then(setEco)
      .catch(() => setEco(null));
  }, [safe]);

  useEffect(() => {
    load();
    refreshHealth();
  }, [load, refreshHealth]);

  if (!data) return <p className="text-sm text-[var(--color-muted-foreground)]">&#x62f;&#x627;&#x62f;&#x647;‌&#x627;&#x6cc; &#x646;&#x6cc;&#x633;&#x62a; — &#x6a9;&#x644;&#x6cc;&#x62f; &#x631;&#x627; &#x628;&#x631;&#x631;&#x633;&#x6cc; &#x6a9;&#x646;&#x6cc;&#x62f;.</p>;
  const maxRev = Math.max(1, ...data.revenueSeries.map((r) => r.din));

  return (
    <section className="flex flex-col gap-6">
      {/* Section 1 — economy and treasury stats (Article 6) */}
      {eco && (
        <div className="flex flex-col gap-2">
          <h3 className="text-sm font-bold text-[var(--color-royal-400)]">
            &#x622;&#x645;&#x627;&#x631; &#x627;&#x642;&#x62a;&#x635;&#x627;&#x62f; &#x648; &#x62e;&#x632;&#x627;&#x646;&#x647; (Economy &amp; Overview)
          </h3>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
            <Card title="&#x639;&#x631;&#x636;&#x647; &#x62f;&#x631; &#x6af;&#x631;&#x62f;&#x634;" value={`${din(eco.circulatingSupply)} Đ`} hint="&#x645;&#x627;&#x62f;&#x647;&#x654; &#x6f6;" />
            <Card
              title="&#x62e;&#x632;&#x627;&#x646;&#x647;&#x654; &#x645;&#x644;&#x6cc; (&#x645;&#x627;&#x644;&#x6cc;&#x627;&#x62a; &#x627;&#x646;&#x628;&#x627;&#x634;&#x62a;&#x647;)"
              value={`${din(eco.treasuryBalance)} Đ`}
              hint="&#x6f1;&#x6f0;&#x66a; &#x647;&#x631; &#x62a;&#x633;&#x648;&#x6cc;&#x647; (&#x645;&#x627;&#x62f;&#x647;&#x654; &#x6f2;)"
            />
            <Card title="&#x642;&#x641;&#x644;‌&#x634;&#x62f;&#x647; &#x62f;&#x631; &#x627;&#x633;&#x6a9;&#x631;&#x648;" value={`${din(eco.escrowLocked)} Đ`} />
            <Card title="&#x627;&#x633;&#x6a9;&#x631;&#x648;&#x647;&#x627;&#x6cc; &#x628;&#x627;&#x632;" value={din(eco.openEscrows)} />
            <Card title="&#x636;&#x631;&#x628; &#x627;&#x645;&#x633;&#x627;&#x644; / &#x633;&#x642;&#x641; (&#x645;&#x627;&#x62f;&#x647;&#x654; &#x6f3;)" value={`${din(eco.mintedThisYear)} / ${din(eco.yearlyMintCap)} Đ`} />
            <Card title="&#x62d;&#x633;&#x627;&#x628;‌&#x647;&#x627;&#x6cc; &#x62f;&#x641;&#x62a;&#x631;" value={din(eco.accountCount)} />
            <Card
              title="&#x62a;&#x648;&#x627;&#x632;&#x646; &#x62f;&#x641;&#x62a;&#x631; &#x62f;&#x648;&#x637;&#x631;&#x641;&#x647;"
              value={eco.balanced ? "\u0645\u062a\u0648\u0627\u0632\u0646 ✓" : "\u0646\u0627\u0645\u062a\u0648\u0627\u0632\u0646!"}
              hint={`Σ=${din(eco.trialBalanceSum)} Đ`}
            />
            <Card title="&#x639;&#x631;&#x636;&#x647;&#x654; &#x622;&#x63a;&#x627;&#x632;&#x6cc;&#x646; (genesis)" value={`${din(eco.genesisSupply)} Đ`} />
          </div>
        </div>
      )}

      <div className="flex flex-col gap-2">
        <h3 className="text-sm font-bold text-[var(--color-royal-400)]">&#x622;&#x645;&#x627;&#x631; &#x639;&#x645;&#x644;&#x6cc;&#x627;&#x62a; (&#x62f;&#x627;&#x62e;&#x644; &#x6a9;&#x631;&#x646;&#x644;)</h3>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
          <Card title="&#x634;&#x647;&#x631;&#x648;&#x646;&#x62f;&#x627;&#x646; &#x62a;&#x644;&#x6af;&#x631;&#x627;&#x645;" value={din(data.users)} />
          <Card title="&#x62d;&#x633;&#x627;&#x628;‌&#x647;&#x627;&#x6cc; &#x62f;&#x641;&#x62a;&#x631;" value={din(data.accounts)} />
          <Card title="&#x627;&#x645;&#x627;&#x646;&#x62a;‌&#x647;&#x627;" value={din(data.escrows)} />
          <Card title="HITL &#x62f;&#x631; &#x627;&#x646;&#x62a;&#x638;&#x627;&#x631;" value={din(data.hitlPending)} hint={data.hitlPending > 0 ? "\u0646\u06cc\u0627\u0632 \u0628\u0647 \u062a\u0635\u0645\u06cc\u0645" : undefined} />
          <Card title="&#x6a9;&#x644; &#x636;&#x631;&#x628; (&#x645;&#x627;&#x62f;&#x647;&#x654; &#x6f3;)" value={`${din(data.mintedAllTime)} Đ`} />
          <Card title="&#x6a9;&#x644; &#x633;&#x648;&#x632;&#x627;&#x646;&#x62f;&#x647;" value={`${din(data.burnedAllTime)} Đ`} />
          <Card title="&#x641;&#x631;&#x627;&#x62e;&#x648;&#x627;&#x646;&#x6cc;‌&#x647;&#x627;&#x6cc; &#x645;&#x63a;&#x632;" value={din(data.aiCalls)} />
          <Card title="&#x62f;&#x631;&#x622;&#x645;&#x62f; &#x628;&#x627;&#x632;&#x627;&#x631; &#x6f1;&#x6f4;&#x631;&#x648;&#x632;&#x647;" value={`${din(data.revenueSeries.reduce((a, r) => a + r.din, 0))} Đ`} />
        </div>
      </div>

      <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-card)] p-4">
        <h3 className="text-sm font-bold mb-3">&#x633;&#x631;&#x6cc; &#x62f;&#x631;&#x622;&#x645;&#x62f; &#x6f1;&#x6f4; &#x631;&#x648;&#x632;&#x647; (&#x645;&#x627;&#x62f;&#x647;&#x654; &#x6f6;)</h3>
        {data.revenueSeries.length === 0 ? (
          <p className="text-xs text-[var(--color-muted-foreground)]">&#x647;&#x646;&#x648;&#x632; &#x641;&#x631;&#x648;&#x634;&#x6cc; &#x62b;&#x628;&#x62a; &#x646;&#x634;&#x62f;&#x647;.</p>
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
          <h3 className="text-sm font-bold mb-2">&#x641;&#x631;&#x648;&#x634; &#x628;&#x647; &#x627;&#x632;&#x627;&#x6cc; &#x641;&#x631;&#x648;&#x634;&#x646;&#x62f;&#x647;</h3>
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
          <h3 className="text-sm font-bold mb-2">&#x641;&#x631;&#x648;&#x634; &#x628;&#x647; &#x627;&#x632;&#x627;&#x6cc; &#x645;&#x62d;&#x635;&#x648;&#x644;</h3>
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

/* ── Section 2 — crypto withdrawal request management ─────────────────────────── */

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
      toast({ title: "TxHash \u0631\u0627 \u0648\u0627\u0631\u062f \u06a9\u0646\u06cc\u062f (\u062d\u062f\u0627\u0642\u0644 \u06f8 \u06a9\u0627\u0631\u0627\u06a9\u062a\u0631)", variant: "destructive" });
      return;
    }
    setBusyId(id);
    try {
      await adminPost(`/wallet/withdrawals/${id}/confirm`, { txHash: hash });
      toast({ title: "\u0628\u0631\u062f\u0627\u0634\u062a \u062a\u0623\u06cc\u06cc\u062f \u0634\u062f — \u067e\u0631\u062f\u0627\u062e\u062a \u0631\u0648\u06cc \u0632\u0646\u062c\u06cc\u0631\u0647 \u062b\u0628\u062a \u0634\u062f", variant: "success" });
      load();
    } catch (e) {
      toast({
        title: "\u062a\u0623\u06cc\u06cc\u062f \u0646\u0627\u0645\u0648\u0641\u0642",
        description: e instanceof Error ? e.message : "\u062e\u0637\u0627\u06cc \u0634\u0628\u06a9\u0647",
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
      toast({ title: "\u062f\u0631\u062e\u0648\u0627\u0633\u062a \u0631\u062f \u0634\u062f — \u0648\u062c\u0647 \u0628\u0647 \u06a9\u06cc\u0641 \u0634\u0647\u0631\u0648\u0646\u062f \u0628\u0631\u06af\u0634\u062a \u062e\u0648\u0631\u062f", variant: "success" });
      load();
    } catch (e) {
      toast({
        title: "\u0631\u062f \u0646\u0627\u0645\u0648\u0641\u0642",
        description: e instanceof Error ? e.message : "\u062e\u0637\u0627\u06cc \u0634\u0628\u06a9\u0647",
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
            {f === "ALL" ? "\u0647\u0645\u0647" : f}
          </Button>
        ))}
        <Button size="sm" variant="outline" className="mr-auto" onClick={() => load()}>
          🔄 &#x628;&#x631;&#x648;&#x632;&#x631;&#x633;&#x627;&#x646;&#x6cc;
        </Button>
      </div>

      {!items ? (
        <p className="text-sm text-[var(--color-muted-foreground)]">&#x62f;&#x631; &#x62d;&#x627;&#x644; &#x628;&#x627;&#x631;&#x6af;&#x630;&#x627;&#x631;&#x6cc;…</p>
      ) : items.length === 0 ? (
        <p className="text-sm text-[var(--color-muted-foreground)]">&#x647;&#x6cc;&#x686; &#x62f;&#x631;&#x62e;&#x648;&#x627;&#x633;&#x62a; &#x628;&#x631;&#x62f;&#x627;&#x634;&#x62a;&#x6cc; &#x62f;&#x631; &#x627;&#x6cc;&#x646; &#x648;&#x636;&#x639;&#x6cc;&#x62a; &#x646;&#x6cc;&#x633;&#x62a;.</p>
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
                    placeholder="TxHash &#x67e;&#x631;&#x62f;&#x627;&#x62e;&#x62a; &#x631;&#x648;&#x6cc; &#x632;&#x646;&#x62c;&#x6cc;&#x631;&#x647; (0x…)"
                    value={txHash[w.id] ?? ""}
                    onChange={(e) => setTxHash((prev) => ({ ...prev, [w.id]: e.target.value }))}
                    className="max-w-xs"
                  />
                  <Button size="sm" disabled={busyId === w.id} onClick={() => void confirm(w.id)}>
                    ✅ &#x62a;&#x627;&#x6cc;&#x6cc;&#x62f; &#x648; &#x62b;&#x628;&#x62a; TxHash
                  </Button>
                  <Button size="sm" variant="destructive" disabled={busyId === w.id} onClick={() => void reject(w.id)}>
                    ❌ &#x631;&#x62f; &#x62f;&#x631;&#x62e;&#x648;&#x627;&#x633;&#x62a; &#x648; &#x628;&#x627;&#x632;&#x6af;&#x634;&#x62a; &#x648;&#x62c;&#x647; &#x628;&#x647; &#x6a9;&#x6cc;&#x641;‌&#x67e;&#x648;&#x644;
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

/* ── Section 3 — escrow adjudication (admin release/refund) ───────────────────── */

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
        title: kind === "release" ? "\u0648\u062c\u0647 \u0628\u0631\u0627\u06cc \u0641\u0631\u0648\u0634\u0646\u062f\u0647 \u0622\u0632\u0627\u062f \u0634\u062f" : "\u0648\u062c\u0647 \u0628\u0647 \u062e\u0631\u06cc\u062f\u0627\u0631 \u0628\u0627\u0632\u06af\u0634\u062a \u062e\u0648\u0631\u062f",
        variant: "success",
      });
      load();
    } catch (e) {
      toast({
        title: "\u0627\u0642\u062f\u0627\u0645 \u062f\u0627\u0648\u0631\u06cc \u0646\u0627\u0645\u0648\u0641\u0642",
        description: e instanceof Error ? e.message : "\u062e\u0637\u0627\u06cc \u0634\u0628\u06a9\u0647",
        variant: "destructive",
      });
    } finally {
      setBusyId(null);
    }
  };

  return (
    <section className="flex flex-col gap-4">
      <p className="text-xs leading-5 text-[var(--color-muted-foreground)]">
        &#x627;&#x633;&#x6a9;&#x631;&#x648;&#x647;&#x627;&#x6cc; &#x62f;&#x631; &#x648;&#x636;&#x639;&#x6cc;&#x62a; DISPUTED — &#x648;&#x62c;&#x647; &#x62a;&#x627; &#x62d;&#x644; &#x627;&#x62e;&#x62a;&#x644;&#x627;&#x641; &#x642;&#x641;&#x644; &#x627;&#x633;&#x62a;. «&#x622;&#x632;&#x627;&#x62f;&#x633;&#x627;&#x632;&#x6cc;» =
        &#x62a;&#x633;&#x648;&#x6cc;&#x647;&#x654; &#x645;&#x627;&#x62f;&#x647;&#x654; &#x6f2; (&#x645;&#x627;&#x644;&#x6cc;&#x627;&#x62a; + &#x67e;&#x648;&#x631;&#x633;&#x627;&#x646;&#x62a; &#x645;&#x639;&#x631;&#x641; + &#x633;&#x647;&#x645; &#x641;&#x631;&#x648;&#x634;&#x646;&#x62f;&#x647;)&#x61b; «&#x628;&#x627;&#x632;&#x67e;&#x631;&#x62f;&#x627;&#x62e;&#x62a;» =
        &#x628;&#x631;&#x6af;&#x634;&#x62a; &#x6a9;&#x627;&#x645;&#x644; &#x628;&#x647; &#x62e;&#x631;&#x6cc;&#x62f;&#x627;&#x631;. &#x635;&#x641; &#x62a;&#x623;&#x6cc;&#x6cc;&#x62f; &#x627;&#x646;&#x633;&#x627;&#x646;&#x6cc; &#x645;&#x627;&#x62f;&#x647;&#x654; &#x6f7; &#x62f;&#x631; &#x62a;&#x628; «&#x635;&#x641; HITL» &#x627;&#x633;&#x62a;.
      </p>
      <div className="flex justify-end">
        <Button size="sm" variant="outline" onClick={() => load()}>🔄 &#x628;&#x631;&#x648;&#x632;&#x631;&#x633;&#x627;&#x646;&#x6cc;</Button>
      </div>
      {!items ? (
        <p className="text-sm text-[var(--color-muted-foreground)]">&#x62f;&#x631; &#x62d;&#x627;&#x644; &#x628;&#x627;&#x631;&#x6af;&#x630;&#x627;&#x631;&#x6cc;…</p>
      ) : items.length === 0 ? (
        <p className="text-sm text-[var(--color-muted-foreground)]">&#x647;&#x6cc;&#x686; &#x627;&#x62e;&#x62a;&#x644;&#x627;&#x641; &#x628;&#x627;&#x632;&#x6cc; &#x62f;&#x631; &#x635;&#x641; &#x62f;&#x627;&#x648;&#x631;&#x6cc; &#x646;&#x6cc;&#x633;&#x62a;. ✅</p>
      ) : (
        <ul className="flex flex-col gap-3">
          {items.map((e) => (
            <li key={e.id} className="rounded-lg border border-[var(--color-border)] bg-[var(--color-card)] p-4 flex flex-col gap-2">
              <div className="flex flex-wrap items-center gap-2 text-xs">
                <StatusBadge status={e.status} />
                <span className="font-bold" dir="ltr">{din(e.amount)} Đ</span>
                <span className="text-[var(--color-muted-foreground)]" dir="ltr">{e.id}</span>
                <span className="text-[var(--color-muted-foreground)]">&#x62e;&#x631;&#x6cc;&#x62f;&#x627;&#x631;: {e.buyerId}</span>
                <span className="text-[var(--color-muted-foreground)]">&#x641;&#x631;&#x648;&#x634;&#x646;&#x62f;&#x647;: {e.sellerId}</span>
              </div>
              {e.disputeReason && (
                <p className="text-xs text-amber-400 leading-5">&#x62f;&#x644;&#x6cc;&#x644; &#x627;&#x62e;&#x62a;&#x644;&#x627;&#x641;: {e.disputeReason}</p>
              )}
              <div className="flex flex-wrap gap-2 pt-1">
                <Button size="sm" disabled={busyId === e.id} onClick={() => void act(e.id, "release")}>
                  &#x622;&#x632;&#x627;&#x62f;&#x633;&#x627;&#x632;&#x6cc; &#x648;&#x62c;&#x647; &#x628;&#x631;&#x627;&#x6cc; &#x641;&#x631;&#x648;&#x634;&#x646;&#x62f;&#x647; (RELEASE_TO_SELLER)
                </Button>
                <Button size="sm" variant="destructive" disabled={busyId === e.id} onClick={() => void act(e.id, "refund")}>
                  &#x628;&#x627;&#x632;&#x67e;&#x631;&#x62f;&#x627;&#x62e;&#x62a; &#x628;&#x647; &#x62e;&#x631;&#x6cc;&#x62f;&#x627;&#x631; (REFUND_TO_BUYER)
                </Button>
              </div>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}

/* ── Section 4 — manual account funding (Direct Admin Mint) ───────────────────────── */

function MintTab({ safe }: { safe: SafeFn }) {
  const [accountId, setAccountId] = useState("citizen:1001");
  const [amount, setAmount] = useState("");
  const [memo, setMemo] = useState("");
  const [busy, setBusy] = useState(false);

  const submit = async (ev: React.FormEvent) => {
    ev.preventDefault();
    const amt = Number.parseInt(amount.trim(), 10);
    if (!Number.isFinite(amt) || amt < 1) {
      toast({ title: "\u0645\u0628\u0644\u063a \u0646\u0627\u0645\u0639\u062a\u0628\u0631", description: "\u0645\u0628\u0644\u063a \u0628\u0627\u06cc\u062f \u0639\u062f\u062f \u0635\u062d\u06cc\u062d \u0628\u0632\u0631\u06af‌\u062a\u0631 \u0627\u0632 \u0635\u0641\u0631 \u0628\u0627\u0634\u062f (Đ).", variant: "destructive" });
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
        title: "\u062d\u0633\u0627\u0628 \u0634\u0627\u0631\u0698 \u0634\u062f",
        description: `\u062a\u0631\u0627\u0632 \u062c\u062f\u06cc\u062f ${res.accountId}: ${din(res.balance)} Đ`,
        variant: "success",
      });
      setAmount("");
      setMemo("");
    } catch (e) {
      toast({
        title: "\u0636\u0631\u0628 \u0646\u0627\u0645\u0648\u0641\u0642",
        description: e instanceof Error ? e.message : "\u062e\u0637\u0627\u06cc \u0634\u0628\u06a9\u0647",
        variant: "destructive",
      });
    } finally {
      setBusy(false);
    }
  };

  return (
    <section className="max-w-xl flex flex-col gap-4">
      <p className="text-xs leading-5 text-[var(--color-muted-foreground)]">
        &#x636;&#x631;&#x628; &#x645;&#x633;&#x62a;&#x642;&#x6cc;&#x645; &#x62f;&#x6cc;&#x6cc;&#x646; &#x628;&#x647; &#x6cc;&#x6a9; &#x62d;&#x633;&#x627;&#x628; &#x62f;&#x641;&#x62a;&#x631; (&#x645;&#x627;&#x62f;&#x647;&#x654; &#x6f3; — &#x633;&#x642;&#x641; &#x633;&#x627;&#x644;&#x627;&#x646;&#x647; &#x627;&#x639;&#x645;&#x627;&#x644; &#x645;&#x6cc;‌&#x634;&#x648;&#x62f;).
        &#x62d;&#x633;&#x627;&#x628; &#x634;&#x647;&#x631;&#x648;&#x646;&#x62f; &#x628;&#x647; &#x634;&#x6a9;&#x644; <code dir="ltr">citizen:1001</code> \u0627\u0633\u062a\u061b \u062e\u0631\u0648\u062c\u06cc \u062a\u0631\u0627\u06a9\u0646\u0634
        &#x62f;&#x631; &#x645;&#x645;&#x6cc;&#x632;&#x6cc; &#x62b;&#x628;&#x62a; &#x645;&#x6cc;‌&#x634;&#x648;&#x62f;.
      </p>
      <form onSubmit={(e) => void submit(e)} className="flex flex-col gap-3 rounded-lg border border-[var(--color-border)] bg-[var(--color-card)] p-4">
        <div className="flex flex-col gap-1">
          <Label htmlFor="m-acct">&#x634;&#x646;&#x627;&#x633;&#x647;&#x654; &#x62d;&#x633;&#x627;&#x628;</Label>
          <Input id="m-acct" dir="ltr" value={accountId} onChange={(e) => setAccountId(e.target.value)} required placeholder="citizen:1001" />
        </div>
        <div className="flex flex-col gap-1">
          <Label htmlFor="m-amt">&#x645;&#x628;&#x644;&#x63a; (Đ / micro-KIAN)</Label>
          <Input id="m-amt" dir="ltr" inputMode="numeric" value={amount} onChange={(e) => setAmount(e.target.value)} required placeholder="5000" />
        </div>
        <div className="flex flex-col gap-1">
          <Label htmlFor="m-memo">&#x6cc;&#x627;&#x62f;&#x62f;&#x627;&#x634;&#x62a; (&#x627;&#x62e;&#x62a;&#x6cc;&#x627;&#x631;&#x6cc;)</Label>
          <Input id="m-memo" value={memo} onChange={(e) => setMemo(e.target.value)} placeholder="\u0634\u0627\u0631\u0698 \u062f\u0633\u062a\u06cc \u0628\u0646\u06cc\u0627\u0646‌\u06af\u0630\u0627\u0631" />
        </div>
        <Button type="submit" disabled={busy}>
          {busy ? "\u062f\u0631 \u062d\u0627\u0644 \u0636\u0631\u0628…" : "\u0636\u0631\u0628 \u0648 \u0634\u0627\u0631\u0698 \u062d\u0633\u0627\u0628"}
        </Button>
      </form>
    </section>
  );
}

/* ── HITL queue ───────────────────────────────────────────────────────────── */

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
            ? `\u062a\u0623\u06cc\u06cc\u062f \u0634\u062f — ${res.settled ? (res.replayed ? "\u062a\u0633\u0648\u06cc\u0647 replay \u0634\u062f" : "\u062a\u0633\u0648\u06cc\u0647 \u06f9\u06f0/\u06f1\u06f0 \u0627\u0646\u062c\u0627\u0645 \u0634\u062f") : "\u0628\u062f\u0648\u0646 \u062a\u0633\u0648\u06cc\u0647"}`
            : "\u0631\u062f \u0634\u062f — \u0647\u06cc\u0686 \u067e\u0648\u0644\u06cc \u062c\u0627\u0628\u0647‌\u062c\u0627 \u0646\u0634\u062f\u0647",
        variant: res.settled || action === "reject" ? "success" : "default",
      });
      setNote("");
      load();
      return true;
    });
  };

  if (!items) return <p className="text-sm text-[var(--color-muted-foreground)]">&#x62f;&#x631; &#x62d;&#x627;&#x644; &#x628;&#x627;&#x631;&#x6af;&#x630;&#x627;&#x631;&#x6cc; &#x635;&#x641;…</p>;
  return (
    <section className="flex flex-col gap-4">
      <Input placeholder="&#x6cc;&#x627;&#x62f;&#x62f;&#x627;&#x634;&#x62a; &#x62a;&#x635;&#x645;&#x6cc;&#x645; (&#x627;&#x62e;&#x62a;&#x6cc;&#x627;&#x631;&#x6cc;)" value={note} onChange={(e) => setNote(e.target.value)} />
      {items.length === 0 && <p className="text-sm text-[var(--color-muted-foreground)]">&#x635;&#x641; &#x62e;&#x627;&#x644;&#x6cc; &#x627;&#x633;&#x62a; — &#x62f;&#x631;&#x62e;&#x648;&#x627;&#x633;&#x62a; HITL &#x62f;&#x631; &#x627;&#x646;&#x62a;&#x638;&#x627;&#x631; &#x646;&#x6cc;&#x633;&#x62a;.</p>}
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
          {h.note && <p className="text-xs">&#x6cc;&#x627;&#x62f;&#x62f;&#x627;&#x634;&#x62a;: {h.note}</p>}
          {h.status === "PENDING" && (
            <div className="flex gap-2">
              <Button size="sm" onClick={() => decide(h.id, "approve")}>&#x62a;&#x623;&#x6cc;&#x6cc;&#x62f; + &#x62a;&#x633;&#x648;&#x6cc;&#x647;&#x654; &#x627;&#x62a;&#x645;&#x6cc;</Button>
              <Button size="sm" variant="destructive" onClick={() => decide(h.id, "reject")}>&#x631;&#x62f;</Button>
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

/* ── Parliament (M5 — /nation/*) ─────────────────────────────────────────── */

const BILL_KIND_LABEL: Record<Bill["kind"], string> = {
  escrowRelease: "\u062d\u0644 \u0627\u062e\u062a\u0644\u0627\u0641 \u0627\u0633\u06a9\u0631\u0648",
  treasuryMove: "\u062c\u0627\u0628\u0647‌\u062c\u0627\u06cc\u06cc \u062e\u0632\u0627\u0646\u0647",
};
const BILL_RESOLUTION_LABEL: Record<NonNullable<Bill["resolution"]>, string> = {
  release: "\u062a\u0633\u0648\u06cc\u0647 \u06f9\u06f0/\u06f1\u06f0 \u0628\u0647 \u0646\u0641\u0639 \u0641\u0631\u0648\u0634\u0646\u062f\u0647",
  refund: "\u0628\u0627\u0632\u06af\u0634\u062a \u06a9\u0627\u0645\u0644 \u0628\u0647 \u062e\u0631\u06cc\u062f\u0627\u0631",
};
const DECISION_REASON_LABEL: Record<string, string> = {
  passed: "\u062a\u0635\u0648\u06cc\u0628 — \u0646\u0635\u0627\u0628 + \u0627\u06a9\u062b\u0631\u06cc\u062a + \u0645\u0627\u062f\u0647\u0654 \u06f7",
  quorum_not_met: "\u0646\u0635\u0627\u0628 \u062a\u0623\u0645\u06cc\u0646 \u0646\u0634\u062f",
  majority_no: "\u0627\u06a9\u062b\u0631\u06cc\u062a \u0631\u0623\u06cc \u0645\u062e\u0627\u0644\u0641",
  human_approval_missing: "\u0631\u0623\u06cc \u0627\u0646\u0633\u0627\u0646\u06cc \u0645\u0627\u062f\u0647\u0654 \u06f7 \u06a9\u0627\u0641\u06cc \u0646\u0628\u0648\u062f",
};

function ParliamentTab({ safe }: { safe: SafeFn }) {
  const [items, setItems] = useState<Bill[] | null>(null);
  const [filter, setFilter] = useState("ALL");

  // reading bills is public (Article 6 transparency) — no key.
  const load = useCallback(() => {
    safe(async () => {
      const res = await publicGet<{ items: Bill[] }>("/nation/bills");
      setItems(res.items);
      return true;
    });
  }, [safe]);

  useEffect(() => { load(); }, [load]);

  // close/execute — admin only; money always passes through the pure M2 domain path.
  const act = async (id: string, action: "close" | "execute") => {
    await safe(async () => {
      const res = await adminPost<Bill>(`/nation/bills/${id}/${action}`);
      toast({
        title:
          action === "close"
            ? `\u0634\u0645\u0627\u0631\u0634 \u0642\u0637\u0639\u06cc \u0634\u062f — ${res.status}${res.decisionReason ? ` (${DECISION_REASON_LABEL[res.decisionReason] ?? res.decisionReason})` : ""}`
            : "\u0644\u0627\u06cc\u062d\u0647 \u0631\u0648\u06cc \u06a9\u0631\u0646\u0644 \u0627\u062c\u0631\u0627 \u0634\u062f — \u0647\u06cc\u0686 \u0645\u0633\u06cc\u0631 \u062c\u0627\u0646\u0628\u06cc \u0628\u0631\u0627\u06cc \u067e\u0648\u0644 \u0646\u06cc\u0633\u062a",
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
        &#x67e;&#x627;&#x631;&#x644;&#x645;&#x627;&#x646; HITL (&#x645;&#x627;&#x62f;&#x647;&#x654; &#x6f7;): &#x628;&#x627;&#x632; &#x6a9;&#x631;&#x62f;&#x646; &#x644;&#x627;&#x6cc;&#x62d;&#x647; &#x648; &#x631;&#x623;&#x6cc;&#x60c; &#x6a9;&#x646;&#x634; &#x634;&#x647;&#x631;&#x648;&#x646;&#x62f; &#x628;&#x627; &#x646;&#x634;&#x633;&#x62a; &#x645;&#x6cc;&#x646;&#x6cc;‌&#x627;&#x67e; &#x627;&#x633;&#x62a; — &#x627;&#x6cc;&#x646;&#x62c;&#x627; &#x641;&#x642;&#x637;
        <b> &#x628;&#x633;&#x62a;&#x646; &#x642;&#x637;&#x639;&#x6cc; </b>\u0648<b> \u0627\u062c\u0631\u0627 </b>&#x628;&#x627; X-KIAN-Key &#x627;&#x646;&#x62c;&#x627;&#x645; &#x645;&#x6cc;‌&#x634;&#x648;&#x62f;. &#x644;&#x627;&#x6cc;&#x62d;&#x647;‌&#x647;&#x627;&#x6cc; ≥ &#x6f5;&#x6f0;,&#x6f0;&#x6f0;&#x6f0; Đ &#x631;&#x623;&#x6cc; &#x627;&#x646;&#x633;&#x627;&#x646;&#x6cc;
        &#x645;&#x6cc;‌&#x62e;&#x648;&#x627;&#x647;&#x646;&#x62f; &#x648; &#x628;&#x62f;&#x648;&#x646; &#x622;&#x646; &#x647;&#x631;&#x6af;&#x632; PASSED &#x646;&#x645;&#x6cc;‌&#x634;&#x648;&#x646;&#x62f;.
      </Alert>

      <div className="flex gap-2 items-center">
        {["ALL", "OPEN", "PASSED", "REJECTED", "EXECUTED"].map((s) => (
          <Button key={s} size="sm" variant={filter === s ? "default" : "ghost"} onClick={() => setFilter(s)}>
            {s === "ALL" ? "\u0647\u0645\u0647" : s}
          </Button>
        ))}
      </div>

      {!items && <p className="text-sm text-[var(--color-muted-foreground)]">&#x62f;&#x631; &#x62d;&#x627;&#x644; &#x628;&#x627;&#x631;&#x6af;&#x630;&#x627;&#x631;&#x6cc; &#x644;&#x627;&#x6cc;&#x62d;&#x647;‌&#x647;&#x627;…</p>}
      {items && shown.length === 0 && (
        <p className="text-sm text-[var(--color-muted-foreground)]">&#x644;&#x627;&#x6cc;&#x62d;&#x647;‌&#x627;&#x6cc; &#x628;&#x627; &#x627;&#x6cc;&#x646; &#x648;&#x636;&#x639;&#x6cc;&#x62a; &#x646;&#x6cc;&#x633;&#x62a; — &#x67e;&#x627;&#x631;&#x644;&#x645;&#x627;&#x646; &#x62e;&#x627;&#x645;&#x648;&#x634; &#x627;&#x645;&#x627; &#x622;&#x645;&#x627;&#x62f;&#x647; &#x627;&#x633;&#x62a;.</p>
      )}

      {shown.map((b) => (
        <div key={b.id} className="rounded-lg border border-[var(--color-border)] bg-[var(--color-card)] p-4 flex flex-col gap-2">
          <div className="flex flex-wrap items-center justify-between gap-2">
            <div className="flex flex-wrap items-center gap-2">
              <StatusBadge status={b.status} />
              <span className="text-sm font-bold">{b.title}</span>
              <Badge variant="secondary">{BILL_KIND_LABEL[b.kind]}</Badge>
              {b.hitlRequired && <Badge variant="warning">&#x645;&#x627;&#x62f;&#x647;&#x654; &#x6f7; — &#x631;&#x623;&#x6cc; &#x627;&#x646;&#x633;&#x627;&#x646;&#x6cc; &#x644;&#x627;&#x632;&#x645;</Badge>}
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
              &#x62f;&#x644;&#x6cc;&#x644; &#x642;&#x637;&#x639;&#x6cc;: {DECISION_REASON_LABEL[b.decisionReason] ?? b.decisionReason}
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
              <Button size="sm" onClick={() => act(b.id, "close")}>&#x628;&#x633;&#x62a;&#x646; + &#x634;&#x645;&#x627;&#x631;&#x634; &#x642;&#x637;&#x639;&#x6cc;</Button>
            </div>
          )}
          {b.status === "PASSED" && (
            <div className="flex gap-2">
              <Button size="sm" onClick={() => act(b.id, "execute")}>&#x627;&#x62c;&#x631;&#x627; &#x631;&#x648;&#x6cc; &#x6a9;&#x631;&#x646;&#x644;</Button>
            </div>
          )}
        </div>
      ))}
    </section>
  );
}

/* ── TON deposits ──────────────────────────────────────────────────────── */

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
        title: action === "confirm" ? "\u062a\u0623\u06cc\u06cc\u062f \u0634\u062f — \u0636\u0631\u0628 \u0645\u0627\u062f\u0647\u0654 \u06f3 \u0627\u062c\u0631\u0627 \u0634\u062f" : "\u0631\u062f \u0634\u062f",
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
      {items?.length === 0 && <p className="text-sm text-[var(--color-muted-foreground)]">&#x648;&#x627;&#x631;&#x6cc;&#x632;&#x6cc; &#x628;&#x627; &#x627;&#x6cc;&#x646; &#x648;&#x636;&#x639;&#x6cc;&#x62a; &#x646;&#x6cc;&#x633;&#x62a;.</p>}
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
              <Button size="sm" onClick={() => decide(d.id, "confirm")}>&#x62a;&#x623;&#x6cc;&#x6cc;&#x62f; + &#x636;&#x631;&#x628; &#x645;&#x627;&#x62f;&#x647;&#x654; &#x6f3;</Button>
              <Button size="sm" variant="destructive" onClick={() => decide(d.id, "reject")}>&#x631;&#x62f;</Button>
            </div>
          )}
        </div>
      ))}
    </section>
  );
}

/* ── Market ─────────────────────────────────────────────────────────────── */

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
      toast({ title: "\u0645\u062d\u0635\u0648\u0644 \u062d\u0630\u0641 \u0634\u062f", variant: "success" });
      load();
      return true;
    });
  };

  if (!items) return <p className="text-sm text-[var(--color-muted-foreground)]">&#x62f;&#x631; &#x62d;&#x627;&#x644; &#x628;&#x627;&#x631;&#x6af;&#x630;&#x627;&#x631;&#x6cc;…</p>;
  return (
    <section className="flex flex-col gap-3">
      {items.length === 0 && <p className="text-sm text-[var(--color-muted-foreground)]">&#x645;&#x62d;&#x635;&#x648;&#x644;&#x6cc; &#x62b;&#x628;&#x62a; &#x646;&#x634;&#x62f;&#x647;.</p>}
      {items.map((p) => (
        <div key={p.id} className="rounded-lg border border-[var(--color-border)] bg-[var(--color-card)] p-4 flex flex-wrap items-center justify-between gap-3">
          <div className="min-w-0">
            <div className="flex items-center gap-2">
              <span className="font-bold">{p.title}</span>
              <Badge variant={p.active ? "success" : "secondary"}>{p.active ? "\u0641\u0639\u0627\u0644" : "\u063a\u06cc\u0631\u0641\u0639\u0627\u0644"}</Badge>
            </div>
            <p className="text-xs text-[var(--color-muted-foreground)] truncate max-w-xl" dir="ltr">{p.sellerId}</p>
          </div>
          <div className="flex items-center gap-3">
            <span className="font-bold text-gold-400" dir="ltr">{din(p.priceDin)} Đ</span>
            <Button size="sm" variant="destructive" onClick={() => remove(p.id)}>&#x62d;&#x630;&#x641;</Button>
          </div>
        </div>
      ))}
    </section>
  );
}

/* ── Brain ───────────────────────────────────────────────────────────────── */

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
        <h3 className="text-sm font-bold mb-2">&#x6af;&#x6cc;&#x62a;‌&#x648;&#x6cc; AI — &#x641;&#x627;&#x644;‌&#x628;&#x6a9; &#x633;&#x647;‌&#x627;&#x633;&#x644;&#x627;&#x62a;</h3>
        {ai && (
          <div className="flex flex-col gap-2 text-xs">
            <p>&#x633;&#x642;&#x641; &#x631;&#x648;&#x632;&#x627;&#x646;&#x647;&#x654; &#x647;&#x631; &#x6a9;&#x627;&#x631;&#x628;&#x631;: <b dir="ltr">{din(ai.dailyCap)}</b> \u062a\u0648\u06a9\u0646 — \u06af\u06cc\u062a \u0633\u062e\u062a \u0642\u0628\u0644 \u0627\u0632 \u0647\u0631 \u0641\u0631\u0627\u062e\u0648\u0627\u0646\u06cc</p>
            <div className="grid grid-cols-3 gap-2">
              {ai.slots.length === 0 && <p className="text-[var(--color-muted-foreground)]">&#x627;&#x633;&#x644;&#x627;&#x62a;&#x6cc; &#x6a9;&#x627;&#x646;&#x641;&#x6cc;&#x6af; &#x646;&#x634;&#x62f;&#x647; — &#x635;&#x627;&#x62f;&#x642;&#x627;&#x646;&#x647; &#x62e;&#x627;&#x645;&#x648;&#x634; (env&#x647;&#x627;&#x6cc; KIAN_AI_*_URL/KEY/MODEL).</p>}
              {ai.slots.map((s) => (
                <div key={s.slot} className="rounded border border-[var(--color-border)] p-2">
                  <div className="flex items-center justify-between">
                    <b>{s.name}</b>
                    <Badge variant={s.enabled ? "success" : "secondary"}>{s.enabled ? "\u0641\u0639\u0627\u0644" : "\u062e\u0627\u0645\u0648\u0634"}</Badge>
                  </div>
                  <p className="text-[var(--color-muted-foreground)] mt-1" dir="ltr">{s.model || "—"}</p>
                </div>
              ))}
            </div>
            <Alert variant="royal">&#x6a9;&#x644;&#x6cc;&#x62f;&#x647;&#x627;&#x6cc; &#x627;&#x633;&#x644;&#x627;&#x62a; &#x641;&#x642;&#x637; &#x627;&#x632; env &#x6a9;&#x631;&#x646;&#x644; &#x645;&#x6cc;‌&#x622;&#x6cc;&#x646;&#x62f; &#x648; &#x647;&#x631;&#x6af;&#x632; &#x631;&#x648;&#x6cc; &#x633;&#x6cc;&#x645; &#x646;&#x645;&#x6cc;‌&#x622;&#x6cc;&#x646;&#x62f; — &#x627;&#x6cc;&#x646; &#x67e;&#x627;&#x633;&#x62e; &#x628;&#x62f;&#x648;&#x646; &#x631;&#x627;&#x632; &#x627;&#x633;&#x62a;.</Alert>
          </div>
        )}
      </div>

      <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-card)] p-4">
        <h3 className="text-sm font-bold mb-2">&#x6f1;&#x6f6; &#x645;&#x647;&#x627;&#x631;&#x62a; &#x647;&#x631;&#x645;&#x633;</h3>
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

/* ── Fleet ────────────────────────────────────────────────────────────── */

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
        <h3 className="text-sm font-bold mb-2">&#x646;&#x627;&#x648;&#x6af;&#x627;&#x646; &#x6f7; &#x646;&#x642;&#x634; — &#x648;&#x636;&#x639;&#x6cc;&#x62a; &#x635;&#x627;&#x62f;&#x642;&#x627;&#x646;&#x647;</h3>
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
        <h3 className="text-sm font-bold mb-2">&#x627;&#x628;&#x632;&#x627;&#x631;&#x647;&#x627;&#x6cc; MCP — allowlist &#x628;&#x633;&#x62a;&#x647;</h3>
        <div className="flex flex-wrap gap-2 text-xs">
          {fleet?.mcpTools.map((t) => (
            <div key={t.name} className="rounded border border-[var(--color-border)] p-2" title={t.description}>
              <b dir="ltr">{t.name}</b>
              {t.adminOnly && <Badge variant="warning" className="ms-1">&#x627;&#x62f;&#x645;&#x6cc;&#x646;</Badge>}
              <p className="text-[var(--color-muted-foreground)] mt-1" dir="ltr">args: {t.args.join(", ") || "—"}</p>
            </div>
          ))}
        </div>
      </div>

      <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-card)] p-4">
        <h3 className="text-sm font-bold mb-2">&#x635;&#x646;&#x62f;&#x648;&#x642; A2A — &#x6af;&#x6cc;&#x62a;‌&#x648;&#x6cc; &#x627;&#x62f;&#x645;&#x6cc;&#x646;</h3>
        <div className="flex flex-wrap gap-2 mb-3">
          {["orchestrator", "merchant", "finance", "support", "analyst", "scout", "guardian"].map((a) => (
            <Button key={a} size="sm" variant={selected === a ? "default" : "ghost"} onClick={() => loadInbox(a)}>
              {a}
            </Button>
          ))}
        </div>
        {inbox?.length === 0 && <p className="text-xs text-[var(--color-muted-foreground)]">&#x635;&#x646;&#x62f;&#x648;&#x642; &#x62e;&#x627;&#x644;&#x6cc; &#x627;&#x633;&#x62a;.</p>}
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

/* ── Audit ─────────────────────────────────────────────────────────────── */

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
      {items?.length === 0 && <p className="text-sm text-[var(--color-muted-foreground)]">&#x62f;&#x646;&#x628;&#x627;&#x644;&#x647;&#x654; &#x645;&#x645;&#x6cc;&#x632;&#x6cc; &#x62e;&#x627;&#x644;&#x6cc; &#x627;&#x633;&#x62a;.</p>}
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

/* ── Legacy coverage ───────────────────────────────────────────────────────── */

function LegacyTab() {
  const done = LEGACY_COVERAGE.filter((c) => c.status === "done").reduce((a, c) => a + c.count, 0);
  const total = LEGACY_COVERAGE.reduce((a, c) => a + c.count, 0);
  return (
    <section className="flex flex-col gap-4">
      <Alert variant="royal">
        &#x62c;&#x627;&#x646;&#x634;&#x6cc;&#x646; &#x635;&#x627;&#x62f;&#x642;&#x627;&#x646;&#x647;&#x654; <b>web_admin.py</b>: {done} \u0627\u0632 {total} \u0627\u0646\u062f\u067e\u0648\u06cc\u0646\u062a legacy \u0627\u06a9\u0646\u0648\u0646 \u0631\u0648\u06cc \u06a9\u0631\u0646\u0644 KIAN \u067e\u0648\u0634\u0634
        &#x62f;&#x627;&#x62f;&#x647; &#x634;&#x62f;&#x647;‌&#x627;&#x646;&#x62f;&#x61b; &#x628;&#x627;&#x642;&#x6cc;‌&#x645;&#x627;&#x646;&#x62f;&#x647; (&#x6a9;&#x627;&#x631;&#x628;&#x631;&#x627;&#x646;/\u067e\u0634\u062a\u06cc\u0628\u0627\u0646\u06cc \u06a9\u0627\u0645\u0644 \u0648 \u062e\u0632\u0627\u0646\u0647\u0654 USDT \u0686\u0647\u0627\u0631\u0632\u0646\u062c\u06cc\u0631\u0647) \u0628\u0627 M5 \u0648\u0627\u06cc\u0631 \u0645\u06cc‌\u0634\u0648\u062f —
        &#x647;&#x6cc;&#x686; &#x639;&#x62f;&#x62f;&#x6cc; &#x627;&#x6cc;&#x646;&#x62c;&#x627; &#x67e;&#x645;&#x67e;&#x627;&#x698; &#x646;&#x634;&#x62f;&#x647; &#x627;&#x633;&#x62a;.
      </Alert>
      <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-card)] p-4 flex flex-col gap-2 text-sm">
        {LEGACY_COVERAGE.map((c) => (
          <div key={c.group} className="flex flex-wrap items-center justify-between gap-2 py-1 border-b border-[var(--color-border)] last:border-0">
            <div>
              <b>{c.group}</b>
              <span className="text-xs text-[var(--color-muted-foreground)] ms-2">({c.count} &#x627;&#x646;&#x62f;&#x67e;&#x648;&#x6cc;&#x646;&#x62a; legacy)</span>
            </div>
            <div className="flex items-center gap-2">
              <span className="text-xs text-[var(--color-muted-foreground)]" dir="ltr">{c.kian}</span>
              <Badge variant={c.status === "done" ? "success" : "secondary"}>
                {c.status === "done" ? "\u067e\u0648\u0634\u0634 \u062f\u0627\u062f\u0647 \u0634\u062f" : "M5"}
              </Badge>
            </div>
          </div>
        ))}
      </div>
    </section>
  );
}
