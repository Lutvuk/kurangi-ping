# PRD Addendum - First-Run Onboarding Flow
> Feature: First-Run Onboarding Flow
> Date: 2026-05-04
> Status: In Progress

---

## 1. Feature Metadata

| Field | Value |
|---|---|
| Feature Name | First-Run Onboarding Flow |
| Parent Epic | Product Features |
| Status | In Progress |
| Owner | Product Owner / Solo Developer |
| Related Global Docs | docs/prd.md, docs/fsd.md, docs/design-system.md, docs/tech-stack.md |

---

## 2. Requirements

| ID | User Story | Acceptance Criteria |
|---|---|---|
| US-OB-01 | As a first-time user, I want a short onboarding flow so I can reach usable state quickly. | Given first app launch, when onboarding starts, then user is guided through 5 concise steps without account/signup friction. |
| US-OB-02 | As a user, I want permission and environment checks surfaced early so setup failures are understandable. | Given permission or environment constraints exist, when checks run, then onboarding shows actionable recovery guidance and retry options. |
| US-OB-03 | As a user, I want onboarding progress saved so interrupted sessions can resume from the correct step. | Given onboarding is interrupted, when app relaunches, then progress resumes from last completed checkpoint. |
| US-OB-04 | As a user, I want onboarding completion tied to first successful connection so completion reflects real readiness. | Given first route activation succeeds, when onboarding final step resolves, then `onboarding_state` is set to completed. |
| US-OB-05 | As a product owner, I want completion telemetry so activation funnel performance can be measured. | Given onboarding completes, when event emits, then `onboarding_completed` payload includes duration and success path fields with schema compliance. |

---

## 3. ERD Delta

No new entities are required.

Data usage notes:
- Reuse `user_settings.onboarding_state` for progress lifecycle.
- Reuse session/event data for completion analytics.

---

## 4. API Contract Delta

No new endpoint is introduced.

Feature consumes existing relay health and telemetry contract flows.

---

## 5. Integration Notes

- Onboarding sequence orchestrates detection, relay health checks, and one-click activation flows.
- UI must remain concise, accessible, and state-aware.
- Skip/fail paths must keep app in safe recoverable state.
- Completion status and telemetry must be deterministic.

---

## 6. Open Questions

| ID | Question | Priority |
|---|---|---|
| OB-OQ-01 | Final per-step timeout thresholds for auto-check steps? | Medium |
| OB-OQ-02 | Should users be allowed to skip relay test step in beta builds? | Low |
| OB-OQ-03 | Should onboarding completion allow partial state if connect succeeds but telemetry fails? | Medium |
