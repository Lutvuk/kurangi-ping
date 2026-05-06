# KP-104: Restart Prompt and Post-Update UX Flow

**Epic:** EPIC-UPDATER
**Layer:** L4-feature-ui
**Role:** Frontend
**Estimation:** 2
**Priority:** Should

---

## Objective
Implement restart-ready prompt and post-update confirmation UX flow.

## Technical Specifications
*   **Proposed Files:** `apps/desktop/src/features/updater/RestartPrompt.tsx`
*   **Functions/Classes:** `RestartPrompt`
*   **API Endpoints:** N/A
*   **Data Models:** Restart prompt model

## Acceptance Criteria (Technical)
*   [x] Prompt appears only when update is ready-to-restart.
*   [x] User can defer restart safely.
*   [x] Post-update confirmation state is visible on relaunch.
*   [x] Accessibility checks pass for modal/prompt interactions.

## Business Rules & Logic
*   Restart UX should be explicit and low-friction.

## Dependencies
*   Depends on: KP-103

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
