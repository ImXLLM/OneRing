"use client";

/**
 * KIAN M3 — the mini-app market page.
 *
 * Active products → atomic purchase (mint + escrow funding in one kernel command; article 8),
 * product registration for sellers, and "My orders" with the parties' actions:
 * delivery only from the seller side, release only from the buyer side (403 otherwise).
 */

import { useCallback, useEffect, useState } from "react";
import { Badge, toast } from "@kian/ui";
import { HandCoins, Loader2, PackagePlus, ShoppingBasket, Truck } from "lucide-react";
import { useSession } from "@/components/kian/session";
import { ApiError, api, type EscrowView, type ProductView } from "@/lib/api";
import { ESCROW_STATUS, formatDin, formatWhen } from "@/lib/format";

function StatusBadge({ status }: { status: string }) {
  const st = ESCROW_STATUS[status] ?? {
    fa: status,
    className: "bg-zinc-500/15 text-zinc-400 ring-zinc-400/30",
  };
  return <Badge variant="default" className={`ring-1 ${st.className}`}>{st.fa}</Badge>;
}

function GuestGate() {
  return (
    <div className="flex flex-col items-center gap-4 rounded-2xl border border-royal-500/20 bg-zinc-900/60 p-10 text-center">
      <ShoppingBasket className="size-10 text-gold-400" aria-hidden />
      <h1 className="text-xl font-black">MAIRON Market</h1>
      <p className="max-w-md text-sm leading-6 text-zinc-400">
        Buying and selling require signing in from inside Telegram — atomic purchases with the
        Article 8 escrow need a verified session.
      </p>
    </div>
  );
}

export default function MarketPage() {
  const { state, refresh } = useSession();
  const [products, setProducts] = useState<ProductView[]>([]);
  const [orders, setOrders] = useState<EscrowView[]>([]);
  const [myProducts, setMyProducts] = useState<ProductView[]>([]);
  const [busyId, setBusyId] = useState<string | null>(null);
  const [showCreate, setShowCreate] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [price, setPrice] = useState("");
  const [category, setCategory] = useState("");

  const loadMarket = useCallback(async (token: string | null) => {
    try {
      const res = await api.listProducts();
      setProducts(res.items);
    } catch {
      /* Public list — silently */
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
      <div className="flex items-center justify-center gap-3 py-24 text-zinc-500">
        <Loader2 className="size-5 animate-spin" aria-hidden />
        <span className="text-sm">Establishing the kernel session…</span>
      </div>
    );
  }

  if (state.phase !== "ready") return <GuestGate />;

  const { me, token } = state;

  const purchase = async (p: ProductView) => {
    if (!window.confirm(`Buy "${p.title}" for ${formatDin(p.priceDin)}?\nThe amount locks in escrow immediately.`)) return;
    setBusyId(p.id);
    try {
      const res = await api.purchase(token, p.id);
      toast({
        title: res.replayed ? "This purchase was already recorded (replay)" : "Purchase complete — the amount is locked in escrow",
        description: "After the seller delivers, the \"Release\" action pays them out.",
        variant: "success",
      });
      await Promise.all([loadMarket(token), refresh()]);
    } catch (err) {
      toast({
        title: "Purchase failed",
        description: err instanceof ApiError ? `${err.code}: ${err.message}` : "network error",
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
        toast({ title: "Delivery recorded", description: "The buyer can now release the escrow.", variant: "success" });
      } else {
        await api.release(token, escrowId);
        toast({ title: "Escrow released", description: "The 10 percent Article 8 tax was withheld by the kernel.", variant: "success" });
      }
      await Promise.all([loadMarket(token), refresh()]);
    } catch (err) {
      toast({
        title: kind === "deliver" ? "Delivery failed" : "Release failed",
        description: err instanceof ApiError ? `${err.code}: ${err.message}` : "network error",
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
        ...(category.trim() ? { category: category.trim() } : {}),
      });
      toast({ title: "Product published and live", variant: "success" });
      setTitle("");
      setDescription("");
      setPrice("");
      setCategory("");
      setShowCreate(false);
      await loadMarket(token);
    } catch (err) {
      toast({
        title: "Product submission failed",
        description: err instanceof ApiError ? `${err.code}: ${err.message}` : "network error",
        variant: "destructive",
      });
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="flex flex-col gap-6">
      <section className="flex items-center justify-between">
        <h1 className="text-lg font-black">
          Market
          <span className="mr-2 text-sm font-medium text-zinc-500">buy with the atomic Article 8 escrow</span>
        </h1>
        <button
          type="button"
          onClick={() => setShowCreate((s) => !s)}
          className="flex items-center gap-1.5 rounded-xl bg-royal-500/20 px-3 py-2 text-xs font-semibold text-gold-400 ring-1 ring-gold-400/30 hover:bg-royal-500/30"
        >
          <PackagePlus className="size-4" aria-hidden />
          Become a seller
        </button>
      </section>

      {showCreate && (
        <form
          onSubmit={(e) => void createProduct(e)}
          className="grid gap-3 rounded-2xl border border-royal-500/20 bg-zinc-900/60 p-5 sm:grid-cols-2"
        >
          <label className="text-xs text-zinc-400 sm:col-span-2">
            Product title (1 to 120 characters)
            <input
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              required
              minLength={1}
              maxLength={120}
              className="mt-1 w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm outline-none focus:border-gold-400/50"
            />
          </label>
          <label className="text-xs text-zinc-400 sm:col-span-2">
            Description (1 to 2000 characters)
            <textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              required
              minLength={1}
              maxLength={2000}
              rows={3}
              className="mt-1 w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm outline-none focus:border-gold-400/50"
            />
          </label>
          <label className="text-xs text-zinc-400">
            Price (Đ — integer)
            <input
              value={price}
              onChange={(e) => setPrice(e.target.value)}
              required
              inputMode="numeric"
              dir="ltr"
              aria-label="Price in Din (integer)"
              className="mt-1 w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 font-mono text-sm outline-none focus:border-gold-400/50"
            />
          </label>
          <label className="text-xs text-zinc-400">
            Category (optional, up to 40 characters)
            <input
              value={category}
              onChange={(e) => setCategory(e.target.value)}
              maxLength={40}
              className="mt-1 w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm outline-none focus:border-gold-400/50"
            />
          </label>
          <button
            type="submit"
            disabled={submitting}
            className="sm:col-span-2 rounded-xl bg-gradient-to-b from-royal-400 to-royal-600 px-4 py-2.5 text-sm font-bold text-white shadow-lg shadow-royal-600/30 ring-1 ring-gold-400/40 hover:opacity-90 disabled:opacity-50"
          >
            {submitting ? "submitting…" : "Publish product to market"}
          </button>
        </form>
      )}

      {/* Products */}
      <section>
        <h2 className="mb-3 text-sm font-bold text-zinc-300">Active products</h2>
        {products.length === 0 ? (
          <p className="rounded-2xl border border-zinc-800 bg-zinc-900/60 p-8 text-center text-sm text-zinc-600">
            No products yet — be the first seller.
          </p>
        ) : (
          <ul className="grid gap-3 sm:grid-cols-2">
            {products.map((p) => {
              const mine = p.sellerId === me.user.accountId;
              return (
                <li
                  key={p.id}
                  className="flex flex-col gap-2 rounded-2xl border border-zinc-800 bg-zinc-900/60 p-4"
                >
                  <div className="flex items-start justify-between gap-2">
                    <h3 className="font-bold leading-6">{p.title}</h3>
                    {p.category && (
                      <span className="shrink-0 rounded-md bg-royal-500/15 px-2 py-0.5 text-xs text-royal-400">
                        {p.category}
                      </span>
                    )}
                  </div>
                  <p className="line-clamp-2 text-xs leading-5 text-zinc-400">{p.description}</p>
                  <div className="mt-1 flex items-center justify-between">
                    <span className="font-mono font-bold text-gold-400">{formatDin(p.priceDin)}</span>
                    <button
                      type="button"
                      onClick={() => void purchase(p)}
                      disabled={mine || busyId === p.id}
                      className="flex items-center gap-1.5 rounded-lg bg-gradient-to-b from-royal-400 to-royal-600 px-3 py-1.5 text-xs font-bold text-white ring-1 ring-gold-400/40 hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-40"
                    >
                      {busyId === p.id ? (
                        <Loader2 className="size-3.5 animate-spin" aria-hidden />
                      ) : (
                        <HandCoins className="size-3.5" aria-hidden />
                      )}
                      {mine ? "your product" : "escrow-safe buy"}
                    </button>
                  </div>
                  <span className="text-[11px] text-zinc-600">
                    seller: <span className="font-mono" dir="ltr">{p.sellerId}</span> · {formatWhen(p.createdAt)}
                  </span>
                </li>
              );
            })}
          </ul>
        )}
      </section>

      {/* My orders */}
      <section className="rounded-2xl border border-zinc-800 bg-zinc-900/60 p-5">
        <h2 className="mb-3 text-sm font-bold text-zinc-300">My orders</h2>
        {orders.length === 0 ? (
          <p className="py-4 text-center text-sm text-zinc-600">You have no orders.</p>
        ) : (
          <ul className="flex flex-col gap-3">
            {orders.map((o) => {
              const isSeller = o.sellerId === me.user.accountId;
              const isBuyer = o.buyerId === me.user.accountId;
              const canDeliver = isSeller && o.status === "FUNDED";
              const canRelease = isBuyer && o.status === "DELIVERED";
              return (
                <li
                  key={o.id}
                  className="flex flex-wrap items-center gap-x-3 gap-y-2 rounded-xl bg-zinc-950/60 p-3 ring-1 ring-zinc-800"
                >
                  <span className="font-mono text-sm font-bold text-gold-400">{formatDin(o.amount)}</span>
                  <StatusBadge status={o.status} />
                  <span className="text-xs text-zinc-400">
                    {isSeller ? "seller" : "buyer"} · {isSeller ? `buyer: ${o.buyerId}` : `seller: ${o.sellerId}`}
                  </span>
                  {o.productRef && (
                    <span className="font-mono text-xs text-zinc-600" dir="ltr">
                      {o.productRef.slice(0, 12)}…
                    </span>
                  )}
                  <span className="mr-auto flex items-center gap-2">
                    {canDeliver && (
                      <button
                        type="button"
                        onClick={() => void act(o.id, "deliver")}
                        disabled={busyId === o.id}
                        className="flex items-center gap-1.5 rounded-lg bg-royal-500/20 px-3 py-1.5 text-xs font-semibold text-gold-400 ring-1 ring-gold-400/30 hover:bg-royal-500/30 disabled:opacity-50"
                      >
                        <Truck className="size-3.5" aria-hidden />
                        I delivered it
                      </button>
                    )}
                    {canRelease && (
                      <button
                        type="button"
                        onClick={() => void act(o.id, "release")}
                        disabled={busyId === o.id}
                        className="rounded-lg bg-emerald-500/20 px-3 py-1.5 text-xs font-semibold text-emerald-400 ring-1 ring-emerald-400/30 hover:bg-emerald-500/30 disabled:opacity-50"
                      >
                        Confirm and release
                      </button>
                    )}
                  </span>
                </li>
              );
            })}
          </ul>
        )}
      </section>

      {/* My products */}
      {myProducts.length > 0 && (
        <section className="rounded-2xl border border-zinc-800 bg-zinc-900/60 p-5">
          <h2 className="mb-3 text-sm font-bold text-zinc-300">My products</h2>
          <ul className="flex flex-col gap-2">
            {myProducts.map((p) => (
              <li key={p.id} className="flex items-center gap-3 rounded-lg bg-zinc-950/60 px-3 py-2 ring-1 ring-zinc-800">
                <span className="truncate text-sm">{p.title}</span>
                <span className="mr-auto font-mono text-sm text-gold-400">{formatDin(p.priceDin)}</span>
              </li>
            ))}
          </ul>
        </section>
      )}
    </div>
  );
}
