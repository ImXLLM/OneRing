import { type HTMLAttributes } from "react";
import { cn } from "../lib/utils";

interface ProgressProps extends HTMLAttributes<HTMLDivElement> {
  /** Value between 0 and 100 */
  value?: number;
  /** Fill color: royal (default) or gold (Din) */
  tone?: "royal" | "gold";
}

/** RTL-aware progress bar — fills from the right */
export function Progress({ className, value = 0, tone = "royal", ...props }: ProgressProps) {
  const clamped = Math.min(100, Math.max(0, value));
  return (
    <div
      role="progressbar"
      aria-valuemin={0}
      aria-valuemax={100}
      aria-valuenow={clamped}
      data-slot="progress"
      className={cn("relative h-2 w-full overflow-hidden rounded-full bg-secondary", className)}
      {...props}
    >
      <div
        data-slot="progress-indicator"
        className={cn(
          "absolute inset-y-0 right-0 rounded-full transition-[width] duration-500 ease-out",
          tone === "gold" ? "bg-gold-400" : "bg-gradient-to-l from-royal-500 to-royal-400",
        )}
        style={{ width: `${clamped}%` }}
      />
    </div>
  );
}
