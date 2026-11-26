/**
 * The MAIRON admin API client.
 *
 * Two credential planes, kept strictly apart:
 * 1. The admin control-plane key — `X-KIAN-Key`, sessionStorage-scoped
 *    (`kian.adminKey`); it dies with the tab, never touches localStorage,
 *    cookies, or URLs.
 * 2. The operator citizen session — Bearer token for kernel routes that are
 *    session-bound (war-room directive/reset, escrow actions). Obtained via
 *    `POST /auth/dev-login` which the kernel only serves with
 *    `KIAN_DEV_MODE=1` (403 otherwise — reported honestly in the UI).
 *
 * Every error maps to `{ code, message }` — kernel messages stay honest.
 */

export const KERNEL_URL =
  process.env.NEXT_PUBLIC_KERNEL_URL ?? "http://127.0.0.1:8088";

const KEY_STORAGE = "kian.adminKey";
const SESSION_STORAGE = "mairon.operatorSession";

/** The admin key stored for this session — absent means logged out. */
export function getKey(): string | null {
  if (typeof window === "undefined") return null;
  return window.sessionStorage.getItem(KEY_STORAGE);
}

/** Store/remove the admin key. */
export function setKey(key: string | null): void {
  if (typeof window === "undefined") return;
  if (key) window.sessionStorage.setItem(KEY_STORAGE, key);
  else window.sessionStorage.removeItem(KEY_STORAGE);
}

/** The operator Bearer token for session-bound kernel routes. */
export function getOperatorToken(): string | null {
  if (typeof window === "undefined") return null;
  return window.sessionStorage.getItem(SESSION_STORAGE);
}

/** Store/remove the operator session token. */
export function setOperatorToken(token: string | null): void {
  if (typeof window === "undefined") return;
  if (token) window.sessionStorage.setItem(SESSION_STORAGE, token);
  else window.sessionStorage.removeItem(SESSION_STORAGE);
}

export class AdminApiError extends Error {
  readonly code: string;
  readonly status: number;
  constructor(code: string, message: string, status: number) {
    super(message);
    this.name = "AdminApiError";
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
      err?.message ?? `Kernel request failed with HTTP ${res.status}`,
      res.status,
    );
  }
  return body as T;
}

/** Public GET — no key (health/economy/monopoly/war-room state). */
export async function publicGet<T>(path: string): Promise<T> {
  const res = await fetch(`${KERNEL_URL}/api/v1${path}`, { cache: "no-store" });
  return handle<T>(res);
}

/** Admin GET — with X-KIAN-Key. */
export async function adminGet<T>(path: string): Promise<T> {
  const key = getKey();
  if (!key) throw new AdminApiError("NO_KEY", "Admin key is not registered", 0);
  const res = await fetch(`${KERNEL_URL}/api/v1${path}`, {
    cache: "no-store",
    headers: { "X-KIAN-Key": key },
  });
  return handle<T>(res);
}

/** Admin POST with a JSON body. */
export async function adminPost<T>(path: string, body: unknown = {}): Promise<T> {
  const key = getKey();
  if (!key) throw new AdminApiError("NO_KEY", "Admin key is not registered", 0);
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
  if (!key) throw new AdminApiError("NO_KEY", "Admin key is not registered", 0);
  const res = await fetch(`${KERNEL_URL}/api/v1${path}`, {
    method: "DELETE",
    headers: { "X-KIAN-Key": key },
  });
  return handle<void>(res);
}

/** Operator session request — Bearer-token citizen routes (war-room, escrow). */
export async function operatorRequest<T>(
  path: string,
  init: { method?: string; body?: unknown } = {},
): Promise<T> {
  const token = getOperatorToken();
  if (!token) {
    throw new AdminApiError("NO_SESSION", "Operator session missing — log in as an operator first", 0);
  }
  const res = await fetch(`${KERNEL_URL}/api/v1${path}`, {
    method: init.method ?? "GET",
    cache: "no-store",
    headers: {
      authorization: `Bearer ${token}`,
      ...(init.body !== undefined ? { "Content-Type": "application/json" } : {}),
    },
    ...(init.body !== undefined ? { body: JSON.stringify(init.body) } : {}),
  });
  return handle<T>(res);
}

/** Operator login (dev mode only — 403 in production, surfaced honestly). */
export async function operatorLogin(tgUserId: string, username: string): Promise<{ token: string }> {
  const res = await fetch(`${KERNEL_URL}/api/v1/auth/dev-login`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ tgUserId, username }),
  });
  return handle<{ token: string }>(res);
}
