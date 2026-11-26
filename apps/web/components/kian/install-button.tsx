"use client";

import { useEffect, useState } from "react";
import { Download, Crown } from "lucide-react";
import { Button } from "@kian/ui";

type BIPEvent = Event & { prompt: () => Promise<void>; userChoice: Promise<{ outcome: string }> };

/** PWA install button capturing the beforeinstallprompt event */
export function InstallButton({ className }: { className?: string }) {
  const [deferred, setDeferred] = useState<BIPEvent | null>(null);
  const [installed, setInstalled] = useState(false);

  useEffect(() => {
    const mq = window.matchMedia("(display-mode: standalone)");
    const onPrompt = (e: Event) => {
      e.preventDefault();
      setDeferred(e as BIPEvent);
    };
    const onInstalled = () => {
      setInstalled(true);
      setDeferred(null);
    };
    const syncMode = () => setInstalled(mq.matches);
    window.addEventListener("beforeinstallprompt", onPrompt);
    window.addEventListener("appinstalled", onInstalled);
    mq.addEventListener("change", syncMode);
    const raf = requestAnimationFrame(syncMode);
    return () => {
      window.removeEventListener("beforeinstallprompt", onPrompt);
      window.removeEventListener("appinstalled", onInstalled);
      mq.removeEventListener("change", syncMode);
      cancelAnimationFrame(raf);
    };
  }, []);

  const install = async () => {
    if (!deferred) return;
    await deferred.prompt();
    const choice = await deferred.userChoice;
    if (choice.outcome === "accepted") setInstalled(true);
    setDeferred(null);
  };

  if (installed) {
    return (
      <Button
        variant="outline"
        className={`border-gold-400/40 text-gold-400 hover:bg-gold-400/10 ${className ?? ""}`}
        disabled
      >
        <Crown className="size-4" /> &#x646;&#x635;&#x628; &#x634;&#x62f;&#x647;
      </Button>
    );
  }

  return (
    <Button
      onClick={install}
      disabled={!deferred}
      title={deferred ? "\u0646\u0635\u0628 KIAN \u0631\u0648\u06cc \u0627\u06cc\u0646 \u062f\u0633\u062a\u06af\u0627\u0647" : "\u0627\u0632 \u0645\u0646\u0648\u06cc \u0645\u0631\u0648\u0631\u06af\u0631 \u0646\u0635\u0628 \u06a9\u0646\u06cc\u062f"}
      className={`bg-gold-400 text-zinc-950 hover:bg-gold-400/90 font-bold ${className ?? ""}`}
    >
      <Download className="size-4" /> &#x646;&#x635;&#x628; KIAN
    </Button>
  );
}
