# KP-100: Update Package Verification and Apply Orchestrator

**Epic:** EPIC-UPDATER
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 5
**Priority:** Must

---

## Objective
Orchestrate safe update download/verify/apply flow with recoverable failure handling.

## Technical Specifications
*   **Proposed Files:** `crates/client-engine/src/updater/apply_orchestrator.rs`
*   **Functions/Classes:** `check_for_update()`, `download_update()`, `apply_update()`
*   **API Endpoints:** Updater release source metadata/feed
*   **Data Models:** Update operation result model

## Acceptance Criteria (Technical)
*   [x] Update package verification occurs before apply.
*   [x] Failed downloads/applies move to recoverable error state.
*   [x] Core app remains operational after updater failure.
*   [x] Apply flow is deterministic and test-covered.

## Business Rules & Logic
*   Update safety is higher priority than update speed.

## Dependencies
*   Depends on: KP-099

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
