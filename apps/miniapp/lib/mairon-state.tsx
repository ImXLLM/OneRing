"use client";

/**
 * MAIRON Mini-App OS — the Single Authoritative World State.
 *
 * IMPERIUM law 1: every tab (ToySpark Forge, Pocket Desk, Sovereign Wallet,
 * Founder War-Room) reads and writes the same `MaironMiniAppState` store,
 * backed by `MaironSdkClient.syncWorldState()` against the real kernel.
 * IMPERIUM law 2: switching tabs never resets, drops, or re-fetches another
 * tab's state — jobs, drafts, backtest results and the war-room tick cursor
 * live in one reducer above the tab bar.
 */

import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useReducer,
  useRef,
} from "react";
import { ApiError, createMaironSdk, type MaironSdkClient, type MaironWorldStateT } from "@kian/sdk";

/** The four mini-app perspectives, in switcher order. */
export const MINIAPP_PERSPECTIVES = ["forge", "desk", "wallet", "war-room"] as const;
export type MiniAppPerspective = (typeof MINIAPP_PERSPECTIVES)[number];

/** Human labels of the four perspectives. */
export const MINIAPP_PERSPECTIVE_LABELS: Record<MiniAppPerspective, string> = {
  forge: "ToySpark Forge",
  desk: "Pocket Desk",
  wallet: "Sovereign Wallet",
  "war-room": "War-Room",
};

/** One background job surviving perspective switches. */
export interface MiniAppJob {
  jobId: string;
  perspective: MiniAppPerspective;
  label: string;
  status: "running" | "succeeded" | "failed";
  startedAtIso: string;
  detail: string;
}

/** Per-tab working state that survives switching (IMPERIUM law 2). */
export interface MiniAppDrafts {
  forgeIdea: string;
  forgeSessionId: string | null;
  forgeProductName: string;
  forgeLocale: string;
  deskVibePrompt: string;
  deskSymbol: string;
  walletGrossUsd: string;
  walletScenario: "marketplace_sale" | "instant_boost";
  walletReferred: boolean;
  warRoomVoice: string;
}

/** The authoritative mini-app state — one store, four perspectives. */
export interface MaironMiniAppState {
  revision: number;
  activeTab: MiniAppPerspective;
  syncPosture: "idle" | "syncing" | "live" | "degraded";
  syncedAtIso: string | null;
  world: MaironWorldStateT | null;
  jobs: MiniAppJob[];
  drafts: MiniAppDrafts;
  /** Last kernel error message (honest, never swallowed). */
  lastError: string | null;
}

type MiniAppAction =
  | { type: "switch-tab"; tab: MiniAppPerspective }
  | { type: "sync-start" }
  | { type: "sync-live"; world: MaironWorldStateT }
  | { type: "sync-degraded"; message: string; world: MaironWorldStateT | null }
  | { type: "patch-draft"; patch: Partial<MiniAppDrafts> }
  | { type: "job-upsert"; job: MiniAppJob }
  | { type: "clear-error" };

const INITIAL_DRAFTS: MiniAppDrafts = {
  forgeIdea: "",
  forgeSessionId: null,
  forgeProductName: "",
  forgeLocale: "en",
  deskVibePrompt: "buy the dip with a trailing stop, max 10 percent equity per trade",
  deskSymbol: "BTC/USDT",
  walletGrossUsd: "50.00",
  walletScenario: "marketplace_sale",
  walletReferred: false,
  warRoomVoice: "mairon_sovereign",
};

const INITIAL_STATE: MaironMiniAppState = {
  revision: 0,
  activeTab: "forge",
  syncPosture: "idle",
  syncedAtIso: null,
  world: null,
  jobs: [],
  drafts: INITIAL_DRAFTS,
  lastError: null,
};

function reducer(state: MaironMiniAppState, action: MiniAppAction): MaironMiniAppState {
  switch (action.type) {
    case "switch-tab":
      // IMPERIUM law 2 — only the pointer moves; drafts, jobs and the world stay.
      return { ...state, activeTab: action.tab, revision: state.revision + 1 };
    case "sync-start":
      return { ...state, syncPosture: "syncing" };
    case "sync-live":
      return {
        ...state,
        syncPosture: "live",
        world: action.world,
        syncedAtIso: action.world.snapshotAtIso,
        revision: state.revision + 1,
        lastError: null,
      };
    case "sync-degraded":
      // degraded keeps the last known world — non-destructive.
      return {
        ...state,
        syncPosture: "degraded",
        lastError: action.message,
        world: action.world ?? state.world,
        revision: state.revision + 1,
      };
    case "patch-draft":
      return { ...state, drafts: { ...state.drafts, ...action.patch } };
    case "job-upsert": {
      const rest = state.jobs.filter((j) => j.jobId !== action.job.jobId);
      return { ...state, jobs: [action.job, ...rest].slice(0, 12) };
    }
    case "clear-error":
      return { ...state, lastError: null };
    default:
      return state;
  }
}

interface MaironMiniAppStateContextValue {
  state: MaironMiniAppState;
  sdk: MaironSdkClient;
  switchTab: (tab: MiniAppPerspective) => void;
  patchDraft: (patch: Partial<MiniAppDrafts>) => void;
  refreshWorld: () => Promise<void>;
  registerJob: (job: MiniAppJob) => void;
  setError: (message: string | null) => void;
}

const MaironMiniAppContext = createContext<MaironMiniAppStateContextValue | null>(null);

/** Extract the honest message from a kernel error envelope. */
export function describeError(err: unknown): string {
  if (err instanceof ApiError) return `${err.code}: ${err.message}`;
  if (err instanceof Error) return err.message;
  return "Unknown kernel error";
}

export function MaironMiniAppStateProvider({
  children,
  kernelBaseUrl,
}: {
  children: React.ReactNode;
  kernelBaseUrl?: string;
}) {
  const [state, dispatch] = useReducer(reducer, INITIAL_STATE);
  const sdkRef = useRef<MaironSdkClient | null>(null);
  if (sdkRef.current === null) {
    sdkRef.current = createMaironSdk(kernelBaseUrl ? { baseUrl: kernelBaseUrl } : {});
  }
  const sdk = sdkRef.current;

  const refreshWorld = useCallback(async () => {
    dispatch({ type: "sync-start" });
    try {
      const world = await sdk.syncWorldState("war-room");
      dispatch({ type: "sync-live", world });
    } catch (err) {
      dispatch({ type: "sync-degraded", message: describeError(err), world: null });
    }
  }, [sdk]);

  useEffect(() => {
    void refreshWorld();
  }, [refreshWorld]);

  const value = useMemo<MaironMiniAppStateContextValue>(
    () => ({
      state,
      sdk,
      switchTab: (tab) => dispatch({ type: "switch-tab", tab }),
      patchDraft: (patch) => dispatch({ type: "patch-draft", patch }),
      refreshWorld,
      registerJob: (job) => dispatch({ type: "job-upsert", job }),
      setError: (message) =>
        message === null ? dispatch({ type: "clear-error" }) : dispatch({ type: "sync-degraded", message, world: null }),
    }),
    [state, sdk, refreshWorld],
  );

  return <MaironMiniAppContext.Provider value={value}>{children}</MaironMiniAppContext.Provider>;
}

export function useMaironMiniApp(): MaironMiniAppStateContextValue {
  const ctx = useContext(MaironMiniAppContext);
  if (!ctx) throw new Error("useMaironMiniApp must be used inside <MaironMiniAppStateProvider>");
  return ctx;
}
