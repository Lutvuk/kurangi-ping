# KP-091: Resume Checkpoint Persistence (onboarding_state)

**Epic:** EPIC-ONBOARDING
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 3
**Priority:** Must

---

## Objective
Persist onboarding progress checkpoints so flow can resume after app interruption.

## Technical Specifications
*   **Proposed Files:** `crates/client-engine/src/db/onboarding_repo.rs`
*   **Functions/Classes:** `save_onboarding_checkpoint()`, `load_onboarding_checkpoint()`
*   **API Endpoints:** N/A
*   **Data Models:** `user_settings.onboarding_state` mapping

## Acceptance Criteria (Technical)
*   [x] Progress checkpoint persists after each completed step.
*   [x] Reload restores correct next step on restart.
*   [x] Corrupt/unknown state falls back to safe restart strategy.
*   [x] Persistence failures are surfaced without crash.

## Business Rules & Logic
*   Users should not repeat already completed onboarding steps unnecessarily.

## Dependencies
*   Depends on: KP-089, KP-015

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
