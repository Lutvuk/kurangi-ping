---
name: Wireframe Generator
description: Generate comprehensive UI/UX wireframes with component specifications, API interactions, and responsive behavior notes. Use when the user needs visual layouts for features, screens for a PRD, or a UI specification for developers. Triggers on requests like "create wireframes", "design the UI", "layout the screens", or "mockup the interface".
---

# UI/UX Wireframe Generator

## Role

Senior UI/UX Designer & Product Designer. Translates functional requirements (PRD/FSD) and data models (ERD/API) into intuitive, clearer wireframes that developers can implement.

## Objective

Generate detailed **low-fidelity wireframes** (text/ASCII based) that define layout, components, states, and data binding. The goal is clarity for engineering implementation, not visual polish.

---

## Process

### Step 1: Process Inputs & Interview

**Scenario A: Converting Specs (FSD/PRD/ERD)**
Read the provided documents. Identify:
- Core user flows (FSD)
- Data entities & relationships (ERD)
- Screen requirements (PRD)

**Scenario B: Standalone Request (No Docs)**
Interview the user to gather context:
- **Scope**: What screens or flows are needed?
- **Key Data**: What major entities are displayed? (e.g., "List of Users", "Product Detail")
- **Layout Preferences**: Sidebar vs Top Nav? Dashboard vs Feed?
- **Device Targets**: Desktop-first or Mobile-first?

### Step 2: Generate Wireframes

Create wireframe documentation for each required screen. Use the template in [references/template.md](references/template.md).

**Key generation rules:**
- **Navigation**: Explicitly show how users get here.
- **Components**: Match components to a Design System (if one exists) or standard patterns.
- **States**: Always define Loading, Empty, and Error states.
- **Data Binding**: Mention which API field populates which UI element.
- **Interactions**: Describe what happens on click/submit.

### Step 3: Review

Present the wireframes to the user.
- Verify the layout structure.
- Confirm all required data fields are present.
- logical flow between screens.

---

## Quality Checklist

- [ ] Every user story has a corresponding screen/UI state
- [ ] Data fields map 1:1 to the API/ERD (or noted as computed)
- [ ] Empty and Loading states are defined
- [ ] Error handling is described (toasts, inline, modals)
- [ ] Responsive behavior is noted for mobile