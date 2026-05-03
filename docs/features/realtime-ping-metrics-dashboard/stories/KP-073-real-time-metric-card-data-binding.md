# KP-073: Real-Time Metric Card Data Binding

**Epic:** EPIC-METRICS
**Layer:** L4-feature-ui
**Role:** Frontend
**Estimation:** 3
**Priority:** Must

---

## Objective
Bind live metric stream into ping metric card UI components.

## Technical Specifications
*   **Proposed Files:** `apps/desktop/src/features/metrics/PingMetricsPanel.tsx`
*   **Functions/Classes:** `PingMetricsPanel`
*   **API Endpoints:** N/A
*   **Data Models:** Metrics view-model

## Acceptance Criteria (Technical)
*   [ ] Baseline/routed/reduction values update in near real-time.
*   [ ] Numeric formatting is stable and legible.
*   [ ] Color semantics align with live/degraded state.
*   [ ] Render updates avoid flicker under rapid samples.

## Business Rules & Logic
*   Primary value readability is highest UI priority.

## Dependencies
*   Depends on: KP-070, KP-025

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
