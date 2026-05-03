# KP-072: ping_measured Telemetry Emitter with Schema Guard

**Epic:** EPIC-METRICS
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 3
**Priority:** Must

---

## Objective
Emit `ping_measured` telemetry events with strict schema-compliant payload fields.

## Technical Specifications
*   **Proposed Files:** `crates/client-engine/src/telemetry/events/ping_metrics.rs`
*   **Functions/Classes:** `emit_ping_measured_event()`
*   **API Endpoints:** N/A
*   **Data Models:** Ping telemetry payload model

## Acceptance Criteria (Technical)
*   [ ] Event payload includes baseline/routed ping, jitter, packet_loss only from allowlist.
*   [ ] Invalid fields are rejected before queueing.
*   [ ] Emit cadence follows measurement loop policy.
*   [ ] Queue backpressure does not block probe pipeline.

## Business Rules & Logic
*   Performance telemetry must remain privacy-safe and analyzable.

## Dependencies
*   Depends on: KP-069, KP-043

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
