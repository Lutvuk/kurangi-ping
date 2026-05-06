# KP-080: Event Schema Allowlist Validator Core

**Epic:** EPIC-TELEMETRY
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 4
**Priority:** Must

---

## Objective
Create centralized schema allowlist validator for all telemetry event producers.

## Technical Specifications
*   **Proposed Files:** `crates/client-engine/src/telemetry/validator.rs`
*   **Functions/Classes:** `validate_event_payload()`, `SchemaAllowlist`
*   **API Endpoints:** N/A
*   **Data Models:** Telemetry schema model

## Acceptance Criteria (Technical)
*   [x] Validator enforces event-name specific allowed keys.
*   [x] Unknown keys are rejected or dropped per policy mode.
*   [x] Validation failures return deterministic reason codes.
*   [x] Validator is reused by all event emitters.

## Business Rules & Logic
*   Schema consistency is mandatory for privacy and analytics quality.

## Dependencies
*   Depends on: KP-072, KP-063, KP-053

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
