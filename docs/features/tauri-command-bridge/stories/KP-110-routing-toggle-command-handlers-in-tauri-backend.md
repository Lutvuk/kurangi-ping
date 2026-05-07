# KP-110: Routing Toggle Command Handlers in Tauri Backend

**Epic:** EPIC-WIRING
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 3
**Priority:** Must

---

## Objective
Implement Tauri command handlers that bridge frontend toggle actions to Rust routing lifecycle orchestration.

## Technical Specifications
*   **Proposed Files:** `apps/desktop/src-tauri/src/main.rs`, `apps/desktop/src-tauri/src/ipc/routing_commands.rs`
*   **Functions/Classes:** `routing_toggle_on`, `routing_toggle_off`
*   **API Endpoints:** N/A
*   **Data Models:** Routing command request/response contract

## Acceptance Criteria (Technical)
*   [x] `routing_toggle_on` command invokes Rust routing lifecycle ON path.
*   [x] `routing_toggle_off` command invokes Rust routing lifecycle OFF path.
*   [x] Command responses return normalized state payload usable by frontend presenter.
*   [x] Recoverable backend failures are mapped to UI-safe reason codes.

## Business Rules & Logic
*   Toggle actions must stay idempotent and safe under repeated clicks.

## Dependencies
*   Depends on: KP-109, KP-058, KP-059, KP-060

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
