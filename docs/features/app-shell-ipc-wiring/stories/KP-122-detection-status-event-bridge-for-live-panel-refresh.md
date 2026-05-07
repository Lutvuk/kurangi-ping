# KP-122: Detection Status Event Bridge for Live Panel Refresh

**Epic:** EPIC-WIRING
**Layer:** L4-feature-ui
**Role:** Frontend
**Estimation:** 2
**Priority:** Should

---

## Objective
Subscribe event `detection_status_updated` agar perubahan detection dari engine bisa langsung memutakhirkan panel tanpa invoke ulang manual.

## Technical Specifications
*   **Proposed Files:** `apps/desktop/src/features/shell/useAppShellIpcState.ts`, `apps/desktop/src/features/detection/DetectionPanel.tsx`
*   **Functions/Classes:** `bindDetectionStatusListener`, `applyDetectionStatusEvent`
*   **API Endpoints:** N/A
*   **Data Models:** `DetectionStatusEventPayload`

## Acceptance Criteria (Technical)
*   [ ] App shell attach listener `detection_status_updated` dan menerapkan payload ke state detection.
*   [ ] UI panel update otomatis saat status berubah (detected -> not_detected atau sebaliknya).
*   [ ] Event payload invalid ditangani graceful tanpa merusak state existing.
*   [ ] Listener cleanup benar saat unmount.

## Business Rules & Logic
*   Detection state harus responsif terhadap perubahan runtime (mis. game ditutup/dibuka) tanpa refresh aplikasi.

## Dependencies
*   Depends on: KP-118, KP-121, KP-112

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
