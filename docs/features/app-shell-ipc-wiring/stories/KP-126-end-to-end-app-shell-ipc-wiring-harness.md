# KP-126: End-to-End App Shell IPC Wiring Harness

**Epic:** EPIC-WIRING
**Layer:** L5-integration
**Role:** Fullstack
**Estimation:** 3
**Priority:** Should

---

## Objective
Validate full app shell IPC journey (toggle invoke, routing badge sync, detection startup query, detection live update, metrics live stream) pada alur terintegrasi.

## Technical Specifications
*   **Proposed Files:** `apps/desktop/src/App.ipc-wiring.test.tsx`, `crates/client-engine/tests/app_shell_ipc_journey_test.rs`
*   **Functions/Classes:** `runAppShellIpcJourneySuite()`
*   **API Endpoints:** N/A
*   **Data Models:** End-to-end IPC fixture matrix

## Acceptance Criteria (Technical)
*   [x] ON action memicu invoke dan transisi badge `offline -> connecting` pada path sukses.
*   [x] Event `routing_state_changed` memutakhirkan badge state tanpa reload.
*   [x] Startup invoke `detection_get_status` menampilkan status awal detection panel.
*   [x] Event `detection_status_updated` dan `metrics_ping_sampled` memperbarui UI model secara real-time.
*   [x] Repeated mount/unmount tetap deterministic tanpa duplicate listener side-effect.

## Business Rules & Logic
*   Fitur wiring shell dianggap release-ready hanya bila alur IPC inti tervalidasi end-to-end.

## Dependencies
*   Depends on: KP-119, KP-120, KP-121, KP-122, KP-123, KP-124, KP-125, KP-116, KP-117

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
