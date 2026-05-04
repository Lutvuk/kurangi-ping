# KP-043: Detection Event Emission (game_detected) with Schema Guard

**Epic:** EPIC-DETECTION
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 3
**Priority:** Must

---

## Objective
Emit `game_detected` telemetry event with strict payload allowlist and schema checks.

## Technical Specifications
*   **Proposed Files:** `crates/client-engine/src/telemetry/events/detection.rs`
*   **Functions/Classes:** `emit_game_detected_event()`
*   **API Endpoints:** N/A
*   **Data Models:** Telemetry detection event payload model

## Acceptance Criteria (Technical)
*   [x] Event emits only on successful detection transitions.
*   [x] Payload includes only approved keys (`game_id`, `process_name`, `detection_time_ms`).
*   [x] Non-allowlisted fields are rejected before enqueue.
*   [x] Event path integrates with existing telemetry queue.

## Business Rules & Logic
*   Telemetry must preserve privacy and schema compliance.

## Dependencies
*   Depends on: KP-041, KP-034

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
