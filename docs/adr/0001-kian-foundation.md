# ADR-0001 — The KIAN Foundation

**Status:** Accepted — 2026-11-20

## Context

Five legacy product lines (Python gen-1, TS gen-2 and gen-3, Rust gen-4, and the 10x flagship) with duplicated components.
Goal: a single, secure, extensible ecosystem with a frontend-first priority.

## Decision

1. **Name:** KIAN — "origin, source, kingdom"; brand continuity with KiaAgentX; aligned with the nation/parliament metaphor.
2. **Architecture:** monorepo — Rust kernel (13 crates) + TypeScript frontend (Next.js 16.4 / React 19.3 / Tailwind 4.3).
3. **First contract:** DTOs in Rust are the source of truth; OpenAPI 3.1 → automated SDK → Zod mirror.
4. **REA gate 10/10:** no artifact is exposed (source maps, credentials, debug companions, Rust metadata) — docs/rea-gates.md.
5. **Languages:** Rust + TypeScript + SQL + Shell. Legacy Python stays outside the monorepo until full parity is reached.

## Consequences

- Long-term elimination of the Python/TS/Rust triplication; the dual-run heritage in `crates/observe` guarantees a safe migration.
- Frontend first: M1 = apps/web + packages/ui.
