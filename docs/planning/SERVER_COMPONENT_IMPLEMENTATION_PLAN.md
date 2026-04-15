# Server Component Implementation Plan

## Purpose

Add a new server component to support:

- Sync data across devices
- Multi-user collaboration
- Centralized backup ingestion

The existing offline-first client behavior must be preserved.

## Final Decisions

| Concern | Decision |
|---|---|
| Server crate | `crates/ez-booth-server` |
| HTTP framework | `axum` |
| Persistence | PostgreSQL (`sqlx`) |
| Sync model | Real-time WebSocket + manual pull/push |
| Conflict model | Append-only event log with full entity snapshots in events |
| Auth | Shared API key (`X-Ez-Booth-Key`) |
| Backup endpoint | Reuse existing `BackupData` JSON format |
| Offline support | Keep IndexedDB offline-first behavior |
| Client identity | Auto UUID + optional user-defined display name |
| Sequence cursor storage | `localStorage` |
| Primary deployment | Docker Compose in Coolify |
| Image strategy | Pre-built image via GHCR |

## Architecture Overview

### New Workspace Crate

Create `crates/ez-booth-server` and add it to the workspace. This crate exposes HTTP and WebSocket interfaces for sync and backup while reusing domain/storage types where appropriate.

Suggested module layout:

```
crates/ez-booth-server/src/
  main.rs
  config.rs
  error.rs
  app_state.rs
  broadcast.rs
  db/
    mod.rs
    pool.rs
  middleware/
    mod.rs
    auth.rs
  models/
    mod.rs
    event.rs
    backup.rs
  routes/
    mod.rs
    health.rs
    events.rs
    snapshot.rs
    backup.rs
    ws.rs
```

### PostgreSQL Schema

Use migrations for three core tables:

1. `events` (append-only):
   - `id` UUID PK
   - `event_type` TEXT
   - `entity_id` TEXT
   - `booth_id` UUID NULL
   - `payload` JSONB
   - `client_id` TEXT
   - `occurred_at` TIMESTAMPTZ
   - `sequence` BIGSERIAL

2. `backups`:
   - `id` UUID PK
   - `client_id` TEXT
   - `received_at` TIMESTAMPTZ
   - `app_version` TEXT
   - `checksum` TEXT NULL
   - `payload` JSONB

3. `clients`:
   - `client_id` TEXT PK
   - `display_name` TEXT NULL
   - `platform` TEXT NULL
   - `last_seen_at` TIMESTAMPTZ

### API Surface

- `GET /health` (no auth)
- `GET /events?after={seq}`
- `POST /events`
- `GET /snapshot`
- `POST /backup`
- `GET /backups`
- `GET /backups/{id}`
- `GET /ws` (WebSocket)

All routes except `/health` require `X-Ez-Booth-Key`.

### WebSocket Flow

- Client subscribes with `after` sequence cursor.
- Server sends catch-up events.
- Server streams new events via broadcast channel.
- Client persists newest `sequence` in `localStorage`.

## Implementation Phases

### Phase 1: Server Scaffold

- Add `crates/ez-booth-server` to workspace.
- Add base dependencies: `axum`, `tokio`, `tower-http`, `sqlx`, `serde`, `serde_json`, `uuid`, `chrono`, `thiserror`, `dotenvy`.
- Implement startup wiring (`main.rs`) and typed config parsing.

### Phase 2: Database and Migrations

- Add migration setup using `sqlx::migrate!()`.
- Create migrations for `events`, `backups`, `clients`.
- Add indexes:
  - `events(sequence)`
  - `events(booth_id)`
  - `events(entity_id)`
  - `backups(received_at)`

### Phase 3: Models and Validation

- Define `EventType`, `EventRecord`, `NewEvent`.
- Define backup metadata models.
- Reuse and validate incoming `BackupData` format via storage export/import validation components where possible.

### Phase 4: HTTP Routes

- Implement health route.
- Implement event write/read routes with sequence-based pagination.
- Implement backup ingest and listing routes.
- Implement snapshot route by replaying latest effective state from event stream.

### Phase 5: Real-Time Broadcast

- Add `tokio::sync::broadcast` for event fan-out.
- Broadcast on successful event persistence.
- Implement WebSocket subscription and reconnection-safe catch-up semantics.

### Phase 6: Client Integration (`crates/ez-booth-ui`)

- Add server sync configuration in settings (URL, API key, display name).
- Add client UUID generation and persistence in `localStorage`.
- On local mutations, enqueue/push server events.
- On incoming events, apply via repositories with replace semantics.
- Add manual `Sync now` action (`GET /events?after=...`).

### Phase 7: Testing and Validation

- Server unit tests:
  - auth middleware
  - event serialization/deserialization
  - route error mapping
- Server integration tests with Postgres:
  - event ingestion + retrieval order
  - backup ingestion + retrieval
  - websocket catch-up + live stream
- UI tests for sync service behavior and cursor persistence.
- Run relevant repo checks:
  - `cargo fmt --all --check`
  - `cargo clippy --workspace --all-targets --locked`
  - targeted tests first, then broader suite as needed

### Phase 8: Docker/Coolify + GHCR

- Add `crates/ez-booth-server/Dockerfile` (multi-stage).
- Add `.dockerignore` at repo root.
- Add `docker-compose.yml` tailored for Coolify deployment.
- Define required env vars with compose required syntax where applicable.
- Keep database internal-only in compose networking (no public port mapping by default).
- Add CI job to verify Docker image build.
- Update release workflow to publish multi-arch images to GHCR (`linux/amd64`, `linux/arm64`).
- Configure Coolify to pull GHCR image by tag (`latest` or pinned version).

## Out of Scope (Initial Milestone)

- User account system and per-user authorization
- CRDT implementation beyond append-only event snapshots
- TLS termination inside app process (handled by reverse proxy)
- Admin dashboard

## Risks and Mitigations

1. **Financial data consistency risk**
   - Mitigation: full snapshot payloads, append-only log, narrow regression tests around money/fees.

2. **Sync conflict edge cases**
   - Mitigation: deterministic server ordering (`sequence`), explicit replace semantics, reconciliation tests.

3. **Offline/online transition complexity**
   - Mitigation: cursor-based pull, idempotent event application, resilient reconnect backoff.

4. **Deployment drift (Coolify vs local)**
   - Mitigation: one compose source of truth and CI image verification.

## Documentation Follow-up

When implementation starts/lands, update:

- `README.md` (server mode + deployment path)
- `TESTING.md` (server integration and sync tests)
- relevant validation docs under `docs/validation/` if operator workflow changes
