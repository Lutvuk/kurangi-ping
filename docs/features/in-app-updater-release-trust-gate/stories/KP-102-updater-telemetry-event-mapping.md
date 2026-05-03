# KP-102: Updater Telemetry Event Mapping

**Epic:** EPIC-UPDATER
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 3
**Priority:** Should

---

## Objective
Emit updater lifecycle events for adoption and failure analysis with schema-safe payloads.

## Technical Specifications
*   **Proposed Files:** `crates/client-engine/src/telemetry/events/updater.rs`
*   **Functions/Classes:** `emit_updater_event()`
*   **API Endpoints:** N/A
*   **Data Models:** Updater telemetry payload model

## Acceptance Criteria (Technical)
*   [ ] Check, available, download, apply, and failure events can be emitted.
*   [ ] Payloads follow telemetry allowlist rules.
*   [ ] Sensitive release or local path details are excluded.
*   [ ] Event sequence remains deterministic.

## Business Rules & Logic
*   Updater analytics must be useful without compromising privacy.

## Dependencies
*   Depends on: KP-101, KP-080

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
