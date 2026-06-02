## 1. Crate Scaffold

- [ ] 1.1 Add `crates/ez-booth-server` to workspace `Cargo.toml` members list
- [ ] 1.2 Create `crates/ez-booth-server/Cargo.toml` — binary crate; dependencies: `axum`, `sqlx` (sqlite + postgres features), `clap`, `tokio`, `tower`, `tower-http`, `tracing`, `tracing-subscriber`, `anyhow`, `thiserror`, `serde`, `serde_json`, `uuid`, `chrono`, `argon2`, `rand`, `qrcode`; feature flag `multi-tenant = []`
- [ ] 1.3 Create `crates/ez-booth-server/src/main.rs` — parse CLI args via `clap`; dispatch to `serve` subcommand or `key` / `tenant` subcommands; initialize tracing subscriber
- [ ] 1.4 Create `crates/ez-booth-server/src/config.rs` — load `DATABASE_URL`, `CORS_ALLOWED_ORIGINS`, `OIDC_ISSUER_URL`, `SERVER_URL` from environment; validate at startup; implement auth conflict check (OIDC + API keys → exit with error)
- [ ] 1.5 Create `crates/ez-booth-server/src/db.rs` — establish `sqlx` pool from `DATABASE_URL`; detect SQLite vs Postgres; run `sqlx::migrate!()` at startup; expose pool as shared state

## 2. Database Migrations

- [ ] 2.1 Create `crates/ez-booth-server/migrations/0001_initial.sql` — tables: `events` (id, entity_id, event_type, payload, sequence, client_id, event_code, created_at), `api_keys` (id, key_hash, tenant_id, role, label, created_at, last_used_at, revoked_at), `pairing_codes` (code, key_id, expires_at, used_at)
- [ ] 2.2 Create `crates/ez-booth-server/migrations/0002_purchase_dedup_index.sql` — `CREATE UNIQUE INDEX idx_events_purchase_dedup ON events (entity_id) WHERE event_type = 'purchase_upserted'` (moved from qr-labels-webcam-mobile-sync task 2.1)
- [ ] 2.3 Create `crates/ez-booth-server/migrations/0003_booth_event_code.sql` — `ALTER TABLE events ADD COLUMN IF NOT EXISTS event_code TEXT`; backfill from payload; add NOT NULL constraint (moved from qr-labels-webcam-mobile-sync task 2.2)

## 3. Sync API Routes

- [ ] 3.1 Create `crates/ez-booth-server/src/routes/mod.rs` — register all routes on an `axum::Router`; attach CORS layer from `tower-http`; attach request tracing layer
- [ ] 3.2 Create `crates/ez-booth-server/src/routes/sync.rs` — implement `POST /api/sync`: deserialize body `{ purchases, client_id }`, insert via `ON CONFLICT DO NOTHING`, log collision warning when two `client_id` values share `event_code` in the same calendar month, return `{ accepted: <count> }`
- [ ] 3.3 Implement `GET /api/sync?since={seq}` in `routes/sync.rs` — query `events` where `sequence > since` and `event_type = 'purchase_upserted'`, limit 500, return `{ purchases, next_sequence }`; guarantee `next_sequence > since` when result is non-empty
- [ ] 3.4 Create `crates/ez-booth-server/src/routes/health.rs` — implement `GET /health`; check DB connectivity; return `{ "status": "ok" }` (200) or `{ "status": "degraded", "detail": "..." }` (503)
- [ ] 3.5 Create `crates/ez-booth-server/src/routes/pair.rs` — implement unauthenticated `POST /api/pair`; validate code exists, is unexpired, and unused; mark `used_at`; return `{ key, role }`; return 410 for expired/used, 404 for unknown

## 4. Auth Middleware

- [ ] 4.1 Create `crates/ez-booth-server/src/auth/mod.rs` — define `AuthContext` enum (`NoAuth`, `ApiKey { key_id, tenant_id, role }`, `OidcClaims { sub, tenant_id, role }`) and `AuthRole` enum (`Organiser`, `Cashier`); implement `require_organiser(ctx)` and `require_cashier(ctx)` guards
- [ ] 4.2 Create `crates/ez-booth-server/src/auth/api_key.rs` — axum middleware: extract `Authorization: Bearer <key>` header; hash input with argon2; look up in `api_keys` where `revoked_at IS NULL`; update `last_used_at`; reject with 401 if missing or invalid; inject `AuthContext` into request extensions
- [ ] 4.3 Create `crates/ez-booth-server/src/auth/oidc.rs` — fetch OIDC discovery document at startup from `{OIDC_ISSUER_URL}/.well-known/openid-configuration`; cache JWKS; validate `Authorization: Bearer <jwt>` on each request; map `groups` claim to `AuthRole` (Authentik: `ez-booth-organisers` → Organiser, `ez-booth-cashiers` → Cashier); refresh JWKS on signature failure with rate limit; inject `AuthContext`
- [ ] 4.4 Implement auth mode selection in `config.rs` — at startup: if `OIDC_ISSUER_URL` set → OIDC mode; else if API keys exist in DB → API key mode; else → no-auth mode; if OIDC + keys both present → exit with error; wire selected middleware into router
- [ ] 4.5 Implement multi-tenant auth guard (feature-flagged) — at startup with `--features multi-tenant`: exit with error if neither OIDC nor any API keys are configured

## 5. CLI — Key Management

- [ ] 5.1 Create `crates/ez-booth-server/src/cli/mod.rs` — top-level `clap` command structure: `serve`, `key`, `tenant` (tenant subcommand behind `#[cfg(feature = "multi-tenant")]`)
- [ ] 5.2 Create `crates/ez-booth-server/src/services/key.rs` — service functions: `create(pool, role, label) -> (KeyId, PlaintextKey)`, `list(pool, include_revoked) -> Vec<KeyRecord>`, `revoke(pool, id) -> Result`, `generate_pairing_code(pool, key_id, server_url) -> PairingCode`; all DB interaction lives here
- [ ] 5.3 Create `crates/ez-booth-server/src/cli/key.rs` — thin adapter over `services::key`; implement `key create --role --label`, `key list [--include-revoked]`, `key revoke --id`, `key pair --id [--direct]`
- [ ] 5.4 Implement terminal QR output in `key pair` — use `qrcode` crate to render `ez-booth://pair?s={server_url}&c={code}` as unicode block characters; print 6-digit code below for manual entry fallback
- [ ] 5.5 Implement `key pair --direct` — render `ez-booth://setup?s={server_url}&k={plaintext_key}&r={role}` as QR; prepend prominent warning to output

## 6. Multi-Tenant Provisioning (feature-flagged)

- [ ] 6.1 Create `crates/ez-booth-server/src/services/tenant.rs` — service functions: `create(pool, name) -> TenantId`, `list(pool) -> Vec<TenantRecord>`, `delete(pool, id) -> Result`; `create` must: generate UUID, `CREATE SCHEMA tenant_{uuid}`, run migrations in schema, insert into `public.tenants`; `delete` must: `DROP SCHEMA CASCADE`, remove from `public.tenants`
- [ ] 6.2 Create `crates/ez-booth-server/src/cli/tenant.rs` — thin adapter over `services::tenant`; implement `tenant create --name`, `tenant list`, `tenant delete --id [--force]`; `delete` without `--force` prompts for confirmation
- [ ] 6.3 Create `crates/ez-booth-server/src/middleware/tenant.rs` — axum middleware (multi-tenant only): extract `tenant_id` from `AuthContext`; set `SET search_path TO tenant_{uuid}` at the start of every transaction; apply before route handlers
- [ ] 6.4 Add per-tenant key management to CLI — `tenant key create --tenant <id> --role --label`, `tenant key pair --tenant <id> --id`, `tenant key revoke --tenant <id> --id`, `tenant key list --tenant <id>`; delegate to `services::key` with tenant scope

## 7. Server Entrypoint and Graceful Shutdown

- [ ] 7.1 Implement `serve` subcommand in `main.rs` — bind axum server on configurable host:port (env: `SERVER_HOST`, `SERVER_PORT`, default `0.0.0.0:3000`); wire all middleware and routes
- [ ] 7.2 Implement graceful shutdown — listen for `SIGTERM` and `Ctrl-C`; allow in-flight requests to complete before closing the pool and exiting

## 8. Docker and Distribution

- [ ] 8.1 Create `crates/ez-booth-server/Dockerfile` — multi-stage build: `rust:alpine` builder stage compiles release binary; minimal `alpine` runtime stage copies binary; exposes port 3000; sets `DATABASE_URL` as required env
- [ ] 8.2 Create `docker-compose.sqlite.yml` at repo root — single service `ez-booth-server` with SQLite volume mount; documents required env vars
- [ ] 8.3 Create `docker-compose.postgres.yml` at repo root — services: `ez-booth-server` + `postgres`; Postgres `DATABASE_URL` pre-wired; documents optional `OIDC_ISSUER_URL` and `CORS_ALLOWED_ORIGINS`

## 9. CI

- [ ] 9.1 Create `.github/workflows/server.yml` — trigger on push/PR touching `crates/ez-booth-server/**` or `Cargo.toml`; steps: `cargo test -p ez-booth-server`, `cargo test -p ez-booth-server --features multi-tenant`, `cargo clippy -p ez-booth-server -- -D warnings`
- [ ] 9.2 Add `Server Build` as a required status check in branch protection (update `.github/branch-protection.yml` or document in AGENTS.md)
- [ ] 9.3 Add Docker build+push step to `server.yml` triggered on `server-v*` tags — build multi-arch image (linux/amd64, linux/arm64); push to `ghcr.io/tschuba/ez-booth-server`

## 10. Adapt qr-labels-webcam-mobile-sync

- [ ] 10.1 Update `openspec/changes/qr-labels-webcam-mobile-sync/proposal.md` — add "Requires: ez-booth-server change"; remove server crate from "What Changes" section
- [ ] 10.2 Update `openspec/changes/qr-labels-webcam-mobile-sync/tasks.md` — remove tasks 2.1, 2.2 (migrations, now in ez-booth-server); remove tasks 6.1–6.4 (server crate, routes, route registration, now in ez-booth-server)
- [ ] 10.3 Update `openspec/changes/qr-labels-webcam-mobile-sync/design.md` §11 — replace server implementation details with reference to ez-booth-server change; retain client-side wire format (request/response shapes, pagination contract, error handling)
