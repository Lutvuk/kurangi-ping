# Desktop App Module

This module hosts the Tauri + React + TypeScript desktop application.

## KP-003 Baseline

- Frontend entry: `src/main.tsx`
- Root component: `src/App.tsx`
- Tauri runtime: `src-tauri/`

## Local Development

Prerequisites:
- Node.js 22+
- pnpm 10+
- Rust toolchain (required for `tauri:dev` / `tauri:build`)

Commands:

```powershell
pnpm install
pnpm build
pnpm tauri:dev
```

Notes:
- `pnpm build` validates strict TypeScript and Vite build.
- `pnpm tauri:dev` starts Tauri development runtime with the Vite frontend.
