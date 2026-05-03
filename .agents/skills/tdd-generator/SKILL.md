---
name: TDD Generator
description: Generate a Technical Design Document (TDD) or lightweight Tech Spec for a feature. Supports LIGHT mode (~3-page spec for simple changes) and HEAVY mode (full architecture document for complex features). Use when the user needs technical implementation specs, architecture decisions for a feature, or a developer-facing design document. Triggers on requests like "create a TDD", "write a tech spec", "technical design for this feature", or "architecture document".
---

# Technical Design Document (TDD) Generator

## Role

Senior Solutions Architect. Focuses strictly on the **DELTA** — what is changing, what is new. Communicates via Mermaid diagrams rather than prose where possible. Excludes cross-cutting concerns governed by global standards unless explicitly changed.

## Objective

Generate a technical document that gives development teams everything needed to implement the feature described in the FSD / Tech Spec inputs. Focus on **what is new or changed only**.

---

## Process

### Step 1: Mode Selection

Determine LIGHT or HEAVY mode:

| Condition | Mode |
|-----------|------|
| New service or bounded context | HEAVY |
| Persistent data model changes (new entities) | HEAVY |
| External system integrations | HEAVY |
| Public API changes | HEAVY |
| Non-trivial async flows or state machines | HEAVY |
| Everything else | LIGHT |

If the mode is not specified by the user or workflow, state your assumption clearly.

### Step 2: Process Inputs & Interview

**Scenario A: Converting FSD / Specs**
Read the provided documents. Identify:
- What is **NEW** (new components, entities, endpoints)
- What is **CHANGED** (modified behavior, schema migrations)
- What is **UNCHANGED** (explicitly list, then ignore)

**Scenario B: Standalone Request (No Docs)**
Interview the user to gather context:
- **Feature**: What is being built or changed?
- **Scope**: New service, new endpoints, UI changes, data model changes?
- **Constraints**: Performance targets, tech stack constraints, integration requirements?
- **Inputs available**: FSD, ERD, API Contract, Wireframes?

### Step 3: Generate Document

Use the template in [references/template.md](references/template.md).

**Key generation rules:**
- **Delta only**: Do not document the entire existing system. Only what's new or changed.
- **Mermaid preferred**: Use `sequenceDiagram`, `flowchart TD`, or `erDiagram` for all non-trivial flows.
- **No boilerplate**: Omit sections entirely if no delta exists. Reference global standards; do not restate them.
- **LIGHT mode**: Produce ~3 pages. Skip sections 6, 8, 10 unless critical.
- **HEAVY mode**: Full template. All sections that have deltas.
- **Gap identification**: If input artifacts have inconsistencies, document them clearly.

### Step 4: Review

Present the document to the user.
- Verify architecture decisions are sound.
- Confirm all new components are accounted for.
- Validate data model changes.

---

## Quality Checklist

- [ ] Every technical decision traces back to a functional requirement
- [ ] All NEW entities and API endpoints are detailed
- [ ] Diagrams used for non-trivial flows (not just prose)
- [ ] Delta is clearly separated from unchanged system
- [ ] LIGHT/HEAVY mode is stated and justified