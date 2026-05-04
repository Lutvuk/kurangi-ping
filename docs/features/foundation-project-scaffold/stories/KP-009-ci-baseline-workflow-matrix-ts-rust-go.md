# KP-009: CI Baseline Workflow Matrix (TS/Rust/Go)

**Epic:** EPIC-FOUNDATION
**Layer:** L5-integration
**Role:** DevOps
**Estimation:** 3
**Priority:** Must

---

## Objective
Implement baseline CI workflows that validate lint/type/test/build across desktop TypeScript, Rust engine, and Go relay modules.

## Technical Specifications
*   **Proposed Files:** `.github/workflows/ci.yml`, optional split workflows under `.github/workflows/`
*   **Functions/Classes:** CI jobs and matrices
*   **API Endpoints:** N/A
*   **Data Models:** N/A

## Acceptance Criteria (Technical)
*   [x] Pull request triggers CI checks for TS, Rust, and Go.
*   [x] CI fails on lint/type/test/build regressions.
*   [x] Workflow artifacts and logs are clear enough for debugging.
*   [x] Release pipeline placeholders are present for future signing and updater stages.

## Business Rules & Logic
*   CI is the minimum quality gate before merge.
*   Public release trust requirements (signing stage) must be represented as planned gate.

## Dependencies
*   Depends on: KP-003, KP-005, KP-006, KP-008

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing (workflow lint checks)
*   [x] Lint/Type check clear
