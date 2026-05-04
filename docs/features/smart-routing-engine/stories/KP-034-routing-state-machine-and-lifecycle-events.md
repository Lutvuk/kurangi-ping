# KP-034: Routing State Machine and Lifecycle Events

**Epic:** EPIC-ROUTING
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 4
**Priority:** Must

---

## Objective
Implement normalized routing state machine and lifecycle event emissions for UI and telemetry integration.

## Technical Specifications
*   **Proposed Files:** `crates/client-engine/src/routing/state_machine.rs`, `crates/client-engine/src/telemetry/events/routing.rs`
*   **Functions/Classes:** `RoutingStateMachine`, `emit_routing_event()`
*   **API Endpoints:** N/A
*   **Data Models:** Routing state/event payload model

## Acceptance Criteria (Technical)
*   [x] States include at minimum: off, connecting, connected, degraded, failed.
*   [x] Transitions are validated and illegal transitions rejected.
*   [x] `routing_enabled`, `routing_disabled`, and `relay_failed` event mappings are consistent.
*   [x] State outputs are serializable for UI consumption.

## Business Rules & Logic
*   User-facing state must match actual engine lifecycle.

## Dependencies
*   Depends on: KP-033

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
