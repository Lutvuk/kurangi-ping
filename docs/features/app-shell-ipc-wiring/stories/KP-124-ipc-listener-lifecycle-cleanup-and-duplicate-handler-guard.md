# KP-124: IPC Listener Lifecycle Cleanup and Duplicate-Handler Guard

**Epic:** EPIC-WIRING
**Layer:** L4-feature-ui
**Role:** Frontend
**Estimation:** 2
**Priority:** Must

---

## Objective
Harden listener lifecycle management agar tidak ada duplicate handler, memory leak, atau event fan-out tak terduga ketika shell re-render/remount.

## Technical Specifications
*   **Proposed Files:** `apps/desktop/src/features/shell/useAppShellIpcState.ts`, `apps/desktop/src/lib/ipc/client.ts`
*   **Functions/Classes:** `attachShellListeners`, `detachShellListeners`, `listenerRegistryGuard`
*   **API Endpoints:** N/A
*   **Data Models:** Listener registry metadata

## Acceptance Criteria (Technical)
*   [x] Semua listener (`routing_state_changed`, `detection_status_updated`, `metrics_ping_sampled`) punya cleanup deterministic.
*   [x] Re-mount tidak menghasilkan handler duplikat.
*   [x] Listener attach failure tidak crash app shell.
*   [x] Unit test membuktikan subscribe/unsubscribe cycle stabil pada eksekusi berulang.

## Business Rules & Logic
*   Reliability app shell harus tetap stabil selama sesi panjang tanpa degradasi perilaku UI.

## Dependencies
*   Depends on: KP-120, KP-122, KP-123

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
