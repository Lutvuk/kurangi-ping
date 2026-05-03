# Wireframe Documentation Template

**File Name Format:** `docs/wireframes/[kebab-case-screen-name].md` or one large `docs/wireframes.md`

```markdown
# Screen: [Screen Name / ID]

**Purpose:** [What this screen accomplishes]
**User Story:** [Link to relevant Story/Epic]

## Layout & Composition

[Describe the screen layout using ASCII art, a written description, or both.
Choose a layout pattern appropriate for this screen's purpose:]

**Common patterns:**
- **List/Table View** — Nav + Filter bar + Data table + Pagination
- **Detail View** — Nav + Hero/Header + Content sections + Actions
- **Form View** — Nav + Form fields (grouped) + Submit/Cancel
- **Dashboard** — Nav + Grid of cards/widgets
- **Modal/Dialog** — Overlay with focused content + Close/Actions
- **Split View** — Nav + List panel + Detail panel
- **Wizard/Stepper** — Nav + Step indicator + Step content + Next/Back

[Draw the layout for THIS specific screen below]

## Component Specifications

| Component | Type | Data Source (Field/API) | Interaction |
|-----------|------|-------------------------|-------------|
| [Name] | [Button/Input/Table] | [API Field] | [On Click/Change...] |

## Key States
- **Empty:** [Message/Action when no data]
- **Loading:** [Skeleton/Spinner behavior]
- **Error:** [Error message location and retry logic]

## API Interactions
- **Init/Load:** `GET /endpoint` (Query params: ...)
- **Actions:** `POST /endpoint` (Payload: ...)

## Responsive Behavior
- **Mobile:** [Stacking order / hidden elements]
- **Desktop:** [Expanded view / multi-column]
```
