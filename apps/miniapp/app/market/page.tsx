"use client";

/**
 * KIAN miniapp — بازار: پیش‌نمایش امن HTML، لینک‌های وایرال رفرال، خرید اتمی
 * با ثبت خودکار معرف، اجرای/دانلود آنی محصول پس از خرید و ثبت اختلاف خریدار.
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

/** تایم‌لاین بصری سه‌مرحله‌ای معاملهٔ امانی — قابل‌فهم برای هر کاربری. */
function EscrowTimeline({ status }: { status: string }) {
  // مرحلهٔ فعلی: ۱ تأمین، ۲ تحویل، ۳ آزادسازی؛ ۰ یعنی هنوز تأمین نشده
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
    { n: 1, label: "پرداخت در امانت 🔒" },
    { n: 2, label: "فایل HTML آمادهٔ اجرا 🚀" },
    { n: 3, label: "تأیید و آزادسازی وجه ✅" },
  ];
  const closedNote: Record<string, string> = {
    DISPUTED: "⚠️ معامله در داوری است — وجه تا حل اختلاف در امانت قفل می‌ماند.",
    REFUNDED: "↩️ وجه به خریدار برگشت خورد و معامله بسته شد.",
    CANCELLED: "این معامله لغو شد.",
    EXPIRED: "مهلت این معامله تمام شد.",
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

  // شیر اعتبار آزمایشی — برای پرسوناهای دمو داخل بنرِ موجودی ناکافی
  const [claimingFaucet, setClaimingFaucet] = useState(false);

  // رفرال فعال + محصول هدفِ لینک مستقیم وایرال
  const [activeReferrer, setActiveReferrer] = useState<string | null>(null);
  const [spotlightProduct, setSpotlightProduct] = useState<ProductView | null>(null);
  const [previewProduct, setPreviewProduct] = useState<ProductView | null>(null);

  // تحویل HTML خریداری‌شده و ثبت اختلاف خریدار
  const [deliveryView, setDeliveryView] = useState<EscrowDeliveryView | null>(null);
  const [disputeEscrowId, setDisputeEscrowId] = useState<string | null>(null);
  const [disputeReason, setDisputeReason] = useState("");

  // بنر دوستانهٔ موجودی ناکافی — با میان‌بر شارژ کیف و درآمد رفرال
  const [needFunds, setNeedFunds] = useState<{ product: ProductView } | null>(null);

  // فرم ثبت دستی محصول
  const [showCreate, setShowCreate] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [price, setPrice] = useState("");
  const [category, setCategory] = useState("");
  const [affiliateBps, setAffiliateBps] = useState("1000");

  // خواندن لینک وایرال ورودی و بارگیری تک‌محصول هدف (حتی قبل از نشست)
  useEffect(() => {
    const inbound = captureInboundViralLink();
    setActiveReferrer(inbound.referrer ?? getActiveReferrer());
    if (inbound.productId) {
      api
        .getProduct(inbound.productId)
        .then((p) => setSpotlightProduct(p))
        .catch(() => {
          /* محصول ناموجود — لیست عادی نمایش داده می‌شود */
        });
    }
  }, []);

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
      <MiniShell>
        <div className="flex items-center justify-center gap-3 py-24 text-zinc-500">
          <Loader2 className="size-5 animate-spin" aria-hidden />
          <span className="text-sm">برقراری نشست با کرنل…</span>
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
                    پیش‌نمایش لینک مستقیم
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
                  badgeLabel="پیش‌نمایش عمومی"
                />
              )}
            </section>
          )}
          {preview ? (
            <p className="rounded-2xl border border-zinc-800 bg-zinc-900/60 px-4 py-3 text-center text-xs leading-6 text-zinc-400">
              در حالت مرورگر فقط تماشا فعال است؛ برای خرید و فروش از داخل تلگرام
              وارد شو — دکمهٔ «تلاش برای ورود واقعی» بالای همین صفحه است.
            </p>
          ) : (
            <GuestGate
              title="بازار KIAN با ورود از تلگرام باز می‌شود"
              hint="خرید اتمی با اسکرو مادهٔ ۸ نیازمند نشست احرازشدهٔ توست — اما می‌توانی همین حالا با «ورود آزمایشی» محصولات و پیش‌نمایش‌ها را ببینی."
            />
          )}
        </div>
      </MiniShell>
    );
  }

  const { me, token } = state;

  // شیر اعتبار آزمایشی — فقط برای پرسوناهای دمو (در تولید سرور صادقانه ۴۰۳ می‌دهد)
  const claimFaucet = async () => {
    setClaimingFaucet(true);
    try {
      const res = await api.claimDemoFaucet(token);
      toast({
        title: "🎁 اعتبار تست اضافه شد",
        description: `${formatDin(res.balance)} در کیف‌پول تو آمادهٔ خرید است.`,
        variant: "success",
      });
      setNeedFunds(null);
      await refresh();
    } catch (err) {
      toast({
        title: "دریافت اعتبار تست ناموفق",
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
        ? `\nپاداش معرفی (${formatAffiliatePercent(p.affiliateBps)}) به معرف ثبت می‌شود.`
        : "";
    if (
      !window.confirm(
        `خرید «${p.title}» به قیمت ${formatDin(p.priceDin)}؟\nمبلغ بلافاصله در اسکرو قفل می‌شود.${refNote}`,
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
          ? "این خرید قبلاً ثبت شده بود (replay)"
          : "خرید انجام شد — محصول آمادهٔ مشاهده و تحویل است",
        description:
          "از بخش «سفارش‌های من» فایل HTML را باز کن و پس از بررسی، آزادسازی یا اختلاف ثبت کن.",
        variant: "success",
      });
      setPreviewProduct(null);
      setNeedFunds(null);
      await Promise.all([loadMarket(token), refresh()]);
    } catch (err) {
      // تجربهٔ خرید بدون اصطکاک: موجودی ناکافی → بنر دوستانه + دو میان‌بر، نه خطای خام
      if (
        err instanceof ApiError &&
        (err.code === "ACCOUNT_OVERDRAFT" || err.code === "INSUFFICIENT_BALANCE")
      ) {
        setNeedFunds({ product: p });
        toast({
          title: "موجودی کیف‌پول کافی نیست",
          description: "از میان‌برهای پایین بازار: شارژ کیف یا درآمد رایگان رفرال.",
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
        title: "لینک رفرال شما کپی شد!",
        description: `با هر خرید از لینک شما، ${formatAffiliatePercent(p.affiliateBps)} پورسانت به کیف شما واریز می‌شود.`,
        variant: "success",
      });
    } catch {
      toast({
        title: "کپی خودکار مجاز نبود",
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
        title: "دریافت محتوای محصول ناموفق بود",
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
          title: "تحویل ثبت شد",
          description: "حالا خریدار می‌تواند آزادسازی کند.",
          variant: "success",
        });
      } else {
        await api.release(token, escrowId);
        toast({
          title: "آزادسازی و تسویه انجام شد",
          description: "تقسیم خودکار بین فروشنده، خزانه و معرف (در صورت وجود) ثبت شد.",
          variant: "success",
        });
      }
      await Promise.all([loadMarket(token), refresh()]);
    } catch (err) {
      toast({
        title: kind === "deliver" ? "تحویل ناموفق" : "آزادسازی ناموفق",
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
      toast({ title: "دلیل اختلاف را بنویسید", variant: "destructive" });
      return;
    }
    setBusyId(escrowId);
    try {
      await api.disputeEscrow(token, escrowId, clean);
      toast({
        title: "اختلاف ثبت شد — وجه در اسکرو قفل ماند",
        description: "پرونده تا داوری بررسی می‌شود.",
        variant: "success",
      });
      setDisputeEscrowId(null);
      setDisputeReason("");
      await Promise.all([loadMarket(token), refresh()]);
    } catch (err) {
      toast({
        title: "ثبت اختلاف ناموفق بود",
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
      toast({ title: "محصول ثبت شد و فعال است", variant: "success" });
      setTitle("");
      setDescription("");
      setPrice("");
      setCategory("");
      setAffiliateBps("1000");
      setShowCreate(false);
      await loadMarket(token);
    } catch (err) {
      toast({
        title: "ثبت محصول ناموفق",
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
        {/* سرصفحه + میان‌بر استودیو AI و فروش دستی */}
        <div className="flex flex-wrap items-center justify-between gap-2">
          <h1 className="text-lg font-black">بازار</h1>
          <div className="flex items-center gap-2">
            <Link
              href="/builder"
              className="flex items-center gap-1.5 rounded-xl bg-gradient-to-b from-royal-400 to-royal-600 px-3 py-2 text-xs font-bold text-white ring-1 ring-gold-400/40 hover:opacity-90"
            >
              <Sparkles className="size-4" aria-hidden />
              ساخت با AI
            </Link>
            <button
              type="button"
              onClick={() => setShowCreate((v) => !v)}
              className="flex items-center gap-1.5 rounded-xl bg-royal-500/20 px-3 py-2 text-xs font-bold text-gold-400 ring-1 ring-gold-400/30 hover:bg-royal-500/30"
            >
              <PackagePlus className="size-4" aria-hidden />
              فروش دستی
            </button>
          </div>
        </div>

        {/* نشانگر معرف فعالِ ذخیره‌شده از لینک وایرال */}
        {activeReferrer && activeReferrer !== me.user.telegramId && (
          <div className="flex items-center gap-2 rounded-xl border border-emerald-500/30 bg-emerald-950/25 px-3 py-2 text-xs text-emerald-300">
            <Gift className="size-4 shrink-0" aria-hidden />
            <span>
              لینک معرفی فعال است (شناسهٔ معرف:{" "}
              <strong className="font-mono" dir="ltr">
                #{activeReferrer}
              </strong>
              ) — هنگام خرید به‌طور خودکار ثبت می‌شود.
            </span>
          </div>
        )}

        {/* بنر دوستانهٔ موجودی ناکافی — با دو میان‌بر مستقیم */}
        {needFunds && (
          <div className="flex flex-col gap-2.5 rounded-2xl border border-amber-500/40 bg-amber-950/25 p-4">
            <p className="text-sm font-black text-amber-300">
              💳 موجودی کیف‌پول برای این خرید کافی نیست
            </p>
            <p className="text-xs leading-6 text-zinc-300">
              برای خرید «{needFunds.product.title}» به{" "}
              <strong className="font-mono text-gold-400" dir="ltr">
                {formatDin(needFunds.product.priceDin)}
              </strong>{" "}
              نیاز داری؛ موجودی فعلی تو{" "}
              <strong className="font-mono text-zinc-100" dir="ltr">
                {formatDin(me.balance)}
              </strong>{" "}
              است. دو راه سریع داری:
            </p>
            <div className="flex flex-wrap gap-2">
              {activePersona && (
                <button
                  type="button"
                  disabled={claimingFaucet}
                  onClick={() => void claimFaucet()}
                  className="flex items-center gap-1.5 rounded-xl bg-gradient-to-b from-emerald-500 to-emerald-600 px-3.5 py-2.5 text-xs font-black text-white shadow-lg shadow-emerald-600/25 ring-1 ring-emerald-400/50 transition hover:opacity-90 disabled:opacity-50"
                >
                  {claimingFaucet ? "در حال دریافت…" : "🎁 دریافت ۵٬۰۰۰ KIAN اعتبار تست رایگان (Faucet)"}
                </button>
              )}
              <Link
                href="/wallet"
                onClick={() => setNeedFunds(null)}
                className="flex items-center gap-1.5 rounded-xl bg-gradient-to-b from-gold-400 to-gold-600 px-3.5 py-2.5 text-xs font-black text-zinc-950 shadow-lg shadow-gold-500/20 ring-1 ring-gold-400/50 hover:opacity-90"
              >
                ➕ شارژ کیف‌پول
              </Link>
              <button
                type="button"
                onClick={() => void copyReferralLink(needFunds.product)}
                className="flex items-center gap-1.5 rounded-xl bg-emerald-500/15 px-3.5 py-2.5 text-xs font-bold text-emerald-300 ring-1 ring-emerald-400/40 hover:bg-emerald-500/25"
              >
                🔗 کسب درآمد رایگان با کپیِ لینک رفرال محصولات
              </button>
              <button
                type="button"
                onClick={() => setNeedFunds(null)}
                className="rounded-xl bg-zinc-800 px-3 py-2.5 text-xs text-zinc-400 hover:bg-zinc-700"
              >
                بستن
              </button>
            </div>
            <p className="text-[11px] leading-5 text-zinc-500">
              راهنما: با هر خریدِ دیگران از لینک رفرال تو، پورسانت آنی واریز می‌شود؛
              یا با ثبت اثبات واریز TON، دیین ضرب می‌شود.
            </p>
          </div>
        )}

        {/* کارت ویژهٔ محصول لینک مستقیم (`startapp=p_<id>_r_<ref>`) */}
        {spotlightProduct && (
          <section className="flex flex-col gap-3 rounded-2xl border border-gold-400/40 bg-zinc-900/80 p-4 shadow-xl shadow-black/40">
            <div className="flex items-start justify-between gap-3">
              <div className="min-w-0">
                <span className="inline-flex items-center gap-1 rounded-full bg-gold-400/15 px-2.5 py-0.5 text-[11px] font-bold text-gold-400">
                  لینک مستقیم محصول
                </span>
                <h2 className="mt-1 text-base font-black">{spotlightProduct.title}</h2>
                <p className="mt-1 text-xs leading-5 text-zinc-300">
                  {spotlightProduct.description}
                </p>
              </div>
              <button
                type="button"
                onClick={() => setSpotlightProduct(null)}
                aria-label="بستن کارت محصول"
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
                badgeLabel="پیش‌نمایش زنده پیش از خرید"
              />
            )}

            <div className="flex flex-wrap items-center justify-between gap-2 pt-1">
              <div className="flex items-center gap-2">
                <span className="font-mono text-base font-black text-gold-400" dir="ltr">
                  {spotlightProduct.priceDin.toLocaleString("fa-IR")} Đ
                </span>
                {spotlightProduct.affiliateBps > 0 && (
                  <span className="rounded-full bg-emerald-500/15 px-2.5 py-0.5 text-[11px] font-bold text-emerald-300 ring-1 ring-emerald-500/30">
                    🎁 {formatAffiliatePercent(spotlightProduct.affiliateBps)} پاداش معرفی
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
                  کپی لینک رفرال
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
                    ? "محصول من"
                    : "خرید آنی با اسکرو"}
                </button>
              </div>
            </div>
          </section>
        )}

        {/* فرم ثبت دستی محصول */}
        {showCreate && (
          <form
            onSubmit={(e) => void createProduct(e)}
            className="grid gap-3 rounded-2xl border border-royal-500/20 bg-zinc-900/60 p-5"
          >
            <label className="text-xs text-zinc-400">
              عنوان
              <input
                value={title}
                onChange={(e) => setTitle(e.target.value)}
                required
                minLength={3}
                maxLength={120}
                placeholder="مثلاً: کانال VIP سیگنال"
                className="mt-1 w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm outline-none focus:border-gold-400/50"
              />
            </label>
            <label className="text-xs text-zinc-400">
              توضیحات
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
                قیمت (Đ)
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
                پورسانت معرف
                <select
                  value={affiliateBps}
                  onChange={(e) => setAffiliateBps(e.target.value)}
                  className="mt-1 w-full rounded-lg border border-zinc-700 bg-zinc-950 px-2 py-2 text-sm outline-none focus:border-gold-400/50"
                >
                  <option value="0">۰٪</option>
                  <option value="1000">۱۰٪</option>
                  <option value="1500">۱۵٪</option>
                  <option value="2000">۲۰٪</option>
                  <option value="3000">۳۰٪</option>
                  <option value="5000">۵۰٪</option>
                </select>
              </label>
              <label className="text-xs text-zinc-400">
                دسته
                <input
                  value={category}
                  onChange={(e) => setCategory(e.target.value)}
                  maxLength={40}
                  placeholder="خدمات"
                  className="mt-1 w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm outline-none focus:border-gold-400/50"
                />
              </label>
            </div>
            <button
              type="submit"
              disabled={submitting}
              className="rounded-xl bg-gradient-to-b from-royal-400 to-royal-600 px-4 py-2.5 text-sm font-bold text-white shadow-lg shadow-royal-600/30 ring-1 ring-gold-400/40 hover:opacity-90 disabled:opacity-50"
            >
              {submitting ? "در حال ثبت…" : "ثبت محصول"}
            </button>
          </form>
        )}

        {/* محصولات فعال */}
        <section>
          <h2 className="mb-3 flex items-center gap-2 font-bold">
            <HandCoins className="size-4 text-gold-400" aria-hidden />
            محصولات فعال
          </h2>
          {products.length === 0 ? (
            <p className="rounded-2xl border border-zinc-800 bg-zinc-900/60 py-8 text-center text-sm text-zinc-600">
              هنوز محصولی فعال نیست — اولین محصول تعاملی را در استودیو AI بساز.
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
                            🎁 {formatAffiliatePercent(p.affiliateBps)} پاداش معرفی
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
                            ? "محصول من"
                            : "خرید"}
                      </button>
                    </div>
                  </div>

                  {/* دکمه‌های پیش‌نمایش زنده و کپی لینک رفرال */}
                  <div className="mt-3 flex flex-wrap items-center gap-2 border-t border-zinc-800/80 pt-3">
                    {p.previewHtml && (
                      <button
                        type="button"
                        onClick={() => setPreviewProduct(p)}
                        className="flex items-center gap-1.5 rounded-lg bg-royal-500/20 px-2.5 py-1.5 text-[11px] font-bold text-gold-300 ring-1 ring-gold-400/30 hover:bg-royal-500/30"
                      >
                        <Eye className="size-3.5" aria-hidden />
                        پیش‌نمایش زنده
                      </button>
                    )}
                    <button
                      type="button"
                      onClick={() => void copyReferralLink(p)}
                      className="flex items-center gap-1.5 rounded-lg bg-zinc-800/90 px-2.5 py-1.5 text-[11px] font-semibold text-zinc-200 ring-1 ring-zinc-700 hover:bg-zinc-700"
                    >
                      <Link2 className="size-3.5 text-emerald-400" aria-hidden />
                      کپی لینک رفرال من
                    </button>
                  </div>
                </li>
              ))}
            </ul>
          )}
        </section>

        {/* سفارش‌های من (اسکروها + تحویل آنی HTML + آزادسازی + ثبت اختلاف) */}
        <section>
          <h2 className="mb-3 flex items-center gap-2 font-bold">
            <Truck className="size-4 text-royal-400" aria-hidden />
            سفارش‌های من (اسکروها)
          </h2>
          {orders.length === 0 ? (
            <p className="rounded-2xl border border-zinc-800 bg-zinc-900/60 py-8 text-center text-sm text-zinc-600">
              سفارشی ندارید.
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
                        {iAmSeller ? "فروشندهٔ این معامله" : "خریدار این معامله"}
                      </span>
                      <span className="mr-auto text-[11px] text-zinc-600">
                        {formatWhen(o.createdAt)}
                      </span>
                    </div>

                    {o.disputeReason && (
                      <p className="mt-2 rounded-lg bg-rose-950/40 px-2.5 py-1.5 text-xs text-rose-300 ring-1 ring-rose-500/30">
                        دلیل اختلاف: {o.disputeReason}
                      </p>
                    )}

                    {/* تایم‌لاین بصری معامله: امانت ← تحویل ← آزادسازی */}
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
                          باز کردن / دانلود محصول (HTML)
                        </button>
                      )}

                      {iAmSeller && o.status === "FUNDED" && (
                        <button
                          type="button"
                          onClick={() => void act(o.id, "deliver")}
                          disabled={busyId === o.id}
                          className="rounded-lg bg-royal-500/20 px-3 py-1.5 text-xs font-bold text-gold-400 ring-1 ring-gold-400/30 hover:bg-royal-500/30 disabled:opacity-40"
                        >
                          ثبت تحویل
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
                            تایید و آزادسازی وجه
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
                            ثبت اختلاف
                          </button>
                        </>
                      )}
                    </div>

                    {/* فرم درون‌خطی ثبت اختلاف */}
                    {disputingThis && (
                      <div className="mt-3 flex flex-col gap-2 rounded-xl border border-rose-500/30 bg-zinc-950/80 p-3">
                        <textarea
                          value={disputeReason}
                          onChange={(e) => setDisputeReason(e.target.value)}
                          rows={2}
                          maxLength={500}
                          placeholder="دلیل اختلاف را کوتاه بنویس (مثلاً فایل خراب یا ناقص است)…"
                          className="w-full rounded-lg border border-zinc-700 bg-zinc-900 px-3 py-2 text-xs outline-none focus:border-rose-400/60"
                        />
                        <div className="flex justify-end gap-2">
                          <button
                            type="button"
                            onClick={() => setDisputeEscrowId(null)}
                            className="rounded-lg bg-zinc-800 px-3 py-1.5 text-xs text-zinc-300 hover:bg-zinc-700"
                          >
                            انصراف
                          </button>
                          <button
                            type="button"
                            onClick={() => void submitDispute(o.id)}
                            disabled={busyId === o.id}
                            className="rounded-lg bg-rose-600 px-3 py-1.5 text-xs font-bold text-white hover:bg-rose-500 disabled:opacity-50"
                          >
                            ارسال اختلاف و قفل وجه
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

        {/* محصولات من */}
        {myProducts.length > 0 && (
          <section>
            <h2 className="mb-3 font-bold">محصولات فروشگاه من</h2>
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
                      لینک فروش
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

      {/* مودال پیش‌نمایش زندهٔ محصول پیش از خرید */}
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
                    ? ` • 🎁 ${formatAffiliatePercent(previewProduct.affiliateBps)} پاداش معرفی`
                    : ""}
                </p>
              </div>
              <button
                type="button"
                onClick={() => setPreviewProduct(null)}
                aria-label="بستن پیش‌نمایش"
                className="grid size-8 shrink-0 place-items-center rounded-lg bg-zinc-800 text-zinc-300 hover:text-white"
              >
                <X className="size-4" aria-hidden />
              </button>
            </div>

            <HtmlPreviewFrame
              html={previewProduct.previewHtml}
              title={previewProduct.title}
              heightClassName="h-[460px]"
              badgeLabel="پیش‌نمایش زنده پیش از خرید"
            />

            <div className="flex justify-end gap-2">
              <button
                type="button"
                onClick={() => void copyReferralLink(previewProduct)}
                className="rounded-xl bg-zinc-800 px-3.5 py-2 text-xs font-bold text-zinc-200 hover:bg-zinc-700"
              >
                کپی لینک رفرال
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
                  ? "محصول من"
                  : "خرید و دریافت فایل کامل"}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* مودال تحویل و اجرای فایل کامل HTML خریداری‌شده */}
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
                  نسخهٔ کامل محصول تحویلی — وضعیت اسکرو: {deliveryView.state}
                </p>
              </div>
              <button
                type="button"
                onClick={() => setDeliveryView(null)}
                aria-label="بستن تحویل"
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
              badgeLabel="فایل کامل تحویلی (سندباکس امن)"
            />
          </div>
        </div>
      )}
    </MiniShell>
  );
}
