"use client";

/**
 * MAIRON mini-app — the 4-tab Sovereign OS.
 *
 * One `MaironMiniAppState` store above the tab bar; switching tabs moves
 * only the perspective pointer (IMPATIUM law 2: lossless switching). Every
 * tab is a real surface wired to the kernel — no mockup state anywhere.
 */

import { PerspectiveSwitcherBar } from "@kian/ui";
import { MiniShell } from "@/components/kian/mini-shell";
import { GuestGate, PreviewBanner } from "@/components/kian/guest-gate";
import { useSession } from "@/components/kian/session";
import {
  MINIAPP_PERSPECTIVES,
  MINIAPP_PERSPECTIVE_LABELS,
  MaironMiniAppStateProvider,
  useMaironMiniApp,
  type MiniAppPerspective,
} from "@/lib/mairon-state";
import { TabForge } from "@/components/mairon/tab-forge";
import { TabQuantDesk } from "@/components/mairon/tab-quant";
import { TabWallet } from "@/components/mairon/tab-wallet";
import { TabWarRoom } from "@/components/mairon/tab-warroom";

function SyncPostureChip() {
  const { state } = useMaironMiniApp();
  const tone =
    state.syncPosture === "live"
      ? "bg-emerald-500/15 text-emerald-300"
      : state.syncPosture === "degraded"
        ? "bg-red-500/15 text-red-300"
        : state.syncPosture === "syncing"
          ? "bg-sky-500/15 text-sky-300"
          : "bg-zinc-700/60 text-zinc-400";
  return (
    <span className={`rounded-full px-2 py-0.5 text-[10px] font-bold uppercase tracking-wider ${tone}`}>
      world state · {state.syncPosture}
    </span>
  );
}

function JobRail() {
  const { state } = useMaironMiniApp();
  if (state.jobs.length === 0) return null;
  return (
    <div className="flex flex-col gap-1" aria-label="Background jobs surviving tab switches">
      {state.jobs.slice(0, 3).map((job) => (
        <div key={job.jobId} className="flex items-center gap-2 rounded-lg border border-zinc-800 bg-zinc-900/70 px-2.5 py-1.5">
          <span
            className={`size-1.5 rounded-full ${
              job.status === "running"
                ? "bg-sky-400 animate-pulse"
                : job.status === "succeeded"
                  ? "bg-emerald-400"
                  : "bg-red-400"
            }`}
            aria-hidden
          />
          <span className="text-[11px] font-semibold text-zinc-300">{job.label}</span>
          <span className="ml-auto text-[10px] text-zinc-500">{MINIAPP_PERSPECTIVE_LABELS[job.perspective]}</span>
        </div>
      ))}
    </div>
  );
}

function TabPanels() {
  const { state, switchTab } = useMaironMiniApp();
  return (
    <div className="flex flex-col gap-4">
      <div className="flex items-center justify-between gap-2">
        <PerspectiveSwitcherBar
          perspectives={MINIAPP_PERSPECTIVES}
          active={state.activeTab}
          onChange={(p) => switchTab(p as MiniAppPerspective)}
          compact
          className="flex-1"
        />
        <SyncPostureChip />
      </div>

      <JobRail />

      {state.lastError ? (
        <p className="rounded-xl border border-red-500/30 bg-red-500/10 px-3 py-2 text-[11px] text-red-200">
          {state.lastError}
        </p>
      ) : null}

      {/*
       * Lossless perspective switching — every tab stays mounted in the DOM and
       * only its visibility toggles, so zero component-local state is ever lost
       * when the perspective pointer moves (in addition to the shared store).
       */}
      <div
        hidden={state.activeTab !== "forge"}
        style={{ display: state.activeTab === "forge" ? "block" : "none" }}
      >
        <TabForge />
      </div>
      <div
        hidden={state.activeTab !== "desk"}
        style={{ display: state.activeTab === "desk" ? "block" : "none" }}
      >
        <TabQuantDesk />
      </div>
      <div
        hidden={state.activeTab !== "wallet"}
        style={{ display: state.activeTab === "wallet" ? "block" : "none" }}
      >
        <TabWallet />
      </div>
      <div
        hidden={state.activeTab !== "war-room"}
        style={{ display: state.activeTab === "war-room" ? "block" : "none" }}
      >
        <TabWarRoom />
      </div>

      <p className="pb-2 text-center text-[10px] text-zinc-600">
        MAIRON OS · one world state · switching never resets your work
      </p>
    </div>
  );
}

function SovereignBody() {
  const { state } = useSession();
  if (state.phase === "loading") {
    return (
      <div className="flex flex-1 items-center justify-center py-16 text-sm text-zinc-500">
        Connecting to the MAIRON kernel…
      </div>
    );
  }
  if (state.phase === "guest") {
    return <GuestGate />;
  }
  if (state.phase === "preview") {
    return (
      <>
        <PreviewBanner />
        <div className="px-4 py-4">
          <MaironMiniAppStateProvider>
            <TabPanels />
          </MaironMiniAppStateProvider>
        </div>
      </>
    );
  }
  return (
    <div className="px-4 py-4">
      <MaironMiniAppStateProvider>
        <TabPanels />
      </MaironMiniAppStateProvider>
    </div>
  );
}

export default function MaironMiniAppPage() {
  return (
    <MiniShell>
      <SovereignBody />
    </MiniShell>
  );
}
