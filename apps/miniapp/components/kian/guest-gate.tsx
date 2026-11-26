"use client";

/**
 * MAIRON mini-app — guest gate: a clean "Quick experimental sign-in / Browser"
 * banner + one-click demo persona sign-in (only when the kernel is in dev mode)
 * + a transparent guide to connecting from inside Telegram. Replaces the locked
 * screen and dry technical error.
 *
 * Real sign-in still goes only through Telegram's signed initData and the kernel
 * HMAC; browser mode opens only public endpoints, and demo sign-in works only
 * when the server is up with KIAN_DEV_MODE=1.
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
  title = "Sign-in from inside Telegram is required for this section",
  hint = "Your MAIRON identity is built only from Telegram's HMAC-signed initData verified by the kernel — we never ask for a password and never fabricate one in the browser.",
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
        title: "Experimental sign-in failed",
        description: friendlyError(err),
        variant: "destructive",
      });
    } finally {
      setPersonaBusy(null);
    }
  };

  const personas: { key: DemoPersona; label: string }[] = [
    { key: "creator", label: "🎨 One-click: Creator (#1001)" },
    { key: "buyer", label: "🛒 One-click: Buyer (#2002)" },
    { key: "affiliate", label: "🔗 One-click: Affiliate (#3003)" },
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
          🧪 Quick experimental sign-in / Browser
        </button>
      )}

      {/* One-click demo persona sign-in — the full creator→buyer→marketer loop in the browser */}
      {whatWorksInPreview && (
        <div className="flex w-full flex-col gap-2 rounded-xl bg-zinc-950/70 p-3 ring-1 ring-royal-500/30">
          <p className="text-[11px] font-bold leading-5 text-royal-300">
            🧬 Sign in with a demo persona (full local test — build, buy and withdraw):
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
            With these three personas you can test the whole loop in the browser:
            build an app → publish → referral link → buy → escrow → withdraw crypto.
          </p>
        </div>
      )}
      {whatWorksInPreview && (
        <p className="text-[11px] leading-5 text-zinc-500">
          In this mode you can see the market, product previews and the national economy;
          buying, selling and the AI studio run only inside Telegram.
        </p>
      )}

      {/* Transparent guide to connecting for real */}
      <div className="mt-1 flex flex-col gap-2 rounded-xl bg-zinc-950/70 p-3 text-left ring-1 ring-zinc-800">
        <div className="flex items-center gap-2 text-[11px] font-bold text-gold-400">
          <Wifi className="size-3.5" aria-hidden />
          Real sign-in in 3 steps:
        </div>
        <ol className="list-inside list-decimal space-y-1 text-left text-[11px] leading-5 text-zinc-400">
          <li>
            Open the MAIRON bot in Telegram: <code className="font-mono text-zinc-300">@{botUsername()}</code>
          </li>
          <li>Tap the “Open the mini-app” button (the Start menu, or /link).</li>
          <li>The same page opens inside Telegram and sign-in happens silently.</li>
        </ol>
        <button
          type="button"
          onClick={() => void copyBotLink()}
          className="mx-auto mt-1 w-full max-w-[240px] rounded-lg bg-zinc-800 px-3 py-2 text-[11px] font-bold text-zinc-200 ring-1 ring-zinc-700 hover:bg-zinc-700"
        >
          {copied ? "Bot link copied ✓" : "Copy the Telegram bot link"}
        </button>
      </div>

      <p className="flex items-center justify-center gap-1.5 text-[10px] text-zinc-600">
        <ShieldCheck className="size-3" aria-hidden />
        Your session is built only on the official kernel server — no passwords, no risk.
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
        🧪 <strong className="font-bold">Experimental browser mode</strong> — only public sections
        are active. For buying, selling and the AI studio, enter from inside Telegram.
      </span>
      <button
        type="button"
        onClick={exitPreview}
        className="shrink-0 rounded-lg bg-gold-400/15 px-2.5 py-1 font-bold text-gold-300 ring-1 ring-gold-400/40 hover:bg-gold-400/25"
      >
        Try real sign-in
      </button>
    </div>
  );
}

const PERSONA_META: Record<
  DemoPersona,
  { label: string; tag: string; href: string }
> = {
  creator: { label: "Creator", tag: "#1001", href: "/builder" },
  buyer: { label: "Buyer", tag: "#2002", href: "/market" },
  affiliate: { label: "Affiliate", tag: "#3003", href: "/market" },
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
        🧬 Test mode:
        {" "}
        <strong className="text-gold-400">
          {PERSONA_META[activePersona].label} {PERSONA_META[activePersona].tag}
        </strong>
      </span>
      <span className="ml-auto flex flex-wrap gap-1">
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
