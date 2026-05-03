# PRD Addendum - Foundation Base DB Schema
> Feature: [FOUNDATION] Base DB Schema
> Date: 2026-05-03
> Status: In Progress

---

## 1. Feature Metadata

| Field | Value |
|---|---|
| Feature Name | Foundation Base DB Schema |
| Parent Epic | Foundation |
| Status | In Progress |
| Owner | Product Owner / Solo Developer |
| Related Global Docs | docs/erd/core-erd.md, docs/fsd.md, docs/prd.md |

---

## 2. Requirements

| ID | User Story | Acceptance Criteria |
|---|---|---|
| US-DB-01 | As a developer, I want a baseline SQLite schema generated from Core ERD so that core entities are ready for implementation. | Given migration runs on clean DB, when init migration executes, then all core tables from `core-erd.md` are created successfully. |
| US-DB-02 | As a developer, I want strong constraints so that invalid state is rejected at DB layer. | Given inserts/updates run, when values violate FK/CHECK constraints, then DB rejects writes with deterministic errors. |
| US-DB-03 | As a developer, I want baseline indexes so that session and telemetry access paths remain performant. | Given query paths for session metrics and telemetry queue, when EXPLAIN plan is checked, then indexed access exists for primary lookup/filter fields. |
| US-DB-04 | As a developer, I want seed data for supported games so that detection features can start integration without manual DB edits. | Given fresh DB, when seed script runs, then initial supported game rows exist and are idempotent. |
| US-DB-05 | As a maintainer, I want migration/versioning scripts so that schema changes are reproducible across environments. | Given migration CLI/script runs, when applying and rolling forward versions, then schema version tracking is persisted and deterministic. |

---

## 3. ERD Delta

No new entities are added. This feature physicalizes Core ERD into SQLite.

Physicalization rules:
- SQLite foreign key enforcement must be enabled (`PRAGMA foreign_keys = ON`).
- Enum semantics are enforced via `CHECK` constraints.
- Timestamps are persisted as ISO-8601 UTC text.
- Telemetry tables must not include PII columns.

---

## 4. API Contract Delta

No new API endpoints are introduced.

Impact:
- DB baseline supports existing contract payload requirements in:
  - `docs/api/contracts/relay-public.v1.yaml`
  - `docs/api/contracts/telemetry-ingest.v1.yaml`
  - `docs/api/contracts/relay-admin.v1.yaml`

---

## 5. Integration Notes

- Schema implementation MUST align with `docs/erd/core-erd.md` entity/relationship definitions.
- Baseline retention strategy should prepare ring-buffer behavior for metric/event tables.
- Migration scripts should be automation-friendly for CI and local setup.

---

## 6. Open Questions

| ID | Question | Priority |
|---|---|---|
| DB-OQ-01 | Final retention threshold for `ping_samples` rows per installation/session? | Medium |
| DB-OQ-02 | Preferred migration toolchain (SQL-only scripts vs wrapper CLI) in monorepo? | Medium |
| DB-OQ-03 | At what stage should telemetry archival/compaction jobs be introduced? | Low |
