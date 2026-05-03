# KP-021: Surface Components (Card, Modal, Toast)

**Epic:** EPIC-FOUNDATION
**Layer:** L2-ui-foundation
**Role:** Frontend
**Estimation:** 5
**Priority:** Must

---

## Objective
Build reusable surface components aligned with dark-ops visual system and interaction patterns.

## Technical Specifications
*   **Proposed Files:** `Card.tsx`, `Modal.tsx`, `Toast.tsx` under `apps/desktop/src/components/primitives/`
*   **Functions/Classes:** `Card`, `Modal`, `Toast`
*   **API Endpoints:** N/A
*   **Data Models:** N/A

## Acceptance Criteria (Technical)
*   [ ] Card uses sharp data-oriented styling.
*   [ ] Modal supports overlay, focus trap, and keyboard close handling.
*   [ ] Toast supports info/warning/error/success types with semantic accents.
*   [ ] Motion timings align with design tokens.

## Business Rules & Logic
*   No generic shadow-heavy UI style drift is allowed.

## Dependencies
*   Depends on: KP-019, KP-020

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
