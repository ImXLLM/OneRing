"use client";

/**
 * KIAN miniapp — market: safe HTML preview, viral referral links, atomic purchase
 * with automatic referrer registration, instant product run/download after purchase and buyer dispute filing.
 */

import { useCallback, useEffect, useState } from "react";
import Link from "next/link";
import { Badge, toast } from "@kian/ui";
import {
  AlertTriangle,
  CheckCircle2,
  Eye,
  Gift,
  HandCoins,
  Link2,
  Loader2,
  PackagePlus,
  Rocket,
  Sparkles,
  Truck,
  X,
} from "lucide-react";
import { MiniShell } from "@/components/kian/mini-shell";
import { HtmlPreviewFrame } from "@/components/kian/html-preview-frame";
import { GuestGate, PreviewBanner } from "@/components/kian/guest-gate";
import { useSession } from "@/components/kian/session";
import { friendlyError, friendlyErrorTitle } from "@/lib/friendly";
import {
  api,
  ApiError,
  type EscrowDeliveryView,
  type EscrowView,
  type ProductView,
} from "@kian/sdk";
import { ESCROW_STATUS, formatDin, formatWhen } from "@/lib/format";
import {
  buildTelegramViralLink,
  captureInboundViralLink,
  formatAffiliatePercent,
  getActiveReferrer,
} from "@/lib/referral";

function StatusBadge({ status }: { status: string }) {
  const st = ESCROW_STATUS[status] ?? {
    fa: status,
    className: "bg-zinc-500/15 text-zinc-400 ring-zinc-400/30",
  };
  return (
    <Badge variant="default" className={`ring-1 ${st.className}`}>
      {st.fa}
    </Badge>
  );
}

/** Three-step visual timeline of the escrow deal — understandable to any user. */
function EscrowTimeline({ status }: { status: string }) {
  // current stage: 1 funded, 2 delivered, 3 released; 0 means not yet funded
  const step =
    status === "CREATED"
      ? 0
      : status === "FUNDED"
        ? 1
        : status === "DELIVERED" || status === "DISPUTED"
          ? 2
          : status === "RELEASED"
            ? 3
            : 0;
  const nodes = [
    { n: 1, label: "\u067e\u0631\u062f\u0627\u062e\u062a \u062f\u0631 \u0627\u0645\u0627\u0646\u062a 🔒" },
    { n: 2, label: "\u0641\u0627\u06cc\u0644 HTML \u0622\u0645\u0627\u062f\u0647\u0654 \u0627\u062c\u0631\u0627 🚀" },
    { n: 3, label: "\u062a\u0623\u06cc\u06cc\u062f \u0648 \u0622\u0632\u0627\u062f\u0633\u0627\u0632\u06cc \u0648\u062c\u0647 ✅" },
  ];
  const closedNote: Record<string, string> = {
    DISPUTED: "⚠️ \u0645\u0639\u0627\u0645\u0644\u0647 \u062f\u0631 \u062f\u0627\u0648\u0631\u06cc \u0627\u0633\u062a — \u0648\u062c\u0647 \u062a\u0627 \u062d\u0644 \u0627\u062e\u062a\u0644\u0627\u0641 \u062f\u0631 \u0627\u0645\u0627\u0646\u062a \u0642\u0641\u0644 \u0645\u06cc‌\u0645\u0627\u0646\u062f.",
    REFUNDED: "↩️ \u0648\u062c\u0647 \u0628\u0647 \u062e\u0631\u06cc\u062f\u0627\u0631 \u0628\u0631\u06af\u0634\u062a \u062e\u0648\u0631\u062f \u0648 \u0645\u0639\u0627\u0645\u0644\u0647 \u0628\u0633\u062a\u0647 \u0634\u062f.",
    CANCELLED: "\u0627\u06cc\u0646 \u0645\u0639\u0627\u0645\u0644\u0647 \u0644\u063a\u0648 \u0634\u062f.",
    EXPIRED: "\u0645\u0647\u0644\u062a \u0627\u06cc\u0646 \u0645\u0639\u0627\u0645\u0644\u0647 \u062a\u0645\u0627\u0645 \u0634\u062f.",
  };
  return (
    <div className="mt-3 rounded-xl bg-zinc-950/70 px-3 py-3 ring-1 ring-zinc-800/80">
      <div className="flex items-start">
        {nodes.map((node, i) => {
          const done = step >= node.n;
          const active = step === node.n - 1 && status !== "RELEASED";
          return (
            <div
              key={node.n}
              className={`flex min-w-0 ${i < nodes.length - 1 ? "flex-1" : ""} items-start`}
            >
              <div className="flex min-w-0 flex-col items-center gap-1 px-1">
                <span
                  className={`grid size-6 shrink-0 place-items-center rounded-full text-[10px] font-black transition-colors ${
                    done
                      ? "bg-emerald-500/25 text-emerald-300 ring-1 ring-emerald-400/50"
                      : active
                        ? "animate-pulse bg-gold-400/20 text-gold-300 ring-2 ring-gold-400/60"
                        : "bg-zinc-800 text-zinc-500 ring-1 ring-zinc-700"
                  }`}
                  aria-hidden
                >
                  {done ? "✓" : node.n.toLocaleString("fa-IR")}
                </span>
                <span
                  className={`max-w-[104px] text-center text-[9.5px] font-bold leading-3.5 ${
                    done
                      ? "text-emerald-300"
                      : active
                        ? "text-gold-300"
                        : "text-zinc-500"
                  }`}
                >
                  {node.label}
                </span>
              </div>
              {i < nodes.length - 1 && (
                <span
                  className={`mt-3 h-0.5 min-w-2 flex-1 rounded ${
                    step > node.n ? "bg-emerald-400/60" : "bg-zinc-800"
                  }`}
                  aria-hidden
                />
              )}
            </div>
          );
        })}
      </div>
      {closedNote[status] && (
        <p className="mt-2.5 rounded-lg bg-amber-950/40 px-2.5 py-1.5 text-[10.5px] leading-4 text-amber-300 ring-1 ring-amber-500/30">
          {closedNote[status]}
        </p>
      )}
    </div>
  );
}

export default function MarketPage() {
  const { state, refresh, activePersona } = useSession();
  const [products, setProducts] = useState<ProductView[]>([]);
  const [orders, setOrders] = useState<EscrowView[]>([]);
  const [myProducts, setMyProducts] = useState<ProductView[]>([]);
  const [busyId, setBusyId] = useState<string | null>(null);

  // demo credit faucet — for demo personas inside the insufficient-balance banner
  const [claimingFaucet, setClaimingFaucet] = useState(false);

  // active referral + the viral direct-link target product
  const [activeReferrer, setActiveReferrer] = useState<string | null>(null);
  const [spotlightProduct, setSpotlightProduct] = useState<ProductView | null>(null);
  const [previewProduct, setPreviewProduct] = useState<ProductView | null>(null);

  // delivery of the purchased HTML and buyer dispute filing
  const [deliveryView, setDeliveryView] = useState<EscrowDeliveryView | null>(null);
  const [disputeEscrowId, setDisputeEscrowId] = useState<string | null>(null);
  const [disputeReason, setDisputeReason] = useState("");

  // friendly insufficient-balance banner — with wallet top-up and referral income shortcuts
  const [needFunds, setNeedFunds] = useState<{ product: ProductView } | null>(null);

  // manual product listing form
  const [showCreate, setShowCreate] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [price, setPrice] = useState("");
  const [category, setCategory] = useState("");
  const [affiliateBps, setAffiliateBps] = useState("1000");

  // read the incoming viral link and load the single target product (even before a session)
  useEffect(() => {
    const inbound = captureInboundViralLink();
    setActiveReferrer(inbound.referrer ?? getActiveReferrer());
    if (inbound.productId) {
      api
        .getProduct(inbound.productId)
        .then((p) => setSpotlightProduct(p))
        .catch(() => {
          /* Product missing — the regular list is shown */
        });
    }
  }, []);

  const loadMarket = useCallback(async (token: string | null) => {
    try {
      const res = await api.listProducts();
      setProducts(res.items);
    } catch {
      /* Public list — silent */
    }
    if (token) {
      try {
        const mine = await api.myMarket(token);
        setOrders(mine.orders);
        setMyProducts(mine.products);
      } catch {
        /* The session may have just died */
      }
    }
  }, []);

  useEffect(() => {
    const token = state.phase === "ready" ? state.token : null;
    void loadMarket(token);
  }, [state, loadMarket]);

  if (state.phase === "loading") {
    return (
      <MiniShell>
        <div className="flex items-center justify-center gap-3 py-24 text-zinc-500">
          <Loader2 className="size-5 animate-spin" aria-hidden />
          <span className="text-sm">&#x628;&#x631;&#x642;&#x631;&#x627;&#x631;&#x6cc; &#x646;&#x634;&#x633;&#x62a; &#x628;&#x627; &#x6a9;&#x631;&#x646;&#x644;…</span>
        </div>
      </MiniShell>
    );
  }

  if (state.phase !== "ready") {
    const preview = state.phase === "preview";
    return (
      <MiniShell>
        <div className="flex flex-col gap-5">
          {preview && <PreviewBanner />}
          {spotlightProduct && (
            <section className="flex flex-col gap-3 rounded-2xl border border-gold-400/35 bg-zinc-900/70 p-4">
              <div className="flex items-start justify-between gap-3">
                <div>
                  <span className="inline-block rounded-full bg-gold-400/15 px-2.5 py-0.5 text-[11px] font-bold text-gold-400">
                    &#x67e;&#x6cc;&#x634;‌&#x646;&#x645;&#x627;&#x6cc;&#x634; &#x644;&#x6cc;&#x646;&#x6a9; &#x645;&#x633;&#x62a;&#x642;&#x6cc;&#x645;
                  </span>
                  <h1 className="mt-1 text-lg font-black">{spotlightProduct.title}</h1>
                  <p className="mt-1 text-xs leading-5 text-zinc-400">
                    {spotlightProduct.description}
                  </p>
                </div>
                <span className="font-mono text-sm font-bold text-gold-400" dir="ltr">
                  {spotlightProduct.priceDin.toLocaleString("fa-IR")} Đ
                </span>
              </div>
              {spotlightProduct.previewHtml && (
                <HtmlPreviewFrame
                  html={spotlightProduct.previewHtml}
                  title={spotlightProduct.title}
                  badgeLabel="&#x67e;&#x6cc;&#x634;‌&#x646;&#x645;&#x627;&#x6cc;&#x634; &#x639;&#x645;&#x648;&#x645;&#x6cc;"
                />
              )}
            </section>
          )}
          {preview ? (
            <p className="rounded-2xl border border-zinc-800 bg-zinc-900/60 px-4 py-3 text-center text-xs leading-6 text-zinc-400">
              &#x62f;&#x631; &#x62d;&#x627;&#x644;&#x62a; &#x645;&#x631;&#x648;&#x631;&#x6af;&#x631; &#x641;&#x642;&#x637; &#x62a;&#x645;&#x627;&#x634;&#x627; &#x641;&#x639;&#x627;&#x644; &#x627;&#x633;&#x62a;&#x61b; &#x628;&#x631;&#x627;&#x6cc; &#x62e;&#x631;&#x6cc;&#x62f; &#x648; &#x641;&#x631;&#x648;&#x634; &#x627;&#x632; &#x62f;&#x627;&#x62e;&#x644; &#x62a;&#x644;&#x6af;&#x631;&#x627;&#x645;
              &#x648;&#x627;&#x631;&#x62f; &#x634;&#x648; — &#x62f;&#x6a9;&#x645;&#x647;&#x654; «&#x62a;&#x644;&#x627;&#x634; &#x628;&#x631;&#x627;&#x6cc; &#x648;&#x631;&#x648;&#x62f; &#x648;&#x627;&#x642;&#x639;&#x6cc;» &#x628;&#x627;&#x644;&#x627;&#x6cc; &#x647;&#x645;&#x6cc;&#x646; &#x635;&#x641;&#x62d;&#x647; &#x627;&#x633;&#x62a;.
            </p>
          ) : (
            <GuestGate
              title="&#x628;&#x627;&#x632;&#x627;&#x631; KIAN &#x628;&#x627; &#x648;&#x631;&#x648;&#x62f; &#x627;&#x632; &#x62a;&#x644;&#x6af;&#x631;&#x627;&#x645; &#x628;&#x627;&#x632; &#x645;&#x6cc;‌&#x634;&#x648;&#x62f;"
              hint="&#x62e;&#x631;&#x6cc;&#x62f; &#x627;&#x62a;&#x645;&#x6cc; &#x628;&#x627; &#x627;&#x633;&#x6a9;&#x631;&#x648; &#x645;&#x627;&#x62f;&#x647;&#x654; &#x6f8; &#x646;&#x6cc;&#x627;&#x632;&#x645;&#x646;&#x62f; &#x646;&#x634;&#x633;&#x62a; &#x627;&#x62d;&#x631;&#x627;&#x632;&#x634;&#x62f;&#x647;&#x654; &#x62a;&#x648;&#x633;&#x62a; — &#x627;&#x645;&#x627; &#x645;&#x6cc;‌&#x62a;&#x648;&#x627;&#x646;&#x6cc; &#x647;&#x645;&#x6cc;&#x646; &#x62d;&#x627;&#x644;&#x627; &#x628;&#x627; «&#x648;&#x631;&#x648;&#x62f; &#x622;&#x632;&#x645;&#x627;&#x6cc;&#x634;&#x6cc;» &#x645;&#x62d;&#x635;&#x648;&#x644;&#x627;&#x62a; &#x648; &#x67e;&#x6cc;&#x634;‌&#x646;&#x645;&#x627;&#x6cc;&#x634;‌&#x647;&#x627; &#x631;&#x627; &#x628;&#x628;&#x6cc;&#x646;&#x6cc;."
            />
          )}
        </div>
      </MiniShell>
    );
  }

  const { me, token } = state;

  // demo credit faucet — demo personas only (in production the server honestly returns 403)
  const claimFaucet = async () => {
    setClaimingFaucet(true);
    try {
      const res = await api.claimDemoFaucet(token);
      toast({
        title: "🎁 \u0627\u0639\u062a\u0628\u0627\u0631 \u062a\u0633\u062a \u0627\u0636\u0627\u0641\u0647 \u0634\u062f",
        description: `${formatDin(res.balance)} \u062f\u0631 \u06a9\u06cc\u0641‌\u067e\u0648\u0644 \u062a\u0648 \u0622\u0645\u0627\u062f\u0647\u0654 \u062e\u0631\u06cc\u062f \u0627\u0633\u062a.`,
        variant: "success",
      });
      setNeedFunds(null);
      await refresh();
    } catch (err) {
      toast({
        title: "\u062f\u0631\u06cc\u0627\u0641\u062a \u0627\u0639\u062a\u0628\u0627\u0631 \u062a\u0633\u062a \u0646\u0627\u0645\u0648\u0641\u0642",
        description: friendlyError(err),
        variant: "destructive",
      });
    } finally {
      setClaimingFaucet(false);
    }
  };

  const purchase = async (p: ProductView) => {
    const refNote =
      activeReferrer && activeReferrer !== me.user.telegramId
        ? `\n\u067e\u0627\u062f\u0627\u0634 \u0645\u0639\u0631\u0641\u06cc (${formatAffiliatePercent(p.affiliateBps)}) \u0628\u0647 \u0645\u0639\u0631\u0641 \u062b\u0628\u062a \u0645\u06cc‌\u0634\u0648\u062f.`
        : "";
    if (
      !window.confirm(
        `\u062e\u0631\u06cc\u062f «${p.title}» \u0628\u0647 \u0642\u06cc\u0645\u062a ${formatDin(p.priceDin)}\u061f\n\u0645\u0628\u0644\u063a \u0628\u0644\u0627\u0641\u0627\u0635\u0644\u0647 \u062f\u0631 \u0627\u0633\u06a9\u0631\u0648 \u0642\u0641\u0644 \u0645\u06cc‌\u0634\u0648\u062f.${refNote}`,
      )
    )
      return;
    setBusyId(p.id);
    try {
      const res = await api.purchase(
        token,
        p.id,
        activeReferrer ? { referrer: activeReferrer } : undefined,
      );
      toast({
        title: res.replayed
          ? "\u0627\u06cc\u0646 \u062e\u0631\u06cc\u062f \u0642\u0628\u0644\u0627\u064b \u062b\u0628\u062a \u0634\u062f\u0647 \u0628\u0648\u062f (replay)"
          : "\u062e\u0631\u06cc\u062f \u0627\u0646\u062c\u0627\u0645 \u0634\u062f — \u0645\u062d\u0635\u0648\u0644 \u0622\u0645\u0627\u062f\u0647\u0654 \u0645\u0634\u0627\u0647\u062f\u0647 \u0648 \u062a\u062d\u0648\u06cc\u0644 \u0627\u0633\u062a",
        description:
          "\u0627\u0632 \u0628\u062e\u0634 «\u0633\u0641\u0627\u0631\u0634‌\u0647\u0627\u06cc \u0645\u0646» \u0641\u0627\u06cc\u0644 HTML \u0631\u0627 \u0628\u0627\u0632 \u06a9\u0646 \u0648 \u067e\u0633 \u0627\u0632 \u0628\u0631\u0631\u0633\u06cc\u060c \u0622\u0632\u0627\u062f\u0633\u0627\u0632\u06cc \u06cc\u0627 \u0627\u062e\u062a\u0644\u0627\u0641 \u062b\u0628\u062a \u06a9\u0646.",
        variant: "success",
      });
      setPreviewProduct(null);
      setNeedFunds(null);
      await Promise.all([loadMarket(token), refresh()]);
    } catch (err) {
      // frictionless purchase experience: insufficient balance → friendly banner + two shortcuts, not a raw error
      if (
        err instanceof ApiError &&
        (err.code === "ACCOUNT_OVERDRAFT" || err.code === "INSUFFICIENT_BALANCE")
      ) {
        setNeedFunds({ product: p });
        toast({
          title: "\u0645\u0648\u062c\u0648\u062f\u06cc \u06a9\u06cc\u0641‌\u067e\u0648\u0644 \u06a9\u0627\u0641\u06cc \u0646\u06cc\u0633\u062a",
          description: "\u0627\u0632 \u0645\u06cc\u0627\u0646‌\u0628\u0631\u0647\u0627\u06cc \u067e\u0627\u06cc\u06cc\u0646 \u0628\u0627\u0632\u0627\u0631: \u0634\u0627\u0631\u0698 \u06a9\u06cc\u0641 \u06cc\u0627 \u062f\u0631\u0622\u0645\u062f \u0631\u0627\u06cc\u06af\u0627\u0646 \u0631\u0641\u0631\u0627\u0644.",
          variant: "destructive",
        });
      } else {
        toast({
          title: friendlyErrorTitle(err),
          description: friendlyError(err),
          variant: "destructive",
        });
      }
    } finally {
      setBusyId(null);
    }
  };

  const copyReferralLink = async (p: ProductView) => {
    const { telegramUrl } = buildTelegramViralLink(p.id, me.user.telegramId);
    try {
      await navigator.clipboard.writeText(telegramUrl);
      toast({
        title: "\u0644\u06cc\u0646\u06a9 \u0631\u0641\u0631\u0627\u0644 \u0634\u0645\u0627 \u06a9\u067e\u06cc \u0634\u062f!",
        description: `\u0628\u0627 \u0647\u0631 \u062e\u0631\u06cc\u062f \u0627\u0632 \u0644\u06cc\u0646\u06a9 \u0634\u0645\u0627\u060c ${formatAffiliatePercent(p.affiliateBps)} \u067e\u0648\u0631\u0633\u0627\u0646\u062a \u0628\u0647 \u06a9\u06cc\u0641 \u0634\u0645\u0627 \u0648\u0627\u0631\u06cc\u0632 \u0645\u06cc‌\u0634\u0648\u062f.`,
        variant: "success",
      });
    } catch {
      toast({
        title: "\u06a9\u067e\u06cc \u062e\u0648\u062f\u06a9\u0627\u0631 \u0645\u062c\u0627\u0632 \u0646\u0628\u0648\u062f",
        description: telegramUrl,
        variant: "destructive",
      });
    }
  };

  const openDelivery = async (escrowId: string) => {
    setBusyId(escrowId);
    try {
      const del = await api.getEscrowDelivery(token, escrowId);
      setDeliveryView(del);
    } catch (err) {
      toast({
        title: "\u062f\u0631\u06cc\u0627\u0641\u062a \u0645\u062d\u062a\u0648\u0627\u06cc \u0645\u062d\u0635\u0648\u0644 \u0646\u0627\u0645\u0648\u0641\u0642 \u0628\u0648\u062f",
        description: friendlyError(err),
        variant: "destructive",
      });
    } finally {
      setBusyId(null);
    }
  };

  const act = async (escrowId: string, kind: "deliver" | "release") => {
    setBusyId(escrowId);
    try {
      if (kind === "deliver") {
        await api.deliver(token, escrowId);
        toast({
          title: "\u062a\u062d\u0648\u06cc\u0644 \u062b\u0628\u062a \u0634\u062f",
          description: "\u062d\u0627\u0644\u0627 \u062e\u0631\u06cc\u062f\u0627\u0631 \u0645\u06cc‌\u062a\u0648\u0627\u0646\u062f \u0622\u0632\u0627\u062f\u0633\u0627\u0632\u06cc \u06a9\u0646\u062f.",
          variant: "success",
        });
      } else {
        await api.release(token, escrowId);
        toast({
          title: "\u0622\u0632\u0627\u062f\u0633\u0627\u0632\u06cc \u0648 \u062a\u0633\u0648\u06cc\u0647 \u0627\u0646\u062c\u0627\u0645 \u0634\u062f",
          description: "\u062a\u0642\u0633\u06cc\u0645 \u062e\u0648\u062f\u06a9\u0627\u0631 \u0628\u06cc\u0646 \u0641\u0631\u0648\u0634\u0646\u062f\u0647\u060c \u062e\u0632\u0627\u0646\u0647 \u0648 \u0645\u0639\u0631\u0641 (\u062f\u0631 \u0635\u0648\u0631\u062a \u0648\u062c\u0648\u062f) \u062b\u0628\u062a \u0634\u062f.",
          variant: "success",
        });
      }
      await Promise.all([loadMarket(token), refresh()]);
    } catch (err) {
      toast({
        title: kind === "deliver" ? "\u062a\u062d\u0648\u06cc\u0644 \u0646\u0627\u0645\u0648\u0641\u0642" : "\u0622\u0632\u0627\u062f\u0633\u0627\u0632\u06cc \u0646\u0627\u0645\u0648\u0641\u0642",
        description: friendlyError(err),
        variant: "destructive",
      });
    } finally {
      setBusyId(null);
    }
  };

  const submitDispute = async (escrowId: string) => {
    const clean = disputeReason.trim();
    if (!clean) {
      toast({ title: "\u062f\u0644\u06cc\u0644 \u0627\u062e\u062a\u0644\u0627\u0641 \u0631\u0627 \u0628\u0646\u0648\u06cc\u0633\u06cc\u062f", variant: "destructive" });
      return;
    }
    setBusyId(escrowId);
    try {
      await api.disputeEscrow(token, escrowId, clean);
      toast({
        title: "\u0627\u062e\u062a\u0644\u0627\u0641 \u062b\u0628\u062a \u0634\u062f — \u0648\u062c\u0647 \u062f\u0631 \u0627\u0633\u06a9\u0631\u0648 \u0642\u0641\u0644 \u0645\u0627\u0646\u062f",
        description: "\u067e\u0631\u0648\u0646\u062f\u0647 \u062a\u0627 \u062f\u0627\u0648\u0631\u06cc \u0628\u0631\u0631\u0633\u06cc \u0645\u06cc‌\u0634\u0648\u062f.",
        variant: "success",
      });
      setDisputeEscrowId(null);
      setDisputeReason("");
      await Promise.all([loadMarket(token), refresh()]);
    } catch (err) {
      toast({
        title: "\u062b\u0628\u062a \u0627\u062e\u062a\u0644\u0627\u0641 \u0646\u0627\u0645\u0648\u0641\u0642 \u0628\u0648\u062f",
        description: friendlyError(err),
        variant: "destructive",
      });
    } finally {
      setBusyId(null);
    }
  };

  const createProduct = async (e: React.FormEvent) => {
    e.preventDefault();
    setSubmitting(true);
    try {
      await api.createProduct(token, {
        title: title.trim(),
        description: description.trim(),
        priceDin: Number(price),
        affiliateBps: Number(affiliateBps) || 1000,
        ...(category.trim() ? { category: category.trim() } : {}),
      });
      toast({ title: "\u0645\u062d\u0635\u0648\u0644 \u062b\u0628\u062a \u0634\u062f \u0648 \u0641\u0639\u0627\u0644 \u0627\u0633\u062a", variant: "success" });
      setTitle("");
      setDescription("");
      setPrice("");
      setCategory("");
      setAffiliateBps("1000");
      setShowCreate(false);
      await loadMarket(token);
    } catch (err) {
      toast({
        title: "\u062b\u0628\u062a \u0645\u062d\u0635\u0648\u0644 \u0646\u0627\u0645\u0648\u0641\u0642",
        description: friendlyError(err),
        variant: "destructive",
      });
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <MiniShell>
      <div className="flex flex-col gap-5">
        {/* Header + AI studio and manual-sale shortcuts */}
        <div className="flex flex-wrap items-center justify-between gap-2">
          <h1 className="text-lg font-black">&#x628;&#x627;&#x632;&#x627;&#x631;</h1>
          <div className="flex items-center gap-2">
            <Link
              href="/builder"
              className="flex items-center gap-1.5 rounded-xl bg-gradient-to-b from-royal-400 to-royal-600 px-3 py-2 text-xs font-bold text-white ring-1 ring-gold-400/40 hover:opacity-90"
            >
              <Sparkles className="size-4" aria-hidden />
              &#x633;&#x627;&#x62e;&#x62a; &#x628;&#x627; AI
            </Link>
            <button
              type="button"
              onClick={() => setShowCreate((v) => !v)}
              className="flex items-center gap-1.5 rounded-xl bg-royal-500/20 px-3 py-2 text-xs font-bold text-gold-400 ring-1 ring-gold-400/30 hover:bg-royal-500/30"
            >
              <PackagePlus className="size-4" aria-hidden />
              &#x641;&#x631;&#x648;&#x634; &#x62f;&#x633;&#x62a;&#x6cc;
            </button>
          </div>
        </div>

        {/* Saved active-referrer badge from the viral link */}
        {activeReferrer && activeReferrer !== me.user.telegramId && (
          <div className="flex items-center gap-2 rounded-xl border border-emerald-500/30 bg-emerald-950/25 px-3 py-2 text-xs text-emerald-300">
            <Gift className="size-4 shrink-0" aria-hidden />
            <span>
              &#x644;&#x6cc;&#x646;&#x6a9; &#x645;&#x639;&#x631;&#x641;&#x6cc; &#x641;&#x639;&#x627;&#x644; &#x627;&#x633;&#x62a; (&#x634;&#x646;&#x627;&#x633;&#x647;&#x654; &#x645;&#x639;&#x631;&#x641;:{" "}
              <strong className="font-mono" dir="ltr">
                #{activeReferrer}
              </strong>
              ) — &#x647;&#x646;&#x6af;&#x627;&#x645; &#x62e;&#x631;&#x6cc;&#x62f; &#x628;&#x647;‌&#x637;&#x648;&#x631; &#x62e;&#x648;&#x62f;&#x6a9;&#x627;&#x631; &#x62b;&#x628;&#x62a; &#x645;&#x6cc;‌&#x634;&#x648;&#x62f;.
            </span>
          </div>
        )}

        {/* Friendly insufficient-balance banner — with two direct shortcuts */}
        {needFunds && (
          <div className="flex flex-col gap-2.5 rounded-2xl border border-amber-500/40 bg-amber-950/25 p-4">
            <p className="text-sm font-black text-amber-300">
              💳 &#x645;&#x648;&#x62c;&#x648;&#x62f;&#x6cc; &#x6a9;&#x6cc;&#x641;‌&#x67e;&#x648;&#x644; &#x628;&#x631;&#x627;&#x6cc; &#x627;&#x6cc;&#x646; &#x62e;&#x631;&#x6cc;&#x62f; &#x6a9;&#x627;&#x641;&#x6cc; &#x646;&#x6cc;&#x633;&#x62a;
            </p>
            <p className="text-xs leading-6 text-zinc-300">
              &#x628;&#x631;&#x627;&#x6cc; &#x62e;&#x631;&#x6cc;&#x62f; «{needFunds.product.title}» &#x628;&#x647;{" "}
              <strong className="font-mono text-gold-400" dir="ltr">
                {formatDin(needFunds.product.priceDin)}
              </strong>{" "}
              &#x646;&#x6cc;&#x627;&#x632; &#x62f;&#x627;&#x631;&#x6cc;&#x61b; &#x645;&#x648;&#x62c;&#x648;&#x62f;&#x6cc; &#x641;&#x639;&#x644;&#x6cc; &#x62a;&#x648;{" "}
              <strong className="font-mono text-zinc-100" dir="ltr">
                {formatDin(me.balance)}
              </strong>{" "}
              &#x627;&#x633;&#x62a;. &#x62f;&#x648; &#x631;&#x627;&#x647; &#x633;&#x631;&#x6cc;&#x639; &#x62f;&#x627;&#x631;&#x6cc;:
            </p>
            <div className="flex flex-wrap gap-2">
              {activePersona && (
                <button
                  type="button"
                  disabled={claimingFaucet}
                  onClick={() => void claimFaucet()}
                  className="flex items-center gap-1.5 rounded-xl bg-gradient-to-b from-emerald-500 to-emerald-600 px-3.5 py-2.5 text-xs font-black text-white shadow-lg shadow-emerald-600/25 ring-1 ring-emerald-400/50 transition hover:opacity-90 disabled:opacity-50"
                >
                  {claimingFaucet ? "\u062f\u0631 \u062d\u0627\u0644 \u062f\u0631\u06cc\u0627\u0641\u062a…" : "🎁 \u062f\u0631\u06cc\u0627\u0641\u062a \u06f5\u066c\u06f0\u06f0\u06f0 KIAN \u0627\u0639\u062a\u0628\u0627\u0631 \u062a\u0633\u062a \u0631\u0627\u06cc\u06af\u0627\u0646 (Faucet)"}
                </button>
              )}
              <Link
                href="/wallet"
                onClick={() => setNeedFunds(null)}
                className="flex items-center gap-1.5 rounded-xl bg-gradient-to-b from-gold-400 to-gold-600 px-3.5 py-2.5 text-xs font-black text-zinc-950 shadow-lg shadow-gold-500/20 ring-1 ring-gold-400/50 hover:opacity-90"
              >
                ➕ &#x634;&#x627;&#x631;&#x698; &#x6a9;&#x6cc;&#x641;‌&#x67e;&#x648;&#x644;
              </Link>
              <button
                type="button"
                onClick={() => void copyReferralLink(needFunds.product)}
                className="flex items-center gap-1.5 rounded-xl bg-emerald-500/15 px-3.5 py-2.5 text-xs font-bold text-emerald-300 ring-1 ring-emerald-400/40 hover:bg-emerald-500/25"
              >
                🔗 &#x6a9;&#x633;&#x628; &#x62f;&#x631;&#x622;&#x645;&#x62f; &#x631;&#x627;&#x6cc;&#x6af;&#x627;&#x646; &#x628;&#x627; &#x6a9;&#x67e;&#x6cc;&#x650; &#x644;&#x6cc;&#x646;&#x6a9; &#x631;&#x641;&#x631;&#x627;&#x644; &#x645;&#x62d;&#x635;&#x648;&#x644;&#x627;&#x62a;
              </button>
              <button
                type="button"
                onClick={() => setNeedFunds(null)}
                className="rounded-xl bg-zinc-800 px-3 py-2.5 text-xs text-zinc-400 hover:bg-zinc-700"
              >
                &#x628;&#x633;&#x62a;&#x646;
              </button>
            </div>
            <p className="text-[11px] leading-5 text-zinc-500">
              &#x631;&#x627;&#x647;&#x646;&#x645;&#x627;: &#x628;&#x627; &#x647;&#x631; &#x62e;&#x631;&#x6cc;&#x62f;&#x650; &#x62f;&#x6cc;&#x6af;&#x631;&#x627;&#x646; &#x627;&#x632; &#x644;&#x6cc;&#x646;&#x6a9; &#x631;&#x641;&#x631;&#x627;&#x644; &#x62a;&#x648;&#x60c; &#x67e;&#x648;&#x631;&#x633;&#x627;&#x646;&#x62a; &#x622;&#x646;&#x6cc; &#x648;&#x627;&#x631;&#x6cc;&#x632; &#x645;&#x6cc;‌&#x634;&#x648;&#x62f;&#x61b;
              &#x6cc;&#x627; &#x628;&#x627; &#x62b;&#x628;&#x62a; &#x627;&#x62b;&#x628;&#x627;&#x62a; &#x648;&#x627;&#x631;&#x6cc;&#x632; TON&#x60c; &#x62f;&#x6cc;&#x6cc;&#x646; &#x636;&#x631;&#x628; &#x645;&#x6cc;‌&#x634;&#x648;&#x62f;.
            </p>
          </div>
        )}

        {/* Special card for the direct-link product (`startapp=p_<id>_r_<ref>`) */}
        {spotlightProduct && (
          <section className="flex flex-col gap-3 rounded-2xl border border-gold-400/40 bg-zinc-900/80 p-4 shadow-xl shadow-black/40">
            <div className="flex items-start justify-between gap-3">
              <div className="min-w-0">
                <span className="inline-flex items-center gap-1 rounded-full bg-gold-400/15 px-2.5 py-0.5 text-[11px] font-bold text-gold-400">
                  &#x644;&#x6cc;&#x646;&#x6a9; &#x645;&#x633;&#x62a;&#x642;&#x6cc;&#x645; &#x645;&#x62d;&#x635;&#x648;&#x644;
                </span>
                <h2 className="mt-1 text-base font-black">{spotlightProduct.title}</h2>
                <p className="mt-1 text-xs leading-5 text-zinc-300">
                  {spotlightProduct.description}
                </p>
              </div>
              <button
                type="button"
                onClick={() => setSpotlightProduct(null)}
                aria-label="\u0628\u0633\u062a\u0646 \u06a9\u0627\u0631\u062a \u0645\u062d\u0635\u0648\u0644"
                className="grid size-7 shrink-0 place-items-center rounded-lg bg-zinc-800 text-zinc-400 hover:text-white"
              >
                <X className="size-4" aria-hidden />
              </button>
            </div>

            {spotlightProduct.previewHtml && (
              <HtmlPreviewFrame
                html={spotlightProduct.previewHtml}
                title={spotlightProduct.title}
                heightClassName="h-[340px]"
                badgeLabel="&#x67e;&#x6cc;&#x634;‌&#x646;&#x645;&#x627;&#x6cc;&#x634; &#x632;&#x646;&#x62f;&#x647; &#x67e;&#x6cc;&#x634; &#x627;&#x632; &#x62e;&#x631;&#x6cc;&#x62f;"
              />
            )}

            <div className="flex flex-wrap items-center justify-between gap-2 pt-1">
              <div className="flex items-center gap-2">
                <span className="font-mono text-base font-black text-gold-400" dir="ltr">
                  {spotlightProduct.priceDin.toLocaleString("fa-IR")} Đ
                </span>
                {spotlightProduct.affiliateBps > 0 && (
                  <span className="rounded-full bg-emerald-500/15 px-2.5 py-0.5 text-[11px] font-bold text-emerald-300 ring-1 ring-emerald-500/30">
                    🎁 {formatAffiliatePercent(spotlightProduct.affiliateBps)} &#x67e;&#x627;&#x62f;&#x627;&#x634; &#x645;&#x639;&#x631;&#x641;&#x6cc;
                  </span>
                )}
              </div>
              <div className="flex items-center gap-2">
                <button
                  type="button"
                  onClick={() => void copyReferralLink(spotlightProduct)}
                  className="flex items-center gap-1 rounded-lg bg-zinc-800 px-3 py-2 text-xs font-bold text-zinc-200 hover:bg-zinc-700"
                >
                  <Link2 className="size-3.5" aria-hidden />
                  &#x6a9;&#x67e;&#x6cc; &#x644;&#x6cc;&#x646;&#x6a9; &#x631;&#x641;&#x631;&#x627;&#x644;
                </button>
                <button
                  type="button"
                  onClick={() => void purchase(spotlightProduct)}
                  disabled={
                    busyId === spotlightProduct.id ||
                    spotlightProduct.sellerId === me.user.id
                  }
                  className="rounded-xl bg-gradient-to-b from-royal-400 to-royal-600 px-4 py-2 text-xs font-black text-white ring-1 ring-gold-400/40 hover:opacity-90 disabled:opacity-40"
                >
                  {spotlightProduct.sellerId === me.user.id
                    ? "\u0645\u062d\u0635\u0648\u0644 \u0645\u0646"
                    : "\u062e\u0631\u06cc\u062f \u0622\u0646\u06cc \u0628\u0627 \u0627\u0633\u06a9\u0631\u0648"}
                </button>
              </div>
            </div>
          </section>
        )}

        {/* manual product listing form */}
        {showCreate && (
          <form
            onSubmit={(e) => void createProduct(e)}
            className="grid gap-3 rounded-2xl border border-royal-500/20 bg-zinc-900/60 p-5"
          >
            <label className="text-xs text-zinc-400">
              &#x639;&#x646;&#x648;&#x627;&#x646;
              <input
                value={title}
                onChange={(e) => setTitle(e.target.value)}
                required
                minLength={3}
                maxLength={120}
                placeholder="\u0645\u062b\u0644\u0627\u064b: \u06a9\u0627\u0646\u0627\u0644 VIP \u0633\u06cc\u06af\u0646\u0627\u0644"
                className="mt-1 w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm outline-none focus:border-gold-400/50"
              />
            </label>
            <label className="text-xs text-zinc-400">
              &#x62a;&#x648;&#x636;&#x6cc;&#x62d;&#x627;&#x62a;
              <textarea
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                required
                minLength={3}
                maxLength={2000}
                rows={3}
                className="mt-1 w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm outline-none focus:border-gold-400/50"
              />
            </label>
            <div className="grid grid-cols-3 gap-3">
              <label className="text-xs text-zinc-400">
                &#x642;&#x6cc;&#x645;&#x62a; (Đ)
                <input
                  value={price}
                  onChange={(e) => setPrice(e.target.value)}
                  required
                  inputMode="numeric"
                  placeholder="1000"
                  dir="ltr"
                  className="mt-1 w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 font-mono text-sm outline-none focus:border-gold-400/50"
                />
              </label>
              <label className="text-xs text-zinc-400">
                &#x67e;&#x648;&#x631;&#x633;&#x627;&#x646;&#x62a; &#x645;&#x639;&#x631;&#x641;
                <select
                  value={affiliateBps}
                  onChange={(e) => setAffiliateBps(e.target.value)}
                  className="mt-1 w-full rounded-lg border border-zinc-700 bg-zinc-950 px-2 py-2 text-sm outline-none focus:border-gold-400/50"
                >
                  <option value="0">&#x6f0;&#x66a;</option>
                  <option value="1000">&#x6f1;&#x6f0;&#x66a;</option>
                  <option value="1500">&#x6f1;&#x6f5;&#x66a;</option>
                  <option value="2000">&#x6f2;&#x6f0;&#x66a;</option>
                  <option value="3000">&#x6f3;&#x6f0;&#x66a;</option>
                  <option value="5000">&#x6f5;&#x6f0;&#x66a;</option>
                </select>
              </label>
              <label className="text-xs text-zinc-400">
                &#x62f;&#x633;&#x62a;&#x647;
                <input
                  value={category}
                  onChange={(e) => setCategory(e.target.value)}
                  maxLength={40}
                  placeholder="\u062e\u062f\u0645\u0627\u062a"
                  className="mt-1 w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm outline-none focus:border-gold-400/50"
                />
              </label>
            </div>
            <button
              type="submit"
              disabled={submitting}
              className="rounded-xl bg-gradient-to-b from-royal-400 to-royal-600 px-4 py-2.5 text-sm font-bold text-white shadow-lg shadow-royal-600/30 ring-1 ring-gold-400/40 hover:opacity-90 disabled:opacity-50"
            >
              {submitting ? "\u062f\u0631 \u062d\u0627\u0644 \u062b\u0628\u062a…" : "\u062b\u0628\u062a \u0645\u062d\u0635\u0648\u0644"}
            </button>
          </form>
        )}

        {/* Active products */}
        <section>
          <h2 className="mb-3 flex items-center gap-2 font-bold">
            <HandCoins className="size-4 text-gold-400" aria-hidden />
            &#x645;&#x62d;&#x635;&#x648;&#x644;&#x627;&#x62a; &#x641;&#x639;&#x627;&#x644;
          </h2>
          {products.length === 0 ? (
            <p className="rounded-2xl border border-zinc-800 bg-zinc-900/60 py-8 text-center text-sm text-zinc-600">
              &#x647;&#x646;&#x648;&#x632; &#x645;&#x62d;&#x635;&#x648;&#x644;&#x6cc; &#x641;&#x639;&#x627;&#x644; &#x646;&#x6cc;&#x633;&#x62a; — &#x627;&#x648;&#x644;&#x6cc;&#x646; &#x645;&#x62d;&#x635;&#x648;&#x644; &#x62a;&#x639;&#x627;&#x645;&#x644;&#x6cc; &#x631;&#x627; &#x62f;&#x631; &#x627;&#x633;&#x62a;&#x648;&#x62f;&#x6cc;&#x648; AI &#x628;&#x633;&#x627;&#x632;.
            </p>
          ) : (
            <ul className="flex flex-col gap-3">
              {products.map((p) => (
                <li
                  key={p.id}
                  className="rounded-2xl border border-zinc-800 bg-zinc-900/60 p-4"
                >
                  <div className="flex items-start justify-between gap-3">
                    <div className="min-w-0">
                      <p className="truncate font-bold">{p.title}</p>
                      <p className="mt-1 line-clamp-2 text-xs leading-5 text-zinc-400">
                        {p.description}
                      </p>
                      <div className="mt-2 flex flex-wrap items-center gap-1.5">
                        {p.category && (
                          <span className="inline-block rounded-full bg-zinc-800 px-2 py-0.5 text-[11px] text-zinc-400">
                            {p.category}
                          </span>
                        )}
                        {p.affiliateBps > 0 && (
                          <span className="inline-flex items-center gap-1 rounded-full bg-emerald-500/15 px-2.5 py-0.5 text-[11px] font-bold text-emerald-300 ring-1 ring-emerald-500/25">
                            🎁 {formatAffiliatePercent(p.affiliateBps)} &#x67e;&#x627;&#x62f;&#x627;&#x634; &#x645;&#x639;&#x631;&#x641;&#x6cc;
                          </span>
                        )}
                      </div>
                    </div>
                    <div className="flex shrink-0 flex-col items-end gap-2">
                      <span className="font-mono text-sm font-bold text-gold-400" dir="ltr">
                        {p.priceDin.toLocaleString("fa-IR")} Đ
                      </span>
                      <button
                        type="button"
                        onClick={() => void purchase(p)}
                        disabled={busyId === p.id || p.sellerId === me.user.id}
                        className="rounded-lg bg-gradient-to-b from-royal-400 to-royal-600 px-3 py-1.5 text-xs font-bold text-white ring-1 ring-gold-400/40 hover:opacity-90 disabled:opacity-40"
                      >
                        {busyId === p.id
                          ? "…"
                          : p.sellerId === me.user.id
                            ? "\u0645\u062d\u0635\u0648\u0644 \u0645\u0646"
                            : "\u062e\u0631\u06cc\u062f"}
                      </button>
                    </div>
                  </div>

                  {/* Live preview and referral link copy buttons */}
                  <div className="mt-3 flex flex-wrap items-center gap-2 border-t border-zinc-800/80 pt-3">
                    {p.previewHtml && (
                      <button
                        type="button"
                        onClick={() => setPreviewProduct(p)}
                        className="flex items-center gap-1.5 rounded-lg bg-royal-500/20 px-2.5 py-1.5 text-[11px] font-bold text-gold-300 ring-1 ring-gold-400/30 hover:bg-royal-500/30"
                      >
                        <Eye className="size-3.5" aria-hidden />
                        &#x67e;&#x6cc;&#x634;‌&#x646;&#x645;&#x627;&#x6cc;&#x634; &#x632;&#x646;&#x62f;&#x647;
                      </button>
                    )}
                    <button
                      type="button"
                      onClick={() => void copyReferralLink(p)}
                      className="flex items-center gap-1.5 rounded-lg bg-zinc-800/90 px-2.5 py-1.5 text-[11px] font-semibold text-zinc-200 ring-1 ring-zinc-700 hover:bg-zinc-700"
                    >
                      <Link2 className="size-3.5 text-emerald-400" aria-hidden />
                      &#x6a9;&#x67e;&#x6cc; &#x644;&#x6cc;&#x646;&#x6a9; &#x631;&#x641;&#x631;&#x627;&#x644; &#x645;&#x646;
                    </button>
                  </div>
                </li>
              ))}
            </ul>
          )}
        </section>

        {/* My orders (escrows + instant HTML delivery + release + dispute filing) */}
        <section>
          <h2 className="mb-3 flex items-center gap-2 font-bold">
            <Truck className="size-4 text-royal-400" aria-hidden />
            &#x633;&#x641;&#x627;&#x631;&#x634;‌&#x647;&#x627;&#x6cc; &#x645;&#x646; (&#x627;&#x633;&#x6a9;&#x631;&#x648;&#x647;&#x627;)
          </h2>
          {orders.length === 0 ? (
            <p className="rounded-2xl border border-zinc-800 bg-zinc-900/60 py-8 text-center text-sm text-zinc-600">
              &#x633;&#x641;&#x627;&#x631;&#x634;&#x6cc; &#x646;&#x62f;&#x627;&#x631;&#x6cc;&#x62f;.
            </p>
          ) : (
            <ul className="flex flex-col gap-3">
              {orders.map((o) => {
                const iAmSeller = o.sellerId === me.user.id;
                const canOpenDelivery =
                  o.productRef !== null &&
                  (o.status === "FUNDED" ||
                    o.status === "DELIVERED" ||
                    o.status === "RELEASED");
                const buyerCanSettle =
                  !iAmSeller &&
                  (o.status === "FUNDED" || o.status === "DELIVERED");
                const disputingThis = disputeEscrowId === o.id;

                return (
                  <li
                    key={o.id}
                    className="rounded-2xl border border-zinc-800 bg-zinc-900/60 p-4"
                  >
                    <div className="flex flex-wrap items-center gap-x-3 gap-y-2">
                      <StatusBadge status={o.status} />
                      <span className="font-mono text-sm font-bold text-gold-400" dir="ltr">
                        {o.amount.toLocaleString("fa-IR")} Đ
                      </span>
                      <span className="text-[11px] text-zinc-500">
                        {iAmSeller ? "\u0641\u0631\u0648\u0634\u0646\u062f\u0647\u0654 \u0627\u06cc\u0646 \u0645\u0639\u0627\u0645\u0644\u0647" : "\u062e\u0631\u06cc\u062f\u0627\u0631 \u0627\u06cc\u0646 \u0645\u0639\u0627\u0645\u0644\u0647"}
                      </span>
                      <span className="mr-auto text-[11px] text-zinc-600">
                        {formatWhen(o.createdAt)}
                      </span>
                    </div>

                    {o.disputeReason && (
                      <p className="mt-2 rounded-lg bg-rose-950/40 px-2.5 py-1.5 text-xs text-rose-300 ring-1 ring-rose-500/30">
                        &#x62f;&#x644;&#x6cc;&#x644; &#x627;&#x62e;&#x62a;&#x644;&#x627;&#x641;: {o.disputeReason}
                      </p>
                    )}

                    {/* Deal visual timeline: escrow ← delivery ← release */}
                    <EscrowTimeline status={o.status} />

                    <div className="mt-3 flex flex-wrap gap-2">
                      {canOpenDelivery && (
                        <button
                          type="button"
                          onClick={() => void openDelivery(o.id)}
                          disabled={busyId === o.id}
                          className="flex items-center gap-1.5 rounded-lg bg-gradient-to-b from-royal-400 to-royal-600 px-3 py-1.5 text-xs font-bold text-white ring-1 ring-gold-400/40 hover:opacity-90 disabled:opacity-40"
                        >
                          <Rocket className="size-3.5" aria-hidden />
                          &#x628;&#x627;&#x632; &#x6a9;&#x631;&#x62f;&#x646; / \u062f\u0627\u0646\u0644\u0648\u062f \u0645\u062d\u0635\u0648\u0644 (HTML)
                        </button>
                      )}

                      {iAmSeller && o.status === "FUNDED" && (
                        <button
                          type="button"
                          onClick={() => void act(o.id, "deliver")}
                          disabled={busyId === o.id}
                          className="rounded-lg bg-royal-500/20 px-3 py-1.5 text-xs font-bold text-gold-400 ring-1 ring-gold-400/30 hover:bg-royal-500/30 disabled:opacity-40"
                        >
                          &#x62b;&#x628;&#x62a; &#x62a;&#x62d;&#x648;&#x6cc;&#x644;
                        </button>
                      )}

                      {buyerCanSettle && (
                        <>
                          <button
                            type="button"
                            onClick={() => void act(o.id, "release")}
                            disabled={busyId === o.id}
                            className="flex items-center gap-1 rounded-lg bg-emerald-500/15 px-3 py-1.5 text-xs font-bold text-emerald-400 ring-1 ring-emerald-400/30 hover:bg-emerald-500/25 disabled:opacity-40"
                          >
                            <CheckCircle2 className="size-3.5" aria-hidden />
                            &#x62a;&#x627;&#x6cc;&#x6cc;&#x62f; &#x648; &#x622;&#x632;&#x627;&#x62f;&#x633;&#x627;&#x632;&#x6cc; &#x648;&#x62c;&#x647;
                          </button>
                          <button
                            type="button"
                            onClick={() => {
                              setDisputeEscrowId(disputingThis ? null : o.id);
                              setDisputeReason("");
                            }}
                            disabled={busyId === o.id}
                            className="flex items-center gap-1 rounded-lg bg-rose-500/15 px-3 py-1.5 text-xs font-bold text-rose-300 ring-1 ring-rose-500/30 hover:bg-rose-500/25 disabled:opacity-40"
                          >
                            <AlertTriangle className="size-3.5" aria-hidden />
                            &#x62b;&#x628;&#x62a; &#x627;&#x62e;&#x62a;&#x644;&#x627;&#x641;
                          </button>
                        </>
                      )}
                    </div>

                    {/* Inline dispute filing form */}
                    {disputingThis && (
                      <div className="mt-3 flex flex-col gap-2 rounded-xl border border-rose-500/30 bg-zinc-950/80 p-3">
                        <textarea
                          value={disputeReason}
                          onChange={(e) => setDisputeReason(e.target.value)}
                          rows={2}
                          maxLength={500}
                          placeholder="\u062f\u0644\u06cc\u0644 \u0627\u062e\u062a\u0644\u0627\u0641 \u0631\u0627 \u06a9\u0648\u062a\u0627\u0647 \u0628\u0646\u0648\u06cc\u0633 (\u0645\u062b\u0644\u0627\u064b \u0641\u0627\u06cc\u0644 \u062e\u0631\u0627\u0628 \u06cc\u0627 \u0646\u0627\u0642\u0635 \u0627\u0633\u062a)…"
                          className="w-full rounded-lg border border-zinc-700 bg-zinc-900 px-3 py-2 text-xs outline-none focus:border-rose-400/60"
                        />
                        <div className="flex justify-end gap-2">
                          <button
                            type="button"
                            onClick={() => setDisputeEscrowId(null)}
                            className="rounded-lg bg-zinc-800 px-3 py-1.5 text-xs text-zinc-300 hover:bg-zinc-700"
                          >
                            &#x627;&#x646;&#x635;&#x631;&#x627;&#x641;
                          </button>
                          <button
                            type="button"
                            onClick={() => void submitDispute(o.id)}
                            disabled={busyId === o.id}
                            className="rounded-lg bg-rose-600 px-3 py-1.5 text-xs font-bold text-white hover:bg-rose-500 disabled:opacity-50"
                          >
                            &#x627;&#x631;&#x633;&#x627;&#x644; &#x627;&#x62e;&#x62a;&#x644;&#x627;&#x641; &#x648; &#x642;&#x641;&#x644; &#x648;&#x62c;&#x647;
                          </button>
                        </div>
                      </div>
                    )}
                  </li>
                );
              })}
            </ul>
          )}
        </section>

        {/* My products */}
        {myProducts.length > 0 && (
          <section>
            <h2 className="mb-3 font-bold">&#x645;&#x62d;&#x635;&#x648;&#x644;&#x627;&#x62a; &#x641;&#x631;&#x648;&#x634;&#x6af;&#x627;&#x647; &#x645;&#x646;</h2>
            <ul className="flex flex-col gap-2">
              {myProducts.map((p) => (
                <li
                  key={p.id}
                  className="flex items-center justify-between gap-2 rounded-xl bg-zinc-950/60 px-3 py-2 ring-1 ring-zinc-800"
                >
                  <span className="truncate text-sm">{p.title}</span>
                  <div className="flex shrink-0 items-center gap-2">
                    <button
                      type="button"
                      onClick={() => void copyReferralLink(p)}
                      className="rounded-md bg-zinc-800 px-2 py-1 text-[11px] text-emerald-300 hover:bg-zinc-700"
                    >
                      &#x644;&#x6cc;&#x646;&#x6a9; &#x641;&#x631;&#x648;&#x634;
                    </button>
                    <span className="font-mono text-xs text-gold-400" dir="ltr">
                      {p.priceDin.toLocaleString("fa-IR")} Đ
                    </span>
                  </div>
                </li>
              ))}
            </ul>
          </section>
        )}
      </div>

      {/* Live product preview modal before purchase */}
      {previewProduct && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/80 p-3 backdrop-blur-sm"
          role="dialog"
          aria-modal="true"
          aria-label={previewProduct.title}
        >
          <div className="flex max-h-[92vh] w-full max-w-2xl flex-col gap-3 overflow-y-auto rounded-2xl border border-royal-500/35 bg-zinc-900 p-4 shadow-2xl">
            <div className="flex items-center justify-between gap-2">
              <div className="min-w-0">
                <h3 className="truncate text-base font-black">
                  {previewProduct.title}
                </h3>
                <p className="text-xs text-zinc-400">
                  {formatDin(previewProduct.priceDin)}
                  {previewProduct.affiliateBps > 0
                    ? ` • 🎁 ${formatAffiliatePercent(previewProduct.affiliateBps)} \u067e\u0627\u062f\u0627\u0634 \u0645\u0639\u0631\u0641\u06cc`
                    : ""}
                </p>
              </div>
              <button
                type="button"
                onClick={() => setPreviewProduct(null)}
                aria-label="\u0628\u0633\u062a\u0646 \u067e\u06cc\u0634‌\u0646\u0645\u0627\u06cc\u0634"
                className="grid size-8 shrink-0 place-items-center rounded-lg bg-zinc-800 text-zinc-300 hover:text-white"
              >
                <X className="size-4" aria-hidden />
              </button>
            </div>

            <HtmlPreviewFrame
              html={previewProduct.previewHtml}
              title={previewProduct.title}
              heightClassName="h-[460px]"
              badgeLabel="&#x67e;&#x6cc;&#x634;‌&#x646;&#x645;&#x627;&#x6cc;&#x634; &#x632;&#x646;&#x62f;&#x647; &#x67e;&#x6cc;&#x634; &#x627;&#x632; &#x62e;&#x631;&#x6cc;&#x62f;"
            />

            <div className="flex justify-end gap-2">
              <button
                type="button"
                onClick={() => void copyReferralLink(previewProduct)}
                className="rounded-xl bg-zinc-800 px-3.5 py-2 text-xs font-bold text-zinc-200 hover:bg-zinc-700"
              >
                &#x6a9;&#x67e;&#x6cc; &#x644;&#x6cc;&#x646;&#x6a9; &#x631;&#x641;&#x631;&#x627;&#x644;
              </button>
              <button
                type="button"
                onClick={() => void purchase(previewProduct)}
                disabled={
                  busyId === previewProduct.id ||
                  previewProduct.sellerId === me.user.id
                }
                className="rounded-xl bg-gradient-to-b from-royal-400 to-royal-600 px-4 py-2 text-xs font-black text-white ring-1 ring-gold-400/40 hover:opacity-90 disabled:opacity-40"
              >
                {previewProduct.sellerId === me.user.id
                  ? "\u0645\u062d\u0635\u0648\u0644 \u0645\u0646"
                  : "\u062e\u0631\u06cc\u062f \u0648 \u062f\u0631\u06cc\u0627\u0641\u062a \u0641\u0627\u06cc\u0644 \u06a9\u0627\u0645\u0644"}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Modal for delivering and running the purchased full HTML file */}
      {deliveryView && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/85 p-3 backdrop-blur-sm"
          role="dialog"
          aria-modal="true"
          aria-label={deliveryView.title}
        >
          <div className="flex max-h-[92vh] w-full max-w-3xl flex-col gap-3 overflow-y-auto rounded-2xl border border-emerald-500/35 bg-zinc-900 p-4 shadow-2xl">
            <div className="flex items-center justify-between gap-2">
              <div className="min-w-0">
                <h3 className="truncate text-base font-black text-emerald-300">
                  🚀 {deliveryView.title}
                </h3>
                <p className="text-xs text-zinc-400">
                  &#x646;&#x633;&#x62e;&#x647;&#x654; &#x6a9;&#x627;&#x645;&#x644; &#x645;&#x62d;&#x635;&#x648;&#x644; &#x62a;&#x62d;&#x648;&#x6cc;&#x644;&#x6cc; — &#x648;&#x636;&#x639;&#x6cc;&#x62a; &#x627;&#x633;&#x6a9;&#x631;&#x648;: {deliveryView.state}
                </p>
              </div>
              <button
                type="button"
                onClick={() => setDeliveryView(null)}
                aria-label="\u0628\u0633\u062a\u0646 \u062a\u062d\u0648\u06cc\u0644"
                className="grid size-8 shrink-0 place-items-center rounded-lg bg-zinc-800 text-zinc-300 hover:text-white"
              >
                <X className="size-4" aria-hidden />
              </button>
            </div>

            <HtmlPreviewFrame
              html={deliveryView.deliveryPayload}
              title={deliveryView.title}
              allowDownload
              downloadFilename={`${deliveryView.title || deliveryView.productId}.html`}
              heightClassName="h-[520px]"
              badgeLabel="&#x641;&#x627;&#x6cc;&#x644; &#x6a9;&#x627;&#x645;&#x644; &#x62a;&#x62d;&#x648;&#x6cc;&#x644;&#x6cc; (&#x633;&#x646;&#x62f;&#x628;&#x627;&#x6a9;&#x633; &#x627;&#x645;&#x646;)"
            />
          </div>
        </div>
      )}
    </MiniShell>
  );
}
