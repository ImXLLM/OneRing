"use client";

import { useState } from "react";
import { cn } from "../lib/utils";

/**
 * Base8453AddressPill — a Base Mainnet (chain id 8453) address pill.
 *
 * Renders the truncated address with the chain badge, and a one-tap copy
 * action backed by the async clipboard API (with an honest fallback when
 * the clipboard is unavailable — never a silent fake "copied").
 */
export interface Base8453AddressPillProps {
  /** The 0x-prefixed address. */
  address: string;
  /** Display prefix length before the ellipsis (default 6). */
  prefixChars?: number;
  /** Display suffix length after the ellipsis (default 4). */
  suffixChars?: number;
  /** Show the chain-8453 badge (default true). */
  showChainBadge?: boolean;
  className?: string;
}

/** The canonical Base Mainnet USDC contract — exported for desk widgets. */
export const BASE_CHAIN_ID = 8453 as const;
export const BASE_USDC_CONTRACT = "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913" as const;

function truncate(value: string, prefix: number, suffix: number): string {
  if (value.length <= prefix + suffix + 1) return value;
  return `${value.slice(0, prefix)}…${value.slice(-suffix)}`;
}

export function Base8453AddressPill({
  address,
  prefixChars = 6,
  suffixChars = 4,
  showChainBadge = true,
  className,
}: Base8453AddressPillProps) {
  const [copied, setCopied] = useState(false);

  async function copy(): Promise<void> {
    try {
      await navigator.clipboard.writeText(address);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1500);
    } catch {
      setCopied(false);
    }
  }

  return (
    <span
      data-slot="base-8453-address-pill"
      className={cn(
        "inline-flex items-center gap-1.5 rounded-full border border-sky-400/40 bg-sky-400/10 px-2.5 py-0.5 font-mono text-[11px] text-sky-200",
        className,
      )}
    >
      {showChainBadge ? (
        <span className="rounded-full bg-sky-400/20 px-1.5 py-0.5 text-[9px] font-bold uppercase tracking-wider text-sky-300">
          Base 8453
        </span>
      ) : null}
      <span title={address}>{truncate(address, prefixChars, suffixChars)}</span>
      <button
        type="button"
        onClick={() => void copy()}
        aria-label={`Copy address ${address} to clipboard`}
        className="rounded px-1 text-[10px] uppercase tracking-wider text-sky-300/80 transition hover:bg-sky-400/20 hover:text-sky-200"
      >
        {copied ? "copied" : "copy"}
      </button>
    </span>
  );
}
