"use client";

/**
 * MAIRON mini-app — session provider (@kian/sdk layer).
 *
 * Fail-closed flow with a friction-free sign-in experience:
 * 1. A valid stored token → straight to /me.
 * 2. Telegram initData (WebApp or browser fragment) → POST /auth/telegram → token.
 * 3. Neither → guest gate with a clean "Quick experimental sign-in / Browser" banner —
 *    not a locked screen or a dry technical error.
 *
 * The experimental browser mode opens only public endpoints (market and national economy);
 * identity still comes only from the kernel HMAC and no identity claim is fabricated.
 */

import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
} from "react";
import {
  api,
  clearToken,
  DEMO_PERSONAS,
  extractInitData,
  loadToken,
  saveToken,
  setupTelegramWebApp,
  type DemoPersona,
  type MeResponse,
} from "@kian/sdk";
import {
  parseViralStartParam,
  saveActiveReferrer,
} from "@/lib/referral";

const PREVIEW_KEY = "kian.browser.preview";

/** Builds the active demo persona from the Telegram ID — null outside the personas. */
function personaFromTgId(tgId: string): DemoPersona | null {
  const entry = (Object.entries(DEMO_PERSONAS) as [DemoPersona, { tgUserId: string }][]).find(
    ([, seed]) => seed.tgUserId === tgId,
  );
  return entry?.[0] ?? null;
}

type SessionState =
  | { phase: "loading" }
  | { phase: "guest"; reason: "outside-telegram" | "auth-failed"; detail?: string }
  | { phase: "preview" }
  | { phase: "ready"; me: MeResponse; token: string };

type SessionContextValue = {
  state: SessionState;
  refresh: () => Promise<void>;
  signOut: () => Promise<void>;
  /** Enter the experimental browser mode (public endpoints only). */
  enterPreview: () => void;
  /** Leave browser mode and retry Telegram authentication. */
  exitPreview: () => void;
  /**
   * One-click sign-in with a demo persona — only when the kernel is in dev mode (otherwise
   * an honest 403). The browser referral link (`?startapp=p_..._r_...`) is also captured
   * and stored here so the next purchase is recorded with the referrer.
   */
  loginAsDemoPersona: (persona: DemoPersona, startParam?: string | null) => Promise<void>;
  /** Active demo persona — when the sessioned user is one of the three demo personas. */
  activePersona: DemoPersona | null;
};

const SessionContext = createContext<SessionContextValue | null>(null);

function readPreviewFlag(): boolean {
  if (typeof window === "undefined") return false;
  try {
    return window.sessionStorage.getItem(PREVIEW_KEY) === "1";
  } catch {
    return false;
  }
}

function writePreviewFlag(on: boolean): void {
  try {
    if (on) window.sessionStorage.setItem(PREVIEW_KEY, "1");
    else window.sessionStorage.removeItem(PREVIEW_KEY);
  } catch {
    /* Restricted browser mode — the preview is valid only for this opening */
  }
}

export function SessionProvider({ children }: { children: React.ReactNode }) {
  const [state, setState] = useState<SessionState>(() => {
    if (readPreviewFlag()) return { phase: "preview" };
    return { phase: "loading" };
  });

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
    // 2) If the user already chose browser mode, keep it (no jumping)
    if (readPreviewFlag()) {
      setState({ phase: "preview" });
      return;
    }
    // 3) Authenticate with Telegram initData
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
    setState({ phase: "loading" });
    await authenticate();
  }, [state, authenticate]);

  const enterPreview = useCallback(() => {
    writePreviewFlag(true);
    setState({ phase: "preview" });
  }, []);

  const exitPreview = useCallback(() => {
    writePreviewFlag(false);
    setState({ phase: "loading" });
    void authenticate();
  }, []);

  const loginAsDemoPersona = useCallback(
    async (persona: DemoPersona, startParam?: string | null) => {
      const seed = DEMO_PERSONAS[persona];
      // Grab the browser referral link: explicit parameter, otherwise this page's `?startapp=`
      const raw =
        startParam?.trim() ||
        (typeof window !== "undefined"
          ? new URLSearchParams(window.location.search).get("startapp")
          : null);
      const viral = parseViralStartParam(raw);
      if (viral.referrer) saveActiveReferrer(viral.referrer);
      writePreviewFlag(false);
      setState({ phase: "loading" });
      try {
        const auth = await api.devLogin({
          tgUserId: seed.tgUserId,
          username: seed.username,
          firstName: seed.firstName,
          startParam: raw ?? null,
        });
        saveToken(auth.token);
        const me = await api.me(auth.token);
        setState({ phase: "ready", me, token: auth.token });
      } catch (err) {
        setState({
          phase: "guest",
          reason: "auth-failed",
          detail:
            err instanceof Error
              ? err.message
              : "Experimental sign-in is not enabled on this server",
        });
        throw err;
      }
    },
    [],
  );

  const activePersona = useMemo<DemoPersona | null>(
    () =>
      state.phase === "ready"
        ? personaFromTgId(state.me.user.telegramId)
        : null,
    [state],
  );

  const value = useMemo(
    () => ({
      state,
      refresh,
      signOut,
      enterPreview,
      exitPreview,
      loginAsDemoPersona,
      activePersona,
    }),
    [state, refresh, signOut, enterPreview, exitPreview, loginAsDemoPersona, activePersona],
  );

  return <SessionContext.Provider value={value}>{children}</SessionContext.Provider>;
}

export function useSession(): SessionContextValue {
  const ctx = useContext(SessionContext);
  if (!ctx) throw new Error("useSession must be used inside <SessionProvider>");
  return ctx;
}
