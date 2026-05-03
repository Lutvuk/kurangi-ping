---
name: API Contract Generator
description: Generate comprehensive OpenAPI 3.0+ specifications from FSDs and ERDs. Use when the user needs a contract-first API design, Swagger documentation, or interface definitions for frontend/backend alignment. Triggers on requests like "create API contract", "generate OpenAPI spec", "design the API", or "Swagger definition".
---

# API Contract Generator

## Role

Senior API Architect. Specialist in RESTful design and OpenAPI 3.0+ standards. Translates business logic (FSD) and data models (ERD) into precise, implementable API contracts.

## Objective

Generate a production-ready **OpenAPI 3.0+ specification** (YAML) that validatess against the schema and covers all required functionality.

---

## Process

### Step 1: Process Inputs & Interview

**Scenario A: Converting FSD / ERD**
Read the provided documents. Extract:
- Resources (from ERD entities)
- Operations (from FSD user stories)
- Data types & validation rules (from ERD)

**Scenario B: Standalone Request (No Docs)**
Interview the user to gather context:
- **Scope**: What resources are we exposing? (e.g., Users, Products)
- **Operations**: CRUD only? Custom actions (e.g., `POST /checkout`)?
- **Security**: Public? Bearer Auth? API Key?
- **Standard**: JSON:API? Simple REST?

### Step 2: Design & Generate

Use the template in [references/template.md](references/template.md).

**Key generation rules:**
- **Naming**: Plural nouns for resources (`/users`), kebab-case for paths.
- **RESTful**: Use correct HTTP methods (GET, POST, PUT, PATCH, DELETE).
- **Pagination**: Implement standard limit/offset or cursor pagination for lists.
- **Error Handling**: Use the standard error object defined in the template.
- **Components**: Reuse schemas in `components/schemas` to avoid duplication.
- **Security**: Apply default security schemes globally, override per endpoint if needed.

### Step 3: Review

Present the YAML to the user (or a summary of endpoints).
- Verify resource hierarchy.
- Confirm required fields and validation rules.
- Check security scopes.

---

## Quality Checklist

- [ ] Valid OpenAPI 3.0+ syntax
- [ ] Every endpoint has a summary, description, and operationId
- [ ] Request/Response bodies reuse shared schemas
- [ ] Error responses defined for 4xx/5xx cases
- [ ] Auth defined globally or per-endpoint
- [ ] No "Any" or "Object" types without properties (strict typing)