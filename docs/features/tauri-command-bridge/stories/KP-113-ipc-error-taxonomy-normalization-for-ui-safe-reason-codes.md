# KP-113: IPC Error Taxonomy Normalization for UI-safe Reason Codes

**Epic:** EPIC-WIRING
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 2
**Priority:** Should

---

## Objective
Normalize IPC command and event failure cases into stable reason codes that frontend can render consistently.

## Technical Specifications
*   **Proposed Files:** `apps/desktop/src-tauri/src/ipc/error_map.rs`, `apps/desktop/src-tauri/src/ipc/mod.rs`
*   **Functions/Classes:** `map_ipc_error_reason`
*   **API Endpoints:** N/A
*   **Data Models:** IPC reason code enum

## Acceptance Criteria (Technical)
*   [ ] Routing/detection/metrics IPC failures map to bounded reason code set.
*   [ ] Unknown failures map to safe fallback reason code.
*   [ ] Reason-code format is compatible with existing UI error presenters.
*   [ ] Unit tests cover deterministic mapping behavior.

## Business Rules & Logic
*   Diagnostic errors should be actionable without exposing sensitive internals.

## Dependencies
*   Depends on: KP-110, KP-111, KP-112, KP-101

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
