# PRD Addendum - Smart Routing Engine
> Feature: Smart Routing Engine (WireGuard -> TCP/TLS -> QUIC)
> Date: 2026-05-03
> Status: In Progress

---

## 1. Feature Metadata

| Field | Value |
|---|---|
| Feature Name | Smart Routing Engine |
| Parent Epic | Product Features |
| Status | In Progress |
| Owner | Product Owner / Solo Developer |
| Related Global Docs | docs/prd.md, docs/fsd.md, docs/tech-stack.md, docs/api/contracts/relay-public.v1.yaml |

---

## 2. Requirements

| ID | User Story | Acceptance Criteria |
|---|---|---|
| US-RT-01 | As a user, I want route activation to prioritize the best protocol path so latency reduction is consistent. | Given routing is enabled, when protocol selection runs, then engine attempts WireGuard first, TCP/TLS second, QUIC third. |
| US-RT-02 | As a user, I want automatic fallback when route establishment fails so session continuity is preserved. | Given current protocol/relay fails, when failure is detected, then engine attempts next protocol/relay according to bounded retry policy. |
| US-RT-03 | As a maintainer, I want signed manifest verification before route use so tampered relay data cannot be applied. | Given manifest is fetched, when signature verification fails, then route activation is blocked and error state is raised. |
| US-RT-04 | As a maintainer, I want deterministic route scoring so behavior is predictable and debuggable. | Given relay candidates exist, when selection executes, then score output reflects health, region preference, and policy weights. |
| US-RT-05 | As a product owner, I want lifecycle state outputs for UI and telemetry so user feedback and analytics remain accurate. | Given routing lifecycle transitions, when states change, then engine emits normalized status and event payloads. |

---

## 3. ERD Delta

No new entities are introduced.

Data usage notes:
- Reuse `route_session`, `relay_node`, and `telemetry_event` structures.
- Runtime scoring/fallback internals may remain in-memory unless explicitly required for persistence.

---

## 4. API Contract Delta

No new public endpoint is introduced.

This feature consumes existing contracts:
- `docs/api/contracts/relay-public.v1.yaml` (`/v1/relay/manifest`, `/v1/relay/health`)

---

## 5. Integration Notes

- Routing order is fixed by ADR-03: WireGuard -> TCP/TLS -> QUIC.
- Engine must remain process-external and anti-cheat-safe.
- Telemetry/event outputs must align with existing telemetry contract event names and allowed payload keys.
- Failover behavior must be bounded (retry budget + backoff) to avoid unstable reconnect loops.

---

## 6. Open Questions

| ID | Question | Priority |
|---|---|---|
| RT-OQ-01 | Final protocol/relay score weighting constants for v1 beta? | Medium |
| RT-OQ-02 | Retry/backoff schedule differentiation per protocol required at v1? | Medium |
| RT-OQ-03 | Should partial session diagnostics be persisted for offline debugging in v1? | Low |
