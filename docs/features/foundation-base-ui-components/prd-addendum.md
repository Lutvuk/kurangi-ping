# PRD Addendum - Foundation Base UI Components
> Feature: [FOUNDATION] Base UI Components
> Date: 2026-05-03
> Status: In Progress

---

## 1. Feature Metadata

| Field | Value |
|---|---|
| Feature Name | Foundation Base UI Components |
| Parent Epic | Foundation |
| Status | In Progress |
| Owner | Product Owner / Solo Developer |
| Related Global Docs | docs/design-system.md, docs/prd.md, docs/fsd.md |

---

## 2. Requirements

| ID | User Story | Acceptance Criteria |
|---|---|---|
| US-UI-01 | As a developer, I want token-backed primitive components so that all future screens stay visually consistent. | Given primitive components are rendered, when inspected, then colors/spacing/typography/radius are sourced from approved design tokens. |
| US-UI-02 | As a developer, I want foundation layout components so that feature modules can plug into stable shell regions. | Given AppShell is mounted, when viewport changes between compact/full breakpoints, then layout regions adapt as specified. |
| US-UI-03 | As a user, I want operational status components that are readable instantly so that I can act quickly before gameplay. | Given state changes occur (off/connecting/on/degraded), when status components update, then semantic colors and text states match design rules. |
| US-UI-04 | As a developer, I want accessibility baseline in interactive components so that keyboard users can operate core UI safely. | Given focus and keyboard navigation are used, when interacting with controls, then focus visibility, tab order, and ARIA labels are present. |
| US-UI-05 | As a maintainer, I want reusable typed component APIs so that feature implementation can wire data without refactoring primitives. | Given future feature pages consume components, when props are passed, then TypeScript contracts are explicit and stable. |

---

## 3. ERD Delta

No new entities are introduced.

UI components only consume domain semantics already defined by existing documents.

---

## 4. API Contract Delta

No new endpoints are introduced.

UI component contracts remain API-agnostic and rely on typed view-model props.

---

## 5. Integration Notes

- All components MUST follow `docs/design-system.md` and `docs/design-system.yaml` rules.
- Signal/probe color semantics are strict and non-decorative.
- Component library should avoid stylistic drift from approved visual language.
- Foundation UI is implementation-ready baseline, not full feature wiring.

---

## 6. Open Questions

| ID | Question | Priority |
|---|---|---|
| UI-OQ-01 | Preferred component folder taxonomy (`primitives/`, `modules/`, `patterns/`) final naming? | Low |
| UI-OQ-02 | Snapshot testing scope for visual regressions in this phase? | Medium |
| UI-OQ-03 | At what milestone should storybook (or equivalent) be introduced? | Low |
