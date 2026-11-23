"use client";

/**
 * قاب رندر امن HTML برای محصولات تک‌صفحه‌ای تعاملی KIAN.
 *
 * امنیت حیاتی:
 * - تنها `sandbox="allow-scripts"` فعال است.
 * - `allow-same-origin` و `allow-top-navigation` عمداً حذف شده‌اند تا کد JS داخل
 *   محصول HTML هرگز نتواند به `localStorage`، کوکی یا توکن نشست مینی‌اپ KIAN دسترسی یابد.
 */

import { useEffect, useState } from "react";
import { Download, Maximize2, RotateCw, ShieldCheck, X } from "lucide-react";

export type HtmlPreviewFrameProps = {
  /** سند کامل HTML برای رندر داخل `iframe` ایزوله */
  html: string;
  /** نسخهٔ دانلودی سند — پیش‌فرض همان `html`؛ برای پیش‌نمایش گاردخورده و
   *  دانلود سند کاملِ دست‌نخورده فرق می‌کنند. */
  downloadHtml?: string;
  /** عنوان نمایشی قاب و فایل دانلودی */
  title?: string;
  /** نمایش دکمهٔ دانلود فایل `.html` مستقل (مناسب حالت تحویل پس از خرید) */
  allowDownload?: boolean;
  /** نام فایل دانلودی (مثلاً `kian-app.html`) */
  downloadFilename?: string;
  /** کلاس ارتفاع قاب درون‌صفحه‌ای */
  heightClassName?: string;
  /** برچسب کوتاه کنار عنوان قاب */
  badgeLabel?: string;
};

function sanitizeFilename(name: string): string {
  const cleaned = name
    .trim()
    .replace(/[^a-zA-Z0-9\u0600-\u06FF._-]+/g, "-")
    .replace(/^-+|-+$/g, "");
  const base = cleaned || "kian-artifact";
  return base.toLowerCase().endsWith(".html") ? base : `${base}.html`;
}

export function HtmlPreviewFrame({
  html,
  downloadHtml,
  title = "پیش‌نمایش زنده HTML",
  allowDownload = false,
  downloadFilename,
  heightClassName = "h-[420px]",
  badgeLabel = "سندباکس ایزوله",
}: HtmlPreviewFrameProps) {
  const [reloadKey, setReloadKey] = useState(0);
  const [fullscreen, setFullscreen] = useState(false);

  useEffect(() => {
    if (!fullscreen) return;
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") setFullscreen(false);
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [fullscreen]);

  const handleDownload = () => {
    // نسخهٔ دانلودی سند کامل است — گارد پیش‌نمایش (`html`) به فایل محصول نشت نمی‌کند.
    const artifact = downloadHtml ?? html;
    if (!artifact) return;
    const blob = new Blob([artifact], { type: "text/html;charset=utf-8" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = sanitizeFilename(downloadFilename ?? title);
    document.body.appendChild(a);
    a.click();
    a.remove();
    window.setTimeout(() => URL.revokeObjectURL(url), 1000);
  };

  if (!html.trim()) {
    return (
      <div className="flex flex-col items-center justify-center gap-2 rounded-2xl border border-dashed border-zinc-700 bg-zinc-950/60 p-8 text-center text-xs text-zinc-500">
        <span>هنوز پیش‌نمایش HTML برای این محصول ثبت نشده است.</span>
      </div>
    );
  }

  return (
    <>
      <div className="overflow-hidden rounded-2xl border border-royal-500/30 bg-zinc-950 shadow-xl shadow-black/40">
        {/* نوار ابزار قاب */}
        <div className="flex flex-wrap items-center justify-between gap-2 border-b border-zinc-800 bg-zinc-900/90 px-3 py-2">
          <div className="flex min-w-0 items-center gap-2">
            <ShieldCheck className="size-4 shrink-0 text-emerald-400" aria-hidden />
            <span className="truncate text-xs font-bold text-zinc-200">{title}</span>
            <span className="hidden rounded-full bg-emerald-500/10 px-2 py-0.5 text-[10px] font-semibold text-emerald-300 ring-1 ring-emerald-500/25 sm:inline">
              {badgeLabel}
            </span>
          </div>

          <div className="flex items-center gap-1.5">
            <button
              type="button"
              onClick={() => setReloadKey((k) => k + 1)}
              title="تازه‌سازی پیش‌نمایش"
              className="flex items-center gap-1 rounded-lg bg-zinc-800 px-2 py-1 text-[11px] font-semibold text-zinc-200 hover:bg-zinc-700"
            >
              <RotateCw className="size-3.5" aria-hidden />
              <span>تازه‌سازی</span>
            </button>
            <button
              type="button"
              onClick={() => setFullscreen(true)}
              title="بزرگ‌نمایی تمام‌صفحه"
              className="flex items-center gap-1 rounded-lg bg-royal-500/20 px-2 py-1 text-[11px] font-semibold text-gold-400 ring-1 ring-gold-400/30 hover:bg-royal-500/30"
            >
              <Maximize2 className="size-3.5" aria-hidden />
              <span>تمام‌صفحه</span>
            </button>
            {allowDownload && (
              <button
                type="button"
                onClick={handleDownload}
                title="دانلود فایل HTML مستقل"
                className="flex items-center gap-1 rounded-lg bg-emerald-500/20 px-2.5 py-1 text-[11px] font-bold text-emerald-300 ring-1 ring-emerald-400/35 hover:bg-emerald-500/30"
              >
                <Download className="size-3.5" aria-hidden />
                <span>دانلود .html</span>
              </button>
            )}
          </div>
        </div>

        {/* قاب ایزوله — بدون allow-same-origin و بدون allow-top-navigation */}
        <iframe
          key={`inline-${reloadKey}`}
          title={title}
          srcDoc={html}
          sandbox="allow-scripts"
          referrerPolicy="no-referrer"
          className={`w-full bg-zinc-950 ${heightClassName}`}
        />
      </div>

      {/* مودال تمام‌صفحه */}
      {fullscreen && (
        <div
          className="fixed inset-0 z-50 flex flex-col bg-zinc-950/95 backdrop-blur-md"
          role="dialog"
          aria-modal="true"
          aria-label={title}
        >
          <div className="flex items-center justify-between gap-2 border-b border-royal-500/30 bg-zinc-900 px-4 py-3">
            <div className="flex min-w-0 items-center gap-2">
              <ShieldCheck className="size-4 shrink-0 text-emerald-400" aria-hidden />
              <span className="truncate text-sm font-bold text-white">{title}</span>
            </div>
            <div className="flex items-center gap-2">
              <button
                type="button"
                onClick={() => setReloadKey((k) => k + 1)}
                className="flex items-center gap-1 rounded-lg bg-zinc-800 px-2.5 py-1.5 text-xs font-semibold text-zinc-200 hover:bg-zinc-700"
              >
                <RotateCw className="size-3.5" aria-hidden />
                تازه‌سازی
              </button>
              {allowDownload && (
                <button
                  type="button"
                  onClick={handleDownload}
                  className="flex items-center gap-1 rounded-lg bg-emerald-500/20 px-2.5 py-1.5 text-xs font-bold text-emerald-300 ring-1 ring-emerald-400/35 hover:bg-emerald-500/30"
                >
                  <Download className="size-3.5" aria-hidden />
                  دانلود .html
                </button>
              )}
              <button
                type="button"
                onClick={() => setFullscreen(false)}
                aria-label="بستن تمام‌صفحه"
                className="grid size-8 place-items-center rounded-lg bg-zinc-800 text-zinc-300 hover:bg-zinc-700 hover:text-white"
              >
                <X className="size-4" aria-hidden />
              </button>
            </div>
          </div>
          <iframe
            key={`full-${reloadKey}`}
            title={`${title} — تمام‌صفحه`}
            srcDoc={html}
            sandbox="allow-scripts"
            referrerPolicy="no-referrer"
            className="w-full flex-1 bg-zinc-950"
          />
        </div>
      )}
    </>
  );
}
