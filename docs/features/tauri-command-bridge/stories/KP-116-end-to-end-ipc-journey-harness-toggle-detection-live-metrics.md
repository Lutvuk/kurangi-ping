# KP-116: End-to-End IPC Journey Harness (toggle, detection, live metrics)

**Epic:** EPIC-WIRING
**Layer:** L5-integration
**Role:** Fullstack
**Estimation:** 3
**Priority:** Should

---

## Objective
Simulate full IPC flow from frontend command invocation to backend event delivery for routing toggle, detection status, and live metrics updates.

## Technical Specifications
*   **Proposed Files:** `crates/client-engine/tests/ipc_e2e_journey_test.rs`, `apps/desktop/src/features/routing/ToggleController.test.tsx`
*   **Functions/Classes:** `run_ipc_journey_suite()`
*   **API Endpoints:** N/A
*   **Data Models:** IPC journey scenario fixtures

## Acceptance Criteria (Technical)
*   [x] Success path validates ON toggle command to active state event.
*   [x] Detection startup query path validates frontend-visible status contract.
*   [x] Metrics event path validates continuous UI updates.
*   [x] Journey traces are deterministic across repeated runs.

## Business Rules & Logic
*   IPC wiring quality must be validated before production hardening.

## Dependencies
*   Depends on: KP-110, KP-111, KP-112, KP-115

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
