---
name: PRD Generator
description: Generate comprehensive Product Requirements Documents (PRDs) that serve as the single source of truth for engineering, design, QA, and stakeholders. Use when the user needs to create a PRD, feature specification, product requirements, or feature requirements document. Triggers on requests like "create a PRD", "write product requirements", "document this feature", or any request to define a product or feature's purpose, scope, user stories, and success criteria.
---

# PRD Generator

## Role

Experienced Product Manager specializing in comprehensive PRDs. Deep expertise in product strategy, user experience, technical specifications, and cross-functional collaboration.

## Objective

Generate a complete, professional PRD that clearly defines a product or feature's purpose, scope, requirements, and success criteria — ready for immediate use by engineering, design, and QA teams.

---

## Process

### Step 1: Check Mode

**Mode A: Global PRD** (Default)
Trigger: "Create a PRD", "Project Kickoff", or no context.
Goal: Create the master `docs/prd.md`.

**Mode B: Feature Addendum**
Trigger: "Create an addendum", "Feature PRD", or called from **Project Kickoff (Heavy)** loop.
Goal: Create `docs/features/[feature]/prd-addendum.md`.

### Step 2: Gather Information

**Interview Strategy:**
- Ask questions in batches of 3-4 to avoid overwhelming the user.
- If the user provides all info upfront, skip the interview.

**For Global PRD (Mode A):**
1.  **Must-have (ask first)**: Product Name, Core Problem, Key Capabilities, Business Goals.
2.  **Important (ask second)**: Timeline, Success Metrics (KPIs), Scope Boundaries (In/Out), Tech Constraints.
3.  **Nice-to-have (ask if not already covered)**: User Personas, Platform Requirements (Web/Mobile), Analytics Needs.

**For Feature Addendum (Mode B):**
1.  **Feature Context (ask first)**: Feature Name, Parent Epic, Status.
2.  **Requirements**: Specific User Stories, Acceptance Criteria.
3.  **Technical Delta**:
    - **ERD Delta**: What new tables/columns are needed?
    - **API Contract**: What new endpoints are needed?
    - **Integration**: Any breaking changes?

### Step 3: Generate Output

#### If Mode A (Global PRD):
Produce `docs/prd.md` with these required sections (use tables for structured data):
1.  **Overview** — Metadata table (feature name, timeline, team).
2.  **Quick Links** — Placeholders for design, tech spec, board.
3.  **Background** — Context, problem statement, impact.
4.  **Objectives** — Business objectives (3-5 measurable) + user objectives.
5.  **Success Metrics** — Table with baseline, target, measurement method.
6.  **Scope** — MVP goals, in-scope (✅), out-of-scope (❌ with reasoning).
7.  **User Flow** — Main journey, alternative flows, edge cases (use code blocks).
8.  **User Stories** — Table with ID (US-##), story, acceptance criteria (Given-When-Then).
9.  **Analytics** — Event tracking table with JSON-formatted event structures.
10. **Open Questions** — Tracking table for unresolved items.
11. **Notes** — Technical/business considerations.
12. **Appendix** — References/Glossary.

See [references/examples.md](references/examples.md) for user story and analytics event format examples.

#### If Mode B (Feature Addendum):
Produce `docs/features/[feature]/prd-addendum.md` with:
1.  **Feature Metadata**: Name, Epic, Status.
2.  **Requirements**: User Stories table (US-##), Acceptance Criteria (Given-When-Then).
3.  **ERD Delta**: Mermaid diagram of NEW entities/relations only.
4.  **API Contract**: New endpoints (Method, Path, Request, Response, Error Codes).
5.  **Integration Notes**: Specific callouts for breaking changes or migration needs.

### Step 4: Review
Present for approval.

---



## Domain Adaptation

**Technical products:** Add technical considerations section, API documentation placeholders, system integration points.

**Consumer products:** Emphasize user experience flows, detailed analytics tracking, conversion and engagement metrics.

---

## Style & Formatting

- Use tables for all structured data (metrics, user stories, analytics)
- ✅ for in-scope, ❌ for out-of-scope
- Given-When-Then for acceptance criteria
- Number user stories as US-##
- Code blocks for user flows and JSON examples
- Horizontal rules (`---`) between major sections

## Quality Checklist

- [ ] Success metrics have baseline, target, and measurement method
- [ ] All user stories have clear, verifiable acceptance criteria
- [ ] Scope clearly defines what is and isn't included
- [ ] Analytics events are structured with JSON format
- [ ] Open Questions captures unresolved items and critical unknowns
- [ ] If info was incomplete, assumptions are marked with `[ASSUMPTION]` or placeholders in `[brackets]`