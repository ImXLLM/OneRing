// MAIRON Sovereign Design System — @kian/ui
// Tailwind 4 (CSS-first @theme) + shadcn pattern + motion 14 — LTR-first, royal palette.
//
// The package keeps its historical `@kian/ui` name so every existing import
// keeps working; the design language itself is 100% MAIRON.
//
// Core primitives:
export { Button, buttonVariants, type ButtonProps } from "./components/button";
export {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
  CardFooter,
} from "./components/card";
export { Badge, badgeVariants, type BadgeProps } from "./components/badge";
export { Input } from "./components/input";
export { Textarea } from "./components/textarea";
export { Label } from "./components/label";
export { Separator } from "./components/separator";
export { Skeleton } from "./components/skeleton";
export { Progress } from "./components/progress";
export { Alert, AlertTitle, AlertDescription, alertVariants, type AlertProps } from "./components/alert";
export { Toaster } from "./components/toaster";
export { Slot } from "./components/slot";

// MAIRON sovereign kit:
export { MaironSealBadge, type MaironSealBadgeProps } from "./components/mairon-seal-badge";
export {
  TrinityIqMeter,
  TRINITY_IQ_TIERS,
  REA_GATE_MIN_SCORE,
  type TrinityIqMeterProps,
} from "./components/trinity-iq-meter";
export {
  Base8453AddressPill,
  BASE_CHAIN_ID,
  BASE_USDC_CONTRACT,
  type Base8453AddressPillProps,
} from "./components/base-8453-address-pill";
export {
  PerspectiveSwitcherBar,
  type PerspectiveSwitcherBarProps,
} from "./components/perspective-switcher-bar";

// Hooks and utilities:
export { toast, dismiss, useToast, type ToastItem, type ToastVariant } from "./hooks/use-toast";
export { cn } from "./lib/utils";

export const DESIGN_TOKENS = {
  theme: "mairon-royal-dark",
  font: "Inter",
  monoFont: "Geist Mono",
  accent: "oklch(0.55 0.18 280)", // royal-500
  gold: "oklch(0.85 0.12 85)", // Mairon gold
  baseChainId: 8453,
} as const;
