# Relay Controller Module

This module hosts the Go relay controller service.

Scope for scaffold phase:
- Provide service skeleton only.
- Reserve API wiring for upcoming stories.

## KP-006 Baseline

- Entry point: `cmd/server/main.go`
- Health handler: `internal/http/health.go`
- Endpoint: `GET /v1/relay/health`
