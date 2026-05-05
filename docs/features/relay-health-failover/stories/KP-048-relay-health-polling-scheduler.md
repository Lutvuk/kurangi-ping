# KP-048: Relay Health Polling Scheduler

**Epic:** EPIC-RELAY
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 3
**Priority:** Must

---

## Objective
Implement periodic scheduler for relay health refresh with bounded polling cadence.

## Technical Specifications
*   **Proposed Files:** `crates/client-engine/src/routing/health_scheduler.rs`
*   **Functions/Classes:** `start_health_polling()`, `stop_health_polling()`
*   **API Endpoints:** `[GET] /v1/relay/health`
*   **Data Models:** Health polling context model

## Acceptance Criteria (Technical)
*   [x] Polling interval is configurable and bounded.
*   [x] Scheduler avoids concurrent overlapping polls.
*   [x] Poll failures propagate to failover evaluator path.
*   [x] Polling can be paused/resumed by routing lifecycle.

## Business Rules & Logic
*   Polling cadence should balance responsiveness and overhead.

## Dependencies
*   Depends on: KP-031

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
