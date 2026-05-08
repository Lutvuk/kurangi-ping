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

## Relay Probe Env Contract (KP-127)

Live relay probing configuration is environment-driven.

- `KP_RELAY_PROBE_TARGETS`
  - Required for live probe mode.
  - Format: comma-separated `<relay_id>|<probe_url>` entries.
  - Example: `sin-01|https://sin-01.example.net/healthz,nrt-01|https://nrt-01.example.net/healthz`
- `KP_RELAY_PROBE_TIMEOUT_MS`
  - Per-target probe timeout in milliseconds.
  - Safe default recommendation: `1200`.
  - Invalid/empty values should fall back to controller default in runtime loader.

This contract is additive and does not rename/remove existing relay controller env keys.
