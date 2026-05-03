# KP-071: Metrics Ring-Buffer Persistence Adapter

**Epic:** EPIC-METRICS
**Layer:** L3-backend
**Role:** Backend
**Estimation:** 3
**Priority:** Should

---

## Objective
Persist recent metric samples into ring-buffer storage with bounded retention.

## Technical Specifications
*   **Proposed Files:** `crates/client-engine/src/db/metrics_repo.rs`
*   **Functions/Classes:** `append_metric_sample()`, `prune_old_samples()`
*   **API Endpoints:** N/A
*   **Data Models:** `ping_sample` persistence mapping

## Acceptance Criteria (Technical)
*   [ ] Samples persist with session context and timestamp.
*   [ ] Retention cap enforces oldest-first eviction.
*   [ ] Persistence failure does not crash probe loop.
*   [ ] Query helper supports recent trend retrieval.

## Business Rules & Logic
*   Short history supports diagnostics without long-term heavy storage.

## Dependencies
*   Depends on: KP-069, KP-015

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
