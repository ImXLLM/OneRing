"use client";

import { useEffect, useState } from "react";
import Image from "next/image";
import { InstallButton } from "./install-button";

const LINKS = [
  { href: "/wallet", label: "\u0648\u0644\u062a" },
  { href: "/market", label: "\u0628\u0627\u0632\u0627\u0631" },
  { href: "#subsystems", label: "\u0632\u06cc\u0631\u0633\u06cc\u0633\u062a\u0645‌\u0647\u0627" },
  { href: "#architecture", label: "\u0645\u0639\u0645\u0627\u0631\u06cc" },
  { href: "#roadmap", label: "\u0646\u0642\u0634\u0647 \u0631\u0627\u0647" },
  { href: "#security", label: "\u0627\u0645\u0646\u06cc\u062a REA" },
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
        <a href="#top" className="flex items-center gap-2.5" aria-label="KIAN — &#x628;&#x627;&#x632;&#x6af;&#x634;&#x62a; &#x628;&#x647; &#x628;&#x627;&#x644;&#x627;">
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
            <span className="mr-2 text-xs font-medium text-zinc-500">&#x67e;&#x627;&#x62f;&#x634;&#x627;&#x647;&#x6cc; &#x627;&#x6cc;&#x62c;&#x646;&#x62a;‌&#x647;&#x627;</span>
          </span>
        </a>

        <nav aria-label="&#x646;&#x627;&#x648;&#x628;&#x631;&#x6cc; &#x627;&#x635;&#x644;&#x6cc;" className="hidden items-center gap-1 md:flex">
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
