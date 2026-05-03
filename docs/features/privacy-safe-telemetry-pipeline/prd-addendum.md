# PRD Addendum - Privacy-Safe Telemetry Pipeline
> Feature: Privacy-Safe Telemetry Pipeline
> Date: 2026-05-04
> Status: In Progress

---

## 1. Feature Metadata

| Field | Value |
|---|---|
| Feature Name | Privacy-Safe Telemetry Pipeline |
| Parent Epic | Product Features |
| Status | In Progress |
| Owner | Product Owner / Solo Developer |
| Related Global Docs | docs/prd.md, docs/fsd.md, docs/api-standards.md, docs/api/contracts/telemetry-ingest.v1.yaml |

---

## 2. Requirements

| ID | User Story | Acceptance Criteria |
|---|---|---|
| US-TP-01 | As a privacy-conscious user, I want telemetry payloads validated against strict allowlist so sensitive data is never transmitted. | Given event payloads are produced, when validation runs, then only allowed schema fields pass and invalid fields are rejected/dropped by policy. |
| US-TP-02 | As a privacy-conscious user, I want PII/sensitive field scrubbing so accidental private data is never queued or sent. | Given payload includes prohibited fields, when scrubber runs, then payload is rejected or sanitized per policy and logged as non-sensitive violation. |
| US-TP-03 | As a maintainer, I want reliable batch queueing so transient network failures do not drop all telemetry immediately. | Given telemetry endpoint is unavailable, when send fails, then events are batched with retry budget and expiry policy. |
| US-TP-04 | As a maintainer, I want delivery idempotency and backpressure handling so pipeline remains stable under high event volume. | Given repeated send attempts or high queue size, when client processes batches, then idempotency key and bounded queue policy prevent duplication and runaway growth. |
| US-TP-05 | As a product owner, I want telemetry pipeline health visibility so event quality and delivery reliability can be monitored. | Given pipeline operates, when status is inspected, then queue depth, drop counts, retry counts, and last delivery outcome are available. |

---

## 3. ERD Delta

No new entities are mandatory.

Data usage notes:
- Reuse `telemetry_batch` and `telemetry_event`.
- Optional migration may extend metadata fields for retry diagnostics if required.

---

## 4. API Contract Delta

No new endpoint is introduced.

Feature consumes existing telemetry ingest contract:
- `POST /v1/telemetry/events:batch`

---

## 5. Integration Notes

- All feature event producers must pass through the same validation/scrubbing pipeline.
- Invalid telemetry must fail-safe without blocking core routing/gameplay flows.
- Delivery layer must preserve privacy policy and avoid raw sensitive field persistence.
- Retry/expiry policies should be deterministic and testable.

---

## 6. Open Questions

| ID | Question | Priority |
|---|---|---|
| TP-OQ-01 | Reject vs sanitize default behavior for unknown nested keys in v1? | Medium |
| TP-OQ-02 | Max queue depth limit before dropping low-priority events? | Medium |
| TP-OQ-03 | Should telemetry health summary be surfaced in hidden diagnostics UI in v1? | Low |
