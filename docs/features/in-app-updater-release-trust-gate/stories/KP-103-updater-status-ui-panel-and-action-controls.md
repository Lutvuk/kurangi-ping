# KP-103: Updater Status UI Panel and Action Controls

**Epic:** EPIC-UPDATER
**Layer:** L4-feature-ui
**Role:** Frontend
**Estimation:** 3
**Priority:** Must

---

## Objective
Create updater status panel with check/update action controls and state-driven messaging.

## Technical Specifications
*   **Proposed Files:** `apps/desktop/src/features/updater/UpdaterPanel.tsx`
*   **Functions/Classes:** `UpdaterPanel`
*   **API Endpoints:** N/A
*   **Data Models:** Updater view model

## Acceptance Criteria (Technical)
*   [ ] Panel reflects normalized updater states.
*   [ ] Controls support check/download/apply actions as allowed by state.
*   [ ] Error states provide actionable retry path.
*   [ ] Styling remains aligned with design-system rules.

## Business Rules & Logic
*   User should understand update status quickly without technical complexity.

## Dependencies
*   Depends on: KP-101, KP-020

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
