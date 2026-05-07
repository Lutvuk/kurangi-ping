# KP-120: Routing State Event Subscription and Badge Sync

**Epic:** EPIC-WIRING
**Layer:** L4-feature-ui
**Role:** Frontend
**Estimation:** 2
**Priority:** Must

---

## Objective
Wire `routing_state_changed` event subscription in app shell so `ConnectionStatusBadge` selalu sinkron dengan state Rust terbaru tanpa reload.

## Technical Specifications
*   **Proposed Files:** `apps/desktop/src/App.tsx`, `apps/desktop/src/features/shell/useAppShellIpcState.ts`
*   **Functions/Classes:** `bindRoutingStateListener`, `applyRoutingStateEvent`
*   **API Endpoints:** N/A
*   **Data Models:** `RoutingStateEventPayload`, `ConnectionStatusViewModel`

## Acceptance Criteria (Technical)
*   [x] App shell subscribe ke `routing_state_changed` saat mount.
*   [x] Payload event valid memutakhirkan state badge (`offline/connecting/active/error`) secara deterministic.
*   [x] Out-of-order/duplicate event tidak membuat state loop atau UI flicker.
*   [x] Error listener di-normalize ke reason code UI-safe.

## Business Rules & Logic
*   Badge harus mencerminkan source-of-truth dari Rust engine, bukan asumsi frontend lokal.

## Dependencies
*   Depends on: KP-118, KP-114, KP-112

## Definition of Done
*   [x] Code implemented
*   [x] Unit tests passing
*   [x] Lint/Type check clear
