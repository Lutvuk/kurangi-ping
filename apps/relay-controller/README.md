# Relay Controller Module

This module hosts the Go relay controller service.

Scope for scaffold phase:
- Provide service skeleton only.
- Reserve API wiring for upcoming stories.

## KP-006 Baseline

- Entry point: `cmd/server/main.go`
- Health handler: `internal/http/health.go`
- Endpoint: `GET /v1/relay/health`

## DB Bootstrap Stub (KP-016)

- DB package: `internal/db`
- Env vars:
  - `KP_RELAY_DB_PATH`: sqlite file path (empty = DB bootstrap skipped)
  - `KP_RELAY_MIGRATIONS_PATH`: optional path for `*.up.sql` migrations
- Startup behavior:
  - If `KP_RELAY_DB_PATH` is set, service runs `OpenDB()` then `Migrate()`.
  - Migration/bootstrap failures are logged and fail fast at startup.
