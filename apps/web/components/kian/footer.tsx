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
            <p className="text-xs text-zinc-500">پادشاهی ایجنت‌ها — ساخته‌شده توسط KiaAgentX</p>
          </div>
        </div>

        <nav aria-label="پیوندهای پایین صفحه" className="flex flex-wrap items-center justify-center gap-x-6 gap-y-2 text-sm text-zinc-400">
          <a href="#subsystems" className="transition-colors hover:text-gold-400">زیرسیستم‌ها</a>
          <a href="#roadmap" className="transition-colors hover:text-gold-400">نقشه راه</a>
          <a href="#security" className="transition-colors hover:text-gold-400">امنیت</a>
          <a href="#install" className="transition-colors hover:text-gold-400">نصب</a>
        </nav>

        <div className="flex items-center gap-4 text-xs text-zinc-500">
          <span className="flex items-center gap-1.5">
            <ShieldCheck className="size-3.5 text-royal-400" /> REA-ready
          </span>
          <span className="flex items-center gap-1.5">
            <FolderGit2 className="size-3.5" /> مونوریپو kian/
          </span>
          <span dir="ltr">© 2026</span>
        </div>
      </div>
    </footer>
  );
}
