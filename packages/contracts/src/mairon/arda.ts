import { z } from "zod";

/**
 * MAIRON — Mairon Arda i18n engine wire contracts.
 *
 * TypeScript mirror of `crates/contracts/src/arda.rs`: the 20-locale planetary
 * manifest, the vector flag catalog, and the SKILL.md forge. English (`en`) is
 * the native default; every other language — including `fa-IR` — is a pluggable
 * `PATCH_READY_SLOT` applied at runtime from ASCII-escaped locale patches
 * (`locales/fa.patch.json`). Source code never carries raw non-ASCII literals.
 */

/** The Arda locale-slot policy — the native default plus pluggable patch slots. */
export const ARDA_LOCALE_POLICY = {
  nativeDefault: "en",
  patchReadySlot: "PATCH_READY_SLOT",
  totalSlots: 20,
  patchRegistryPath: "locales/fa.patch.json",
} as const;

/** The 20 canonical Arda locale codes in Fishkal lexicographic order. */
export const ARDA_LOCALE_CODES = [
  "ar", "de", "en", "es", "fa", "fr", "he", "hi", "id", "it",
  "ja", "ko", "nl", "pl", "pt", "ru", "sv", "th", "tr", "zh",
] as const;

export type ArdaLocaleCodeT = (typeof ARDA_LOCALE_CODES)[number];

/** One locale slot in the manifest — native default or a patch-ready slot. */
export const ArdaLocaleSlot = z.object({
  code: z.string().min(2).max(12),
  intlLocale: z.string().min(2).max(16),
  /** `native` | `PATCH_READY_SLOT` */
  slotKind: z.enum(["native", "PATCH_READY_SLOT"]),
  dir: z.enum(["ltr", "rtl"]),
  fontFamily: z.string().min(1),
  glyphSample: z.string().min(1),
  patchedAtIso: z.string().nullable(),
});
export type ArdaLocaleSlotT = z.infer<typeof ArdaLocaleSlot>;

/** The full 20-locale manifest with the native default pinned first. */
export const ArdaLocaleManifest = z.object({
  signature: z.literal("mairon-arda-20"),
  nativeDefault: z.literal("en"),
  slots: z.array(ArdaLocaleSlot).length(20),
});
export type ArdaLocaleManifestT = z.infer<typeof ArdaLocaleManifest>;

/**
 * Vector flag specification — the canonical Arda flag geometry
 * (`viewBox="0 0 20 14"`, circular-badge derivation included).
 */
export const SvgFlagSpec = z.object({
  /** ISO-ish locale code the flag belongs to. */
  code: z.string().min(2).max(12),
  viewBox: z.literal("0 0 20 14"),
  /** Verbatim SVG markup — geometry only; labels live in locale patches. */
  svg: z.string().min(1),
  /** Pre-wrapped circular badge markup rendered from `svg`. */
  circularBadgeHtml: z.string().min(1),
});
export type SvgFlagSpecT = z.infer<typeof SvgFlagSpec>;

/** SKILL.md export package — the forged, stamp-bearing agent skill file. */
export const SkillMdExport = z.object({
  productRef: z.string().min(1).max(160),
  fileName: z.literal("SKILL.md"),
  content: z.string().min(1),
  reaScore: z.number().int().min(0).max(100),
  badge: z.string().min(1).max(64),
  contentSha256: z.string().min(16).max(128),
  generatedAtIso: z.string(),
});
export type SkillMdExportT = z.infer<typeof SkillMdExport>;

// ── Direct mirrors of crates/contracts/src/arda.rs ─────────────────────────────

/** Mirror of `ArdaLocaleParityView`. */
export const ArdaLocaleParityView = z.object({
  locale: z.string(),
  missingKeys: z.array(z.string()),
  coverageScore: z.number(),
});
export type ArdaLocaleParityViewT = z.infer<typeof ArdaLocaleParityView>;

/** Mirror of `ArdaParityReportView`. */
export const ArdaParityReportView = z.object({
  baseLocale: z.string(),
  totalKeys: z.number().int().min(0),
  coverageScore: z.number(),
  fullyParity: z.boolean(),
  locales: z.array(ArdaLocaleParityView),
});
export type ArdaParityReportViewT = z.infer<typeof ArdaParityReportView>;

/** Mirror of `ArdaFlagView`. */
export const ArdaFlagView = z.object({
  code: z.string(),
  nativeName: z.string(),
  strCode: z.string(),
  dir: z.enum(["rtl", "ltr"]),
  intlLocale: z.string(),
  svg: z.string(),
  circularBadgeHtml: z.string(),
});
export type ArdaFlagViewT = z.infer<typeof ArdaFlagView>;

/** Mirror of `ArdaFlagsCatalogResponse` — 20 planetary languages. */
export const ArdaFlagsCatalogResponse = z.object({
  locales: z.array(ArdaFlagView),
  total: z.number().int(),
  signature: z.string(),
});
export type ArdaFlagsCatalogResponseT = z.infer<typeof ArdaFlagsCatalogResponse>;

/** Mirror of `ArdaLocalizeRequest`. */
export const ArdaLocalizeRequest = z.object({
  productId: z.string().optional(),
  html: z.string().optional(),
  originLocale: z.string().optional(),
  targetLocales: z.array(z.string()).optional(),
});
export type ArdaLocalizeRequestT = z.infer<typeof ArdaLocalizeRequest>;

/** Mirror of `ArdaLocalizeResponse`. */
export const ArdaLocalizeResponse = z.object({
  productId: z.string().optional(),
  originLocale: z.string(),
  supportedLocales: z.array(z.string()),
  localizedHtml: z.string(),
  circularFlagStripHtml: z.string(),
  parityReport: ArdaParityReportView,
  injected: z.boolean(),
  contentSha256: z.string(),
  lastLocalizedAt: z.string().optional(),
});
export type ArdaLocalizeResponseT = z.infer<typeof ArdaLocalizeResponse>;

/** Runtime locale patch — the ASCII-escaped payload a PATCH_READY_SLOT loads. */
export const ArdaLocalePatch = z.object({
  locale: z.string().min(2).max(12),
  dir: z.enum(["ltr", "rtl"]),
  entries: z.record(z.string(), z.string()),
});
export type ArdaLocalePatchT = z.infer<typeof ArdaLocalePatch>;
