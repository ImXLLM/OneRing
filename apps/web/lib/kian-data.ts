/**
 * KIAN — منبع حقیقت داده‌های صفحه (قرینهٔ docs/migration-map.md و تحلیل وزن‌دهی)
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
    title: "پول",
    weight: 27,
    crates: "domain · wallet",
    stack: ["Rust", "sqlx", "TRX/SOL/TON/BSC"],
    description:
      "قلب اکوسیستم: ارز عدد صحیح Din (Đ) با سقف ۲⁵³−۱، دفتر دوطرفه، امانت ۸ حالته، کیف پول USDT چهارزنجیره‌ای با خزانه و حضانت کلید.",
  },
  {
    id: "protocol",
    icon: "network",
    title: "پروتکل",
    weight: 24,
    crates: "agents · contracts",
    stack: ["A2A", "MCP", "OpenAPI 3.1"],
    description:
      "زبان مشترک ایجنت‌ها: کشف و گفتگوی ایجنت‌به‌ایجنت (A2A)، ناوگان ۷ نقش، قراردادهای وایر با OpenAPI 3.1 و SDK خودکار برای بیلدرهای خارجی.",
  },
  {
    id: "brain",
    icon: "brain",
    title: "مغز",
    weight: 20,
    crates: "ai · hermes",
    stack: ["Rust", "fallback سه‌اسلات", "حافظه ۷لایه"],
    description:
      "گیت‌وی AI با سوئیچ خودکار مدل، پاکت سخت بودجه و redaction؛ موتور هرمس با ۱۶ مهارت تجاری، RL هویت و سپر دفاعی.",
  },
  {
    id: "growth",
    icon: "landmark",
    title: "حاکمیت و رشد",
    weight: 17,
    crates: "nation · superapp",
    stack: ["HITL", "پارلمان", "CRM"],
    description:
      "نادرترین قطعه بازار: پارلمان ایجنت‌ها با آستانه ۵۰,۰۰۰ Đ برای عمل‌های بزرگ؛ حلقه‌های رشد کوئست، معرفی، رتبه‌بندی و CRM.",
  },
  {
    id: "channel",
    icon: "channel",
    title: "کانال",
    weight: 12,
    crates: "bot · web · miniapp",
    stack: ["teloxide", "TON Connect", "PWA"],
    description:
      "دسترسی از هرجا: ربات تلگرام با orchestrator خالص، مینی‌اپ، همین PWA و ویجت قابل جاسازی برای سایت‌های دیگر.",
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
    title: "فرانتاند + PWA",
    scope: "apps/web · packages/ui",
    items: [
      "اسکلت مونوریپو با ۱۳ کرت Rust",
      "لندینگ سلطنتی RTL با Next.js 16",
      "PWA نصب‌پذیر با حالت آفلاین",
      "دیزاین سیستم کامل (packages/ui)",
    ],
    status: "done",
  },
  {
    id: "M2",
    title: "کرنل پول",
    scope: "domain → db → contracts → server",
    items: [
      "مهاجرت Din، دفتر دوطرفه و امانت ۸ حالته",
      "مهاجرت‌های نسخه‌دار sqlx",
      "دروازه axum با X-KIAN-Key",
      "گیت REA گام ۲: باینری بدون سیمبل",
    ],
    status: "done",
  },
  {
    id: "M3",
    title: "مینی‌اپ تلگرام",
    scope: "apps/miniapp · TON Connect",
    items: [
      "ولت، بازار و کوئست روی کرنل",
      "احراز initData سمت کرنل",
      "گیت REA گام ۳: تطبیق اندپوینت فرانت ↔ قرارداد",
    ],
    status: "done",
  },
  {
    id: "M4",
    title: "مغز + پنل عملیات",
    scope: "ai · hermes · agents · apps/admin",
    items: [
      "گیت‌وی AI فال‌بک سه‌اسلات + redaction + پاکت بودجه",
      "موتور هرمس: ۱۶ مهارت، سپر دفاعی، حافظهٔ ۷لایه",
      "ناوگان A2A/MCP با ۷ نقش صادقانه",
      "پنل ادمین جانشین ۷۱ اندپوینت (apps/admin)",
    ],
    status: "done",
  },
  {
    id: "M5",
    title: "حاکمیت + ولت + انتشار",
    scope: "nation · wallet · release",
    items: [
      "پارلمان HITL با آستانه ۵۰,۰۰۰ Đ",
      "ولت USDT چهارزنجیره‌ای + خزانه",
      "گیت REA گام ۵: audit کامل → 10/10 نهایی",
    ],
    status: "next",
  },
];

export const REA_LADDER = [
  {
    step: 1,
    title: "وب + PWA (M1)",
    detail: "بدون سورس‌مپ مرورگر · بدون کریدنشال · هدرهای امن · اسکلت آفلاین",
    status: "passed",
  },
  {
    step: 2,
    title: "کرنل Rust (M2)",
    detail: "strip=symbols · بدون debug companion · بدون متادیتای وابستگی",
    status: "passed",
  },
  {
    step: 3,
    title: "مینی‌اپ (M3)",
    detail: "تطبیق اندپوینت استاتیک ↔ مشاهده‌شده (compare-static-and-observed-endpoints)",
    status: "passed",
  },
  {
    step: 4,
    title: "ادمین و مغز (M4)",
    detail: "سطح دسترسی X-KIAN-Key · redaction · sandbox",
    status: "passed",
  },
  {
    step: 5,
    title: "انتشار نهایی (M5)",
    detail: "rea audit-release-exposure --fail_on low → صفر ردیف exposed → 10/10",
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
  { value: "۱۳", label: "کرت کرنل Rust" },
  { value: "۵", label: "زیرسیستم" },
  { value: "۸", label: "حالت امانت" },
  { value: "۱۶", label: "مهارت هرمس" },
];

export const FUTURE_TARGETS = ["PWA — امروز", "APK", "iOS", "Electron", "macOS"];
