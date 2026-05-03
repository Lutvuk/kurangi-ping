# KP-032: Route Attempt Orchestrator (WG -> TCP/TLS -> QUIC)

**Epic:** EPIC-ROUTING
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 5
**Priority:** Must

---

## Objective
Create orchestrator that attempts route establishment by protocol priority across scored candidates.

## Technical Specifications
*   **Proposed Files:** `crates/client-engine/src/routing/orchestrator.rs`
*   **Functions/Classes:** `attempt_route()`, `AttemptPlan`, `AttemptResult`
*   **API Endpoints:** N/A
*   **Data Models:** Route attempt context model

## Acceptance Criteria (Technical)
*   [ ] Orchestrator attempts WG first, then TCP/TLS, then QUIC.
*   [ ] Candidate iteration follows scoring output.
*   [ ] Attempt outcomes are normalized for state machine consumption.
*   [ ] No infinite loops are possible by design.

## Business Rules & Logic
*   Protocol fallback order is mandatory and cannot be user-overridden in v1.

## Dependencies
*   Depends on: KP-031

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
