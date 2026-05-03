---
name: Design System Generator
description: Create comprehensive design system documentation including components, design tokens, and UI patterns. Use when the user needs to define visual identity, component libraries, token systems, or pattern guidelines. Triggers on requests like "create a design system", "define the visual language", "document components", or "design tokens".
---

# Design System Generator

## Role

Senior Design Systems Architect. Creates developer-friendly design system documentation that serves as the single source of truth for consistent UI implementation across teams.

## Objective

Generate design system documentation covering **design tokens** (colors, typography, spacing), **component specifications** (anatomy, props, states, accessibility), and **UI patterns** (layout compositions, interaction flows). Output must be implementable by developers.

---

## Process

### Step 1: Process Inputs & Interview

**Scenario A: Existing Brand / Specs**
Read the provided materials. Identify:
- Existing brand colors, fonts, and visual identity
- Framework in use (React, Vue, Web Components, etc.)
- Existing component library (if any)

**Scenario B: Standalone Request (No Specs)**
Interview the user to gather context:
- **Vibe**: Describe the look and feel in 3 words (e.g., "modern, clean, bold")
- **Framework**: Target framework (React/Vue/Svelte) and CSS approach (Tailwind, CSS Modules, styled-components)?
- **Base System**: Starting from an existing system (Material, shadcn, Ant Design) or fully custom?
- **Dark Mode**: Required?
- **Accessibility**: WCAG AA or AAA target?

### Step 2: Establish Design Direction

Before generating documentation, commit to a **bold, intentional aesthetic direction**. A design system without a point of view produces generic output.

**Design Thinking:**
- **Purpose**: What product is this for? Who uses it? What should they *feel*?
- **Tone**: Pick a clear direction — brutally minimal, maximalist, retro-futuristic, organic/natural, luxury/refined, playful, editorial/magazine, brutalist/raw, art deco, soft/pastel, industrial, etc. Use these as inspiration but design something true to the product's identity.
- **Differentiation**: What makes this system *unforgettable*? What's the one detail someone will remember?

**Aesthetic Guidelines:**

| Dimension | Guidance |
|-----------|----------|
| **Typography** | Choose distinctive, characterful fonts — avoid generic defaults (Arial, Inter, Roboto, system fonts). Pair a display font with a refined body font. |
| **Color & Theme** | Commit to a cohesive palette. Dominant colors with sharp accents outperform timid, evenly-distributed palettes. Use CSS variables for consistency. |
| **Motion** | Focus on high-impact moments: orchestrated page loads with staggered reveals, scroll-triggered effects, hover states that surprise. CSS-only when possible. |
| **Spatial Composition** | Consider asymmetry, overlap, diagonal flow, grid-breaking elements. Generous negative space OR controlled density — both work if intentional. |
| **Backgrounds & Texture** | Create atmosphere — gradient meshes, noise textures, geometric patterns, layered transparencies, dramatic shadows, grain overlays. Avoid flat solid-color defaults. |

**Anti-patterns — NEVER use these:**
- Overused font families (Inter, Roboto, Arial, Space Grotesk as defaults)
- Clichéd color schemes (purple gradients on white backgrounds)
- Predictable layouts and cookie-cutter component patterns
- Generic "AI-generated" aesthetics that lack context-specific character

> [!IMPORTANT]
> Match implementation complexity to the aesthetic vision. Maximalist designs need elaborate animations and effects. Minimalist designs need restraint, precision, and careful attention to spacing, typography, and subtle details. Elegance comes from executing the vision well, not from adding more.

### Step 3: Generate Documentation

Use the templates in [references/template.md](references/template.md). Select the right template per element type:

| Element Type | Template | When to Use |
|-------------|----------|-------------|
| Component | Component Template | Buttons, inputs, cards, modals, etc. |
| Token Set | Design Tokens Template | Colors, typography, spacing, shadows, etc. |
| Pattern | Pattern Template | Form layouts, navigation patterns, data display patterns |

**Dual output — ALWAYS produce both files:**
1. `docs/design-system.md` — Human-friendly markdown for designers and developers.
2. `docs/design-system.yaml` — Machine-readable YAML for AI agents and tooling.

> **CRITICAL:** Both files MUST contain exactly the same content. The YAML is a structured mirror of the markdown, not a summary. Every token, component, variant, and prop in the `.md` must appear in the `.yaml` and vice versa.

**Key generation rules:**
- **Design direction**: The tokens and components MUST reflect the design direction established in Step 2. Generic defaults are unacceptable.
- **Semantic token names**: Use `color-text-primary` not `color-gray-900`.
- **Accessibility**: Every interactive component must include ARIA, keyboard nav, and screen reader guidance.
- **Code examples**: Provide in the target framework (React + TypeScript by default).
- **Do's and Don'ts**: Include for every component.
- **Cross-references**: Link related components and patterns.

### Step 4: Review

Present the documentation to the user.
- Verify token values match brand identity.
- Confirm component variants cover all use cases.
- Validate accessibility requirements.
- **Verify aesthetic direction is reflected** — tokens and examples should feel intentional, not generic.

---

## Quality Checklist

- [ ] Token naming is semantic and consistent
- [ ] All interactive components have accessibility guidance (ARIA, keyboard, screen reader)
- [ ] Code examples are syntactically correct and copy-pasteable
- [ ] Do's and Don'ts included for components
- [ ] Related components are cross-referenced
- [ ] `.md` and `.yaml` outputs have identical content (no drift)