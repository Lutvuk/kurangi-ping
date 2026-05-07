# KP-111: Detection Status Query Command and Result Mapping

**Epic:** EPIC-WIRING
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 2
**Priority:** Must

---

## Objective
Expose detection status query command from Tauri backend that maps Rust detection scanner output to stable UI response model.

## Technical Specifications
*   **Proposed Files:** `apps/desktop/src-tauri/src/ipc/detection_commands.rs`, `apps/desktop/src-tauri/src/main.rs`
*   **Functions/Classes:** `detection_get_status`
*   **API Endpoints:** N/A
*   **Data Models:** Detection status response contract

## Acceptance Criteria (Technical)
*   [ ] Command returns deterministic `detected` or `not_detected` state for UI.
*   [ ] Scanner and freshness resolver are invoked through existing client-engine boundaries.
*   [ ] Error mapping avoids sensitive process/system details.
*   [ ] Response contract aligns with `DetectionPanel` data needs.

## Business Rules & Logic
*   Startup detection status should be instantly actionable for non-technical users.

## Dependencies
*   Depends on: KP-109, KP-041, KP-042

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
