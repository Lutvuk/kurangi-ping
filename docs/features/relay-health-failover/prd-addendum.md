# PRD Addendum - Relay Health & Failover Handling
> Feature: Relay Health & Failover Handling
> Date: 2026-05-04
> Status: In Progress

---

## 1. Feature Metadata

| Field | Value |
|---|---|
| Feature Name | Relay Health & Failover Handling |
| Parent Epic | Product Features |
| Status | In Progress |
| Owner | Product Owner / Solo Developer |
| Related Global Docs | docs/prd.md, docs/fsd.md, docs/tech-stack.md, docs/api/contracts/relay-public.v1.yaml |

---

## 2. Requirements

| ID | User Story | Acceptance Criteria |
|---|---|---|
| US-RH-01 | As a user, I want relay health visibility so I can trust routing quality before and during sessions. | Given health checks run, when relay status updates, then each relay is classified into `ok`, `warn`, or `dead` with latency metadata. |
| US-RH-02 | As a user, I want automatic failover from unhealthy relays so my session can continue with minimal interruption. | Given active relay degrades/fails, when failover policy triggers, then engine selects next suitable relay and attempts reconnection within retry budget. |
| US-RH-03 | As a maintainer, I want anti-flapping guardrails so relay switching does not oscillate excessively. | Given health values fluctuate around thresholds, when failover logic evaluates, then grace window and hysteresis prevent rapid toggle loops. |
| US-RH-04 | As a user, I want understandable failover states in UI so I know what the app is doing during instability. | Given failover lifecycle transitions occur, when state is emitted, then UI receives normalized reason/state metadata. |
| US-RH-05 | As a product owner, I want failover telemetry signals so reliability and node quality trends can be monitored. | Given failover events occur, when telemetry emits, then `relay_failed` and recovery events follow schema allowlist and retry context rules. |

---

## 3. ERD Delta

No new entities are required.

Data usage notes:
- Reuse `relay_node`, `route_session`, and `telemetry_event` for relay health/failover observability.

---

## 4. API Contract Delta

No new endpoint is introduced.

Feature consumes existing relay health contract:
- `GET /v1/relay/health` from `docs/api/contracts/relay-public.v1.yaml`

---

## 5. Integration Notes

- Must integrate with Smart Routing orchestrator and retry/backoff controller.
- Failover decisions must remain bounded and deterministic.
- UI integration targets existing status badge and relay health list components.
- Event emission must remain telemetry schema-compliant.

---

## 6. Open Questions

| ID | Question | Priority |
|---|---|---|
| RH-OQ-01 | Final threshold boundaries for `ok/warn/dead` classification in beta? | Medium |
| RH-OQ-02 | Recovery event naming: separate event vs attribute on routing lifecycle events? | Medium |
| RH-OQ-03 | Should relay score history be persisted in v1 or computed transiently only? | Low |
