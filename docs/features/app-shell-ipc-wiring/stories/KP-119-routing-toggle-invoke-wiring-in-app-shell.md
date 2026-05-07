# KP-119: Routing Toggle Invoke Wiring in App Shell

**Epic:** EPIC-WIRING
**Layer:** L4-feature-ui
**Role:** Frontend
**Estimation:** 2
**Priority:** Must

---

## Objective
Wire app-shell toggle actions to existing Rust IPC routing commands through frontend adapter.

## Technical Specifications
*   **Proposed Files:** `apps/desktop/src/App.tsx`, `apps/desktop/src/features/routing/ToggleController.tsx`
*   **Functions/Classes:** `onEnableRouting`, `onDisableRouting`
*   **API Endpoints:** N/A
*   **Data Models:** `RoutingLifecycleResponse`

## Acceptance Criteria (Technical)
*   [x] Clicking ON triggers `invokeRoutingToggleOn` with production-safe payload.
*   [x] Clicking OFF triggers `invokeRoutingToggleOff` with production-safe payload.
*   [x] Response mapping updates badge to `connecting` / `off` / `error` consistently.
*   [x] Failure fallback shows user-safe feedback without leaking backend internals.

## Business Rules & Logic
*   User action latency should stay responsive while command execution is in progress.

## Dependencies
*   Depends on: KP-118, KP-114

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
