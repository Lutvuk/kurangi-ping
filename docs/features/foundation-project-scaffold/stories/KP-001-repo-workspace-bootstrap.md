# KP-001: Repo Workspace Bootstrap

**Epic:** EPIC-FOUNDATION
**Layer:** L1-data
**Role:** DevOps
**Estimation:** 3
**Priority:** Must

---

## Objective
Establish the initial monorepo scaffold and deterministic directory layout for desktop app, Rust engine, Go relay service, docs, and automation scripts.

## Technical Specifications
*   **Proposed Files:** `apps/desktop/`, `apps/relay-controller/`, `crates/client-engine/`, `scripts/`, `docs/`
*   **Functions/Classes:** N/A (scaffold and folder conventions)
*   **API Endpoints:** N/A
*   **Data Models:** N/A

## Acceptance Criteria (Technical)
*   [x] Root workspace directories exist and follow documented layout.
*   [x] Directory naming conventions are consistent with tech-stack document.
*   [x] Placeholder README exists in each top-level module.
*   [x] Structure is documented in root README.

## Business Rules & Logic
*   Preserve clear separation: UI, client engine, relay service.
*   Scaffold should be minimal and avoid production logic.

## Dependencies
*   Depends on: None

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing (N/A for scaffold)
*   [x] Lint/Type check clear (N/A for scaffold)
