# KP-083: Telemetry Delivery Client with Idempotency Key

**Epic:** EPIC-TELEMETRY
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 3
**Priority:** Must

---

## Objective
Implement telemetry HTTP delivery client with idempotency headers and robust response handling.

## Technical Specifications
*   **Proposed Files:** `crates/client-engine/src/telemetry/delivery_client.rs`
*   **Functions/Classes:** `send_batch()`, `build_idempotency_key()`
*   **API Endpoints:** `POST /v1/telemetry/events:batch`
*   **Data Models:** Delivery request/response model

## Acceptance Criteria (Technical)
*   [ ] Requests include idempotency key.
*   [ ] Response handling supports accepted/partial/failure outcomes.
*   [ ] Transport errors propagate to retry state machine.
*   [ ] Sensitive payload data is not leaked in logs.

## Business Rules & Logic
*   Delivery must be resilient and non-duplicative.

## Dependencies
*   Depends on: KP-082

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
