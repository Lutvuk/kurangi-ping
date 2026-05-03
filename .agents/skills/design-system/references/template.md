# Design System Templates

This file contains templates for the three element types a design system documents.
Each element type must be output in **two formats** with identical content:
1. `design-system.md` — Human-friendly markdown
2. `design-system.yaml` — Machine-readable YAML for AI/tooling

---

## Component Template

### Markdown Format (`design-system.md`)

```markdown
# [Component Name]

## Overview
Brief description of the component's purpose and when to use it.

## Usage Guidelines
### When to Use
- [Scenario 1]

### When Not to Use
- [Alternative recommendation]

## Anatomy
| Part | Description | Required |
|------|-------------|----------|
| [Part name] | [Purpose] | Yes/No |

## Variants
### [Variant Name]
- **Use case:** [When to use this variant]
- **Visual:** [Description]

## Props / API
| Prop | Type | Default | Description |
|------|------|---------|-------------|
| [name] | [type] | [default] | [description] |

## Design Tokens
| Token | Value | Usage |
|-------|-------|-------|
| [token-name] | [value] | [where applied] |

## States
- **Default / Hover / Active / Focus / Disabled:** [description per state]

## Accessibility
- **ARIA:** [Required attributes]
- **Keyboard:** [Interaction patterns]
- **Screen Reader:** [Announcements]

## Code Examples
[Basic, Variants, Complex — in the target framework]

## Do's and Don'ts
| ✅ Do | ❌ Don't |
|-------|---------|
| [Good practice] | [Anti-pattern] |

## Related Components
- [Component] — [Relationship]
```

### YAML Format (`design-system.yaml`)

```yaml
components:
  - name: "[Component Name]"
    overview: "[Brief description]"
    usage:
      when_to_use:
        - "[Scenario 1]"
      when_not_to_use:
        - "[Alternative recommendation]"
    anatomy:
      - part: "[Part name]"
        description: "[Purpose]"
        required: true
    variants:
      - name: "[Variant Name]"
        use_case: "[When to use]"
        visual: "[Description]"
    props:
      - name: "[name]"
        type: "[type]"
        default: "[default]"
        description: "[description]"
    tokens:
      - name: "[token-name]"
        value: "[value]"
        usage: "[where applied]"
    states:
      default: "[description]"
      hover: "[description]"
      active: "[description]"
      focus: "[description]"
      disabled: "[description]"
    accessibility:
      aria: "[Required attributes]"
      keyboard: "[Interaction patterns]"
      screen_reader: "[Announcements]"
    dos_and_donts:
      - do: "[Good practice]"
        dont: "[Anti-pattern]"
    related_components:
      - name: "[Component]"
        relationship: "[Relationship]"
```

---

## Design Tokens Template

### Markdown Format

```markdown
# Design Tokens

## Colors

### Brand
| Token | Value | Usage |
|-------|-------|-------|
| color-brand-primary | #XXXXXX | Primary actions, links |
| color-brand-secondary | #XXXXXX | Secondary actions |
| color-brand-accent | #XXXXXX | Highlights, badges |

### Neutral
| Token | Value | Usage |
|-------|-------|-------|
| color-neutral-50 | #XXXXXX | Background (lightest) |
| color-neutral-100 | #XXXXXX | Subtle backgrounds |
| ... | | |
| color-neutral-900 | #XXXXXX | Primary text (darkest) |

### Semantic / Status
| Token | Value | Usage |
|-------|-------|-------|
| color-success | #XXXXXX | Success states, confirmations |
| color-warning | #XXXXXX | Warning states |
| color-error | #XXXXXX | Error states, destructive actions |
| color-info | #XXXXXX | Informational states |

### Surface & Background
| Token | Value | Usage |
|-------|-------|-------|
| color-bg-primary | #XXXXXX | Main background |
| color-bg-secondary | #XXXXXX | Card/section backgrounds |
| color-bg-overlay | rgba(...) | Modal/dialog overlays |

---

## Typography

### Font Families
| Token | Value | Usage |
|-------|-------|-------|
| font-family-heading | "[Font], sans-serif" | Headings (h1–h6) |
| font-family-body | "[Font], sans-serif" | Body text, UI labels |
| font-family-mono | "[Font], monospace" | Code blocks, technical content |

### Font Sizes
| Token | Value (px/rem) | Usage |
|-------|----------------|-------|
| font-size-xs | 12px / 0.75rem | Captions, fine print |
| font-size-sm | 14px / 0.875rem | Secondary text, labels |
| font-size-base | 16px / 1rem | Body text |
| font-size-lg | 18px / 1.125rem | Large body, subheadings |
| font-size-xl | 20px / 1.25rem | Section headings |
| font-size-2xl | 24px / 1.5rem | Page headings |
| font-size-3xl | 30px / 1.875rem | Hero headings |

### Font Weights
| Token | Value | Usage |
|-------|-------|-------|
| font-weight-regular | 400 | Body text |
| font-weight-medium | 500 | Labels, emphasis |
| font-weight-semibold | 600 | Subheadings |
| font-weight-bold | 700 | Headings, CTAs |

### Line Heights
| Token | Value | Usage |
|-------|-------|-------|
| line-height-tight | 1.25 | Headings |
| line-height-normal | 1.5 | Body text |
| line-height-relaxed | 1.75 | Long-form content |

---

## Spacing

| Token | Value (px/rem) | Usage |
|-------|----------------|-------|
| spacing-1 | 4px / 0.25rem | Inline padding, icon gaps |
| spacing-2 | 8px / 0.5rem | Tight element gaps |
| spacing-3 | 12px / 0.75rem | Compact padding |
| spacing-4 | 16px / 1rem | Default padding/gap |
| spacing-5 | 20px / 1.25rem | Medium padding |
| spacing-6 | 24px / 1.5rem | Section padding |
| spacing-8 | 32px / 2rem | Large section gaps |
| spacing-10 | 40px / 2.5rem | Page-level spacing |
| spacing-12 | 48px / 3rem | Hero/banner spacing |
| spacing-16 | 64px / 4rem | Major section dividers |

---

## Borders

### Border Radius
| Token | Value | Usage |
|-------|-------|-------|
| radius-none | 0 | Sharp corners |
| radius-sm | 4px | Inputs, small elements |
| radius-md | 8px | Cards, buttons |
| radius-lg | 12px | Modals, panels |
| radius-xl | 16px | Large cards |
| radius-full | 9999px | Circular buttons, avatars |

### Border Widths
| Token | Value | Usage |
|-------|-------|-------|
| border-width-thin | 1px | Default borders |
| border-width-medium | 2px | Focus rings, emphasis |
| border-width-thick | 4px | Active indicators |

### Border Colors
| Token | Value | Usage |
|-------|-------|-------|
| border-color-default | #XXXXXX | Default borders |
| border-color-focus | #XXXXXX | Focus state |
| border-color-error | #XXXXXX | Error state |

---

## Shadows / Elevation

| Token | Value | Usage |
|-------|-------|-------|
| shadow-sm | 0 1px 2px rgba(...) | Subtle lift (buttons) |
| shadow-md | 0 4px 6px rgba(...) | Cards, dropdowns |
| shadow-lg | 0 10px 15px rgba(...) | Modals, popovers |
| shadow-xl | 0 20px 25px rgba(...) | Floating panels |

---

## Breakpoints

| Token | Value | Usage |
|-------|-------|-------|
| breakpoint-sm | 640px | Small tablets |
| breakpoint-md | 768px | Tablets |
| breakpoint-lg | 1024px | Small desktops |
| breakpoint-xl | 1280px | Desktops |
| breakpoint-2xl | 1536px | Large screens |

---

## Z-Index

| Token | Value | Usage |
|-------|-------|-------|
| z-base | 0 | Default layer |
| z-dropdown | 100 | Dropdowns, tooltips |
| z-sticky | 200 | Sticky headers |
| z-overlay | 300 | Overlays, backdrops |
| z-modal | 400 | Modals, dialogs |
| z-toast | 500 | Toast notifications |
```

### YAML Format

```yaml
tokens:
  colors:
    brand:
      - { name: color-brand-primary, value: "#XXXXXX", usage: "Primary actions, links" }
      - { name: color-brand-secondary, value: "#XXXXXX", usage: "Secondary actions" }
      - { name: color-brand-accent, value: "#XXXXXX", usage: "Highlights, badges" }
    neutral:
      - { name: color-neutral-50, value: "#XXXXXX", usage: "Background (lightest)" }
      - { name: color-neutral-900, value: "#XXXXXX", usage: "Primary text (darkest)" }
    semantic:
      - { name: color-success, value: "#XXXXXX", usage: "Success states" }
      - { name: color-warning, value: "#XXXXXX", usage: "Warning states" }
      - { name: color-error, value: "#XXXXXX", usage: "Error states" }
      - { name: color-info, value: "#XXXXXX", usage: "Informational states" }
    surface:
      - { name: color-bg-primary, value: "#XXXXXX", usage: "Main background" }
      - { name: color-bg-secondary, value: "#XXXXXX", usage: "Card backgrounds" }
      - { name: color-bg-overlay, value: "rgba(...)", usage: "Modal overlays" }

  typography:
    font_families:
      - { name: font-family-heading, value: "'[Font]', sans-serif", usage: "Headings" }
      - { name: font-family-body, value: "'[Font]', sans-serif", usage: "Body text" }
      - { name: font-family-mono, value: "'[Font]', monospace", usage: "Code blocks" }
    font_sizes:
      - { name: font-size-xs, value: "0.75rem", usage: "Captions" }
      - { name: font-size-sm, value: "0.875rem", usage: "Labels" }
      - { name: font-size-base, value: "1rem", usage: "Body text" }
      - { name: font-size-lg, value: "1.125rem", usage: "Subheadings" }
      - { name: font-size-xl, value: "1.25rem", usage: "Section headings" }
      - { name: font-size-2xl, value: "1.5rem", usage: "Page headings" }
      - { name: font-size-3xl, value: "1.875rem", usage: "Hero headings" }
    font_weights:
      - { name: font-weight-regular, value: 400, usage: "Body text" }
      - { name: font-weight-medium, value: 500, usage: "Labels" }
      - { name: font-weight-semibold, value: 600, usage: "Subheadings" }
      - { name: font-weight-bold, value: 700, usage: "Headings" }
    line_heights:
      - { name: line-height-tight, value: 1.25, usage: "Headings" }
      - { name: line-height-normal, value: 1.5, usage: "Body text" }
      - { name: line-height-relaxed, value: 1.75, usage: "Long-form" }

  spacing:
    - { name: spacing-1, value: "0.25rem", usage: "Inline padding" }
    - { name: spacing-2, value: "0.5rem", usage: "Tight gaps" }
    - { name: spacing-3, value: "0.75rem", usage: "Compact padding" }
    - { name: spacing-4, value: "1rem", usage: "Default padding" }
    - { name: spacing-6, value: "1.5rem", usage: "Section padding" }
    - { name: spacing-8, value: "2rem", usage: "Large gaps" }
    - { name: spacing-12, value: "3rem", usage: "Hero spacing" }
    - { name: spacing-16, value: "4rem", usage: "Major dividers" }

  borders:
    radius:
      - { name: radius-none, value: "0", usage: "Sharp corners" }
      - { name: radius-sm, value: "4px", usage: "Inputs" }
      - { name: radius-md, value: "8px", usage: "Cards, buttons" }
      - { name: radius-lg, value: "12px", usage: "Modals" }
      - { name: radius-full, value: "9999px", usage: "Avatars" }
    widths:
      - { name: border-width-thin, value: "1px", usage: "Default borders" }
      - { name: border-width-medium, value: "2px", usage: "Focus rings" }
    colors:
      - { name: border-color-default, value: "#XXXXXX", usage: "Default" }
      - { name: border-color-focus, value: "#XXXXXX", usage: "Focus state" }
      - { name: border-color-error, value: "#XXXXXX", usage: "Error state" }

  shadows:
    - { name: shadow-sm, value: "0 1px 2px rgba(...)", usage: "Buttons" }
    - { name: shadow-md, value: "0 4px 6px rgba(...)", usage: "Cards" }
    - { name: shadow-lg, value: "0 10px 15px rgba(...)", usage: "Modals" }
    - { name: shadow-xl, value: "0 20px 25px rgba(...)", usage: "Floating panels" }

  breakpoints:
    - { name: breakpoint-sm, value: "640px", usage: "Small tablets" }
    - { name: breakpoint-md, value: "768px", usage: "Tablets" }
    - { name: breakpoint-lg, value: "1024px", usage: "Small desktops" }
    - { name: breakpoint-xl, value: "1280px", usage: "Desktops" }

  z_index:
    - { name: z-base, value: 0, usage: "Default layer" }
    - { name: z-dropdown, value: 100, usage: "Dropdowns" }
    - { name: z-sticky, value: 200, usage: "Sticky headers" }
    - { name: z-overlay, value: 300, usage: "Overlays" }
    - { name: z-modal, value: 400, usage: "Modals" }
    - { name: z-toast, value: 500, usage: "Toasts" }
```

---

## Pattern Template

### Markdown Format

```markdown
# [Pattern Name] Pattern

## Overview
[What problem this pattern solves]

## Use Cases
[Scenarios where this pattern applies]

## Structure
[Layout and component composition]

## Behavior
[Interaction specifications]

## Responsive Considerations
[How pattern adapts across breakpoints]

## Implementation Examples
[Code for common scenarios]

## Variations
[Alternative approaches within the pattern]
```

### YAML Format

```yaml
patterns:
  - name: "[Pattern Name]"
    overview: "[What problem this solves]"
    use_cases:
      - "[Scenario]"
    structure: "[Layout and component composition]"
    behavior: "[Interaction specifications]"
    responsive:
      mobile: "[Adaptation]"
      tablet: "[Adaptation]"
      desktop: "[Adaptation]"
    variations:
      - "[Alternative approach]"
```
