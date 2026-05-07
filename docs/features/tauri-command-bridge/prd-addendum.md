# PRD Addendum - Tauri IPC Command Bridge
> Feature: Tauri IPC Command Bridge
> Date: 2026-05-07
> Status: Proposed

---

## 1. Feature Metadata

| Field | Value |
|---|---|
| Feature Name | Tauri IPC Command Bridge |
| Feature Slug | tauri-command-bridge |
| Parent Epic | EPIC-WIRING |
| Status | Proposed |
| Owner | Product Owner / Solo Developer |
| Related Global Docs | docs/prd.md, docs/tech-stack.md, docs/erd/core-erd.md, docs/api-standards.md |

---

## 2. Requirements

| ID | User Story | Acceptance Criteria |
|---|---|---|
| US-IPC-01 | As a user, I want routing toggle in UI to trigger Rust routing lifecycle via Tauri command so connect flow is end-to-end wired. | Given app berjalan dan game terdeteksi, when user klik ON, then frontend invoke command IPC ke Rust, Rust memulai routing pipeline, dan UI state berubah `connecting` lalu `active` sesuai transition valid. |
| US-IPC-02 | As a user, I want live ping updates from Rust to UI without refresh so I can monitor optimization in real time. | Given routing aktif, when Rust selesai siklus pengukuran ping, then event IPC dipush ke frontend dan `PingMetricsPanel` update data terbaru tanpa reload halaman. |
| US-IPC-03 | As a user, I want detection status query on app start so I immediately know whether my game is detected. | Given app baru dibuka, when frontend request detection status via IPC, then Rust scanner proses Windows dijalankan dan `DetectionPanel` menampilkan `detected` atau `not_detected` secara konsisten. |

---

## 3. ERD Delta

No new entities are mandatory.

Data usage notes:
- Bridge ini fokus pada transport command/event antara frontend dan Rust engine.
- Persistensi tetap memakai skema existing (`RouteSession`, `PingSample`, `TelemetryEvent`) tanpa tabel baru.
- [ASSUMPTION] Tidak ada kolom DB baru untuk KP initial bridge wiring.

---

## 4. API Contract Delta

No external HTTP endpoint is introduced.

Internal IPC contract notes (desktop local boundary):
- Command group (frontend -> Rust):
  - `routing_toggle_on`
  - `routing_toggle_off`
  - `detection_get_status`
- Event group (Rust -> frontend):
  - `metrics_ping_sampled`
  - `routing_state_changed`
  - `detection_status_updated`

[ASSUMPTION] Nama command/event final akan dinormalisasi ke naming convention existing saat story decomposition.

---

## 5. Integration Notes

- Bridge wajib menjaga separation:
  - React/TS hanya invoke command + subscribe event.
  - Rust client-engine tetap jadi source of truth state machine.
- Error IPC harus dipetakan ke reason code non-sensitive (selaras taxonomy updater/routing existing).
- Event payload harus deterministic dan schema-safe untuk mencegah UI drift.
- State transition UI tidak boleh bypass guard state machine Rust.

---

## 6. Open Questions

| ID | Question | Priority |
|---|---|---|
| IPC-OQ-01 | Naming convention final untuk command/event: snake_case vs dotted domain style? | Medium |
| IPC-OQ-02 | Apakah `detection_get_status` hanya pull-once saat startup atau juga interval polling fallback? | Medium |
| IPC-OQ-03 | Perlu ack/receipt event untuk menjamin delivery saat window UI freeze/reload? | Low |
