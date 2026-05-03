# KP-055: Failover Progress Feedback UI States

**Epic:** EPIC-RELAY
**Layer:** L4-feature-ui
**Role:** Frontend
**Estimation:** 3
**Priority:** Should

---

## Objective
Add explicit failover progress and outcome messaging states to reduce user uncertainty.

## Technical Specifications
*   **Proposed Files:** `apps/desktop/src/features/relay/FailoverStatusNotice.tsx`
*   **Functions/Classes:** `FailoverStatusNotice`
*   **API Endpoints:** N/A
*   **Data Models:** Failover reason/message mapping model

## Acceptance Criteria (Technical)
*   [ ] UI shows reconnecting/recovered/no-relay states with clear text.
*   [ ] Messaging maps to backend reason codes consistently.
*   [ ] Error states avoid exposing technical internals unnecessarily.
*   [ ] Accessibility checks pass for state announcements.

## Business Rules & Logic
*   Transparent status reduces panic and support burden.

## Dependencies
*   Depends on: KP-054

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
