# KP-038: Supported Games Allowlist Schema Refinement

**Epic:** EPIC-DETECTION
**Layer:** L1-data
**Role:** Backend
**Estimation:** 2
**Priority:** Must

---

## Objective
Refine DB schema fields supporting executable-based allowlist matching and metadata validity.

## Technical Specifications
*   **Proposed Files:** `db/migrations/0004_supported_games_refine.up.sql`
*   **Functions/Classes:** N/A
*   **API Endpoints:** N/A
*   **Data Models:** `supported_games`

## Acceptance Criteria (Technical)
*   [x] Allowlist fields required for matching are present and constrained.
*   [x] Existing seed compatibility is preserved.
*   [x] Migration is forward-only and idempotent on replay.
*   [x] Schema notes updated in DB docs.

## Business Rules & Logic
*   Detection accuracy starts from strict allowlist data quality.

## Dependencies
*   Depends on: KP-011, KP-012

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
