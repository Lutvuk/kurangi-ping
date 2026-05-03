# PRD Addendum - Supported Game Detection
> Feature: Supported Game Detection (allowlist process scan)
> Date: 2026-05-04
> Status: In Progress

---

## 1. Feature Metadata

| Field | Value |
|---|---|
| Feature Name | Supported Game Detection |
| Parent Epic | Product Features |
| Status | In Progress |
| Owner | Product Owner / Solo Developer |
| Related Global Docs | docs/prd.md, docs/fsd.md, docs/tech-stack.md, docs/design-system.md |

---

## 2. Requirements

| ID | User Story | Acceptance Criteria |
|---|---|---|
| US-GD-01 | As a user, I want the app to detect supported games automatically so I do not need manual process configuration. | Given a supported game process is running, when scan loop executes, then game is marked detected within configured scan interval. |
| US-GD-02 | As a user, I want to trigger rescan manually so I can refresh detection state instantly. | Given detection UI is open, when user clicks rescan, then process scan executes and status updates immediately. |
| US-GD-03 | As a maintainer, I want allowlist-based validation so false-positive detection remains low. | Given unknown process names, when scanner runs, then only allowlisted executable identifiers are accepted. |
| US-GD-04 | As a product owner, I want normalized detection states so UI and onboarding behave consistently. | Given scan outcomes vary, when state is emitted, then output is one of `not_found`, `detected`, `stale`, `error` with clear metadata. |
| US-GD-05 | As a product owner, I want telemetry events for detection so activation funnel analysis is measurable. | Given detection success event occurs, when telemetry is emitted, then `game_detected` payload follows allowlisted schema. |

---

## 3. ERD Delta

No new entities are required.

Data usage notes:
- Reuse `supported_games` for detection allowlist source.
- Detection outcomes may optionally link to `route_session` when routing starts.

---

## 4. API Contract Delta

No new API endpoints are introduced.

Feature relies on local allowlist persistence and does not require network contract changes.

---

## 5. Integration Notes

- Scanner MUST remain process-external (anti-cheat-safe).
- Detection output gates routing activation flow.
- UI integration should align with `GameDetectionRow` and onboarding flow patterns.
- Telemetry emission must comply with event schema policy.

---

## 6. Open Questions

| ID | Question | Priority |
|---|---|---|
| GD-OQ-01 | Should executable hash verification be introduced in v1 or deferred? | Medium |
| GD-OQ-02 | Scan interval default for balancing responsiveness vs CPU overhead? | Medium |
| GD-OQ-03 | Should stale-state timeout be configurable in settings v1? | Low |
