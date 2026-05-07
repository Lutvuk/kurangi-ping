# KP-112: Rust-to-Frontend Event Emission Bridge

**Epic:** EPIC-WIRING
**Layer:** L3-backend
**Role:** Fullstack
**Estimation:** 3
**Priority:** Must

---

## Objective
Bridge Rust lifecycle outputs to Tauri emitted events for routing state, ping metrics, and detection updates.

## Technical Specifications
*   **Proposed Files:** `apps/desktop/src-tauri/src/ipc/event_bridge.rs`, `apps/desktop/src-tauri/src/main.rs`
*   **Functions/Classes:** `emit_routing_state_changed`, `emit_metrics_ping_sampled`, `emit_detection_status_updated`
*   **API Endpoints:** N/A
*   **Data Models:** Event payload contracts for routing, metrics, detection

## Acceptance Criteria (Technical)
*   [ ] Routing lifecycle transitions emit `routing_state_changed` events.
*   [ ] Metrics probe cycles emit `metrics_ping_sampled` events with normalized fields.
*   [ ] Detection status changes emit `detection_status_updated` events.
*   [ ] Event emission order remains deterministic across repeated runs.

## Business Rules & Logic
*   UI must observe backend state changes without polling-heavy fallback.

## Dependencies
*   Depends on: KP-109, KP-034, KP-041, KP-068

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
