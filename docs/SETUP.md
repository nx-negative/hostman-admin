# SETUP — hostman-admin (P0 skeleton)

> Final 5-step guide lands in P9. Below: what works today.

## 1. Install deps
- Rust stable (rustup) · Bun latest · no local Postgres (cloud = Neon).

## 2. Clone
```sh
git clone <repo> && cd hostman-admin
```

## 3. Fill `.env`
```sh
cp .env.example .env
```
`DATABASE_URL` = Neon/Supabase **pooled** connection string (§5 #1). See Appendix B of the spec.

## 4. Run setup (migrations)
```sh
cd backend
cargo run -p admin-api -- setup
```
P0: runs the initial migration (`settings`). Ed25519 keypair + mother-admin bootstrap arrive in P1.

## 5. Start
```sh
cargo run -p admin-api          # terminal 1 — API on 127.0.0.1:8080
cd ../frontend && bun install && bun run dev   # terminal 2 — UI on http://localhost:5173
```
Dashboard must show a green **db ok** badge.
