# KP-040: Windows Process Scanner Core (allowlist match)

**Epic:** EPIC-DETECTION
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 5
**Priority:** Must

---

## Objective
Implement Windows process scanner core using allowlist executable matching only.

## Technical Specifications
*   **Proposed Files:** `crates/client-engine/src/detection/scanner_windows.rs`
*   **Functions/Classes:** `scan_processes()`, `match_allowlist()`
*   **API Endpoints:** N/A
*   **Data Models:** Detection match model

## Acceptance Criteria (Technical)
*   [x] Scanner enumerates running processes safely.
*   [x] Matching is restricted to allowlist entries.
*   [x] Unknown processes are ignored without false positives.
*   [x] Scanner handles permission limitations gracefully.

## Business Rules & Logic
*   Process-external detection only; no injection behavior.

## Dependencies
*   Depends on: KP-005, KP-039

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
