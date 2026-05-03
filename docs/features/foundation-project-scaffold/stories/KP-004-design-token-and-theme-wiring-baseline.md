# KP-004: Design Token and Theme Wiring Baseline

**Epic:** EPIC-FOUNDATION
**Layer:** L2-ui-foundation
**Role:** Frontend
**Estimation:** 3
**Priority:** Must

---

## Objective
Wire design token primitives from approved design system into desktop UI baseline.

## Technical Specifications
*   **Proposed Files:** `apps/desktop/src/styles/tokens.css`, `apps/desktop/src/styles/theme.css`, `apps/desktop/src/styles/index.css`
*   **Functions/Classes:** N/A
*   **API Endpoints:** N/A
*   **Data Models:** N/A

## Acceptance Criteria (Technical)
*   [ ] Core color, typography, spacing, and motion tokens are declared in CSS variables.
*   [ ] Base app shell consumes approved tokens for background/text/border.
*   [ ] Theme files match `docs/design-system.md` semantics.
*   [ ] No unauthorized default theme substitutions introduced.

## Business Rules & Logic
*   Visual language must preserve tactical/trustworthy direction.
*   Signal/probe color usage rules remain enforceable.

## Dependencies
*   Depends on: KP-003

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing (visual token checks optional)
*   [ ] Lint/Type check clear
