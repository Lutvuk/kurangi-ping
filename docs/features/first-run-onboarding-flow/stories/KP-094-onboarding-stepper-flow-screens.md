# KP-094: Onboarding Stepper Flow Screens

**Epic:** EPIC-ONBOARDING
**Layer:** L4-feature-ui
**Role:** Frontend
**Estimation:** 5
**Priority:** Must

---

## Objective
Implement UI step flow screens for welcome, checks, relay test, detection test, and first connect.

## Technical Specifications
*   **Proposed Files:** `apps/desktop/src/features/onboarding/OnboardingFlow.tsx`, `steps/*`
*   **Functions/Classes:** `OnboardingFlow`, step components
*   **API Endpoints:** N/A
*   **Data Models:** Onboarding step view models

## Acceptance Criteria (Technical)
*   [ ] Five-step flow renders in defined order.
*   [ ] Step state reflects backend onboarding state machine.
*   [ ] UI copy remains concise and non-technical.
*   [ ] Layout follows design-system onboarding patterns.

## Business Rules & Logic
*   Onboarding should feel fast, clear, and low-friction.

## Dependencies
*   Depends on: KP-089, KP-026

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
