# KP-093: onboarding_completed Telemetry Emission

**Epic:** EPIC-ONBOARDING
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 3
**Priority:** Must

---

## Objective
Emit `onboarding_completed` event with duration and success-path metadata.

## Technical Specifications
*   **Proposed Files:** `crates/client-engine/src/telemetry/events/onboarding.rs`
*   **Functions/Classes:** `emit_onboarding_completed()`
*   **API Endpoints:** N/A
*   **Data Models:** Onboarding completion event payload

## Acceptance Criteria (Technical)
*   [ ] Event emits only when completion gate succeeds.
*   [ ] Payload includes duration and path summary fields.
*   [ ] Payload remains schema-allowlisted and privacy-safe.
*   [ ] Event duplication is prevented for same completion cycle.

## Business Rules & Logic
*   Completion analytics must be accurate for funnel tracking.

## Dependencies
*   Depends on: KP-092, KP-080

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
