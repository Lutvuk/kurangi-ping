# KP-053: Relay Failure and Recovery Telemetry Emission

**Epic:** EPIC-RELAY
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 3
**Priority:** Must

---

## Objective
Emit relay failure and recovery telemetry signals with schema-compliant payloads.

## Technical Specifications
*   **Proposed Files:** `crates/client-engine/src/telemetry/events/relay_failover.rs`
*   **Functions/Classes:** `emit_relay_failed()`, `emit_relay_recovered()`
*   **API Endpoints:** N/A
*   **Data Models:** Relay failover event payload model

## Acceptance Criteria (Technical)
*   [ ] `relay_failed` emits on terminal and transition failures.
*   [ ] Recovery event emits on successful post-failover stabilization.
*   [ ] Payloads respect telemetry allowlist and avoid sensitive data.
*   [ ] Emission path tolerates queue backpressure/failure safely.

## Business Rules & Logic
*   Reliability analytics depend on high-quality failover event streams.

## Dependencies
*   Depends on: KP-051, KP-052, KP-043

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
