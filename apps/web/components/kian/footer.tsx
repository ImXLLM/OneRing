import { Crown, ShieldCheck, FolderGit2 } from "lucide-react";

export function SiteFooter() {
  return (
    <footer className="mt-auto border-t border-royal-500/20 bg-zinc-950/80">
      <div className="mx-auto flex max-w-6xl flex-col items-center justify-between gap-6 px-4 py-10 sm:px-6 md:flex-row">
        <div className="flex items-center gap-3">
          <span className="flex size-9 items-center justify-center rounded-xl bg-gradient-to-b from-royal-400 to-royal-600 ring-1 ring-gold-400/40">
            <Crown className="size-4.5 text-gold-400" aria-hidden />
          </span>
          <div>
            <p className="font-black">KIAN</p>
            <p className="text-xs text-zinc-500">&#x67e;&#x627;&#x62f;&#x634;&#x627;&#x647;&#x6cc; &#x627;&#x6cc;&#x62c;&#x646;&#x62a;‌&#x647;&#x627; — &#x633;&#x627;&#x62e;&#x62a;&#x647;‌&#x634;&#x62f;&#x647; &#x62a;&#x648;&#x633;&#x637; KiaAgentX</p>
          </div>
        </div>

        <nav aria-label="&#x67e;&#x6cc;&#x648;&#x646;&#x62f;&#x647;&#x627;&#x6cc; &#x67e;&#x627;&#x6cc;&#x6cc;&#x646; &#x635;&#x641;&#x62d;&#x647;" className="flex flex-wrap items-center justify-center gap-x-6 gap-y-2 text-sm text-zinc-400">
          <a href="#subsystems" className="transition-colors hover:text-gold-400">&#x632;&#x6cc;&#x631;&#x633;&#x6cc;&#x633;&#x62a;&#x645;‌&#x647;&#x627;</a>
          <a href="#roadmap" className="transition-colors hover:text-gold-400">&#x646;&#x642;&#x634;&#x647; &#x631;&#x627;&#x647;</a>
          <a href="#security" className="transition-colors hover:text-gold-400">&#x627;&#x645;&#x646;&#x6cc;&#x62a;</a>
          <a href="#install" className="transition-colors hover:text-gold-400">&#x646;&#x635;&#x628;</a>
        </nav>

        <div className="flex items-center gap-4 text-xs text-zinc-500">
          <span className="flex items-center gap-1.5">
            <ShieldCheck className="size-3.5 text-royal-400" /> REA-ready
          </span>
          <span className="flex items-center gap-1.5">
            <FolderGit2 className="size-3.5" /> &#x645;&#x648;&#x646;&#x648;&#x631;&#x6cc;&#x67e;&#x648; kian/
          </span>
          <span dir="ltr">© 2026</span>
        </div>
      </div>
    </footer>
  );
}
