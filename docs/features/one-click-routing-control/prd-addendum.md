# PRD Addendum - One-Click Routing Control
> Feature: One-Click Routing Control (ON/OFF lifecycle)
> Date: 2026-05-04
> Status: In Progress

---

## 1. Feature Metadata

| Field | Value |
|---|---|
| Feature Name | One-Click Routing Control |
| Parent Epic | Product Features |
| Status | In Progress |
| Owner | Product Owner / Solo Developer |
| Related Global Docs | docs/prd.md, docs/fsd.md, docs/tech-stack.md, docs/design-system.md |

---

## 2. Requirements

| ID | User Story | Acceptance Criteria |
|---|---|---|
| US-OC-01 | As a user, I want one-click ON flow so routing starts without manual network setup. | Given pre-checks pass, when user presses ON, then activation pipeline completes and state becomes active. |
| US-OC-02 | As a user, I want one-click OFF flow so routing stops safely at any time. | Given route is active, when user presses OFF, then route teardown completes and state returns idle. |
| US-OC-03 | As a user, I want stable behavior under rapid toggling so accidental double-click does not break state. | Given repeated toggle input, when commands overlap, then idempotency and command serialization prevent inconsistent lifecycle state. |
| US-OC-04 | As a user, I want transparent lifecycle status so I understand whether app is connecting, active, disconnecting, or failed. | Given lifecycle transitions, when state changes, then UI receives normalized status values (`idle`, `arming`, `active`, `disarming`, `error`). |
| US-OC-05 | As a product owner, I want ON/OFF telemetry events so activation behavior and failure reasons can be analyzed. | Given ON/OFF operations complete or fail, when events emit, then payloads follow telemetry schema and include reason codes. |

---

## 3. ERD Delta

No new entities are required.

Data usage notes:
- Reuse `route_session` lifecycle fields.
- Reuse telemetry event stream for ON/OFF lifecycle analytics.

---

## 4. API Contract Delta

No new endpoint is introduced.

Feature uses existing relay/telemetry flows and local engine orchestration.

---

## 5. Integration Notes

- ON flow must respect detection gate and manifest verification gate.
- OFF flow must guarantee safe teardown even under partial failures.
- Toggle UI state must mirror backend lifecycle state machine.
- Command handling must be serialized and idempotent.

---

## 6. Open Questions

| ID | Question | Priority |
|---|---|---|
| OC-OQ-01 | Should ON/OFF command queue be single-slot (latest-wins) or strict FIFO in v1? | Medium |
| OC-OQ-02 | How long should arming/disarming timeout be before transitioning to error? | Medium |
| OC-OQ-03 | Should OFF failure auto-retry once or require explicit user retry? | Low |
