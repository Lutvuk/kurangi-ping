# KP-123: Ping Metrics Event Stream Binding in App Shell

**Epic:** EPIC-WIRING
**Layer:** L4-feature-ui
**Role:** Frontend
**Estimation:** 2
**Priority:** Should

---

## Objective
Bind `metrics_ping_sampled` stream ke app shell state supaya `PingMetricsPanel` menerima data live dari Rust tanpa reload.

## Technical Specifications
*   **Proposed Files:** `apps/desktop/src/features/shell/useAppShellIpcState.ts`, `apps/desktop/src/features/ping/PingMetricsPanel.tsx`
*   **Functions/Classes:** `bindMetricsListener`, `applyPingSampleEvent`
*   **API Endpoints:** N/A
*   **Data Models:** `PingSampleEventPayload`, `PingMetricsViewModel`

## Acceptance Criteria (Technical)
*   [x] Listener `metrics_ping_sampled` aktif saat shell mount.
*   [x] Nilai metrics event di-map ke panel model yang existing (current/baseline/reduction, state).
*   [x] Event throughput normal tidak menyebabkan memory leak atau render storm signifikan.
*   [x] Cleanup subscription berjalan konsisten saat shell unmount.

## Business Rules & Logic
*   Monitoring value harus live dan konsisten dengan engine agar user bisa keputusan cepat sebelum bermain.

## Dependencies
*   Depends on: KP-118, KP-114, KP-112, KP-115

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
