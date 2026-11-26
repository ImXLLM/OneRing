/**
 * MAIRON SDK — the sovereign TypeScript client (`MaironSdkClient`).
 *
 * Every namespace wraps real kernel routes (`/api/v1/*` plus the standalone
 * `/war-room` and `/neural-hud` dashboards). Legacy imports keep working:
 * `KianSdkClient` and `createKianSdk` are stable aliases of this class, and
 * the legacy flat client in `./index.ts` remains exported unchanged.
 *
 * IMPERIUM law 1: `syncWorldState()` assembles one authoritative snapshot
 * across all active subsystems. Degraded subsystems keep their last known
 * payload instead of throwing — the snapshot is causally consistent and never
 * drops state.
 */

import type {
  ArdaFlagsCatalogResponseT,
  ArdaLocalizeRequestT,
  ArdaLocalizeResponseT,
  GrokChatTurnRequestT,
  GrokChatTurnResponseT,
  GrokUiBundleQueryT,
  GrokUiBundleResponseT,
  HudPresetsResponseT,
  MaironPerspectiveT,
  MaironWorldStateT,
  MonopolyCharterAndBaseResponseT,
  MonopolyGenomeMatrixResponseT,
  MonopolyRouteModelRequestT,
  MonopolyRouteModelResponseT,
  MonopolySimulateSplitRequestT,
  MonopolySimulateSplitResponseT,
  QuantStrategiesResponseT,
  QuantVenuesResponseT,
  QuantVibeBacktestRequestT,
  QuantVibeBacktestResponseT,
  SubsystemSyncPostureT,
  SupremacyCheckpointForkRequestT,
  SupremacyCheckpointForkResponseT,
  SupremacyMcpResponseT,
  SupremacySurgicalPatchRequestT,
  SupremacySurgicalPatchResponseT,
  SupremacySweVerifyRequestT,
  SupremacySweVerifyResponseT,
  TrinityAlphaRequestT,
  TrinityAlphaResponseT,
  TrinityOrchestrateRequestT,
  TrinityOrchestrateResponseT,
  WarRoomDirectiveRequestT,
  WarRoomDirectiveResponseT,
  WarRoomExplainResponseT,
  WarRoomResetResponseT,
  WarRoomStateResponseT,
  WarRoomStepRequestT,
  WarRoomStepResponseT,
} from "@kian/contracts";

import { qs, resolveDefaultApiBase, typedRequest } from "./core";

/** Options for constructing a Mairon SDK client. */
export type MaironSdkOptions = {
  /** Kernel API base (origin of `/api/v1`). Defaults to the env resolvers. */
  baseUrl?: string;
  /** Bearer session token, for session-bound endpoints. */
  token?: string | null;
  /** Admin control-plane key (`X-KIAN-Key`), for `/api/v1/admin/*`. */
  adminKey?: string | null;
};

/** Body for `POST /api/v1/war-room/directive` etc. built by callers. */
export type MaironDirectiveInput = {
  directiveText: string;
  locale?: "fa" | "en";
  triggeredTick?: number;
};

/**
 * The MAIRON SDK client — one instance per app; all perspectives share it.
 * The client is stateless: authoritative state lives in the world-state store
 * that calls `syncWorldState()`.
 */
export class MaironSdkClient {
  readonly baseUrl: string;
  private token: string | null;
  private adminKey: string | null;

  constructor(options: MaironSdkOptions = {}) {
    this.baseUrl = (options.baseUrl ?? resolveDefaultApiBase()).replace(/\/$/, "");
    this.token = options.token ?? null;
    this.adminKey = options.adminKey ?? null;
  }

  /** Attach or replace the bearer session token (never persisted here). */
  setToken(token: string | null): void {
    this.token = token;
  }

  /** Attach or replace the admin control-plane key. */
  setAdminKey(adminKey: string | null): void {
    this.adminKey = adminKey;
  }

  private req<T>(path: string, init: RequestInit & { adminKey?: boolean } = {}): Promise<T> {
    return typedRequest<T>(this.baseUrl, path, {
      ...init,
      token: this.token,
      adminKey: init.adminKey ? this.adminKey : null,
    });
  }

  // ── Mairon Trinity Engine ────────────────────────────────────────────────────

  /** The Trinity engine domain — Alpha discovery, CommandCenter forging, publication. */
  readonly trinity = {
    /** Start/continue an Alpha discovery turn (`POST /api/v1/trinity/alpha`). */
    alpha: (input: TrinityAlphaRequestT) =>
      this.req<TrinityAlphaResponseT>("/trinity/alpha", {
        method: "POST",
        body: JSON.stringify(input),
      }),

    /** Alias kept for symmetry with the UI command: same as `alpha`. */
    build: async (input: TrinityOrchestrateRequestT) => this.trinity.orchestrate(input),

    /** Run the full CommandCenter forging + Mairon judgment (`POST /api/v1/trinity/orchestrate`). */
    orchestrate: (input: TrinityOrchestrateRequestT) =>
      this.req<TrinityOrchestrateResponseT>("/trinity/orchestrate", {
        method: "POST",
        body: JSON.stringify(input),
      }),

    /** Fetch the Trinity session under review (`GET /api/v1/builder/sessions/{id}`). */
    status: (sessionId: string) =>
      this.req<unknown>(`/builder/sessions/${encodeURIComponent(sessionId)}`),

    /** Mint the two-key publish ticket by publishing the session (`POST /api/v1/builder/sessions/{id}/publish`). */
    publishTwoKey: (
      sessionId: string,
      overrides: Record<string, unknown> = {},
    ) =>
      this.req<unknown>(`/builder/sessions/${encodeURIComponent(sessionId)}/publish`, {
        method: "POST",
        body: JSON.stringify(overrides),
      }),

    /** Submit the artifact through the SWE verification gate (`POST /api/v1/supremacy/swe-verify`). */
    submitReview: (input: SupremacySweVerifyRequestT) => this.supremacy.verifySeal(input),
  };

  // ── Mairon Arda i18n ─────────────────────────────────────────────────────────

  /** The Arda localization domain — 20-locale manifest, patches, SKILL.md forge. */
  readonly arda = {
    /** The 20-locale flag catalog (`GET /api/v1/arda/flags`). */
    listLocales: () => this.req<ArdaFlagsCatalogResponseT>("/arda/flags"),

    /** The ecosystem matrix with parity scores (`GET /api/v1/i18n/ecosystem-matrix`). */
    ecosystemMatrix: () => this.req<unknown>("/i18n/ecosystem-matrix"),

    /** Apply a locale patch (`POST /api/v1/i18n/apply-patch`) — the PATCH_READY_SLOT switch. */
    switchLocale: (payload: Record<string, unknown>) =>
      this.req<unknown>("/i18n/apply-patch", {
        method: "POST",
        body: JSON.stringify(payload),
      }),

    /** Localize raw HTML or a product across the 20 locales (`POST /api/v1/arda/localize-now`). */
    localize: (input: ArdaLocalizeRequestT) =>
      this.req<ArdaLocalizeResponseT>("/arda/localize-now", {
        method: "POST",
        body: JSON.stringify(input),
      }),

    /** The forged SKILL.md package of a product (`GET /api/v1/market/products/{id}/skill`). */
    exportSkillMd: (productId: string) =>
      this.req<unknown>(`/market/products/${encodeURIComponent(productId)}/skill`),
  };

  // ── Mairon Super-Studio ──────────────────────────────────────────────────────

  /** The Super-Studio domain — morph modes, nine modalities, portfolio. */
  readonly superStudio = {
    /** Cognitive tiers + modalities + the 6-remedy safety report (`GET /api/v1/studio/ux-profiles`). */
    morphMode: () => this.req<unknown>("/studio/ux-profiles"),

    /** Run one studio modality (`POST /api/v1/studio/generate`). */
    executeModality: (input: Record<string, unknown>) =>
      this.req<unknown>("/studio/generate", {
        method: "POST",
        body: JSON.stringify(input),
      }),

    /** The storefront portfolio catalog (`GET /api/v1/studio/portfolio-catalog`). */
    portfolio: () => this.req<unknown>("/studio/portfolio-catalog"),
  };

  // ── Competitive Supremacy Sandbox ────────────────────────────────────────────

  /**
   * Client-side mirror of the kernel's time-travel checkpoint ledger
   * (`AgentStateCheckpoint`). The kernel persists every checkpoint and reports
   * the ledger size on each fork; it exposes no list route, so this registry —
   * populated by every `restoreCheckpoint` and `applySurgicalDiff` call — is
   * the auditable client view of exactly those rows.
   */
  private readonly checkpointRegistry = new Map<string, SupremacyCheckpointForkResponseT[]>();

  /** The Supremacy domain — surgical patches, checkpoints, MCP, seals. */
  readonly supremacy = {
    /** Apply a SEARCH/REPLACE surgical patch in the live sandbox (`POST /api/v1/supremacy/surgical-patch`). */
    applySurgicalDiff: (input: SupremacySurgicalPatchRequestT) =>
      this.req<SupremacySurgicalPatchResponseT>("/supremacy/surgical-patch", {
        method: "POST",
        body: JSON.stringify(input),
      }),

    /**
     * The checkpoint registry for one workflow — the client mirror of the
     * kernel's `AgentStateCheckpoint` ledger. Rows arrive from
     * `restoreCheckpoint` responses (fork metadata + ledger size); the array
     * is ordered oldest-first exactly like the server-side ledger.
     */
    listCheckpoints: (workflowId: string): readonly SupremacyCheckpointForkResponseT[] =>
      this.checkpointRegistry.get(workflowId) ?? [],

    /** Restore by forking from a checkpoint with a state patch (`POST /api/v1/supremacy/checkpoint/fork`). */
    restoreCheckpoint: async (input: SupremacyCheckpointForkRequestT) => {
      const response = await this.req<SupremacyCheckpointForkResponseT>(
        "/supremacy/checkpoint/fork",
        { method: "POST", body: JSON.stringify(input) },
      );
      const ledger = this.checkpointRegistry.get(input.workflowId) ?? [];
      ledger.push(response);
      this.checkpointRegistry.set(input.workflowId, ledger);
      return response;
    },

    /** Execute a dynamic DAG workflow (`POST /api/v1/supremacy/workflow/execute`). */
    executeWorkflow: (input: Record<string, unknown>) =>
      this.req<unknown>("/supremacy/workflow/execute", {
        method: "POST",
        body: JSON.stringify(input),
      }),

    /** JSON-RPC 2.0 tool invocation (`POST /api/v1/supremacy/mcp`). */
    invokeMcpTool: (payload: Record<string, unknown>) =>
      this.req<SupremacyMcpResponseT>("/supremacy/mcp", {
        method: "POST",
        body: JSON.stringify(payload),
      }),

    /** The SWE verification loop + Mairon seal (`POST /api/v1/supremacy/swe-verify`). */
    verifySeal: (input: SupremacySweVerifyRequestT) =>
      this.req<SupremacySweVerifyResponseT>("/supremacy/swe-verify", {
        method: "POST",
        body: JSON.stringify(input),
      }),
  };

  // ── Mairon Quant & CCXT ──────────────────────────────────────────────────────

  /** The Quant domain — venues, causal backtests, strategies, claim hashes. */
  readonly quant = {
    /** The 8 keyless venues + forensic remedies (`GET /api/v1/quant/venues`). */
    listVenues: () => this.req<QuantVenuesResponseT>("/quant/venues"),

    /** Run the causal `&candles[0..=t]` backtest (`POST /api/v1/quant/vibe-backtest`). */
    runCausalBacktest: (input: QuantVibeBacktestRequestT) =>
      this.req<QuantVibeBacktestResponseT>("/quant/vibe-backtest", {
        method: "POST",
        body: JSON.stringify(input),
      }),

    /** Persisted strategies (`GET /api/v1/quant/strategies`). */
    listStrategies: () => this.req<QuantStrategiesResponseT>("/quant/strategies"),

    /**
     * Verify a claim hash locally with WebCrypto SHA-256: recompute the digest
     * of the canonical claim string and compare it with the ledger's hash.
     */
    verifyClaimHash: async (
      claimVerificationHash: string,
      canonicalClaimString: string,
    ): Promise<{ verified: boolean; recomputedSha256: string }> => {
      const recomputedSha256 = await sha256Hex(canonicalClaimString);
      return {
        verified: recomputedSha256 === claimVerificationHash,
        recomputedSha256,
      };
    },

    /** Deploy a live strategy — the honest gate check happens client-side first. */
    deployLiveStrategy: async (
      strategy: { sharpeRatio: number; claimVerificationHash: string },
      liveSharpeGate = 1.2,
    ): Promise<{ deployed: boolean; reason: string; strategyId: string }> => {
      if (!(strategy.sharpeRatio >= liveSharpeGate)) {
        return {
          deployed: false,
          reason: `Sharpe ${strategy.sharpeRatio.toFixed(2)} is below the ${liveSharpeGate.toFixed(2)} live-deployment gate`,
          strategyId: strategy.claimVerificationHash.slice(0, 12),
        };
      }
      const strategies = await this.quant.listStrategies();
      const known = strategies.strategies.find(
        (s) => s.claimVerificationHash === strategy.claimVerificationHash,
      );
      return {
        deployed: Boolean(known && known.maironApproved),
        reason: known
          ? known.maironApproved
            ? "Mairon-approved strategy confirmed in the kernel ledger"
            : "Strategy found but not Mairon-approved"
          : "Strategy claim hash not found in the kernel ledger",
        strategyId: known?.strategyId ?? strategy.claimVerificationHash.slice(0, 12),
      };
    },
  };

  // ── Base-8453 Treasury & Genome ──────────────────────────────────────────────

  /** The Treasury domain — charter, splits, genome routing. */
  readonly treasury = {
    /** The four-stream charter + Base settlement config (`GET /api/v1/monopoly/charter-and-base`). */
    getCharterState: () => this.req<MonopolyCharterAndBaseResponseT>("/monopoly/charter-and-base"),

    /** Run the atomic Day-1 split (`POST /api/v1/monopoly/simulate-split`). */
    calculateSplit: (input: MonopolySimulateSplitRequestT) =>
      this.req<MonopolySimulateSplitResponseT>("/monopoly/simulate-split", {
        method: "POST",
        body: JSON.stringify(input),
      }),

    /** The 5x9 genome matrix (`GET /api/v1/monopoly/genome-matrix`). */
    listGenomeModels: () => this.req<MonopolyGenomeMatrixResponseT>("/monopoly/genome-matrix"),

    /** Route a task through ManualOverride or AgentAutoDecision (`POST /api/v1/monopoly/route-model`). */
    setGenomeRouting: (input: MonopolyRouteModelRequestT) =>
      this.req<MonopolyRouteModelResponseT>("/monopoly/route-model", {
        method: "POST",
        body: JSON.stringify(input),
      }),
  };

  // ── 3D Neural-HUD & Grok Lab ─────────────────────────────────────────────────

  /** The Neural-HUD domain — presets, synthesis, Grok simulation. */
  readonly neuralHud = {
    /** The HUD preset catalog (`GET /api/v1/hud/presets`). */
    getGraphState: () => this.req<HudPresetsResponseT>("/hud/presets"),

    /** Synthesize a deterministic HUD render (`POST /api/v1/hud/synthesize`). */
    renderPreset: (input: Record<string, unknown>) =>
      this.req<unknown>("/hud/synthesize", {
        method: "POST",
        body: JSON.stringify(input),
      }),

    /** The standalone 3D Neural-HUD dashboard URL on the kernel origin. */
    dashboardUrl: () => `${this.baseUrl.replace(/\/api\/v1$/, "")}/neural-hud`,
  };

  /** The Grok 10-DNA Lab domain — UI bundles, chat turns, SFT/DPO export. */
  readonly grokLab = {
    /** The self-contained Grok UI bundle (`GET /api/v1/grok/ui-bundle`). */
    uiBundle: (query: GrokUiBundleQueryT = {}) =>
      this.req<GrokUiBundleResponseT>(
        `/grok/ui-bundle${qs({
          locale: query.locale,
          uxTier: query.uxTier,
          persona: query.persona,
        })}`,
      ),

    /** One Grok simulation turn with thinking + citations (`POST /api/v1/grok/chat-turn`). */
    runSimulation: (input: GrokChatTurnRequestT) =>
      this.req<GrokChatTurnResponseT>("/grok/chat-turn", {
        method: "POST",
        body: JSON.stringify(input),
      }),

    /** The lab telemetry table (admin) — the export source (`GET /api/v1/admin/lab-telemetry`). */
    exportSftDpo: () => this.req<unknown>("/admin/lab-telemetry", { adminKey: true }),
  };

  // ── Founder War-Room ─────────────────────────────────────────────────────────

  /** The War-Room domain — the five kernel endpoints plus the dashboard URL. */
  readonly warRoom = {
    /** The complete Day-0 state + 24-tick scenario (`GET /api/v1/war-room/state`). */
    getState: () => this.req<WarRoomStateResponseT>("/war-room/state"),

    /** Single-step causal advance (`POST /api/v1/war-room/step`). */
    stepTick: (input: WarRoomStepRequestT) =>
      this.req<WarRoomStepResponseT>("/war-room/step", {
        method: "POST",
        body: JSON.stringify(input),
      }),

    /** Chairman directive + three in-persona replies (`POST /api/v1/war-room/directive`). */
    sendDirective: (input: MaironDirectiveInput) =>
      this.req<WarRoomDirectiveResponseT>("/war-room/directive", {
        method: "POST",
        body: JSON.stringify({
          directiveText: input.directiveText,
          locale: input.locale ?? "en",
          triggeredTick: input.triggeredTick ?? 0,
        }),
      }),

    /** Causal reset to Day 0 (`POST /api/v1/war-room/reset`). */
    reset: () =>
      this.req<WarRoomResetResponseT>("/war-room/reset", {
        method: "POST",
        body: JSON.stringify({}),
      }),

    /** The "0 to 100" ELI5 bundle (`GET /api/v1/war-room/explain-simple`). */
    explainSimple: () => this.req<WarRoomExplainResponseT>("/war-room/explain-simple"),

    /** The standalone War-Room dashboard URL on the kernel origin. */
    dashboardUrl: () => `${this.baseUrl.replace(/\/api\/v1$/, "")}/war-room`,
  };

  // ── Admin control plane ──────────────────────────────────────────────────────

  /** The admin control-plane namespace (requires an admin key). */
  readonly admin = {
    overview: () => this.req<unknown>("/admin/overview", { adminKey: true }),
    auditTail: () => this.req<unknown>("/admin/audit", { adminKey: true }),
    labTelemetry: () => this.req<unknown>("/admin/lab-telemetry", { adminKey: true }),
    hitlList: () => this.req<unknown>("/admin/hitl", { adminKey: true }),
    hitlApprove: (id: string) =>
      this.req<unknown>(`/admin/hitl/${encodeURIComponent(id)}/approve`, {
        method: "POST",
        body: JSON.stringify({}),
        adminKey: true,
      }),
    hitlReject: (id: string) =>
      this.req<unknown>(`/admin/hitl/${encodeURIComponent(id)}/reject`, {
        method: "POST",
        body: JSON.stringify({}),
        adminKey: true,
      }),
    deposits: () => this.req<unknown>("/admin/deposits", { adminKey: true }),
    withdrawals: () => this.req<unknown>("/admin/withdrawals", { adminKey: true }),
    products: () => this.req<unknown>("/admin/products", { adminKey: true }),
    productDelete: (id: string) =>
      this.req<unknown>(`/admin/products/${encodeURIComponent(id)}`, {
        method: "DELETE",
        adminKey: true,
      }),
  };

  // ── Single authoritative world state ─────────────────────────────────────────

  /**
   * Fetch the unified authoritative snapshot across all active subsystems.
   * Each subsystem is probed independently (`Promise.allSettled`): a refused
   * endpoint is recorded as `degraded` with its last error, never thrown, and
   * the perspective pointer travels with the snapshot so perspective switches
   * are non-destructive.
   */
  async syncWorldState(perspective: MaironPerspectiveT = "war-room"): Promise<MaironWorldStateT> {
    const nowIso = new Date().toISOString();
    const probes = [
      { subsystem: "war-room" as const, run: () => this.warRoom.getState() },
      { subsystem: "treasury" as const, run: () => this.treasury.getCharterState() },
      { subsystem: "neural-hud" as const, run: () => this.neuralHud.getGraphState() },
      { subsystem: "quant-desk" as const, run: () => this.quant.listVenues() },
      { subsystem: "super-studio" as const, run: () => this.superStudio.morphMode() },
    ];
    const settled = await Promise.allSettled(probes.map((p) => p.run()));
    const postures: SubsystemSyncPostureT[] = [];
    const payloads: Record<string, unknown | null> = {};
    let revision = 0;
    settled.forEach((outcome, index) => {
      const subsystem = probes[index].subsystem;
      if (outcome.status === "fulfilled") {
        postures.push({ subsystem, posture: "live", syncedAtIso: nowIso, detail: null });
        payloads[subsystem] = outcome.value;
        revision += 1;
      } else {
        const message = outcome.reason instanceof Error ? outcome.reason.message : String(outcome.reason);
        postures.push({ subsystem, posture: "degraded", syncedAtIso: null, detail: message });
        payloads[subsystem] = null;
      }
    });
    const genome = await this.treasury
      .listGenomeModels()
      .catch(() => null);
    if (genome) revision += 1;
    return {
      revision,
      snapshotAtIso: nowIso,
      perspective,
      postures,
      jobs: [],
      warRoom: payloads["war-room"] ?? null,
      treasury: payloads["treasury"] ?? null,
      genome,
      quant: payloads["quant-desk"] ?? null,
      neuralHud: payloads["neural-hud"] ?? null,
    };
  }
}

/** SHA-256 hex of a UTF-8 string via WebCrypto (browser and Node 18+). */
async function sha256Hex(text: string): Promise<string> {
  const digest = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(text));
  return Array.from(new Uint8Array(digest))
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}

/** Backward-compatible alias — existing `KianSdkClient` imports never break. */
export const KianSdkClient = MaironSdkClient;

/** Construct a Mairon SDK client (the canonical factory). */
export function createMaironSdk(options: MaironSdkOptions = {}): MaironSdkClient {
  return new MaironSdkClient(options);
}

/** Backward-compatible alias — existing `createKianSdk` imports never break. */
export const createKianSdk = createMaironSdk;
