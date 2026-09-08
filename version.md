# HOSTMAN-ADMIN — version ledger
updated: 2026-09-08 · phase: 1/10 · status: awaiting-approval

## Completed
- P0 scaffold: backend workspace (axum/SeaORM/health), frontend shell (Vite+shadcn b1D0dv72+TanStack), CI, docs.
- P1 admin auth: argon2id + TOTP (totp-rs 6 Builder) + EdDSA session JWT (root Ed25519 key, `aws_lc_rs` provider); admin_users/admin_sessions/recovery_codes/audit_log (SeaORM 2.0 + PG enums); rotating refresh 7d + access 15min httpOnly session cookies; lockout 5/15 + rate 5/min login, 120/min general; mother_admin-only admin mgmt (create=list+one-time creds, delete, cannot remove self/mother); setup bootstraps mother admin + auto-gens FIELD_ENC_KEY + root key. Frontend: login wizard (credentials→QR→TOTP→recovery codes), AuthProvider, shell logout + user badge + conditional Admins nav, admins page. Live-verified on Neon.

## Current phase
- P1 done — awaiting human browser test + approval. / left: nothing in P1 scope.

## Decisions log (1 line each)
- 2026-09-08 — jsonwebtoken 11 requires explicit CryptoProvider → enabled `aws_lc_rs` feature (common + admin-api).
- 2026-09-08 — sea-orm-migration 2.0 `enumeration()` only sets column type; PG enum types created explicitly via `CREATE TYPE ... AS ENUM` in migration up.
- 2026-09-08 — ed25519-dalek 3.0: `to_pkcs8_pem` needs `pem` feature; `Signature::from_bytes` infallible; `Verifier` trait must be in scope for `verify`.
- 2026-09-08 — totp-rs 6: `TOTP::new` deprecated → `Builder::new().with_*().build()`; `check_current` returns `Option<u64>` (`.is_some()`); `get_url`→`to_url`, 0 args.
- 2026-09-08 — argon2 0.6 / password-hash 0.6: `hash_password(password)` auto-generates salt (no SaltString); `verify_password(password, hash)`.
- 2026-09-08 — rand 0.10: `fill`/`random_range` via `RngExt` trait (must import).
- 2026-09-08 — auth guard done component-level (window.location redirect) not router-context beforeLoad (context typing didn't flow).
- 2026-09-08 — new admins get system-generated login_code + one-time password shown once in modal (mirrors reseller flow in P3).

## Handoff notes for next agent
- Mother admin credentials printed by `setup` (shown once) — for local dev: code `28RBVVVQGDB5A766RPDAQBJ6D8VTFBM0`, pw `Y9YAX-QATE8` (this Neon DB). TOTP secret `TULEFH36HQUJ2QCWS327N6FI6IHEHEGW`.
- `FIELD_ENC_KEY` now set in `.env` (gitignored); root key at `backend/keys/root.ed25519` (gitignored).
- Frontend: `bunx vite build` regenerates `src/routeTree.gen.ts` (TanStack router plugin) — run it after adding routes, before `tsc`.
- shadcn `@base-ui` DialogTrigger has NO `asChild` — apply button classes directly.
- Build is slow (~1-2 min) due to aws-lc-sys C compile from jsonwebtoken's `aws_lc_rs`.

## Exit gate reminder
- §0 contract applies. Do not start next phase without approval.

