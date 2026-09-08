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

## Setup subcommand
`admin-api setup` — connects to `DATABASE_URL`, applies all migrations (P0: `settings` table). Keygen + bootstrap: P1.
