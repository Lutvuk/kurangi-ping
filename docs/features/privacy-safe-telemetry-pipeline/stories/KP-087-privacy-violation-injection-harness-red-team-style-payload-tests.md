# KP-087: Privacy Violation Injection Harness (red-team style payload tests)

**Epic:** EPIC-TELEMETRY
**Layer:** L5-integration
**Role:** Fullstack
**Estimation:** 3
**Priority:** Must

---

## Objective
Stress-test privacy controls with intentionally malicious/sensitive payload patterns.

## Technical Specifications
*   **Proposed Files:** `tests/privacy/telemetry_red_team_harness.rs`
*   **Functions/Classes:** `run_privacy_violation_suite()`
*   **API Endpoints:** N/A
*   **Data Models:** Malicious payload fixture set

## Acceptance Criteria (Technical)
*   [ ] PII-like fields are always rejected/scrubbed by policy.
*   [ ] Nested sensitive keys are detected.
*   [ ] Pipeline remains stable under repeated violation attempts.
*   [ ] Diagnostics remain non-sensitive.

## Business Rules & Logic
*   Privacy defense-in-depth must be demonstrably robust.

## Dependencies
*   Depends on: KP-081, KP-086

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
