# KP-064: Primary Toggle Wiring to Lifecycle Commands

**Epic:** EPIC-TOGGLE
**Layer:** L4-feature-ui
**Role:** Frontend
**Estimation:** 3
**Priority:** Must

---

## Objective
Bind primary toggle interaction to backend ON/OFF command lifecycle with lock-aware UX.

## Technical Specifications
*   **Proposed Files:** `apps/desktop/src/features/routing/ToggleController.tsx`
*   **Functions/Classes:** `ToggleController`
*   **API Endpoints:** N/A
*   **Data Models:** Toggle command/result view model

## Acceptance Criteria (Technical)
*   [ ] ON click triggers ON pipeline command.
*   [ ] OFF click triggers OFF pipeline command.
*   [ ] UI blocks/queues duplicate commands per guard policy.
*   [ ] Error feedback is surfaced without technical overload.

## Business Rules & Logic
*   One-click UX should feel decisive and safe.

## Dependencies
*   Depends on: KP-061, KP-024

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
