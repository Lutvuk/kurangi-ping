# KP-024: Primary Toggle and Connection Status Badge

**Epic:** EPIC-FOUNDATION
**Layer:** L4-feature-ui
**Role:** Frontend
**Estimation:** 3
**Priority:** Must

---

## Objective
Implement operational control toggle and status badge components with strict state semantics.

## Technical Specifications
*   **Proposed Files:** `apps/desktop/src/components/modules/PrimaryToggle.tsx`, `ConnectionStatusBadge.tsx`
*   **Functions/Classes:** `PrimaryToggle`, `ConnectionStatusBadge`
*   **API Endpoints:** N/A
*   **Data Models:** Status view-model props

## Acceptance Criteria (Technical)
*   [x] Toggle OFF/ON/connecting/degraded states are represented clearly.
*   [x] Signal/probe colors follow design rules.
*   [x] Instant snap motion is used for toggle state transitions.
*   [x] Components expose typed props for future integration.

## Business Rules & Logic
*   These controls are mission-critical for user confidence during gameplay.

## Dependencies
*   Depends on: KP-020, KP-023

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
