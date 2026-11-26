import { z } from "zod";

/**
 * MAIRON — Core Ledger & Escrow wire contracts.
 *
 * The double-entry heart of the platform: `Account`, `JournalEntry`, `Posting`
 * (with the `SUM(Posting.amount) == 0` invariant), the eight-state escrow
 * machine, and the full 19-column `Product` row. Mirrors of the kernel's
 * `Account` / `Tx` / `Posting` / `Escrow` / `Product` tables and the
 * `crates/contracts` escrow views. Money is always an integer (`Din` micros);
 * the trial balance must sum to exactly zero — the auditor treats any other
 * value as a hard failure.
 */

/** Din — the integer money unit; cap 2^53-1 (the safe-integer boundary is deliberate).
 * Re-exported here for module-local use; the package root re-exports the canonical one. */
const DinAmount = z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER);

/** Signed Din — postings carry negative debits and positive credits. */
export const SignedDinAmount = z.number().int().min(Number.MIN_SAFE_INTEGER).max(Number.MAX_SAFE_INTEGER);

// ── Account ────────────────────────────────────────────────────────────────────

/** The canonical account kinds of the double-entry ledger. */
export const ACCOUNT_KINDS = [
  "user", "treasury", "escrow", "creator_payout", "affiliate",
  "marketplace_tax", "settlement_spread", "reserve", "burn", "system",
] as const;
export type AccountKindT = (typeof ACCOUNT_KINDS)[number];

/** Mirror of the kernel's `Account` table + derived balance (never stored). */
export const Account = z.object({
  id: z.string().min(1).max(120),
  kind: z.enum([
    "user", "treasury", "escrow", "creator_payout", "affiliate",
    "marketplace_tax", "settlement_spread", "reserve", "burn", "system",
  ]),
  label: z.string().max(120).nullable(),
  /** Derived from `SUM(Posting.amount)` — ADR-003: balances are never stored. */
  balance: z.number().int(),
  createdAt: z.string(),
});
export type AccountT = z.infer<typeof Account>;

// ── JournalEntry / Posting ─────────────────────────────────────────────────────

/** Mirror of the kernel's `Tx` table — the journal-entry header. */
export const JournalEntry = z.object({
  id: z.string().min(1).max(120),
  /** Unique idempotency key — a retry race can never move money twice. */
  idempotencyKey: z.string().max(160).nullable(),
  memo: z.string().max(200).nullable(),
  at: z.string(),
});
export type JournalEntryT = z.infer<typeof JournalEntry>;

/** Mirror of the kernel's `Posting` table — one signed leg of a journal entry. */
export const Posting = z.object({
  id: z.number().int().min(0),
  txId: z.string().min(1).max(120),
  accountId: z.string().min(1).max(120),
  amount: z.number().int(),
});
export type PostingT = z.infer<typeof Posting>;

/** A balanced journal entry — the posting set with its zero-sum proof attached. */
export const BalancedJournalEntry = z.object({
  entry: JournalEntry,
  postings: z.array(Posting).min(2),
  /** Always exactly `0` — the auditor's hard invariant. */
  trialBalanceSum: z.literal(0),
});
export type BalancedJournalEntryT = z.infer<typeof BalancedJournalEntry>;

// ── EscrowState ────────────────────────────────────────────────────────────────

/** The eight escrow state-machine states (SCREAMING_SNAKE_CASE, kernel truth). */
export const EscrowState = z.enum([
  "CREATED",
  "FUNDED",
  "DELIVERED",
  "RELEASED",
  "DISPUTED",
  "REFUNDED",
  "CANCELLED",
  "EXPIRED",
]);
export type EscrowStateT = z.infer<typeof EscrowState>;

/** Allowed transitions — mirror of ADR-004 (UI guidance; truth lives in the kernel). */
export const ESCROW_STATE_TRANSITIONS: Record<EscrowStateT, readonly EscrowStateT[]> = {
  CREATED: ["FUNDED", "CANCELLED"],
  FUNDED: ["DELIVERED", "DISPUTED", "EXPIRED"],
  DELIVERED: ["RELEASED", "DISPUTED"],
  RELEASED: [],
  DISPUTED: ["REFUNDED", "RELEASED"],
  REFUNDED: [],
  CANCELLED: [],
  EXPIRED: [],
};

// ── Product — the full 19-column row ───────────────────────────────────────────

/** Column census of the kernel's `Product` table — 9 base + 10 migrated = 19. */
export const PRODUCT_COLUMN_COUNT = 19 as const;

/** The 19 canonical `Product` columns, in physical order. */
export const PRODUCT_COLUMNS = [
  "id", "sellerId", "title", "description", "priceDin", "category",
  "deliveryPayload", "active", "createdAt",
  "previewHtml", "affiliateBps", "artifactKind",
  "maironScore", "maironBadge",
  "originLocale", "supportedLocalesJson", "skillMd", "skillJson",
  "ardaLastLocalizedAt",
] as const satisfies readonly string[];

/** Mirror of the full 19-column `Product` row (market + trinity + arda fields). */
export const Product = z.object({
  id: z.string(),
  sellerId: z.string(),
  title: z.string(),
  description: z.string(),
  priceDin: DinAmount,
  category: z.string().nullable(),
  deliveryPayload: z.string().nullable(),
  active: z.boolean(),
  createdAt: z.string(),
  previewHtml: z.string(),
  affiliateBps: z.number().int().min(0).max(5_000),
  artifactKind: z.string(),
  maironScore: z.number().int().min(0).max(100),
  maironBadge: z.string(),
  originLocale: z.string(),
  supportedLocalesJson: z.string(),
  skillMd: z.string().nullable(),
  skillJson: z.string().nullable(),
  ardaLastLocalizedAt: z.string().nullable(),
});
export type ProductT = z.infer<typeof Product>;

/** Ledger-audit summary — what `GET /api/v1/economy` exposes about the invariant. */
export const LedgerAuditSummary = z.object({
  trialBalanceSum: z.number().int(),
  /** `true` only when `trialBalanceSum === 0`. */
  balanced: z.boolean(),
  accountCount: z.number().int().min(0),
  openEscrows: z.number().int().min(0),
  escrowLocked: DinAmount,
});
export type LedgerAuditSummaryT = z.infer<typeof LedgerAuditSummary>;
