# KP-115: Feature Wiring for ToggleController, DetectionPanel, and PingMetricsPanel via IPC

**Epic:** EPIC-WIRING
**Layer:** L4-feature-ui
**Role:** Frontend
**Estimation:** 4
**Priority:** Must

---

## Objective
Wire core feature panels to IPC adapter so routing toggle, detection status, and ping metrics are driven by Rust backend events.

## Technical Specifications
*   **Proposed Files:** `apps/desktop/src/features/routing/ToggleController.tsx`, `apps/desktop/src/features/detection/DetectionPanel.tsx`, `apps/desktop/src/features/metrics/PingMetricsPanel.tsx`
*   **Functions/Classes:** `useRoutingIpcBridge`, `useDetectionIpcBridge`, `useMetricsIpcBridge`
*   **API Endpoints:** N/A
*   **Data Models:** UI view models fed by IPC stream

## Acceptance Criteria (Technical)
*   [x] ToggleController uses IPC invoke for ON/OFF transitions.
*   [x] DetectionPanel fetches startup status via IPC and reflects updates from events.
*   [x] PingMetricsPanel updates live from metrics event stream without page reload.
*   [x] Subscription lifecycle is cleaned up on unmount to prevent duplicate event handling.

## Business Rules & Logic
*   Core user flows must remain responsive and deterministic during long sessions.

## Dependencies
*   Depends on: KP-114, KP-064, KP-045, KP-074

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
