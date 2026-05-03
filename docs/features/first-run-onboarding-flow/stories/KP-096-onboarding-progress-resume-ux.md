# KP-096: Onboarding Progress Resume UX

**Epic:** EPIC-ONBOARDING
**Layer:** L4-feature-ui
**Role:** Frontend
**Estimation:** 3
**Priority:** Should

---

## Objective
Provide clear resume behavior and messaging when user returns mid-onboarding.

## Technical Specifications
*   **Proposed Files:** `apps/desktop/src/features/onboarding/ResumeEntry.tsx`
*   **Functions/Classes:** `ResumeEntry`
*   **API Endpoints:** N/A
*   **Data Models:** Resume state model

## Acceptance Criteria (Technical)
*   [ ] UI resumes at persisted checkpoint step.
*   [ ] Resume context is clearly indicated to user.
*   [ ] Reset/restart option is available with confirmation.
*   [ ] Resume path remains consistent with backend checkpoint data.

## Business Rules & Logic
*   Resume should preserve momentum and avoid confusion.

## Dependencies
*   Depends on: KP-091, KP-094

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
