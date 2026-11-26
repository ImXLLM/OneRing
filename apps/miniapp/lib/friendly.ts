/**
 * Translates kernel error codes into short, actionable English (presentation-layer UX).
 *
 * Codes come from the Rust kernel's standard `{ error: { code, message } }` envelope;
 * they are rewritten here for display only — no security logic lives here.
 */

import { ApiError } from "@kian/sdk";

const FRIENDLY_CODES: Record<string, string> = {
  ACCOUNT_OVERDRAFT:
    "Not enough wallet balance — top up first, or earn with your referral link.",
  INSUFFICIENT_BALANCE:
    "Not enough wallet balance — top up first, or earn with your referral link.",
  UNKNOWN_ACCOUNT: "This account was not found — the session may be stale; try signing in again.",
  UNKNOWN_ESCROW: "This transaction was not found — it may have been removed or the session is stale.",
  INVALID_STATE: "This transaction cannot run from its current state — refresh the page.",
  REQUIRES_HUMAN_APPROVAL:
    "This amount is large and needs admin approval — your request is now in the review queue.",
  SELF_PURCHASE: "You cannot buy your own product.",
  RATE_LIMITED: "Easy there — try again in a few moments.",
  SESSION_REQUIRED: "Your session has expired — please sign in again.",
  SESSION_EXPIRED: "Your session has expired — please sign in again.",
  INVALID_INIT_DATA: "Telegram identity verification failed — close and reopen the mini-app.",
  IDENTITY_DISABLED: "Telegram sign-in is currently off — try again shortly.",
  MINT_CAP_EXCEEDED:
    "The annual mint cap is full — this limit is published and public, it is not an account problem.",
  VALIDATION: "The form data is incomplete or invalid — please double-check and retry.",
  BODY_TOO_LARGE: "The submitted content is too large (the HTML cap is 150 kilobytes).",
  NETWORK: "Could not reach the kernel — check your internet or VPN and try again.",
  UNKNOWN: "An unexpected error occurred — try again; if it repeats, contact support.",
};

/** Friendly message for any API or network error — always user-comprehensible English. */
export function friendlyError(err: unknown): string {
  if (err instanceof ApiError) {
    return FRIENDLY_CODES[err.code] ?? `${err.message || "kernel error"} — please try again.`;
  }
  if (err instanceof Error && err.message) {
    if (/fetch|network|Failed to fetch/i.test(err.message)) {
      return FRIENDLY_CODES.NETWORK;
    }
    return err.message;
  }
  return FRIENDLY_CODES.UNKNOWN;
}

/** Short title for the error toast, based on the error type. */
export function friendlyErrorTitle(err: unknown): string {
  if (err instanceof ApiError && (err.code === "ACCOUNT_OVERDRAFT" || err.code === "INSUFFICIENT_BALANCE")) {
    return "Insufficient balance";
  }
  if (err instanceof ApiError && (err.code === "SESSION_REQUIRED" || err.code === "SESSION_EXPIRED")) {
    return "Session expired";
  }
  return "Action failed";
}
