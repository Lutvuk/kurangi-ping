# KP-101: Updater Lifecycle State Model and Error Taxonomy

**Epic:** EPIC-UPDATER
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 3
**Priority:** Must

---

## Objective
Define normalized updater lifecycle states and reason taxonomy for UI + telemetry mapping.

## Technical Specifications
*   **Proposed Files:** `crates/client-engine/src/updater/state_model.rs`
*   **Functions/Classes:** `resolve_updater_state()`, `map_updater_error_reason()`
*   **API Endpoints:** N/A
*   **Data Models:** Updater state enum + reason code model

## Acceptance Criteria (Technical)
*   [ ] States include `up_to_date`, `update_available`, `downloading`, `ready_to_restart`, `update_error`.
*   [ ] Reason codes are deterministic and stable.
*   [ ] Unknown failures map to safe fallback category.
*   [ ] State transitions are validated by tests.

## Business Rules & Logic
*   Clear state semantics reduce user confusion and support overhead.

## Dependencies
*   Depends on: KP-100

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
