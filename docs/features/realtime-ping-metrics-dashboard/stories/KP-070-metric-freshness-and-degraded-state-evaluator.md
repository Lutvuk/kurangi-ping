# KP-070: Metric Freshness and Degraded-State Evaluator

**Epic:** EPIC-METRICS
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 3
**Priority:** Must

---

## Objective
Evaluate sample freshness and trigger degraded state when probe signals become stale/unreliable.

## Technical Specifications
*   **Proposed Files:** `crates/client-engine/src/metrics/freshness.rs`
*   **Functions/Classes:** `evaluate_metric_freshness()`, `resolve_metrics_state()`
*   **API Endpoints:** N/A
*   **Data Models:** Metrics state enum

## Acceptance Criteria (Technical)
*   [x] Freshness timeout is configurable.
*   [x] Stale metrics transition to degraded state deterministically.
*   [x] Last-known values remain available for UI rendering.
*   [x] Error states include reason codes for diagnostics.

## Business Rules & Logic
*   Dashboard must remain readable during transient probe issues.

## Dependencies
*   Depends on: KP-069

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
