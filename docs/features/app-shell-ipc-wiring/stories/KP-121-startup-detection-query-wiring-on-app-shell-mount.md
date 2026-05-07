# KP-121: Startup Detection Query Wiring on App Shell Mount

**Epic:** EPIC-WIRING
**Layer:** L4-feature-ui
**Role:** Frontend
**Estimation:** 2
**Priority:** Must

---

## Objective
Fetch detection status via IPC saat app shell selesai mount agar `DetectionPanel` langsung tampil `detected/not_detected` sesuai kondisi aktual.

## Technical Specifications
*   **Proposed Files:** `apps/desktop/src/App.tsx`, `apps/desktop/src/features/shell/useAppShellIpcState.ts`
*   **Functions/Classes:** `requestInitialDetectionStatus`, `mapDetectionStatusResponse`
*   **API Endpoints:** N/A
*   **Data Models:** `DetectionStatusResponse`, `DetectionPanelViewModel`

## Acceptance Criteria (Technical)
*   [x] App shell memanggil `invoke("detection_get_status")` sekali pada startup lifecycle.
*   [x] Response sukses di-map ke state UI `detected/not_detected` yang kompatibel dengan `DetectionPanel`.
*   [x] Response failure di-map ke error state UI-safe tanpa raw internal error.
*   [x] Startup query bersifat idempotent untuk remount/hot-reload.

## Business Rules & Logic
*   User harus mendapat status detection secepat mungkin tanpa menunggu event tambahan.

## Dependencies
*   Depends on: KP-118, KP-114, KP-111

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
