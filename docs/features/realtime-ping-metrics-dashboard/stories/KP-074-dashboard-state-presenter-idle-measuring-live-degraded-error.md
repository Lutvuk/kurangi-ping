# KP-074: Dashboard State Presenter (idle/measuring/live/degraded/error)

**Epic:** EPIC-METRICS
**Layer:** L4-feature-ui
**Role:** Frontend
**Estimation:** 3
**Priority:** Must

---

## Objective
Present normalized measurement lifecycle states with clear user messaging.

## Technical Specifications
*   **Proposed Files:** `apps/desktop/src/features/metrics/MetricsStatePresenter.tsx`
*   **Functions/Classes:** `MetricsStatePresenter`
*   **API Endpoints:** N/A
*   **Data Models:** Metrics state display model

## Acceptance Criteria (Technical)
*   [x] All five dashboard states are represented.
*   [x] Degraded/error states show actionable guidance.
*   [x] Motion and styling remain consistent with design tokens.
*   [x] Accessibility support for state announcement is included.

## Business Rules & Logic
*   State clarity prevents misinterpretation of unstable measurements.

## Dependencies
*   Depends on: KP-070, KP-073

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
