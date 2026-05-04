# SQLite Migrations

Deterministic migration conventions for Kurangi Ping.

## Naming Convention

- Apply files: `<version>_<name>.up.sql`
- Rollback files (reserved): `<version>_<name>.down.sql`
- Version format: zero-padded lexical sequence, example `0001`, `0002`

Examples:

- `0001_init_schema.up.sql`
- `0001_init_schema.down.sql`

## Ordering Rule

- Runner applies pending `*.up.sql` files in lexical filename order.
- Applied versions are tracked in `schema_migrations`.
- Re-running migrations is idempotent; already-applied versions are skipped.

## Metadata Table

`schema_migrations` stores:

- `version` (TEXT, PRIMARY KEY)
- `filename` (TEXT)
- `applied_at` (UTC timestamp)

## Execution

From repo root:

```powershell
pnpm db:migrate
```
