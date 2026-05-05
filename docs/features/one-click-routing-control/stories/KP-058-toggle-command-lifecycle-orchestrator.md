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
*   [x] Orchestrator supports ON and OFF command paths.
*   [x] Lifecycle transitions are explicit and validated.
*   [x] Illegal transitions are rejected safely.
*   [x] Orchestrator exposes deterministic result contract.

## Business Rules & Logic
*   One-click behavior must remain predictable under all command timings.

## Dependencies
*   Depends on: KP-034

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
