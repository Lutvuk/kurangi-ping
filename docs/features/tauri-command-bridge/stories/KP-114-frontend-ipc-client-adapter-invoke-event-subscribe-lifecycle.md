# KP-114: Frontend IPC Client Adapter (invoke + event subscribe lifecycle)

**Epic:** EPIC-WIRING
**Layer:** L4-feature-ui
**Role:** Frontend
**Estimation:** 3
**Priority:** Must

---

## Objective
Create frontend IPC adapter that encapsulates Tauri invoke/listen APIs for command calls and event subscriptions.

## Technical Specifications
*   **Proposed Files:** `apps/desktop/src/lib/ipc/client.ts`, `apps/desktop/src/lib/ipc/index.ts`
*   **Functions/Classes:** `invokeRoutingToggleOn`, `invokeRoutingToggleOff`, `invokeDetectionGetStatus`, `subscribeRoutingState`, `subscribePingMetrics`, `subscribeDetectionStatus`
*   **API Endpoints:** N/A
*   **Data Models:** Frontend IPC adapter request/response types

## Acceptance Criteria (Technical)
*   [x] Adapter exposes typed invoke wrappers for routing and detection commands.
*   [x] Adapter exposes subscribe/unsubscribe wrappers for bridge events.
*   [x] Adapter provides deterministic cleanup lifecycle for event listeners.
*   [x] Frontend code no longer directly hardcodes raw IPC command strings.

## Business Rules & Logic
*   IPC complexity should stay hidden from feature components.

## Dependencies
*   Depends on: KP-109, KP-112

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
