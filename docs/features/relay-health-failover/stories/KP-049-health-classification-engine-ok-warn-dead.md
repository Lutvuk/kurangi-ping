# KP-049: Health Classification Engine (ok/warn/dead)

**Epic:** EPIC-RELAY
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 3
**Priority:** Must

---

## Objective
Map relay metrics into normalized health buckets used by routing and UI.

## Technical Specifications
*   **Proposed Files:** `crates/client-engine/src/routing/health_classification.rs`
*   **Functions/Classes:** `classify_relay_health()`
*   **API Endpoints:** N/A
*   **Data Models:** `RelayHealthStatus` enum

## Acceptance Criteria (Technical)
*   [x] Classification outputs only `ok`, `warn`, `dead`.
*   [x] Threshold configuration is centralized and typed.
*   [x] Unknown/missing metrics map to deterministic fallback state.
*   [x] Unit tests cover threshold boundaries.

## Business Rules & Logic
*   Health labels must be stable and interpretable by users.

## Dependencies
*   Depends on: KP-048

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
