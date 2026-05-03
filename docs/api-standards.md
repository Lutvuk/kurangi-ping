# Core API Standards - Kurangi Ping
> Version 1.0 · Date: 2026-05-03 · Status: Approved Baseline

---

## 1. Overview

This document defines mandatory standards for Kurangi Ping v1 APIs used by relay metadata, relay control, and telemetry ingestion services.

Core principles:
- Privacy-first by default.
- Backward-compatible contracts.
- Fail-safe behavior for unstable networks.
- Strictly no user-identifying telemetry fields.

---

## 2. Versioning

- API versioning SHALL be path-based: `/v1/...`.
- Breaking changes SHALL be published under a new major path (example: `/v2/...`).
- Non-breaking additions (optional fields/endpoints) MAY remain in the same major version.
- Deprecated endpoints SHOULD include sunset notice in response headers.

---

## 3. Transport, Content, and Security

- HTTPS is mandatory (TLS 1.2+; TLS 1.3 preferred).
- Content-Type for request/response SHALL be `application/json` unless explicitly documented.
- Maximum request body size SHALL be enforced per endpoint class.
- Services SHOULD set security headers where applicable:
  - `Strict-Transport-Security`
  - `X-Content-Type-Options: nosniff`
  - `Cache-Control` policy per endpoint

---

## 4. Authentication and Authorization

v1 client model:
- End users do not authenticate with user accounts.
- Relay manifest trust is enforced via signed manifest verification on client side.

Service protections:
- Public read endpoints MAY be unauthenticated (manifest fetch, health summary).
- Administrative/mutating endpoints SHALL require service credentials (API key or equivalent).
- Credential rotation policy SHALL be documented and tested before launch.

---

## 5. Idempotency and Retry

- Mutating endpoints (`POST`/`PUT`/`PATCH`/`DELETE`) SHALL support `Idempotency-Key` where duplicate submission risk exists.
- Clients SHALL apply bounded exponential backoff for retryable failures.
- Retry eligibility matrix:
  - Retryable: `408`, `429`, `500`, `502`, `503`, `504`.
  - Non-retryable by default: `400`, `401`, `403`, `404`, `409`, `422`.

---

## 6. Error Standard

All non-2xx responses SHALL follow this schema:

```json
{
  "error": {
    "code": "STRING_CODE",
    "message": "Human-readable explanation",
    "retryable": true,
    "request_id": "uuid"
  }
}
```

### Minimum Error Code Catalog

| Code | HTTP | Retryable | Meaning |
|---|---:|---|---|
| INVALID_REQUEST | 400 | false | Input validation failed |
| UNAUTHORIZED | 401 | false | Missing/invalid credentials |
| FORBIDDEN | 403 | false | Authenticated but not allowed |
| NOT_FOUND | 404 | false | Resource not found |
| CONFLICT | 409 | false | Version/resource conflict |
| RATE_LIMITED | 429 | true | Request throttled |
| RELAY_UNAVAILABLE | 503 | true | No healthy relay available |
| MANIFEST_INVALID_SIGNATURE | 422 | false | Manifest signature invalid |
| TELEMETRY_SCHEMA_VIOLATION | 422 | false | Event payload outside allowlist |
| INTERNAL_ERROR | 500 | true | Server-side failure |

---

## 7. Pagination, Filtering, Sorting

For list endpoints:
- Cursor pagination SHALL be default.
- Query parameters:
  - `limit` (default 50, max 200)
  - `cursor` (opaque string)
  - `sort` (field allowlist)
  - `order` (`asc` or `desc`)

Standard list response envelope:

```json
{
  "data": [],
  "page": {
    "next_cursor": "opaque-or-null",
    "limit": 50
  }
}
```

---

## 8. Rate Limiting

Default policies (subject to capacity tuning):
- Manifest/health read endpoints: 120 req/min per IP.
- Telemetry ingest endpoint: 60 batch req/min per installation_id hash.
- Admin/control endpoints: 30 req/min per credential.

`429` responses SHALL include `Retry-After` header.

---

## 9. Telemetry Ingestion Standard

### 9.1 Endpoint Contract (baseline)

- Endpoint: `POST /v1/telemetry/events:batch`
- Payload: array of allowlisted events
- Response: accept/partial reject summary

### 9.2 Privacy Rules (mandatory)

- Prohibited fields: username, email, phone, IP address, packet payload, raw destination host list, full process command line.
- Event payload SHALL be validated against strict schema allowlist.
- Unknown fields SHALL be rejected or dropped by policy.

### 9.3 Network Logging Boundary

- Edge/proxy SHALL strip `X-Forwarded-For` before app-layer persistence.
- Raw source IP SHALL NOT be persisted in application storage.

---

## 10. Observability and Audit

- Each request SHOULD carry/return `X-Request-Id` for correlation.
- Structured logs SHALL avoid PII and payload-sensitive content.
- Minimum audit events:
  - Manifest published/rotated.
  - Relay node status changed.
  - Admin credential rotated/revoked.

---

## 11. Contract Examples

### 11.1 Manifest Fetch

Request:
```http
GET /v1/relay/manifest
Accept: application/json
```

Response (200):
```json
{
  "version": "2026.08.0",
  "valid_until": "2026-08-31T23:59:59Z",
  "signature": "base64-signature",
  "relays": [
    { "relay_id": "sin-01", "region": "sin", "hostname": "sin-01.example.net", "priority": 1 },
    { "relay_id": "nrt-01", "region": "nrt", "hostname": "nrt-01.example.net", "priority": 2 }
  ]
}
```

### 11.2 Relay Health Summary

Request:
```http
GET /v1/relay/health
Accept: application/json
```

Response (200):
```json
{
  "data": [
    { "relay_id": "sin-01", "status": "ok", "latency_ms": 38, "updated_at": "2026-05-03T09:00:00Z" },
    { "relay_id": "nrt-01", "status": "warn", "latency_ms": 121, "updated_at": "2026-05-03T09:00:00Z" }
  ]
}
```

### 11.3 Telemetry Batch Ingest

Request:
```http
POST /v1/telemetry/events:batch
Content-Type: application/json
Idempotency-Key: 2de4f145-2d4c-4b11-99d8-4b7d5ff0a35d
```

```json
{
  "installation_id_hash": "sha256:...",
  "events": [
    {
      "event_name": "routing_enabled",
      "occurred_at": "2026-05-03T09:01:00Z",
      "payload": { "game_id": "ffxiv", "relay_id": "sin-01", "baseline_ping_ms": 228 }
    }
  ]
}
```

Response (202):
```json
{
  "accepted": 1,
  "rejected": 0,
  "request_id": "uuid"
}
```

---

## 12. Change Management

Before any API contract change is released:
1. Update OpenAPI contract files.
2. Run compatibility review against current client behavior.
3. Validate privacy constraints still hold.
4. Add migration notes for breaking changes.
5. Update PRD/FSD references if behavior changes materially.
