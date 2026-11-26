/**
 * MAIRON — source of truth for page data (mirror of docs/migration-map.md and the weighting analysis)
 */

export type Subsystem = {
  id: string;
  icon: "coins" | "brain" | "landmark" | "network" | "channel";
  title: string;
  weight: number;
  crates: string;
  stack: string[];
  description: string;
};

export const SUBSYSTEMS: Subsystem[] = [
  {
    id: "money",
    icon: "coins",
    title: "Money",
    weight: 27,
    crates: "domain · wallet",
    stack: ["Rust", "sqlx", "TRX/SOL/TON/BSC"],
    description:
      "The heart of the ecosystem: the exact-integer Din (Đ) currency with the 2^53−1 cap, the double-entry ledger, the 8-state escrow, and the four-chain USDT wallet with treasury and cold storage.",
  },
  {
    id: "protocol",
    icon: "network",
    title: "Protocol",
    weight: 24,
    crates: "agents · contracts",
    stack: ["A2A", "MCP", "OpenAPI 3.1"],
    description:
      "The agents' shared language: agent-to-agent discovery and conversation (A2A), the seven-role fleet, wire contracts with OpenAPI 3.1, and an automatic SDK for external builders.",
  },
  {
    id: "brain",
    icon: "brain",
    title: "Brain",
    weight: 20,
    crates: "ai · hermes",
    stack: ["Rust", "three-slot fallback", "seven-layer memory"],
    description:
      "The AI gateway with automatic model switching, a hard budget envelope and redaction; the Hermes engine with 16 commercial skills, an RL identity and a defensive shield.",
  },
  {
    id: "growth",
    icon: "landmark",
    title: "Governance & Growth",
    weight: 17,
    crates: "nation · viral",
    stack: ["HITL", "Parliament", "CRM"],
    description:
      "The market's rarest piece: the agents' parliament with the 50,000 Đ threshold for large operations; quest, referral and ranking growth loops.",
  },
  {
    id: "channel",
    icon: "channel",
    title: "Channels",
    weight: 12,
    crates: "bot · web · miniapp",
    stack: ["teloxide", "TON Connect", "PWA"],
    description:
      "Access from anywhere: the Telegram bot with the pure orchestrator, the mini-app, this PWA, and an embeddable widget for other sites.",
  },
];

export const LEGACY_REPOS = [
  "HermesA", "Xdrop", "dropagent", "3-TRINITY",
  "DropAgentX", "AgentX", "DropAgentX-v3", "Bot", "Agent-One",
  "R4", "DropAgentX-pro", "10x", "ShopifyHermes",
  "0xKiaBOT", "skills", "portfolio", "rtm", "0xKIA",
];

export type Milestone = {
  id: string;
  title: string;
  scope: string;
  items: string[];
  status: "done" | "active" | "next" | "later";
};

export const ROADMAP: Milestone[] = [
  {
    id: "M1",
    title: "Frontend + PWA",
    scope: "apps/web · packages/ui",
    items: [
      "The monorepo skeleton with 13 Rust crates",
      "The royal Next.js 16 landing",
      "An installable PWA with an offline mode",
      "The complete design system (packages/ui)",
    ],
    status: "done",
  },
  {
    id: "M2",
    title: "Money kernel",
    scope: "domain → db → contracts → server",
    items: [
      "The Din migration, the double-entry ledger and the 8-state escrow",
      "Versioned sqlx migrations",
      "The axum gateway with X-KIAN-Key",
      "REA gate step 2: a symbol-free binary",
    ],
    status: "done",
  },
  {
    id: "M3",
    title: "Telegram mini-app",
    scope: "apps/miniapp · TON Connect",
    items: [
      "Wallet, market and quests on the kernel",
      "Kernel-side initData verification",
      "REA gate step 3: static endpoints ↔ observed parity",
    ],
    status: "done",
  },
  {
    id: "M4",
    title: "Brain + operations console",
    scope: "ai · hermes · agents · apps/admin",
    items: [
      "The AI gateway with three-slot fallback + redaction + budget envelope",
      "The Hermes engine: 16 skills, defensive shield, seven-layer memory",
      "The honest A2A/MCP fleet with 7 roles",
      "The admin console replacing 71 endpoints (apps/admin)",
    ],
    status: "done",
  },
  {
    id: "M5",
    title: "Governance + wallet + release",
    scope: "nation · wallet · release",
    items: [
      "The HITL parliament with the 50,000 Đ threshold",
      "The four-chain USDT wallet + treasury",
      "REA gate step 5: full audit → final 10/10",
    ],
    status: "next",
  },
];

export const REA_LADDER = [
  {
    step: 1,
    title: "Web + PWA (M1)",
    detail: "no browser source maps · no credentials · secure headers · offline skeleton",
    status: "passed",
  },
  {
    step: 2,
    title: "Rust kernel (M2)",
    detail: "strip=symbols · no debug companion · no dependency metadata",
    status: "passed",
  },
  {
    step: 3,
    title: "Mini-app (M3)",
    detail: "static ↔ observed endpoint parity (compare-static-and-observed-endpoints)",
    status: "passed",
  },
  {
    step: 4,
    title: "Admin and brain (M4)",
    detail: "X-KIAN-Key access surface · redaction · sandbox",
    status: "passed",
  },
  {
    step: 5,
    title: "Final release (M5)",
    detail: "rea audit-release-exposure --fail_on low → zero exposed rows → 10/10",
    status: "pending",
  },
] as const;

export const REA_FILTERS = [
  { filter: "javascript.source_maps", result: "not_observed" },
  { filter: "text.credential_markers", result: "not_observed" },
  { filter: "native.debug_companions", result: "not_observed" },
  { filter: "rust.dependency_metadata", result: "not_observed" },
  { filter: "electron.run_as_node", result: "not_observed" },
  { filter: "packaging.extractable_payload", result: "monitored" },
];

export const HERO_STATS = [
  { value: "13", label: "Rust kernel crates" },
  { value: "5", label: "subsystems" },
  { value: "8", label: "escrow states" },
  { value: "16", label: "Hermes skills" },
];

export const FUTURE_TARGETS = ["PWA — today", "APK", "iOS", "Electron", "macOS"];
