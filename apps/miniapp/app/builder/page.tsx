"use client";

/**
 * KIAN miniapp — استودیوی ساخت محصول HTML با هوش مصنوعی (فاز ۵).
 *
 * چرخهٔ کامل در یک صفحه با ناوبری سه‌مرحله‌ای شفاف:
 * ۱) گفتگو و ایده — چت چندنوبتی + ۴ چیپ ایدهٔ آمادهٔ یک‌لمسی
 * ۲) پیش‌نمایش زنده و امن داخل `HtmlPreviewFrame` (`sandbox="allow-scripts"`)
 * ۳) قیمت‌گذاری، پورسانت رفرال و انتشار یک‌کلیکه + لینک وایرال تلگرام
 */

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import Link from "next/link";
import { toast } from "@kian/ui";
import {
  Check,
  Copy,
  ExternalLink,
  Loader2,
  MessageCircle,
  Plus,
  Rocket,
  Send,
  Sparkles,
  Zap,
} from "lucide-react";
import { MiniShell } from "@/components/kian/mini-shell";
import { HtmlPreviewFrame } from "@/components/kian/html-preview-frame";
import { GuestGate, PreviewBanner } from "@/components/kian/guest-gate";
import { useSession } from "@/components/kian/session";
import { friendlyError, friendlyErrorTitle } from "@/lib/friendly";
import { sanitizePartialStudioHtml } from "@/lib/studio-html-guard";
import {
  api,
  ApiError,
  type BuilderSessionSummary,
  type BuilderSessionView,
  type PublishBuilderResponse,
} from "@kian/sdk";
import { buildTelegramViralLink, formatAffiliatePercent } from "@/lib/referral";

const AFFILIATE_PRESETS = [1000, 1500, 2000, 2500, 3000, 4000, 5000] as const;

/** ایده‌های آمادهٔ یک‌لمسی — کاربر هرگز مقابل چت خالی نمی‌ماند. */
const STARTER_PROMPTS: { emoji: string; label: string; prompt: string }[] = [
  {
    emoji: "📊",
    label: "ماشین‌حساب مدیریت ریسک و سود ترید",
    prompt:
      "یک ماشین‌حساب مدیریت ریسک و سود برای تریدر بساز: ورودی سرمایه، درصد ریسک هر معامله، حد ضرر و حد سود، محاسبهٔ خودکار حجم پوزیشن و نسبت ریسک/ریوارد با کارت‌های رنگی سبز و قرمز، اعداد فارسی و تم تیرهٔ حرفه‌ای.",
  },
  {
    emoji: "🎯",
    label: "داشبورد پیگیری عادت‌ها و تسک‌ها",
    prompt:
      "یک داشبورد هوشمند پیگیری عادت‌ها و تسک‌های روزانه بساز: افزودن عادت و تسک، تیک زدن روزانه، شمارش روزهای پیوسته (استریک)، نوار پیشرفت هفته و ذخیرهٔ خودکار در localStorage؛ طراحی مینیمال تیره با انیمیشن ملایم.",
  },
  {
    emoji: "🃏",
    label: "کارت ویزیت و لندینگ نئونی کریپتو",
    prompt:
      "یک لندینگ پیج نئونی برای برند کریپتو بساز: هیرو با گرادیان بنفش-آبی و افکت گلو، کارت ویزیت دیجیتال با دکمه‌های شبکه‌های اجتماعی، تایپوگرافی بزرگ و بخش معرفی خدمات؛ کاملاً ریسپانسیو برای موبایل.",
  },
  {
    emoji: "🧮",
    label: "ابزار محاسبهٔ سود مرکب و وام",
    prompt:
      "یک ابزار محاسبهٔ سود مرکب و اقساط وام بساز: دو تب (سود مرکب با نمودار رشد سرمایه، وام با قسط ماهانه)، اسلایدر برای نرخ سود و مدت، جدول سالانه و اعداد فارسی؛ تم تیرهٔ لوکس با طلایی.",
  },
];

/** نوار وضعیت سه‌مرحله‌ای بالای استودیو. */
function StudioSteps({ current }: { current: 1 | 2 | 3 }) {
  const steps = [
    { n: 1 as const, label: "گفتگو و ایده" },
    { n: 2 as const, label: "پیش‌نمایش زنده" },
    { n: 3 as const, label: "قیمت‌گذاری و لینک وایرال" },
  ];
  return (
    <div className="flex items-center gap-1.5 rounded-2xl border border-royal-500/20 bg-zinc-900/60 px-3 py-2.5">
      {steps.map((s, i) => {
        const done = current > s.n;
        const active = current === s.n;
        return (
          <div key={s.n} className="flex min-w-0 flex-1 items-center gap-1.5">
            {i > 0 && (
              <span
                className={`mx-0.5 h-px w-3 shrink-0 ${done ? "bg-emerald-400/60" : "bg-zinc-700"}`}
                aria-hidden
              />
            )}
            <span
              className={`grid size-5 shrink-0 place-items-center rounded-full text-[10px] font-black ${
                done
                  ? "bg-emerald-500/20 text-emerald-400 ring-1 ring-emerald-400/40"
                  : active
                    ? "bg-gold-400/20 text-gold-300 ring-1 ring-gold-400/50"
                    : "bg-zinc-800 text-zinc-500"
              }`}
              aria-hidden
            >
              {done ? <Check className="size-3" aria-hidden /> : s.n.toLocaleString("fa-IR")}
            </span>
            <span
              className={`min-w-0 truncate text-[10px] font-bold leading-4 ${
                done
                  ? "text-emerald-400"
                  : active
                    ? "text-gold-300"
                    : "text-zinc-500"
              }`}
            >
              {s.label}
            </span>
          </div>
        );
      })}
    </div>
  );
}

export default function BuilderPage() {
  const { state } = useSession();

  const [sessions, setSessions] = useState<BuilderSessionSummary[]>([]);
  const [active, setActive] = useState<BuilderSessionView | null>(null);
  const [loadingSession, setLoadingSession] = useState(false);

  const [draft, setDraft] = useState("");
  const [sending, setSending] = useState(false);
  const [lastMeta, setLastMeta] = useState<{
    usedFallback: boolean;
    provider: string | null;
    totalTokens: number;
  } | null>(null);

  // فرم انتشار یک‌کلیکه (پرشده از پیشنهادهای AI)
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [priceDin, setPriceDin] = useState("200");
  const [affiliateBps, setAffiliateBps] = useState<number>(1500);
  const [category, setCategory] = useState("html-app");
  const [publishing, setPublishing] = useState(false);
  const [published, setPublished] = useState<PublishBuilderResponse | null>(null);
  const [copiedLink, setCopiedLink] = useState(false);

  const chatEndRef = useRef<HTMLDivElement | null>(null);
  const draftRef = useRef<HTMLTextAreaElement | null>(null);

  const syncFormFromSession = useCallback((s: BuilderSessionView) => {
    setTitle(s.title);
    setDescription(s.description);
    setPriceDin(String(Math.max(1, s.suggestedPrice || 100)));
    setAffiliateBps(s.affiliateBps || 1000);
  }, []);

  const refreshList = useCallback(async (token: string) => {
    const res = await api.listBuilderSessions(token);
    setSessions(res.items);
    return res.items;
  }, []);

  const openSession = useCallback(
    async (token: string, id: string) => {
      setLoadingSession(true);
      try {
        const detail = await api.getBuilderSession(token, id);
        setActive(detail);
        setPublished(null);
        syncFormFromSession(detail);
      } catch (err) {
        toast({
          title: "بارگیری جلسه ناموفق بود",
          description: friendlyError(err),
          variant: "destructive",
        });
      } finally {
        setLoadingSession(false);
      }
    },
    [syncFormFromSession],
  );

  const startNewSession = useCallback(
    async (token: string) => {
      setLoadingSession(true);
      try {
        const created = await api.createBuilderSession(token);
        setActive(created);
        setPublished(null);
        setLastMeta(null);
        syncFormFromSession(created);
        await refreshList(token);
        toast({
          title: "استودیوی تازه آماده است",
          description: "یکی از ایده‌های آماده را بزن یا ایدهٔ خودت را بنویس ✨",
          variant: "success",
        });
      } catch (err) {
        toast({
          title: "ساخت جلسهٔ جدید ناموفق بود",
          description: friendlyError(err),
          variant: "destructive",
        });
      } finally {
        setLoadingSession(false);
      }
    },
    [refreshList, syncFormFromSession],
  );

  useEffect(() => {
    if (state.phase !== "ready") return;
    const { token } = state;
    let cancelled = false;

    const bootstrap = async () => {
      setLoadingSession(true);
      try {
        const items = await refreshList(token);
        if (cancelled) return;
        if (items.length > 0) {
          const detail = await api.getBuilderSession(token, items[0].id);
          if (!cancelled) {
            setActive(detail);
            syncFormFromSession(detail);
          }
        } else {
          const created = await api.createBuilderSession(token);
          if (!cancelled) {
            setActive(created);
            syncFormFromSession(created);
            await refreshList(token);
          }
        }
      } catch {
        /* کاربر می‌تواند دستی جلسهٔ جدید بسازد */
      } finally {
        if (!cancelled) setLoadingSession(false);
      }
    };

    void bootstrap();
    return () => {
      cancelled = true;
    };
  }, [state, refreshList, syncFormFromSession]);

  useEffect(() => {
    chatEndRef.current?.scrollIntoView({ behavior: "smooth", block: "nearest" });
  }, [active?.messages.length, sending]);

  if (state.phase === "loading") {
    return (
      <MiniShell>
        <div className="flex items-center justify-center gap-3 py-24 text-zinc-500">
          <Loader2 className="size-5 animate-spin" aria-hidden />
          <span className="text-sm">در حال اتصال به استودیو AI…</span>
        </div>
      </MiniShell>
    );
  }

  if (state.phase !== "ready") {
    return (
      <MiniShell>
        <div className="flex flex-col gap-4">
          {state.phase === "preview" && <PreviewBanner />}
          <GuestGate
            title="استودیو AI داخل تلگرام فعال می‌شود"
            hint="برای ساخت مینی‌اپ‌های HTML تعاملی و فروش با لینک رفرال، نشست شهروندی شما باید از داخل تلگرام احراز شود — بدون رمز و فقط با امضای امن تلگرام."
            whatWorksInPreview={state.phase !== "preview"}
          />
        </div>
      </MiniShell>
    );
  }

  const { me, token } = state;

  const sendMessage = async (forceGenerate: boolean, preset?: string) => {
    if (!active) return;
    const text = preset?.trim() || draft.trim();
    const finalText =
      text || (forceGenerate ? "لطفاً همین الان نسخهٔ کامل و تعاملی HTML این محصول را بساز." : "");
    if (!finalText) return;

    setSending(true);
    setDraft("");
    try {
      const res = await api.chatBuilderSession(
        token,
        active.id,
        finalText,
        forceGenerate,
      );
      setActive(res.session);
      setLastMeta({
        usedFallback: res.usedFallback,
        provider: res.provider,
        totalTokens: res.totalTokens,
      });
      syncFormFromSession(res.session);
      await refreshList(token);
      if (res.session.generatedHtml.trim()) {
        toast({
          title: "نسخهٔ زندهٔ HTML آماده شد 🎉",
          description: "پیش‌نمایش را تست کن؛ اگر دوستش داشتی برو سراغ قیمت‌گذاری.",
          variant: "success",
        });
      }
    } catch (err) {
      setDraft(finalText);
      toast({
        title: friendlyErrorTitle(err),
        description: friendlyError(err),
        variant: "destructive",
      });
    } finally {
      setSending(false);
    }
  };

  /** چیپ ایدهٔ آماده — اگر چت هنوز خالی است، مستقیم بساز؛ وگرنه در کادر بنشین. */
  const useStarterPrompt = (prompt: string) => {
    if (!active || sending) return;
    const chatIsEmpty = !active.messages.some((m) => m.role === "user");
    if (chatIsEmpty) {
      void sendMessage(true, prompt);
    } else {
      setDraft(prompt);
      draftRef.current?.focus();
      toast({ title: "ایده در کادر پیام نشست ✍️", description: "کمی ویرایشش کن و بفرست." });
    }
  };

  const handlePublish = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!active) return;
    const numericPrice = Math.max(1, Math.floor(Number(priceDin) || 0));
    if (!numericPrice) {
      toast({ title: "قیمت معتبر به دیین وارد کنید", variant: "destructive" });
      return;
    }

    setPublishing(true);
    try {
      const res = await api.publishBuilderSession(token, active.id, {
        ...(title.trim() ? { title: title.trim() } : {}),
        ...(description.trim() ? { description: description.trim() } : {}),
        priceDin: numericPrice,
        affiliateBps,
        ...(category.trim() ? { category: category.trim() } : {}),
      });
      setPublished(res);
      setCopiedLink(false);
      await openSession(token, active.id);
      setPublished(res);
      await refreshList(token);
      toast({
        title: res.alreadyPublished
          ? "این محصول قبلاً منتشر شده است"
          : "محصول با موفقیت در بازار کیان منتشر شد! 🚀",
        description: "لینک فروش و رفرال شما آمادهٔ اشتراک‌گذاری است.",
        variant: "success",
      });
    } catch (err) {
      toast({
        title: "انتشار محصول ناموفق بود",
        description: friendlyError(err),
        variant: "destructive",
      });
    } finally {
      setPublishing(false);
    }
  };

  const viralProductId =
    published?.product.id ?? active?.publishedProductId ?? null;
  const viralLink = viralProductId
    ? buildTelegramViralLink(viralProductId, me.user.telegramId)
    : null;

  const copyViralUrl = async () => {
    if (!viralLink) return;
    try {
      await navigator.clipboard.writeText(viralLink.telegramUrl);
      setCopiedLink(true);
      toast({ title: "لینک فروش و رفرال کپی شد ✅", variant: "success" });
      window.setTimeout(() => setCopiedLink(false), 2000);
    } catch {
      toast({
        title: "کپی خودکار مجاز نبود",
        description: viralLink.telegramUrl,
        variant: "destructive",
      });
    }
  };

  const hasHtml = Boolean(active?.generatedHtml.trim());
  // گارد zero-flicker (پورت OpenIntelligentUI): پیش‌نمایش هرگز HTML نیمه‌راهِ
  // تگ باز/اسکریپت باز/موجودیت ناقص نمی‌بیند؛ نسخهٔ دانلود همیشه سند کامل است.
  const previewHtml = useMemo(
    () => sanitizePartialStudioHtml(active?.generatedHtml ?? ""),
    [active?.generatedHtml],
  );
  const currentStep: 1 | 2 | 3 = viralProductId ? 3 : hasHtml ? 2 : 1;

  return (
    <MiniShell>
      <div className="flex flex-col gap-5">
        {/* سرصفحه + انتخاب/ساخت جلسه */}
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div>
            <h1 className="flex items-center gap-2 text-lg font-black">
              <Sparkles className="size-5 text-gold-400" aria-hidden />
              استودیو AI (ساخت محصول)
            </h1>
            <p className="mt-0.5 text-xs text-zinc-400">
              گفتگو کن، فایل HTML تعاملی بساز و با یک کلیک همراه لینک رفرال بفروش.
            </p>
          </div>
          <button
            type="button"
            onClick={() => void startNewSession(token)}
            disabled={loadingSession}
            className="flex items-center gap-1.5 rounded-xl bg-royal-500/20 px-3 py-2 text-xs font-bold text-gold-400 ring-1 ring-gold-400/30 hover:bg-royal-500/30 disabled:opacity-50"
          >
            <Plus className="size-4" aria-hidden />
            جلسهٔ جدید
          </button>
        </div>

        {/* نوار وضعیت سه‌مرحله‌ای */}
        <StudioSteps current={currentStep} />

        {/* نوار جلسات قبلی */}
        {sessions.length > 1 && (
          <div className="flex gap-2 overflow-x-auto pb-1">
            {sessions.slice(0, 8).map((s) => {
              const selected = active?.id === s.id;
              return (
                <button
                  key={s.id}
                  type="button"
                  onClick={() => void openSession(token, s.id)}
                  className={`shrink-0 rounded-xl px-3 py-1.5 text-xs font-semibold ring-1 transition ${
                    selected
                      ? "bg-royal-500/25 text-gold-300 ring-gold-400/40"
                      : "bg-zinc-900/70 text-zinc-400 ring-zinc-800 hover:text-zinc-200"
                  }`}
                >
                  {s.title.trim() || "جلسهٔ بدون عنوان"}
                  {s.publishedProductId ? " ✓" : s.hasHtml ? " • HTML" : ""}
                </button>
              );
            })}
          </div>
        )}

        {/* بخش ۱: چت مصاحبه و ساخت */}
        <section className="flex flex-col gap-3 rounded-2xl border border-royal-500/20 bg-zinc-900/60 p-4">
          <div className="flex items-center justify-between gap-2">
            <h2 className="flex items-center gap-1.5 text-sm font-bold text-zinc-200">
              <MessageCircle className="size-4 text-royal-300" aria-hidden />
              گفتگوی طراحی و ساخت
            </h2>
            <div className="flex items-center gap-2 text-[11px] text-zinc-400">
              {active && (
                <span className="rounded-full bg-zinc-800 px-2.5 py-0.5 font-semibold text-zinc-300">
                  مرحله:{" "}
                  {active.stage === "published"
                    ? "منتشرشده"
                    : active.stage === "ready"
                      ? "آمادهٔ انتشار"
                      : "کشف ایده"}
                </span>
              )}
              {lastMeta && (
                <span className="rounded-full bg-royal-500/15 px-2 py-0.5 text-royal-300">
                  {lastMeta.usedFallback
                    ? "مولد داخلی"
                    : `${lastMeta.provider ?? "AI"} • ${lastMeta.totalTokens} tok`}
                </span>
              )}
            </div>
          </div>

          <div className="flex max-h-80 flex-col gap-2.5 overflow-y-auto rounded-xl border border-zinc-800 bg-zinc-950/70 p-3">
            {loadingSession && !active ? (
              <div className="flex items-center justify-center gap-2 py-8 text-xs text-zinc-500">
                <Loader2 className="size-4 animate-spin" aria-hidden />
                در حال آماده‌سازی استودیو…
              </div>
            ) : (
              (active?.messages ?? []).map((m) => {
                const isUser = m.role === "user";
                return (
                  <div
                    key={m.id}
                    className={`max-w-[88%] whitespace-pre-wrap rounded-2xl px-3.5 py-2.5 text-xs leading-6 ${
                      isUser
                        ? "self-start bg-royal-500/25 text-zinc-100 ring-1 ring-royal-400/30"
                        : "self-end bg-zinc-900 text-zinc-200 ring-1 ring-zinc-800"
                    }`}
                  >
                    {m.content}
                  </div>
                );
              })
            )}
            {sending && (
              <div className="flex items-center gap-2 self-end rounded-2xl bg-royal-500/10 px-3.5 py-2.5 text-xs text-royal-300 ring-1 ring-royal-400/25">
                <Loader2 className="size-3.5 animate-spin" aria-hidden />
                در حال طراحیِ کدِ HTML… (معمولاً چند ثانیه)
              </div>
            )}
            <div ref={chatEndRef} />
          </div>

          {/* چیپ‌های ایدهٔ آمادهٔ یک‌لمسی — هیچ‌کس مقابل چت خالی نمی‌ماند */}
          <div className="flex flex-col gap-1.5">
            <p className="text-[11px] font-bold text-zinc-400">
              یا یکی از این ایده‌های آماده را بزن:
            </p>
            <div className="flex flex-col gap-1.5">
              {STARTER_PROMPTS.map((p) => (
                <div key={p.label} className="flex items-stretch gap-1.5">
                  <button
                    type="button"
                    onClick={() => useStarterPrompt(p.prompt)}
                    disabled={sending || !active}
                    className="flex min-w-0 flex-1 items-center gap-1.5 rounded-xl bg-zinc-950/80 px-3 py-2 text-right text-[11px] font-semibold leading-5 text-zinc-300 ring-1 ring-zinc-800 transition hover:bg-zinc-900 hover:text-white hover:ring-gold-400/30 disabled:opacity-40"
                  >
                    <span className="shrink-0 text-sm" aria-hidden>
                      {p.emoji}
                    </span>
                    <span className="min-w-0 flex-1 truncate">{p.label}</span>
                  </button>
                  <button
                    type="button"
                    onClick={() => void sendMessage(true, p.prompt)}
                    disabled={sending || !active}
                    aria-label={`ساخت فوری: ${p.label}`}
                    title="ساخت فوری همین ایده"
                    className="grid w-9 shrink-0 place-items-center rounded-xl bg-amber-500/15 text-gold-400 ring-1 ring-gold-400/35 transition hover:bg-amber-500/30 disabled:opacity-40"
                  >
                    <Zap className="size-4" aria-hidden />
                  </button>
                </div>
              ))}
            </div>
          </div>

          <form
            onSubmit={(e) => {
              e.preventDefault();
              void sendMessage(false);
            }}
            className="flex flex-col gap-2"
          >
            <textarea
              ref={draftRef}
              value={draft}
              onChange={(e) => setDraft(e.target.value)}
              rows={2}
              maxLength={4000}
              disabled={sending || !active}
              placeholder="ایده‌ات را بنویس یا بخواه تغییر دهد (مثلاً: ماشین‌حساب سود کریپتو با تم نئون تیره)…"
              className="w-full resize-none rounded-xl border border-zinc-700 bg-zinc-950 px-3 py-2.5 text-sm outline-none focus:border-gold-400/50 disabled:opacity-50"
            />
            <div className="flex flex-wrap items-center gap-2">
              <button
                type="submit"
                disabled={sending || !active || !draft.trim()}
                className="flex flex-1 items-center justify-center gap-1.5 rounded-xl bg-gradient-to-b from-royal-400 to-royal-600 px-4 py-2.5 text-xs font-bold text-white shadow-lg shadow-royal-600/30 ring-1 ring-gold-400/40 hover:opacity-90 disabled:opacity-40"
              >
                {sending ? (
                  <Loader2 className="size-4 animate-spin" aria-hidden />
                ) : (
                  <Send className="size-4" aria-hidden />
                )}
                {sending ? "در حال طراحی…" : "ارسال پیام"}
              </button>
              <button
                type="button"
                onClick={() => void sendMessage(true)}
                disabled={sending || !active}
                className="flex items-center gap-1.5 rounded-xl bg-amber-500/15 px-3.5 py-2.5 text-xs font-bold text-gold-400 ring-1 ring-gold-400/35 hover:bg-amber-500/25 disabled:opacity-40"
              >
                <Zap className="size-4" aria-hidden />
                ساخت فوری نسخهٔ اول
              </button>
            </div>
          </form>
        </section>

        {/* بخش ۲: پیش‌نمایش زنده در iframe sandbox */}
        {hasHtml && active && (
          <section className="flex flex-col gap-2.5">
            <h2 className="text-sm font-bold text-zinc-200">
              پیش‌نمایش زندهٔ محصول — مرحلهٔ ۲ از ۳
            </h2>
            <HtmlPreviewFrame
              html={previewHtml}
              downloadHtml={active.generatedHtml}
              title={title || "پیش‌نمایش زندهٔ محصول"}
              allowDownload
              downloadFilename={`${title || active.id}.html`}
              badgeLabel="پیش‌نمایش زندهٔ استودیو"
            />
          </section>
        )}

        {/* بخش ۳: فرم قیمت‌گذاری، پورسانت رفرال و انتشار یک‌کلیکه */}
        {hasHtml && active && (
          <section className="rounded-2xl border border-gold-400/25 bg-zinc-900/70 p-4">
            <h2 className="mb-3 flex items-center gap-2 text-sm font-bold text-gold-400">
              <Rocket className="size-4" aria-hidden />
              قیمت‌گذاری، پورسانت معرفی و انتشار در بازار — مرحلهٔ ۳ از ۳
            </h2>
            <form onSubmit={(e) => void handlePublish(e)} className="grid gap-3">
              <label className="text-xs text-zinc-400">
                عنوان محصول
                <input
                  value={title}
                  onChange={(e) => setTitle(e.target.value)}
                  required
                  minLength={1}
                  maxLength={120}
                  placeholder="عنوان جذاب محصول"
                  className="mt-1 w-full rounded-xl border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 outline-none focus:border-gold-400/50"
                />
              </label>

              <label className="text-xs text-zinc-400">
                توضیحات فروش (CAST)
                <textarea
                  value={description}
                  onChange={(e) => setDescription(e.target.value)}
                  required
                  minLength={1}
                  maxLength={2000}
                  rows={3}
                  className="mt-1 w-full rounded-xl border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 outline-none focus:border-gold-400/50"
                />
              </label>

              <div className="grid grid-cols-2 gap-3">
                <label className="text-xs text-zinc-400">
                  قیمت فروش (Đ)
                  <input
                    value={priceDin}
                    onChange={(e) => setPriceDin(e.target.value)}
                    required
                    inputMode="numeric"
                    dir="ltr"
                    className="mt-1 w-full rounded-xl border border-zinc-700 bg-zinc-950 px-3 py-2 font-mono text-sm text-gold-300 outline-none focus:border-gold-400/50"
                  />
                </label>

                <label className="text-xs text-zinc-400">
                  پاداش معرف ({formatAffiliatePercent(affiliateBps)})
                  <select
                    value={affiliateBps}
                    onChange={(e) => setAffiliateBps(Number(e.target.value))}
                    className="mt-1 w-full rounded-xl border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 outline-none focus:border-gold-400/50"
                  >
                    {AFFILIATE_PRESETS.map((bps) => (
                      <option key={bps} value={bps}>
                        {formatAffiliatePercent(bps)} پورسانت رفرال
                      </option>
                    ))}
                  </select>
                </label>
              </div>

              <label className="text-xs text-zinc-400">
                دسته‌بندی
                <input
                  value={category}
                  onChange={(e) => setCategory(e.target.value)}
                  maxLength={40}
                  placeholder="html-app"
                  className="mt-1 w-full rounded-xl border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 outline-none focus:border-gold-400/50"
                />
              </label>

              <p className="rounded-xl bg-zinc-950/80 px-3 py-2 text-[11px] leading-5 text-zinc-400 ring-1 ring-zinc-800">
                تقسیم خودکار دفتر دوطرفه در لحظهٔ تسویه: ۱۰٪ خزانهٔ کیان +{" "}
                <strong className="text-emerald-400">
                  {formatAffiliatePercent(affiliateBps)}
                </strong>{" "}
                پاداش معرف (در خریدهای رفرالی) + مابقی خالص به کیف شما.
              </p>

              <button
                type="submit"
                disabled={publishing}
                className="flex items-center justify-center gap-2 rounded-xl bg-gradient-to-b from-royal-400 to-royal-600 px-4 py-3 text-sm font-black text-white shadow-lg shadow-royal-600/30 ring-1 ring-gold-400/40 hover:opacity-90 disabled:opacity-50"
              >
                {publishing ? (
                  <Loader2 className="size-4 animate-spin" aria-hidden />
                ) : (
                  <Rocket className="size-4" aria-hidden />
                )}
                {viralProductId
                  ? "نمایش / تأیید محصول منتشرشده"
                  : "🚀 انتشار در بازار و دریافت لینک فروش"}
              </button>
            </form>
          </section>
        )}

        {/* بخش ۴: جعبهٔ لینک وایرال و رفرال تلگرام */}
        {viralLink && viralProductId && (
          <section className="flex flex-col gap-3 rounded-2xl border border-emerald-500/35 bg-emerald-950/20 p-4">
            <div className="flex items-center justify-between gap-2">
              <h2 className="text-sm font-black text-emerald-300">
                🎉 لینک آمادهٔ فروش و رفرال تلگرام
              </h2>
              <Link
                href={`/market?product=${encodeURIComponent(viralProductId)}`}
                className="flex items-center gap-1 text-xs font-bold text-gold-400 hover:underline"
              >
                <span>نمایش در بازار</span>
                <ExternalLink className="size-3.5" aria-hidden />
              </Link>
            </div>
            <p className="text-xs leading-5 text-zinc-300">
              این لینک را در کانال، گروه یا پی‌وی پخش کن؛ هر کسی با آن وارد شود
              پیش‌نمایش محصولت را می‌بیند و شناسهٔ معرف به‌طور خودکار روی خریدش ثبت می‌شود.
            </p>
            <div className="flex items-center gap-2 rounded-xl border border-emerald-500/30 bg-zinc-950 px-3 py-2">
              <code
                dir="ltr"
                className="min-w-0 flex-1 truncate font-mono text-xs text-emerald-300"
              >
                {viralLink.telegramUrl}
              </code>
              <button
                type="button"
                onClick={() => void copyViralUrl()}
                className="flex shrink-0 items-center gap-1 rounded-lg bg-emerald-500/20 px-3 py-1.5 text-xs font-bold text-emerald-300 ring-1 ring-emerald-400/40 hover:bg-emerald-500/30"
              >
                {copiedLink ? (
                  <Check className="size-3.5" aria-hidden />
                ) : (
                  <Copy className="size-3.5" aria-hidden />
                )}
                {copiedLink ? "کپی شد" : "کپی لینک"}
              </button>
            </div>
            <p className="flex flex-wrap items-center gap-1.5 rounded-xl bg-emerald-500/10 px-3 py-2 text-[11px] font-bold leading-5 text-emerald-300 ring-1 ring-emerald-400/25">
              💰 با هر خرید از این لینک{" "}
              {formatAffiliatePercent(published?.product.affiliateBps ?? affiliateBps)}{" "}
              پورسانت آنی به کیف تو واریز می‌شود.
            </p>
            <p className="font-mono text-[11px] text-zinc-500" dir="ltr">
              startapp={published?.shareStartParam ?? viralLink.startParam}
            </p>
          </section>
        )}
      </div>
    </MiniShell>
  );
}
