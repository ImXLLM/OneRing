"use client";

/**
 * KIAN miniapp — guest gate: a clean "Quick experimental sign-in / Browser" banner + one-click
 * demo persona sign-in (only when the kernel is in dev mode) + a transparent guide to connecting
 * from inside Telegram. Replaces the locked screen and dry technical error.
 *
 * Real sign-in still goes only through Telegram's signed initData and the kernel HMAC; browser
 * mode opens only public endpoints (market and national economy), and demo sign-in
 * works only when the server is up with KIAN_DEV_MODE=1.
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
  title = "\u0628\u0631\u0627\u06cc \u0627\u06cc\u0646 \u0628\u062e\u0634\u060c \u0648\u0631\u0648\u062f \u0627\u0632 \u062f\u0627\u062e\u0644 \u062a\u0644\u06af\u0631\u0627\u0645 \u0644\u0627\u0632\u0645 \u0627\u0633\u062a",
  hint = "\u0647\u0648\u06cc\u062a KIAN \u0641\u0642\u0637 \u0628\u0627 \u0627\u0645\u0636\u0627\u06cc \u0631\u0645\u0632\u0646\u06af\u0627\u0631\u06cc‌\u0634\u062f\u0647\u0654 \u062a\u0644\u06af\u0631\u0627\u0645 (HMAC \u06a9\u0631\u0646\u0644) \u0633\u0627\u062e\u062a\u0647 \u0645\u06cc‌\u0634\u0648\u062f — \u067e\u0633 \u0631\u0645\u0632\u06cc \u0627\u0632 \u062a\u0648 \u0646\u0645\u06cc‌\u062e\u0648\u0627\u0647\u06cc\u0645 \u0648 \u0647\u0631\u06af\u0632 \u062f\u0631 \u0645\u0631\u0648\u0631\u06af\u0631 \u062c\u0639\u0644 \u0646\u0645\u06cc‌\u0634\u0648\u062f.",
  whatWorksInPreview = true,
}: {
  title?: string;
  hint?: string;
  /** Does browser mode have anything worth seeing on this page? */
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
      /* Copy unavailable — the link below is shown instead */
    }
  };

  const personaLogin = async (persona: DemoPersona) => {
    setPersonaBusy(persona);
    try {
      await loginAsDemoPersona(persona);
    } catch (err) {
      toast({
        title: "\u0648\u0631\u0648\u062f \u0622\u0632\u0645\u0627\u06cc\u0634\u06cc \u0641\u0639\u0627\u0644 \u0646\u0634\u062f",
        description: friendlyError(err),
        variant: "destructive",
      });
    } finally {
      setPersonaBusy(null);
    }
  };

  const personas: { key: DemoPersona; label: string }[] = [
    { key: "creator", label: "🎨 \u0648\u0631\u0648\u062f \u06cc\u06a9‌\u06a9\u0644\u06cc\u06a9\u06cc: \u0633\u0627\u0632\u0646\u062f\u0647 (#1001)" },
    { key: "buyer", label: "🛒 \u0648\u0631\u0648\u062f \u06cc\u06a9‌\u06a9\u0644\u06cc\u06a9\u06cc: \u062e\u0631\u06cc\u062f\u0627\u0631 (#2002)" },
    { key: "affiliate", label: "🔗 \u0648\u0631\u0648\u062f \u06cc\u06a9‌\u06a9\u0644\u06cc\u06a9\u06cc: \u0628\u0627\u0632\u0627\u0631\u06cc\u0627\u0628 (#3003)" },
  ];

  return (
    <div className="flex flex-col gap-4 rounded-2xl border border-royal-500/25 bg-zinc-900/70 p-6 text-center shadow-xl shadow-black/30">
      <div className="mx-auto grid size-14 place-items-center rounded-2xl bg-gradient-to-b from-royal-500/30 to-royal-600/20 ring-1 ring-gold-400/30">
        <Bot className="size-7 text-gold-400" aria-hidden />
      </div>
      <h1 className="text-lg font-black leading-7">{title}</h1>
      <p className="mx-auto max-w-md text-xs leading-6 text-zinc-400">{hint}</p>

      {/* Primary action: experimental / browser sign-in — no friction, no dry error */}
      {whatWorksInPreview && (
        <button
          type="button"
          onClick={enterPreview}
          className="mx-auto flex w-full max-w-xs items-center justify-center gap-2 rounded-xl bg-gradient-to-b from-royal-400 to-royal-600 px-4 py-3 text-sm font-black text-white shadow-lg shadow-royal-600/30 ring-1 ring-gold-400/40 transition hover:opacity-90 active:scale-[0.98]"
        >
          <FlaskConical className="size-4" aria-hidden />
          🧪 &#x648;&#x631;&#x648;&#x62f; &#x633;&#x631;&#x6cc;&#x639; &#x622;&#x632;&#x645;&#x627;&#x6cc;&#x634;&#x6cc; / &#x645;&#x631;&#x648;&#x631;&#x6af;&#x631;
        </button>
      )}

      {/* One-click demo persona sign-in — the full creator→buyer→marketer loop in the browser */}
      {whatWorksInPreview && (
        <div className="flex w-full flex-col gap-2 rounded-xl bg-zinc-950/70 p-3 ring-1 ring-royal-500/30">
          <p className="text-[11px] font-bold leading-5 text-royal-300">
            🧬 &#x648;&#x631;&#x648;&#x62f; &#x628;&#x627; &#x67e;&#x631;&#x633;&#x648;&#x646;&#x627;&#x6cc; &#x62f;&#x645;&#x648; (&#x62a;&#x633;&#x62a;&#x650; &#x645;&#x62d;&#x644;&#x6cc; &#x6a9;&#x627;&#x645;&#x644; — &#x633;&#x627;&#x62e;&#x62a;&#x60c; &#x62e;&#x631;&#x6cc;&#x62f; &#x648; &#x628;&#x631;&#x62f;&#x627;&#x634;&#x62a;):
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
            &#x628;&#x627; &#x627;&#x6cc;&#x646; &#x633;&#x647; &#x67e;&#x631;&#x633;&#x648;&#x646;&#x627; &#x645;&#x6cc;‌&#x62a;&#x648;&#x627;&#x646;&#x6cc; &#x627;&#x645;&#x634;&#x628; &#x6a9;&#x644; &#x686;&#x631;&#x62e;&#x647; &#x631;&#x627; &#x62f;&#x631; &#x645;&#x631;&#x648;&#x631;&#x6af;&#x631; &#x62a;&#x633;&#x62a; &#x6a9;&#x646;&#x6cc;:
            &#x633;&#x627;&#x62e;&#x62a;&#x650; &#x627;&#x67e; ← &#x627;&#x646;&#x62a;&#x634;&#x627;&#x631; ← &#x644;&#x6cc;&#x646;&#x6a9; &#x631;&#x641;&#x631;&#x627;&#x644; ← &#x62e;&#x631;&#x6cc;&#x62f; ← &#x627;&#x633;&#x6a9;&#x631;&#x648; ← &#x628;&#x631;&#x62f;&#x627;&#x634;&#x62a; &#x6a9;&#x631;&#x6cc;&#x67e;&#x62a;&#x648;.
          </p>
        </div>
      )}
      {whatWorksInPreview && (
        <p className="text-[11px] leading-5 text-zinc-500">
          &#x628;&#x627; &#x627;&#x6cc;&#x646; &#x62d;&#x627;&#x644;&#x62a; &#x645;&#x6cc;‌&#x62a;&#x648;&#x627;&#x646;&#x6cc; &#x628;&#x627;&#x632;&#x627;&#x631;&#x60c; &#x67e;&#x6cc;&#x634;‌&#x646;&#x645;&#x627;&#x6cc;&#x634; &#x645;&#x62d;&#x635;&#x648;&#x644;&#x627;&#x62a; &#x648; &#x627;&#x642;&#x62a;&#x635;&#x627;&#x62f; &#x645;&#x644;&#x6cc; &#x631;&#x627; &#x628;&#x628;&#x6cc;&#x646;&#x6cc;&#x61b;
          &#x62e;&#x631;&#x6cc;&#x62f; &#x648; &#x641;&#x631;&#x648;&#x634; &#x648; &#x627;&#x633;&#x62a;&#x648;&#x62f;&#x6cc;&#x648;&#x6cc; AI &#x641;&#x642;&#x637; &#x62f;&#x627;&#x62e;&#x644; &#x62a;&#x644;&#x6af;&#x631;&#x627;&#x645; &#x641;&#x639;&#x627;&#x644; &#x645;&#x6cc;‌&#x634;&#x648;&#x62f;.
        </p>
      )}

      {/* Transparent guide to connecting for real */}
      <div className="mt-1 flex flex-col gap-2 rounded-xl bg-zinc-950/70 p-3 text-right ring-1 ring-zinc-800">
        <div className="flex items-center gap-2 text-[11px] font-bold text-gold-400">
          <Wifi className="size-3.5" aria-hidden />
          &#x627;&#x62a;&#x635;&#x627;&#x644; &#x648;&#x627;&#x642;&#x639;&#x6cc; &#x62f;&#x631; &#x6f3; &#x642;&#x62f;&#x645;:
        </div>
        <ol className="list-inside list-decimal space-y-1 text-[11px] leading-5 text-zinc-400">
          <li>&#x631;&#x628;&#x627;&#x62a; KIAN &#x631;&#x627; &#x62f;&#x631; &#x62a;&#x644;&#x6af;&#x631;&#x627;&#x645; &#x628;&#x627;&#x632; &#x6a9;&#x646;: <code dir="ltr" className="font-mono text-zinc-300">@{botUsername()}</code></li>
          <li>&#x62f;&#x6a9;&#x645;&#x647;&#x654; «&#x648;&#x631;&#x648;&#x62f; &#x628;&#x647; &#x645;&#x6cc;&#x646;&#x6cc;‌&#x627;&#x67e;» &#x631;&#x627; &#x628;&#x632;&#x646; (&#x645;&#x646;&#x648;&#x6cc; Start &#x6cc;&#x627; /link).</li>
          <li>&#x647;&#x645;&#x6cc;&#x646; &#x635;&#x641;&#x62d;&#x647; &#x62f;&#x627;&#x62e;&#x644; &#x62a;&#x644;&#x6af;&#x631;&#x627;&#x645; &#x628;&#x627;&#x632; &#x645;&#x6cc;‌&#x634;&#x648;&#x62f; &#x648; &#x648;&#x631;&#x648;&#x62f; &#x628;&#x6cc;‌&#x635;&#x62f;&#x627; &#x627;&#x646;&#x62c;&#x627;&#x645; &#x645;&#x6cc;‌&#x634;&#x648;&#x62f;.</li>
        </ol>
        <button
          type="button"
          onClick={() => void copyBotLink()}
          className="mx-auto mt-1 w-full max-w-[240px] rounded-lg bg-zinc-800 px-3 py-2 text-[11px] font-bold text-zinc-200 ring-1 ring-zinc-700 hover:bg-zinc-700"
        >
          {copied ? "\u0644\u06cc\u0646\u06a9 \u0631\u0628\u0627\u062a \u06a9\u067e\u06cc \u0634\u062f ✓" : "\u06a9\u067e\u06cc \u0644\u06cc\u0646\u06a9 \u0631\u0628\u0627\u062a \u062a\u0644\u06af\u0631\u0627\u0645"}
        </button>
      </div>

      <p className="flex items-center justify-center gap-1.5 text-[10px] text-zinc-600">
        <ShieldCheck className="size-3" aria-hidden />
        &#x646;&#x634;&#x633;&#x62a; &#x62a;&#x648; &#x641;&#x642;&#x637; &#x631;&#x648;&#x6cc; &#x633;&#x631;&#x648;&#x631; &#x631;&#x633;&#x645;&#x6cc; &#x6a9;&#x631;&#x646;&#x644; &#x633;&#x627;&#x62e;&#x62a;&#x647; &#x645;&#x6cc;‌&#x634;&#x648;&#x62f; — &#x628;&#x62f;&#x648;&#x646; &#x631;&#x645;&#x632;&#x60c; &#x628;&#x62f;&#x648;&#x646; &#x631;&#x6cc;&#x633;&#x6a9;.
      </p>
    </div>
  );
}

/** Slim banner atop pages in experimental browser mode. */
export function PreviewBanner() {
  const { exitPreview } = useSession();
  return (
    <div className="flex flex-wrap items-center justify-between gap-2 rounded-xl border border-gold-400/35 bg-gold-400/10 px-3 py-2 text-[11px] leading-5 text-gold-300">
      <span>
        🧪 <strong className="font-bold">&#x62d;&#x627;&#x644;&#x62a; &#x645;&#x631;&#x648;&#x631;&#x6af;&#x631; &#x622;&#x632;&#x645;&#x627;&#x6cc;&#x634;&#x6cc;</strong> — \u0641\u0642\u0637 \u0628\u062e\u0634‌\u0647\u0627\u06cc
        &#x639;&#x645;&#x648;&#x645;&#x6cc; &#x641;&#x639;&#x627;&#x644; &#x627;&#x633;&#x62a;&#x61b; &#x628;&#x631;&#x627;&#x6cc; &#x62e;&#x631;&#x6cc;&#x62f;&#x60c; &#x641;&#x631;&#x648;&#x634; &#x648; &#x627;&#x633;&#x62a;&#x648;&#x62f;&#x6cc;&#x648;&#x6cc; AI &#x627;&#x632; &#x62f;&#x627;&#x62e;&#x644; &#x62a;&#x644;&#x6af;&#x631;&#x627;&#x645; &#x648;&#x627;&#x631;&#x62f; &#x634;&#x648;.
      </span>
      <button
        type="button"
        onClick={exitPreview}
        className="shrink-0 rounded-lg bg-gold-400/15 px-2.5 py-1 font-bold text-gold-300 ring-1 ring-gold-400/40 hover:bg-gold-400/25"
      >
        &#x62a;&#x644;&#x627;&#x634; &#x628;&#x631;&#x627;&#x6cc; &#x648;&#x631;&#x648;&#x62f; &#x648;&#x627;&#x642;&#x639;&#x6cc;
      </button>
    </div>
  );
}

const PERSONA_META: Record<
  DemoPersona,
  { label: string; tag: string; href: string }
> = {
  creator: { label: "\u0633\u0627\u0632\u0646\u062f\u0647", tag: "#1001", href: "/builder" },
  buyer: { label: "\u062e\u0631\u06cc\u062f\u0627\u0631", tag: "#2002", href: "/market" },
  affiliate: { label: "\u0628\u0627\u0632\u0627\u0631\u06cc\u0627\u0628", tag: "#3003", href: "/market" },
};

/**
 * Compact demo-persona switcher bar — shown only for demo sessions;
 * the founder can switch between creator/buyer/marketer with one click mid-test.
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
      /* Demo sign-in errors surface in the hook itself and a page-level toast */
    } finally {
      setBusy(null);
    }
  };

  return (
    <div className="flex flex-wrap items-center gap-1.5 rounded-xl border border-royal-500/30 bg-royal-500/10 px-2.5 py-2 text-[11px]">
      <span className="font-bold text-royal-300">
        🧬 &#x62d;&#x627;&#x644;&#x62a; &#x62a;&#x633;&#x62a;:
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
