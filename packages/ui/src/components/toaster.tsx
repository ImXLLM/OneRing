"use client";

import { AnimatePresence, motion } from "motion/react";
import { CircleCheck, CircleX, Coins, Info, X } from "lucide-react";
import { cn } from "../lib/utils";
import { dismiss, useToast, type ToastVariant } from "../hooks/use-toast";

const ICONS: Record<ToastVariant, React.ReactNode> = {
  default: <Info className="size-4 text-royal-400" aria-hidden />,
  success: <CircleCheck className="size-4 text-emerald-400" aria-hidden />,
  destructive: <CircleX className="size-4 text-destructive" aria-hidden />,
  gold: <Coins className="size-4 text-gold-400" aria-hidden />,
};

const BORDER: Record<ToastVariant, string> = {
  default: "border-royal-500/40",
  success: "border-emerald-500/40",
  destructive: "border-destructive/50",
  gold: "border-gold-400/50",
};

/** Container for global toasts — mounted in RootLayout; RTL, bottom of the page */
export function Toaster() {
  const { toasts } = useToast();

  return (
    <div
      role="region"
      aria-label="&#x627;&#x639;&#x644;&#x627;&#x646;‌&#x647;&#x627;"
      className="pointer-events-none fixed inset-x-0 bottom-4 z-[100] flex flex-col items-center gap-2 px-4"
    >
      <AnimatePresence initial={false}>
        {toasts.map((t) => (
          <motion.div
            key={t.id}
            layout
            initial={{ opacity: 0, y: 24, scale: 0.96 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: 12, scale: 0.96 }}
            transition={{ type: "spring", stiffness: 380, damping: 30 }}
            className={cn(
              "pointer-events-auto flex w-full max-w-md items-start gap-3 rounded-xl border bg-card/95 p-4 shadow-2xl backdrop-blur",
              BORDER[t.variant ?? "default"],
            )}
            role="status"
          >
            {ICONS[t.variant ?? "default"]}
            <div className="flex-1 space-y-0.5">
              <p className="text-sm font-semibold">{t.title}</p>
              {t.description ? (
                <p className="text-xs leading-relaxed text-muted-foreground">{t.description}</p>
              ) : null}
            </div>
            <button
              type="button"
              onClick={() => dismiss(t.id)}
              aria-label="Dismiss the announcement"
              className="mt-0.5 text-muted-foreground transition-colors hover:text-foreground"
            >
              <X className="size-4" aria-hidden />
            </button>
          </motion.div>
        ))}
      </AnimatePresence>
    </div>
  );
}
