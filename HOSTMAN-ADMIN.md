# HOSTMAN-ADMIN — Build Specification (Admin Control Plane)

> **AUDIENCE:** AI coding agents. Implement this repo end-to-end, phase by phase.
> **HUMAN:** tests every phase in the browser (`bun run dev`), approves before you continue.
> **SIBLING:** `hostman/` (reseller node) is built AFTER this. Service catalog is registry-driven — pending full list never blocks phases.

---

## 0. AGENT CONTRACT — re-read at every phase start

1. Phases in §11. A phase is done only when 100% of its scope exists.
2. **Exit gate — ALL green:** backend `cargo fmt --check && cargo clippy -- -D warnings && cargo test` · frontend `bun install && bunx tsc --noEmit && bun run build && bun run vitest run` · `bun run dev` serves UI · human passes phase checklist visually.
3. **You STOP at two points only:** (a) phase complete → write report + version.md → STOP, wait for human to test and say "push and continue". (b) you need a credential/decision/URL from the human → STOP and ask. You NEVER push to git. You NEVER start the next phase. You NEVER self-authorize anything.
4. If you need ANYTHING — credential, decision, URL, clarification — STOP and ask. Do not guess. Do not fabricate. Do not use fake-looking placeholders. Tell the human exactly what you need and wait.
5. Human tests ONLY via frontend (`bun run dev`). Automated tests are your responsibility.

## 0.1 TOKEN COMPRESSION (hard contract — violation = wasted tokens = failed phase)

1. **Write code, not essays.** No explanations, no summaries, no "here's what I did." The code speaks. Phase report is the ONLY documentation.
2. **Never re-read the full spec.** Read `version.md` + your current phase section + the files you're editing. That's it.
3. **Never dump file contents into chat.** Write it, move on.
4. **No filler comments.** Comment only public APIs and genuinely non-obvious logic. One line max.
5. **Dense code.** Short variable names where context is clear.
6. **No repeated patterns.** Same validation twice → extract helper. Two routes share logic → share it.
7. **No planning monologues.** Don't narrate "I will now..." — just do it.
8. **Phase report = max 20 lines.** version.md = max 150 lines.
9. **If output exceeds what's needed, you're doing it wrong.**

## 0.2 HARD RULES (violation = failed phase)

- ORM only (SeaORM). Raw SQL forbidden (sea-orm-cli migrations excepted).
- Secrets only in `.env`; never hardcoded; `.env` gitignored.
- Never restyle shadcn preset `b1D0dv72` (flat, no shadows).
- Never invent features outside this spec; blockers go in phase report.
- **Platform monitors DISCLOSED devices only** — no code path may conceal monitoring from a device user.
- `../yaarsa/` (legacy PHP) = anti-pattern museum (Appendix D). Never replicate.

## 1. MISSION

Federated, disclosed device-monitoring platform (MDM-style, reseller-distributed + direct-user). This repo = admin control plane.

```
                    ┌─────────────────────────────────────┐
                    │    NEON / SUPABASE (cloud psql)     │
                    │  ADMIN'S PRIMARY DATABASE           │
                    │  all admin CRUD happens here        │
                    │  + node credential backup target    │
                    └────────────▲────────────────────────┘
                                 │ admin API = sole writer
                                 │ event-driven pushes from nodes
┌──────────────────┐  enroll    ┌───────────────────────────────────┐
│ ADMIN SERVER     │◄──────────►│ NODE RDP ×N (hostman/)            │
│ (THIS REPO, 1 box)  tokens  │ TWO types:                        │
│ • admin API (Rust)  sync ↑  │ • RESELLER node (reseller manages)│
│ • admin dashboard    flags ↓│ • DIRECT node (admin manages,     │
│   (React SPA)               │   buyer uses directly,            │
│   (NO local DB)             │   Netflix-style member logins)    │
└──────────────────┘           └───────────────────────────────────┘
```

**Admin has NO local database.** All admin CRUD = direct on Neon. Node has its own local Postgres. Credentials dual-written (cloud + local). Sync = event-driven only.

Nodes contact 5 endpoints: `enroll`, `sync-push`, `flags-pull`, `revoke-check`, `credentials-pull`. **Nodes never receive cloud DB credentials.**

## 2. DESIGN PRINCIPLES (non-negotiable)

| # | Principle |
|---|---|
| P1 | Cloud DB (Neon) = admin's primary store. All admin CRUD = direct on cloud. No local admin DB. |
| P2 | Node local Postgres = only writer for node data. Node pushes event-driven updates (business events only) to admin → admin writes to cloud. Zero cloud credentials on nodes. |
| P3 | Credentials (reseller + user) = dual-written to cloud AND node local. Normal = local. Failover = cloud via admin API. Failback = pull diff. First-time node = pull credentials cloud → local. |
| P4 | Signed everything: Ed25519 root key signs node tokens, sync ACKs, service catalog. |
| P5 | Kill switch: pause any node → token revoke effective ≤60s. |
| P6 | Audit log on every admin mutation. Secrets in `.env` only. |
| P7 | Immutable versioned artifacts; deploy = switch; one-command rollback. |
| P8 | Disclosure enforced platform-wide. Admin tracks `disclosure_attested` per node build. |
| P9 | TOTP-only auth. No email OTP. No SMTP. No email infrastructure anywhere. |
| P10 | **Multi-admin:** first admin = `mother_admin` (supreme, manages other admins). Other admins = full power EXCEPT admin management. |
| P11 | **Two node types:** `reseller` (reseller manages users) and `direct` (admin provisions, buyer uses directly, Netflix-style member logins, buyer sets own limits within hardware ceiling, exclusive device monitoring). |

## 3. TECH STACK (pinned — latest of everything, no downgrades)

### Backend (Rust — latest stable)

| Crate | Version | Purpose |
|---|---|---|
| axum | latest | HTTP framework |
| tokio | latest | async runtime |
| sea-orm | latest | ORM (only DB access) |
| tower-http | latest | trace, cors, limit, compression |
| argon2 | latest | password hashing |
| ed25519-dalek | latest | signing |
| x25519-dalek | latest | ECDH |
| aes-gcm | latest | field encryption + session crypto |
| totp-rs | latest | TOTP |
| jsonwebtoken | latest | JWT |
| validator | latest | input validation |
| tracing + tracing-subscriber | latest | JSON logs |
| config + dotenv | latest | env |
| uuid | latest | v7 |
| rustls / axum-server | latest | TLS |

### Frontend (latest everything)

| Tool | Version | Purpose |
|---|---|---|
| Bun | **latest** | package manager + dev runner + test runner |
| Vite | **latest** + **Rolldown** (Rust bundler) | build |
| React | **latest** | UI |
| TypeScript | **latest, strict** | types |
| **TanStack Query** | **latest** | all data fetching, caching, invalidation |
| **TanStack Router** | **latest** | type-safe routing |
| **TanStack Table** | **latest** | all tables (server-side) |
| **TanStack Virtual** | **latest** | 10k+ lists |
| **TanStack Form** | **latest** | all forms + zod |
| Zod | **latest** | schemas |
| shadcn/ui | **latest**, preset `b1D0dv72`, flat | UI |
| Tailwind CSS | **latest** | styling |
| Framer Motion | **latest** | slight animations |

### Infra
| Tool | Version |
|---|---|
| Docker | latest |
| GitHub Actions | latest |
| systemd / NSSM | — |
| Caddy + WireGuard | latest |

shadcn init: `bunx --bun shadcn@latest init --preset b1D0dv72 --template next --pointer`
(adapt to Vite if needed — keep exact preset tokens, never restyle)

## 4. REPO LAYOUT (create exactly)

```
hostman-admin/
├── backend/
│   ├── Cargo.toml
│   └── crates/{admin-api,domain,sync-worker,common}/
├── frontend/src/{app,features,components,lib,routes}
├── docs/{SETUP.md,TESTING.md,API.md,PHASE-REPORTS/}
├── .github/workflows/ci.yml
├── .env.example
├── version.md
└── README.md
```

## 5. GET-THESE-KEYS — prerequisites, costs, exact click-paths

| # | Credential | Cost | Click-path | Used for |
|---|---|---|---|---|
| 1 | Neon/Supabase DB URL | Free tier | neon.tech → sign up → **New Project** → **Connection Details → Pooled connection** → copy `postgresql://...` | Admin's primary database |
| 2 | Ed25519 root keypair | — | **Auto-generated** by setup script into `backend/keys/` | Node tokens, sync ACKs, catalog |
| 3 | Node enrollment tokens | — | Admin dashboard → **Nodes → Enroll** | Registering nodes |
| 4 | TOTP | — | Authenticator app (Google Authenticator / Authy / any) | Admin 2FA |
| 5 | Relay VPS (recommended) | ~$4–5/mo | Any VPS provider → Ubuntu 22.04 → IP + SSH key | Origin IP hiding |
| 6 | GitHub repo + PAT | Free | github.com → New private repo → fine-grained PAT | CI/CD |
| 7 | Domain | NOT USED | — TLS = self-signed + pinning; optional free `*.duckdns.org` via relay | — |

## 6. DATA MODEL (SeaORM — all in cloud DB)

All tables: `id uuid v7 pk`, `created_at/updated_at timestamptz`. **(PII)** = AES-256-GCM encrypted (`FIELD_ENC_KEY`).

**admin_users**: `login_code` 32-char Crockford unique (argon2 hash + 4-char prefix) · `password_hash` argon2id · `email` (PII, optional) · `totp_secret` (PII) · `totp_enabled` bool · `role` enum(`mother_admin`, `admin`) · `status` enum(`active`, `locked`). First created = `mother_admin`.

**resellers**: `name` · `login_code` (PII-hash) · `initial_password_hash` (one-time) · `totp_secret` (PII, set at first login) · `totp_enabled` bool · `email` (PII, optional) · `status` enum(`active`, `suspended`) · `max_users` int · `max_monitor_slots` int · `account_created_at` · `notes`.

**reseller_subscriptions**: `reseller_id` fk · `plan` enum(`basic`, `pro`, `custom`) · `starts_at` · `expires_at` · `is_renewed` bool · `renewal_days` int nullable · `status` enum(`active`, `expired`, `renewed`) · `renewed_from_id` nullable fk.

**nodes**: `name` · `region` · `reseller_id` nullable fk (NULL + `is_test` for test, NULL + `is_direct` for direct) · `is_test` bool · `is_direct` bool · `status` enum(`pending`, `approved`, `rejected`, `paused`, `revoked`) · `enroll_token_hash` · `pubkey` · `hw` jsonb {cpu_cores, ram_gb, disk_gb, os} · `max_devices` · `max_concurrent_streams` · `capacity_test` jsonb · `last_seen_at` · `disclosure_attested` bool.

**node_tokens**: `node_id` fk · `token_id` (jti) · `pubkey` · `issued_at` · `expires_at` nullable · `revoked_at` nullable.

**users** (cloud copy for oversight — node is primary writer): `node_id` fk · `login_code` (PII-hash) · `status` enum(`active`, `banned`) · `monitor_slots` int nullable (reseller users: 1–4; direct users: NULL = user sets own) · `concurrent_logins` int nullable (direct users only, set by buyer) · `max_monitors` int nullable (direct users only, set by buyer) · `sub_expires_at` · `account_created_at` · `is_renewed` bool · `renewal_days` int nullable.

**totp_members** (direct users only — cloud + local): `user_id` fk · `member_id` uuid unique · `label` text (buyer sets, e.g. "Person 1") · `totp_secret` (PII) · `totp_enabled` bool · `active` bool · `last_login_at`. Each member = own QR + own TOTP code.

**service_catalog**: `slug` unique · `name` · `description` · `default_state` enum(`active`, `disabled`, `maintenance`) · `apk_selectable` bool.

**service_flags**: `node_id` fk · `service_id` fk · `state` enum · unique(node_id, service_id).

**notifications**: `title` · `body` · `audience` enum(`all`, `nodes`, `resellers`, `direct`) · `created_by` · `sent_at`.

**audit_log**: `actor_admin_id` nullable fk · `action` · `target_type` · `target_id` · `meta` jsonb · `ip` · `at`.

**settings**: `key` unique · `value` jsonb.

**sync_inbox**: `node_id` · `seq` bigint · `payload` jsonb · `sig` · `received_at` · `applied_at` nullable · unique(node_id, seq). Admin applies directly to cloud tables (no separate worker).

**cloud_sync_state**: `node_id` · `last_applied_seq`.

## 7. AUTH & TOTP FLOW (TOTP only, Netflix-style member selection for direct users)

### 7.1 Admin login (mother + other admins — same flow)

**First time:** crypto code + password → QR page → scan → TOTP code → dashboard
**After:** crypto code + password → TOTP page → enter code → dashboard

### 7.2 Reseller login (same pattern)
First time: crypto code + one-time password → QR page → scan → TOTP verify → dashboard
After: crypto code + password → TOTP code → dashboard

### 7.3 Direct user login (Netflix-style member selection)

**First member (buyer) first login:**
```
Login page → crypto code + password
    → "You're the first member" → QR page → scan → TOTP code
    → set your limits (concurrent_logins, max_monitors — within hardware ceiling)
    → dashboard
```

**Returning member:**
```
Login page → crypto code + password
    → MEMBER SELECTION SCREEN (like Netflix profiles)
       shows tiles: [Person 1] [Person 2] [Person 3] [+ Add Member (buyer only)]
    → click your tile
    → TOTP page → enter YOUR code (each member has own secret)
    → dashboard (top bar shows: "Person 2 · Device-A")
```

### 7.4 Implementation
- `POST /auth/login` {login_code, password} → verify argon2id → if TOTP not enabled → return `{qr_secret, otpauth_url}` + temp token → `POST /auth/totp/verify` → session.
- If TOTP enabled (single-user: admin/reseller) → require `totp` → verify → session.
- Direct user: after code+password → `GET /auth/members` → returns list of active member tiles → `POST /auth/member/select` {member_id} → `POST /auth/totp/verify` {member_id, code} → session tagged with member_id.
- Session: JWT 15 min (httpOnly cookie) + rotating refresh 7d · session-scoped = dies on browser close.
- Lockout 5/15 min. Rate: login 5/min, general 120/min.

### 7.5 Multi-admin roles
- `mother_admin`: full power + `POST /admins` + `DELETE /admins/:id` (cannot remove self or other mother_admins).
- `admin`: everything EXCEPT admin management.

### 7.6 Direct user rules
- Buyer sets own limits: `concurrent_logins` + `max_monitors` (capped by node hardware from capacity test).
- Devices are EXCLUSIVE: one device = one monitor at a time across all members.
- Top bar: live sessions list (member label + device being monitored + status dot).
- Buyer can add/remove members (generate new QR / revoke existing).

### 7.7 Field encryption
PII (login_code hashes, totp_secret, email) encrypted AES-256-GCM with `FIELD_ENC_KEY`.

## 8. API SURFACE (`/api/v1`, JSON, Appendix A)

**Auth**: `POST /auth/login` · `POST /auth/totp/verify` · `POST /auth/refresh` · `POST /auth/logout` · `GET /auth/me` · `GET /auth/members` (direct: list member tiles) · `POST /auth/member/select` (direct: pick member)

**Admin management (mother only)**: `GET /admins` · `POST /admins` · `DELETE /admins/:id`

**Nodes**: `GET /nodes?status=&type=` · `GET /nodes/:id` · `POST /nodes/enroll-token` · `POST /nodes/:id/approve|reject|pause|resume|revoke`

**Node-facing (node JWT)**: `POST /node/enroll` · `GET /node/flags` · `GET /node/revoke-check` · `POST /node/heartbeat` · `POST /node/credentials-pull` · `POST /internal/sync`

**Resellers**: `GET /resellers` · `POST /resellers` · `PATCH /resellers/:id` · `POST /resellers/:id/renew` {days} · `GET /resellers/:id/tree`

**Direct users** (admin manages): `GET /direct-users?node=` · `POST /direct-users/:id/renew` {days} · `POST /direct-users/:id/ban|unban`

**Services**: `GET /services` · `POST /services` · `PATCH /services/:id` · `PUT /nodes/:id/flags`

**Notifications**: `POST /notifications` · `GET /notifications`

**Audit**: `GET /audit?actor=&action=&from=&to=`

**Health**: `GET /healthz` · `GET /readyz`

### 8.3 SERVICE REGISTRY
Adding service = `INSERT INTO service_catalog` + node handler. Zero schema changes. Seed: `screen_stream`, `camera_stream`.

## 9. SYNC CONTRACT (event-driven — NOT periodic)

**Node → admin pushes ONLY on business events:**

| Event | Data pushed |
|---|---|
| User created | user row (login_code, status, slots, sub_expiry, creation date) |
| User renewed | sub_expiry, is_renewed, renewal_days |
| User banned/unbanned | status |
| Reseller renewed | expiry (pushed to node via flags) |
| Node enrolled/paused/revoked | node status |

**Everything else (devices, history, favorites, builds, sessions, totp_members) = local only.**

**Neon compute:** only real business events. Busy node with 10k devices = same cloud cost as idle.

**Implementation:**
1. Node domain layer writes event mutation → `sync_outbox` (same transaction).
2. Immediately POSTs `POST {admin}/internal/sync` {node_jwt, batch: {seq, events[]}, sig} — signed node Ed25519.
3. Admin verifies → stores sync_inbox (idempotent) → applies to cloud tables → signed ACK.
4. No timer, no batcher. One event = one push. Immediate.

**Credentials dual-write:** credential written to cloud (admin) + node local (via sync event). Both always current.

**Failover:** node local DB fails → credentials read/write via admin API (proxy) → on recovery → `POST /node/credentials-pull` (diff by seq) → switch back to local.

**First-time node:** fresh → enroll → `POST /node/credentials-pull` → download all credentials → local DB → run offline.

## 10. CLOUD CREDENTIAL FLOW (cloud ↔ local)

- **Cloud = source of truth for credentials** (admin_users, resellers, users copy, totp_members copy).
- **Node local = working copy** for fast offline login.
- **Dual-write:** any credential change → written cloud-side (admin API) AND pushed to node local via sync events. Both always current.
- **First-time node:** `POST /node/credentials-pull` → download all active credentials → local DB.
- **Failover:** node local DB down → node auth proxies to admin API (read/write) → on recovery pulls diff by seq → resumes local.
- **Enforcement rule:** admin API is the FALLBACK, not the normal path — normal requests always use node local DB.

## 11. THE 10 PHASES (each = backend + frontend + automated tests + docs + report)

> Every phase ends with the full §0 exit gate. "Human check" = what the owner sees in the browser.

### P0 — Scaffold & Skeleton
**Backend:** cargo workspace (4 crates) · axum `/healthz` + `/readyz` (cloud db ping) · config/dotenv · JSON tracing · error envelope (Appendix A) · SeaORM connect to Neon + initial migration · CI workflow (§12).
**Frontend:** Vite + Rolldown scaffold + shadcn preset + TanStack (Query/Router/Form/Table/Virtual) + zod + Framer Motion · app shell: vertical sidebar + horizontal topbar, light/dark toggle, flat theme · routes stubbed: Login, Dashboard, Nodes, Resellers, Direct Users, Services, Notifications, Audit, Settings · Dashboard shows live `/readyz` badge.
**Human check:** shell renders both themes; Dashboard green "db ok".
**Also:** `version.md` (§14), `docs/SETUP.md` skeleton, phase report.

### P1 — Admin Auth (code + password + TOTP + multi-admin)
**Backend:** `admin_users` + `sessions` migrations · bootstrap owner (mother_admin) via setup · login flow §7 (argon2id, lockout 5/15) · TOTP enroll (QR) + verify · JWT 15min + rotating refresh · session-scoped cookie · rate limits · audit · multi-admin: role enum (mother_admin/admin), first = mother_admin, GET/POST/DELETE /admins mother-only.
**Frontend:** login (code → password → QR first / TOTP after) · first-login QR + recovery codes · auth guard · logout · admin management page (mother only: list/add/remove).
**Human check:** login works; first login QR; wrong TOTP rejected; tab close → re-login; mother sees admin mgmt, admin doesn't; add/remove admin works; 5 bad logins locked.

### P2 — Node Registry & Enrollment
**Backend:** `nodes` + `node_tokens` migrations · enroll-token issuance · node endpoints (§8: enroll/flags/revoke-check/heartbeat/credentials-pull) · approve/reject/pause/resume/revoke · signed node JWT · capacity_test + hw + disclosure_attested · flags signed (root key) · credentials-pull (reseller + user + totp_members rows).
**Frontend:** Nodes table (status/type/region/hw/last_seen/capacity) · Enroll dialog (token + admin pubkey + copy) · detail drawer (hw, capacity, actions) · status chips · type badge (reseller/direct/test).
**Agent test:** `just demo-node` registers fake node.
**Human check:** issue token → demo node pending → approve → pause → chip changes; revoke works; type badge correct.

### P3 — Resellers & Subscriptions + Direct Users
**Backend:** resellers + reseller_subscriptions migrations · create reseller (login_code + one-time password, shown once) · renew {days} (is_renewed, renewal_days, expiry) · suspend/activate · limits · tree · audit. Also: `direct_users` under nodes · `POST /direct-users/:id/renew {days}` · `POST /direct-users/:id/ban|unban`. All CRUD direct on Neon.
**Frontend:** Resellers TanStack Table (search/status/expiry) · create dialog → credentials modal (once, copy) · detail drawer (subscription card, renew, limits, creation date, renewal history) · Direct Users page: per-node list, renew/ban/unban buttons, hardware ceiling shown.
**Human check:** create reseller → modal shows code+password once; renew adds days; suspend flips status; direct user renew/ban works; audit entries present.

### P4 — Service Catalog & Flags
**Backend:** service_catalog + service_flags migrations · seed screen_stream, camera_stream · CRUD + per-node override (absent = inherit) · cat_version bump · signed flags payload.
**Frontend:** Services grid — cards with state select (active/disabled/maintenance) · per-node overrides drawer · test nodes ignore overrides note.
**Human check:** flip default → version bump visible; set override on node; reset works.

### P5 — Cloud Sync Ingest (event-driven)
**Backend:** `sync_inbox` + `cloud_sync_state` migrations · `POST /internal/sync` (verify node JWT + Ed25519 sig → idempotent → apply direct to cloud → signed ACK) · `POST /node/credentials-pull` (all credential rows, diff-aware by seq) · `GET /cloud/users?node=&q=` · `GET /cloud/status` (per-node freshness).
**Frontend:** Cloud section: users readonly table · Cloud Status: per-node freshness badge.
**Agent test:** demo-node pushes signed event batches.
**Human check:** cloud shows synced user rows; freshness green; user banned event updates cloud view.

### P6 — Notifications & Audit UI
**Backend:** notifications broadcast (audience: all/nodes/resellers/direct) + list · audit filters (actor/action/date) paginated + indexed.
**Frontend:** bell + unread dropdown + notifications page · compose dialog · Audit page: TanStack Table, filters, date-range, CSV export.
**Human check:** broadcast → appears; audit filters correct; CSV downloads.

### P7 — Kill Switch, Revocation, Test/Direct Servers, Live Health
**Backend:** pause → flags payload `paused:true` + revoke-check honors · revoke → token dead ≤60s · heartbeat metrics jsonb served · `is_test`: flags always all-active · `is_direct`: admin direct-user panel enabled.
**Frontend:** node detail "Health" tab (connections, CPU/RAM, streams — sparklines) · Pause/Resume confirm · Revoke typed confirm ("REVOKE") · TEST badge · DIRECT badge + manage-direct-users button.
**Human check:** pause demo node → chip + flags paused; revoke → dead; TEST ignores overrides; DIRECT shows manage panel.

### P8 — Hardening & Ops
**Backend:** rate limits all routes · security headers (CSP, X-Content-Type-Options, X-Frame-Options DENY, Referrer-Policy) · PII encryption tests · `cargo audit` in CI · root-key rotation (settings history, re-sign catalog) · `/readyz` includes cloud reachability.
**Frontend:** Settings page: key rotation status, relay status, inbox depth.
**Human check:** Settings green panels; headers in devtools.

### P9 — Docs, Cold-Start, Release
**Deliverables:** final `docs/API.md` (§16) · final `docs/SETUP.md` (5-step dual-OS, GET-THESE-KEYS §5, relay §12.4) · final `docs/TESTING.md` (§16) · `version.md` final · tag `v1.0.0` + release artifacts.
**Human check:** clean machine + SETUP.md alone → system fully works; sign-off.

## 12. CI/CD, DEPLOY, ROLLBACK, RELAY

**12.1 CI:** backend (`fmt`, `clippy -D warnings`, `test`, `cargo audit`) + frontend (`bun install`, `tsc --noEmit`, `vitest run`, `build`). Red CI = phase not approvable.
**12.2 Release:** tag → `admin-api` binaries (linux-gnu + windows-gnu) + embedded frontend dist → GitHub Release. Immutable.
**12.3 Deploy & rollback:** single binary (API + UI). systemd/NSSM. Deploy: upload → migrate → stop → swap → start → poll /healthz 30s → auto-restore previous (last 5). Manual: `admin-cli rollback <version>`.
**12.4 Origin IP hiding:** relay VPS (Caddy + WireGuard). Firewall allows ONLY WireGuard interface. Caddy TLS (self-signed or free `*.duckdns.org`) → proxy over tunnel. Strip Server headers. Public sees only relay IP.

## 13. SETUP — 5 STEPS (content for `docs/SETUP.md`; Linux Ubuntu 22.04 AND Windows Server via RDP)

1. **Install deps** — Linux: `scripts/install-linux.sh` (rustup, caddy optional). Windows: `scripts/install-windows.ps1` (winget: rustup). No local Postgres (cloud = Neon).
2. **Clone** — `git clone <repo> && cd hostman-admin`.
3. **Fill `.env`** — `cp .env.example .env`; values per Appendix B; Neon URL from §5.
4. **Run setup** — `cargo run --release -p admin-api -- setup` → Ed25519 root keypair into `backend/keys/` → migrations on Neon → bootstrap mother admin (prompts/env) → service configs (systemd/NSSM).
5. **Start** — Linux: `systemctl enable --now hostman-admin`. Windows: NSSM. Open via relay → login → `/readyz` green.

**13.2 `.env` reference → Appendix B. Never commit `.env`.**

## 14. version.md TEMPLATE (create at P0; update at EVERY phase end; keep <150 lines)

```markdown
# HOSTMAN-ADMIN — version ledger
updated: <date> · phase: <N>/10 · status: <working|blocked|awaiting-approval>
## Completed
- P0 scaffold: <one line — what exists, where>
## Current phase
- P<N> <name> — done: <…> / left: <…>
## Decisions log (1 line each)
- <date> — <decision> — <why>
## Handoff notes for next agent
- <gotchas: env quirks, demo credentials location, flaky tests>
## Exit gate reminder
- §0 contract applies. Do not start next phase without approval.
```

## 15. PHASE REPORT TEMPLATE (`docs/PHASE-REPORTS/phase-N.md`)

```markdown
# Phase N — <name>
scope: <one line>
built: backend <crates/files> · frontend <pages/components>
checks: fmt ✓ clippy ✓ cargo test <N>✓ · tsc ✓ vitest <N>✓ build ✓
human test: 1) bun run dev  2) <exact click-path>  3) expected: <…>
known limitations: <…>
blockers/questions for owner: <…>
```

## 16. TESTING & API DOCS STANDARD

**16.1** Human approval = browser-only (`bun run dev`). Agents never require owner to run CLI/API for verification.
**16.2** `docs/TESTING.md` — per phase: preconditions · exact click-path · expected result · failure look.
**16.3** `docs/API.md` — per route: method · path · auth (`admin-session` | `node-jwt` | `public`) · request/response fields · error codes (Appendix A) · example. Kept in sync with code.
**16.4** `just reset-db` reseeds demo data.

## Appendix A — Error envelope

```json
{"ok": false, "error": {"code": "SLOT_LIMIT", "msg": "human-readable", "details": {}}}
{"ok": true, "data": {…}}
```
Codes: `VALIDATION` 400 · `AUTH` 401 · `TOTP` 401 · `FORBIDDEN` 403 · `NOT_FOUND` 404 · `LOCKED` 423 · `CONFLICT` 409 · `SLOT_LIMIT` 409 · `DEVICE_OFFLINE` 409 · `NODE_PAUSED` 403 · `MEMBER_LIMIT` 409 · `RATE_LIMIT` 429 · `INTERNAL` 500 · `UPSTREAM` 502.

## Appendix B — `.env` reference (admin)

| Var | Example | Notes |
|---|---|---|
| `DATABASE_URL` | postgres://…@neon…/hostman | Neon pooled — admin's ONLY database |
| `FIELD_ENC_KEY` | base64 32 bytes | PII encryption |
| `ROOT_KEY_PATH` | backend/keys/root.ed25519 | auto-generated |
| `ADMIN_SETUP_CODE` / `ADMIN_SETUP_PASSWORD` | — | bootstrap only, delete after |
| `BIND` | 127.0.0.1:8080 | behind relay only |
| `RUST_LOG` | info | tracing |

## Appendix C — Defaults

Login lockout 5 fails/15 min · access JWT 15 min · refresh 7 d · TOTP window ±1 step · rate login 5/min, general 120/min · node heartbeat 60 s · flags TTL 60 s · cloud freshness green <1 h.

## Appendix D — ANTI-PATTERNS (from legacy `yaarsa` — forbidden)

1. Hardcoded secrets/keys/IVs in source → `.env` only.
2. New DB connection per function + raw SQL strings → pooled SeaORM.
3. Obfuscated filenames (`yarsap_14881.php`) → descriptive modules.
4. Binaries committed to repo (composer.phar, 1.1 GB stores) → releases + `.gitignore`.
5. 200-line `.htaccess` UA blocklists → rate limiting + edge controls.
6. Wrong error codes on pages → envelope + correct status.
7. Artificial `usleep()` delays → never.
8. Copy-pasted near-duplicate pages → shared components.
9. No tests/migrations/docs → CI gates all.
10. Business logic in templates → services layer.

*End of HOSTMAN-ADMIN specification.*




