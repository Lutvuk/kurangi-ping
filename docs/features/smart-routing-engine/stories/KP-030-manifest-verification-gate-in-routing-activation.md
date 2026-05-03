# KP-030: Manifest Verification Gate in Routing Activation

**Epic:** EPIC-ROUTING
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 4
**Priority:** Must

---

## Objective
Enforce signed manifest verification before any route candidate can be used for activation.

## Technical Specifications
*   **Proposed Files:** `crates/client-engine/src/routing/manifest_gate.rs`, `crates/client-engine/src/security/signature.rs`
*   **Functions/Classes:** `verify_manifest_or_fail()`, `ManifestGateResult`
*   **API Endpoints:** Consumes `[GET] /v1/relay/manifest`
*   **Data Models:** Manifest payload DTO

## Acceptance Criteria (Technical)
*   [ ] Invalid signature blocks route activation.
*   [ ] Expired manifest blocks route activation.
*   [ ] Valid manifest passes and exposes candidate set.
*   [ ] Error path emits deterministic failure code for UI/telemetry.

## Business Rules & Logic
*   No unsigned relay metadata may reach route selection stage.

## Dependencies
*   Depends on: KP-029

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
