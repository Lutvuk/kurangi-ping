# KP-127: Environment Contract Extension for Relay Probe Targets and Timeouts

**Epic:** EPIC-RELAY
**Layer:** L1-data
**Role:** Backend
**Estimation:** 2
**Priority:** Must

---

## Objective
Define environment contract keys for relay probe target list and probe timeout settings so relay health can be configured without code changes.

## Technical Specifications
*   **Proposed Files:** `.env.example`, `apps/relay-controller/README.md`
*   **Functions/Classes:** N/A
*   **API Endpoints:** N/A
*   **Data Models:** Relay probe environment contract

## Acceptance Criteria (Technical)
*   [x] `.env.example` documents relay probe target source and timeout keys.
*   [x] Contract explains expected target format and safe defaults.
*   [x] Relay controller README references the new env keys and usage.
*   [x] No existing env keys are removed or renamed.

## Business Rules & Logic
*   Relay targets must be environment-driven, not hardcoded in source.

## Dependencies
*   Depends on: KP-002

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
