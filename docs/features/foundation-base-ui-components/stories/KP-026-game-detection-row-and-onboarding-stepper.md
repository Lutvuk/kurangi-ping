# KP-026: Game Detection Row and Onboarding Stepper

**Epic:** EPIC-FOUNDATION
**Layer:** L4-feature-ui
**Role:** Frontend
**Estimation:** 4
**Priority:** Should

---

## Objective
Implement foundational UI blocks for game detection display and guided onboarding steps.

## Technical Specifications
*   **Proposed Files:** `GameDetectionRow.tsx`, `OnboardingStepper.tsx` under `apps/desktop/src/components/modules/`
*   **Functions/Classes:** `GameDetectionRow`, `OnboardingStepper`
*   **API Endpoints:** N/A
*   **Data Models:** Onboarding step model, game detection row model

## Acceptance Criteria (Technical)
*   [x] Game row shows icon/name/server/status using token styles.
*   [x] Stepper supports inactive/active/completed visuals.
*   [x] Transition motion for steps aligns with design duration tokens.
*   [x] Components are keyboard navigable where applicable.

## Business Rules & Logic
*   Onboarding should remain concise and operationally clear.

## Dependencies
*   Depends on: KP-022, KP-023

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
