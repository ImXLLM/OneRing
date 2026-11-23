"use client";

/**
 * KIAN M3 — صفحهٔ بازار مینی‌اپ.
 *
 * محصولات فعال → خرید اتمی (ساخت + تأمین اسکرو در یک فرمان کرنل؛ مادهٔ ۸)،
 * ثبت محصول برای فروشندگان، و «سفارش‌های من» با کنش‌های طرفین:
 * تحویل فقط از سمت فروشنده، آزادسازی فقط از سمت خریدار (۴۰۳ در غیر این صورت).
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
      <h1 className="text-xl font-black">بازار KIAN</h1>
      <p className="max-w-md text-sm leading-6 text-zinc-400">
        برای خرید و فروش باید از داخل تلگرام وارد شوید — خرید اتمی با اسکرو
        مادهٔ ۸ نیازمند نشست احرازشده است.
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
      /* لیست عمومی — بی‌سروصدا */
    }
    if (token) {
      try {
        const mine = await api.myMarket(token);
        setOrders(mine.orders);
        setMyProducts(mine.products);
      } catch {
        /* نشست ممکن است تازه مرده باشد */
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
        <span className="text-sm">برقراری نشست با کرنل…</span>
      </div>
    );
  }

  if (state.phase !== "ready") return <GuestGate />;

  const { me, token } = state;

  const purchase = async (p: ProductView) => {
    if (!window.confirm(`خرید «${p.title}» به قیمت ${formatDin(p.priceDin)}؟\nمبلغ بلافاصله در اسکرو قفل می‌شود.`)) return;
    setBusyId(p.id);
    try {
      const res = await api.purchase(token, p.id);
      toast({
        title: res.replayed ? "این خرید قبلاً ثبت شده بود (replay)" : "خرید انجام شد — مبلغ در اسکرو قفل شد",
        description: "پس از تحویل فروشنده، با «آزادسازی» پول به او می‌رسد.",
        variant: "success",
      });
      await Promise.all([loadMarket(token), refresh()]);
    } catch (err) {
      toast({
        title: "خرید ناموفق",
        description: err instanceof ApiError ? `${err.code}: ${err.message}` : "خطای شبکه",
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
        toast({ title: "تحویل ثبت شد", description: "حالا خریدار می‌تواند آزادسازی کند.", variant: "success" });
      } else {
        await api.release(token, escrowId);
        toast({ title: "آزادسازی انجام شد", description: "۱۰٪ مالیات مادهٔ ۸ در کرنل کسر شد.", variant: "success" });
      }
      await Promise.all([loadMarket(token), refresh()]);
    } catch (err) {
      toast({
        title: kind === "deliver" ? "تحویل ناموفق" : "آزادسازی ناموفق",
        description: err instanceof ApiError ? `${err.code}: ${err.message}` : "خطای شبکه",
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
      toast({ title: "محصول ثبت شد و فعال است", variant: "success" });
      setTitle("");
      setDescription("");
      setPrice("");
      setCategory("");
      setShowCreate(false);
      await loadMarket(token);
    } catch (err) {
      toast({
        title: "ثبت محصول ناموفق",
        description: err instanceof ApiError ? `${err.code}: ${err.message}` : "خطای شبکه",
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
          بازار
          <span className="mr-2 text-sm font-medium text-zinc-500">خرید با اسکرو اتمی مادهٔ ۸</span>
        </h1>
        <button
          type="button"
          onClick={() => setShowCreate((s) => !s)}
          className="flex items-center gap-1.5 rounded-xl bg-royal-500/20 px-3 py-2 text-xs font-semibold text-gold-400 ring-1 ring-gold-400/30 hover:bg-royal-500/30"
        >
          <PackagePlus className="size-4" aria-hidden />
          فروشنده شوید
        </button>
      </section>

      {showCreate && (
        <form
          onSubmit={(e) => void createProduct(e)}
          className="grid gap-3 rounded-2xl border border-royal-500/20 bg-zinc-900/60 p-5 sm:grid-cols-2"
        >
          <label className="text-xs text-zinc-400 sm:col-span-2">
            عنوان محصول (۱ تا ۱۲۰ نویسه)
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
            توضیح (۱ تا ۲۰۰۰ نویسه)
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
            قیمت (Đ — عدد صحیح)
            <input
              value={price}
              onChange={(e) => setPrice(e.target.value)}
              required
              inputMode="numeric"
              dir="ltr"
              placeholder="500"
              className="mt-1 w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 font-mono text-sm outline-none focus:border-gold-400/50"
            />
          </label>
          <label className="text-xs text-zinc-400">
            دسته (اختیاری، تا ۴۰ نویسه)
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
            {submitting ? "در حال ثبت…" : "ثبت محصول در بازار"}
          </button>
        </form>
      )}

      {/* محصولات */}
      <section>
        <h2 className="mb-3 text-sm font-bold text-zinc-300">محصولات فعال</h2>
        {products.length === 0 ? (
          <p className="rounded-2xl border border-zinc-800 bg-zinc-900/60 p-8 text-center text-sm text-zinc-600">
            هنوز محصولی ثبت نشده — اولین فروشنده باشید.
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
                      {mine ? "محصول خودتان" : "خرید امن"}
                    </button>
                  </div>
                  <span className="text-[11px] text-zinc-600">
                    فروشنده: <span className="font-mono" dir="ltr">{p.sellerId}</span> · {formatWhen(p.createdAt)}
                  </span>
                </li>
              );
            })}
          </ul>
        )}
      </section>

      {/* سفارش‌های من */}
      <section className="rounded-2xl border border-zinc-800 bg-zinc-900/60 p-5">
        <h2 className="mb-3 text-sm font-bold text-zinc-300">سفارش‌های من</h2>
        {orders.length === 0 ? (
          <p className="py-4 text-center text-sm text-zinc-600">سفارشی ندارید.</p>
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
                    {isSeller ? "فروشنده" : "خریدار"} · {isSeller ? `خریدار: ${o.buyerId}` : `فروشنده: ${o.sellerId}`}
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
                        تحویل دادم
                      </button>
                    )}
                    {canRelease && (
                      <button
                        type="button"
                        onClick={() => void act(o.id, "release")}
                        disabled={busyId === o.id}
                        className="rounded-lg bg-emerald-500/20 px-3 py-1.5 text-xs font-semibold text-emerald-400 ring-1 ring-emerald-400/30 hover:bg-emerald-500/30 disabled:opacity-50"
                      >
                        تأیید و آزادسازی
                      </button>
                    )}
                  </span>
                </li>
              );
            })}
          </ul>
        )}
      </section>

      {/* محصولات من */}
      {myProducts.length > 0 && (
        <section className="rounded-2xl border border-zinc-800 bg-zinc-900/60 p-5">
          <h2 className="mb-3 text-sm font-bold text-zinc-300">محصولات من</h2>
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
