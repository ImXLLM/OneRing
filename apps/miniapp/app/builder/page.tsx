"use client";

/**
 * KIAN miniapp — the AI HTML product builder studio (Phase 5).
 *
 * The full cycle on one page with transparent three-step navigation:
 * 1) chat and ideas — multi-turn chat + 4 ready-made one-tap idea chips
 * 2) live and safe preview inside `HtmlPreviewFrame` (`sandbox="allow-scripts"`)
 * 3) pricing, referral commission and one-click publishing + a Telegram viral link
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
  Share2,
  Sparkles,
  Zap,
} from "lucide-react";
import { MiniShell } from "@/components/kian/mini-shell";
import { HtmlPreviewFrame } from "@/components/kian/html-preview-frame";
import { GuestGate, PreviewBanner } from "@/components/kian/guest-gate";
import { useSession } from "@/components/kian/session";
import { friendlyError, friendlyErrorTitle } from "@/lib/friendly";
import { sanitizePartialStudioHtml } from "@/lib/studio-html-guard";
import { haptics, setSwipeBehavior } from "@/lib/kian-tma-bridge";
import {
  api,
  ApiError,
  type BuilderSessionSummary,
  type BuilderSessionView,
  type PublishBuilderResponse,
} from "@kian/sdk";
import {
  buildTelegramViralLink,
  buildViralShareLinks,
  formatAffiliatePercent,
} from "@/lib/referral";

const AFFILIATE_PRESETS = [1000, 1500, 2000, 2500, 3000, 4000, 5000] as const;

/** Ready one-tap ideas — the user never faces an empty chat. */
const STARTER_PROMPTS: { emoji: string; label: string; prompt: string }[] = [
  {
    emoji: "📊",
    label: "\u0645\u0627\u0634\u06cc\u0646‌\u062d\u0633\u0627\u0628 \u0645\u062f\u06cc\u0631\u06cc\u062a \u0631\u06cc\u0633\u06a9 \u0648 \u0633\u0648\u062f \u062a\u0631\u06cc\u062f",
    prompt:
      "\u06cc\u06a9 \u0645\u0627\u0634\u06cc\u0646‌\u062d\u0633\u0627\u0628 \u0645\u062f\u06cc\u0631\u06cc\u062a \u0631\u06cc\u0633\u06a9 \u0648 \u0633\u0648\u062f \u0628\u0631\u0627\u06cc \u062a\u0631\u06cc\u062f\u0631 \u0628\u0633\u0627\u0632: \u0648\u0631\u0648\u062f\u06cc \u0633\u0631\u0645\u0627\u06cc\u0647\u060c \u062f\u0631\u0635\u062f \u0631\u06cc\u0633\u06a9 \u0647\u0631 \u0645\u0639\u0627\u0645\u0644\u0647\u060c \u062d\u062f \u0636\u0631\u0631 \u0648 \u062d\u062f \u0633\u0648\u062f\u060c \u0645\u062d\u0627\u0633\u0628\u0647\u0654 \u062e\u0648\u062f\u06a9\u0627\u0631 \u062d\u062c\u0645 \u067e\u0648\u0632\u06cc\u0634\u0646 \u0648 \u0646\u0633\u0628\u062a \u0631\u06cc\u0633\u06a9/\u0631\u06cc\u0648\u0627\u0631\u062f \u0628\u0627 \u06a9\u0627\u0631\u062a‌\u0647\u0627\u06cc \u0631\u0646\u06af\u06cc \u0633\u0628\u0632 \u0648 \u0642\u0631\u0645\u0632\u060c \u0627\u0639\u062f\u0627\u062f \u0641\u0627\u0631\u0633\u06cc \u0648 \u062a\u0645 \u062a\u06cc\u0631\u0647\u0654 \u062d\u0631\u0641\u0647‌\u0627\u06cc.",
  },
  {
    emoji: "🎯",
    label: "\u062f\u0627\u0634\u0628\u0648\u0631\u062f \u067e\u06cc\u06af\u06cc\u0631\u06cc \u0639\u0627\u062f\u062a‌\u0647\u0627 \u0648 \u062a\u0633\u06a9‌\u0647\u0627",
    prompt:
      "\u06cc\u06a9 \u062f\u0627\u0634\u0628\u0648\u0631\u062f \u0647\u0648\u0634\u0645\u0646\u062f \u067e\u06cc\u06af\u06cc\u0631\u06cc \u0639\u0627\u062f\u062a‌\u0647\u0627 \u0648 \u062a\u0633\u06a9‌\u0647\u0627\u06cc \u0631\u0648\u0632\u0627\u0646\u0647 \u0628\u0633\u0627\u0632: \u0627\u0641\u0632\u0648\u062f\u0646 \u0639\u0627\u062f\u062a \u0648 \u062a\u0633\u06a9\u060c \u062a\u06cc\u06a9 \u0632\u062f\u0646 \u0631\u0648\u0632\u0627\u0646\u0647\u060c \u0634\u0645\u0627\u0631\u0634 \u0631\u0648\u0632\u0647\u0627\u06cc \u067e\u06cc\u0648\u0633\u062a\u0647 (\u0627\u0633\u062a\u0631\u06cc\u06a9)\u060c \u0646\u0648\u0627\u0631 \u067e\u06cc\u0634\u0631\u0641\u062a \u0647\u0641\u062a\u0647 \u0648 \u0630\u062e\u06cc\u0631\u0647\u0654 \u062e\u0648\u062f\u06a9\u0627\u0631 \u062f\u0631 localStorage\u061b \u0637\u0631\u0627\u062d\u06cc \u0645\u06cc\u0646\u06cc\u0645\u0627\u0644 \u062a\u06cc\u0631\u0647 \u0628\u0627 \u0627\u0646\u06cc\u0645\u06cc\u0634\u0646 \u0645\u0644\u0627\u06cc\u0645.",
  },
  {
    emoji: "🃏",
    label: "\u06a9\u0627\u0631\u062a \u0648\u06cc\u0632\u06cc\u062a \u0648 \u0644\u0646\u062f\u06cc\u0646\u06af \u0646\u0626\u0648\u0646\u06cc \u06a9\u0631\u06cc\u067e\u062a\u0648",
    prompt:
      "\u06cc\u06a9 \u0644\u0646\u062f\u06cc\u0646\u06af \u067e\u06cc\u062c \u0646\u0626\u0648\u0646\u06cc \u0628\u0631\u0627\u06cc \u0628\u0631\u0646\u062f \u06a9\u0631\u06cc\u067e\u062a\u0648 \u0628\u0633\u0627\u0632: \u0647\u06cc\u0631\u0648 \u0628\u0627 \u06af\u0631\u0627\u062f\u06cc\u0627\u0646 \u0628\u0646\u0641\u0634-\u0622\u0628\u06cc \u0648 \u0627\u0641\u06a9\u062a \u06af\u0644\u0648\u060c \u06a9\u0627\u0631\u062a \u0648\u06cc\u0632\u06cc\u062a \u062f\u06cc\u062c\u06cc\u062a\u0627\u0644 \u0628\u0627 \u062f\u06a9\u0645\u0647‌\u0647\u0627\u06cc \u0634\u0628\u06a9\u0647‌\u0647\u0627\u06cc \u0627\u062c\u062a\u0645\u0627\u0639\u06cc\u060c \u062a\u0627\u06cc\u067e\u0648\u06af\u0631\u0627\u0641\u06cc \u0628\u0632\u0631\u06af \u0648 \u0628\u062e\u0634 \u0645\u0639\u0631\u0641\u06cc \u062e\u062f\u0645\u0627\u062a\u061b \u06a9\u0627\u0645\u0644\u0627\u064b \u0631\u06cc\u0633\u067e\u0627\u0646\u0633\u06cc\u0648 \u0628\u0631\u0627\u06cc \u0645\u0648\u0628\u0627\u06cc\u0644.",
  },
  {
    emoji: "🧮",
    label: "\u0627\u0628\u0632\u0627\u0631 \u0645\u062d\u0627\u0633\u0628\u0647\u0654 \u0633\u0648\u062f \u0645\u0631\u06a9\u0628 \u0648 \u0648\u0627\u0645",
    prompt:
      "\u06cc\u06a9 \u0627\u0628\u0632\u0627\u0631 \u0645\u062d\u0627\u0633\u0628\u0647\u0654 \u0633\u0648\u062f \u0645\u0631\u06a9\u0628 \u0648 \u0627\u0642\u0633\u0627\u0637 \u0648\u0627\u0645 \u0628\u0633\u0627\u0632: \u062f\u0648 \u062a\u0628 (\u0633\u0648\u062f \u0645\u0631\u06a9\u0628 \u0628\u0627 \u0646\u0645\u0648\u062f\u0627\u0631 \u0631\u0634\u062f \u0633\u0631\u0645\u0627\u06cc\u0647\u060c \u0648\u0627\u0645 \u0628\u0627 \u0642\u0633\u0637 \u0645\u0627\u0647\u0627\u0646\u0647)\u060c \u0627\u0633\u0644\u0627\u06cc\u062f\u0631 \u0628\u0631\u0627\u06cc \u0646\u0631\u062e \u0633\u0648\u062f \u0648 \u0645\u062f\u062a\u060c \u062c\u062f\u0648\u0644 \u0633\u0627\u0644\u0627\u0646\u0647 \u0648 \u0627\u0639\u062f\u0627\u062f \u0641\u0627\u0631\u0633\u06cc\u061b \u062a\u0645 \u062a\u06cc\u0631\u0647\u0654 \u0644\u0648\u06a9\u0633 \u0628\u0627 \u0637\u0644\u0627\u06cc\u06cc.",
  },
];

/** Three-step status bar at the top of the studio. */
function StudioSteps({ current }: { current: 1 | 2 | 3 }) {
  const steps = [
    { n: 1 as const, label: "\u06af\u0641\u062a\u06af\u0648 \u0648 \u0627\u06cc\u062f\u0647" },
    { n: 2 as const, label: "\u067e\u06cc\u0634‌\u0646\u0645\u0627\u06cc\u0634 \u0632\u0646\u062f\u0647" },
    { n: 3 as const, label: "\u0642\u06cc\u0645\u062a‌\u06af\u0630\u0627\u0631\u06cc \u0648 \u0644\u06cc\u0646\u06a9 \u0648\u0627\u06cc\u0631\u0627\u0644" },
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

  // one-click publish form (pre-filled from the AI suggestions)
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [priceDin, setPriceDin] = useState("200");
  const [affiliateBps, setAffiliateBps] = useState<number>(1500);
  const [category, setCategory] = useState("html-app");
  const [publishing, setPublishing] = useState(false);
  const [published, setPublished] = useState<PublishBuilderResponse | null>(null);
  const [copiedLink, setCopiedLink] = useState(false);
  const [copiedShare, setCopiedShare] = useState(false);

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
          title: "\u0628\u0627\u0631\u06af\u06cc\u0631\u06cc \u062c\u0644\u0633\u0647 \u0646\u0627\u0645\u0648\u0641\u0642 \u0628\u0648\u062f",
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
          title: "\u0627\u0633\u062a\u0648\u062f\u06cc\u0648\u06cc \u062a\u0627\u0632\u0647 \u0622\u0645\u0627\u062f\u0647 \u0627\u0633\u062a",
          description: "\u06cc\u06a9\u06cc \u0627\u0632 \u0627\u06cc\u062f\u0647‌\u0647\u0627\u06cc \u0622\u0645\u0627\u062f\u0647 \u0631\u0627 \u0628\u0632\u0646 \u06cc\u0627 \u0627\u06cc\u062f\u0647\u0654 \u062e\u0648\u062f\u062a \u0631\u0627 \u0628\u0646\u0648\u06cc\u0633 ✨",
          variant: "success",
        });
      } catch (err) {
        toast({
          title: "\u0633\u0627\u062e\u062a \u062c\u0644\u0633\u0647\u0654 \u062c\u062f\u06cc\u062f \u0646\u0627\u0645\u0648\u0641\u0642 \u0628\u0648\u062f",
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
        /* The user can start a new session manually */
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

  // lock Telegram’s vertical drag only inside the studio (port of tdesktop-telegram-gems):
  // touch dragging inside the studio canvas never closes the mini-app; on leaving
  // the studio, Telegram’s vertical drag returns (KB convention: restore after interaction).
  useEffect(() => {
    setSwipeBehavior(false);
    return () => setSwipeBehavior(true);
  }, []);

  if (state.phase === "loading") {
    return (
      <MiniShell>
        <div className="flex items-center justify-center gap-3 py-24 text-zinc-500">
          <Loader2 className="size-5 animate-spin" aria-hidden />
          <span className="text-sm">&#x62f;&#x631; &#x62d;&#x627;&#x644; &#x627;&#x62a;&#x635;&#x627;&#x644; &#x628;&#x647; &#x627;&#x633;&#x62a;&#x648;&#x62f;&#x6cc;&#x648; AI…</span>
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
            title="&#x627;&#x633;&#x62a;&#x648;&#x62f;&#x6cc;&#x648; AI &#x62f;&#x627;&#x62e;&#x644; &#x62a;&#x644;&#x6af;&#x631;&#x627;&#x645; &#x641;&#x639;&#x627;&#x644; &#x645;&#x6cc;‌&#x634;&#x648;&#x62f;"
            hint="&#x628;&#x631;&#x627;&#x6cc; &#x633;&#x627;&#x62e;&#x62a; &#x645;&#x6cc;&#x646;&#x6cc;‌&#x627;&#x67e;‌&#x647;&#x627;&#x6cc; HTML &#x62a;&#x639;&#x627;&#x645;&#x644;&#x6cc; &#x648; &#x641;&#x631;&#x648;&#x634; &#x628;&#x627; &#x644;&#x6cc;&#x646;&#x6a9; &#x631;&#x641;&#x631;&#x627;&#x644;&#x60c; &#x646;&#x634;&#x633;&#x62a; &#x634;&#x647;&#x631;&#x648;&#x646;&#x62f;&#x6cc; &#x634;&#x645;&#x627; &#x628;&#x627;&#x6cc;&#x62f; &#x627;&#x632; &#x62f;&#x627;&#x62e;&#x644; &#x62a;&#x644;&#x6af;&#x631;&#x627;&#x645; &#x627;&#x62d;&#x631;&#x627;&#x632; &#x634;&#x648;&#x62f; — &#x628;&#x62f;&#x648;&#x646; &#x631;&#x645;&#x632; &#x648; &#x641;&#x642;&#x637; &#x628;&#x627; &#x627;&#x645;&#x636;&#x627;&#x6cc; &#x627;&#x645;&#x646; &#x62a;&#x644;&#x6af;&#x631;&#x627;&#x645;."
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
      text || (forceGenerate ? "\u0644\u0637\u0641\u0627\u064b \u0647\u0645\u06cc\u0646 \u0627\u0644\u0627\u0646 \u0646\u0633\u062e\u0647\u0654 \u06a9\u0627\u0645\u0644 \u0648 \u062a\u0639\u0627\u0645\u0644\u06cc HTML \u0627\u06cc\u0646 \u0645\u062d\u0635\u0648\u0644 \u0631\u0627 \u0628\u0633\u0627\u0632." : "");
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
          title: "\u0646\u0633\u062e\u0647\u0654 \u0632\u0646\u062f\u0647\u0654 HTML \u0622\u0645\u0627\u062f\u0647 \u0634\u062f 🎉",
          description: "\u067e\u06cc\u0634‌\u0646\u0645\u0627\u06cc\u0634 \u0631\u0627 \u062a\u0633\u062a \u06a9\u0646\u061b \u0627\u06af\u0631 \u062f\u0648\u0633\u062a\u0634 \u062f\u0627\u0634\u062a\u06cc \u0628\u0631\u0648 \u0633\u0631\u0627\u063a \u0642\u06cc\u0645\u062a‌\u06af\u0630\u0627\u0631\u06cc.",
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

  /** Ready idea chip — if the chat is still empty, build directly; otherwise fill the input. */
  const useStarterPrompt = (prompt: string) => {
    if (!active || sending) return;
    const chatIsEmpty = !active.messages.some((m) => m.role === "user");
    if (chatIsEmpty) {
      void sendMessage(true, prompt);
    } else {
      setDraft(prompt);
      draftRef.current?.focus();
      toast({ title: "\u0627\u06cc\u062f\u0647 \u062f\u0631 \u06a9\u0627\u062f\u0631 \u067e\u06cc\u0627\u0645 \u0646\u0634\u0633\u062a ✍️", description: "\u06a9\u0645\u06cc \u0648\u06cc\u0631\u0627\u06cc\u0634\u0634 \u06a9\u0646 \u0648 \u0628\u0641\u0631\u0633\u062a." });
    }
  };

  const handlePublish = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!active) return;
    const numericPrice = Math.max(1, Math.floor(Number(priceDin) || 0));
    if (!numericPrice) {
      toast({ title: "\u0642\u06cc\u0645\u062a \u0645\u0639\u062a\u0628\u0631 \u0628\u0647 \u062f\u06cc\u06cc\u0646 \u0648\u0627\u0631\u062f \u06a9\u0646\u06cc\u062f", variant: "destructive" });
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
          ? "\u0627\u06cc\u0646 \u0645\u062d\u0635\u0648\u0644 \u0642\u0628\u0644\u0627\u064b \u0645\u0646\u062a\u0634\u0631 \u0634\u062f\u0647 \u0627\u0633\u062a"
          : "\u0645\u062d\u0635\u0648\u0644 \u0628\u0627 \u0645\u0648\u0641\u0642\u06cc\u062a \u062f\u0631 \u0628\u0627\u0632\u0627\u0631 \u06a9\u06cc\u0627\u0646 \u0645\u0646\u062a\u0634\u0631 \u0634\u062f! 🚀",
        description: "\u0644\u06cc\u0646\u06a9 \u0641\u0631\u0648\u0634 \u0648 \u0631\u0641\u0631\u0627\u0644 \u0634\u0645\u0627 \u0622\u0645\u0627\u062f\u0647\u0654 \u0627\u0634\u062a\u0631\u0627\u06a9‌\u06af\u0630\u0627\u0631\u06cc \u0627\u0633\u062a.",
        variant: "success",
      });
    } catch (err) {
      toast({
        title: "\u0627\u0646\u062a\u0634\u0627\u0631 \u0645\u062d\u0635\u0648\u0644 \u0646\u0627\u0645\u0648\u0641\u0642 \u0628\u0648\u062f",
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
      toast({ title: "\u0644\u06cc\u0646\u06a9 \u0641\u0631\u0648\u0634 \u0648 \u0631\u0641\u0631\u0627\u0644 \u06a9\u067e\u06cc \u0634\u062f ✅", variant: "success" });
      window.setTimeout(() => setCopiedLink(false), 2000);
    } catch {
      toast({
        title: "\u06a9\u067e\u06cc \u062e\u0648\u062f\u06a9\u0627\u0631 \u0645\u062c\u0627\u0632 \u0646\u0628\u0648\u062f",
        description: viralLink.telegramUrl,
        variant: "destructive",
      });
    }
  };

  // one-click viral link copy via the Telegram share sheet (port of tdesktop-telegram-gems —
  // `buildViralShareLinks` / `p_<productId>_r_<refTgId>`):
  const copyViralShareLink = async () => {
    if (!viralProductId) return;
    const links = buildViralShareLinks(viralProductId, me.user.telegramId);
    try {
      await navigator.clipboard.writeText(links.shareUrl);
      setCopiedShare(true);
      haptics.success();
      toast({ title: "\u0644\u06cc\u0646\u06a9 \u0648\u0627\u06cc\u0631\u0627\u0644 \u06a9\u067e\u06cc \u0634\u062f — \u0628\u0641\u0631\u0633\u062a \u0648 \u067e\u0627\u062f\u0627\u0634 \u0628\u06af\u06cc\u0631 🚀", variant: "success" });
      window.setTimeout(() => setCopiedShare(false), 2000);
    } catch {
      // clipboard unavailable — the Telegram share sheet opens
      window.location.href = links.shareUrl;
    }
  };

  const hasHtml = Boolean(active?.generatedHtml.trim());
  // zero-flicker guard (port of OpenIntelligentUI): the preview never sees half-way HTML with
  // an open tag/open script/incomplete entity; the download version is always a complete document.
  const previewHtml = useMemo(
    () => sanitizePartialStudioHtml(active?.generatedHtml ?? ""),
    [active?.generatedHtml],
  );
  const currentStep: 1 | 2 | 3 = viralProductId ? 3 : hasHtml ? 2 : 1;

  return (
    <MiniShell>
      <div className="flex flex-col gap-5">
        {/* Header + session selection/creation */}
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div>
            <h1 className="flex items-center gap-2 text-lg font-black">
              <Sparkles className="size-5 text-gold-400" aria-hidden />
              &#x627;&#x633;&#x62a;&#x648;&#x62f;&#x6cc;&#x648; AI (&#x633;&#x627;&#x62e;&#x62a; &#x645;&#x62d;&#x635;&#x648;&#x644;)
            </h1>
            <p className="mt-0.5 text-xs text-zinc-400">
              &#x6af;&#x641;&#x62a;&#x6af;&#x648; &#x6a9;&#x646;&#x60c; &#x641;&#x627;&#x6cc;&#x644; HTML &#x62a;&#x639;&#x627;&#x645;&#x644;&#x6cc; &#x628;&#x633;&#x627;&#x632; &#x648; &#x628;&#x627; &#x6cc;&#x6a9; &#x6a9;&#x644;&#x6cc;&#x6a9; &#x647;&#x645;&#x631;&#x627;&#x647; &#x644;&#x6cc;&#x646;&#x6a9; &#x631;&#x641;&#x631;&#x627;&#x644; &#x628;&#x641;&#x631;&#x648;&#x634;.
            </p>
          </div>
          <button
            type="button"
            onClick={() => void startNewSession(token)}
            disabled={loadingSession}
            className="flex items-center gap-1.5 rounded-xl bg-royal-500/20 px-3 py-2 text-xs font-bold text-gold-400 ring-1 ring-gold-400/30 hover:bg-royal-500/30 disabled:opacity-50"
          >
            <Plus className="size-4" aria-hidden />
            &#x62c;&#x644;&#x633;&#x647;&#x654; &#x62c;&#x62f;&#x6cc;&#x62f;
          </button>
        </div>

        {/* Three-step status bar */}
        <StudioSteps current={currentStep} />

        {/* Previous sessions bar */}
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
                  {s.title.trim() || "\u062c\u0644\u0633\u0647\u0654 \u0628\u062f\u0648\u0646 \u0639\u0646\u0648\u0627\u0646"}
                  {s.publishedProductId ? " ✓" : s.hasHtml ? " • HTML" : ""}
                </button>
              );
            })}
          </div>
        )}

        {/* Section 1: interview-and-build chat */}
        <section className="flex flex-col gap-3 rounded-2xl border border-royal-500/20 bg-zinc-900/60 p-4">
          <div className="flex items-center justify-between gap-2">
            <h2 className="flex items-center gap-1.5 text-sm font-bold text-zinc-200">
              <MessageCircle className="size-4 text-royal-300" aria-hidden />
              &#x6af;&#x641;&#x62a;&#x6af;&#x648;&#x6cc; &#x637;&#x631;&#x627;&#x62d;&#x6cc; &#x648; &#x633;&#x627;&#x62e;&#x62a;
            </h2>
            <div className="flex items-center gap-2 text-[11px] text-zinc-400">
              {active && (
                <span className="rounded-full bg-zinc-800 px-2.5 py-0.5 font-semibold text-zinc-300">
                  &#x645;&#x631;&#x62d;&#x644;&#x647;:{" "}
                  {active.stage === "published"
                    ? "\u0645\u0646\u062a\u0634\u0631\u0634\u062f\u0647"
                    : active.stage === "ready"
                      ? "\u0622\u0645\u0627\u062f\u0647\u0654 \u0627\u0646\u062a\u0634\u0627\u0631"
                      : "\u06a9\u0634\u0641 \u0627\u06cc\u062f\u0647"}
                </span>
              )}
              {lastMeta && (
                <span className="rounded-full bg-royal-500/15 px-2 py-0.5 text-royal-300">
                  {lastMeta.usedFallback
                    ? "\u0645\u0648\u0644\u062f \u062f\u0627\u062e\u0644\u06cc"
                    : `${lastMeta.provider ?? "AI"} • ${lastMeta.totalTokens} tok`}
                </span>
              )}
            </div>
          </div>

          <div className="flex max-h-80 flex-col gap-2.5 overflow-y-auto rounded-xl border border-zinc-800 bg-zinc-950/70 p-3">
            {loadingSession && !active ? (
              <div className="flex items-center justify-center gap-2 py-8 text-xs text-zinc-500">
                <Loader2 className="size-4 animate-spin" aria-hidden />
                &#x62f;&#x631; &#x62d;&#x627;&#x644; &#x622;&#x645;&#x627;&#x62f;&#x647;‌&#x633;&#x627;&#x632;&#x6cc; &#x627;&#x633;&#x62a;&#x648;&#x62f;&#x6cc;&#x648;…
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
                &#x62f;&#x631; &#x62d;&#x627;&#x644; &#x637;&#x631;&#x627;&#x62d;&#x6cc;&#x650; &#x6a9;&#x62f;&#x650; HTML… (&#x645;&#x639;&#x645;&#x648;&#x644;&#x627;&#x64b; &#x686;&#x646;&#x62f; &#x62b;&#x627;&#x646;&#x6cc;&#x647;)
              </div>
            )}
            <div ref={chatEndRef} />
          </div>

          {/* Ready one-tap idea chips — no one faces an empty chat */}
          <div className="flex flex-col gap-1.5">
            <p className="text-[11px] font-bold text-zinc-400">
              &#x6cc;&#x627; &#x6cc;&#x6a9;&#x6cc; &#x627;&#x632; &#x627;&#x6cc;&#x646; &#x627;&#x6cc;&#x62f;&#x647;‌&#x647;&#x627;&#x6cc; &#x622;&#x645;&#x627;&#x62f;&#x647; &#x631;&#x627; &#x628;&#x632;&#x646;:
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
                    aria-label={`\u0633\u0627\u062e\u062a \u0641\u0648\u0631\u06cc: ${p.label}`}
                    title="\u0633\u0627\u062e\u062a \u0641\u0648\u0631\u06cc \u0647\u0645\u06cc\u0646 \u0627\u06cc\u062f\u0647"
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
              placeholder="\u0627\u06cc\u062f\u0647‌\u0627\u062a \u0631\u0627 \u0628\u0646\u0648\u06cc\u0633 \u06cc\u0627 \u0628\u062e\u0648\u0627\u0647 \u062a\u063a\u06cc\u06cc\u0631 \u062f\u0647\u062f (\u0645\u062b\u0644\u0627\u064b: \u0645\u0627\u0634\u06cc\u0646‌\u062d\u0633\u0627\u0628 \u0633\u0648\u062f \u06a9\u0631\u06cc\u067e\u062a\u0648 \u0628\u0627 \u062a\u0645 \u0646\u0626\u0648\u0646 \u062a\u06cc\u0631\u0647)…"
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
                {sending ? "\u062f\u0631 \u062d\u0627\u0644 \u0637\u0631\u0627\u062d\u06cc…" : "\u0627\u0631\u0633\u0627\u0644 \u067e\u06cc\u0627\u0645"}
              </button>
              <button
                type="button"
                onClick={() => void sendMessage(true)}
                disabled={sending || !active}
                className="flex items-center gap-1.5 rounded-xl bg-amber-500/15 px-3.5 py-2.5 text-xs font-bold text-gold-400 ring-1 ring-gold-400/35 hover:bg-amber-500/25 disabled:opacity-40"
              >
                <Zap className="size-4" aria-hidden />
                &#x633;&#x627;&#x62e;&#x62a; &#x641;&#x648;&#x631;&#x6cc; &#x646;&#x633;&#x62e;&#x647;&#x654; &#x627;&#x648;&#x644;
              </button>
            </div>
          </form>
        </section>

        {/* Section 2: live preview in the sandboxed iframe */}
        {hasHtml && active && (
          <section className="flex flex-col gap-2.5">
            <h2 className="text-sm font-bold text-zinc-200">
              &#x67e;&#x6cc;&#x634;‌&#x646;&#x645;&#x627;&#x6cc;&#x634; &#x632;&#x646;&#x62f;&#x647;&#x654; &#x645;&#x62d;&#x635;&#x648;&#x644; — &#x645;&#x631;&#x62d;&#x644;&#x647;&#x654; &#x6f2; &#x627;&#x632; &#x6f3;
            </h2>
            <HtmlPreviewFrame
              html={previewHtml}
              downloadHtml={active.generatedHtml}
              title={title || "\u067e\u06cc\u0634‌\u0646\u0645\u0627\u06cc\u0634 \u0632\u0646\u062f\u0647\u0654 \u0645\u062d\u0635\u0648\u0644"}
              allowDownload
              downloadFilename={`${title || active.id}.html`}
              badgeLabel="&#x67e;&#x6cc;&#x634;‌&#x646;&#x645;&#x627;&#x6cc;&#x634; &#x632;&#x646;&#x62f;&#x647;&#x654; &#x627;&#x633;&#x62a;&#x648;&#x62f;&#x6cc;&#x648;"
            />
          </section>
        )}

        {/* Section 3: pricing, referral commission and one-click publish form */}
        {hasHtml && active && (
          <section className="rounded-2xl border border-gold-400/25 bg-zinc-900/70 p-4">
            <h2 className="mb-3 flex items-center gap-2 text-sm font-bold text-gold-400">
              <Rocket className="size-4" aria-hidden />
              &#x642;&#x6cc;&#x645;&#x62a;‌&#x6af;&#x630;&#x627;&#x631;&#x6cc;&#x60c; &#x67e;&#x648;&#x631;&#x633;&#x627;&#x646;&#x62a; &#x645;&#x639;&#x631;&#x641;&#x6cc; &#x648; &#x627;&#x646;&#x62a;&#x634;&#x627;&#x631; &#x62f;&#x631; &#x628;&#x627;&#x632;&#x627;&#x631; — &#x645;&#x631;&#x62d;&#x644;&#x647;&#x654; &#x6f3; &#x627;&#x632; &#x6f3;
            </h2>
            <form onSubmit={(e) => void handlePublish(e)} className="grid gap-3">
              <label className="text-xs text-zinc-400">
                &#x639;&#x646;&#x648;&#x627;&#x646; &#x645;&#x62d;&#x635;&#x648;&#x644;
                <input
                  value={title}
                  onChange={(e) => setTitle(e.target.value)}
                  required
                  minLength={1}
                  maxLength={120}
                  placeholder="\u0639\u0646\u0648\u0627\u0646 \u062c\u0630\u0627\u0628 \u0645\u062d\u0635\u0648\u0644"
                  className="mt-1 w-full rounded-xl border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 outline-none focus:border-gold-400/50"
                />
              </label>

              <label className="text-xs text-zinc-400">
                &#x62a;&#x648;&#x636;&#x6cc;&#x62d;&#x627;&#x62a; &#x641;&#x631;&#x648;&#x634; (CAST)
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
                  &#x642;&#x6cc;&#x645;&#x62a; &#x641;&#x631;&#x648;&#x634; (Đ)
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
                  &#x67e;&#x627;&#x62f;&#x627;&#x634; &#x645;&#x639;&#x631;&#x641; ({formatAffiliatePercent(affiliateBps)})
                  <select
                    value={affiliateBps}
                    onChange={(e) => setAffiliateBps(Number(e.target.value))}
                    className="mt-1 w-full rounded-xl border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 outline-none focus:border-gold-400/50"
                  >
                    {AFFILIATE_PRESETS.map((bps) => (
                      <option key={bps} value={bps}>
                        {formatAffiliatePercent(bps)} &#x67e;&#x648;&#x631;&#x633;&#x627;&#x646;&#x62a; &#x631;&#x641;&#x631;&#x627;&#x644;
                      </option>
                    ))}
                  </select>
                </label>
              </div>

              <label className="text-xs text-zinc-400">
                &#x62f;&#x633;&#x62a;&#x647;‌&#x628;&#x646;&#x62f;&#x6cc;
                <input
                  value={category}
                  onChange={(e) => setCategory(e.target.value)}
                  maxLength={40}
                  placeholder="html-app"
                  className="mt-1 w-full rounded-xl border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 outline-none focus:border-gold-400/50"
                />
              </label>

              <p className="rounded-xl bg-zinc-950/80 px-3 py-2 text-[11px] leading-5 text-zinc-400 ring-1 ring-zinc-800">
                &#x62a;&#x642;&#x633;&#x6cc;&#x645; &#x62e;&#x648;&#x62f;&#x6a9;&#x627;&#x631; &#x62f;&#x641;&#x62a;&#x631; &#x62f;&#x648;&#x637;&#x631;&#x641;&#x647; &#x62f;&#x631; &#x644;&#x62d;&#x638;&#x647;&#x654; &#x62a;&#x633;&#x648;&#x6cc;&#x647;: &#x6f1;&#x6f0;&#x66a; &#x62e;&#x632;&#x627;&#x646;&#x647;&#x654; &#x6a9;&#x6cc;&#x627;&#x646; +{" "}
                <strong className="text-emerald-400">
                  {formatAffiliatePercent(affiliateBps)}
                </strong>{" "}
                &#x67e;&#x627;&#x62f;&#x627;&#x634; &#x645;&#x639;&#x631;&#x641; (&#x62f;&#x631; &#x62e;&#x631;&#x6cc;&#x62f;&#x647;&#x627;&#x6cc; &#x631;&#x641;&#x631;&#x627;&#x644;&#x6cc;) + &#x645;&#x627;&#x628;&#x642;&#x6cc; &#x62e;&#x627;&#x644;&#x635; &#x628;&#x647; &#x6a9;&#x6cc;&#x641; &#x634;&#x645;&#x627;.
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
                  ? "\u0646\u0645\u0627\u06cc\u0634 / \u062a\u0623\u06cc\u06cc\u062f \u0645\u062d\u0635\u0648\u0644 \u0645\u0646\u062a\u0634\u0631\u0634\u062f\u0647"
                  : "🚀 \u0627\u0646\u062a\u0634\u0627\u0631 \u062f\u0631 \u0628\u0627\u0632\u0627\u0631 \u0648 \u062f\u0631\u06cc\u0627\u0641\u062a \u0644\u06cc\u0646\u06a9 \u0641\u0631\u0648\u0634"}
              </button>
            </form>
          </section>
        )}

        {/* Section 4: Telegram viral and referral link box */}
        {viralLink && viralProductId && (
          <section className="flex flex-col gap-3 rounded-2xl border border-emerald-500/35 bg-emerald-950/20 p-4">
            <div className="flex items-center justify-between gap-2">
              <h2 className="text-sm font-black text-emerald-300">
                🎉 &#x644;&#x6cc;&#x646;&#x6a9; &#x622;&#x645;&#x627;&#x62f;&#x647;&#x654; &#x641;&#x631;&#x648;&#x634; &#x648; &#x631;&#x641;&#x631;&#x627;&#x644; &#x62a;&#x644;&#x6af;&#x631;&#x627;&#x645;
              </h2>
              <Link
                href={`/market?product=${encodeURIComponent(viralProductId)}`}
                className="flex items-center gap-1 text-xs font-bold text-gold-400 hover:underline"
              >
                <span>&#x646;&#x645;&#x627;&#x6cc;&#x634; &#x62f;&#x631; &#x628;&#x627;&#x632;&#x627;&#x631;</span>
                <ExternalLink className="size-3.5" aria-hidden />
              </Link>
            </div>
            <p className="text-xs leading-5 text-zinc-300">
              &#x627;&#x6cc;&#x646; &#x644;&#x6cc;&#x646;&#x6a9; &#x631;&#x627; &#x62f;&#x631; &#x6a9;&#x627;&#x646;&#x627;&#x644;&#x60c; &#x6af;&#x631;&#x648;&#x647; &#x6cc;&#x627; &#x67e;&#x6cc;‌&#x648;&#x6cc; &#x67e;&#x62e;&#x634; &#x6a9;&#x646;&#x61b; &#x647;&#x631; &#x6a9;&#x633;&#x6cc; &#x628;&#x627; &#x622;&#x646; &#x648;&#x627;&#x631;&#x62f; &#x634;&#x648;&#x62f;
              &#x67e;&#x6cc;&#x634;‌&#x646;&#x645;&#x627;&#x6cc;&#x634; &#x645;&#x62d;&#x635;&#x648;&#x644;&#x62a; &#x631;&#x627; &#x645;&#x6cc;‌&#x628;&#x6cc;&#x646;&#x62f; &#x648; &#x634;&#x646;&#x627;&#x633;&#x647;&#x654; &#x645;&#x639;&#x631;&#x641; &#x628;&#x647;‌&#x637;&#x648;&#x631; &#x62e;&#x648;&#x62f;&#x6a9;&#x627;&#x631; &#x631;&#x648;&#x6cc; &#x62e;&#x631;&#x6cc;&#x62f;&#x634; &#x62b;&#x628;&#x62a; &#x645;&#x6cc;‌&#x634;&#x648;&#x62f;.
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
                {copiedLink ? "\u06a9\u067e\u06cc \u0634\u062f" : "\u06a9\u067e\u06cc \u0644\u06cc\u0646\u06a9"}
              </button>
            </div>
            {/* One-click viral — Telegram share sheet with the referral commission payload
                (port of tdesktop-telegram-gems: buildViralShareLinks /
                p_<productId>_r_<refTgId>) */}
            <button
              type="button"
              onClick={() => void copyViralShareLink()}
              className="flex w-full items-center justify-center gap-2 rounded-xl bg-gradient-to-b from-emerald-400 to-emerald-600 px-4 py-3 text-sm font-black text-zinc-950 shadow-lg shadow-emerald-600/30 ring-1 ring-gold-400/40 hover:opacity-90"
            >
              {copiedShare ? (
                <Check className="size-4" aria-hidden />
              ) : (
                <Share2 className="size-4" aria-hidden />
              )}
              {copiedShare ? "\u0644\u06cc\u0646\u06a9 \u0648\u0627\u06cc\u0631\u0627\u0644 \u06a9\u067e\u06cc \u0634\u062f" : "\u06a9\u067e\u06cc \u0644\u06cc\u0646\u06a9 \u0648\u0627\u06cc\u0631\u0627\u0644 (\u06f2\u06f5\u066a \u067e\u0627\u062f\u0627\u0634)"}
            </button>
            <p className="flex flex-wrap items-center gap-1.5 rounded-xl bg-emerald-500/10 px-3 py-2 text-[11px] font-bold leading-5 text-emerald-300 ring-1 ring-emerald-400/25">
              💰 &#x628;&#x627; &#x647;&#x631; &#x62e;&#x631;&#x6cc;&#x62f; &#x627;&#x632; &#x627;&#x6cc;&#x646; &#x644;&#x6cc;&#x646;&#x6a9;{" "}
              {formatAffiliatePercent(published?.product.affiliateBps ?? affiliateBps)}{" "}
              &#x67e;&#x648;&#x631;&#x633;&#x627;&#x646;&#x62a; &#x622;&#x646;&#x6cc; &#x628;&#x647; &#x6a9;&#x6cc;&#x641; &#x62a;&#x648; &#x648;&#x627;&#x631;&#x6cc;&#x632; &#x645;&#x6cc;‌&#x634;&#x648;&#x62f;.
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
