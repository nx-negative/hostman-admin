# Phase 0 — Scaffold & Skeleton
scope: cargo workspace + axum health/ready + SeaORM migration + Vite/shadcn/TanStack SPA shell + CI.
built: backend `common` (config/logging/error envelope) · `domain` (settings entity, migrator, initial migration) · `admin-api` (healthz/readyz, setup subcommand, router tests) · `sync-worker` (stub) · frontend: app shell (`src/app/shell.tsx`), theme provider (light/dark, persisted), 9 stub routes, Dashboard live readyz badge, api client + envelope parser + env schema (zod), vitest unit tests · `.github/workflows/ci.yml` · docs skeletons · `.env.example`.
checks: fmt ✓ clippy ✓ cargo test 5✓ · tsc ✓ vitest 8✓ build ✓ · `bun run dev` smoke ✓ (healthz OK via proxy, readyz 503-degraded, UI 200).
human test: 1) `cargo run -p admin-api` in `backend/` + `bun run dev` in `frontend/`  2) open http://localhost:5173 → Dashboard → toggle theme → click all sidebar items + /login  3) expected: shell renders in both themes; badge shows **db ok** once `DATABASE_URL` is in `.env` (currently red **db down** = degraded mode, by design).
known limitations: no auth (P1), no nodes (P2), CORS permissive dev-only (P8 hardens), `setup` runs migrations only — keygen/bootstrap in P1.
blockers/questions for owner: need Neon/Supabase pooled `DATABASE_URL` (§5 #1) to flip Dashboard badge green and run migrations on the cloud DB.
