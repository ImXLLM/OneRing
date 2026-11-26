"use client";

import { useEffect, useState } from "react";
import Image from "next/image";
import { InstallButton } from "./install-button";

const LINKS = [
  { href: "/wallet", label: "Wallet" },
  { href: "/market", label: "Market" },
  { href: "#subsystems", label: "Subsystems" },
  { href: "#architecture", label: "Architecture" },
  { href: "#roadmap", label: "Roadmap" },
  { href: "#security", label: "REA Security" },
];

export function SiteHeader() {
  const [scrolled, setScrolled] = useState(false);

  useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 24);
    onScroll();
    window.addEventListener("scroll", onScroll, { passive: true });
    return () => window.removeEventListener("scroll", onScroll);
  }, []);

  return (
    <header
      className={`sticky top-0 z-50 w-full transition-all duration-300 ${
        scrolled
          ? "border-b border-royal-500/20 bg-zinc-950/80 backdrop-blur-xl"
          : "border-b border-transparent bg-transparent"
      }`}
    >
      <div className="mx-auto flex h-16 max-w-6xl items-center justify-between gap-4 px-4 sm:px-6">
        <a href="#top" className="flex items-center gap-2.5" aria-label="MAIRON — back to top">
          <Image
            src="/logo-badge.png"
            alt=""
            width={36}
            height={36}
            priority
            className="size-9 rounded-xl shadow-lg shadow-royal-600/30 ring-1 ring-gold-400/40"
          />
          <span className="text-xl font-black tracking-tight">
            KIAN
            <span className="mr-2 text-xs font-medium text-zinc-500">Kingdom of Agents</span>
          </span>
        </a>

        <nav aria-label="primary navigation" className="hidden items-center gap-1 md:flex">
          {LINKS.map((l) => (
            <a
              key={l.href}
              href={l.href}
              className="rounded-lg px-3 py-2 text-sm text-zinc-400 transition-colors hover:bg-royal-500/10 hover:text-gold-400"
            >
              {l.label}
            </a>
          ))}
        </nav>

        <InstallButton className="h-9 px-4 text-sm" />
      </div>
    </header>
  );
}
