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
*   [x] ON click triggers ON pipeline command.
*   [x] OFF click triggers OFF pipeline command.
*   [x] UI blocks/queues duplicate commands per guard policy.
*   [x] Error feedback is surfaced without technical overload.

## Business Rules & Logic
*   One-click UX should feel decisive and safe.

## Dependencies
*   Depends on: KP-061, KP-024

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
