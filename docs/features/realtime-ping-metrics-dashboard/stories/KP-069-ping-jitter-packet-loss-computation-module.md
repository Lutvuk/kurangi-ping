# KP-069: Ping/Jitter/Packet-Loss Computation Module

**Epic:** EPIC-METRICS
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 4
**Priority:** Must

---

## Objective
Compute baseline/routed ping plus jitter and packet loss from probe observations.

## Technical Specifications
*   **Proposed Files:** `crates/client-engine/src/metrics/computation.rs`
*   **Functions/Classes:** `compute_ping_metrics()`, `compute_jitter()`, `compute_packet_loss()`
*   **API Endpoints:** N/A
*   **Data Models:** Metric sample model

## Acceptance Criteria (Technical)
*   [x] Baseline and routed ping values computed consistently.
*   [x] Jitter and packet loss formulas are deterministic and tested.
*   [x] Invalid/missing sample windows are handled safely.
*   [x] Output format is compatible with UI and persistence modules.

## Business Rules & Logic
*   Accuracy and consistency of metrics are critical for user trust.

## Dependencies
*   Depends on: KP-068

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
