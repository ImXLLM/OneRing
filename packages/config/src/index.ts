/**
 * MAIRON — canonical configuration constants, the priority-ordered environment
 * schema, and runtime endpoint resolvers.
 *
 * Every deployment-facing knob of the platform is declared here exactly once:
 * the sovereign brand identity, the Base-8453 settlement constants, the
 * canonical endpoint table, and the 69-key `P0 -> P5` environment schema whose
 * names match what the kernel and the web clients actually read.
 */

// ── Sovereign brand identity ───────────────────────────────────────────────────

/** The one and only public brand of this platform. */
export const MAIRON_BRAND = {
  name: "MAIRON",
  fullName: "MAIRON — Sovereign Intelligence, Quantitative Execution & Autonomous Capital OS",
  shortTagline: "Sovereign Intelligence, Quantitative Execution & Autonomous Capital",
  kernelService: "mairon-kernel-server",
  defaultLocale: "en",
  defaultDirection: "ltr",
} as const;

// ── Base-8453 settlement constants ─────────────────────────────────────────────

/** Base Mainnet — the sole sovereign settlement chain. */
export const BASE_CHAIN_ID = 8453 as const;

/** Base-native USDC contract address. */
export const BASE_USDC_CONTRACT = "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913" as const;

/** The sovereign platform token symbol. */
export const KIA_TOKEN_SYMBOL = "$0xKIA" as const;

/** Telegram Stars currency code — the instant-settlement rail. */
export const TELEGRAM_STARS_CURRENCY = "XTR" as const;

/** Charter rates, in basis points. */
export const MAIRON_CHARTER_BPS = {
  platformTakeUnreferred: 2_000,
  platformTakeReferred: 1_000,
  creatorPayout: 8_000,
  affiliateReferred: 1_000,
  instantBoostTreasury: 10_000,
  settlementSpread: 150,
} as const;

/** Instant boosts route 0% to the creator — Stream 3 is a 100%-margin treasury purchase. */
export const INSTANT_BOOST_CREATOR_BPS = 0 as const;

/** Instant boosts route 100% (10000 bps) to the MAIRON Sovereign Treasury. */
export const INSTANT_BOOST_TREASURY_BPS = 10_000 as const;

/** Capital reserve partition — the honest Day-0 figures. */
export const MAIRON_CAPITAL_RESERVE = {
  cryptoSeedMicros: 33_000_000,
  cashSeedCents: 1_500,
  paperReserveLockedCents: 35_000,
} as const;

/** Quality gates every subsystem must pass. */
export const MAIRON_GATES = {
  reaMinScore: 85,
  quantLiveSharpe: 1.2,
  trinityAvatarIq: { alpha: 136, commandCenter: 139, mairon: 150 },
  genomeCategories: 5,
  genomeSlotsPerCategory: 9,
  genomeTotalModels: 45,
  warRoomTicks: 24,
} as const;

// ── Canonical endpoint table ───────────────────────────────────────────────────

/** Every kernel route the MAIRON clients are allowed to call. */
export const MAIRON_ENDPOINTS = {
  health: "GET /api/v1/health",
  economy: "GET /api/v1/economy",
  trinityAlpha: "POST /api/v1/trinity/alpha",
  trinityOrchestrate: "POST /api/v1/trinity/orchestrate",
  builderSession: "GET /api/v1/builder/sessions/{id}",
  builderPublish: "POST /api/v1/builder/sessions/{id}/publish",
  ardaFlags: "GET /api/v1/arda/flags",
  ardaLocalizeNow: "POST /api/v1/arda/localize-now",
  i18nEcosystemMatrix: "GET /api/v1/i18n/ecosystem-matrix",
  i18nApplyPatch: "POST /api/v1/i18n/apply-patch",
  studioUxProfiles: "GET /api/v1/studio/ux-profiles",
  studioGenerate: "POST /api/v1/studio/generate",
  studioPortfolioCatalog: "GET /api/v1/studio/portfolio-catalog",
  supremacySurgicalPatch: "POST /api/v1/supremacy/surgical-patch",
  supremacyCheckpointFork: "POST /api/v1/supremacy/checkpoint/fork",
  supremacyWorkflowExecute: "POST /api/v1/supremacy/workflow/execute",
  supremacyMcp: "POST /api/v1/supremacy/mcp",
  supremacySweVerify: "POST /api/v1/supremacy/swe-verify",
  mcpTools: "GET /api/v1/mcp/tools",
  mcpInvoke: "POST /api/v1/mcp/invoke",
  quantVenues: "GET /api/v1/quant/venues",
  quantVibeBacktest: "POST /api/v1/quant/vibe-backtest",
  quantStrategies: "GET /api/v1/quant/strategies",
  monopolyGenomeMatrix: "GET /api/v1/monopoly/genome-matrix",
  monopolyRouteModel: "POST /api/v1/monopoly/route-model",
  monopolyCharterAndBase: "GET /api/v1/monopoly/charter-and-base",
  monopolySimulateSplit: "POST /api/v1/monopoly/simulate-split",
  hudPresets: "GET /api/v1/hud/presets",
  hudSynthesize: "POST /api/v1/hud/synthesize",
  grokUiBundle: "GET /api/v1/grok/ui-bundle",
  grokChatTurn: "POST /api/v1/grok/chat-turn",
  warRoomState: "GET /api/v1/war-room/state",
  warRoomStep: "POST /api/v1/war-room/step",
  warRoomDirective: "POST /api/v1/war-room/directive",
  warRoomReset: "POST /api/v1/war-room/reset",
  warRoomExplainSimple: "GET /api/v1/war-room/explain-simple",
  warRoomHtml: "GET /war-room",
  neuralHudHtml: "GET /neural-hud",
  adminOverview: "GET /api/v1/admin/overview",
  adminAudit: "GET /api/v1/admin/audit",
  adminLabTelemetry: "GET /api/v1/admin/lab-telemetry",
  adminHitl: "GET /api/v1/admin/hitl",
  adminHitlApprove: "POST /api/v1/admin/hitl/{id}/approve",
  adminHitlReject: "POST /api/v1/admin/hitl/{id}/reject",
  adminDeposits: "GET /api/v1/admin/deposits",
  adminWithdrawals: "GET /api/v1/admin/withdrawals",
  adminProducts: "GET /api/v1/admin/products",
} as const;

// ── Runtime endpoint resolvers ─────────────────────────────────────────────────

/** True in a browser runtime (fetch + window available). */
export function isBrowserRuntime(): boolean {
  return typeof window !== "undefined" && typeof window.location !== "undefined";
}

function firstDefined(...values: Array<string | undefined>): string | undefined {
  for (const v of values) {
    if (typeof v === "string" && v.trim().length > 0) return v;
  }
  return undefined;
}

/**
 * Resolve the kernel API base (origin of `/api/v1`). Priority:
 * explicit override -> MAIRON_API_BASE_URL -> NEXT_PUBLIC_KIAN_API ->
 * NEXT_PUBLIC_KERNEL_URL + `/api/v1` -> same-origin `/api/v1` -> local kernel.
 */
export function resolveApiBase(override?: string): string {
  const explicit = firstDefined(
    override,
    typeof process !== "undefined" ? process.env?.MAIRON_API_BASE_URL : undefined,
    typeof process !== "undefined" ? process.env?.NEXT_PUBLIC_KIAN_API : undefined,
  );
  if (explicit) return explicit.replace(/\/$/, "");
  const kernelUrl = firstDefined(
    typeof process !== "undefined" ? process.env?.NEXT_PUBLIC_KERNEL_URL : undefined,
    typeof process !== "undefined" ? process.env?.KIAN_KERNEL_URL : undefined,
  );
  if (kernelUrl) return `${kernelUrl.replace(/\/$/, "")}/api/v1`;
  if (isBrowserRuntime()) return `${window.location.origin}/api/v1`;
  return "http://127.0.0.1:8088/api/v1";
}

/** Resolve the kernel origin (no `/api/v1` suffix). */
export function resolveKernelUrl(override?: string): string {
  const apiBase = resolveApiBase(override);
  return apiBase.replace(/\/api\/v1$/, "");
}

/** Resolve the standalone Founder War-Room dashboard URL. */
export function resolveWarRoomUrl(override?: string): string {
  return `${resolveKernelUrl(override)}/war-room`;
}

/** Resolve the standalone 3D Neural-HUD dashboard URL. */
export function resolveNeuralHudUrl(override?: string): string {
  return `${resolveKernelUrl(override)}/neural-hud`;
}

// ── The 69-key P0 -> P5 environment schema ─────────────────────────────────────

export type MaironEnvTier = "P0" | "P1" | "P2" | "P3" | "P4" | "P5";

export type MaironEnvKeySpec = {
  readonly key: string;
  readonly tier: MaironEnvTier;
  readonly description: string;
  readonly reader: "kernel" | "web" | "admin" | "deployment";
  readonly required: boolean;
  readonly secret: boolean;
};

/**
 * The complete, priority-ordered environment schema — 69 keys.
 * P0 kernel core, P1 identity/session, P2 AI primary ladder, P3 AI pools,
 * P4 economy + web client, P5 aliases + deployment platform.
 */
export const MAIRON_ENV_SPEC: readonly MaironEnvKeySpec[] = [
  // order mirrors .env.example / railway.env.example index-for-index (0..68)
  // P0 — critical boot, network & persistent storage (69)
  { key: "KIAN_DATABASE_URL", tier: "P0", description: "SQLite database path or URL of the kernel ledger", reader: "kernel", required: true, secret: false },
  { key: "KIAN_HOST", tier: "P0", description: "Kernel bind host (default 127.0.0.1)", reader: "kernel", required: true, secret: false },
  { key: "KIAN_PORT", tier: "P0", description: "Kernel bind port (default 8088)", reader: "kernel", required: true, secret: false },
  { key: "KIAN_SEED_DATABASE_PATH", tier: "P0", description: "Seed database baked into the deployment image", reader: "kernel", required: false, secret: false },
  { key: "KIAN_BACKUP_DIR", tier: "P0", description: "VACUUM INTO backup folder", reader: "kernel", required: false, secret: false },
  { key: "KIAN_KEEP_BACKUPS", tier: "P0", description: "Backup retention count (default 7)", reader: "kernel", required: false, secret: false },
  { key: "KIAN_ADMIN_KEY", tier: "P0", description: "Admin control-plane key; absent = admin endpoints fail closed", reader: "kernel", required: true, secret: true },
  { key: "KIAN_TRUST_PROXY", tier: "P0", description: "Trust X-Forwarded-For for rate limiting", reader: "kernel", required: false, secret: false },
  { key: "KIAN_CORS_ORIGIN", tier: "P0", description: "Allowed browser origins, comma-separated", reader: "kernel", required: true, secret: false },
  { key: "KIAN_DEV_MODE", tier: "P0", description: "Developer mode gate (only 1/true/yes enables dev-login and faucet)", reader: "kernel", required: false, secret: false },
  { key: "OPENCODE_API_KEY", tier: "P0", description: "Optional OpenCode Zen key (absent = Bearer public lane)", reader: "kernel", required: false, secret: true },
  { key: "KIAN_AI_BASE_URL", tier: "P0", description: "Default AI provider base URL", reader: "kernel", required: false, secret: false },
  { key: "KIAN_AI_API_KEY", tier: "P0", description: "Default AI provider API key", reader: "kernel", required: false, secret: true },
  { key: "KIAN_AI_MODEL", tier: "P0", description: "Default AI model slug", reader: "kernel", required: false, secret: false },
  { key: "KIAN_AI_FALLBACK_MODEL", tier: "P0", description: "Fallback provider model slug", reader: "kernel", required: false, secret: false },
  { key: "KIAN_AI_FALLBACK_URL", tier: "P0", description: "Fallback provider base URL", reader: "kernel", required: false, secret: false },
  { key: "KIAN_AI_FALLBACK_KEY", tier: "P0", description: "Fallback provider API key", reader: "kernel", required: false, secret: true },
  { key: "KIAN_AI_PRIMARY_URL", tier: "P0", description: "Primary provider base URL", reader: "kernel", required: false, secret: false },
  { key: "KIAN_AI_PRIMARY_KEY", tier: "P0", description: "Primary provider API key", reader: "kernel", required: false, secret: true },
  { key: "KIAN_AI_PRIMARY_MODEL", tier: "P0", description: "Primary provider model slug", reader: "kernel", required: false, secret: false },
  { key: "KIAN_AI_SECONDARY_URL", tier: "P0", description: "Secondary provider base URL", reader: "kernel", required: false, secret: false },
  { key: "KIAN_AI_SECONDARY_KEY", tier: "P0", description: "Secondary provider API key", reader: "kernel", required: false, secret: true },
  { key: "KIAN_AI_SECONDARY_MODEL", tier: "P0", description: "Secondary provider model slug", reader: "kernel", required: false, secret: false },
  { key: "KIAN_AI_TERTIARY_URL", tier: "P0", description: "Tertiary provider base URL", reader: "kernel", required: false, secret: false },
  { key: "KIAN_AI_TERTIARY_KEY", tier: "P0", description: "Tertiary provider API key", reader: "kernel", required: false, secret: true },
  { key: "KIAN_AI_TERTIARY_MODEL", tier: "P0", description: "Tertiary provider model slug", reader: "kernel", required: false, secret: false },
  { key: "KIAN_AI_COARENA_URL", tier: "P0", description: "CoArena base URL", reader: "kernel", required: false, secret: false },
  { key: "KIAN_AI_COARENA_KEY", tier: "P0", description: "CoArena pool key", reader: "kernel", required: false, secret: true },
  { key: "KIAN_AI_COARENA_KEYS", tier: "P0", description: "CoArena pooled keys, comma-separated", reader: "kernel", required: false, secret: true },
  { key: "KIAN_AI_COARENA_MODELS", tier: "P0", description: "CoArena pooled models, comma-separated", reader: "kernel", required: false, secret: false },
  { key: "KIAN_AI_OPENROUTER_URL", tier: "P0", description: "OpenRouter base URL", reader: "kernel", required: false, secret: false },
  { key: "KIAN_AI_OPENROUTER_KEY", tier: "P0", description: "OpenRouter pool key", reader: "kernel", required: false, secret: true },
  { key: "KIAN_AI_OPENROUTER_KEYS", tier: "P0", description: "OpenRouter pooled keys, comma-separated", reader: "kernel", required: false, secret: true },
  { key: "KIAN_AI_OPENROUTER_MODELS", tier: "P0", description: "OpenRouter pooled models, comma-separated", reader: "kernel", required: false, secret: false },
  { key: "KIAN_AI_ROUTER9_URL", tier: "P0", description: "Router9 base URL", reader: "kernel", required: false, secret: false },
  { key: "KIAN_AI_ROUTER9_KEY", tier: "P0", description: "Router9 pool key", reader: "kernel", required: false, secret: true },
  { key: "KIAN_AI_ROUTER9_KEYS", tier: "P0", description: "Router9 pooled keys, comma-separated", reader: "kernel", required: false, secret: true },
  { key: "KIAN_AI_ROUTER9_MODELS", tier: "P0", description: "Router9 pooled models, comma-separated", reader: "kernel", required: false, secret: false },
  { key: "KIAN_AI_DAILY_TOKENS", tier: "P0", description: "Daily token budget across providers", reader: "kernel", required: false, secret: false },
  { key: "KIAN_BOT_TOKEN", tier: "P0", description: "Telegram bot token; absent = Telegram identity disabled", reader: "kernel", required: false, secret: true },
  { key: "KIAN_BOT_WEBHOOK_SECRET", tier: "P0", description: "Telegram webhook secret token", reader: "kernel", required: false, secret: true },
  { key: "KIAN_MINIAPP_URL", tier: "P0", description: "Mini-app base URL used in bot deep links", reader: "kernel", required: false, secret: false },
  { key: "KIAN_KERNEL_URL", tier: "P0", description: "Kernel origin used by server-side clients", reader: "kernel", required: false, secret: false },
  { key: "NEXT_PUBLIC_KIAN_API", tier: "P0", description: "Kernel API base exposed to the browser clients", reader: "web", required: false, secret: false },
  { key: "NEXT_PUBLIC_TELEGRAM_BOT_USERNAME", tier: "P0", description: "Bot username for deep links in the Web OS", reader: "web", required: false, secret: false },
  { key: "KIAN_TON_TREASURY", tier: "P0", description: "TON treasury address for deposit proofs", reader: "kernel", required: false, secret: false },
  { key: "KIAN_TON_TO_DIN", tier: "P0", description: "TON to Din conversion rate (default 1000)", reader: "kernel", required: false, secret: false },
  { key: "KIAN_USDT_TO_DIN", tier: "P0", description: "USDT to Din conversion rate (default 1000)", reader: "kernel", required: false, secret: false },
  { key: "KIAN_TREASURY_TRX", tier: "P0", description: "Per-chain treasury address: TRX", reader: "kernel", required: false, secret: false },
  { key: "KIAN_TREASURY_SOL", tier: "P0", description: "Per-chain treasury address: SOL", reader: "kernel", required: false, secret: false },
  { key: "KIAN_TREASURY_TON", tier: "P0", description: "Per-chain treasury address: TON", reader: "kernel", required: false, secret: false },
  { key: "KIAN_TREASURY_BSC", tier: "P0", description: "Per-chain treasury address: BSC", reader: "kernel", required: false, secret: false },
  { key: "KIAN_NATION_QUORUM", tier: "P0", description: "Parliament quorum for bill execution (default 3)", reader: "kernel", required: false, secret: false },
  { key: "KIAN_SESSION_TTL_HOURS", tier: "P0", description: "Bearer session TTL in hours (default 168)", reader: "kernel", required: false, secret: false },
  { key: "KIAN_RATE_LIMIT", tier: "P0", description: "Requests per 60-second window per identity (default 120)", reader: "kernel", required: false, secret: false },
  { key: "RUST_LOG", tier: "P0", description: "Rust tracing filter for the kernel process", reader: "deployment", required: false, secret: false },
  { key: "RAILPACK_CARGO_WORKSPACE", tier: "P0", description: "Railpack cargo workspace root for the deploy builder", reader: "deployment", required: false, secret: false },
  { key: "NEXT_PUBLIC_SITE_URL", tier: "P0", description: "Web OS canonical site URL for metadata", reader: "web", required: false, secret: false },
  { key: "NEXT_PUBLIC_KIAN_URL", tier: "P0", description: "Legacy web client kernel origin alias", reader: "web", required: false, secret: false },
  { key: "NEXT_PUBLIC_KERNEL_URL", tier: "P0", description: "Kernel origin exposed to the browser clients", reader: "web", required: false, secret: false },
  { key: "KIAN_STUDIO_DEFAULT_TIER", tier: "P0", description: "Default studio cognitive tier for the studio lane", reader: "kernel", required: false, secret: false },
  { key: "KIAN_STUDIO_MAX_ARTIFACT_BYTES", tier: "P0", description: "Studio artifact size cap on the wire", reader: "kernel", required: false, secret: false },
  { key: "KIAN_PORTFOLIO_CATALOG_PATH", tier: "P0", description: "Storefront portfolio catalog path", reader: "kernel", required: false, secret: false },
  { key: "KIAN_STUDIO_STOREFRONT_DEFAULT_XTR", tier: "P0", description: "Default XTR storefront price for studio products", reader: "kernel", required: false, secret: false },
  { key: "KIAN_QUANT_DEFAULT_BARS", tier: "P0", description: "Default bar count for the causal backtest lane", reader: "kernel", required: false, secret: false },
  { key: "KIAN_QUANT_DEFAULT_SEED", tier: "P0", description: "Deterministic default seed for the causal backtest", reader: "kernel", required: false, secret: false },
  { key: "KIAN_QUANT_MAX_LIST", tier: "P0", description: "Max persisted strategies listed per request", reader: "kernel", required: false, secret: false },
  { key: "KIAN_GENOME_ROOT_PATH", tier: "P0", description: "OMNI-MODEL-GENOME root directory for the genome loader", reader: "kernel", required: false, secret: false },
  { key: "KIAN_MONOPOLY_MAX_LIST", tier: "P0", description: "Max genome rows listed per request", reader: "kernel", required: false, secret: false },
];

/** Total keys in the environment schema — the hard contract is exactly 69. */
export const MAIRON_ENV_SPEC_TOTAL = MAIRON_ENV_SPEC.length;

/** Tier ordering P0 -> P5. */
export const MAIRON_ENV_TIERS: readonly MaironEnvTier[] = ["P0", "P1", "P2", "P3", "P4", "P5"];

/** All keys of one tier, in declaration order. */
export function envKeysOfTier(tier: MaironEnvTier): readonly MaironEnvKeySpec[] {
  return MAIRON_ENV_SPEC.filter((spec) => spec.tier === tier);
}

/** Look up one key's spec by exact name. */
export function envKeySpec(key: string): MaironEnvKeySpec | undefined {
  return MAIRON_ENV_SPEC.find((spec) => spec.key === key);
}

export type MaironEnvReadinessRow = MaironEnvKeySpec & {
  present: boolean;
};

export type MaironEnvReadiness = {
  totalKeys: number;
  presentKeys: number;
  requiredMissing: MaironEnvReadinessRow[];
  byTier: Record<MaironEnvTier, { total: number; present: number }>;
};

/**
 * Audit a concrete environment record against the schema. In the browser,
 * pass `process.env` (Next.js inlines the NEXT_PUBLIC_* subset); on the
 * server, pass the real environment.
 */
export function auditEnvReadiness(env: Record<string, string | undefined> = {}): MaironEnvReadiness {
  const rows: MaironEnvReadinessRow[] = MAIRON_ENV_SPEC.map((spec) => ({
    ...spec,
    present: typeof env[spec.key] === "string" && env[spec.key]!.trim().length > 0,
  }));
  const byTier = {} as Record<MaironEnvTier, { total: number; present: number }>;
  for (const tier of MAIRON_ENV_TIERS) {
    const tierRows = rows.filter((r) => r.tier === tier);
    byTier[tier] = { total: tierRows.length, present: tierRows.filter((r) => r.present).length };
  }
  return {
    totalKeys: rows.length,
    presentKeys: rows.filter((r) => r.present).length,
    requiredMissing: rows.filter((r) => r.required && !r.present),
    byTier,
  };
}

/** Read a single key with an optional default (browser-safe). */
export function readEnv(key: string, fallback = ""): string {
  if (typeof process === "undefined" || !process.env) return fallback;
  const value = process.env[key];
  return typeof value === "string" && value.trim().length > 0 ? value : fallback;
}
