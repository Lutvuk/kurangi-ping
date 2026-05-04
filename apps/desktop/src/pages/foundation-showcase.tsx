import { useState } from "react";
import {
  ConnectionStatusBadge,
  GameDetectionRow,
  OnboardingStepper,
  PingMetricCard,
  PrimaryToggle,
  RelayHealthListItem
} from "../components/modules";
import { DetectionPanel, type DetectionViewModel } from "../features/detection";
import { AppShell } from "../layout/AppShell";
import { Panel } from "../layout/Panel";
import "./foundation-showcase.css";

const onboardingSteps = [
  { id: "welcome", label: "Welcome", state: "completed" as const },
  { id: "permission", label: "Permission", state: "completed" as const },
  { id: "relay-test", label: "Relay Test", state: "active" as const },
  { id: "first-connect", label: "First Connect", state: "inactive" as const }
];

function ShowcaseViewport({ title, compact }: { title: string; compact?: boolean }) {
  const detectionCycle: DetectionViewModel[] = [
    {
      state: "not_found",
      gameId: "ffxiv",
      processName: "process unavailable",
      detectionTimeMs: 0,
      reasonCode: "not_found",
      message: "Game belum terdeteksi. Jalankan game lalu scan ulang."
    },
    {
      state: "detected",
      gameId: "ffxiv",
      processName: "ffxiv_dx11.exe",
      detectionTimeMs: 742
    },
    {
      state: "stale",
      gameId: "ffxiv",
      processName: "ffxiv_dx11.exe",
      detectionTimeMs: 21000,
      reasonCode: "stale_window_exceeded"
    },
    {
      state: "error",
      gameId: "ffxiv",
      processName: "ffxiv_dx11.exe",
      detectionTimeMs: 21000,
      reasonCode: "permission_denied"
    }
  ];
  const [detectionStateIndex, setDetectionStateIndex] = useState(0);

  async function handleRescanMock() {
    const nextIndex = (detectionStateIndex + 1) % detectionCycle.length;
    setDetectionStateIndex(nextIndex);
    return detectionCycle[nextIndex];
  }

  return (
    <section
      className={["kp-showcase-viewport", compact ? "kp-showcase-viewport--compact" : ""]
        .filter(Boolean)
        .join(" ")}
      aria-label={title}
    >
      <h2 className="kp-showcase-title">{title}</h2>

      <div className="kp-showcase-group">
        <h3 className="kp-showcase-subtitle">Connection Controls</h3>
        <div className="kp-showcase-inline">
          <PrimaryToggle state="off" />
          <PrimaryToggle state="connecting" />
          <PrimaryToggle state="on" />
          <PrimaryToggle state="degraded" />
        </div>
        <div className="kp-showcase-inline">
          <ConnectionStatusBadge state="off" />
          <ConnectionStatusBadge state="connecting" />
          <ConnectionStatusBadge state="on" />
          <ConnectionStatusBadge state="degraded" />
        </div>
      </div>

      <div className="kp-showcase-group">
        <h3 className="kp-showcase-subtitle">Ping and Relay Health</h3>
        <div className="kp-showcase-stack">
          <PingMetricCard state="on" currentPingMs={42} baselinePingMs={71} reductionPct={40.8} />
          <PingMetricCard
            state="degraded"
            currentPingMs={128}
            baselinePingMs={95}
            reductionPct={-34.7}
          />
        </div>
        <div role="list" className="kp-showcase-stack">
          <RelayHealthListItem hostname="sin-01.relay.local" latencyMs={35} region="sin" health="ok" />
          <RelayHealthListItem
            hostname="nrt-01.relay.local"
            latencyMs={102}
            region="nrt"
            health="warn"
            active
          />
          <RelayHealthListItem hostname="lax-01.relay.local" latencyMs={null} region="lax" health="dead" />
        </div>
      </div>

      <div className="kp-showcase-group">
        <h3 className="kp-showcase-subtitle">Detection and Onboarding</h3>
        <div className="kp-showcase-stack">
          <GameDetectionRow
            gameName="Final Fantasy XIV"
            serverInfo="Elemental - Tonberry"
            state="detected"
            icon="FF"
          />
          <GameDetectionRow
            gameName="Genshin Impact"
            serverInfo="Asia Cluster"
            state="not-detected"
            icon="GI"
          />
          <DetectionPanel
            title="Detection Control"
            model={detectionCycle[detectionStateIndex]}
            onTriggerRescan={handleRescanMock}
          />
        </div>
        <OnboardingStepper steps={onboardingSteps} />
      </div>
    </section>
  );
}

export function FoundationShowcasePage() {
  return (
    <AppShell
      sidebar={{ activeId: "routing", title: "Kurangi Ping 2" }}
      topBar={{ statusLabel: "Build", statusValue: "Foundation Showcase", meta: "KP-024 to KP-028" }}
      contentClassName="kp-showcase-content"
    >
      <Panel eyebrow="Foundation QA" title="UI Composition Showcase">
        <p className="kp-copy">
          Visual contract surface for fast manual checks across foundation component states.
        </p>
      </Panel>

      <div className="kp-showcase-grid">
        <ShowcaseViewport title="Compact Preview (<= 960px)" compact />
        <ShowcaseViewport title="Full Preview (>= 1280px)" />
      </div>
    </AppShell>
  );
}
