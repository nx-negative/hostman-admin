# TESTING — hostman-admin

> Browser-only testing for the human (§16.1). Detailed per-phase click-paths land with each phase.

## P0 — Scaffold & Skeleton
**Preconditions:** `.env` has `DATABASE_URL`; API running (`cargo run -p admin-api` in `backend/`); UI running (`bun run dev` in `frontend/`).

| # | Click-path | Expected | Failure look |
|---|---|---|---|
| 1 | Open http://localhost:5173 → Dashboard | Shell: sidebar (8 items) + topbar | blank page / 404 |
| 2 | Topbar sun/moon icon | Theme flips light ↔ dark, persists on reload | theme resets |
| 3 | Dashboard "System" card | Green dot **db ok** (needs `.env` + API running) | red **db down** / amber checking… |
| 4 | Click every sidebar item + /login | Each stub page renders with its phase badge | render error |
