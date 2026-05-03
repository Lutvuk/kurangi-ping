---
name: Epic Generator
description: Generate comprehensive EPICs from FSDs, TDDs, or high-level requirements. Use when the user needs to plan a large feature, break down a project into deliverables, or create high-level roadmap items. Triggers on requests like "create epics", "break down this project", "generate high-level tasks", or "plan the feature delivery".
---

# EPIC Generation Prompt

## Role

Senior Product Owner & Business Analyst. Translates complex functional (FSD) and technical (TDD) specifications into well-structured, actionable EPICs for agile planning.

## Objective

Generate a set of **EPICs** (prefixed with `[PRODUCT-CODE]-EPIC-`) that capture all major feature areas and technical deliverables. Each EPIC must be traceable to verify that all requirements are covered.

---

## Process

### Step 1: Process Inputs & Interview

**Scenario A: Converting FSD / TDD**
Read the provided documents. Identify:
- Business capabilities (FSD)
- User workflows (FSD + Wireframes)
- System modules & services (TDD)

**Scenario B: Standalone Request (No Docs)**
Interview the user to gather context:
- **Product Code**: What short code to use for IDs? (e.g., `VORA`, `QUEUE`)
- **Scope**: What is the high-level goal or feature set?
- **Timeline/Phasing**: Are there specific release milestones?

### Step 2: Identify & Define EPICs

Group requirements into logical feature areas. Use the template in [references/template.md](references/template.md).

**Key generation rules:**
- **Naming**: `[PRODUCT-CODE]-EPIC-[XXX]-[kebab-case-title].md` (e.g., `VORA-EPIC-001-user-auth.md`)
- **Scope**: Each EPIC must be independently deliverable.
- **Traceability**: Link back to specific FSD sections or TDD components.
- **Dependencies**: Explicitly list prerequisite EPICs.
- **Sizing**: Should be breakable into 5-15 user stories.

### Step 3: Review

Present the list of generated EPICs (just titles and summaries first).
- Verify the breakdown (too granular? too broad?).
- Confirm dependencies logic.

---

## Quality Checklist

- [ ] Every functional requirement maps to at least one EPIC
- [ ] Dependencies form a valid directed acyclic graph (no circular loops)
- [ ] Acceptance criteria are clear and measurable
- [ ] Business value is articulated in user terms
- [ ] Naming convention follows `[PRODUCT-CODE]-EPIC-[Number]`