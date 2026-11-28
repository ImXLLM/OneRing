"use client";

/**
 * MAIRON — Institutional Governance & Audit Console.
 *
 * Six operational sections over the real kernel routes:
 * 1. Ledger auditor — the `SUM(Posting.amount) == 0` invariant monitor
 *    (`GET /api/v1/economy`) plus the escrow state machine.
 * 2. Two-Key Publish Vault — the HITL queue (`GET /api/v1/admin/hitl` with
 *    approve/reject) and the `MaironVerifiedSeal` gate inspector.
 * 3. Genome override matrix — 5 x 9 = 45 models (`GET /api/v1/monopoly/
 *    genome-matrix`, `POST /api/v1/monopoly/route-model`) with the 69-key
 *    P0 -> P5 environment readiness monitor.
 * 4. 10-DNA Lab Telemetry — the seeded benchmark logs with the 1-click
 *    SFT/DPO JSONL dataset export.
 * 5. War-Room Chairman console — directive broadcast and simulation reset
 *    through the operator session.
 */

import { useCallback, useEffect, useMemo, useState } from "react";

import {
  AdminApiError,
  adminGet,
  adminPost,
  getKey,
  getOperatorToken,
  operatorLogin,
  operatorRequest,
  publicGet,
  setKey,
  setOperatorToken,
} from "@/lib/admin-api";
import { MAIRON_ENV_SPEC, MAIRON_ENV_TIERS, resolveKernelUrl } from "@kian/config";

// ── Wire types (mirrors of the kernel DTOs) ───────────────────────────────────

type Health = {
  status: string;
  service: string;
  version: string;
  llmConfigured: boolean;
  llmModel: string | null;
  db?: string;
  bot?: string;
  ai?: string;
};

type Economy = {
  currency: string;
  trialBalanceSum: number;
  balanced: boolean;
  accountCount: number;
  openEscrows: number;
  escrowLocked: number;
  circulatingSupply: number;
  treasuryBalance: number;
  mintedAllTime: number;
  burnedAllTime: number;
};

type Overview = {
  users: number;
  accounts: number;
  escrows: number;
  hitlPending: number;
  mintedAllTime: number;
  burnedAllTime: number;
  aiCalls: number;
  revenueSeries: Array<{ date: string; din: number }>;
};

type HitlView = {
  id: string;
  kind: string;
  escrowId: string;
  amount: number;
  requesterId: string;
  note?: string;
  status: string;
  createdAt: string;
  decidedBy?: string;
  decidedAt?: string;
};

type GenomeModel = {
  modelId: string;
  aliases: string[];
  genomeDir: string;
  specialty: string;
  canonicalFiles: string[];
  filesVerified: number;
  isComplete: boolean;
  trinityAvatar: string;
  verifiedAgainst: string;
};

type GenomeMatrix = {
  signature: string;
  models: GenomeModel[];
  routingRules: Array<{ domain: string; championModelId: string; trinityAvatar: string }>;
  totalFilesVerified: number;
  expectedFileCount: number;
  selectionModes: string[];
};

type TelemetryEntry = {
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
  responseText: string;
  reaScore: number;
  latencyMs: number;
  usableForTraining: boolean;
  createdAt: string;
};

type TelemetryResponse = {
  entries: TelemetryEntry[];
  dashboard: {
    totalEntries: number;
    avgReaScore: number;
    minReaScore: number;
    maxReaScore: number;
    cohortHealth: Array<{
      dnaCode: string;
      userHandle: string;
      turns: number;
      avgReaScore: number;
      unusable: number;
      frictionEvents: number;
      dominantModel: string;
    }>;
    byModel: Record<string, number>;
    byLocale: Record<string, number>;
    byUxTier: Record<string, number>;
    byPersona: Record<string, number>;
  };
};

type WarRoomState = {
  tickCursor: number;
  progressPercent: number;
  wallet: { earnedRevenueUsd: string; cryptoSeedUsd: string };
  githubRepo: string;
  githubCommitShort: string;
};

const TABS = ["overview", "ledger", "vault", "genome", "telemetry", "warroom"] as const;
type Tab = (typeof TABS)[number];

const TAB_LABELS: Record<Tab, string> = {
  overview: "Institutional Overview",
  ledger: "Ledger Auditor",
  vault: "Two-Key Publish Vault",
  genome: "Genome & Environment",
  telemetry: "10-DNA Lab Telemetry",
  warroom: "War-Room Chairman",
};

const din = (n: number) => new Intl.NumberFormat("en-US").format(n);

export default function AdminPage() {
  const [unlocked, setUnlocked] = useState(false);
  const [keyInput, setKeyInput] = useState("");
  const [tab, setTab] = useState<Tab>("overview");
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (getKey()) setUnlocked(true);
  }, []);

  const login = useCallback(() => {
    if (keyInput.trim().length === 0) return;
    setKey(keyInput.trim());
    setUnlocked(true);
    setError(null);
  }, [keyInput]);

  const logout = useCallback(() => {
    setKey(null);
    setOperatorToken(null);
    setUnlocked(false);
  }, []);

  /** Wraps an async call; admin failures surface as an honest banner. */
  const safe = useCallback(async <T,>(fn: () => Promise<T>): Promise<T | null> => {
    try {
      return await fn();
    } catch (e) {
      const message =
        e instanceof AdminApiError ? `${e.code}: ${e.message}` : "Network failure — kernel unreachable";
      setError(message);
      return null;
    }
  }, []);

  if (!unlocked) {
    return (
      <main className="flex min-h-screen items-center justify-center bg-zinc-950 px-4">
        <div className="w-full max-w-md rounded-2xl border border-amber-500/30 bg-zinc-900/70 p-6">
          <h1 className="text-lg font-black tracking-[0.3em] text-amber-400">MAIRON</h1>
          <p className="mt-1 text-xs text-zinc-400">
            Institutional Governance &amp; Audit Console — admin key required
          </p>
          <input
            type="password"
            value={keyInput}
            onChange={(event) => setKeyInput(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === "Enter") login();
            }}
            aria-label="Admin key"
            className="mt-4 w-full rounded-lg border border-zinc-700 bg-zinc-950/80 px-3 py-2 text-sm text-zinc-100 outline-none focus:border-amber-500/60"
          />
          <button
            type="button"
            onClick={login}
            className="mt-3 w-full rounded-lg bg-amber-500 px-4 py-2 text-sm font-semibold text-zinc-950"
          >
            Unlock console
          </button>
          {error ? <p className="mt-2 text-xs text-red-400">{error}</p> : null}
        </div>
      </main>
    );
  }

  return (
    <main className="min-h-screen bg-zinc-950 text-zinc-100">
      <header className="border-b border-amber-500/20 px-4 py-3 md:px-6">
        <div className="mx-auto flex max-w-7xl flex-wrap items-center justify-between gap-3">
          <div>
            <h1 className="text-lg font-black tracking-[0.3em] text-amber-400">MAIRON</h1>
            <p className="text-xs text-zinc-400">Institutional Governance &amp; Audit Console</p>
          </div>
          <button
            type="button"
            onClick={logout}
            className="rounded-lg border border-zinc-600 px-3 py-1.5 text-xs font-semibold text-zinc-300"
          >
            Lock console
          </button>
        </div>
      </header>

      <nav className="border-b border-zinc-800 px-4 py-2 md:px-6">
        <div className="mx-auto flex max-w-7xl flex-wrap gap-2">
          {TABS.map((entry) => (
            <button
              key={entry}
              type="button"
              onClick={() => setTab(entry)}
              className={`rounded-lg px-3 py-1.5 text-xs font-semibold transition ${
                tab === entry
                  ? "bg-amber-500 text-zinc-950"
                  : "border border-zinc-700 text-zinc-300 hover:border-amber-500/60"
              }`}
            >
              {TAB_LABELS[entry]}
            </button>
          ))}
        </div>
      </nav>

      {error ? (
        <p className="mx-auto mt-3 max-w-7xl px-4 text-xs text-red-400 md:px-6">{error}</p>
      ) : null}

      <div className="mx-auto max-w-7xl px-4 py-5 md:px-6">
        {/*
         * Lossless tab switching — every tab stays mounted in the DOM and only
         * its visibility toggles, so zero component-local state (loaded data,
         * form inputs, selections) is ever lost when switching tabs.
         */}
        <div
          hidden={tab !== "overview"}
          style={{ display: tab === "overview" ? "block" : "none" }}
        >
          <OverviewTab safe={safe} />
        </div>
        <div
          hidden={tab !== "ledger"}
          style={{ display: tab === "ledger" ? "block" : "none" }}
        >
          <LedgerTab safe={safe} />
        </div>
        <div
          hidden={tab !== "vault"}
          style={{ display: tab === "vault" ? "block" : "none" }}
        >
          <VaultTab safe={safe} />
        </div>
        <div
          hidden={tab !== "genome"}
          style={{ display: tab === "genome" ? "block" : "none" }}
        >
          <GenomeTab safe={safe} />
        </div>
        <div
          hidden={tab !== "telemetry"}
          style={{ display: tab === "telemetry" ? "block" : "none" }}
        >
          <TelemetryTab safe={safe} />
        </div>
        <div
          hidden={tab !== "warroom"}
          style={{ display: tab === "warroom" ? "block" : "none" }}
        >
          <WarRoomTab safe={safe} />
        </div>
      </div>
    </main>
  );
}

// ── Shared UI atoms ────────────────────────────────────────────────────────────

function Stat({ label, value, hint, danger }: { label: string; value: string; hint?: string; danger?: boolean }) {
  return (
    <div className={`rounded-xl border p-3 ${danger ? "border-red-500/50 bg-red-500/5" : "border-zinc-700/60 bg-zinc-900/60"}`}>
      <p className="text-[11px] uppercase tracking-widest text-zinc-500">{label}</p>
      <p className={`mt-1 text-xl font-bold ${danger ? "text-red-300" : "text-zinc-100"}`}>{value}</p>
      {hint ? <p className="mt-0.5 text-[11px] text-zinc-500">{hint}</p> : null}
    </div>
  );
}

// ── Tab 1 — Institutional overview ─────────────────────────────────────────────

function OverviewTab({ safe }: { safe: <T>(fn: () => Promise<T>) => Promise<T | null> }) {
  const [health, setHealth] = useState<Health | null>(null);
  const [overview, setOverview] = useState<Overview | null>(null);
  const [economy, setEconomy] = useState<Economy | null>(null);

  const load = useCallback(() => {
    void (async () => {
      setHealth(await safe(() => publicGet<Health>("/health")));
      setOverview(await safe(() => adminGet<Overview>("/admin/overview")));
      setEconomy(await safe(() => publicGet<Economy>("/economy")));
    })();
  }, [safe]);

  useEffect(() => {
    load();
  }, [load]);

  return (
    <section className="grid gap-4">
      <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
        <Stat label="Kernel" value={health ? `${health.service} v${health.version}` : "unreachable"} hint={health?.db} />
        <Stat label="Telegram users" value={overview ? din(overview.users) : "—"} />
        <Stat label="Ledger accounts" value={overview ? din(overview.accounts) : "—"} />
        <Stat label="Escrows" value={overview ? din(overview.escrows) : "—"} hint={overview ? `${overview.hitlPending} HITL pending` : undefined} />
        <Stat label="Trial balance sum" value={economy ? String(economy.trialBalanceSum) : "—"} danger={economy ? !economy.balanced : false} hint={economy?.balanced ? "SUM(Posting.amount) == 0 holds" : "invariant violated"} />
        <Stat label="Escrow locked" value={economy ? din(economy.escrowLocked) : "—"} />
        <Stat label="Minted all-time" value={economy ? din(economy.mintedAllTime) : "—"} />
        <Stat label="AI calls" value={overview ? din(overview.aiCalls) : "—"} hint={health ? `llm ${health.llmConfigured ? health.llmModel ?? "configured" : "off"}` : undefined} />
      </div>
      {overview ? (
        <div className="rounded-xl border border-zinc-700/60 bg-zinc-900/60 p-4">
          <h3 className="text-sm font-semibold uppercase tracking-widest text-amber-400">
            14-day revenue series (Din)
          </h3>
          <div className="mt-3 flex items-end gap-1.5" aria-label="14-day revenue bar chart">
            {overview.revenueSeries.map((point) => {
              const max = Math.max(1, ...overview.revenueSeries.map((r) => r.din));
              return (
                <div key={point.date} className="flex flex-1 flex-col items-center gap-1">
                  <div
                    className="w-full rounded-t bg-amber-500/80"
                    style={{ height: `${Math.max(4, (point.din / max) * 120)}px` }}
                    title={`${point.date}: ${din(point.din)}`}
                  />
                  <span className="text-[9px] text-zinc-500">{point.date.slice(5)}</span>
                </div>
              );
            })}
          </div>
        </div>
      ) : null}
    </section>
  );
}

// ── Tab 2 — Ledger auditor ─────────────────────────────────────────────────────

const ESCROW_MACHINE: Array<[string, string[]]> = [
  ["CREATED", ["FUNDED", "CANCELLED"]],
  ["FUNDED", ["DELIVERED", "DISPUTED", "EXPIRED"]],
  ["DELIVERED", ["RELEASED", "DISPUTED"]],
  ["RELEASED", []],
  ["DISPUTED", ["REFUNDED", "RELEASED"]],
  ["REFUNDED", []],
  ["CANCELLED", []],
  ["EXPIRED", []],
];

function LedgerTab({ safe }: { safe: <T>(fn: () => Promise<T>) => Promise<T | null> }) {
  const [economy, setEconomy] = useState<Economy | null>(null);
  const [tokenInput, setTokenInput] = useState("");
  const [hasSession, setHasSession] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);

  useEffect(() => {
    setHasSession(getOperatorToken() !== null);
  }, []);

  const load = useCallback(() => {
    void (async () => {
      setEconomy(await safe(() => publicGet<Economy>("/economy")));
    })();
  }, [safe]);

  useEffect(() => {
    load();
  }, [load]);

  const attachOperatorSession = useCallback(async () => {
    setNotice(null);
    const tgUserId = tokenInput.trim() || "9001";
    const result = await safe(() => operatorLogin(tgUserId, "console_operator"));
    if (result) {
      setOperatorToken(result.token);
      setHasSession(true);
      setNotice("Operator session attached — escrow actions and war-room overrides are now callable.");
    }
  }, [safe, tokenInput]);

  return (
    <section className="grid gap-4 lg:grid-cols-2">
      <div className="rounded-xl border border-zinc-700/60 bg-zinc-900/60 p-4">
        <h3 className="text-sm font-semibold uppercase tracking-widest text-amber-400">
          Double-entry invariant monitor
        </h3>
        {economy ? (
          <div className="mt-3 grid gap-2 text-xs">
            <p className={`rounded-lg border px-3 py-2 font-mono ${economy.balanced ? "border-emerald-500/50 bg-emerald-500/5 text-emerald-300" : "border-red-500/60 bg-red-500/5 text-red-300"}`}>
              SUM(Posting.amount) = {economy.trialBalanceSum} — {economy.balanced ? "OK" : "VIOLATION"}
            </p>
            <p className="text-zinc-400">Accounts: {din(economy.accountCount)} · open escrows: {din(economy.openEscrows)} · escrow locked: {din(economy.escrowLocked)} Din</p>
            <p className="text-zinc-400">Circulating: {din(economy.circulatingSupply)} · treasury: {din(economy.treasuryBalance)} Din</p>
            <p className="text-zinc-500">
              The monitor polls <code>GET /api/v1/economy</code>; the kernel derives every balance from
              postings (ADR-003) — any non-zero sum is a hard institutional failure.
            </p>
          </div>
        ) : (
          <p className="mt-2 text-xs text-zinc-500">Kernel economy feed unreachable.</p>
        )}
      </div>

      <div className="rounded-xl border border-zinc-700/60 bg-zinc-900/60 p-4">
        <h3 className="text-sm font-semibold uppercase tracking-widest text-amber-400">
          Escrow release / refund controls
        </h3>
        <p className="mt-2 text-xs text-zinc-400">
          Escrow mutations are session-bound citizen routes. Attach an operator session (dev mode)
          to act; without one the console shows the machine read-only — never a decorative button.
        </p>
        <div className="mt-3 flex gap-2">
          <input
            value={tokenInput}
            onChange={(event) => setTokenInput(event.target.value)}
            aria-label="Operator Telegram user id"
            className="w-40 rounded-lg border border-zinc-700 bg-zinc-950/80 px-2 py-1.5 text-xs text-zinc-100"
          />
          <button
            type="button"
            onClick={() => void attachOperatorSession()}
            className="rounded-lg bg-amber-500 px-3 py-1.5 text-xs font-semibold text-zinc-950"
          >
            Attach operator session
          </button>
        </div>
        {notice ? <p className="mt-2 text-xs text-emerald-400">{notice}</p> : null}
        {hasSession ? (
          <p className="mt-2 text-xs text-zinc-400">
            Session attached. Escrow release/refund calls dispatch through the kernel state machine
            (`POST /api/v1/escrows/&#123;id&#125;/release` / `refund`) and fail honestly on a bad transition.
          </p>
        ) : null}
        <div className="mt-3 overflow-hidden rounded-lg border border-zinc-800">
          <table className="w-full text-left text-[11px]">
            <thead className="bg-zinc-950/80 text-zinc-400">
              <tr>
                <th className="px-2 py-1.5">State</th>
                <th className="px-2 py-1.5">Allowed transitions</th>
              </tr>
            </thead>
            <tbody>
              {ESCROW_MACHINE.map(([state, transitions]) => (
                <tr key={state} className="border-t border-zinc-800/60">
                  <td className="px-2 py-1 font-mono text-zinc-200">{state}</td>
                  <td className="px-2 py-1 text-zinc-400">{transitions.length === 0 ? "terminal" : transitions.join(", ")}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </section>
  );
}

// ── Tab 3 — Two-Key Publish Vault ──────────────────────────────────────────────

function VaultTab({ safe }: { safe: <T>(fn: () => Promise<T>) => Promise<T | null> }) {
  const [items, setItems] = useState<HitlView[] | null>(null);
  const [busyId, setBusyId] = useState<string | null>(null);
  const [shaInput, setShaInput] = useState("");
  const [scoreInput, setScoreInput] = useState("");
  const [verdict, setVerdict] = useState<string | null>(null);

  const load = useCallback(() => {
    void (async () => {
      const res = await safe(() => adminGet<{ items?: HitlView[] } | HitlView[]>("/admin/hitl"));
      if (res) setItems(Array.isArray(res) ? res : (res.items ?? []));
    })();
  }, [safe]);

  useEffect(() => {
    load();
  }, [load]);

  const decide = useCallback(
    async (id: string, decision: "approve" | "reject") => {
      setBusyId(id);
      await safe(() => adminPost(`/admin/hitl/${encodeURIComponent(id)}/${decision}`));
      setBusyId(null);
      load();
    },
    [load, safe],
  );

  /** Seal inspector — mirrors the kernel rule: `mairon-seal:<sha256[0..16]>` at REA >= 85. */
  const inspectSeal = useCallback(async () => {
    const sha = shaInput.trim().toLowerCase();
    const score = Number.parseInt(scoreInput, 10);
    if (sha.length !== 64 || Number.isNaN(score)) {
      setVerdict("Input a 64-hex artifact SHA-256 and a numeric REA score.");
      return;
    }
    const digest = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(sha));
    const doubleSha = Array.from(new Uint8Array(digest))
      .map((b) => b.toString(16).padStart(2, "0"))
      .join("");
    const expected = `mairon-seal:${doubleSha.slice(0, 16)}`;
    const gate = score >= 85;
    setVerdict(
      gate
        ? `Gate OPEN (REA ${score} >= 85). Sealable id: ${expected} — mint it via POST /api/v1/supremacy/swe-verify.`
        : `Gate CLOSED (REA ${score} < 85). No seal may be minted for this artifact.`,
    );
  }, [scoreInput, shaInput]);

  return (
    <section className="grid gap-4 lg:grid-cols-2">
      <div className="rounded-xl border border-zinc-700/60 bg-zinc-900/60 p-4">
        <h3 className="text-sm font-semibold uppercase tracking-widest text-amber-400">
          Two-key publish queue (HITL)
        </h3>
        <p className="mt-1 text-xs text-zinc-500">
          Key 1 requests release (creator/agent), key 2 (institution) approves or rejects. A listing
          never ships on one key alone.
        </p>
        <div className="mt-3 space-y-2">
          {items === null ? (
            <p className="text-xs text-zinc-500">Queue unreachable or empty.</p>
          ) : items.length === 0 ? (
            <p className="text-xs text-zinc-500">Queue clear — zero pending decisions.</p>
          ) : (
            items.map((item) => (
              <div key={item.id} className="rounded-lg border border-zinc-800 bg-zinc-950/70 p-2.5 text-xs">
                <p className="flex flex-wrap items-center justify-between gap-2">
                  <span className="font-mono text-zinc-200">{item.kind} · {item.id.slice(0, 14)}...</span>
                  <span
                    className={`rounded-full px-2 py-0.5 text-[10px] font-semibold ${
                      item.status === "PENDING"
                        ? "bg-amber-500/20 text-amber-300"
                        : item.status === "APPROVED"
                          ? "bg-emerald-500/20 text-emerald-300"
                          : "bg-zinc-600/30 text-zinc-300"
                    }`}
                  >
                    {item.status}
                  </span>
                </p>
                <p className="mt-1 text-zinc-400">
                  escrow {item.escrowId.slice(0, 12)}... · {din(item.amount)} Din · requester {item.requesterId}
                </p>
                {item.status === "PENDING" ? (
                  <div className="mt-2 flex gap-2">
                    <button
                      type="button"
                      onClick={() => void decide(item.id, "approve")}
                      disabled={busyId === item.id}
                      className="rounded bg-emerald-500 px-3 py-1 font-semibold text-zinc-950 disabled:opacity-50"
                    >
                      Approve (key 2)
                    </button>
                    <button
                      type="button"
                      onClick={() => void decide(item.id, "reject")}
                      disabled={busyId === item.id}
                      className="rounded border border-red-500/60 px-3 py-1 font-semibold text-red-300 disabled:opacity-50"
                    >
                      Reject
                    </button>
                  </div>
                ) : null}
              </div>
            ))
          )}
        </div>
      </div>

      <div className="rounded-xl border border-zinc-700/60 bg-zinc-900/60 p-4">
        <h3 className="text-sm font-semibold uppercase tracking-widest text-amber-400">
          MaironVerifiedSeal gate inspector
        </h3>
        <p className="mt-1 text-xs text-zinc-500">
          Kernel rule: a seal exists only at `REA {'>='} 85`, and its id is
          `mairon-seal:&lt;sha256(artifact_sha256)[0..16]&gt;` — recomputable, therefore auditable.
        </p>
        <input
          value={shaInput}
          onChange={(event) => setShaInput(event.target.value)}
          aria-label="Artifact SHA-256"
          className="mt-3 w-full rounded-lg border border-zinc-700 bg-zinc-950/80 px-2 py-1.5 font-mono text-xs text-zinc-100"
        />
        <input
          value={scoreInput}
          onChange={(event) => setScoreInput(event.target.value)}
          aria-label="REA score"
          className="mt-2 w-32 rounded-lg border border-zinc-700 bg-zinc-950/80 px-2 py-1.5 text-xs text-zinc-100"
        />
        <button
          type="button"
          onClick={() => void inspectSeal()}
          className="mt-2 rounded-lg bg-amber-500 px-4 py-1.5 text-xs font-semibold text-zinc-950"
        >
          Inspect gate
        </button>
        {verdict ? <p className="mt-2 rounded-lg border border-zinc-800 bg-zinc-950/70 p-2 text-xs text-zinc-300">{verdict}</p> : null}
      </div>
    </section>
  );
}

// ── Tab 4 — Genome & environment ───────────────────────────────────────────────

function GenomeTab({ safe }: { safe: <T>(fn: () => Promise<T>) => Promise<T | null> }) {
  const [matrix, setMatrix] = useState<GenomeMatrix | null>(null);
  const [override, setOverride] = useState("");
  const [routing, setRouting] = useState<string | null>(null);
  const [envPresent, setEnvPresent] = useState<Record<string, boolean>>({});

  const load = useCallback(() => {
    void (async () => {
      const res = await safe(() => publicGet<GenomeMatrix>("/monopoly/genome-matrix"));
      if (res) setMatrix(res);
    })();
  }, [safe]);

  useEffect(() => {
    load();
  }, [load]);

  const routeModel = useCallback(async () => {
    const result = await fetch(resolveKernelUrl() + "/api/v1/monopoly/route-model", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        taskPrompt: override.length > 0 ? `Administrative override: ${override}` : "Autonomous administrative routing probe",
        agentMode: override.trim().length === 0,
        manualModelOverride: override.trim().length > 0 ? override.trim() : null,
      }),
    });
    const body = (await result.json()) as {
      selectedModel?: string;
      selectionMode?: string;
      routedDomain?: string;
      trinityAvatar?: string;
      error?: { message?: string };
    };
    setRouting(
      body.selectedModel
        ? `${body.selectedModel} (${body.selectionMode}, domain ${body.routedDomain}, avatar ${body.trinityAvatar})`
        : `Kernel refused: ${body.error?.message ?? `HTTP ${result.status}`}`,
    );
  }, [override, safe]);

  /** Browser-visible posture — NEXT_PUBLIC_* keys are inlined; others show tier coverage. */
  useEffect(() => {
    const probe: Record<string, boolean> = {};
    for (const spec of MAIRON_ENV_SPEC) {
      const value = (process.env as Record<string, string | undefined>)[spec.key];
      probe[spec.key] = typeof value === "string" && value.length > 0;
    }
    setEnvPresent(probe);
  }, []);

  return (
    <section className="grid gap-4">
      <div className="rounded-xl border border-zinc-700/60 bg-zinc-900/60 p-4">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <h3 className="text-sm font-semibold uppercase tracking-widest text-amber-400">
            45-model genome override matrix
          </h3>
          <span className="text-xs text-zinc-400">
            {matrix ? `${matrix.totalFilesVerified}/${matrix.expectedFileCount} files verified · modes: ${matrix.selectionModes.join(" / ")}` : "matrix unreachable"}
          </span>
        </div>
        {matrix ? (
          <div className="mt-3 overflow-x-auto">
            <table className="w-full text-left text-[11px]">
              <thead className="text-zinc-400">
                <tr>
                  <th className="px-2 py-1.5">Model</th>
                  <th className="px-2 py-1.5">Specialty</th>
                  <th className="px-2 py-1.5">Files</th>
                  <th className="px-2 py-1.5">Trinity avatar</th>
                  <th className="px-2 py-1.5">Verified against</th>
                </tr>
              </thead>
              <tbody>
                {matrix.models.map((model) => (
                  <tr key={model.modelId} className="border-t border-zinc-800/60">
                    <td className="px-2 py-1 font-mono text-zinc-200">{model.modelId}</td>
                    <td className="px-2 py-1 text-zinc-400">{model.specialty}</td>
                    <td className={`px-2 py-1 ${model.isComplete ? "text-emerald-300" : "text-amber-300"}`}>
                      {model.filesVerified}/{model.canonicalFiles.length}
                    </td>
                    <td className="px-2 py-1 text-zinc-400">{model.trinityAvatar}</td>
                    <td className="px-2 py-1 text-zinc-500">{model.verifiedAgainst}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : (
          <p className="mt-2 text-xs text-zinc-500">Sync once the kernel is reachable.</p>
        )}
        <div className="mt-3 flex flex-wrap items-center gap-2 text-xs">
          <input
            value={override}
            onChange={(event) => setOverride(event.target.value)}
            aria-label="Administrative model override"
            className="w-64 rounded-lg border border-zinc-700 bg-zinc-950/80 px-2 py-1.5 text-zinc-100"
          />
          <button
            type="button"
            onClick={() => void routeModel()}
            className="rounded-lg border border-amber-500/60 px-3 py-1.5 font-semibold text-amber-300"
          >
            Route task (override / auto)
          </button>
          {routing ? <span className="text-zinc-300">{routing}</span> : null}
        </div>
      </div>

      <div className="rounded-xl border border-zinc-700/60 bg-zinc-900/60 p-4">
        <h3 className="text-sm font-semibold uppercase tracking-widest text-amber-400">
          69-key environment readiness monitor (P0 -&gt; P5)
        </h3>
        <div className="mt-3 grid gap-3 md:grid-cols-2 lg:grid-cols-3">
          {MAIRON_ENV_TIERS.map((tier) => {
            const rows = MAIRON_ENV_SPEC.filter((spec) => spec.tier === tier);
            const present = rows.filter((row) => envPresent[row.key]).length;
            return (
              <div key={tier} className="rounded-lg border border-zinc-800 bg-zinc-950/70 p-2.5">
                <p className="text-xs font-semibold text-zinc-200">
                  {tier} <span className="text-zinc-500">({present}/{rows.length} visible client-side)</span>
                </p>
                <ul className="mt-1 space-y-0.5 text-[10px]">
                  {rows.map((row) => (
                    <li key={row.key} className="flex items-start justify-between gap-1">
                      <span className="font-mono text-zinc-400">{row.key}</span>
                      <span className={envPresent[row.key] ? "text-emerald-400" : "text-zinc-600"}>
                        {envPresent[row.key] ? "set" : row.required ? "required" : "unset"}
                      </span>
                    </li>
                  ))}
                </ul>
              </div>
            );
          })}
        </div>
        <p className="mt-2 text-[11px] text-zinc-500">
          The browser sees only non-secret, inlined keys; secret-tier posture is audited server-side
          against the same schema exported from `@kian/config` — one source of truth, both planes.
        </p>
      </div>
    </section>
  );
}

// ── Tab 5 — 10-DNA telemetry ───────────────────────────────────────────────────

function TelemetryTab({ safe }: { safe: <T>(fn: () => Promise<T>) => Promise<T | null> }) {
  const [data, setData] = useState<TelemetryResponse | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  const load = useCallback(() => {
    void (async () => {
      const res = await safe(() => adminGet<TelemetryResponse>("/admin/lab-telemetry"));
      if (res) setData(res);
    })();
  }, [safe]);

  useEffect(() => {
    load();
  }, [load]);

  const exportDataset = useCallback(
    (format: "sft" | "dpo") => {
      if (!data) return;
      const lines: string[] = [];
      for (const entry of data.entries) {
        if (!entry.usableForTraining) continue;
        const line =
          format === "sft"
            ? {
                id: entry.id,
                dnaCode: entry.dnaCode,
                format,
                prompt: entry.promptText,
                chosen: entry.responseText,
                reaScore: entry.reaScore,
              }
            : {
                id: entry.id,
                dnaCode: entry.dnaCode,
                format,
                prompt: entry.promptText,
                chosen: entry.responseText,
                rejected: "",
                reaScore: entry.reaScore,
              };
        lines.push(JSON.stringify(line));
      }
      const blob = new Blob([lines.join("\n") + "\n"], { type: "application/jsonl" });
      const url = URL.createObjectURL(blob);
      const anchor = document.createElement("a");
      anchor.href = url;
      anchor.download = `mairon-lab-${format}-dataset.jsonl`;
      anchor.click();
      URL.revokeObjectURL(url);
      setNotice(`Exported ${lines.length} trainable JSONL lines (${format.toUpperCase()}).`);
    },
    [data],
  );

  return (
    <section className="grid gap-4">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div className="grid gap-2 text-xs sm:grid-cols-4">
          <Stat label="Seeded logs" value={data ? din(data.dashboard.totalEntries) : "—"} />
          <Stat label="Avg REA" value={data ? data.dashboard.avgReaScore.toFixed(1) : "—"} />
          <Stat label="Min / Max REA" value={data ? `${data.dashboard.minReaScore} / ${data.dashboard.maxReaScore}` : "—"} />
          <Stat label="Cohorts" value={data ? din(data.dashboard.cohortHealth.length) : "—"} />
        </div>
        <div className="flex gap-2">
          <button
            type="button"
            onClick={() => exportDataset("sft")}
            className="rounded-lg bg-amber-500 px-3 py-1.5 text-xs font-semibold text-zinc-950"
          >
            Export SFT JSONL
          </button>
          <button
            type="button"
            onClick={() => exportDataset("dpo")}
            className="rounded-lg border border-amber-500/60 px-3 py-1.5 text-xs font-semibold text-amber-300"
          >
            Export DPO JSONL
          </button>
        </div>
      </div>
      {notice ? <p className="text-xs text-emerald-400">{notice}</p> : null}
      <div className="overflow-x-auto rounded-xl border border-zinc-700/60 bg-zinc-900/60">
        <table className="w-full text-left text-[11px]">
          <thead className="bg-zinc-950/80 text-zinc-400">
            <tr>
              <th className="px-2 py-1.5">DNA</th>
              <th className="px-2 py-1.5">Handle</th>
              <th className="px-2 py-1.5">Hour</th>
              <th className="px-2 py-1.5">UX tier</th>
              <th className="px-2 py-1.5">Persona</th>
              <th className="px-2 py-1.5">Model</th>
              <th className="px-2 py-1.5">REA</th>
              <th className="px-2 py-1.5">Latency</th>
              <th className="px-2 py-1.5">Trainable</th>
              <th className="px-2 py-1.5">Prompt excerpt</th>
            </tr>
          </thead>
          <tbody>
            {data?.entries.slice(0, 64).map((entry) => (
              <tr key={entry.id} className="border-t border-zinc-800/60">
                <td className="px-2 py-1 font-mono text-amber-300">{entry.dnaCode}</td>
                <td className="px-2 py-1 text-zinc-300">{entry.userHandle}</td>
                <td className="px-2 py-1 text-zinc-400">{entry.hourOffset}</td>
                <td className="px-2 py-1 text-zinc-400">{entry.uxTier}</td>
                <td className="px-2 py-1 text-zinc-400">{entry.grokPersona}</td>
                <td className="px-2 py-1 text-zinc-400">{entry.modelSlug}</td>
                <td className={`px-2 py-1 ${entry.reaScore >= 85 ? "text-emerald-300" : "text-zinc-300"}`}>{entry.reaScore}</td>
                <td className="px-2 py-1 text-zinc-500">{entry.latencyMs} ms</td>
                <td className="px-2 py-1">{entry.usableForTraining ? "yes" : "no"}</td>
                <td className="max-w-[280px] truncate px-2 py-1 text-zinc-500">{entry.promptText}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </section>
  );
}

// ── Tab 6 — War-Room chairman console ─────────────────────────────────────────

function WarRoomTab({ safe }: { safe: <T>(fn: () => Promise<T>) => Promise<T | null> }) {
  const [state, setState] = useState<WarRoomState | null>(null);
  const [directive, setDirective] = useState("");
  const [replies, setReplies] = useState<Array<{ speakerNameEn: string; messageEn: string }>>([]);
  const [busy, setBusy] = useState(false);
  const [hasSession, setHasSession] = useState(false);

  useEffect(() => {
    setHasSession(getOperatorToken() !== null);
  }, []);

  const load = useCallback(() => {
    void (async () => {
      const res = await safe(() => publicGet<WarRoomState>("/war-room/state"));
      if (res) setState(res);
    })();
  }, [safe]);

  useEffect(() => {
    load();
  }, [load]);

  const broadcast = useCallback(async () => {
    if (directive.trim().length === 0) return;
    setBusy(true);
    const res = await safe(() =>
      operatorRequest<{ replies: Array<{ speakerNameEn: string; messageEn: string }> }>("/war-room/directive", {
        method: "POST",
        body: { directiveText: directive.trim(), locale: "en", triggeredTick: state?.tickCursor ?? 0 },
      }),
    );
    if (res) {
      setReplies(res.replies);
      setDirective("");
    }
    setBusy(false);
  }, [directive, safe, state?.tickCursor]);

  const resetSimulation = useCallback(async () => {
    setBusy(true);
    const res = await safe(() =>
      operatorRequest<{ deletedTicks: number; deletedDirectives: number }>("/war-room/reset", {
        method: "POST",
        body: {},
      }),
    );
    if (res) {
      await safe(() => operatorRequest<unknown>("/war-room/state"));
      load();
    }
    setBusy(false);
  }, [load, safe]);

  return (
    <section className="grid gap-4 lg:grid-cols-2">
      <div className="rounded-xl border border-zinc-700/60 bg-zinc-900/60 p-4">
        <h3 className="text-sm font-semibold uppercase tracking-widest text-amber-400">
          Simulation posture
        </h3>
        {state ? (
          <div className="mt-3 grid gap-2 text-xs text-zinc-400">
            <p>
              tick {state.tickCursor}/24 · progress {state.progressPercent}% · earned{" "}
              <span className="font-mono text-emerald-300">{state.wallet.earnedRevenueUsd}</span> on the{" "}
              <span className="font-mono">{state.wallet.cryptoSeedUsd}</span> seed
            </p>
            <p className="text-zinc-500">
              provenance {state.githubRepo} @ {state.githubCommitShort}
            </p>
            <div className="h-2 w-full overflow-hidden rounded-full bg-zinc-800">
              <div className="h-full rounded-full bg-amber-500" style={{ width: `${state.progressPercent}%` }} />
            </div>
          </div>
        ) : (
          <p className="mt-2 text-xs text-zinc-500">War-room state unreachable.</p>
        )}
        {!hasSession ? (
          <p className="mt-3 text-xs text-amber-300">
            Attach an operator session in the Ledger Auditor tab first — chairman routes are
            session-bound by design.
          </p>
        ) : null}
      </div>

      <div className="rounded-xl border border-zinc-700/60 bg-zinc-900/60 p-4">
        <h3 className="text-sm font-semibold uppercase tracking-widest text-amber-400">
          Chairman override &amp; reset
        </h3>
        <textarea
          value={directive}
          onChange={(event) => setDirective(event.target.value)}
          rows={3}
          aria-label="Chairman directive"
          className="mt-3 w-full rounded-lg border border-zinc-700 bg-zinc-950/80 p-3 text-sm text-zinc-100 outline-none focus:border-amber-500/60"
        />
        <div className="mt-2 flex gap-2">
          <button
            type="button"
            onClick={() => void broadcast()}
            disabled={busy || !hasSession || directive.trim().length === 0}
            className="rounded-lg bg-amber-500 px-4 py-2 text-sm font-semibold text-zinc-950 disabled:opacity-50"
          >
            {busy ? "Working..." : "Broadcast directive"}
          </button>
          <button
            type="button"
            onClick={() => void resetSimulation()}
            disabled={busy || !hasSession}
            className="rounded-lg border border-red-500/60 px-4 py-2 text-sm font-semibold text-red-300 disabled:opacity-50"
          >
            Reset simulation to Day 0
          </button>
        </div>
        {replies.length > 0 ? (
          <ul className="mt-3 space-y-1.5 text-xs">
            {replies.map((reply) => (
              <li key={reply.speakerNameEn} className="rounded-lg border border-zinc-800 bg-zinc-950/70 p-2">
                <span className="font-semibold text-zinc-100">{reply.speakerNameEn}:</span>{" "}
                <span className="text-zinc-300">{reply.messageEn}</span>
              </li>
            ))}
          </ul>
        ) : null}
      </div>
    </section>
  );
}
