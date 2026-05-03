# KP-002: Environment Contract and Secrets Boundary

**Epic:** EPIC-FOUNDATION
**Layer:** L1-data
**Role:** DevOps
**Estimation:** 2
**Priority:** Must

---

## Objective
Define environment variable contracts and ensure secret handling boundaries are enforced from day one.

## Technical Specifications
*   **Proposed Files:** `.env.example`, `.gitignore`, `docs/setup/environment.md`
*   **Functions/Classes:** N/A
*   **API Endpoints:** N/A
*   **Data Models:** N/A

## Acceptance Criteria (Technical)
*   [x] `.env.example` contains approved variables (`KP_ENV`, `KP_MANIFEST_URL`, `KP_MANIFEST_PUBKEY`, `KP_TELEMETRY_ENDPOINT`, `KP_UPDATE_CHANNEL`, `KP_RELAY_HEALTH_URL`).
*   [x] Secret-bearing env files are gitignored.
*   [x] Setup documentation explains required env variables and safe local setup.
*   [x] No hardcoded secret values are committed.

## Business Rules & Logic
*   Privacy and key material safety are non-negotiable.
*   Developer onboarding should remain simple and explicit.

## Dependencies
*   Depends on: KP-001

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing (N/A)
*   [x] Lint/Type check clear (N/A)
