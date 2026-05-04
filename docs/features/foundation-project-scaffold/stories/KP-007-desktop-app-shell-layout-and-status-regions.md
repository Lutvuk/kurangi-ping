# KP-007: Desktop App Shell Layout and Status Regions

**Epic:** EPIC-FOUNDATION
**Layer:** L4-feature-ui
**Role:** Frontend
**Estimation:** 3
**Priority:** Should

---

## Objective
Implement base desktop shell regions to host future feature modules (sidebar, top status bar, content panels).

## Technical Specifications
*   **Proposed Files:** `apps/desktop/src/layout/AppShell.tsx`, `apps/desktop/src/layout/Sidebar.tsx`, `apps/desktop/src/layout/TopBar.tsx`
*   **Functions/Classes:** `AppShell`, `Sidebar`, `TopBar`
*   **API Endpoints:** N/A
*   **Data Models:** N/A

## Acceptance Criteria (Technical)
*   [x] Shell renders with sidebar and top status region placeholders.
*   [x] Layout follows spacing/typography/color tokens from design system.
*   [x] Compact behavior is prepared for `--bp-compact` breakpoint.
*   [x] Placeholder content area supports future module injection.

## Business Rules & Logic
*   Data-centric visual hierarchy must be preserved.
*   No decorative styling outside approved design system.

## Dependencies
*   Depends on: KP-003, KP-004

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing (component render tests)
*   [x] Lint/Type check clear
