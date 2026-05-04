# KP-027: UI Composition Showcase Page (foundation states)

**Epic:** EPIC-FOUNDATION
**Layer:** L5-integration
**Role:** Frontend
**Estimation:** 3
**Priority:** Should

---

## Objective
Build a local showcase page to validate composition and state variants of foundation UI components.

## Technical Specifications
*   **Proposed Files:** `apps/desktop/src/pages/foundation-showcase.tsx`
*   **Functions/Classes:** `FoundationShowcasePage`
*   **API Endpoints:** N/A
*   **Data Models:** Mock state models

## Acceptance Criteria (Technical)
*   [x] Showcase renders all major foundation components.
*   [x] Each component exposes key operational states.
*   [x] Layout verifies compact/full behavior quickly.
*   [x] Page is excluded from production routing if required.

## Business Rules & Logic
*   Showcase acts as visual contract and manual QA surface.

## Dependencies
*   Depends on: KP-024, KP-025, KP-026

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
