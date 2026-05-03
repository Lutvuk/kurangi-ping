# Story Template

**File Name Format:** `[PRODUCT-CODE]-[XXX]-[kebab-case-title].md`

```markdown
# [PRODUCT-CODE]-[XXX]: [Technical Story Title]

**Epic:** [EPIC-XXX]
**Layer:** [L1-data / L2-ui-foundation / L3-backend / L4-feature-ui / L5-integration]
**Role:** [Frontend / Backend / Fullstack / DevOps]
**Estimation:** [Fibonacci Points]
**Priority:** [Must / Should / Could]

---

## Objective
[Concise description of code to be written or modified]

## Technical Specifications
*   **Proposed Files:** `[path/to/file.ts]`
*   **Functions/Classes:** `[FunctionName]`, `[ClassName]`
*   **API Endpoints:** `[METHOD] /path/to/endpoint` (if applicable)
*   **Data Models:** `[EntityName]` (if applicable)

## Acceptance Criteria (Technical)
*   [ ] Component renders correctly with props X, Y
*   [ ] API returns 200 OK with JSON structure Z on success
*   [ ] Error handling implemented for case A
*   [ ] Unit tests pass

## Business Rules & Logic
*   [Rule 1]
*   [Rule 2]

## Dependencies
*   Depends on: [CODE-XXX]

## Definition of Done
*   [ ] Code implemented
*   [ ] Unit tests passing
*   [ ] Lint/Type check clear
```
