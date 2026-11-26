"use client";

/**
 * MAIRON mini-app — app shell: sticky header + the sovereign brand mark.
 *
 * The four perspectives are switched in-page by the `PerspectiveSwitcherBar`
 * (lossless switching inside one world state), so the shell only carries the
 * header and the content area — no destructive route-based tab bar.
 */

import Link from "next/link";
import { LogOut } from "lucide-react";
import { useSession } from "@/components/kian/session";
import { DemoPersonaBar } from "@/components/kian/guest-gate";
import { formatDin } from "@/lib/format";

export function MiniShell({ children }: { children: React.ReactNode }) {
  const { state, signOut } = useSession();
  const ready = state.phase === "ready";

  return (
    <div className="flex min-h-screen flex-col">
      <header className="sticky top-0 z-40 border-b border-royal-500/20 bg-zinc-950/85 backdrop-blur-xl">
        <div
          className="mx-auto flex h-14 w-full max-w-3xl items-center gap-3 px-4"
          style={{ paddingTop: "env(safe-area-inset-top)" }}
        >
          <Link href="/" className="flex items-center gap-2" aria-label="MAIRON home">
            <span className="grid size-8 place-items-center rounded-xl bg-gradient-to-b from-royal-400 to-royal-600 font-black text-white shadow-lg shadow-royal-600/30 ring-1 ring-gold-400/40">
              M
            </span>
            <span className="text-sm font-black tracking-tight">MAIRON</span>
          </Link>
          <div className="ml-auto flex items-center gap-2">
            {ready ? (
              <>
                <span className="rounded-full bg-gold-400/10 px-3 py-1 text-xs font-bold text-gold-400 ring-1 ring-gold-400/30">
                  {formatDin(state.me.balance)}
                </span>
                <button
                  type="button"
                  onClick={() => void signOut()}
                  aria-label="Sign out of the session"
                  className="grid size-8 place-items-center rounded-lg text-zinc-500 ring-1 ring-zinc-800 hover:text-zinc-300"
                >
                  <LogOut className="size-4" aria-hidden />
                </button>
              </>
            ) : (
              <span className="text-xs text-zinc-500">no session</span>
            )}
          </div>
        </div>
      </header>

      {/* Demo persona switcher bar — only for test sessions (1001/2002/3003) */}
      <div className="mx-auto w-full max-w-3xl px-4 pt-3">
        <DemoPersonaBar />
      </div>

      <main
        className="mx-auto w-full max-w-3xl flex-1 px-4 py-5"
        style={{
          paddingBottom: "calc(env(safe-area-inset-bottom, 0px) + 1.5rem)",
        }}
      >
        {children}
      </main>
    </div>
  );
}
