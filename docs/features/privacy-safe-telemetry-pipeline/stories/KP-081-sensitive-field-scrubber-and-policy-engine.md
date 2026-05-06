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
*   [x] Prohibited fields are never persisted or transmitted.
*   [x] Policy mode (reject/sanitize) is configurable.
*   [x] Violations are logged in non-sensitive diagnostic form.
*   [x] Nested payload fields are handled safely.

## Business Rules & Logic
*   Privacy boundaries must be enforced even on malformed producer payloads.

## Dependencies
*   Depends on: KP-080

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
