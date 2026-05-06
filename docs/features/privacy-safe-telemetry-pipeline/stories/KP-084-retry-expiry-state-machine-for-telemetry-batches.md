# KP-084: Retry/Expiry State Machine for Telemetry Batches

**Epic:** EPIC-TELEMETRY
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 3
**Priority:** Must

---

## Objective
Manage retry progression and expiry transitions for queued telemetry batches.

## Technical Specifications
*   **Proposed Files:** `crates/client-engine/src/telemetry/retry_state_machine.rs`
*   **Functions/Classes:** `next_batch_state()`, `compute_next_retry_at()`
*   **API Endpoints:** N/A
*   **Data Models:** Telemetry batch lifecycle state model

## Acceptance Criteria (Technical)
*   [x] Retry budget is respected.
*   [x] Expired batches transition and purge eligibility is marked.
*   [x] Backoff schedule is deterministic.
*   [x] State transitions are test-covered.

## Business Rules & Logic
*   Retrying must be bounded to avoid infinite resource drain.

## Dependencies
*   Depends on: KP-082, KP-083, KP-079

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
