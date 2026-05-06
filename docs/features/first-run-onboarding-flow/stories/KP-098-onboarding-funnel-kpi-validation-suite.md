# KP-098: Onboarding Funnel KPI Validation Suite

**Epic:** EPIC-ONBOARDING
**Layer:** L5-integration
**Role:** Fullstack
**Estimation:** 3
**Priority:** Should

---

## Objective
Validate event/state data can derive onboarding funnel KPIs (completion rate, step drop-off, time-to-connect).

## Technical Specifications
*   **Proposed Files:** `tests/telemetry/onboarding_funnel_kpi_test.rs`
*   **Functions/Classes:** Funnel KPI validation suite
*   **API Endpoints:** N/A
*   **Data Models:** Onboarding event/step timeline fixtures

## Acceptance Criteria (Technical)
*   [x] Completion and drop-off KPI derivation is reproducible.
*   [x] Missing/invalid event sequences fail assertions.
*   [x] Time-to-connect metric derivation is validated.
*   [x] Output supports product tuning decisions.

## Business Rules & Logic
*   Funnel analytics must be reliable for launch optimization.

## Dependencies
*   Depends on: KP-093, KP-097, KP-088

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
