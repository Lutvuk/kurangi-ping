# KP-022: Shell Layout Components (AppShell, Sidebar, TopBar, Panel)

**Epic:** EPIC-FOUNDATION
**Layer:** L2-ui-foundation
**Role:** Frontend
**Estimation:** 5
**Priority:** Must

---

## Objective
Implement core layout regions and responsive shell behavior for compact/full desktop modes.

## Technical Specifications
*   **Proposed Files:** `AppShell.tsx`, `Sidebar.tsx`, `TopBar.tsx`, `Panel.tsx` under `apps/desktop/src/layout/`
*   **Functions/Classes:** `AppShell`, `Sidebar`, `TopBar`, `Panel`
*   **API Endpoints:** N/A
*   **Data Models:** N/A

## Acceptance Criteria (Technical)
*   [ ] Sidebar/topbar/content regions render consistently.
*   [ ] Breakpoint behavior follows `--bp-compact` and `--bp-full`.
*   [ ] Layout consumes spacing and typography tokens.
*   [ ] Panel structure supports future feature mounting.

## Business Rules & Logic
*   Layout must prioritize data readability and operational scanning speed.

## Dependencies
*   Depends on: KP-019, KP-020

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
