# hostman-admin

Admin control plane for the Hostman disclosed device-monitoring platform (spec: `HOSTMAN-ADMIN.md`).

Admin has **no local database** — all CRUD goes to the cloud Postgres (Neon/Supabase). The sibling `hostman/` reseller node repo is built after this one.

## Dev quick start

```sh
cp .env.example .env              # fill DATABASE_URL (Neon pooled connection)
cd backend
cargo run -p admin-api -- setup   # runs migrations on the cloud DB
cargo run -p admin-api            # serves API on 127.0.0.1:8080
cd ../frontend
bun install
bun run dev                       # Vite dev server on :5173, proxies /api → :8080
```

Progress ledger: `version.md` · Phase reports: `docs/PHASE-REPORTS/`.
