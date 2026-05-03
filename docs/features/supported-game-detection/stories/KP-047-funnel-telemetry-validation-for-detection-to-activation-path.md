# KP-047: Funnel Telemetry Validation for Detection-to-Activation Path

**Epic:** EPIC-DETECTION
**Layer:** L5-integration
**Role:** Fullstack
**Estimation:** 3
**Priority:** Should

---

## Objective
Validate detection-to-activation funnel telemetry consistency across engine and UI transitions.

## Technical Specifications
*   **Proposed Files:** `tests/telemetry/detection_activation_funnel_test.rs`
*   **Functions/Classes:** Funnel validation test suite
*   **API Endpoints:** N/A
*   **Data Models:** Event sequence fixtures

## Acceptance Criteria (Technical)
*   [ ] Event sequence includes `game_detected` before routing activation in success flow.
*   [ ] Negative flows do not emit invalid activation events.
*   [ ] Payload schema remains compliant for all funnel events.
*   [ ] Test reports make drop-off/debug points explicit.

## Business Rules & Logic
*   Funnel analytics quality is needed for onboarding optimization.

## Dependencies
*   Depends on: KP-043, KP-045, KP-046

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
