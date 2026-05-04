# KP-041: Detection State Resolver and Freshness Window

**Epic:** EPIC-DETECTION
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 3
**Priority:** Must

---

## Objective
Resolve normalized detection states and stale timeout behavior from raw scan outcomes.

## Technical Specifications
*   **Proposed Files:** `crates/client-engine/src/detection/state_resolver.rs`
*   **Functions/Classes:** `resolve_detection_state()`, `is_stale()`
*   **API Endpoints:** N/A
*   **Data Models:** `DetectionState` enum

## Acceptance Criteria (Technical)
*   [x] States map to `not_found`, `detected`, `stale`, `error`.
*   [x] Freshness window logic is deterministic and configurable.
*   [x] Resolver output carries metadata for UI rendering.
*   [x] Error states include non-sensitive reason codes.

## Business Rules & Logic
*   UI and routing gate rely on stable state semantics.

## Dependencies
*   Depends on: KP-040

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
