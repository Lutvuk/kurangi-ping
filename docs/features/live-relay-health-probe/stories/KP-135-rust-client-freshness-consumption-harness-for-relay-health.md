# KP-135: Rust Client Freshness Consumption Harness for `/v1/relay/health`

**Epic:** EPIC-RELAY
**Layer:** L5-integration
**Role:** Fullstack
**Estimation:** 3
**Priority:** Should

---

## Objective
Add client-engine side harness that validates fresh relay health snapshots from controller are consumable by existing relay health polling/failover path.

## Technical Specifications
*   **Proposed Files:** `crates/client-engine/tests/relay_input_contract_test.rs`, `crates/client-engine/tests/routing_fallback_harness.rs`
*   **Functions/Classes:** `relay_health_freshness_consumption_harness`
*   **API Endpoints:** `[GET] /v1/relay/health`
*   **Data Models:** Relay health polling snapshot model

## Acceptance Criteria (Technical)
*   [x] Harness ingests controller-like relay health payloads with dynamic `updated_at` values.
*   [x] Freshness-sensitive flow in client engine accepts live updates without schema adaptation.
*   [x] Repeated polling cycles remain deterministic with dynamic latency/status changes.
*   [x] Contract compatibility remains intact with no parser regressions.

## Business Rules & Logic
*   Rust client must consume fresh relay snapshots safely to support reliable failover decisions.

## Dependencies
*   Depends on: KP-132, KP-048

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
