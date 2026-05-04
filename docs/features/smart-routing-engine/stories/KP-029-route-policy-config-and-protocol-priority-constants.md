# KP-029: Route Policy Config and Protocol Priority Constants

**Epic:** EPIC-ROUTING
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 3
**Priority:** Must

---

## Objective
Define routing policy constants and config model for protocol priority and failover behavior.

## Technical Specifications
*   **Proposed Files:** `crates/client-engine/src/routing/policy.rs`, `crates/client-engine/src/routing/config.rs`
*   **Functions/Classes:** `RoutingPolicy`, `ProtocolPriority`, `RetryPolicy`
*   **API Endpoints:** N/A
*   **Data Models:** N/A

## Acceptance Criteria (Technical)
*   [x] Default protocol order is WG -> TCP/TLS -> QUIC.
*   [x] Policy constants are centralized and typed.
*   [x] Config validation rejects invalid priority sets.
*   [x] Policy module is unit-testable independently.

## Business Rules & Logic
*   Protocol order is fixed by approved architecture decision.

## Dependencies
*   Depends on: KP-005

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
