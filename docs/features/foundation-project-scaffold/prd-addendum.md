# PRD Addendum - Foundation Project Scaffold
> Feature: [FOUNDATION] Project Scaffold
> Date: 2026-05-03
> Status: In Progress

---

## 1. Feature Metadata

| Field | Value |
|---|---|
| Feature Name | Foundation Project Scaffold |
| Parent Epic | Foundation |
| Status | In Progress |
| Owner | Product Owner / Solo Developer |
| Related Global Docs | docs/prd.md, docs/tech-stack.md, docs/fsd.md |

---

## 2. Requirements

| ID | User Story | Acceptance Criteria |
|---|---|---|
| US-SCF-01 | As a developer, I want a clear monorepo structure so that frontend, Rust engine, and Go relay service can evolve independently without confusion. | Given repo is initialized, when source folders are created, then workspace has deterministic structure for desktop app, core engine, relay controller, docs, and scripts. |
| US-SCF-02 | As a developer, I want reproducible local setup commands so that onboarding and daily development are fast. | Given fresh machine setup, when bootstrap command is executed, then required dependencies and run targets are documented and executable. |
| US-SCF-03 | As a maintainer, I want environment variable templates and secret boundaries so that no sensitive values are committed. | Given repo scaffold is created, when env files are prepared, then `.env.example` is present and secret files are gitignored. |
| US-SCF-04 | As a maintainer, I want baseline CI workflows so that lint/test/build checks run consistently before merge. | Given pull request is opened, when CI executes, then baseline checks run for TypeScript, Rust, and Go modules. |
| US-SCF-05 | As a maintainer, I want release pipeline placeholders so that updater, signing, and contract packaging can be integrated safely. | Given release workflow exists, when stable release branch is tagged, then pipeline has explicit stages for signing and artifact publishing. |

---

## 3. ERD Delta

No new data entities are introduced in this scaffold feature.

Implementation note:
- Data model implementation in later features SHALL follow `docs/erd/core-erd.md`.

---

## 4. API Contract Delta

No new endpoint behavior is introduced in this scaffold feature.

Implementation note:
- Service skeletons SHALL be wired to consume contracts under `docs/api/contracts/*.yaml` in subsequent features.

---

## 5. Integration Notes

- Scaffold MUST align with approved stack in `docs/tech-stack.md`:
  - Tauri + React + TypeScript for desktop UI.
  - Rust for client network engine.
  - Go for relay controller.
- Guardrails from global docs apply immediately:
  - No process injection patterns.
  - Privacy-by-default for telemetry boundaries.
  - Signed manifest verification integration point reserved in architecture skeleton.
- This feature is foundational only; no production routing behavior is expected yet.

---

## 6. Open Questions

| ID | Question | Priority |
|---|---|---|
| SCF-OQ-01 | Final workspace package manager strategy (pnpm workspace only or hybrid tooling)? | Medium |
| SCF-OQ-02 | Preferred CI trigger matrix for heavy jobs (every PR vs protected branches only)? | Medium |
| SCF-OQ-03 | Signing service integration target for pre-launch pipeline (provider lock-in decision)? | High |
