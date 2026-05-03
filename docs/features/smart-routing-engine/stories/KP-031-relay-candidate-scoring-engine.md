# KP-031: Relay Candidate Scoring Engine

**Epic:** EPIC-ROUTING
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 4
**Priority:** Must

---

## Objective
Implement deterministic relay scoring based on health, region preference, and policy weight.

## Technical Specifications
*   **Proposed Files:** `crates/client-engine/src/routing/scoring.rs`
*   **Functions/Classes:** `score_candidates()`, `CandidateScore`
*   **API Endpoints:** Consumes `[GET] /v1/relay/health`
*   **Data Models:** Relay health + manifest candidate model

## Acceptance Criteria (Technical)
*   [ ] Score algorithm produces stable ordering for identical inputs.
*   [ ] Region preference affects score as configured.
*   [ ] Unhealthy relays are deprioritized/excluded per threshold.
*   [ ] Score output is inspectable for debugging.

## Business Rules & Logic
*   Selection must be predictable and explainable.

## Dependencies
*   Depends on: KP-029, KP-030

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
