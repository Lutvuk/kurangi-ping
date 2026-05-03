# PRD Addendum - Real-Time Ping Metrics Dashboard
> Feature: Real-Time Ping Metrics Dashboard
> Date: 2026-05-04
> Status: In Progress

---

## 1. Feature Metadata

| Field | Value |
|---|---|
| Feature Name | Real-Time Ping Metrics Dashboard |
| Parent Epic | Product Features |
| Status | In Progress |
| Owner | Product Owner / Solo Developer |
| Related Global Docs | docs/prd.md, docs/fsd.md, docs/design-system.md, docs/erd/core-erd.md |

---

## 2. Requirements

| ID | User Story | Acceptance Criteria |
|---|---|---|
| US-PM-01 | As a user, I want baseline and routed ping shown in near-real-time so I can see optimization impact immediately. | Given probe loop runs, when samples arrive, then baseline and routed values update in UI within target refresh interval. |
| US-PM-02 | As a user, I want jitter and packet loss shown with freshness status so I can interpret connection quality accurately. | Given probe samples are calculated, when values update, then jitter/loss and freshness indicators are rendered consistently. |
| US-PM-03 | As a user, I want dashboard behavior that remains readable during degraded measurement conditions. | Given probe failures/timeouts occur, when metrics cannot refresh, then dashboard enters degraded state with last-known values and warning status. |
| US-PM-04 | As a maintainer, I want short metric history retained so trend views and diagnostics can be supported. | Given sampling runs over time, when ring-buffer limit is reached, then oldest entries are evicted and recent data remains queryable. |
| US-PM-05 | As a product owner, I want `ping_measured` telemetry signals so performance KPIs can be tracked longitudinally. | Given measurement cycles complete, when telemetry emits, then payload follows allowed schema and excludes sensitive network payload data. |

---

## 3. ERD Delta

No new entities are required.

Data usage notes:
- Reuse `ping_sample` for metrics persistence.
- Reuse `route_session` linkage for contextual measurement grouping.

---

## 4. API Contract Delta

No new endpoint is introduced.

Dashboard relies on local measurement pipeline and existing telemetry event contract.

---

## 5. Integration Notes

- Probe loop must align with routing ON/OFF lifecycle hooks.
- UI rendering must follow design-system metric card rules (mono typography, semantic colors).
- Probe failures should degrade gracefully without blocking app interaction.
- Metric event emission must remain privacy-safe and schema-compliant.

---

## 6. Open Questions

| ID | Question | Priority |
|---|---|---|
| PM-OQ-01 | Final probe interval default for balancing responsiveness and overhead? | Medium |
| PM-OQ-02 | Should trend sparkline be included in v1 dashboard or deferred? | Low |
| PM-OQ-03 | Minimum sample count before reduction percentage is considered stable? | Medium |
