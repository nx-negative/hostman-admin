# TESTING — hostman-admin

> Browser-only testing for the human (§16.1). Detailed per-phase click-paths land with each phase.

## P1 — Admin Auth (code + password + TOTP + multi-admin)
**Preconditions:** `.env` has `DATABASE_URL` + `FIELD_ENC_KEY` (or run `cargo run -p admin-api -- setup`); API running; UI running.

| # | Click-path | Expected | Failure look |
|---|---|---|---|
| 1 | Open http://localhost:5173 → redirects to /login | Login form (code + password) | stays on dashboard |
| 2 | Enter mother-admin code + password → Continue | QR page + 6-digit input | error text |
| 3 | Enter wrong code → Verify | "Invalid code" | — |
| 4 | Scan QR in authenticator, enter code → Verify | Recovery codes page (10 codes, shown once) | — |
| 5 | "I've saved them" → Dashboard | Green db ok, topbar shows prefix + `mother_admin` badge, Admins in nav | — |
| 6 | Click Admins → Add admin → Create | Modal shows generated code + password (copy) | — |
| 7 | New admin logs in → Admins page | "Only mother_admin can manage admins" | sees admin list |
| 8 | Logout (topbar icon) → redirects to /login | Login form | — |
| 9 | Close tab, reopen → /login (session-scoped cookie gone) | Login form | still logged in |
