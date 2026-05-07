# KP-109: Tauri IPC Contract Types and Command/Event Registry

**Epic:** EPIC-WIRING
**Layer:** L3-backend
**Role:** Fullstack
**Estimation:** 2
**Priority:** Must

---

## Objective
Define a centralized IPC contract registry for command names, event names, and payload typing boundaries shared by frontend and Tauri backend.

## Technical Specifications
*   **Proposed Files:** `apps/desktop/src/lib/ipc/contracts.ts`, `apps/desktop/src-tauri/src/ipc/contracts.rs`
*   **Functions/Classes:** `IpcCommand`, `IpcEvent`, `RoutingToggleOnRequest`, `DetectionStatusResponse`
*   **API Endpoints:** N/A
*   **Data Models:** IPC command/event contract schema

## Acceptance Criteria (Technical)
*   [x] All IPC command names for routing toggle and detection status are centralized.
*   [x] Event names for routing, metrics, and detection updates are centralized.
*   [x] Contract naming is deterministic and reused by both backend and frontend adapter.
*   [x] Contract docs include payload field expectations and non-sensitive error boundaries.

## Business Rules & Logic
*   IPC naming drift is not allowed because it can silently break user-critical controls.

## Dependencies
*   Depends on: KP-003, KP-005

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
