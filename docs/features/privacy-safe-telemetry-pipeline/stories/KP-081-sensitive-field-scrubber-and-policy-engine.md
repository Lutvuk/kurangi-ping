# KP-081: Sensitive Field Scrubber and Policy Engine

**Epic:** EPIC-TELEMETRY
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 4
**Priority:** Must

---

## Objective
Implement policy-driven scrubbing/rejection for prohibited fields (PII and sensitive network data).

## Technical Specifications
*   **Proposed Files:** `crates/client-engine/src/telemetry/scrubber.rs`
*   **Functions/Classes:** `scrub_or_reject_payload()`, `SensitiveFieldPolicy`
*   **API Endpoints:** N/A
*   **Data Models:** Scrub result model

## Acceptance Criteria (Technical)
*   [ ] Prohibited fields are never persisted or transmitted.
*   [ ] Policy mode (reject/sanitize) is configurable.
*   [ ] Violations are logged in non-sensitive diagnostic form.
*   [ ] Nested payload fields are handled safely.

## Business Rules & Logic
*   Privacy boundaries must be enforced even on malformed producer payloads.

## Dependencies
*   Depends on: KP-080

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
