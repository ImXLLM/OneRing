"use client";

/**
 * KIAN M3 — the mini-app session provider.
 *
 * Fail-closed flow:
 * 1. A valid stored token → straight to /me.
 * 2. Telegram initData (WebApp or browser fragment) → POST /auth/telegram → token.
 * 3. Neither → guest mode with an "Open inside Telegram" message.
 *
 * Identity comes only from the kernel-side HMAC; this component fabricates no identity claims.
 */

import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
} from "react";
import { api, clearToken, loadToken, saveToken, type MeResponse } from "@/lib/api";
import { extractInitData, setupTelegramWebApp } from "@/lib/telegram";

type SessionState =
  | { phase: "loading" }
  | { phase: "guest"; reason: "outside-telegram" | "auth-failed"; detail?: string }
  | { phase: "ready"; me: MeResponse; token: string };

type SessionContextValue = {
  state: SessionState;
  refresh: () => Promise<void>;
  signOut: () => Promise<void>;
};

const SessionContext = createContext<SessionContextValue | null>(null);

export function SessionProvider({ children }: { children: React.ReactNode }) {
  const [state, setState] = useState<SessionState>({ phase: "loading" });

  const authenticate = useCallback(async () => {
    // 1) Stored token — if still alive, that alone suffices
    const token = loadToken();
    if (token) {
      try {
        const me = await api.me(token);
        setState({ phase: "ready", me, token });
        return;
      } catch {
        clearToken(); // Dead session — re-authenticate
      }
    }
    // 2) Authenticate with Telegram initData
    const initData = extractInitData();
    if (!initData) {
      setState({ phase: "guest", reason: "outside-telegram" });
      return;
    }
    try {
      const auth = await api.authTelegram(initData);
      saveToken(auth.token);
      const me = await api.me(auth.token);
      setState({ phase: "ready", me, token: auth.token });
    } catch (err) {
      setState({
        phase: "guest",
        reason: "auth-failed",
        detail: err instanceof Error ? err.message : undefined,
      });
    }
  }, []);

  useEffect(() => {
    setupTelegramWebApp();
    void authenticate();
  }, [authenticate]);

  const refresh = useCallback(async () => {
    if (state.phase !== "ready") return;
    try {
      const me = await api.me(state.token);
      setState({ phase: "ready", me, token: state.token });
    } catch {
      await authenticate();
    }
  }, [state, authenticate]);

  const signOut = useCallback(async () => {
    if (state.phase === "ready") {
      try {
        await api.logout(state.token);
      } catch {
        /* The session may already be dead */
      }
    }
    clearToken();
    setState({ phase: "guest", reason: "outside-telegram" });
    await authenticate();
  }, [state, authenticate]);

  const value = useMemo(
    () => ({ state, refresh, signOut }),
    [state, refresh, signOut],
  );

  return <SessionContext.Provider value={value}>{children}</SessionContext.Provider>;
}

export function useSession(): SessionContextValue {
  const ctx = useContext(SessionContext);
  if (!ctx) throw new Error("useSession must be used inside <SessionProvider>");
  return ctx;
}
