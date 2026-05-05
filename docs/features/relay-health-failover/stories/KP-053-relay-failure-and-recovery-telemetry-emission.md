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
*   [x] `relay_failed` emits on terminal and transition failures.
*   [x] Recovery event emits on successful post-failover stabilization.
*   [x] Payloads respect telemetry allowlist and avoid sensitive data.
*   [x] Emission path tolerates queue backpressure/failure safely.

## Business Rules & Logic
*   Reliability analytics depend on high-quality failover event streams.

## Dependencies
*   Depends on: KP-051, KP-052, KP-043

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
