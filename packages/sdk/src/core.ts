/**
 * MAIRON SDK core — the shared request engine and error envelope.
 *
 * Every session-bound request sends `Authorization: Bearer`; every failure is
 * the kernel's honest `{ error: { code, message, meta? } }` envelope mapped to
 * `ApiError`. Identity is never constructed here: only the raw Telegram
 * initData string goes to the kernel where the official HMAC is verified
 * (fail-closed).
 */

export class ApiError extends Error {
  readonly status: number;
  readonly code: string;
  constructor(status: number, code: string, message: string) {
    super(message);
    this.name = "ApiError";
    this.status = status;
    this.code = code;
  }
}

/** Resolve the default kernel API base at runtime (browser or server). */
export function resolveDefaultApiBase(): string {
  if (typeof process !== "undefined" && process.env) {
    const env = process.env as Record<string, string | undefined>;
    if (env.MAIRON_API_BASE_URL) return env.MAIRON_API_BASE_URL;
    if (env.NEXT_PUBLIC_KIAN_API) return env.NEXT_PUBLIC_KIAN_API;
    if (env.NEXT_PUBLIC_KERNEL_URL) return `${env.NEXT_PUBLIC_KERNEL_URL.replace(/\/$/, "")}/api/v1`;
  }
  if (typeof window !== "undefined" && typeof window.location !== "undefined") {
    return `${window.location.origin}/api/v1`;
  }
  return "http://127.0.0.1:8088/api/v1";
}

/** One typed kernel request with bearer injection and honest error mapping. */
export async function typedRequest<T>(
  base: string,
  path: string,
  init: RequestInit & { token?: string | null; adminKey?: string | null } = {},
): Promise<T> {
  const { token, adminKey, ...rest } = init;
  const headers = new Headers(rest.headers);
  if (rest.body) headers.set("content-type", "application/json");
  if (token) headers.set("authorization", `Bearer ${token}`);
  if (adminKey) headers.set("x-kian-key", adminKey);
  const res = await fetch(`${base}${path}`, { ...rest, headers });
  if (res.status === 204) return undefined as T;
  const body: unknown = await res.json().catch(() => ({}));
  if (!res.ok) {
    const envelope = (body as { error?: { code?: string; message?: string } }).error;
    throw new ApiError(
      res.status,
      envelope?.code ?? "UNKNOWN",
      envelope?.message ?? `Kernel request failed with HTTP ${res.status}`,
    );
  }
  return body as T;
}

/** Small helper for building query strings from optional parameters. */
export function qs(params: Record<string, string | number | boolean | undefined>): string {
  const parts: string[] = [];
  for (const [key, value] of Object.entries(params)) {
    if (value === undefined) continue;
    parts.push(`${encodeURIComponent(key)}=${encodeURIComponent(String(value))}`);
  }
  return parts.length === 0 ? "" : `?${parts.join("&")}`;
}
