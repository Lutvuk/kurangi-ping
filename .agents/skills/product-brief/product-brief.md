# Product Brief — Kurangi Ping
> Version 1.0 · May 2026 · Status: DRAFT

---

## 1. Problem Statement

Gamers di negara-negara Asia Tenggara (khususnya Indonesia) yang bermain game online dengan server di luar negeri — seperti Final Fantasy XIV (NA/EU server), World of Warcraft, atau GTA Online — mengalami latensi tinggi (ping 200–500ms+) akibat jarak geografis yang jauh dari server game.

Solusi komersial seperti **NoPing**, WTFast, dan Exitlag menyelesaikan masalah ini, tetapi:
- Berbayar (NoPing ~Rp 100.000–300.000/bulan)
- Tidak terjangkau untuk segmen gamer pelajar/mahasiswa
- Tidak transparan soal data privasi pengguna

**Tidak ada alternatif gratis, open, dan aman yang setara secara kualitas.**

---

## 2. Vision & Goals

**Visi:** Membuat pengalaman bermain game online lintas server menjadi adil dan terjangkau untuk semua orang, tanpa biaya berlangganan.

**Misi:** Membangun aplikasi routing pintar gratis yang mengurangi ping secara signifikan, mudah digunakan siapa saja, dan 100% menghormati privasi pengguna.

---

## 3. Target Users & Personas

### Persona Utama — "Gamer Kasual Indonesia"
| Atribut | Detail |
|---|---|
| Usia | 16–30 tahun |
| Lokasi | Indonesia (Jakarta, Surabaya, Medan, dll.) |
| Game | FFXIV (NA/JP server), Valorant (SG/JP), GTA Online, Lost Ark |
| Perangkat | Windows 10/11 PC atau laptop gaming entry-level |
| Kemampuan teknis | Rendah–menengah (tidak paham VPN/routing) |
| Pain point | Ping >200ms, lag saat raid/pvp, tidak mampu bayar NoPing |
| Motivasi | Mau main lancar tanpa keluar uang bulanan |

### Persona Sekunder — "Gamer SE Asia Budget-Conscious"
Filipina, Vietnam, Thailand yang menghadapi masalah serupa dengan server JP/NA.

### Bukan Target Saat Ini
- Enterprise / game studio
- Pengguna macOS / Linux (fase berikutnya)
- Pengguna yang butuh VPN untuk privasi umum (bukan game)

---

## 4. Scope

### ✅ Dalam Scope (v1.0)
- Aplikasi desktop Windows (10/11)
- Smart routing otomatis ke relay node terdekat
- Auto-detect game yang sedang berjalan
- Dashboard ping real-time (before/after)
- UI sederhana: satu tombol ON/OFF
- Dukungan game prioritas: FFXIV, Valorant, GTA Online, World of Warcraft, Star Wars The Old Republic
- Zero-log policy (tidak menyimpan data jaringan pengguna)
- Distribusi gratis, open-source (GitHub)

### ❌ Di Luar Scope (v1.0)
- macOS / Linux support
- Mobile (Android/iOS)
- Custom routing manual oleh user
- Fitur premium / monetisasi
- Dukungan game >10 judul di launch
- VPN general-purpose (bukan game)

---

## 5. Success Metrics (KPIs)

| Metrik | Target v1.0 (3 bulan post-launch) |
|---|---|
| Rata-rata pengurangan ping | ≥30% dari baseline user |
| Ping ke NA server dari Indonesia | <120ms (dari rata-rata 220ms+) |
| User downloads | 1.000 dalam 30 hari pertama |
| Crash rate | <2% per sesi |
| Onboarding time (install → connected) | <3 menit |
| Retensi 7 hari | ≥50% |
| Laporan kebocoran data | 0 insiden |

---

## 6. Risks & Constraints

### Risiko Teknis
| Risiko | Tingkat | Mitigasi |
|---|---|---|
| Relay node bandwidth terbatas (gratis) | Tinggi | Gunakan Cloudflare Tunnel, Fly.io free tier, atau relay komunitas |
| ISP Indonesia melakukan throttling ke relay luar negeri | Menengah | Multi-hop routing, pilih relay yang tidak di-throttle |
| Deteksi anti-cheat salah flag app | Menengah | Routing di level OS/network, tidak inject ke process game |
| Keamanan relay node diretas | Tinggi | E2E enkripsi, zero-log, audit publik |

### Constraints
- **Budget:** Rp 0 (gratis sepenuhnya — infrastruktur dari free tier cloud & open source)
- **Tim:** Solo developer (kamu) + kontribusi komunitas
- **Teknologi:** Harus bisa di-build oleh 1 orang, stack yang familiar
- **Compliance:** GDPR-lite (tidak kumpulkan data apapun), Terms of Service game tidak boleh dilanggar
- **Timeline:** MVP dalam 3–4 bulan dari kick-off

---

## 7. Stakeholders & Decision Owners

| Peran | Nama/Entitas | Tanggung Jawab |
|---|---|---|
| Product Owner / Developer | Kamu | Semua keputusan produk & teknis |
| Early Testers | Komunitas gamer Discord/Reddit ID | Feedback UI dan performa routing |
| Relay Infrastructure | Fly.io, Cloudflare (free tier) | Hosting relay node |
| Open Source Contributors | GitHub Community | Bug fix, tambahan game support |

---

## 8. Milestones & Timeline

```
Bulan 1  [Research & Architecture]
├── Minggu 1–2: Riset routing protokol (WireGuard/QUIC vs TCP relay)
├── Minggu 3: Setup relay node pertama (Fly.io Singapore)
└── Minggu 4: Proof-of-concept CLI — ping reduction terukur

Bulan 2  [Core Build]
├── Minggu 5–6: Windows tray app + auto game detection
├── Minggu 7: Dashboard ping real-time
└── Minggu 8: UI/UX polish — onboarding flow

Bulan 3  [Testing & Hardening]
├── Minggu 9–10: Closed beta (50 tester Discord)
├── Minggu 11: Security audit & zero-log validation
└── Minggu 12: Bug fix dari beta feedback

Bulan 4  [Launch]
├── Minggu 13: Public GitHub release + README
├── Minggu 14: Posting di Reddit r/ffxiv, r/indogamer, komunitas Discord
└── Minggu 15–16: Monitor, hotfix, iterasi berdasar feedback
```

**Target Launch Window:** Q3 2026 (Agustus–September 2026)

---

## 9. Open Questions

| # | Pertanyaan | Priority | Owner |
|---|---|---|---|
| Q1 | Protokol routing mana yang paling efektif: WireGuard, QUIC, atau custom TCP relay? | 🔴 Tinggi | Kamu |
| Q2 | Bagaimana cara mendapat relay node gratis di region NA & JP yang stabil? | 🔴 Tinggi | Kamu |
| Q3 | Apakah perlu mekanisme update otomatis (auto-updater)? | 🟡 Menengah | Kamu |
| Q4 | Apakah akan open-source penuh (MIT) atau source-available? | 🟡 Menengah | Kamu |
| Q5 | Bagaimana deteksi game berjalan tanpa inject ke proses? (Process listing?) | 🟡 Menengah | Kamu |
| Q6 | Apakah butuh akun/login, atau sepenuhnya anonymous? | 🟢 Rendah | Kamu |

---

*Brief ini dibuat berdasarkan informasi awal. Harap diperbarui setiap ada keputusan besar yang dibuat.*
