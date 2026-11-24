/**
 * کلاینت API ادمین — همهٔ تماس‌ها با هدر X-KIAN-Key از sessionStorage.
 *
 * قواعد امنیتی:
 * * کلید فقط در `sessionStorage` زندگی می‌کند — با بستن تب می‌میرد؛
 *   هرگز localStorage، هرگز کوکی، هرگز در URL.
 * * هر خطا به `{ code, message }` نگاشت می‌شود — پیام‌های کرنل صادقانه‌اند.
 */

export const KERNEL_URL =
  process.env.NEXT_PUBLIC_KERNEL_URL ?? "http://127.0.0.1:8088";

const KEY_STORAGE = "kian.adminKey";

/** کلید ذخیره‌شدهٔ این نشست — غایب یعنی لاگ‌اوت. */
export function getKey(): string | null {
  if (typeof window === "undefined") return null;
  return window.sessionStorage.getItem(KEY_STORAGE);
}

/** ذخیره/حذف کلید نشست. */
export function setKey(key: string | null): void {
  if (typeof window === "undefined") return;
  if (key) window.sessionStorage.setItem(KEY_STORAGE, key);
  else window.sessionStorage.removeItem(KEY_STORAGE);
}

export class AdminApiError extends Error {
  readonly code: string;
  readonly status: number;
  constructor(code: string, message: string, status: number) {
    super(message);
    this.code = code;
    this.status = status;
  }
}

async function handle<T>(res: Response): Promise<T> {
  if (res.status === 204) return undefined as T;
  const body: unknown = await res.json().catch(() => ({}));
  if (!res.ok) {
    const err = (body as { error?: { code?: string; message?: string } }).error;
    throw new AdminApiError(
      err?.code ?? `HTTP_${res.status}`,
      err?.message ?? `درخواست شکست خورد (${res.status})`,
      res.status,
    );
  }
  return body as T;
}

/** GET عمومی — بدون کلید (شادهالت/اقتصاد). */
export async function publicGet<T>(path: string): Promise<T> {
  const res = await fetch(`${KERNEL_URL}/api/v1${path}`, { cache: "no-store" });
  return handle<T>(res);
}

/** GET ادمین — با X-KIAN-Key. */
export async function adminGet<T>(path: string): Promise<T> {
  const key = getKey();
  if (!key) throw new AdminApiError("NO_KEY", "کلید ادمین ثبت نشده", 0);
  const res = await fetch(`${KERNEL_URL}/api/v1${path}`, {
    cache: "no-store",
    headers: { "X-KIAN-Key": key },
  });
  return handle<T>(res);
}

/** POST ادمین با بدنهٔ JSON. */
export async function adminPost<T>(path: string, body: unknown = {}): Promise<T> {
  const key = getKey();
  if (!key) throw new AdminApiError("NO_KEY", "کلید ادمین ثبت نشده", 0);
  const res = await fetch(`${KERNEL_URL}/api/v1${path}`, {
    method: "POST",
    headers: { "X-KIAN-Key": key, "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  return handle<T>(res);
}

/** DELETE ادمین. */
export async function adminDelete(path: string): Promise<void> {
  const key = getKey();
  if (!key) throw new AdminApiError("NO_KEY", "کلید ادمین ثبت نشده", 0);
  const res = await fetch(`${KERNEL_URL}/api/v1${path}`, {
    method: "DELETE",
    headers: { "X-KIAN-Key": key },
  });
  return handle<void>(res);
}
