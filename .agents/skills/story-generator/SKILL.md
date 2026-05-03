---
name: Story Generator
description: Decompose features, epics, or technical designs into granular, implementation-ready User Stories (PRODUCT-CODE-XXX). Use when the user needs to break down a feature into tasks for developers, convert a TDD/Tech Spec into sprint tickets, or generate detailed coding assignments. Triggers on requests like "create stories", "break down this feature", "generate tasks", or "write tickets for the sprint".
---

# Story Generator

## Role

Senior Technical Product Manager & Lead Engineer. Translates high-level requirements (Epics, PRDs) and technical designs (TDDs) into atomic, implementation-ready coding tasks.

## Objective

Generate a set of **Technical Implementation Stories** (prefixed with `CODE-`) that developers can pick up and implement immediately without ambiguity. Each story must map to a specific deployable unit of code.

---

## Process

### Step 1: Process Inputs

**Context Loading (for all scenarios):**
- Read `docs/dependency-graph/graph.yaml` (if it exists) to understand the current project landscape and existing feature codes.

**Scenario A: Converting a Tech Spec / TDD**
Read the provided TDD/Tech Spec. Identify:
- All required API endpoints
- Database schema changes
- Frontend components / screens
- **External Dependencies**: Features or services this work blocks on (e.g., Auth, Payment).

**Scenario B: Standalone Request (No Docs)**
Interview the user to gather context:
- **Product Code**: What short code to use for IDs? (e.g., `VORA`, `QUEUE`)
- **Scope**: What specific feature needs stories?
- **Architecture**: Frontend framework, Backend patterns, Database?
- **Dependencies**: Does this feature depend on any existing features (e.g., Auth, Global Layout)?
- **Granularity**: Should stories be per-component (React Component X) or per-feature (Full Stack Login)?

### Step 2: Decompose & Generate

Break the feature down into the smallest logical units. Use the template in [references/template.md](references/template.md).

**Key generation rules:**
- **Atomic**: Each story = one PR (Pull Request).
- **Naming**: `[PRODUCT-CODE]-[XXX]-[kebab-case-title].md` (e.g., `VORA-001-login.md`)
- **Roles**: Explicitly tag as Frontend, Backend, or Fullstack.
- **Specs**: Include specific filenames, function names, and API paths from the TDD if available.
- **Acceptance Criteria**: Must be technical and verifiable (e.g., "API returns 200", "Component renders props").
- **Dependencies**: Link stories that block each other.

**Layer-aware decomposition (Standard L1-L5 Model):**

Use this layered approach to breakdown features deeply.

```
L1 (Data Model) ──→ L3 (Backend / API) ──┐
                                          ├──→ L5 (Integration / E2E)
L2 (UI Foundation) ──→ L4 (Feature UI) ──┘
```

| Tag | Purpose | Depends On | Notes |
|-----|---------|------------|-------|
| `L1-data` | Schema, migrations, seed data | — | *First feature: includes project scaffold* |
| `L2-ui-foundation` | Shared components, layouts, design tokens | — | *First feature: includes UI init* |
| `L3-backend` | API endpoints, services, business logic | L1 | |
| `L4-feature-ui` | Screens, pages, feature-specific UI | L2 | |
| `L5-integration` | E2E flows, cross-cutting wiring | L3 + L4 | |

**Rules:**

1. **Verify Layer Necessity**: Before generating, explicitly check:
   - *Does this feature require schema changes (L1)?*
   - *Does this feature require NEW shared UI components (L2)?*
   If NO, omit stories for that layer.

2. **Generate & Tag**:
   - **Layer-tag** every story with the appropriate layer from the table above.
   - **Order bottom-up** — foundation layers (L1, L2) first, then dependent layers (L3, L4), then integration (L5).
   - **Populate `depends:` metadata**:
     - *Linking Layers*: Use the **Depends On** column from the table above.
     - *External Deps*: If the feature depends on another Epic (e.g., Auth), include `[EPIC-XXX]-[CODE]` in the depends field.
     - *Why*: This metadata is used to build the **Project-Wide Dependency Graph**.
   - **First feature only**: include scaffold / project-init tasks within L1 and L2 stories.
   - **Not all layers apply**: omit layers irrelevant to the project's tech stack (e.g., backend-only projects skip L2/L4).

### Step 3: Review

Present the list of generated stories to the user.
- Verify the granularity (too small? too big?).
- Confirm technical details (filenames, paths).

---

## Quality Checklist

- [ ] Stories are atomic (small enough for 1-2 day's work)
- [ ] Technical specs (filenames, APIs) are explicit
- [ ] Acceptance criteria are verifiable
- [ ] Dependencies are clearly marked
- [ ] No ambiguous requirements ("make it fast" -> "load in <200ms")
