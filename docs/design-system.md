# Design System - Kurangi Ping
> Version 1.0 · May 2026
> Tone: Tactical · Fast · Trustworthy
> Theme: Dark-ops network console

---

## Filosofi Visual

Bukan "gaming app yang keren" - tapi **monitoring tool yang serius**.
Seperti Bloomberg Terminal bertemu game HUD. Data harus terbaca dalam 0.5 detik,
bahkan saat user panik sebelum raid dimulai.

**Tiga prinsip desain:**
1. **Data dulu, dekorasi belakangan** - setiap elemen harus justify kehadirannya
2. **Warna = informasi** - teal hanya untuk "live/connected", amber hanya untuk "measuring/degraded"
3. **Ketajaman bermakna** - komponen data = sharp corner, komponen interaktif = sedikit radius

---

## 1. Color Tokens

### Brand Palette
```css
--color-base          #0A0E0F   /* Near-black dengan cyan tint - "monitor glow" */
--color-surface-01    #111619   /* Elevation 1: cards, sidebar */
--color-surface-02    #181E22   /* Elevation 2: dropdown, hover */
--color-surface-03    #1F272C   /* Elevation 3: modal, overlay */
```

### Signal Colors (pakai SANGAT SPARING)
```css
--color-signal        #00D4B8   /* Teal - HANYA untuk: connected, active, live */
--color-signal-dim    rgba(0,212,184,0.08)   /* Surface tint saat routing ON */
--color-signal-border rgba(0,212,184,0.20)   /* Border saat state active */

--color-probe         #E8A020   /* Amber - HANYA untuk: measuring, degraded, transitioning */
--color-probe-dim     rgba(232,160,32,0.08)
```

### Semantic (Relay Health)
```css
--color-relay-ok      #3DD68C   /* Relay sehat < 50ms */
--color-relay-warn    #E8A020   /* Relay degraded 50-150ms (sama dengan probe) */
--color-relay-dead    #E84040   /* Relay offline / timeout */
```

### Text
```css
--color-text-primary     #E8EDF0   /* Konten utama */
--color-text-secondary   #7A909C   /* Label, metadata */
--color-text-muted       #3D5060   /* Placeholder, disabled, divider label */
--color-text-signal      #00D4B8   /* Data saat state: connected */
--color-text-probe       #E8A020   /* Data saat state: measuring */
```

### Border
```css
--color-border-subtle    #1F2D35   /* Divider halus */
--color-border-default   #253540   /* Border komponen default */
--color-border-strong    #34485A   /* Border komponen fokus/hover */
```

### Aturan Warna (WAJIB DIIKUTI)
- `--color-signal` hanya muncul saat routing = ON. Tidak boleh sebagai dekorasi.
- `--color-probe` hanya muncul saat sedang mengukur atau relay degraded.
- Jangan campurkan signal dan probe dalam satu komponen yang sama.
- Background app selalu `--color-base`. Tidak ada komponen yang lebih gelap dari ini.

---

## 2. Typography

### Font Stack
```txt
Display  : 'Rajdhani', sans-serif      /* Heading, label section, toggle text */
Body     : 'DM Sans', sans-serif       /* Semua body copy, navigasi, deskripsi */
Mono     : 'JetBrains Mono', monospace /* Angka ping, IP, hostname, log output */
```

**Kenapa pilihan ini terasa "human-made":**
- Rajdhani: font geometrik yang dipakai di aplikasi militer dan game HUD nyata.
  Tidak sci-fi seperti Orbitron, tidak generic seperti Roboto.
- DM Sans: sedikit humanist di letterform-nya - terasa warm tapi profesional.
- JetBrains Mono: standar industri developer, tapi juga visual authority untuk data network.
- JANGAN PAKAI: Inter, Roboto, Arial, Space Grotesk, Oxanium

### Scale
```txt
/* Display (Rajdhani) */
--text-display-lg    : 700 24px/1.1  Rajdhani   /* App logo, section besar */
--text-display-md    : 600 18px/1.2  Rajdhani   /* Toggle label, modal title */
--text-display-sm    : 500 14px/1.3  Rajdhani   /* Card heading */

/* Body (DM Sans) */
--text-body-md       : 400 14px/1.6  DM Sans    /* Deskripsi, konten utama */
--text-body-sm       : 400 12px/1.5  DM Sans    /* Secondary info, metadata */
--text-label         : 500 10px/1.4  DM Sans    /* Label uppercase, nav item */
/* letter-spacing: 0.08em untuk --text-label */

/* Mono (JetBrains Mono) */
--text-mono-xl       : 600 32px/1.0  JetBrains Mono  /* Angka ping utama */
--text-mono-lg       : 600 20px/1.0  JetBrains Mono  /* Angka sekunder */
--text-mono-md       : 400 12px/1.4  JetBrains Mono  /* Hostname, IP, server */
--text-mono-sm       : 400 10px/1.4  JetBrains Mono  /* Log, detail teknis */
```

---

## 3. Spacing Scale

Skala ini tidak murni 4px grid - ada "character sizes" yang spesifik untuk domain ini:
```txt
--space-1    : 4px
--space-2    : 8px
--space-3    : 12px
--space-4    : 16px
--space-5    : 20px
--space-6    : 28px    /* Bukan 24 - sedikit lebih lega untuk section break */
--space-7    : 40px    /* Bukan 32/48 - sweet spot untuk padding panel utama */
--space-8    : 64px
```

---

## 4. Radius, Border, Shadow, Z-Index

### Border Radius
```txt
--radius-none   : 0px     /* Data cards, metric display - data punya "edge" */
--radius-xs     : 2px     /* Badge, tag, status indicator kecil */
--radius-sm     : 4px     /* Toggle button, interactive element utama */
--radius-md     : 6px     /* Dropdown, tooltip */
--radius-lg     : 10px    /* App window, modal */
```

**Aturan:** Semakin data-heavy sebuah komponen, semakin tajam corner-nya.
Toggle = 4px. Ping number card = 0px. Modal = 10px.

### Border
```txt
--border-subtle  : 1px solid var(--color-border-subtle)
--border-default : 1px solid var(--color-border-default)
--border-strong  : 1px solid var(--color-border-strong)
--border-signal  : 1px solid var(--color-signal-border)   /* Hanya saat active */
--border-accent  : 2px solid var(--color-signal)          /* HANYA toggle button active state */
```

### Shadow
```txt
--shadow-none    : none                                    /* Default semua elemen */
--shadow-overlay : 0 8px 32px rgba(0,0,0,0.6)             /* Modal saja */
```
Tidak ada drop shadow pada komponen biasa. Elevation disampaikan lewat warna surface, bukan shadow.

### Z-Index
```txt
--z-base      : 0
--z-dropdown  : 100
--z-toast     : 200
--z-modal     : 300
--z-overlay   : 290     /* Di bawah modal, di atas semua */
```

---

## 5. Breakpoints & Motion

### Breakpoints
App ini desktop-only (v1). Satu breakpoint untuk compact panel:
```txt
--bp-compact  : 960px    /* Sidebar collapse ke icon-only */
--bp-full     : 1280px   /* Layout penuh dengan relay panel terbuka */
```

### Motion
```txt
/* Duration */
--duration-instant  : 80ms    /* Toggle state snap - terasa "commit", bukan animate */
--duration-fast     : 150ms   /* Hover state, badge change */
--duration-normal   : 240ms   /* Panel slide, dropdown open */
--duration-slow     : 400ms   /* Onboarding step transition */

/* Easing */
--ease-sharp    : cubic-bezier(0.2, 0, 0, 1)          /* Toggle ON/OFF - decisive */
--ease-out      : cubic-bezier(0, 0, 0.2, 1)          /* Dropdown expand */
--ease-bounce   : cubic-bezier(0.34, 1.56, 0.64, 1)   /* Onboarding success state */
```

**Aturan motion:** Toggle ON/OFF menggunakan `--duration-instant` dengan `--ease-sharp`.
Ini membuat aksi terasa seperti "commit" bukan "slide". Jangan animate ping number - langsung snap.

---

## 6. Core Components

### App Shell
```txt
Sidebar: 180px fixed, background: --color-surface-01
Nav item: height 36px, font: --text-label, uppercase, letter-spacing 0.08em
Active state: border-left 2px solid --color-signal, background rgba(signal, 0.04)
Top bar: height 40px, background: --color-surface-01, border-bottom: --border-subtle
```

### Primary Toggle (ON/OFF Routing)
```txt
Size: 72px x 72px
Border-radius: --radius-sm (4px)
OFF state:
  border: --border-default
  background: transparent
  label color: --color-text-muted
ON state:
  border: 2px solid --color-signal
  background: --color-signal-dim
  label color: --color-signal
  corner bracket ornament: 6px x 6px, warna --color-signal
Transition: --duration-instant, --ease-sharp
Font: --text-display-md (Rajdhani 700)
```

**Catatan desain:** Corner bracket ornament (4 sudut) adalah satu-satunya dekorasi yang
diizinkan di komponen ini. Bukan karena estetika - karena memberi batas visual tegas
yang membantu user fokus pada action saat mau raid.

### Connection Status Badge
```txt
Padding: 3px 10px
Border-radius: --radius-xs (2px)
Font: --text-label uppercase, letter-spacing 0.06em
OFF: color --color-text-muted, background rgba(muted, 0.15), border --border-subtle
ON:  color --color-signal, background --color-signal-dim, border --color-signal-border
Pulse indicator: 6px circle, animasi opacity 2s ease-in-out infinite
```

### Ping Metric Card
```txt
Layout: 3-column grid, gap --space-3
Background: --color-surface-01
Border: --border-subtle
Border-radius: --radius-none (0px) - data harus terasa "sharp"
Padding: 12px 14px
Label: --text-label, color --color-text-muted, uppercase
Value: --text-mono-xl (32px), color berdasarkan state:
  - Current ping saat routing ON: --color-signal
  - Baseline (angka lama): --color-probe
  - Reduction %: --color-signal
Unit: --text-body-sm, color --color-text-muted
```

### Relay Health List Item
```txt
Height: 36px
Layout: [6px indicator] [hostname mono-md] [ms value mono-md] [region label]
Active relay: background --color-signal-dim, extend full width (negative margin)
Indicator: 6px circle, warna sesuai relay health semantic
MS value color: sama dengan indicator
Hostname color: --color-text-secondary (default), --color-text-primary (active)
```

### Game Detection Row
```txt
Background: --color-surface-01
Border: --border-subtle
Border-radius: --radius-xs (2px)
Padding: 10px 14px
Game icon: 28px x 28px, background --color-surface-03, border-radius --radius-xs
Game name: --text-body-md, color --color-text-primary
Server info: --text-mono-sm, color --color-text-secondary
Status badge: "Detected" - color --color-signal, background --color-signal-dim
```

### Toast / Alert
```txt
Width: 320px, fixed bottom-right
Border-radius: --radius-xs (2px)
Padding: 12px 16px
Border-left: 3px solid (semantic color)
Background: --color-surface-02
Auto-dismiss: 4000ms
Types: info (signal), warning (probe), error (relay-dead), success (relay-ok)
```

### Modal (Confirm Disconnect / Update)
```txt
Overlay: rgba(0,0,0,0.7)
Panel: --color-surface-02, border --border-default, border-radius --radius-lg
Padding: 24px
Title: --text-display-md (Rajdhani 600)
Body: --text-body-md
Buttons: lihat Button primitives
```

### Onboarding Stepper
```txt
Steps: numbered, horizontal flow
Inactive step: number + label, color --color-text-muted
Active step: color --color-text-primary, underline border-bottom --border-signal
Complete step: number ganti ke checkmark, color --color-relay-ok
Connector line: 1px dashed --color-border-subtle
Transition: --duration-slow, --ease-out
```

### Button Primitives
```txt
Primary:
  background: --color-signal
  color: --color-base (dark text di atas teal)
  border: none
  border-radius: --radius-xs (2px)
  padding: 8px 20px
  font: --text-label, uppercase, letter-spacing 0.08em

Secondary:
  background: transparent
  color: --color-text-primary
  border: --border-default
  hover: border --border-strong, background --color-surface-02

Destructive:
  background: transparent
  color: --color-relay-dead
  border: 1px solid --color-relay-dead
  hover: background rgba(relay-dead, 0.08)
```

### Input & Select Primitives
```txt
Height: 36px
Background: --color-surface-01
Border: --border-default
Border-radius: --radius-xs (2px)
Font: --text-body-md, color --color-text-primary
Focus: border --border-signal, outline: none
Placeholder: color --color-text-muted
```

---

## 7. Patterns

### One-Click Connect Flow
```txt
1. App launch -> game auto-detected -> banner "FFXIV terdeteksi"
2. User klik toggle (OFF -> ON)
3. Badge animasi: "Connecting..." dengan probe color
4. Route established -> badge snap ke "Routing Active" + signal color
5. Ping metric muncul: baseline vs current, dalam < 2 detik
Waktu total target: < 3 detik dari klik ke angka muncul
```

### Relay Failover Feedback Flow
```txt
1. Relay aktif jadi unreachable
2. Toast muncul: "sin-01 unreachable - switching to nrt-01"
3. Badge sementara: probe color "Reconnecting..."
4. Failover berhasil -> badge kembali signal color, relay list update aktif node
5. Jika semua relay gagal -> badge merah "No relay available", modal disconnect
```

### First-Run Onboarding Flow
```txt
Step 1: Welcome + ringkasan apa yang akan terjadi (< 30 kata)
Step 2: Permission check (Windows route table access)
Step 3: Relay test - otomatis ping semua node, pilih terbaik
Step 4: Game detection test - "Buka game kamu sekarang"
Step 5: First connect + lihat angka ping pertama kali
Tidak ada akun, tidak ada email, tidak ada form panjang.
```

### Error Recovery Flow
```txt
Scenario A - No relay available:
  Toast error -> modal dengan dua opsi: Retry / Continue without routing

Scenario B - Permission denied (Windows route table):
  Layar khusus dengan instruksi UAC step-by-step
  Tombol: "Restart as Administrator"

Scenario C - Game not detected:
  Banner info: "Game tidak terdeteksi - pastikan game sudah berjalan"
  Tombol: "Scan ulang" + link manual game list
```

---

## 8. Yang Membuat Ini Terasa Bukan AI-Generated

Hal-hal yang sengaja TIDAK dilakukan (yang biasanya dilakukan AI):
- Tidak semua corner radius sama - tajam untuk data, rounded untuk interaksi
- Tidak ada purple/gradient - zero
- Teal TIDAK dipakai sebagai warna brand umum, hanya untuk live state
- Spacing tidak murni 4px grid - ada 28px dan 40px yang "off-grid" dengan alasan
- Ping number TIDAK animate - langsung snap, karena ini monitoring tool bukan dashboard marketing
- Toggle tidak slide smooth - snap dengan easing sharp, terasa "commit"
- Tidak ada shadow pada card - elevation via warna, bukan depth effect
- Font Rajdhani dipilih karena dipakai di aplikasi militer nyata, bukan karena terlihat futuristik

---

*Design system ini hidup - update setiap ada keputusan visual baru yang dibuat.*
