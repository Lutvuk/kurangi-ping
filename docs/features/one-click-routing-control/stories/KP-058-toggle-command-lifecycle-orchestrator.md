# KP-058: Toggle Command Lifecycle Orchestrator

**Epic:** EPIC-TOGGLE
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 4
**Priority:** Must

---

## Objective
Implement central orchestrator for ON/OFF lifecycle commands and state transitions.

## Technical Specifications
*   **Proposed Files:** `crates/client-engine/src/routing/toggle_orchestrator.rs`
*   **Functions/Classes:** `handle_toggle_command()`, `ToggleOrchestrator`
*   **API Endpoints:** N/A
*   **Data Models:** Toggle command + lifecycle state model

## Acceptance Criteria (Technical)
*   [ ] Orchestrator supports ON and OFF command paths.
*   [ ] Lifecycle transitions are explicit and validated.
*   [ ] Illegal transitions are rejected safely.
*   [ ] Orchestrator exposes deterministic result contract.

## Business Rules & Logic
*   One-click behavior must remain predictable under all command timings.

## Dependencies
*   Depends on: KP-034

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
