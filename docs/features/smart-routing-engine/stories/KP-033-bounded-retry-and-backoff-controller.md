# KP-033: Bounded Retry and Backoff Controller

**Epic:** EPIC-ROUTING
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 3
**Priority:** Must

---

## Objective
Implement bounded retry budget and backoff strategy for route failures.

## Technical Specifications
*   **Proposed Files:** `crates/client-engine/src/routing/retry.rs`
*   **Functions/Classes:** `next_retry_delay()`, `RetryBudget`
*   **API Endpoints:** N/A
*   **Data Models:** Retry state model

## Acceptance Criteria (Technical)
*   [x] Retry count never exceeds configured budget.
*   [x] Backoff schedule is deterministic and test-covered.
*   [x] Exhausted retries return terminal failure state.
*   [x] Retry metadata is emitted for telemetry.

## Business Rules & Logic
*   Failover behavior must be resilient but bounded.

## Dependencies
*   Depends on: KP-032

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
