# KP-075: Metric Trend Mini-View (optional lightweight sparkline-ready container)

**Epic:** EPIC-METRICS
**Layer:** L4-feature-ui
**Role:** Frontend
**Estimation:** 2
**Priority:** Could

---

## Objective
Add lightweight trend container preparing future sparkline visualization without heavy chart dependency.

## Technical Specifications
*   **Proposed Files:** `apps/desktop/src/features/metrics/MetricsTrendMiniView.tsx`
*   **Functions/Classes:** `MetricsTrendMiniView`
*   **API Endpoints:** N/A
*   **Data Models:** Recent metrics trend model

## Acceptance Criteria (Technical)
*   [x] Container supports rendering of recent sample sequence.
*   [x] Gracefully handles low sample counts.
*   [x] Optional module does not block core dashboard delivery.
*   [x] Performance overhead remains minimal.

## Business Rules & Logic
*   Trend view is additive and must not compromise baseline readability.

## Dependencies
*   Depends on: KP-071, KP-073

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
