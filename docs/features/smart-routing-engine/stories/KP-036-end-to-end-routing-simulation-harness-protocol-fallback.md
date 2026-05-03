# KP-036: End-to-End Routing Simulation Harness (protocol fallback)

**Epic:** EPIC-ROUTING
**Layer:** L5-integration
**Role:** Fullstack
**Estimation:** 3
**Priority:** Must

---

## Objective
Build simulation harness to validate multi-protocol fallback behavior under controlled failure scenarios.

## Technical Specifications
*   **Proposed Files:** `tests/routing/fallback_harness.rs`, `tests/fixtures/relay_scenarios/`
*   **Functions/Classes:** `run_fallback_scenario()`
*   **API Endpoints:** Simulated manifest/health fixtures
*   **Data Models:** Scenario definitions

## Acceptance Criteria (Technical)
*   [ ] Harness validates WG failure to TCP/TLS fallback.
*   [ ] Harness validates TCP/TLS failure to QUIC tertiary path.
*   [ ] Exhaustion scenario produces terminal failed state.
*   [ ] Test output includes deterministic trace of attempts.

## Business Rules & Logic
*   Fallback reliability must be provable before feature wiring expands.

## Dependencies
*   Depends on: KP-032, KP-033, KP-034

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
