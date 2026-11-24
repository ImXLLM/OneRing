"use client";

/**
 * KIAN M3 — فراهم‌کنندهٔ نشست مینی‌اپ.
 *
 * جریان fail-closed:
 * 1. توکن ذخیره‌شدهٔ معتبر → مستقیم /me.
 * 2. initData تلگرام (WebApp یا fragment مرورگر) → POST /auth/telegram → توکن.
 * 3. هیچ‌کدام → حالت مهمان با پیام «باز کردن داخل تلگرام».
 *
 * هویت فقط از HMAC سمت کرنل می‌آید؛ این کامپوننت هیچ ادعای هویتی نمی‌سازد.
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
    // ۱) توکن ذخیره‌شده — اگر زنده بود همین کافی است
    const token = loadToken();
    if (token) {
      try {
        const me = await api.me(token);
        setState({ phase: "ready", me, token });
        return;
      } catch {
        clearToken(); // نشست مرده — احراز دوباره
      }
    }
    // ۲) احراز با initData تلگرام
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
        /* نشست شاید قبلاً مرده باشد */
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
