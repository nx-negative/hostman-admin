# API — hostman-admin

Base: `/api/v1` · JSON · envelope per Appendix A (`{"ok":true,"data":…}` / `{"ok":false,"error":{code,msg,details}}`).

## Health (P0)

### `GET /healthz` · `GET /api/v1/healthz` — public
```json
{"ok": true, "data": {"status": "ok"}}
```

### `GET /readyz` · `GET /api/v1/readyz` — public
Cloud DB ping. Also mounted at root for load-balancer probes.

- 200 → `{"ok": true, "data": {"status": "ok", "db": "ok"}}`
- 503 → `{"ok": false, "error": {"code": "UPSTREAM", "msg": "database unreachable", "details": {"db": "down"}}}`

## Auth (P1) — TOTP-only, §7

All auth routes are under `/api/v1`. Session = httpOnly cookie `hm_access` (15 min) + rotating refresh `hm_refresh` (7 d). Both browser-session-scoped (no Max-Age).

### `POST /auth/login` — admin-session | JSON `{login_code, password}`
- 200 `{"state":"totp_enroll","temp_token":"...","otpauth_url":"otpauth://..."}` — first login, scan QR
- 200 `{"state":"totp_required","temp_token":"..."}` — returning user
- 400 `VALIDATION` · 401 `AUTH` · 423 `LOCKED` · **409 `CONFLICT`** if a live session already exists for this account (single-session, §7.4: log out the other tab first) · 429 `RATE_LIMIT`
- Rate: 5/min per IP. Lockout: 5 fails → 15 min.

### `POST /auth/totp/verify` — public | JSON `{temp_token, code}`
- 200 `{admin, recovery_codes:[...]}` — recovery codes present only on first verify. Sets session cookies.
- 401 `TOTP` (wrong code) · 401 `AUTH` (bad/expired temp token)
- Recovery codes (one-time) accepted as fallback to TOTP.

### `POST /auth/refresh` — public (refresh cookie) · `POST /auth/logout` — public
### `GET /auth/me` — admin-session → `{admin:{id,role,code_prefix,totp_enabled}}` · 401 if no session

## Admin management (P1) — mother_admin only, §7.5

### `GET /admins` · `POST /admins {role,email?}` · `DELETE /admins/{id}`
- `GET`: list all admins. `POST`: returns `{id, login_code, password}` (shown once). `DELETE`: cannot remove self or another mother_admin.
- 403 `FORBIDDEN` for non-mother · 401 if no session.
