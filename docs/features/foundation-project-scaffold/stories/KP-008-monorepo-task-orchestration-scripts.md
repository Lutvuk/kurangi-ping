# KP-008: Monorepo Task Orchestration Scripts (dev/build/test)

**Epic:** EPIC-FOUNDATION
**Layer:** L5-integration
**Role:** Fullstack
**Estimation:** 2
**Priority:** Must

---

## Objective
Provide unified command orchestration for running, building, and testing desktop UI, Rust engine, and Go relay service.

## Technical Specifications
*   **Proposed Files:** `package.json` (workspace scripts) or `Makefile`, `scripts/dev.ps1`, `scripts/test.ps1`, `scripts/build.ps1`
*   **Functions/Classes:** Script entry points
*   **API Endpoints:** N/A
*   **Data Models:** N/A

## Acceptance Criteria (Technical)
*   [x] Single command exists for local dev startup flow.
*   [x] Single command exists for full test flow (TS/Rust/Go).
*   [x] Single command exists for build artifacts generation.
*   [x] Script docs describe prerequisites and expected outputs.

## Business Rules & Logic
*   Developer velocity and repeatability are primary goals.
*   Commands must fail fast with clear messages.

## Dependencies
*   Depends on: KP-003, KP-005, KP-006, KP-007

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing (script smoke tests)
*   [x] Lint/Type check clear
