# KP-023: Accessibility Baseline for Interactive Primitives

**Epic:** EPIC-FOUNDATION
**Layer:** L2-ui-foundation
**Role:** Frontend
**Estimation:** 3
**Priority:** Must

---

## Objective
Apply accessibility baseline across primitive components (ARIA, keyboard nav, focus visibility).

## Technical Specifications
*   **Proposed Files:** primitive components updates + `apps/desktop/src/components/primitives/a11y.md`
*   **Functions/Classes:** Keyboard interaction handlers
*   **API Endpoints:** N/A
*   **Data Models:** N/A

## Acceptance Criteria (Technical)
*   [ ] Interactive components have explicit ARIA labels/roles.
*   [ ] Tab order is logical and deterministic.
*   [ ] Focus ring visibility is consistent and token-driven.
*   [ ] Basic keyboard interactions are covered by tests.

## Business Rules & Logic
*   Accessibility is baseline quality, not optional enhancement.

## Dependencies
*   Depends on: KP-020, KP-021

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
