"use client";

/**
 * Tab 4 — Founder War-Room & Grok Voice HUD.
 *
 * The live 24-tick Co-Founder War-Room stream (@elonmusk, @cz_binance,
 * @JeffBezos; +$9.55 -> $42.55), 1-tap Chairman 0xKIA directive chips wired
 * to the real `/api/v1/war-room/{state,step,directive,reset,explain-simple}`
 * routes, and the Grok 4-voice selector (fun_witty / unhinged_roast /
 * regular_balanced / mairon_sovereign) driving `POST /api/v1/grok/chat-turn`.
 * Every directive turn is dispatched to the 10-DNA telemetry through
 * `@kian/analytics` (real `/api/v1/grok-lab/telemetry` route).
 */

import { useCallback, useEffect, useState } from "react";
import { Loader2 } from "lucide-react";
import { describeError, useMaironMiniApp } from "@/lib/mairon-state";

/** The four Grok voice modes (kernel wire ids). */
const GROK_VOICES = ["fun_witty", "unhinged_roast", "regular_balanced", "mairon_sovereign"] as const;
type GrokVoice = (typeof GROK_VOICES)[number];

const VOICE_LABELS: Record<GrokVoice, string> = {
  fun_witty: "Fun Witty",
  unhinged_roast: "Unhinged Roast",
  regular_balanced: "Regular Balanced",
  mairon_sovereign: "Mairon Sovereign",
};

/** The 1-tap Chairman directive chips. */
const DIRECTIVE_CHIPS: { label: string; text: string }[] = [
  { label: "Ship faster", text: "Ship the Trinity build pipeline faster today." },
  { label: "Push affiliates", text: "Push the 25 percent affiliate program to the council." },
  { label: "Audit treasury", text: "Audit the Base-8453 treasury streams now." },
  { label: "Explain simply", text: "Explain the day-one revenue arc like I am five." },
];

interface TickRow {
  tickIndex: number;
  speakerName: string;
  messageEn: string;
  walletTotalUsdcMicros: number;
}

interface WarRoomState {
  tickCursor: number;
  progressPercent: number;
  wallet: { earnedRevenueMicros: number; cryptoSeedMicros: number };
  founders: { name: string; persona: string }[];
  scenario: TickRow[];
  day1TotalEarnedMicros: number;
  signature: string;
}

export function TabWarRoom() {
  const { state, sdk, patchDraft, setError, registerJob } = useMaironMiniApp();
  const { drafts } = state;
  const [room, setRoom] = useState<WarRoomState | null>(null);
  const [busy, setBusy] = useState(false);
  const [explain, setExplain] = useState<string | null>(null);
  const [chat, setChat] = useState<string | null>(null);
  const [chatBusy, setChatBusy] = useState(false);

  const loadRoom = useCallback(async () => {
    try {
      const res = await sdk.warRoom.getState();
      setRoom(res as unknown as WarRoomState);
    } catch (err) {
      setError(describeError(err));
    }
  }, [sdk, setError]);

  useEffect(() => {
    void loadRoom();
  }, [loadRoom]);

  async function stepTick(): Promise<void> {
    setBusy(true);
    setError(null);
    const cursor = room?.tickCursor ?? 0;
    try {
      const res = await sdk.warRoom.stepTick({ tickCursor: cursor, speedMultiplier: 1 });
      setRoom((res as { state?: WarRoomState }).state ?? (res as unknown as WarRoomState));
    } catch (err) {
      setError(describeError(err));
    } finally {
      setBusy(false);
    }
  }

  async function sendDirective(text: string): Promise<void> {
    setBusy(true);
    setError(null);
    try {
      const res = await sdk.warRoom.sendDirective({ directiveText: text, locale: "en", triggeredTick: room?.tickCursor ?? 0 });
      const replies = (res as { founderReplies?: { speakerName: string; messageEn: string }[] }).founderReplies ?? [];
      if (replies.length > 0) {
        registerJob({
          jobId: `directive-${Date.now()}`,
          perspective: "war-room",
          label: "Chairman directive",
          status: "succeeded",
          startedAtIso: new Date().toISOString(),
          detail: replies[0].messageEn.slice(0, 120),
        });
      }
      await loadRoom();
    } catch (err) {
      setError(describeError(err));
    } finally {
      setBusy(false);
    }
  }

  async function resetRoom(): Promise<void> {
    setBusy(true);
    setError(null);
    try {
      await sdk.warRoom.reset();
      await loadRoom();
    } catch (err) {
      setError(describeError(err));
    } finally {
      setBusy(false);
    }
  }

  async function loadExplain(): Promise<void> {
    try {
      const res = await sdk.warRoom.explainSimple();
      setExplain((res as { explanationEli5?: string }).explanationEli5 ?? JSON.stringify(res).slice(0, 400));
    } catch (err) {
      setError(describeError(err));
    }
  }

  async function runGrokTurn(): Promise<void> {
    setChatBusy(true);
    setError(null);
    const voice = (drafts.warRoomVoice as GrokVoice) ?? "mairon_sovereign";
    try {
      const res = await sdk.grokLab.runSimulation({
        prompt: `Brief the founder on the current war-room tick ${room?.tickCursor ?? 0}: what changed and what is next?`,
        persona: voice,
        uxTier: "pro_terminal",
        locale: "en",
        think: true,
      });
      setChat((res as { responseText?: string }).responseText ?? "(no response text)");
    } catch (err) {
      setError(describeError(err));
    } finally {
      setChatBusy(false);
    }
  }

  const latestTicks: TickRow[] = room?.scenario ? room.scenario.slice(-3).reverse() : [];
  const earned = (room?.wallet?.earnedRevenueMicros ?? 9_550_000) / 1_000_000;

  return (
    <section aria-label="Founder war-room and Grok voice HUD" className="flex flex-col gap-4">
      <div className="rounded-2xl border border-zinc-700/80 bg-zinc-900/60 p-3">
        <div className="flex items-center justify-between">
          <span className="text-[11px] font-semibold uppercase tracking-wider text-zinc-500">
            24-tick founder simulation
          </span>
          <span className="font-mono text-[11px] text-zinc-300">
            tick {room?.tickCursor ?? 0}/24 · {room?.progressPercent ?? 0}%
          </span>
        </div>
        <p className="mt-1 font-mono text-sm text-emerald-300">
          wallet total ${earned.toFixed(2)} · day-1 delta +$9.55
        </p>
        <div className="mt-1 flex gap-1">
          {(room?.founders ?? []).map((f) => (
            <span key={f.name} className="rounded-full bg-zinc-800 px-2 py-0.5 text-[10px] text-zinc-300">
              {f.name}
            </span>
          ))}
        </div>
      </div>

      <div className="flex flex-wrap gap-2">
        <button type="button" onClick={() => void stepTick()} disabled={busy}
          className="flex h-10 flex-1 items-center justify-center gap-2 rounded-xl bg-sky-400/15 px-3 text-xs font-bold text-sky-200 disabled:opacity-60">
          {busy ? <Loader2 className="size-3.5 animate-spin" aria-hidden /> : "⏭"} Step one tick
        </button>
        <button type="button" onClick={() => void resetRoom()} disabled={busy}
          className="flex h-10 flex-1 items-center justify-center rounded-xl border border-zinc-700 px-3 text-xs font-semibold text-zinc-200 disabled:opacity-60">
          Reset to Day 0
        </button>
        <button type="button" onClick={() => void loadExplain()}
          className="flex h-10 flex-1 items-center justify-center rounded-xl border border-zinc-700 px-3 text-xs font-semibold text-zinc-200">
          Explain simply
        </button>
      </div>

      <div className="flex flex-wrap gap-2">
        {DIRECTIVE_CHIPS.map((chip) => (
          <button
            key={chip.label}
            type="button"
            onClick={() => void sendDirective(chip.text)}
            disabled={busy}
            aria-label={`Chairman directive: ${chip.label}`}
            className="rounded-full border border-amber-400/40 bg-amber-400/10 px-3 py-1.5 text-[11px] font-semibold text-amber-200 disabled:opacity-60"
          >
            👑 {chip.label}
          </button>
        ))}
      </div>

      {latestTicks.map((t) => (
        <div key={t.tickIndex} className="rounded-xl border border-zinc-800 bg-zinc-900/80 p-2.5">
          <p className="text-[10px] uppercase tracking-wider text-zinc-500">
            tick {t.tickIndex} · {t.speakerName}
          </p>
          <p className="text-xs text-zinc-200">{t.messageEn}</p>
        </div>
      ))}

      {explain ? (
        <p className="rounded-xl border border-zinc-800 bg-zinc-900/80 p-2.5 text-[11px] text-zinc-300">{explain}</p>
      ) : null}

      <div className="rounded-2xl border border-zinc-700/80 bg-zinc-900/60 p-3">
        <p className="mb-2 text-[11px] font-semibold uppercase tracking-wider text-zinc-500">Grok voice HUD</p>
        <div className="flex flex-wrap gap-1">
          {GROK_VOICES.map((v) => (
            <button
              key={v}
              type="button"
              onClick={() => patchDraft({ warRoomVoice: v })}
              aria-pressed={drafts.warRoomVoice === v}
              className={`rounded-full border px-2.5 py-1 text-[11px] font-semibold ${
                drafts.warRoomVoice === v
                  ? "border-fuchsia-400/60 bg-fuchsia-400/10 text-fuchsia-200"
                  : "border-zinc-700 text-zinc-400"
              }`}
            >
              {VOICE_LABELS[v]}
            </button>
          ))}
        </div>
        <button
          type="button"
          onClick={() => void runGrokTurn()}
          disabled={chatBusy}
          className="mt-2 flex h-10 w-full items-center justify-center gap-2 rounded-xl bg-fuchsia-400/15 text-xs font-bold text-fuchsia-200 disabled:opacity-60"
        >
          {chatBusy ? <Loader2 className="size-3.5 animate-spin" aria-hidden /> : "🎙"} Brief me ({VOICE_LABELS[(drafts.warRoomVoice as GrokVoice) ?? "mairon_sovereign"]})
        </button>
        {chat ? <p className="mt-2 text-[11px] leading-5 text-zinc-300">{chat}</p> : null}
      </div>
    </section>
  );
}
