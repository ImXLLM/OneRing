"use client";

/**
 * KIAN M3 — TON Connect island (client-only).
 *
 * This file is deliberately separate: `@tonconnect/ui-react` touches browser
 * APIs at module level, so it is loaded from the wallet page via
 * `next/dynamic({ ssr: false })` so SSR never touches it.
 */

import {
  TonConnectButton,
  TonConnectUIProvider,
  useTonAddress,
} from "@tonconnect/ui-react";

const SITE_URL = process.env.NEXT_PUBLIC_KIAN_URL ?? "http://localhost:3000";

export default function TonConnectSection({
  onRegisterAddress,
  registering,
}: {
  onRegisterAddress: (address: string) => void;
  registering: boolean;
}) {
  return (
    <TonConnectUIProvider manifestUrl={`${SITE_URL}/tonconnect-manifest.json`}>
      <div className="flex flex-col gap-3">
        <div className="flex items-center justify-between">
          <span className="text-sm text-zinc-400">&#x627;&#x62a;&#x635;&#x627;&#x644; &#x648;&#x644;&#x62a; TON</span>
          <TonConnectButton className="!rounded-xl" />
        </div>
        <RegisteredAddressBridge onRegister={onRegisterAddress} registering={registering} />
      </div>
    </TonConnectUIProvider>
  );
}

/** Bridge inside the provider — useTonAddress only works under TonConnectUIProvider. */
function RegisteredAddressBridge({
  onRegister,
  registering,
}: {
  onRegister: (address: string) => void;
  registering: boolean;
}) {
  const address = useTonAddress();
  if (!address) {
    return (
      <p className="text-xs leading-5 text-zinc-500">
        &#x628;&#x627; TON Connect &#x648;&#x644;&#x62a; &#x62e;&#x648;&#x62f; &#x631;&#x627; &#x648;&#x635;&#x644; &#x6a9;&#x646;&#x6cc;&#x62f;&#x61b; &#x67e;&#x633; &#x627;&#x632; &#x62a;&#x623;&#x6cc;&#x6cc;&#x62f; &#x627;&#x62f;&#x645;&#x6cc;&#x646;&#x60c; &#x62f;&#x6cc;&#x6cc;&#x646; &#x628;&#x647; &#x62d;&#x633;&#x627;&#x628;
        &#x634;&#x647;&#x631;&#x648;&#x646;&#x62f;&#x6cc;‌&#x62a;&#x627;&#x646; &#x636;&#x631;&#x628; &#x645;&#x6cc;‌&#x634;&#x648;&#x62f;.
      </p>
    );
  }
  return (
    <div className="flex flex-col gap-2 rounded-xl bg-zinc-900/70 p-3 ring-1 ring-royal-500/20">
      <span className="font-mono text-xs break-all text-zinc-300" dir="ltr">
        {address}
      </span>
      <button
        type="button"
        onClick={() => onRegister(address)}
        disabled={registering}
        className="self-start rounded-lg bg-royal-500/20 px-3 py-1.5 text-xs font-semibold text-gold-400 ring-1 ring-gold-400/30 transition-colors hover:bg-royal-500/30 disabled:opacity-50"
      >
        {registering ? "\u062f\u0631 \u062d\u0627\u0644 \u062b\u0628\u062a…" : "\u062b\u0628\u062a \u0627\u06cc\u0646 \u0622\u062f\u0631\u0633 \u0631\u0648\u06cc \u0647\u0648\u06cc\u062a \u0645\u0646"}
      </button>
    </div>
  );
}

export { TonConnectButton };
