"use client";

/**
 * KIAN M3 — Telegram mini-app layout (wallet + market).
 *
 * The session is opened once here and shared across the two pages;
 * the mini-app's compact header with the user's live balance stays at the top.
 */

import Link from "next/link";
import Image from "next/image";
import { LogOut, Store, Wallet } from "lucide-react";
import { usePathname } from "next/navigation";
import { SessionProvider, useSession } from "@/components/kian/session";
import { formatDin } from "@/lib/format";

const TABS = [
  { href: "/wallet", label: "Wallet", icon: Wallet },
  { href: "/market", label: "Market", icon: Store },
];

function MiniHeader() {
  const pathname = usePathname();
  const { state, signOut } = useSession();

  return (
    <header className="sticky top-0 z-40 border-b border-royal-500/20 bg-zinc-950/85 backdrop-blur-xl">
      <div className="mx-auto flex h-14 max-w-3xl items-center gap-3 px-4">
        <Link href="/" className="flex items-center gap-2" aria-label="MAIRON — home">
          <Image
            src="/logo-badge.png"
            alt=""
            width={32}
            height={32}
            priority
            className="size-8 rounded-lg ring-1 ring-gold-400/40"
          />
          <span className="font-black tracking-tight">KIAN</span>
        </Link>

        <nav className="mr-2 flex items-center gap-1" aria-label="mini-app">
          {TABS.map((t) => {
            const active = pathname.startsWith(t.href);
            return (
              <Link
                key={t.href}
                href={t.href}
                className={`flex items-center gap-1.5 rounded-lg px-3 py-1.5 text-sm transition-colors ${
                  active
                    ? "bg-royal-500/20 font-semibold text-gold-400 ring-1 ring-gold-400/30"
                    : "text-zinc-400 hover:bg-royal-500/10 hover:text-gold-400"
                }`}
              >
                <t.icon className="size-4" aria-hidden />
                {t.label}
              </Link>
            );
          })}
        </nav>

        <div className="mr-auto flex items-center gap-2">
          {state.phase === "ready" ? (
            <>
              <span className="rounded-lg bg-zinc-900 px-2.5 py-1 text-xs font-semibold text-gold-400 ring-1 ring-gold-400/30">
                {formatDin(state.me.balance)}
              </span>
              <span className="hidden text-xs text-zinc-400 sm:inline">
                `member ${state.me.user.telegramId}`
              </span>
              <button
                type="button"
                onClick={() => void signOut()}
                className="rounded-lg p-2 text-zinc-500 transition-colors hover:bg-red-500/10 hover:text-red-400"
                aria-label="Sign out of the session"
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
  );
}

export default function MiniAppLayout({ children }: { children: React.ReactNode }) {
  return (
    <SessionProvider>
      <div className="flex min-h-screen flex-col bg-zinc-950 text-zinc-100">
        <MiniHeader />
        <main className="mx-auto w-full max-w-3xl flex-1 px-4 py-6">{children}</main>
        <footer className="border-t border-zinc-800/60 py-4 text-center text-xs text-zinc-600">
          Rust kernel · double-entry ledger · 8-state escrow with the 10 percent Article 8 tax
        </footer>
      </div>
    </SessionProvider>
  );
}
