# Product Requirements Document (PRD) - Kurangi Ping
> Version 1.0 · Status: Draft Approved · Last Updated: 2026-05-03

---

## 1) Overview

| Field | Value |
|---|---|
| Product Name | Kurangi Ping |
| Product Type | Windows desktop application (game routing optimizer) |
| Primary Market | Indonesia and Southeast Asia gamers |
| Owner | Solo Developer (Product + Engineering) |
| Team Model | Solo core + open-source contributors |
| Target Platforms | Windows 10/11 |
| Timeline | Internal Beta: August 2026 · Public Launch: October 2026 |

---

## 2) Quick Links

| Artifact | Link |
|---|---|
| Design | [TBD-Figma] |
| Tech Spec | [TBD-TechSpec] |
| Project Board | [TBD-Board] |

---

## 3) Background

Online gamers in Indonesia and SEA who connect to NA/EU/JP game servers often face high latency (commonly 200ms+), creating poor gameplay during raids, PvP, and real-time interactions. Existing tools solve some of this problem but are paid, difficult for student budgets, and often unclear about privacy handling.

Kurangi Ping aims to provide a free, open, privacy-first routing solution that measurably lowers latency without violating game terms or requiring advanced networking knowledge.

---

## 4) Objectives

### Business Objectives (Measurable)

1. Deliver measurable ping reduction for cross-region gaming sessions.
2. Reach meaningful early adoption in launch month.
3. Maintain stable and safe operation for daily gameplay use.
4. Keep onboarding friction low for non-technical users.
5. Maintain strict privacy posture with zero user-network logging.

### User Objectives

1. Connect and optimize route with one-click simplicity.
2. See clear before/after ping impact in real time.
3. Recover gracefully when a relay path fails.
4. Use the app confidently without privacy concerns.

---

## 5) Success Metrics

| Metric | Baseline | Target | Measurement Method |
|---|---|---|---|
| Average ping reduction | 0% (no optimizer) | >=30% reduction | Session-based before/after ping sampling |
| Typical ID -> NA ping | ~220ms+ | <120ms for supported routes | Regional benchmark tests + user telemetry aggregates |
| Launch-month downloads | 0 | >=1,000 in first 30 days | GitHub release download analytics |
| Crash rate per session | Unknown pre-launch | <2% | Crash event ratio over active sessions |
| Onboarding time (install -> first connected) | Unknown pre-launch | <3 minutes median | Onboarding funnel timestamps |
| D7 retention | 0 | >=50% | Anonymous cohort retention tracking |
| Privacy incidents | 0 | 0 | Security reports + issue triage |

---

## 6) Scope

### In Scope (v1.0)

- Windows 10/11 desktop application
- Automatic smart routing to selected relay nodes
- Auto-detection of supported running games
- Real-time ping dashboard (before/after)
- Simple ON/OFF routing control
- Initial game support: FFXIV, Valorant, GTA Online, World of Warcraft, Star Wars: The Old Republic
- Zero-log policy for user network traffic
- Free and open-source distribution

### Out of Scope (v1.0)

- macOS or Linux client
- Mobile platforms (Android/iOS)
- Fully manual custom route editing by users
- Premium paid features or subscriptions
- Broad support for more than 10 game titles at launch
- General-purpose consumer VPN use-cases

---

## 7) User Flow

### Primary Journey

```text
1. User installs Kurangi Ping on Windows 10/11
2. User launches app and sees onboarding
3. App detects supported game process (or waits until game starts)
4. User enables routing via ON button
5. App selects best relay path and establishes connection
6. User sees real-time before/after ping metrics
7. User can disable routing any time and exit safely
```

### Alternative Flows

```text
A1. Game not detected:
- App shows supported game list + detection troubleshooting
- User can retry scan while game is running

A2. Relay unavailable:
- App emits relay_failed event
- App auto-attempts fallback relay candidates
- User is notified with non-technical status message
```

### Edge Cases

```text
E1. Anti-cheat sensitivity:
- Routing implementation must remain process-external (OS/network level)

E2. Free-tier relay instability:
- Health checks and failover logic required

E3. High packet loss despite lower ping:
- Surface packet loss/jitter indicators, not only ping
```

---

## 8) User Stories

| ID | User Story | Acceptance Criteria |
|---|---|---|
| US-01 | As a gamer, I want one-click routing so that I can optimize without networking knowledge. | Given app is open, when user clicks ON, then routing starts and status becomes connected. |
| US-02 | As a gamer, I want supported games detected automatically so that I do not configure processes manually. | Given a supported game is running, when scanner executes, then game appears as detected within acceptable delay. |
| US-03 | As a gamer, I want before/after ping shown live so that I can verify improvement. | Given routing state changes, when ping probes run, then dashboard shows baseline and optimized metrics. |
| US-04 | As a gamer, I want safe OFF behavior so that I can revert network routing instantly. | Given routing is active, when user clicks OFF, then traffic reverts and app confirms disconnected state. |
| US-05 | As a user, I want onboarding to finish quickly so that I can start playing in under 3 minutes. | Given first launch, when user follows onboarding, then first successful connection is achieved in guided flow. |
| US-06 | As a user, I want clear relay failure feedback so that I know what happened and what the app is doing next. | Given relay connection fails, when failure is detected, then app shows status and attempts fallback route. |
| US-07 | As a privacy-conscious user, I want zero-log behavior so that my traffic data is not stored. | Given app routing runs, when telemetry is recorded, then no payload/content or identifiable traffic logs are persisted. |
| US-08 | As a tester, I want crash reports captured anonymously so that reliability can improve quickly. | Given app crashes unexpectedly, when restart occurs, then anonymized crash event can be submitted by policy. |
| US-09 | As an operator, I want relay health checks so that unhealthy nodes are avoided automatically. | Given relay pool exists, when health score drops below threshold, then node is deprioritized or excluded. |
| US-10 | As a product owner, I want event analytics for onboarding and relay outcomes so that we can optimize activation and reliability. | Given user session events occur, when telemetry pipeline runs, then required events are emitted with expected schema. |

---

## 9) Analytics

| Event Name | Trigger | Key Data | Description |
|---|---|---|---|
| app_opened | App process starts | app_version, os_version, locale | Session start marker |
| game_detected | Supported game found | game_id, process_name, detection_time_ms | Detection success metric |
| routing_enabled | User clicks ON and route established | game_id, relay_id, baseline_ping_ms | Activation success |
| ping_measured | Ping probe cycle executes | baseline_ping_ms, routed_ping_ms, jitter_ms, packet_loss_pct | Performance telemetry |
| routing_disabled | User clicks OFF or disconnect requested | session_duration_s, reason | Deactivation behavior |
| crash_reported | Unhandled exception detected | app_version, module, error_type | Stability monitoring |
| relay_failed | Relay connection attempt fails | relay_id, region, error_code, retry_count | Reliability and failover insight |
| onboarding_completed | User reaches first connected state in onboarding | onboarding_duration_s, game_id, success_path | Activation funnel completion |

Example event payload format:

```json
{
  "trigger": "routing_enabled",
  "page": "main_dashboard",
  "data": {
    "game_id": "ffxiv",
    "relay_id": "sg-01",
    "baseline_ping_ms": 228
  },
  "description": "User successfully enables routing"
}
```

---

## 10) Open Questions

| ID | Question | Priority | Owner |
|---|---|---|---|
| OQ-01 | Which routing protocol should be primary for v1: WireGuard, QUIC tunnel, or TCP relay? | High | Product Owner |
| OQ-02 | What is the most stable free-tier relay strategy for NA and JP coverage? | High | Product Owner |
| OQ-03 | Should auto-updater be included in v1 or deferred to post-launch? | Medium | Product Owner |
| OQ-04 | What is final licensing model: full MIT open-source vs source-available? | Medium | Product Owner |
| OQ-05 | What is safest process/game detection implementation across anti-cheat policies? | Medium | Product Owner |
| OQ-06 | Is account/login needed later, or remain fully anonymous long-term? | Low | Product Owner |

---

## 11) Notes

- Budget constraint remains zero-cost infrastructure target.
- Architecture should prioritize components that a solo developer can maintain.
- Compliance baseline: privacy-by-design, no traffic content collection, clear policy docs.
- Product must not violate game terms; no process injection or cheat-like behavior.
- Beta feedback loops should prioritize connection reliability and onboarding clarity.

---

## 13) Foundation Delivery Log

### 13.1 Foundation Feature Registered: Project Scaffold
- Feature Code: FOUNDATION-001
- Source: `docs/features/foundation-project-scaffold/prd-addendum.md`
- Status: In Progress
- Story Set:
  - KP-001 Repo Workspace Bootstrap
  - KP-002 Environment Contract and Secrets Boundary
  - KP-003 Tauri + React + TypeScript UI Foundation Init
  - KP-004 Design Token and Theme Wiring Baseline
  - KP-005 Rust Client Engine Skeleton and Module Boundaries
  - KP-006 Go Relay Controller Skeleton and Health Endpoint Stub
  - KP-007 Desktop App Shell Layout and Status Regions
  - KP-008 Monorepo Task Orchestration Scripts
  - KP-009 CI Baseline Workflow Matrix (TS/Rust/Go)

### 13.2 Global Guardrails Reinforced
- Stack lock: Tauri/React/TS + Rust + Go
- Privacy guardrail: no PII telemetry fields
- Security guardrail: signed manifest integration path required
- Release guardrail: code-signing gate before public launch

### 13.3 Foundation Feature Registered: Base DB Schema
- Feature Code: FOUNDATION-002
- Source: `docs/features/foundation-base-db-schema/prd-addendum.md`
- Status: In Progress
- Story Set:
  - KP-010 SQLite Migration Framework Bootstrap
  - KP-011 Core Tables Creation Migration (v1_init)
  - KP-012 Constraint Enforcement (FK/CHECK/UTC timestamps)
  - KP-013 Baseline Indexing for Sessions and Telemetry
  - KP-014 Supported Games Seed Script (Idempotent)
  - KP-015 Rust DB Access Layer Skeleton (sqlite + migrations)
  - KP-016 Go Relay Controller DB Binding Stub
  - KP-017 Migration Execution in Local Dev Scripts
  - KP-018 CI Migration Verification Job

### 13.4 DB Guardrails Reinforced
- Schema source of truth: `docs/erd/core-erd.md`
- FK and CHECK constraints mandatory
- UTC timestamps mandatory
- No PII columns in telemetry-related tables

### 13.5 Foundation Feature Registered: Base UI Components
- Feature Code: FOUNDATION-003
- Source: `docs/features/foundation-base-ui-components/prd-addendum.md`
- Status: In Progress
- Story Set:
  - KP-019 UI Primitive Component Library Bootstrap
  - KP-020 Token-Compliant Button/Input/Select/Badge Set
  - KP-021 Surface Components (Card, Modal, Toast)
  - KP-022 Shell Layout Components (AppShell, Sidebar, TopBar, Panel)
  - KP-023 Accessibility Baseline for Interactive Primitives
  - KP-024 Primary Toggle and Connection Status Badge
  - KP-025 Ping Metric Card and Relay Health List Item
  - KP-026 Game Detection Row and Onboarding Stepper
  - KP-027 UI Composition Showcase Page (foundation states)
  - KP-028 UI Test Baseline (render + keyboard + visual guardrails)

### 13.6 UI Guardrails Reinforced
- Source of truth: `docs/design-system.md` and `docs/design-system.yaml`
- Signal/probe colors remain semantic-only
- Data-heavy components stay sharp and high-readability
- Accessibility baseline is mandatory for interactive primitives

### 13.7 Product Feature Registered: Smart Routing Engine
- Feature Code: FEATURE-001
- Source: `docs/features/smart-routing-engine/prd-addendum.md`
- Status: In Progress
- Story Set:
  - KP-029 Route Policy Config and Protocol Priority Constants
  - KP-030 Manifest Verification Gate in Routing Activation
  - KP-031 Relay Candidate Scoring Engine
  - KP-032 Route Attempt Orchestrator (WG -> TCP/TLS -> QUIC)
  - KP-033 Bounded Retry and Backoff Controller
  - KP-034 Routing State Machine and Lifecycle Events
  - KP-035 Rust Routing Engine Integration with Session Persistence Hooks
  - KP-036 End-to-End Routing Simulation Harness (protocol fallback)
  - KP-037 Contract Conformance Checks for Relay Input and Telemetry Output

### 13.8 Routing Guardrails Reinforced
- Protocol sequence is fixed: WireGuard -> TCP/TLS -> QUIC
- Manifest signature verification is mandatory pre-route
- Failover must be bounded by retry budget/backoff
- Routing lifecycle events must conform to telemetry schema

### 13.9 Product Feature Registered: Supported Game Detection
- Feature Code: FEATURE-002
- Source: `docs/features/supported-game-detection/prd-addendum.md`
- Status: In Progress
- Story Set:
  - KP-038 Supported Games Allowlist Schema Refinement
  - KP-039 Seed and Versioning for Supported Game Catalog
  - KP-040 Windows Process Scanner Core (allowlist match)
  - KP-041 Detection State Resolver and Freshness Window
  - KP-042 Manual Rescan Command Handler
  - KP-043 Detection Event Emission (game_detected) with Schema Guard
  - KP-044 Detection-Routing Gate Integration Contract
  - KP-045 Detection Status Panel and Rescan UX Wiring
  - KP-046 Detection Simulation and Anti-False-Positive Test Harness
  - KP-047 Funnel Telemetry Validation for Detection-to-Activation Path

### 13.10 Detection Guardrails Reinforced
- Detection must remain allowlist-only and process-external
- Routing activation requires valid detected state
- Detection telemetry must remain schema-compliant and privacy-safe
- False-positive control is a release-blocking quality gate

### 13.11 Product Feature Registered: Relay Health & Failover Handling
- Feature Code: FEATURE-003
- Source: `docs/features/relay-health-failover/prd-addendum.md`
- Status: In Progress
- Story Set:
  - KP-048 Relay Health Polling Scheduler
  - KP-049 Health Classification Engine (ok/warn/dead)
  - KP-050 Failover Trigger Evaluator and Hysteresis Window
  - KP-051 Active Relay Switch Executor
  - KP-052 Failover State/Reason Model for UI Bridge
  - KP-053 Relay Failure and Recovery Telemetry Emission
  - KP-054 Relay Health Surface Wiring (list + badge sync)
  - KP-055 Failover Progress Feedback UI States
  - KP-056 Failover Chaos Scenario Harness
  - KP-057 Reliability KPI Aggregation Test for Relay Events

### 13.12 Relay Guardrails Reinforced
- Health classification must be deterministic (`ok/warn/dead`)
- Failover switching must be anti-flapping and retry-bounded
- Failover UI messaging must map to normalized reason codes
- Reliability KPIs must be derivable from telemetry stream

### 13.13 Product Feature Registered: One-Click Routing Control
- Feature Code: FEATURE-004
- Source: `docs/features/one-click-routing-control/prd-addendum.md`
- Status: In Progress
- Story Set:
  - KP-058 Toggle Command Lifecycle Orchestrator
  - KP-059 ON Pipeline Gate Chain (Detection + Manifest + Route Precheck)
  - KP-060 OFF Pipeline Safe Teardown and Session Finalization
  - KP-061 Toggle Idempotency and Concurrency Guard
  - KP-062 Lifecycle Timeout and Error Transition Policy
  - KP-063 ON/OFF Telemetry Event Mapping with Reason Codes
  - KP-064 Primary Toggle Wiring to Lifecycle Commands
  - KP-065 Lifecycle Status Presenter (idle/arming/active/disarming/error)
  - KP-066 Rapid Toggle Stress Harness
  - KP-067 Activation Funnel Event Sequence Validator

### 13.14 Toggle Guardrails Reinforced
- ON flow must satisfy all safety gates before activation
- OFF flow must remain safe and idempotent
- Concurrency guards must prevent lifecycle corruption
- Lifecycle and telemetry state sequence must remain deterministic

### 13.15 Product Feature Registered: Real-Time Ping Metrics Dashboard
- Feature Code: FEATURE-005
- Source: `docs/features/realtime-ping-metrics-dashboard/prd-addendum.md`
- Status: In Progress
- Story Set:
  - KP-068 Probe Loop Scheduler and Lifecycle Binding
  - KP-069 Ping/Jitter/Packet-Loss Computation Module
  - KP-070 Metric Freshness and Degraded-State Evaluator
  - KP-071 Metrics Ring-Buffer Persistence Adapter
  - KP-072 ping_measured Telemetry Emitter with Schema Guard
  - KP-073 Real-Time Metric Card Data Binding
  - KP-074 Dashboard State Presenter (idle/measuring/live/degraded/error)
  - KP-075 Metric Trend Mini-View (optional lightweight sparkline-ready container)
  - KP-076 Probe Failure Recovery Integration Harness
  - KP-077 Performance KPI Validation Suite from Metrics + Events

### 13.16 Metrics Guardrails Reinforced
- Metric loop must bind strictly to routing lifecycle
- Degraded state must preserve last-known values with explicit status
- Metrics telemetry must remain schema-compliant and privacy-safe
- KPI derivation from samples/events must be reproducible

### 13.17 Product Feature Registered: Privacy-Safe Telemetry Pipeline
- Feature Code: FEATURE-006
- Source: `docs/features/privacy-safe-telemetry-pipeline/prd-addendum.md`
- Status: In Progress
- Story Set:
  - KP-078 Telemetry Queue Metadata Migration (retry/expiry diagnostics)
  - KP-079 Telemetry Retention and Purge Policy SQL Jobs
  - KP-080 Event Schema Allowlist Validator Core
  - KP-081 Sensitive Field Scrubber and Policy Engine
  - KP-082 Batch Builder and Queue Backpressure Controller
  - KP-083 Telemetry Delivery Client with Idempotency Key
  - KP-084 Retry/Expiry State Machine for Telemetry Batches
  - KP-085 Pipeline Health Snapshot Aggregator
  - KP-086 Telemetry Contract Conformance Integration Suite
  - KP-087 Privacy Violation Injection Harness (red-team style payload tests)
  - KP-088 Delivery Reliability KPI Validation Suite

### 13.18 Telemetry Guardrails Reinforced
- All events must pass shared schema allowlist validation
- Sensitive fields must be scrubbed/rejected before queueing
- Queue/retry/expiry policies must remain bounded and deterministic
- Telemetry failures must not block core routing/gameplay flow

### 13.19 Product Feature Registered: First-Run Onboarding Flow
- Feature Code: FEATURE-007
- Source: `docs/features/first-run-onboarding-flow/prd-addendum.md`
- Status: In Progress
- Story Set:
  - KP-089 Onboarding State Machine Core
  - KP-090 Permission & Environment Check Handlers
  - KP-091 Resume Checkpoint Persistence (onboarding_state)
  - KP-092 Completion Gate on First Successful Connect
  - KP-093 onboarding_completed Telemetry Emission
  - KP-094 Onboarding Stepper Flow Screens
  - KP-095 Recovery Branch UX (retry, troubleshoot, continue-safe)
  - KP-096 Onboarding Progress Resume UX
  - KP-097 End-to-End Onboarding Journey Harness
  - KP-098 Onboarding Funnel KPI Validation Suite

### 13.20 Onboarding Guardrails Reinforced
- Completion must require first successful connect
- Resume checkpoints must be deterministic and crash-safe
- Onboarding copy must stay concise and low-friction
- Onboarding telemetry must remain schema-compliant and privacy-safe

### 13.21 Product Feature Registered: In-App Updater + Release Trust Gate
- Feature Code: FEATURE-008
- Source: `docs/features/in-app-updater-release-trust-gate/prd-addendum.md`
- Status: In Progress
- Story Set:
  - KP-099 Updater Channel Resolver and Check Scheduler
  - KP-100 Update Package Verification and Apply Orchestrator
  - KP-101 Updater Lifecycle State Model and Error Taxonomy
  - KP-102 Updater Telemetry Event Mapping
  - KP-103 Updater Status UI Panel and Action Controls
  - KP-104 Restart Prompt and Post-Update UX Flow
  - KP-105 CI Stable Signing Gate Enforcement
  - KP-106 Release Artifact Trust Validation Harness
  - KP-107 End-to-End Updater Journey Harness
  - KP-108 Update Adoption and Failure KPI Validation Suite

### 13.22 Update Trust Guardrails Reinforced
- Stable channel publish must fail when artifacts are unsigned
- Update apply path must verify trust before install
- Updater failures must remain recoverable and non-blocking for core app use
- Updater telemetry must stay schema-compliant and privacy-safe

### 13.23 Product Feature Registered: Tauri IPC Command Bridge
- Feature Code: FEATURE-009
- Source: `docs/features/tauri-command-bridge/prd-addendum.md`
- Status: Proposed
- Story Set:
  - KP-109 Tauri IPC Contract Types and Command/Event Registry
  - KP-110 Routing Toggle Command Handlers in Tauri Backend
  - KP-111 Detection Status Query Command and Result Mapping
  - KP-112 Rust-to-Frontend Event Emission Bridge
  - KP-113 IPC Error Taxonomy Normalization for UI-safe Reason Codes
  - KP-114 Frontend IPC Client Adapter (invoke + event subscribe lifecycle)
  - KP-115 Feature Wiring for ToggleController, DetectionPanel, and PingMetricsPanel via IPC
  - KP-116 End-to-End IPC Journey Harness (toggle, detection, live metrics)
  - KP-117 IPC Contract Conformance and Determinism Test Suite

### 13.24 IPC Bridge Guardrails Reinforced
- Frontend must consume backend state via typed IPC adapter only (no raw hardcoded invoke/listen strings)
- Rust state machines remain source-of-truth for routing/detection/metrics transitions
- IPC error payloads must map to normalized non-sensitive reason codes
- IPC event ordering and payload schema must remain deterministic and test-gated


### 13.25 Product Feature Registered: App Shell IPC Wiring
- Feature Code: FEATURE-010
- Source: `docs/features/app-shell-ipc-wiring/prd-addendum.md`
- Status: Proposed
- Story Set:
  - KP-118 App Shell IPC State Orchestrator and ViewModel Contract
  - KP-119 Routing Toggle Invoke Wiring in App Shell
  - KP-120 Routing State Event Subscription and Badge Sync
  - KP-121 Startup Detection Query Wiring on App Shell Mount
  - KP-122 Detection Status Event Bridge for Live Panel Refresh
  - KP-123 Ping Metrics Event Stream Binding in App Shell
  - KP-124 IPC Listener Lifecycle Cleanup and Duplicate-Handler Guard
  - KP-125 App Shell IPC Error Mapping and User-Safe Feedback
  - KP-126 End-to-End App Shell IPC Wiring Harness

### 13.26 App Shell IPC Guardrails Reinforced
- App shell wajib menggunakan typed IPC adapter; tidak boleh hardcode `invoke/listen` string di composition layer.
- Routing status badge harus mengikuti event `routing_state_changed` sebagai source-of-truth.
- Detection status wajib di-query saat startup dan dapat disinkronkan ulang via event stream.
- Listener lifecycle (attach/cleanup) harus deterministic untuk mencegah duplicate handler dan memory leak.

---

## 12) Appendix

### References

- Product Brief: `docs/product-brief.md`

### Glossary

- Relay Node: Intermediate network endpoint used to optimize path selection.
- Baseline Ping: Measured latency without routing optimization.
- Optimized Ping: Measured latency after routing is enabled.
- D7 Retention: Percentage of users returning on day 7 after first use.





