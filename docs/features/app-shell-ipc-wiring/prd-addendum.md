# PRD Addendum - App Shell IPC Wiring
> Feature: App Shell IPC Wiring
> Date: 2026-05-07
> Status: Proposed

---

## 1. Feature Metadata

| Field | Value |
|---|---|
| Feature Name | App Shell IPC Wiring |
| Feature Slug | app-shell-ipc-wiring |
| Parent Epic | EPIC-WIRING |
| Status | Proposed |
| Owner | Product Owner / Solo Developer |
| Related Global Docs | docs/prd.md, docs/tech-stack.md, docs/features/tauri-command-bridge/prd-addendum.md |

---

## 2. Requirements

| ID | User Story | Acceptance Criteria |
|---|---|---|
| US-SHELL-IPC-01 | As a user, I want ON/OFF toggle in app shell to call existing Rust IPC commands so production app can drive real routing lifecycle. | Given app berjalan di production mode, when user klik ON, then frontend invoke `routing_toggle_on`, menerima response dari Rust, dan badge berubah dari `offline` ke `connecting`. |
| US-SHELL-IPC-02 | As a user, I want shell badge to auto-sync with Rust routing events so status selalu real-time tanpa reload. | Given routing state berubah di Rust engine, when Rust emit `routing_state_changed`, then frontend listen event dan `ConnectionStatusBadge` update otomatis sesuai state terbaru. |
| US-SHELL-IPC-03 | As a user, I want detection status queried at startup so saya langsung tahu game terdeteksi atau tidak. | Given app baru dibuka, when app shell selesai mount, then frontend invoke `detection_get_status` dan hasilnya muncul di `DetectionPanel` (`detected` / `not_detected`). |

---

## 3. ERD Delta

No ERD changes.

Data usage notes:
- Feature ini hanya wiring state runtime frontend <-> Rust IPC.
- Tidak menambah tabel/kolom DB baru.
- [ASSUMPTION] Persistensi tetap mengikuti skema existing tanpa migration tambahan.

---

## 4. API Contract Delta

No external HTTP API delta.

Internal desktop IPC contract usage (existing, no new Rust command/event):
- Commands (frontend -> Rust):
  - `routing_toggle_on`
  - `routing_toggle_off`
  - `detection_get_status`
- Events (Rust -> frontend):
  - `routing_state_changed`
  - `detection_status_updated`
  - `metrics_ping_sampled`

---

## 5. Integration Notes

- Wiring utama dilakukan di app shell composition (`App.tsx`) dan frontend IPC adapter existing (`src/lib/ipc/client.ts`).
- App shell dan komponen turunan wajib memakai adapter methods; tidak boleh hardcode command/event string.
- Listener lifecycle harus deterministic:
  - subscribe saat mount,
  - cleanup saat unmount,
  - tidak boleh duplicate handler saat remount/re-render.
- Error invoke/listen dipetakan ke reason code UI-safe (`ipc_*`) tanpa bocor detail internal.
- Detection startup query wajib idempotent untuk dev hot-reload agar tidak menumpuk request liar.

---

## 6. Open Questions

| ID | Question | Priority |
|---|---|---|
| SHELL-IPC-OQ-01 | Perlu fallback polling detection jika event stream unavailable di kondisi tertentu? | Medium |
| SHELL-IPC-OQ-02 | Perlu UI indikator khusus saat IPC listener gagal attach (`ipc_listener_unavailable`)? | Medium |
| SHELL-IPC-OQ-03 | Perlu batching/rate-limit visual update untuk `metrics_ping_sampled` bila frekuensi tinggi? | Low |
