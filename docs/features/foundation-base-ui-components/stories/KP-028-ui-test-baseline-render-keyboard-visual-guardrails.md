# KP-028: UI Test Baseline (render + keyboard + visual guardrails)

**Epic:** EPIC-FOUNDATION
**Layer:** L5-integration
**Role:** Frontend
**Estimation:** 3
**Priority:** Must

---

## Objective
Add baseline UI tests to guard rendering, keyboard accessibility, and semantic state consistency.

## Technical Specifications
*   **Proposed Files:** `apps/desktop/src/components/__tests__/` suite, `apps/desktop/src/test/setup.ts`
*   **Functions/Classes:** Component render tests and keyboard interaction tests
*   **API Endpoints:** N/A
*   **Data Models:** Mock props for state matrix

## Acceptance Criteria (Technical)
*   [x] Render tests cover key primitives and module components.
*   [x] Keyboard interaction tests cover toggle/modal/navigation essentials.
*   [x] Semantic status color/state mapping has regression coverage.
*   [x] Test suite integrates with CI baseline.

## Business Rules & Logic
*   UI consistency and operational clarity must be protected against regressions.

## Dependencies
*   Depends on: KP-023, KP-027

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
