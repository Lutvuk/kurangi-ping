# KP-099: Updater Channel Resolver and Check Scheduler

**Epic:** EPIC-UPDATER
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 3
**Priority:** Must

---

## Objective
Resolve updater channel policy and schedule update checks with bounded cadence.

## Technical Specifications
*   **Proposed Files:** `crates/client-engine/src/updater/check_scheduler.rs`
*   **Functions/Classes:** `resolve_update_channel()`, `schedule_update_check()`
*   **API Endpoints:** N/A
*   **Data Models:** Updater check context model

## Acceptance Criteria (Technical)
*   [x] Supports `beta` and `stable` channels.
*   [x] Check cadence is configurable and bounded.
*   [x] Manual check trigger coexists safely with scheduled checks.
*   [x] Last-check metadata persists when configured.

## Business Rules & Logic
*   Update checks should be timely but not noisy.

## Dependencies
*   Depends on: KP-002, KP-062

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
