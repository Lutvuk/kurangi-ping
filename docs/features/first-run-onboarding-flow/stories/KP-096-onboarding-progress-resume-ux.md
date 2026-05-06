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
*   [x] UI resumes at persisted checkpoint step.
*   [x] Resume context is clearly indicated to user.
*   [x] Reset/restart option is available with confirmation.
*   [x] Resume path remains consistent with backend checkpoint data.

## Business Rules & Logic
*   Resume should preserve momentum and avoid confusion.

## Dependencies
*   Depends on: KP-091, KP-094

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
