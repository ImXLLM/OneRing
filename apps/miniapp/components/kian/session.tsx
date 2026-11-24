"use client";

/**
 * KIAN miniapp — فراهم‌کنندهٔ نشست (لایهٔ @kian/sdk).
 *
 * جریان fail-closed با تجربهٔ ورود بدون اصطکاک:
 * 1. توکن ذخیره‌شدهٔ معتبر → مستقیم /me.
 * 2. initData تلگرام (WebApp یا fragment مرورگر) → POST /auth/telegram → توکن.
 * 3. هیچ‌کدام → دروازهٔ مهمان با بنر تمیز «🧪 ورود سریع آزمایشی / مرورگر» —
 *    نه صفحهٔ قفل‌شده و نه خطای خشک فنی.
 *
 * حالت مرورگر آزمایشی فقط اندپوینت‌های عمومی (بازار و اقتصاد ملی) را باز
 * می‌کند؛ هویت همچنان فقط از HMAC کرنل می‌آید و هیچ ادعای هویتی ساخته نمی‌شود.
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

/** پرسونای دموی فعال را از شناسهٔ تلگرامی می‌سازد — خارج از پرسوناها null. */
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
  /** ورود به حالت مرورگر آزمایشی (فقط اندپوینت‌های عمومی). */
  enterPreview: () => void;
  /** خروج از حالت مرورگر و تلاش دوباره برای احراز تلگرام. */
  exitPreview: () => void;
  /**
   * ورود یک‌کلیکی با پرسونای دمو — فقط وقتی کرنل در حالت dev است (وگرنه
   * ۴۰۳ صادقانه). لینک رفرال مرورگر (`?startapp=p_..._r_...`) هم همین‌جا
   * گرفته و ذخیره می‌شود تا خریدِ بعدی با معرف ثبت شود.
   */
  loginAsDemoPersona: (persona: DemoPersona, startParam?: string | null) => Promise<void>;
  /** پرسونای دموی فعال — وقتی کاربرِ نشست‌دار یکی از سه پرسونای دمو باشد. */
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
    /* حالت محدود مرورگر — پیش‌نمایش فقط در همین باز شدن معتبر است */
  }
}

export function SessionProvider({ children }: { children: React.ReactNode }) {
  const [state, setState] = useState<SessionState>(() => {
    if (readPreviewFlag()) return { phase: "preview" };
    return { phase: "loading" };
  });

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
    // ۲) اگر کاربر قبلاً حالت مرورگر را انتخاب کرده، همان را نگه دار (بدون پرش)
    if (readPreviewFlag()) {
      setState({ phase: "preview" });
      return;
    }
    // ۳) احراز با initData تلگرام
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
      // گرفتن لینک رفرال مرورگر: پارامتر صریح، وگرنه `?startapp=` همین صفحه
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
              : "ورود آزمایشی در این سرور فعال نیست",
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
