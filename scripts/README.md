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
pnpm db:telemetry:prune
pnpm db:telemetry:prune:test
./scripts/ci-verify-db.ps1
./scripts/ci-verify-signing.ps1 -Channel stable -ArtifactPath <artifact-path> -SignaturePath <artifact-path>.sig.json
```

- `pnpm db:migrate` applies pending SQLite `*.up.sql` files in lexical order.
- Applied versions are tracked in `schema_migrations`.
- `pnpm db:migrate:test` runs idempotency smoke test for the migration runner.
- `pnpm db:seed` applies the catalog seed file selected by `db/seeds/catalog_version.json`.
- `pnpm db:seed:test` verifies idempotent re-run behavior, update path safety, and catalog version tracking output.
- `pnpm db:telemetry:prune` prunes expired telemetry batches/events and prints retention summary metrics.
- `pnpm db:telemetry:prune:test` validates retention pruning policy (preserve retryable rows + idempotent re-run).
- `./scripts/ci-verify-db.ps1` runs CI-style fresh DB verification (migrate + idempotency + constraint/index checks, plus optional seed checks).
- `./scripts/ci-verify-signing.ps1` enforces release signing policy (`stable` always signed, `beta` configurable via `-BetaSigningPolicy`).
