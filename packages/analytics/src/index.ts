/**
 * MAIRON analytics — the causal event client.
 *
 * `MaironAnalyticsClient` dispatches 10-DNA lab telemetry to the kernel's
 * `/api/v1/grok-lab/*` surface and can export the SFT/DPO JSONL dataset,
 * always through the real endpoints — never a local fake store.
 */

import { DNA_COHORT_COUNT, MAIRON_DNA_COHORTS, isValidDnaCode, type DnaCohortSpec } from "./dna";
import { fnv1a32, sessionFingerprint, causalEventId } from "./hashing";
import {
  buildDpoPair,
  buildSftDpoDataset,
  buildSftLine,
  maironSystemPrompt,
  type DnaTelemetryTurn,
  type DpoPair,
  type SftLine,
} from "./preference-pairs";

export {
  DNA_COHORT_COUNT,
  MAIRON_DNA_COHORTS,
  isValidDnaCode,
  fnv1a32,
  sessionFingerprint,
  causalEventId,
  buildDpoPair,
  buildSftDpoDataset,
  buildSftLine,
  maironSystemPrompt,
};
export type { DnaCohortSpec, DnaTelemetryTurn, DpoPair, SftLine };

/** Options for the analytics client. */
export interface MaironAnalyticsOptions {
  /** Kernel base, e.g. `https://kernel.example.com` (no `/api/v1` suffix). */
  baseUrl?: string;
  /** Bearer session token (initData-minted). */
  token?: string | null;
  /** Deterministic session seed for the fingerprint. */
  sessionSeed?: string;
  /** Explicit user id for the fingerprint (defaults to `anon`). */
  userId?: string;
}

/** The wire shape the kernel accepts on `POST /api/v1/grok-lab/telemetry`. */
export interface GrokLabTelemetryPayload {
  id: string;
  dnaCode: string;
  userHandle: string;
  hourOffset: number;
  uxTier: string;
  locale: string;
  grokPersona: string;
  modelSlug: string;
  routingMode: string;
  promptText: string;
  thinkingTraceJson: string;
  responseText: string;
  reaScore: number;
  latencyMs: number;
  usableForTraining: boolean;
}

/** The kernel's telemetry dashboard slice (GET). */
export interface GrokLabSummary {
  signature: string;
  totalEntries: number;
  avgReaScore: number;
  dnaCohorts: number;
}

/** One dispatched event's receipt. */
export interface DispatchReceipt {
  eventId: string;
  accepted: boolean;
  status: number;
}

/** API error envelope mirror — `{ error: { code, message, meta? } }`. */
export class AnalyticsApiError extends Error {
  readonly status: number;
  readonly code: string;
  constructor(status: number, code: string, message: string) {
    super(message);
    this.name = "AnalyticsApiError";
    this.status = status;
    this.code = code;
  }
}

/**
 * The MAIRON 10-DNA analytics client. All dispatches hit the real kernel
 * routes: `POST /api/v1/grok-lab/telemetry`, `GET /api/v1/grok-lab/export-sft-dpo`,
 * `GET /api/v1/grok-lab/summary`.
 */
export class MaironAnalyticsClient {
  private readonly baseUrl: string;
  private token: string | null;
  private readonly fingerprint: string;
  /** Local ring of payloads pending dispatch (persists across flushes). */
  private readonly pending: GrokLabTelemetryPayload[] = [];

  constructor(options: MaironAnalyticsOptions = {}) {
    const base = (options.baseUrl ?? "").trim();
    this.baseUrl = base.length > 0 ? base.replace(/\/+$/, "") : "";
    this.token = options.token ?? null;
    this.fingerprint = sessionFingerprint(options.userId ?? "anon", options.sessionSeed ?? "mairon");
  }

  /** Update the bearer token (after initData mint). */
  setToken(token: string | null): void {
    this.token = token;
  }

  /** The deterministic session fingerprint for this client instance. */
  get sessionId(): string {
    return this.fingerprint;
  }

  /** The number of buffered rows awaiting dispatch. */
  get bufferedCount(): number {
    return this.pending.length;
  }

  /** Resolve the kernel base — runtime endpoints first, then the constructor base. */
  private async resolveBase(): Promise<string> {
    if (this.baseUrl.length > 0) return this.baseUrl;
    const w = typeof window !== "undefined" ? (window as unknown as { __MAIRON_KERNEL_URL__?: string }) : undefined;
    const runtimeBase = w?.__MAIRON_KERNEL_URL__;
    if (typeof runtimeBase === "string" && runtimeBase.length > 0) return runtimeBase.replace(/\/+$/, "");
    if (typeof process !== "undefined" && typeof process.env?.MAIRON_KERNEL_URL === "string") {
      return process.env.MAIRON_KERNEL_URL.replace(/\/+$/, "");
    }
    return "";
  }

  private async request(path: string, init?: RequestInit): Promise<Response> {
    const base = await this.resolveBase();
    if (base.length === 0) {
      throw new AnalyticsApiError(0, "KERNEL_UNRESOLVED", "no MAIRON kernel base URL is configured");
    }
    const headers = new Headers(init?.headers);
    headers.set("Content-Type", "application/json");
    if (this.token) headers.set("Authorization", `Bearer ${this.token}`);
    const res = await fetch(`${base}/api/v1${path}`, { ...init, headers });
    if (!res.ok) {
      let code = `HTTP_${res.status}`;
      let message = res.statusText || "kernel request failed";
      try {
        const body = (await res.json()) as { error?: { code?: string; message?: string } };
        if (body?.error?.code) code = body.error.code;
        if (body?.error?.message) message = body.error.message;
      } catch {
        // keep the HTTP fallbacks
      }
      throw new AnalyticsApiError(res.status, code, message);
    }
    return res;
  }

  /** Build a deterministic wire payload from a lab turn + session context. */
  toPayload(turn: DnaTelemetryTurn, tick: number): GrokLabTelemetryPayload {
    if (!isValidDnaCode(turn.dnaCode)) {
      throw new AnalyticsApiError(0, "DNA_CODE_UNKNOWN", `unknown DNA cohort: ${turn.dnaCode}`);
    }
    const cohort = MAIRON_DNA_COHORTS.find((c) => c.dnaCode === turn.dnaCode);
    return {
      id: causalEventId(this.fingerprint, tick, turn.dnaCode),
      dnaCode: turn.dnaCode,
      userHandle: cohort?.userHandle ?? turn.promptText.slice(0, 24),
      hourOffset: Math.max(0, Math.min(23, Math.trunc(turn.hourOffset))),
      uxTier: turn.uxTier,
      locale: turn.locale,
      grokPersona: turn.grokPersona,
      modelSlug: turn.modelSlug,
      routingMode: turn.routingMode,
      promptText: turn.promptText,
      thinkingTraceJson: turn.thinkingTraceJson,
      responseText: turn.responseText,
      reaScore: Math.max(0, Math.min(100, Math.trunc(turn.reaScore))),
      latencyMs: Math.max(0, Math.trunc(turn.latencyMs)),
      usableForTraining: turn.usableForTraining,
    };
  }

  /**
   * Dispatch one telemetry turn to `POST /api/v1/grok-lab/telemetry`.
   * On network failure the row stays buffered (`flushTelemetry` retries).
   */
  async trackDnaEvent(turn: DnaTelemetryTurn, tick: number): Promise<DispatchReceipt> {
    const payload = this.toPayload(turn, tick);
    try {
      const res = await this.request("/grok-lab/telemetry", {
        method: "POST",
        body: JSON.stringify(payload),
      });
      return { eventId: payload.id, accepted: res.ok, status: res.status };
    } catch (err) {
      this.pending.push(payload);
      if (err instanceof AnalyticsApiError) throw err;
      return { eventId: payload.id, accepted: false, status: 0 };
    }
  }

  /** Flush every buffered row through the same real endpoint. */
  async flushTelemetry(): Promise<DispatchReceipt[]> {
    const receipts: DispatchReceipt[] = [];
    const remaining: GrokLabTelemetryPayload[] = [];
    for (const payload of this.pending) {
      try {
        const res = await this.request("/grok-lab/telemetry", {
          method: "POST",
          body: JSON.stringify(payload),
        });
        receipts.push({ eventId: payload.id, accepted: res.ok, status: res.status });
      } catch {
        remaining.push(payload);
        receipts.push({ eventId: payload.id, accepted: false, status: 0 });
      }
    }
    this.pending.length = 0;
    this.pending.push(...remaining);
    return receipts;
  }

  /** Fetch the SFT/DPO JSONL export from `GET /api/v1/grok-lab/export-sft-dpo`. */
  async exportSftDpo(): Promise<string> {
    const res = await this.request("/grok-lab/export-sft-dpo", { method: "GET" });
    return await res.text();
  }

  /** Fetch the cohort-health summary from `GET /api/v1/grok-lab/summary`. */
  async labSummary(): Promise<GrokLabSummary> {
    const res = await this.request("/grok-lab/summary", { method: "GET" });
    return (await res.json()) as GrokLabSummary;
  }

  /** Build the SFT/DPO dataset locally (same rules as the kernel export). */
  buildDataset(turns: readonly DnaTelemetryTurn[], minReaScore = 85): string {
    return buildSftDpoDataset(turns, minReaScore);
  }
}

/** Default shared client (no token; call `setToken` after initData mint). */
export const maironAnalytics = new MaironAnalyticsClient();
