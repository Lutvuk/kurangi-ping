# KP-025: Ping Metric Card and Relay Health List Item

**Epic:** EPIC-FOUNDATION
**Layer:** L4-feature-ui
**Role:** Frontend
**Estimation:** 4
**Priority:** Must

---

## Objective
Create data-heavy monitoring components for ping metrics and relay health visibility.

## Technical Specifications
*   **Proposed Files:** `PingMetricCard.tsx`, `RelayHealthListItem.tsx` under `apps/desktop/src/components/modules/`
*   **Functions/Classes:** `PingMetricCard`, `RelayHealthListItem`
*   **API Endpoints:** N/A
*   **Data Models:** Metric/relay typed view models

## Acceptance Criteria (Technical)
*   [ ] Ping card uses mono typography and sharp edge styling.
*   [ ] Relay item supports active/inactive and health color states.
*   [ ] Values and units are clearly distinguished.
*   [ ] Component props are independent from transport/API concerns.

## Business Rules & Logic
*   Data scanning speed is primary UX objective.

## Dependencies
*   Depends on: KP-021, KP-022

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
