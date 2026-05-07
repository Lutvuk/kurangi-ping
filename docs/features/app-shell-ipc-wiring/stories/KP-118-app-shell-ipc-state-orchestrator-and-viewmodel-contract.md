# KP-118: App Shell IPC State Orchestrator and ViewModel Contract

**Epic:** EPIC-WIRING
**Layer:** L4-feature-ui
**Role:** Frontend
**Estimation:** 3
**Priority:** Must

---

## Objective
Define app-shell level state orchestration contract so routing, detection, and metrics panels can share one IPC-driven source of truth.

## Technical Specifications
*   **Proposed Files:** `apps/desktop/src/App.tsx`, `apps/desktop/src/features/shell/useAppShellIpcState.ts`
*   **Functions/Classes:** `useAppShellIpcState`
*   **API Endpoints:** N/A
*   **Data Models:** `AppShellIpcViewModel`

## Acceptance Criteria (Technical)
*   [x] App shell maintains deterministic state model for routing/detection/metrics.
*   [x] Initial state defaults are compatible with existing component contracts.
*   [x] State transitions can be driven by both invoke responses and event updates.
*   [x] No raw IPC command/event strings appear in `App.tsx`.

## Business Rules & Logic
*   App shell should be maintainable as single composition point for production-mode IPC behavior.

## Dependencies
*   Depends on: KP-114, KP-115

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
