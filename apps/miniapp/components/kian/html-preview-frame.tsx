"use client";

/**
 * Secure HTML rendering frame for KIAN interactive single-page products.
 *
 * Critical security:
 * - Only `sandbox="allow-scripts"` is enabled.
 * - `allow-same-origin` and `allow-top-navigation` are deliberately omitted so the JS code inside
 *   an HTML product can never reach `localStorage`, cookies, or the KIAN mini-app session token.
 */

import { useEffect, useState } from "react";
import { Download, Maximize2, RotateCw, ShieldCheck, X } from "lucide-react";

export type HtmlPreviewFrameProps = {
  /** The complete HTML document to render inside the isolated `iframe` */
  html: string;
  /** Downloadable version of the document — defaults to `html`; the guard-fed preview and
   *  the untouched full-document download differ. */
  downloadHtml?: string;
  /** Display title for the frame and the downloaded file */
  title?: string;
  /** Show the download button for the standalone `.html` file (fits the post-purchase delivery mode) */
  allowDownload?: boolean;
  /** Downloaded file name (e.g. `kian-app.html`) */
  downloadFilename?: string;
  /** Height class for the in-page frame */
  heightClassName?: string;
  /** Short label beside the frame title */
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
  title = "\u067e\u06cc\u0634‌\u0646\u0645\u0627\u06cc\u0634 \u0632\u0646\u062f\u0647 HTML",
  allowDownload = false,
  downloadFilename,
  heightClassName = "h-[420px]",
  badgeLabel = "\u0633\u0646\u062f\u0628\u0627\u06a9\u0633 \u0627\u06cc\u0632\u0648\u0644\u0647",
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
    // The download version is the full document — the preview guard (`html`) does not leak into the product file.
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
        <span>&#x647;&#x646;&#x648;&#x632; &#x67e;&#x6cc;&#x634;‌&#x646;&#x645;&#x627;&#x6cc;&#x634; HTML &#x628;&#x631;&#x627;&#x6cc; &#x627;&#x6cc;&#x646; &#x645;&#x62d;&#x635;&#x648;&#x644; &#x62b;&#x628;&#x62a; &#x646;&#x634;&#x62f;&#x647; &#x627;&#x633;&#x62a;.</span>
      </div>
    );
  }

  return (
    <>
      <div className="overflow-hidden rounded-2xl border border-royal-500/30 bg-zinc-950 shadow-xl shadow-black/40">
        {/* Frame toolbar */}
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
              title="\u062a\u0627\u0632\u0647‌\u0633\u0627\u0632\u06cc \u067e\u06cc\u0634‌\u0646\u0645\u0627\u06cc\u0634"
              className="flex items-center gap-1 rounded-lg bg-zinc-800 px-2 py-1 text-[11px] font-semibold text-zinc-200 hover:bg-zinc-700"
            >
              <RotateCw className="size-3.5" aria-hidden />
              <span>&#x62a;&#x627;&#x632;&#x647;‌&#x633;&#x627;&#x632;&#x6cc;</span>
            </button>
            <button
              type="button"
              onClick={() => setFullscreen(true)}
              title="\u0628\u0632\u0631\u06af‌\u0646\u0645\u0627\u06cc\u06cc \u062a\u0645\u0627\u0645‌\u0635\u0641\u062d\u0647"
              className="flex items-center gap-1 rounded-lg bg-royal-500/20 px-2 py-1 text-[11px] font-semibold text-gold-400 ring-1 ring-gold-400/30 hover:bg-royal-500/30"
            >
              <Maximize2 className="size-3.5" aria-hidden />
              <span>&#x62a;&#x645;&#x627;&#x645;‌&#x635;&#x641;&#x62d;&#x647;</span>
            </button>
            {allowDownload && (
              <button
                type="button"
                onClick={handleDownload}
                title="&#x62f;&#x627;&#x646;&#x644;&#x648;&#x62f; &#x641;&#x627;&#x6cc;&#x644; HTML &#x645;&#x633;&#x62a;&#x642;&#x644;"
                className="flex items-center gap-1 rounded-lg bg-emerald-500/20 px-2.5 py-1 text-[11px] font-bold text-emerald-300 ring-1 ring-emerald-400/35 hover:bg-emerald-500/30"
              >
                <Download className="size-3.5" aria-hidden />
                <span>&#x62f;&#x627;&#x646;&#x644;&#x648;&#x62f; .html</span>
              </button>
            )}
          </div>
        </div>

        {/* Isolated frame — no allow-same-origin and no allow-top-navigation */}
        <iframe
          key={`inline-${reloadKey}`}
          title={title}
          srcDoc={html}
          sandbox="allow-scripts"
          referrerPolicy="no-referrer"
          className={`w-full bg-zinc-950 ${heightClassName}`}
        />
      </div>

      {/* Fullscreen modal */}
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
                &#x62a;&#x627;&#x632;&#x647;‌&#x633;&#x627;&#x632;&#x6cc;
              </button>
              {allowDownload && (
                <button
                  type="button"
                  onClick={handleDownload}
                  className="flex items-center gap-1 rounded-lg bg-emerald-500/20 px-2.5 py-1.5 text-xs font-bold text-emerald-300 ring-1 ring-emerald-400/35 hover:bg-emerald-500/30"
                >
                  <Download className="size-3.5" aria-hidden />
                  &#x62f;&#x627;&#x646;&#x644;&#x648;&#x62f; .html
                </button>
              )}
              <button
                type="button"
                onClick={() => setFullscreen(false)}
                aria-label="\u0628\u0633\u062a\u0646 \u062a\u0645\u0627\u0645‌\u0635\u0641\u062d\u0647"
                className="grid size-8 place-items-center rounded-lg bg-zinc-800 text-zinc-300 hover:bg-zinc-700 hover:text-white"
              >
                <X className="size-4" aria-hidden />
              </button>
            </div>
          </div>
          <iframe
            key={`full-${reloadKey}`}
            title={`${title} — \u062a\u0645\u0627\u0645‌\u0635\u0641\u062d\u0647`}
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
