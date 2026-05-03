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
*   [ ] Update package verification occurs before apply.
*   [ ] Failed downloads/applies move to recoverable error state.
*   [ ] Core app remains operational after updater failure.
*   [ ] Apply flow is deterministic and test-covered.

## Business Rules & Logic
*   Update safety is higher priority than update speed.

## Dependencies
*   Depends on: KP-099

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
