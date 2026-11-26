/**
 * The admin API client — every call carries the X-KIAN-Key header from sessionStorage.
 *
 * Security rules:
 * * The key lives only in `sessionStorage` — it dies when the tab closes;
 *   never localStorage, never cookies, never in the URL.
 * * Every error maps to `{ code, message }` — kernel messages are honest.
 */

export const KERNEL_URL =
  process.env.NEXT_PUBLIC_KERNEL_URL ?? "http://127.0.0.1:8088";

const KEY_STORAGE = "kian.adminKey";

/** The key stored for this session — absent means logged out. */
export function getKey(): string | null {
  if (typeof window === "undefined") return null;
  return window.sessionStorage.getItem(KEY_STORAGE);
}

/** Store/remove the session key. */
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
      err?.message ?? `\u062f\u0631\u062e\u0648\u0627\u0633\u062a \u0634\u06a9\u0633\u062a \u062e\u0648\u0631\u062f (${res.status})`,
      res.status,
    );
  }
  return body as T;
}

/** Public GET — no key (health/economy). */
export async function publicGet<T>(path: string): Promise<T> {
  const res = await fetch(`${KERNEL_URL}/api/v1${path}`, { cache: "no-store" });
  return handle<T>(res);
}

/** Admin GET — with X-KIAN-Key. */
export async function adminGet<T>(path: string): Promise<T> {
  const key = getKey();
  if (!key) throw new AdminApiError("NO_KEY", "\u06a9\u0644\u06cc\u062f \u0627\u062f\u0645\u06cc\u0646 \u062b\u0628\u062a \u0646\u0634\u062f\u0647", 0);
  const res = await fetch(`${KERNEL_URL}/api/v1${path}`, {
    cache: "no-store",
    headers: { "X-KIAN-Key": key },
  });
  return handle<T>(res);
}

/** Admin POST with a JSON body. */
export async function adminPost<T>(path: string, body: unknown = {}): Promise<T> {
  const key = getKey();
  if (!key) throw new AdminApiError("NO_KEY", "\u06a9\u0644\u06cc\u062f \u0627\u062f\u0645\u06cc\u0646 \u062b\u0628\u062a \u0646\u0634\u062f\u0647", 0);
  const res = await fetch(`${KERNEL_URL}/api/v1${path}`, {
    method: "POST",
    headers: { "X-KIAN-Key": key, "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  return handle<T>(res);
}

/** Admin DELETE. */
export async function adminDelete(path: string): Promise<void> {
  const key = getKey();
  if (!key) throw new AdminApiError("NO_KEY", "\u06a9\u0644\u06cc\u062f \u0627\u062f\u0645\u06cc\u0646 \u062b\u0628\u062a \u0646\u0634\u062f\u0647", 0);
  const res = await fetch(`${KERNEL_URL}/api/v1${path}`, {
    method: "DELETE",
    headers: { "X-KIAN-Key": key },
  });
  return handle<void>(res);
}
