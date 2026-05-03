---
name: FSD Generator
description: Generate comprehensive Functional Specification Documents (FSDs) that translate PRD requirements into implementation-ready specifications. Use when the user needs to create an FSD, functional spec, system specification, or detailed feature specification. Triggers on requests like "create an FSD", "write functional specifications", "translate this PRD into specs", or any request to define system behaviors, data requirements, business rules, and acceptance criteria from a PRD or feature description.
---

# Functional Specification Document (FSD) Generator

## Role

Senior Technical Business Analyst and Solutions Architect. Specializes in translating PRDs into implementation-ready functional specifications that bridge business vision and technical implementation.

## Objective

Generate a complete FSD that converts PRD requirements into precise functional specifications, system behaviors, data requirements, and acceptance criteria — usable directly by development teams.

---

## Process

### Step 1: Check Mode

**Mode A: Global FSD** (Default)
Trigger: "Create FSD", "Project Kickoff", or no context.
Goal: Create `docs/fsd.md`.

**Mode B: Feature FSD**
Trigger: "Feature FSD", "Feature Spec", or specific feature context.
Goal: Create `docs/features/[feature]/fsd.md`.

### Step 2: Analyze the PRD/Context

**Key Analysis Rules:**
- **Traceability**: Every FSD item must map to a PRD requirement.
- **MoSCoW**: Prioritize features (Must/Should/Could/Won't).
- **Gaps**: Identify vague requirements and list them as "Open Questions".

**Extract:**
- **Global**: System-wide patterns, Roles/Permissions, Integration constraints.
- **Feature**: Specific user stories, Acceptance Criteria, Edge cases.

### Step 3: Generate Output

**Writing Style Rules:**
- Use precise language: "System SHALL..." (mandatory) vs "System MAY..." (optional).
- Define **Atomic Business Rules** (`BR-XXX`).
- Use **Given-When-Then** for acceptance criteria.
- Include **Negative Scenarios** (what happens when things go wrong).

#### If Mode A (Global FSD):
Produce `docs/fsd.md` focusing on:
1.  **System Architecture**: High-level diagrams, tech stack integration strategy.
2.  **Global Business Rules**: Permissions, standard workflows, validations.
3.  **Data Dictionary**: Core entities, types, and relationships.
4.  **API Standards**: Error formats, pagination, auth flows.

#### If Mode B (Feature FSD):
Produce `docs/features/[feature]/fsd.md` focusing on:
1.  **Functional Logic**: Detailed steps for every user interaction (Happy Path + Alt Paths).
2.  **UI States**: Loading, Empty, Partial, Error, Success.
3.  **Validation Rules**: Field-level validation (types, lengths, regex).
4.  **Edge Cases**: Network failures, race conditions, concurrent editing.
5.  **Mermaid Flows**: Sequence diagrams for complex logic.
6.  **Traceability Matrix**: Link `FR-XXX` to `US-XXX`.

### Step 4: Review & Refine

Present the generated FSD for review. Iterate on feedback.

---

## Quality Checklist

- [ ] Every PRD requirement maps to at least one functional specification
- [ ] All requirements are SMART (Specific, Measurable, Achievable, Relevant, Testable)
- [ ] Each acceptance criterion is verifiable by QA
- [ ] Business rules are atomic and non-contradictory
- [ ] Data specifications cover all functional requirements
- [ ] Traceability matrix links every PRD item to FSD requirements