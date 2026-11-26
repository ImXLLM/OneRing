"use client";

/**
 * MAIRON World State — the Single Authoritative store of the Web OS (IMPERIUM
 * law 1). Every perspective reads from and dispatches mutations to this one
 * reducer, backed by `@kian/sdk` and `@kian/contracts`.
 *
 * IMPERIUM law 2 — Non-destructive perspective switching: `SWITCH_PERSPECTIVE`
 * touches only the perspective pointer. Every subsystem slice (war-room feed,
 * quant backtest, treasury charter, HUD presets, studio drafts), the job
 * registry, and the telemetry stream survive switches untouched, and the
 * war-room autoplay clock is driven from this provider — not from any mounted
 * component — so background jobs keep advancing while a different perspective
 * is on screen.
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

import type {
  HudPresetsResponseT,
  MaironPerspectiveT,
  SubsystemSyncPostureT,
  SupremacySweVerifyResponseT,
  StudioUxProfilesResponseT,
  WarRoomDirectiveResponseT,
  WarRoomTickViewT,
  WarRoomStateResponseT,
  WorldJobT,
  WorldTelemetryEventT,
} from "@kian/contracts";
import type {
  MonopolyCharterAndBaseResponseT,
  MonopolyGenomeMatrixResponseT,
  MonopolyRouteModelResponseT,
  MonopolySimulateSplitResponseT,
  QuantVenuesResponseT,
  QuantVibeBacktestResponseT,
} from "@kian/contracts";
import type { MaironSdkClient } from "@kian/sdk";
import { createMaironSdk } from "@kian/sdk";

// ── Subsystem identifiers ─────────────────────────────────────────────────────

export type SubsystemId =
  | "super-studio"
  | "quant-desk"
  | "treasury"
  | "neural-hud"
  | "war-room";

const SUBSYSTEM_IDS: readonly SubsystemId[] = [
  "super-studio",
  "quant-desk",
  "treasury",
  "neural-hud",
  "war-room",
];

export type WarRoomSpeed = 1 | 2 | 5;

// ── Store shape ───────────────────────────────────────────────────────────────

export type SuperStudioSlice = {
  profiles: StudioUxProfilesResponseT | null;
  morphMode: "ToySpark" | "CreatorStudio" | "ProTerminal";
  selectedModality: string;
  draftPrompt: string;
  lastGeneration: Record<string, unknown> | null;
  lastSeal: SupremacySweVerifyResponseT | null;
};

export type QuantSlice = {
  venues: QuantVenuesResponseT | null;
  lastBacktest: QuantVibeBacktestResponseT | null;
  claimVerification: { verified: boolean; recomputedSha256: string; checkedAtIso: string } | null;
};

export type TreasurySlice = {
  charter: MonopolyCharterAndBaseResponseT | null;
  genome: MonopolyGenomeMatrixResponseT | null;
  lastSplit: MonopolySimulateSplitResponseT | null;
  lastRouting: MonopolyRouteModelResponseT | null;
};

export type NeuralHudSlice = {
  presets: HudPresetsResponseT | null;
  selectedPresetSlug: string | null;
  lastRender: Record<string, unknown> | null;
};

export type WarRoomSlice = {
  snapshot: WarRoomStateResponseT | null;
  tickFeed: WarRoomTickViewT[];
  directiveLog: WarRoomDirectiveResponseT[];
  explain: Record<string, unknown> | null;
  playing: boolean;
  speed: WarRoomSpeed;
};

export type WorldStoreState = {
  revision: number;
  perspective: MaironPerspectiveT;
  perspectiveEnteredAtIso: string;
  lastSyncAtIso: string | null;
  syncInProgress: boolean;
  postures: Record<SubsystemId, SubsystemSyncPostureT>;
  superStudio: SuperStudioSlice;
  quant: QuantSlice;
  treasury: TreasurySlice;
  neuralHud: NeuralHudSlice;
  warRoom: WarRoomSlice;
  jobs: WorldJobT[];
  telemetry: WorldTelemetryEventT[];
};

function initialPostures(): Record<SubsystemId, SubsystemSyncPostureT> {
  const record = {} as Record<SubsystemId, SubsystemSyncPostureT>;
  for (const id of SUBSYSTEM_IDS) {
    record[id] = { subsystem: id, posture: "stale", syncedAtIso: null, detail: null };
  }
  return record;
}

export function initialWorldState(): WorldStoreState {
  return {
    revision: 0,
    perspective: "war-room",
    perspectiveEnteredAtIso: new Date().toISOString(),
    lastSyncAtIso: null,
    syncInProgress: false,
    postures: initialPostures(),
    superStudio: {
      profiles: null,
      morphMode: "CreatorStudio",
      selectedModality: "agent",
      draftPrompt: "",
      lastGeneration: null,
      lastSeal: null,
    },
    quant: {
      venues: null,
      lastBacktest: null,
      claimVerification: null,
    },
    treasury: {
      charter: null,
      genome: null,
      lastSplit: null,
      lastRouting: null,
    },
    neuralHud: {
      presets: null,
      selectedPresetSlug: null,
      lastRender: null,
    },
    warRoom: {
      snapshot: null,
      tickFeed: [],
      directiveLog: [],
      explain: null,
      playing: false,
      speed: 1,
    },
    jobs: [],
    telemetry: [],
  };
}

// ── Actions ───────────────────────────────────────────────────────────────────

export type WorldAction =
  | { type: "SWITCH_PERSPECTIVE"; perspective: MaironPerspectiveT }
  | { type: "SYNC_START" }
  | {
      type: "SYNC_DONE";
      atIso: string;
      postures: SubsystemSyncPostureT[];
      snapshot?: Partial<{
        warRoom: WarRoomStateResponseT;
        treasury: MonopolyCharterAndBaseResponseT;
        genome: MonopolyGenomeMatrixResponseT;
        quant: QuantVenuesResponseT;
        neuralHud: HudPresetsResponseT;
        superStudio: StudioUxProfilesResponseT;
      }>;
    }
  | { type: "SET_MORPH_MODE"; mode: "ToySpark" | "CreatorStudio" | "ProTerminal" }
  | { type: "SET_STUDIO_MODALITY"; modality: string }
  | { type: "SET_STUDIO_DRAFT"; prompt: string }
  | { type: "SET_STUDIO_GENERATION"; generation: Record<string, unknown> }
  | { type: "SET_STUDIO_SEAL"; seal: SupremacySweVerifyResponseT }
  | { type: "SET_QUANT_VENUES"; venues: QuantVenuesResponseT }
  | { type: "SET_QUANT_BACKTEST"; backtest: QuantVibeBacktestResponseT }
  | { type: "SET_QUANT_CLAIM_VERIFICATION"; verified: boolean; recomputedSha256: string; atIso: string }
  | { type: "SET_CHARTER"; charter: MonopolyCharterAndBaseResponseT }
  | { type: "SET_GENOME"; genome: MonopolyGenomeMatrixResponseT }
  | { type: "SET_LAST_SPLIT"; split: MonopolySimulateSplitResponseT }
  | { type: "SET_LAST_ROUTING"; routing: MonopolyRouteModelResponseT }
  | { type: "SET_HUD_PRESETS"; presets: HudPresetsResponseT }
  | { type: "SET_HUD_SELECTED"; slug: string }
  | { type: "SET_HUD_RENDER"; render: Record<string, unknown> }
  | { type: "SET_WARROOM_SNAPSHOT"; snapshot: WarRoomStateResponseT }
  | { type: "APPEND_WARROOM_TICKS"; ticks: WarRoomTickViewT[] }
  | { type: "APPEND_WARROOM_DIRECTIVE"; directive: WarRoomDirectiveResponseT }
  | { type: "SET_WARROOM_EXPLAIN"; explain: Record<string, unknown> }
  | { type: "SET_WARROOM_PLAYING"; playing: boolean }
  | { type: "SET_WARROOM_SPEED"; speed: WarRoomSpeed }
  | { type: "REGISTER_JOB"; job: WorldJobT }
  | { type: "UPDATE_JOB"; jobId: string; status: WorldJobT["status"]; detail?: string; finishedAtIso?: string }
  | { type: "APPEND_TELEMETRY"; event: WorldTelemetryEventT }
  | { type: "CLEAR_TELEMETRY" };

/** Pure reducer — subsystem slices are independent; switching never resets. */
export function worldStateReducer(state: WorldStoreState, action: WorldAction): WorldStoreState {
  switch (action.type) {
    case "SWITCH_PERSPECTIVE": {
      if (action.perspective === state.perspective) return state;
      return {
        ...state,
        revision: state.revision + 1,
        perspective: action.perspective,
        perspectiveEnteredAtIso: new Date().toISOString(),
      };
    }
    case "SYNC_START":
      return { ...state, syncInProgress: true };
    case "SYNC_DONE": {
      const postures = { ...state.postures };
      for (const posture of action.postures) {
        postures[posture.subsystem as SubsystemId] = posture;
      }
      const snapshot = action.snapshot ?? {};
      return {
        ...state,
        revision: state.revision + 1,
        syncInProgress: false,
        lastSyncAtIso: action.atIso,
        postures,
        warRoom: snapshot.warRoom
          ? { ...state.warRoom, snapshot: snapshot.warRoom }
          : state.warRoom,
        treasury: snapshot.treasury || snapshot.genome
          ? {
              ...state.treasury,
              charter: snapshot.treasury ?? state.treasury.charter,
              genome: snapshot.genome ?? state.treasury.genome,
            }
          : state.treasury,
        neuralHud: snapshot.neuralHud
          ? { ...state.neuralHud, presets: snapshot.neuralHud }
          : state.neuralHud,
        quant: snapshot.quant
          ? { ...state.quant, venues: snapshot.quant }
          : state.quant,
        superStudio: snapshot.superStudio
          ? { ...state.superStudio, profiles: snapshot.superStudio }
          : state.superStudio,
      };
    }
    case "SET_MORPH_MODE":
      return { ...state, revision: state.revision + 1, superStudio: { ...state.superStudio, morphMode: action.mode } };
    case "SET_STUDIO_MODALITY":
      return { ...state, revision: state.revision + 1, superStudio: { ...state.superStudio, selectedModality: action.modality } };
    case "SET_STUDIO_DRAFT":
      return { ...state, superStudio: { ...state.superStudio, draftPrompt: action.prompt } };
    case "SET_STUDIO_GENERATION":
      return { ...state, revision: state.revision + 1, superStudio: { ...state.superStudio, lastGeneration: action.generation } };
    case "SET_STUDIO_SEAL":
      return { ...state, revision: state.revision + 1, superStudio: { ...state.superStudio, lastSeal: action.seal } };
    case "SET_QUANT_VENUES":
      return { ...state, revision: state.revision + 1, quant: { ...state.quant, venues: action.venues } };
    case "SET_QUANT_BACKTEST":
      return { ...state, revision: state.revision + 1, quant: { ...state.quant, lastBacktest: action.backtest } };
    case "SET_QUANT_CLAIM_VERIFICATION":
      return {
        ...state,
        revision: state.revision + 1,
        quant: {
          ...state.quant,
          claimVerification: {
            verified: action.verified,
            recomputedSha256: action.recomputedSha256,
            checkedAtIso: action.atIso,
          },
        },
      };
    case "SET_CHARTER":
      return { ...state, revision: state.revision + 1, treasury: { ...state.treasury, charter: action.charter } };
    case "SET_GENOME":
      return { ...state, revision: state.revision + 1, treasury: { ...state.treasury, genome: action.genome } };
    case "SET_LAST_SPLIT":
      return { ...state, revision: state.revision + 1, treasury: { ...state.treasury, lastSplit: action.split } };
    case "SET_LAST_ROUTING":
      return { ...state, revision: state.revision + 1, treasury: { ...state.treasury, lastRouting: action.routing } };
    case "SET_HUD_PRESETS":
      return { ...state, revision: state.revision + 1, neuralHud: { ...state.neuralHud, presets: action.presets } };
    case "SET_HUD_SELECTED":
      return { ...state, revision: state.revision + 1, neuralHud: { ...state.neuralHud, selectedPresetSlug: action.slug } };
    case "SET_HUD_RENDER":
      return { ...state, revision: state.revision + 1, neuralHud: { ...state.neuralHud, lastRender: action.render } };
    case "SET_WARROOM_SNAPSHOT":
      return { ...state, revision: state.revision + 1, warRoom: { ...state.warRoom, snapshot: action.snapshot } };
    case "APPEND_WARROOM_TICKS":
      return {
        ...state,
        revision: state.revision + 1,
        warRoom: {
          ...state.warRoom,
          tickFeed: dedupeTicks([...state.warRoom.tickFeed, ...action.ticks]),
        },
      };
    case "APPEND_WARROOM_DIRECTIVE":
      return {
        ...state,
        revision: state.revision + 1,
        warRoom: {
          ...state.warRoom,
          directiveLog: [...state.warRoom.directiveLog, action.directive],
        },
      };
    case "SET_WARROOM_EXPLAIN":
      return { ...state, revision: state.revision + 1, warRoom: { ...state.warRoom, explain: action.explain } };
    case "SET_WARROOM_PLAYING":
      return { ...state, revision: state.revision + 1, warRoom: { ...state.warRoom, playing: action.playing } };
    case "SET_WARROOM_SPEED":
      return { ...state, revision: state.revision + 1, warRoom: { ...state.warRoom, speed: action.speed } };
    case "REGISTER_JOB":
      return { ...state, revision: state.revision + 1, jobs: [...state.jobs, action.job] };
    case "UPDATE_JOB":
      return {
        ...state,
        revision: state.revision + 1,
        jobs: state.jobs.map((job) =>
          job.jobId === action.jobId
            ? {
                ...job,
                status: action.status,
                detail: action.detail ?? job.detail,
                finishedAtIso: action.finishedAtIso ?? job.finishedAtIso,
              }
            : job,
        ),
      };
    case "APPEND_TELEMETRY":
      return {
        ...state,
        telemetry: [...state.telemetry.slice(-199), action.event],
      };
    case "CLEAR_TELEMETRY":
      return { ...state, telemetry: [] };
    default:
      return state;
  }
}

/** Tick feed is keyed by tick index — replayed steps never duplicate. */
function dedupeTicks(ticks: WarRoomTickViewT[]): WarRoomTickViewT[] {
  const byIndex = new Map<number, WarRoomTickViewT>();
  for (const tick of ticks) byIndex.set(tick.tickIndex, tick);
  return [...byIndex.values()].sort((a, b) => a.tickIndex - b.tickIndex);
}

// ── Context ───────────────────────────────────────────────────────────────────

export type MaironWorldContextValue = {
  state: WorldStoreState;
  dispatch: (action: WorldAction) => void;
  sdk: MaironSdkClient;
  syncWorldState: () => Promise<void>;
  telemetry: (source: WorldTelemetryEventT["source"], level: WorldTelemetryEventT["level"], message: string) => void;
  startJob: (kind: WorldJobT["kind"], perspective: MaironPerspectiveT, detail: string) => string;
  finishJob: (jobId: string, status: WorldJobT["status"], detail?: string) => void;
};

const MaironWorldContext = createContext<MaironWorldContextValue | null>(null);

/** Job ID factory — monotonic and unique per session. */
let jobSequence = 0;

export function MaironWorldProvider({ children }: { children: React.ReactNode }) {
  const [state, dispatch] = useReducer(worldStateReducer, undefined, initialWorldState);
  const sdk = useMemo(() => createMaironSdk(), []);
  const stateRef = useRef(state);
  stateRef.current = state;

  const telemetry = useCallback(
    (source: WorldTelemetryEventT["source"], level: WorldTelemetryEventT["level"], message: string) => {
      dispatch({ type: "APPEND_TELEMETRY", event: { atIso: new Date().toISOString(), source, level, message } });
    },
    [],
  );

  const startJob = useCallback(
    (kind: WorldJobT["kind"], perspective: MaironPerspectiveT, detail: string) => {
      jobSequence += 1;
      const jobId = `job:${kind}:${jobSequence}`;
      dispatch({
        type: "REGISTER_JOB",
        job: {
          jobId,
          kind,
          perspective,
          status: "running",
          startedAtIso: new Date().toISOString(),
          finishedAtIso: null,
          detail,
        },
      });
      return jobId;
    },
    [],
  );

  const finishJob = useCallback(
    (jobId: string, status: WorldJobT["status"], detail?: string) => {
      dispatch({ type: "UPDATE_JOB", jobId, status, detail, finishedAtIso: new Date().toISOString() });
    },
    [],
  );

  const syncWorldState = useCallback(async () => {
    dispatch({ type: "SYNC_START" });
    const atIso = new Date().toISOString();
    const probes = [
      {
        subsystem: "war-room" as SubsystemId,
        run: () => sdk.warRoom.getState(),
        absorb: (payload: unknown) => ({ warRoom: payload as WarRoomStateResponseT }),
      },
      {
        subsystem: "treasury" as SubsystemId,
        run: () => sdk.treasury.getCharterState(),
        absorb: (payload: unknown) => ({ treasury: payload as MonopolyCharterAndBaseResponseT }),
      },
      {
        subsystem: "neural-hud" as SubsystemId,
        run: () => sdk.neuralHud.getGraphState(),
        absorb: (payload: unknown) => ({ neuralHud: payload as HudPresetsResponseT }),
      },
      {
        subsystem: "quant-desk" as SubsystemId,
        run: () => sdk.quant.listVenues(),
        absorb: (payload: unknown) => ({ quant: payload as QuantVenuesResponseT }),
      },
      {
        subsystem: "super-studio" as SubsystemId,
        run: () => sdk.superStudio.morphMode() as unknown,
        absorb: (payload: unknown) => ({ superStudio: payload as StudioUxProfilesResponseT }),
      },
    ];
    const settled = await Promise.allSettled(probes.map((p) => p.run()));
    const postures: SubsystemSyncPostureT[] = [];
    const snapshot: Record<string, unknown> = {};
    settled.forEach((outcome, index) => {
      const probe = probes[index];
      if (outcome.status === "fulfilled") {
        postures.push({ subsystem: probe.subsystem, posture: "live", syncedAtIso: atIso, detail: null });
        Object.assign(snapshot, probe.absorb(outcome.value));
      } else {
        const detail = outcome.reason instanceof Error ? outcome.reason.message : String(outcome.reason);
        postures.push({ subsystem: probe.subsystem, posture: "degraded", syncedAtIso: null, detail });
      }
    });
    const genome = await sdk.treasury.listGenomeModels().catch(() => null);
    if (genome) {
      postures.push({ subsystem: "treasury", posture: "live", syncedAtIso: atIso, detail: "genome-matrix" });
      snapshot.genome = genome;
    }
    dispatch({ type: "SYNC_DONE", atIso, postures, snapshot: snapshot as never });
    const degraded = postures.filter((p) => p.posture === "degraded");
    telemetry(
      "sdk",
      degraded.length === 0 ? "info" : "warn",
      degraded.length === 0
        ? `World state synced: ${postures.length} subsystems live`
        : `World state synced with ${degraded.length} degraded subsystem(s): ${degraded.map((p) => p.subsystem).join(", ")}`,
    );
  }, [sdk, telemetry]);

  // ── War-room autoplay clock — provider-owned, survives perspective switches ──
  useEffect(() => {
    const current = stateRef.current;
    if (!current.warRoom.playing) return;
    const snapshot = current.warRoom.snapshot;
    if (!snapshot) return;
    const cursor = Math.min(snapshot.tickCursor, 24);
    if (cursor >= 24) {
      dispatch({ type: "SET_WARROOM_PLAYING", playing: false });
      return;
    }
    const jobId = startJob("war-room-autoplay", "war-room", `autonomous tick advance at speed x${current.warRoom.speed}`);
    const intervalMs = Math.max(600, 3600 / current.warRoom.speed);
    const timer = setInterval(() => {
      const live = stateRef.current;
      if (!live.warRoom.playing) {
        clearInterval(timer);
        finishJob(jobId, "cancelled", "autoplay stopped");
        return;
      }
      const liveSnapshot = live.warRoom.snapshot;
      if (!liveSnapshot) return;
      const nextCursor = Math.min(liveSnapshot.tickCursor, 24);
      if (nextCursor >= 24) {
        dispatch({ type: "SET_WARROOM_PLAYING", playing: false });
        clearInterval(timer);
        finishJob(jobId, "succeeded", "scenario exhausted at tick 24");
        return;
      }
      sdk.warRoom
        .stepTick({ tickCursor: nextCursor, speedMultiplier: live.warRoom.speed })
        .then((stepResponse) => {
          dispatch({ type: "APPEND_WARROOM_TICKS", ticks: [stepResponse.executed] });
          dispatch({
            type: "SET_WARROOM_SNAPSHOT",
            snapshot: {
              ...liveSnapshot,
              tickCursor: stepResponse.tickCursor,
              progressPercent: stepResponse.progressPercent,
              elapsedSeconds: stepResponse.elapsedSeconds,
              wallet: stepResponse.wallet,
              playing: true,
              speedMultiplier: live.warRoom.speed,
            },
          });
        })
        .catch((error: unknown) => {
          const message = error instanceof Error ? error.message : String(error);
          telemetry("war-room", "error", `Autoplay step failed: ${message}`);
          finishJob(jobId, "failed", message);
          dispatch({ type: "SET_WARROOM_PLAYING", playing: false });
          clearInterval(timer);
        });
    }, intervalMs);
    return () => {
      clearInterval(timer);
      finishJob(jobId, "cancelled", "autoplay clock re-armed");
    };
    // Re-arm only when the autoplay switch or speed actually changes; snapshot
    // reads go through `stateRef` so tick updates never tear down the clock.
  }, [state.warRoom.playing, state.warRoom.speed, sdk, startJob, finishJob, telemetry]);

  const value = useMemo<MaironWorldContextValue>(
    () => ({ state, dispatch, sdk, syncWorldState, telemetry, startJob, finishJob }),
    [state, sdk, syncWorldState, telemetry, startJob, finishJob],
  );

  return <MaironWorldContext.Provider value={value}>{children}</MaironWorldContext.Provider>;
}

/** Access the single authoritative world state. Throws outside the provider. */
export function useMaironWorld(): MaironWorldContextValue {
  const context = useContext(MaironWorldContext);
  if (!context) {
    throw new Error("useMaironWorld must be used inside MaironWorldProvider");
  }
  return context;
}
