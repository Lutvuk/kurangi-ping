# KP-014: Supported Games Seed Script (Idempotent)

**Epic:** EPIC-FOUNDATION
**Layer:** L1-data
**Role:** Backend
**Estimation:** 2
**Priority:** Should

---

## Objective
Create idempotent seed script for baseline supported games to unblock game detection integration.

## Technical Specifications
*   **Proposed Files:** `db/seeds/0001_supported_games.sql`, `scripts/db-seed.ps1`
*   **Functions/Classes:** `Invoke-DbSeed`
*   **API Endpoints:** N/A
*   **Data Models:** `supported_games`

## Acceptance Criteria (Technical)
*   [ ] Seed inserts initial supported games rows.
*   [ ] Seed can be re-run safely without duplicates.
*   [ ] Seed aligns with detection allowlist conventions.
*   [ ] Script output clearly reports inserted/skipped counts.

## Business Rules & Logic
*   Seed data is baseline only and can be extended later.
*   No hardcoded user-specific values.

## Dependencies
*   Depends on: KP-011

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
