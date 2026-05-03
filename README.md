# Kurangi Ping 

Monorepo scaffold for a Windows desktop game routing optimizer.

## Workspace Layout

```text
.
|-- apps/
|   |-- desktop/             # Tauri + React + TypeScript desktop app
|   |   `-- README.md
|   `-- relay-controller/    # Go relay controller service
|       `-- README.md
|-- crates/
|   `-- client-engine/       # Rust client network engine crate
|       `-- README.md
|-- scripts/                 # Dev/build/test orchestration scripts
|   `-- README.md
`-- docs/                    # Product, technical, API, ERD, and story docs
    `-- README.md
```

## Conventions

- Separation of concerns is mandatory: UI (`apps/desktop`), client engine (`crates/client-engine`), relay service (`apps/relay-controller`).
- Scaffold stays minimal in this phase and avoids production logic.
- Naming follows the approved stack and architecture documentation in `docs/tech-stack.md`.

## Current Story Scope

This baseline satisfies `KP-001` workspace bootstrap requirements only.
Subsequent stories add environment contracts, runtime skeletons, orchestration scripts, and CI.
