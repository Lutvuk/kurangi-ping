# KP-037: Contract Conformance Checks for Relay Input and Telemetry Output

**Epic:** EPIC-ROUTING
**Layer:** L5-integration
**Role:** Fullstack
**Estimation:** 3
**Priority:** Must

---

## Objective
Verify routing engine IO remains conformant with relay public contract inputs and telemetry output schema.

## Technical Specifications
*   **Proposed Files:** `tests/contracts/relay_input_contract_test.rs`, `tests/contracts/telemetry_output_contract_test.rs`
*   **Functions/Classes:** Contract validation test suites
*   **API Endpoints:** `/v1/relay/manifest`, `/v1/relay/health`, telemetry event schema
*   **Data Models:** Contract fixture payloads

## Acceptance Criteria (Technical)
*   [x] Relay manifest input parsing matches OpenAPI schema expectations.
*   [x] Relay health input parsing matches OpenAPI schema expectations.
*   [x] Emitted routing lifecycle events conform to telemetry allowlist rules.
*   [x] Contract regressions fail CI gate.

## Business Rules & Logic
*   Contract drift must be detected early.

## Dependencies
*   Depends on: KP-034, KP-036

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
