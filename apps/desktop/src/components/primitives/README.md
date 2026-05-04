# Primitive Component Library

This folder defines the reusable, feature-agnostic UI primitive layer for the desktop client.

## Taxonomy

- `actions/`: direct user action controls (for example `Button`, `PrimaryToggle`).
- `forms/`: data-entry and selection controls (for example `Input`, `Select`).
- `feedback/`: state and outcome indicators (for example `StatusBadge`, `Toast`).
- `surfaces/`: structural containers and overlays (for example `Card`, `Modal`, `Panel`).
- `shared/`: cross-primitive TypeScript contracts and helpers.

## Single Entry Import

Use one typed entrypoint:

```ts
import { primitiveCatalog, type PrimitiveStatusState } from "../components/primitives";
```

Do not import category internals from feature modules unless there is a strong reason.

## Naming Convention

- Component files: `PascalCase.tsx`
- Type/helper files: `kebab-or-lowercase.ts` when not a component
- Category barrels: `index.ts`
- All exports flow through `primitives/index.ts`

## Rules

- No feature/business logic in primitive layer.
- Keep primitives token-driven and reusable.
- Keep prop contracts explicit and stable.
