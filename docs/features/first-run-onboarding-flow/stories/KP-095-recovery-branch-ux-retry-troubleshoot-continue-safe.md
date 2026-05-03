# KP-095: Recovery Branch UX (retry, troubleshoot, continue-safe)

**Epic:** EPIC-ONBOARDING
**Layer:** L4-feature-ui
**Role:** Frontend
**Estimation:** 3
**Priority:** Should

---

## Objective
Design and wire recovery branches for onboarding failures with clear retry/troubleshoot paths.

## Technical Specifications
*   **Proposed Files:** `apps/desktop/src/features/onboarding/RecoveryPanel.tsx`
*   **Functions/Classes:** `RecoveryPanel`
*   **API Endpoints:** N/A
*   **Data Models:** Recovery action model

## Acceptance Criteria (Technical)
*   [ ] Failure states present at least retry + guidance action.
*   [ ] Non-blocking paths allow safe continuation where permitted.
*   [ ] Error messages map to backend reason codes.
*   [ ] Accessibility checks pass for recovery interactions.

## Business Rules & Logic
*   Recovery UX should reduce abandonment during setup failures.

## Dependencies
*   Depends on: KP-090, KP-094

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
