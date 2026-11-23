"use client";

/**
 * KIAN miniapp — پوستهٔ اپ: هدر چسبان + تب‌بار پایینی (موبایل‌اول).
 */

import { useEffect } from "react";
import Link from "next/link";
import { usePathname, useRouter } from "next/navigation";
import { Home, LogOut, Sparkles, Store, Wallet } from "lucide-react";
import { useSession } from "@/components/kian/session";
import { formatDin } from "@/lib/format";
import {
  captureInboundViralLink,
  consumePendingProductRedirect,
} from "@/lib/referral";

const TABS = [
  { href: "/", label: "خانه", icon: Home },
  { href: "/builder", label: "استودیو AI", icon: Sparkles },
  { href: "/market", label: "بازار", icon: Store },
  { href: "/wallet", label: "ولت", icon: Wallet },
];

export function MiniShell({ children }: { children: React.ReactNode }) {
  const pathname = usePathname();
  const router = useRouter();
  const { state, signOut } = useSession();
  const ready = state.phase === "ready";

  useEffect(() => {
    const { productId } = captureInboundViralLink();
    const target = consumePendingProductRedirect(productId);
    if (target && pathname !== "/market") {
      router.replace(`/market?product=${encodeURIComponent(target)}`);
    }
  }, [pathname, router]);

  return (
    <div className="flex min-h-screen flex-col">
      <header className="sticky top-0 z-40 border-b border-royal-500/20 bg-zinc-950/85 backdrop-blur-xl">
        <div
          className="mx-auto flex h-14 w-full max-w-3xl items-center gap-3 px-4"
          style={{ paddingTop: "env(safe-area-inset-top)" }}
        >
          <Link href="/" className="flex items-center gap-2" aria-label="KIAN — خانه">
            <span className="grid size-8 place-items-center rounded-xl bg-gradient-to-b from-royal-400 to-royal-600 font-black text-white shadow-lg shadow-royal-600/30 ring-1 ring-gold-400/40">
              K
            </span>
            <span className="text-sm font-black tracking-tight">KIAN</span>
          </Link>
          <div className="mr-auto flex items-center gap-2">
            {ready ? (
              <>
                <span className="rounded-full bg-gold-400/10 px-3 py-1 text-xs font-bold text-gold-400 ring-1 ring-gold-400/30">
                  {formatDin(state.me.balance)}
                </span>
                <button
                  type="button"
                  onClick={() => void signOut()}
                  aria-label="خروج از نشست"
                  className="grid size-8 place-items-center rounded-lg text-zinc-500 ring-1 ring-zinc-800 hover:text-zinc-300"
                >
                  <LogOut className="size-4" aria-hidden />
                </button>
              </>
            ) : (
              <span className="text-xs text-zinc-500">نشست برقرار نشده</span>
            )}
          </div>
        </div>
      </header>

      {/* فاصلهٔ پایین به‌اندازهٔ تب‌بار + safe-area — هیچ دکمه‌ای زیر نوار نمی‌رود */}
      <main
        className="mx-auto w-full max-w-3xl flex-1 px-4 py-5"
        style={{
          paddingBottom:
            "calc(env(safe-area-inset-bottom, 0px) + 5.25rem)",
        }}
      >
        {children}
      </main>

      <nav
        className="fixed inset-x-0 bottom-0 z-40 border-t border-royal-500/20 bg-zinc-950/90 backdrop-blur-xl"
        style={{ paddingBottom: "env(safe-area-inset-bottom, 0px)" }}
      >
        <div className="mx-auto grid w-full max-w-3xl grid-cols-4 px-2 py-1">
          {TABS.map((t) => {
            const active = pathname === t.href;
            return (
              <Link
                key={t.href}
                href={t.href}
                className={`flex flex-col items-center gap-1 rounded-xl py-2 text-[11px] font-semibold transition-colors ${
                  active ? "text-gold-400" : "text-zinc-500 hover:text-zinc-300"
                }`}
              >
                <t.icon className="size-5" aria-hidden />
                {t.label}
              </Link>
            );
          })}
        </div>
      </nav>
    </div>
  );
}
