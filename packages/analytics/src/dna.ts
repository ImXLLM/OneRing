/**
 * MAIRON analytics — the 10-DNA cohort registry.
 *
 * Mirror of the kernel's `UserLabTelemetryLog` DNA cohorts (see
 * `crates/hermes/src/grok_ux_and_lab_engine.rs` and the seeded
 * `UserLabTelemetryLog` rows). The wire ids are stable contracts —
 * never rename a DNA code.
 */

/** One DNA cohort descriptor. */
export interface DnaCohortSpec {
  /** Cohort code — `DNA_01`..`DNA_10`. */
  dnaCode: string;
  /** International-English label. */
  label: string;
  /** The seeded user handle for this cohort. */
  userHandle: string;
  /** The UX layer this cohort exercises. */
  uxTier: "toyspark" | "creator-studio" | "pro-terminal" | "quant-desk" | "war-room";
  /** Default locale for the cohort (telemetry join key). */
  locale: string;
  /** Default Grok persona wire id. */
  grokPersona: string;
}

/** The locked 10-DNA registry — order is a wire contract. */
export const MAIRON_DNA_COHORTS: readonly DnaCohortSpec[] = [
  {
    dnaCode: "DNA_01_CHILD_9YO_TOYSPARK",
    label: "9-year-old ToySpark explorer",
    userHandle: "child:panda-fan-9yo",
    uxTier: "toyspark",
    locale: "en",
    grokPersona: "fun_witty",
  },
  {
    dnaCode: "DNA_02_ASSISTED_IQ50_SIMPLE",
    label: "Assisted adult, simple flow",
    userHandle: "adult:assisted-iq50",
    uxTier: "toyspark",
    locale: "en",
    grokPersona: "regular_balanced",
  },
  {
    dnaCode: "DNA_03_MAIRON_ARCHITECT_18YO",
    label: "18-year-old Mairon architect",
    userHandle: "architect:18yo-mairon",
    uxTier: "pro-terminal",
    locale: "en",
    grokPersona: "mairon_sovereign",
  },
  {
    dnaCode: "DNA_04_PRO_QUANT_TRADER_28YO",
    label: "28-year-old professional quant trader",
    userHandle: "quant:pro-trader-28yo",
    uxTier: "quant-desk",
    locale: "en",
    grokPersona: "mairon_sovereign",
  },
  {
    dnaCode: "DNA_05_PERSIAN_CREATOR_STUDIO",
    label: "Persian-native Creator Studio maker",
    userHandle: "creator:tehran-studio",
    uxTier: "creator-studio",
    locale: "fa-IR",
    grokPersona: "regular_balanced",
  },
  {
    dnaCode: "DNA_06_VIRAL_AFFILIATE_HUNTER",
    label: "Viral affiliate hunter",
    userHandle: "affiliate:viral-hunter",
    uxTier: "creator-studio",
    locale: "en",
    grokPersona: "fun_witty",
  },
  {
    dnaCode: "DNA_07_SMART_CONTRACT_AUDITOR",
    label: "Smart-contract auditor",
    userHandle: "auditor:sc-review",
    uxTier: "pro-terminal",
    locale: "en",
    grokPersona: "mairon_sovereign",
  },
  {
    dnaCode: "DNA_08_WORKFLOW_AUTOMATION_BUILDER",
    label: "Workflow automation builder",
    userHandle: "builder:workflow-ops",
    uxTier: "pro-terminal",
    locale: "en",
    grokPersona: "unhinged_roast",
  },
  {
    dnaCode: "DNA_09_ADVERSARIAL_RED_TEAMER",
    label: "Adversarial red-teamer",
    userHandle: "redteam:adversarial",
    uxTier: "war-room",
    locale: "en",
    grokPersona: "unhinged_roast",
  },
  {
    dnaCode: "DNA_10_GLOBAL_EN_INDIE_FOUNDER",
    label: "Global English indie founder",
    userHandle: "founder:indie-global",
    uxTier: "war-room",
    locale: "en",
    grokPersona: "mairon_sovereign",
  },
] as const;

/** The count of DNA cohorts — exactly 10, a wire contract. */
export const DNA_COHORT_COUNT = MAIRON_DNA_COHORTS.length;

/** Find one cohort by its code. */
export function dnaCohortByCode(dnaCode: string): DnaCohortSpec | undefined {
  return MAIRON_DNA_COHORTS.find((c) => c.dnaCode === dnaCode);
}

/** Validate a DNA code against the registry. */
export function isValidDnaCode(dnaCode: string): boolean {
  return MAIRON_DNA_COHORTS.some((c) => c.dnaCode === dnaCode);
}
