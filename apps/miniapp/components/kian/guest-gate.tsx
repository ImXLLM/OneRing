"use client";

/**
 * KIAN miniapp — دروازهٔ مهمان: بنر تمیز «ورود سریع آزمایشی / مرورگر» + ورود
 * یک‌کلیکی پرسوناهای دمو (فقط وقتی کرنل در حالت dev است) + راهنمای شفاف اتصال
 * از داخل تلگرام. جایگزین صفحهٔ قفل‌شده و خطای خشک فنی.
 *
 * ورود واقعی همچنان فقط از initData امضاشدهٔ تلگرام و HMAC کرنل می‌گذرد؛ حالت
 * مرورگر فقط اندپوینت‌های عمومی (بازار و اقتصاد ملی) را باز می‌کند و ورود دمو
 * فقط وقتی کار می‌کند که سرور با KIAN_DEV_MODE=1 بالا آمده باشد.
 */

import { useState } from "react";
import { Bot, FlaskConical, Loader2, ShieldCheck, Wifi } from "lucide-react";
import { toast } from "@kian/ui";
import { useSession } from "@/components/kian/session";
import { friendlyError } from "@/lib/friendly";
import type { DemoPersona } from "@kian/sdk";

function botUsername(): string {
  return (
    (typeof process !== "undefined" && process.env?.NEXT_PUBLIC_TELEGRAM_BOT_USERNAME) ||
    "KianAgentBot"
  ).replace(/^@/, "");
}

export function GuestGate({
  title = "برای این بخش، ورود از داخل تلگرام لازم است",
  hint = "هویت KIAN فقط با امضای رمزنگاری‌شدهٔ تلگرام (HMAC کرنل) ساخته می‌شود — پس رمزی از تو نمی‌خواهیم و هرگز در مرورگر جعل نمی‌شود.",
  whatWorksInPreview = true,
}: {
  title?: string;
  hint?: string;
  /** آیا در این صفحه حالت مرورگر چیزی مفید برای دیدن دارد؟ */
  whatWorksInPreview?: boolean;
}) {
  const { enterPreview, loginAsDemoPersona } = useSession();
  const [copied, setCopied] = useState(false);
  const [personaBusy, setPersonaBusy] = useState<DemoPersona | null>(null);

  const botLink = `https://t.me/${botUsername()}`;

  const copyBotLink = async () => {
    try {
      await navigator.clipboard.writeText(botLink);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 2000);
    } catch {
      /* کپی در دسترس نیست — لینک پایین نمایش داده می‌شود */
    }
  };

  const personaLogin = async (persona: DemoPersona) => {
    setPersonaBusy(persona);
    try {
      await loginAsDemoPersona(persona);
    } catch (err) {
      toast({
        title: "ورود آزمایشی فعال نشد",
        description: friendlyError(err),
        variant: "destructive",
      });
    } finally {
      setPersonaBusy(null);
    }
  };

  const personas: { key: DemoPersona; label: string }[] = [
    { key: "creator", label: "🎨 ورود یک‌کلیکی: سازنده (#1001)" },
    { key: "buyer", label: "🛒 ورود یک‌کلیکی: خریدار (#2002)" },
    { key: "affiliate", label: "🔗 ورود یک‌کلیکی: بازاریاب (#3003)" },
  ];

  return (
    <div className="flex flex-col gap-4 rounded-2xl border border-royal-500/25 bg-zinc-900/70 p-6 text-center shadow-xl shadow-black/30">
      <div className="mx-auto grid size-14 place-items-center rounded-2xl bg-gradient-to-b from-royal-500/30 to-royal-600/20 ring-1 ring-gold-400/30">
        <Bot className="size-7 text-gold-400" aria-hidden />
      </div>
      <h1 className="text-lg font-black leading-7">{title}</h1>
      <p className="mx-auto max-w-md text-xs leading-6 text-zinc-400">{hint}</p>

      {/* اقدام اصلی: ورود آزمایشی / مرورگر — بدون اصطکاک، بدون خطای خشک */}
      {whatWorksInPreview && (
        <button
          type="button"
          onClick={enterPreview}
          className="mx-auto flex w-full max-w-xs items-center justify-center gap-2 rounded-xl bg-gradient-to-b from-royal-400 to-royal-600 px-4 py-3 text-sm font-black text-white shadow-lg shadow-royal-600/30 ring-1 ring-gold-400/40 transition hover:opacity-90 active:scale-[0.98]"
        >
          <FlaskConical className="size-4" aria-hidden />
          🧪 ورود سریع آزمایشی / مرورگر
        </button>
      )}

      {/* ورود یک‌کلیکی پرسوناهای دمو — حلقهٔ کاملِ سازنده→خرید→برداشت در مرورگر */}
      {whatWorksInPreview && (
        <div className="flex w-full flex-col gap-2 rounded-xl bg-zinc-950/70 p-3 ring-1 ring-royal-500/30">
          <p className="text-[11px] font-bold leading-5 text-royal-300">
            🧬 ورود با پرسونای دمو (تستِ محلی کامل — ساخت، خرید و برداشت):
          </p>
          <div className="grid gap-2">
            {personas.map((p) => (
              <button
                key={p.key}
                type="button"
                disabled={personaBusy !== null}
                onClick={() => void personaLogin(p.key)}
                className="flex items-center justify-center gap-2 rounded-lg bg-gradient-to-b from-royal-500/25 to-royal-600/15 px-3 py-2.5 text-xs font-black text-white ring-1 ring-royal-400/40 transition hover:from-royal-500/40 disabled:opacity-50"
              >
                {personaBusy === p.key && (
                  <Loader2 className="size-3.5 animate-spin" aria-hidden />
                )}
                {p.label}
              </button>
            ))}
          </div>
          <p className="text-[10px] leading-4 text-zinc-500">
            با این سه پرسونا می‌توانی امشب کل چرخه را در مرورگر تست کنی:
            ساختِ اپ ← انتشار ← لینک رفرال ← خرید ← اسکرو ← برداشت کریپتو.
          </p>
        </div>
      )}
      {whatWorksInPreview && (
        <p className="text-[11px] leading-5 text-zinc-500">
          با این حالت می‌توانی بازار، پیش‌نمایش محصولات و اقتصاد ملی را ببینی؛
          خرید و فروش و استودیوی AI فقط داخل تلگرام فعال می‌شود.
        </p>
      )}

      {/* راهنمای شفاف اتصال واقعی */}
      <div className="mt-1 flex flex-col gap-2 rounded-xl bg-zinc-950/70 p-3 text-right ring-1 ring-zinc-800">
        <div className="flex items-center gap-2 text-[11px] font-bold text-gold-400">
          <Wifi className="size-3.5" aria-hidden />
          اتصال واقعی در ۳ قدم:
        </div>
        <ol className="list-inside list-decimal space-y-1 text-[11px] leading-5 text-zinc-400">
          <li>ربات KIAN را در تلگرام باز کن: <code dir="ltr" className="font-mono text-zinc-300">@{botUsername()}</code></li>
          <li>دکمهٔ «ورود به مینی‌اپ» را بزن (منوی Start یا /link).</li>
          <li>همین صفحه داخل تلگرام باز می‌شود و ورود بی‌صدا انجام می‌شود.</li>
        </ol>
        <button
          type="button"
          onClick={() => void copyBotLink()}
          className="mx-auto mt-1 w-full max-w-[240px] rounded-lg bg-zinc-800 px-3 py-2 text-[11px] font-bold text-zinc-200 ring-1 ring-zinc-700 hover:bg-zinc-700"
        >
          {copied ? "لینک ربات کپی شد ✓" : "کپی لینک ربات تلگرام"}
        </button>
      </div>

      <p className="flex items-center justify-center gap-1.5 text-[10px] text-zinc-600">
        <ShieldCheck className="size-3" aria-hidden />
        نشست تو فقط روی سرور رسمی کرنل ساخته می‌شود — بدون رمز، بدون ریسک.
      </p>
    </div>
  );
}

/** بنر باریک بالای صفحات در حالت مرورگر آزمایشی. */
export function PreviewBanner() {
  const { exitPreview } = useSession();
  return (
    <div className="flex flex-wrap items-center justify-between gap-2 rounded-xl border border-gold-400/35 bg-gold-400/10 px-3 py-2 text-[11px] leading-5 text-gold-300">
      <span>
        🧪 <strong className="font-bold">حالت مرورگر آزمایشی</strong> — فقط بخش‌های
        عمومی فعال است؛ برای خرید، فروش و استودیوی AI از داخل تلگرام وارد شو.
      </span>
      <button
        type="button"
        onClick={exitPreview}
        className="shrink-0 rounded-lg bg-gold-400/15 px-2.5 py-1 font-bold text-gold-300 ring-1 ring-gold-400/40 hover:bg-gold-400/25"
      >
        تلاش برای ورود واقعی
      </button>
    </div>
  );
}

const PERSONA_META: Record<
  DemoPersona,
  { label: string; tag: string; href: string }
> = {
  creator: { label: "سازنده", tag: "#1001", href: "/builder" },
  buyer: { label: "خریدار", tag: "#2002", href: "/market" },
  affiliate: { label: "بازاریاب", tag: "#3003", href: "/market" },
};

/**
 * نوار فشردهٔ تعویض پرسونای دمو — فقط برای نشست‌های دمو نمایش داده می‌شود؛
 * بنیان‌گذار می‌تواند وسطِ تست، با یک کلیک بین سازنده/خریدار/بازاریاب جابه‌جا شود.
 */
export function DemoPersonaBar() {
  const { activePersona, loginAsDemoPersona, state } = useSession();
  const [busy, setBusy] = useState<DemoPersona | null>(null);
  if (state.phase !== "ready" || !activePersona) return null;

  const switchTo = async (persona: DemoPersona) => {
    if (persona === activePersona || busy) return;
    setBusy(persona);
    try {
      await loginAsDemoPersona(persona);
    } catch {
      /* خطای ورود دمو در خودِ هوک و toast سطح صفحه نمایش داده شده است */
    } finally {
      setBusy(null);
    }
  };

  return (
    <div className="flex flex-wrap items-center gap-1.5 rounded-xl border border-royal-500/30 bg-royal-500/10 px-2.5 py-2 text-[11px]">
      <span className="font-bold text-royal-300">
        🧬 حالت تست:
        {" "}
        <strong className="text-gold-400">
          {PERSONA_META[activePersona].label} {PERSONA_META[activePersona].tag}
        </strong>
      </span>
      <span className="mr-auto flex flex-wrap gap-1">
        {(Object.keys(PERSONA_META) as DemoPersona[]).map((p) => (
          <button
            key={p}
            type="button"
            disabled={busy !== null || p === activePersona}
            onClick={() => void switchTo(p)}
            className={`rounded-lg px-2 py-1 font-bold ring-1 transition disabled:opacity-45 ${
              p === activePersona
                ? "bg-gold-400/20 text-gold-300 ring-gold-400/40"
                : "bg-zinc-800/80 text-zinc-300 ring-zinc-700 hover:bg-zinc-700"
            }`}
          >
            {busy === p ? "…" : `${PERSONA_META[p].label} ${PERSONA_META[p].tag}`}
          </button>
        ))}
      </span>
    </div>
  );
}
