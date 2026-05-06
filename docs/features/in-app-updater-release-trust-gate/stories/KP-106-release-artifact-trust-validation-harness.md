# KP-106: Release Artifact Trust Validation Harness

**Epic:** EPIC-UPDATER
**Layer:** L5-integration
**Role:** Fullstack
**Estimation:** 3
**Priority:** Must

---

## Objective
Validate updater accepts trusted artifacts and rejects tampered/unsigned artifacts in controlled scenarios.

## Technical Specifications
*   **Proposed Files:** `tests/updater/artifact_trust_harness.rs`
*   **Functions/Classes:** `run_artifact_trust_suite()`
*   **API Endpoints:** N/A
*   **Data Models:** Artifact trust scenario fixtures

## Acceptance Criteria (Technical)
*   [x] Trusted signed artifact path passes.
*   [x] Unsigned/tampered artifact path fails deterministically.
*   [x] Failure reasons map to updater error taxonomy.
*   [x] Harness output is reproducible.

## Business Rules & Logic
*   Trust verification should fail closed.

## Dependencies
*   Depends on: KP-100, KP-105

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
