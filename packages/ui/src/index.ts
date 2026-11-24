// KIAN Design System — @kian/ui
// Tailwind 4 (CSS-first @theme) + shadcn الگو + motion 14 — RTL-first، پالت سلطنتی
//
// کامپوننت‌ها:
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

// هوک‌ها و ابزارها:
export { toast, dismiss, useToast, type ToastItem, type ToastVariant } from "./hooks/use-toast";
export { cn } from "./lib/utils";

export const DESIGN_TOKENS = {
  theme: "royal-dark",
  font: "Vazirmatn",
  accent: "oklch(0.55 0.18 280)", // royal-500
  gold: "oklch(0.85 0.12 85)", // Din gold
} as const;
