# KP-020: Token-Compliant Button/Input/Select/Badge Set

**Epic:** EPIC-FOUNDATION
**Layer:** L2-ui-foundation
**Role:** Frontend
**Estimation:** 5
**Priority:** Must

---

## Objective
Implement core interactive primitives using design tokens and approved state semantics.

## Technical Specifications
*   **Proposed Files:** `Button.tsx`, `Input.tsx`, `Select.tsx`, `StatusBadge.tsx` under `apps/desktop/src/components/primitives/`
*   **Functions/Classes:** `Button`, `Input`, `Select`, `StatusBadge`
*   **API Endpoints:** N/A
*   **Data Models:** N/A

## Acceptance Criteria (Technical)
*   [x] Components use token-based styles from design system.
*   [x] Variants (primary/secondary/destructive) are implemented for Button.
*   [x] Focus and disabled states are explicit and accessible.
*   [x] StatusBadge supports off/connecting/on/degraded semantic mapping.

## Business Rules & Logic
*   Signal/probe color semantics must not be used decoratively.

## Dependencies
*   Depends on: KP-019

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
