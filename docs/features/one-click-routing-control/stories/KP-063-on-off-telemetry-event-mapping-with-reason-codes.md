# KP-063: ON/OFF Telemetry Event Mapping with Reason Codes

**Epic:** EPIC-TOGGLE
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 3
**Priority:** Must

---

## Objective
Emit routing activation/deactivation telemetry with normalized reason codes and schema-safe payloads.

## Technical Specifications
*   **Proposed Files:** `crates/client-engine/src/telemetry/events/toggle_lifecycle.rs`
*   **Functions/Classes:** `emit_routing_enabled()`, `emit_routing_disabled()`
*   **API Endpoints:** N/A
*   **Data Models:** ON/OFF telemetry payload model

## Acceptance Criteria (Technical)
*   [ ] `routing_enabled` emitted on successful ON completion.
*   [ ] `routing_disabled` emitted on successful OFF completion.
*   [ ] Failure cases include reason codes without sensitive fields.
*   [ ] Payloads are validated against telemetry allowlist.

## Business Rules & Logic
*   Lifecycle analytics depend on clean, reason-aware events.

## Dependencies
*   Depends on: KP-059, KP-060, KP-043

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
