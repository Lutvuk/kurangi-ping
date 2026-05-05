# KP-054: Relay Health Surface Wiring (list + badge sync)

**Epic:** EPIC-RELAY
**Layer:** L4-feature-ui
**Role:** Frontend
**Estimation:** 3
**Priority:** Must

---

## Objective
Wire backend health and failover state into relay list and status badge components.

## Technical Specifications
*   **Proposed Files:** `apps/desktop/src/features/relay/RelayHealthPanel.tsx`
*   **Functions/Classes:** `RelayHealthPanel`
*   **API Endpoints:** N/A
*   **Data Models:** Relay health/failover view model

## Acceptance Criteria (Technical)
*   [x] Relay list reflects live health classification.
*   [x] Active relay highlight syncs with engine state.
*   [x] Badge transitions reflect failover lifecycle correctly.
*   [x] Visual semantics follow design system color rules.

## Business Rules & Logic
*   UI must make failover behavior understandable at a glance.

## Dependencies
*   Depends on: KP-052, KP-025

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
