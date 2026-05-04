# Scripts Module

This module stores automation scripts for local dev, test, and build orchestration.

Scope for scaffold phase:
- Keep scripts deterministic and easy to read.
- Fail fast with clear error messages.

## KP-008 Orchestration Commands

Run from repository root:

```powershell
pnpm dev
pnpm test
pnpm build
```

### Prerequisites

- Node.js + pnpm
- Go toolchain
- Rust toolchain (`cargo`)

### Expected Outputs

- `pnpm dev`:
  - Starts relay-controller in background.
  - Starts desktop Tauri development runtime in foreground.
  - Writes relay logs to `scripts/logs/`.
- `pnpm test`:
  - Runs desktop UI tests (`vitest`), Rust crate tests, and Go tests.
- `pnpm build`:
  - Runs desktop production build, Rust crate build, and Go build.
