# PRD Addendum - Live Relay Health Probe
> Feature: Live Relay Health Probe
> Date: 2026-05-08
> Status: Proposed

---

## 1. Feature Metadata

| Field | Value |
|---|---|
| Feature Name | Live Relay Health Probe |
| Parent Epic | EPIC-RELAY |
| Status | Proposed |
| Owner | Product Owner / Solo Developer |
| Related Global Docs | docs/prd.md, docs/tech-stack.md, docs/api/contracts/relay-public.v1.yaml |

---

## 2. Requirements

| ID | User Story | Acceptance Criteria |
|---|---|---|
| US-LRHP-01 | As a user, I want relay latency to come from real probes so relay status reflects real network conditions. | Given relay controller is running, when health endpoint is called, then controller probes real relay targets and returns actual measured latency (not hardcoded). |
| US-LRHP-02 | As a user, I want offline relay detection so unhealthy nodes are avoided quickly. | Given a relay does not respond within timeout, when probe execution finishes, then relay is marked `dead/offline` and surfaced in health response for client consumption. |
| US-LRHP-03 | As a client engine, I want fresh relay health snapshots so failover uses current conditions. | Given probe cycle has completed, when Rust engine requests `/v1/relay/health`, then response uses latest probe snapshot (with updated timestamp), not static defaults. |
| US-LRHP-04 | As a maintainer, I want relay targets managed via environment config so infra changes do not require code edits. | Given relay target list changes, when `.env` is updated, then controller reads new targets without hardcoded relay endpoints in source. |
| US-LRHP-05 | As an integrator, I want contract compatibility preserved so existing Rust client parser does not break. | Given feature is enabled, when `/v1/relay/health` responds, then JSON shape remains exactly compatible with current `data[]` and `page` contract consumed by client engine. |

---

## 3. ERD Delta

No database schema change required.

---

## 4. API Contract Delta

No new endpoint introduced.

Existing endpoint behavior is upgraded:
- `GET /v1/relay/health`
  - Response schema: unchanged
  - Data source: dynamic probe results (replacing hardcoded values)

---

## 5. Integration Notes

- Primary implementation target: `apps/relay-controller/internal/http/health.go`.
- Keep response envelope and field names unchanged (`relay_id`, `status`, `latency_ms`, `updated_at`, `page`).
- Relay targets must come from environment configuration (source of truth: `.env`/`.env.example` contract), not literals in handler.
- Timeout/failure probing should map to existing status semantics used by Rust side (`ok`, `warn`, `dead`).
- Probe execution must be bounded (timeout + non-blocking behavior) to avoid degrading API latency.

---

## 6. Open Questions

| ID | Question | Priority |
|---|---|---|
| LRHP-OQ-01 | Final env key format for multi-target relay list (CSV vs JSON) before implementation? | Medium |
| LRHP-OQ-02 | Primary probe transport for v1 execution path (HTTP, ICMP, or hybrid fallback) on Fly free-tier constraints? | Medium |
| LRHP-OQ-03 | Should `/v1/relay/health` trigger probe on-demand or serve from short-lived in-memory snapshot with background refresh? | High |
