# KP-039: Seed and Versioning for Supported Game Catalog

**Epic:** EPIC-DETECTION
**Layer:** L1-data
**Role:** Backend
**Estimation:** 2
**Priority:** Must

---

## Objective
Add version-aware seed mechanism for supported game catalog evolution.

## Technical Specifications
*   **Proposed Files:** `db/seeds/0002_supported_games_catalog.sql`, `db/seeds/catalog_version.json`
*   **Functions/Classes:** `ApplySupportedGameCatalogSeed`
*   **API Endpoints:** N/A
*   **Data Models:** `supported_games`

## Acceptance Criteria (Technical)
*   [x] Seed updates are idempotent and version-tracked.
*   [x] Existing rows update safely without duplication.
*   [x] Seed supports enabling/disabling titles explicitly.
*   [x] Seed output reports applied catalog version.

## Business Rules & Logic
*   Catalog updates must remain controlled and auditable.

## Dependencies
*   Depends on: KP-038

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
