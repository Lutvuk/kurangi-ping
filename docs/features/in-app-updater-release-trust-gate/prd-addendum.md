# PRD Addendum - In-App Updater + Release Trust Gate
> Feature: In-App Updater + Release Trust Gate (code-signing)
> Date: 2026-05-04
> Status: In Progress

---

## 1. Feature Metadata

| Field | Value |
|---|---|
| Feature Name | In-App Updater + Release Trust Gate |
| Parent Epic | Product Features |
| Status | In Progress |
| Owner | Product Owner / Solo Developer |
| Related Global Docs | docs/prd.md, docs/tech-stack.md, docs/fsd.md, docs/api-standards.md |

---

## 2. Requirements

| ID | User Story | Acceptance Criteria |
|---|---|---|
| US-UP-01 | As a user, I want the app to check updates in-app so I can stay current without manual downloads. | Given update channel is configured, when update check runs, then app determines whether a newer valid release exists. |
| US-UP-02 | As a user, I want safe update apply flow so app updates do not corrupt my installation. | Given update package is available, when user applies update, then download/verify/install flow completes or fails with recoverable state. |
| US-UP-03 | As a user, I want clear updater states so I understand what is happening during update operations. | Given update lifecycle changes, when state transitions occur, then UI reflects normalized states and actionable messages. |
| US-UP-04 | As a maintainer, I want stable release artifacts blocked when unsigned so trust policy is enforced. | Given release pipeline runs for stable channel, when signing stage fails or artifact unsigned, then publish gate fails. |
| US-UP-05 | As a product owner, I want update telemetry so adoption and update failure patterns can be monitored. | Given update checks/downloads/applies occur, when telemetry emits, then event payloads remain schema-compliant and privacy-safe. |

---

## 3. ERD Delta

No new entities are mandatory.

Data usage notes:
- Reuse settings for update channel and optional last-check metadata.
- Reuse telemetry stream for updater lifecycle analytics.

---

## 4. API Contract Delta

No public product API endpoint is introduced.

Release/update metadata is consumed from configured updater release source.

---

## 5. Integration Notes

- Must align with `tauri-plugin-updater` integration path from tech-stack decisions.
- Stable releases require signed artifact gate in CI.
- Update errors must be recoverable and must not disrupt core routing functionality.
- Updater lifecycle telemetry must follow schema allowlist rules.

---

## 6. Open Questions

| ID | Question | Priority |
|---|---|---|
| UP-OQ-01 | Update check cadence default for stable vs beta channels? | Medium |
| UP-OQ-02 | Should update apply be auto-download + prompt restart or explicit manual user trigger in v1? | Medium |
| UP-OQ-03 | Should failed update rollback telemetry include anonymized error category taxonomy? | Low |
