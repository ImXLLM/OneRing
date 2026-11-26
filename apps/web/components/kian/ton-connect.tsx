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
          <span className="text-sm text-zinc-400">Connect the TON wallet</span>
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
        Connect your wallet with TON Connect; after admin approval the Din mints to
        your citizen account.
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
        {registering ? "registering…" : "Register this address on my identity"}
      </button>
    </div>
  );
}

export { TonConnectButton };
