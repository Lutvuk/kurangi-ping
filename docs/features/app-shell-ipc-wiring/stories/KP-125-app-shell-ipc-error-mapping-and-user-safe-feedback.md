# KP-125: App Shell IPC Error Mapping and User-Safe Feedback

**Epic:** EPIC-WIRING
**Layer:** L4-feature-ui
**Role:** Frontend
**Estimation:** 2
**Priority:** Should

---

## Objective
Normalize invoke/listen failures di app shell ke reason code UI-safe supaya feedback konsisten dan tidak membocorkan detail internal backend.

## Technical Specifications
*   **Proposed Files:** `apps/desktop/src/features/shell/useAppShellIpcState.ts`, `apps/desktop/src/lib/errors/ipcErrorMapper.ts`, `apps/desktop/src/App.tsx`
*   **Functions/Classes:** `mapIpcErrorToUiReason`, `presentIpcFailureState`
*   **API Endpoints:** N/A
*   **Data Models:** `UiSafeIpcError`, `ShellErrorBannerState`

## Acceptance Criteria (Technical)
*   [x] Error invoke ON/OFF dan detection query dipetakan ke reason code `ipc_*` yang disetujui.
*   [x] Error listener attach dipresentasikan lewat state UI aman (tanpa stack trace/raw command internals).
*   [x] UX fallback tetap memungkinkan user lanjut menggunakan mode aman bila tersedia.
*   [x] Mapping reuse taxonomy dari kontrak backend IPC.

## Business Rules & Logic
*   Kegagalan IPC harus bisa dipahami user tanpa membuka informasi teknis sensitif.

## Dependencies
*   Depends on: KP-119, KP-120, KP-121, KP-113

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
