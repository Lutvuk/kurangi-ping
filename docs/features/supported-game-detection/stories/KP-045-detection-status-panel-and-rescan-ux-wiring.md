# KP-045: Detection Status Panel and Rescan UX Wiring

**Epic:** EPIC-DETECTION
**Layer:** L4-feature-ui
**Role:** Frontend
**Estimation:** 4
**Priority:** Must

---

## Objective
Wire detection states and manual rescan action into UI panel using existing base components.

## Technical Specifications
*   **Proposed Files:** `apps/desktop/src/features/detection/DetectionPanel.tsx`
*   **Functions/Classes:** `DetectionPanel`
*   **API Endpoints:** N/A
*   **Data Models:** Detection view-model props

## Acceptance Criteria (Technical)
*   [ ] Panel renders all normalized detection states.
*   [ ] Rescan action triggers command and refreshes state.
*   [ ] Status visuals follow semantic color rules.
*   [ ] Keyboard interaction and focus behavior are accessible.

## Business Rules & Logic
*   Detection status clarity is required before user toggles routing.

## Dependencies
*   Depends on: KP-042, KP-024, KP-026

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
