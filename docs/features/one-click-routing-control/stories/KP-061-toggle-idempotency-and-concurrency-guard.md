# KP-061: Toggle Idempotency and Concurrency Guard

**Epic:** EPIC-TOGGLE
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 3
**Priority:** Must

---

## Objective
Prevent inconsistent lifecycle from rapid repeated ON/OFF commands.

## Technical Specifications
*   **Proposed Files:** `crates/client-engine/src/routing/command_guard.rs`
*   **Functions/Classes:** `acquire_toggle_lock()`, `dedupe_toggle_command()`
*   **API Endpoints:** N/A
*   **Data Models:** Command guard state model

## Acceptance Criteria (Technical)
*   [ ] Concurrent commands are serialized or rejected by policy.
*   [ ] Duplicate ON/OFF commands are idempotent.
*   [ ] Guard release is guaranteed on success/failure paths.
*   [ ] Race-condition test cases are covered.

## Business Rules & Logic
*   Command safety is required for non-technical users.

## Dependencies
*   Depends on: KP-058

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
