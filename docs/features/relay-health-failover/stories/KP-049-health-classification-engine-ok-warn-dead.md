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
*   [ ] Classification outputs only `ok`, `warn`, `dead`.
*   [ ] Threshold configuration is centralized and typed.
*   [ ] Unknown/missing metrics map to deterministic fallback state.
*   [ ] Unit tests cover threshold boundaries.

## Business Rules & Logic
*   Health labels must be stable and interpretable by users.

## Dependencies
*   Depends on: KP-048

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
