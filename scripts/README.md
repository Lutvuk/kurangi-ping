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
  - Runs local DB migrations before service startup.
  - Optional seed step can be enabled with `KP_DB_SEED_ON_BOOTSTRAP=1` or by running `scripts/dev.ps1 -Seed`.
  - Starts relay-controller in background.
  - Starts desktop Tauri development runtime in foreground.
  - Writes relay logs to `scripts/logs/`.
- `pnpm test`:
  - Runs desktop UI tests (`vitest`), Rust crate tests, and Go tests.
- `pnpm build`:
  - Runs desktop production build, Rust crate build, and Go build.

## Database Migration

```powershell
pnpm db:migrate
pnpm db:migrate:test
pnpm db:seed
pnpm db:seed:test
./scripts/ci-verify-db.ps1
```

- `pnpm db:migrate` applies pending SQLite `*.up.sql` files in lexical order.
- Applied versions are tracked in `schema_migrations`.
- `pnpm db:migrate:test` runs idempotency smoke test for the migration runner.
- `pnpm db:seed` applies baseline SQL seed files in lexical order.
- `pnpm db:seed:test` verifies idempotent re-run behavior and baseline rows.
- `./scripts/ci-verify-db.ps1` runs CI-style fresh DB verification (migrate + idempotency + constraint/index checks, plus optional seed checks).
