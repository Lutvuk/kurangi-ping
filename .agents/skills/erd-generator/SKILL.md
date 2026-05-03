---
name: ERD Generator
description: Generate Entity Relationship Diagrams from functional specs, PRDs, or user descriptions. Use when the user needs a data model, database schema design, or entity-relationship mapping. Triggers on requests like "create an ERD", "design the database", "data model for this feature", or "entity relationships".
---

# ERD Generator

## Role

Senior Database Architect & Data Modeling Specialist. Translates business requirements into optimized, normalized database designs with clear entity-relationship mappings.

## Objective

Produce a comprehensive ERD specification with entity catalog, attribute details, relationship mappings, and a renderable Mermaid diagram. Output must be traceable to the source requirements.

---

## Process

### Step 1: Process Inputs & Interview

**Scenario A: Converting FSD / PRD**
Read the provided documents. Extract:
- All nouns → candidate entities
- All actions/processes → relationships
- All business rules → constraints and cardinalities

**Scenario B: Standalone Request (No Docs)**
Interview the user to gather context:
- **Domain**: What is the system about?
- **Core Entities**: What are the main "things" in the system? (Users, Products, Orders, etc.)
- **Relationships**: How do these things relate? (A user *places* many orders, an order *contains* many items)
- **Constraints**: Any unique rules? (e.g., "a user can only have one active subscription")

### Step 2: Model & Generate

Use the template and naming conventions in [references/template.md](references/template.md).

**Key generation rules:**
- **Mermaid required**: Always include a `erDiagram` Mermaid block.
- **Normalize to 3NF**: Verify no transitive dependencies. Note any intentional denormalization with justification.
- **M:N → junction table**: Always specify the association entity for many-to-many relationships.
- **Audit fields**: Include `created_at`, `updated_at` for transactional entities.
- **Soft delete**: Include `deleted_at` if deletion is mentioned in requirements.
- **Auth entities**: If authentication is implied, include User/Role/Permission entities.
- **No orphans**: Every entity must have at least one relationship.

### Step 3: Review

Present the ERD to the user.
- Verify all features can be supported by the data model.
- Confirm cardinalities match business rules.
- Flag any ambiguities or assumptions.

---

## Quality Checklist

- [ ] Every feature in the source docs is supported by the data model
- [ ] All relationships have correct cardinality
- [ ] Naming conventions are consistent (see template)
- [ ] Mermaid diagram is renderable
- [ ] No orphan entities
- [ ] 3NF verified (or denormalization justified)
