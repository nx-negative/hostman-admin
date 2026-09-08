# HOSTMAN-ADMIN — version ledger
updated: 2026-09-08 · phase: 0/10 · status: awaiting-approval

## Completed
- P0 scaffold: cargo workspace `backend/crates/{admin-api,domain,common,sync-worker}` — axum 0.8 `/healthz`+`/readyz` (Neon ping, degraded-start), Appendix-A error envelope (`common/error.rs`), JSON tracing, config/dotenv, SeaORM 2.0 + migrator (`domain`), initial `settings` migration, `admin-api setup` (migrations), router tests. Frontend: Vite 8 (Rolldown) + React 19 + TS strict, shadcn preset `b1D0dv72` (base-mira — tokens verified against registry item), TanStack Query/Router/Form/Table/Virtual + zod 4 + Framer Motion, app shell (sidebar+topbar, light/dark, flat), 9 stub routes (`src/routes/_app.*`), Dashboard live `/readyz` badge (15s poll). CI (`.github/workflows/ci.yml`), docs skeletons, `.env.example`, `version.md`.

## Current phase
- P0 done — awaiting human browser test + approval. / left: nothing in P0 scope.

## Decisions log (1 line each)
- 2026-09-08 — dotenvy instead of unmaintained `dotenv` crate — same `.env` behavior, maintained.
- 2026-09-08 — sea-orm 2.0.2 (latest); `MigratorTrait` re-exported from `domain::migration` — trait moved in 2.0.
- 2026-09-08 — `/readyz` degraded returns HTTP 503 (UPSTREAM envelope) — health probes expect 503; Appendix A has no 503 code, UPSTREAM chosen.
- 2026-09-08 — CORS permissive in dev — locked down with security headers in P8 (spec).
- 2026-09-08 — TypeScript 7 (native): `baseUrl` removed, `paths` relative — new toolchain default.
- 2026-09-08 — sync-worker crate = stub — §6 says admin applies sync directly; crate kept for P5 tooling.

## Handoff notes for next agent
- `DATABASE_URL` (Neon pooled) is set in `.env` (gitignored); migrations applied to cloud, readyz = 200 db ok (verified live).
- `frontend/src/routeTree.gen.ts` is generated and COMMITTED — `bunx tsc --noEmit` alone doesn't run the Vite plugin.
- Frontend dev proxies `/api` → `127.0.0.1:8080`; no CORS needed in dev.
- Build order frontend: `bunx vite build` regenerates routeTree before tsc if routes changed.
- shadcn init: `--preset b1D0dv72 --pointer -y` (no `--base-color` flag on current CLI).

## Exit gate reminder
- §0 contract applies. Do not start next phase without approval.
