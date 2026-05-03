# KP-065: Lifecycle Status Presenter (idle/arming/active/disarming/error)

**Epic:** EPIC-TOGGLE
**Layer:** L4-feature-ui
**Role:** Frontend
**Estimation:** 3
**Priority:** Must

---

## Objective
Present normalized routing lifecycle states consistently in UI status regions.

## Technical Specifications
*   **Proposed Files:** `apps/desktop/src/features/routing/LifecycleStatusPresenter.tsx`
*   **Functions/Classes:** `LifecycleStatusPresenter`
*   **API Endpoints:** N/A
*   **Data Models:** Lifecycle state view model

## Acceptance Criteria (Technical)
*   [ ] All five lifecycle states are rendered with semantic styling.
*   [ ] State transitions are visually coherent with motion tokens.
*   [ ] Reason codes map to user-friendly text.
*   [ ] Accessibility support for status announcements is present.

## Business Rules & Logic
*   Status visibility is core to user trust.

## Dependencies
*   Depends on: KP-062, KP-064

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
