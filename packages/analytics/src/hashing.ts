/**
 * MAIRON analytics — deterministic hashing primitives.
 *
 * The session fingerprint must be reproducible across devices and reloads
 * for the same (userId, sessionSeed) pair — FNV-1a 32-bit, the same family
 * the kernel's loop guard uses for payload hashing.
 */

/** FNV-1a 32-bit over a UTF-8 byte stream (deterministic, no Math.random). */
export function fnv1a32(input: string): string {
  let hash = 0x811c9dc5;
  for (const byte of new TextEncoder().encode(input)) {
    hash ^= byte;
    hash = Math.imul(hash, 0x01000193) >>> 0;
  }
  return hash.toString(16).padStart(8, "0");
}

/** FNV-1a 64-bit via the two-32-bit lane trick (deterministic). */
export function fnv1a64(input: string): string {
  const hi = fnv1a32(`${input}\u0000lane-hi`);
  const lo = fnv1a32(`${input}\u0000lane-lo`);
  return `${hi}${lo}`;
}

/**
 * Deterministic session fingerprint — `fp:{userPrefix}:{fnv64}`.
 *
 * Same user + same seed always yields the same fingerprint, so the telemetry
 * warehouse can join lab rows across reloads without cookies.
 */
export function sessionFingerprint(userId: string, sessionSeed: string): string {
  const userIdTrimmed = userId.trim().length > 0 ? userId.trim() : "anon";
  return `fp:${userIdTrimmed.slice(0, 8)}:${fnv1a64(`${userIdTrimmed}\u0000${sessionSeed}`)}`;
}

/** Deterministic event id for a causal telemetry event. */
export function causalEventId(fingerprint: string, tick: number, kind: string): string {
  return `evt:${fingerprint}:${tick.toString().padStart(4, "0")}:${fnv1a32(`${kind}:${tick}`)}`;
}
