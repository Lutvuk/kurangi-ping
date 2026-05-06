import { useEffect, useMemo, useState } from "react";
import {
  ConnectionStatusBadge,
  GameDetectionRow,
  PingMetricCard,
  PrimaryToggle,
  RelayHealthListItem
} from "../components/modules";
import { DetectionPanel, type DetectionViewModel } from "../features/detection";
import { PingMetricsPanel, type MetricsTrendSample, type MetricsViewModel } from "../features/metrics";
import { OnboardingFlow, type OnboardingStateMachineView } from "../features/onboarding";
import { RelayHealthPanel } from "../features/relay";
import { LifecycleStatusPresenter, ToggleController } from "../features/routing";
import { AppShell } from "../layout/AppShell";
import { Panel } from "../layout/Panel";
import "./foundation-showcase.css";

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
  const metricsCycle: MetricsViewModel[] = useMemo(
    () => [
      {
        state: "live",
        baselinePingMs: 214.8,
        routedPingMs: 156.2,
        jitterMs: 4.1,
        packetLossPct: 0.0,
        sampledAtUnixMs: 1_700_000_000_000
      },
      {
        state: "measuring",
        baselinePingMs: 214.8,
        routedPingMs: null,
        jitterMs: null,
        packetLossPct: null,
        sampledAtUnixMs: 1_700_000_001_000
      },
      {
        state: "degraded",
        baselinePingMs: 212.3,
        routedPingMs: null,
        jitterMs: null,
        packetLossPct: null,
        sampledAtUnixMs: 1_700_000_002_000,
        reasonCode: "freshness_timeout"
      },
      {
        state: "live",
        baselinePingMs: 210.5,
        routedPingMs: 149.8,
        jitterMs: 2.9,
        packetLossPct: 0.0,
        sampledAtUnixMs: 1_700_000_003_000
      }
    ],
    []
  );
  const [metricsIndex, setMetricsIndex] = useState(0);
  const [trendSamples, setTrendSamples] = useState<MetricsTrendSample[]>([]);
  const [onboardingMachine, setOnboardingMachine] = useState<OnboardingStateMachineView>({
    state: {
      state: "not_started"
    }
  });
  const [onboardingSignals, setOnboardingSignals] = useState({
    permissionGranted: false,
    relayReady: false,
    gameDetected: false,
    connected: false
  });

  async function handleRescanMock() {
    const nextIndex = (detectionStateIndex + 1) % detectionCycle.length;
    setDetectionStateIndex(nextIndex);
    return detectionCycle[nextIndex];
  }

  useEffect(() => {
    setTrendSamples((previous) => {
      const nextSample = {
        sampledAtUnixMs: metricsCycle[metricsIndex].sampledAtUnixMs ?? Date.now(),
        routedPingMs: metricsCycle[metricsIndex].routedPingMs
      };
      const merged = [...previous, nextSample];
      return merged.slice(-24);
    });
  }, [metricsCycle, metricsIndex]);

  useEffect(() => {
    const timer = window.setInterval(() => {
      setMetricsIndex((previous) => (previous + 1) % metricsCycle.length);
    }, 900);

    return () => {
      window.clearInterval(timer);
    };
  }, [metricsCycle.length]);

  function startOnboarding() {
    setOnboardingMachine({
      state: {
        state: "in_progress",
        current_step: "permission_check",
        completed_steps: ["welcome"]
      }
    });
  }

  function runPermissionCheck() {
    setOnboardingSignals((previous) => ({
      ...previous,
      permissionGranted: true
    }));
    setOnboardingMachine({
      state: {
        state: "in_progress",
        current_step: "relay_test",
        completed_steps: ["welcome", "permission_check"]
      }
    });
  }

  function runRelayTest() {
    setOnboardingSignals((previous) => ({
      ...previous,
      relayReady: true
    }));
    setOnboardingMachine({
      state: {
        state: "in_progress",
        current_step: "game_detection_test",
        completed_steps: ["welcome", "permission_check", "relay_test"]
      }
    });
  }

  function runDetectionTest() {
    setOnboardingSignals((previous) => ({
      ...previous,
      gameDetected: true
    }));
    setOnboardingMachine({
      state: {
        state: "in_progress",
        current_step: "first_connect",
        completed_steps: ["welcome", "permission_check", "relay_test", "game_detection_test"]
      }
    });
  }

  function runFirstConnect() {
    setOnboardingSignals((previous) => ({
      ...previous,
      connected: true
    }));
    setOnboardingMachine({
      state: {
        state: "completed",
        completed_steps: [
          "welcome",
          "permission_check",
          "relay_test",
          "game_detection_test",
          "first_connect"
        ]
      }
    });
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
          <ToggleController
            initialState="off"
            onEnableRouting={async () => ({ ok: true, nextState: "on" })}
            onDisableRouting={async () => ({ ok: true, nextState: "off" })}
          />
        </div>
        <div className="kp-showcase-stack">
          <LifecycleStatusPresenter model={{ state: "idle" }} />
          <LifecycleStatusPresenter model={{ state: "arming" }} />
          <LifecycleStatusPresenter model={{ state: "active" }} />
          <LifecycleStatusPresenter model={{ state: "disarming" }} />
          <LifecycleStatusPresenter model={{ state: "error", reasonCode: "arming_timeout" }} />
        </div>
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
          <PingMetricsPanel model={metricsCycle[metricsIndex]} trendSamples={trendSamples} />
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
        <RelayHealthPanel
          failover={{
            currentState: "switching",
            previousRelayId: "sin-01",
            nextRelayId: "nrt-01",
            reasonCode: "dead_relay_detected"
          }}
          relays={[
            {
              relayId: "sin-01",
              hostname: "sin-01.relay.local",
              latencyMs: 35,
              region: "sin",
              health: "dead"
            },
            {
              relayId: "nrt-01",
              hostname: "nrt-01.relay.local",
              latencyMs: 74,
              region: "nrt",
              health: "ok"
            },
            {
              relayId: "lax-01",
              hostname: "lax-01.relay.local",
              latencyMs: 121,
              region: "lax",
              health: "warn"
            }
          ]}
        />
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
        <OnboardingFlow
          machine={onboardingMachine}
          signals={onboardingSignals}
          actions={{
            onStart: startOnboarding,
            onRunPermissionCheck: runPermissionCheck,
            onRunRelayTest: runRelayTest,
            onRunDetectionTest: runDetectionTest,
            onRunFirstConnect: runFirstConnect
          }}
        />
      </div>
    </section>
  );
}

export function FoundationShowcasePage() {
  return (
    <AppShell
      sidebar={{ activeId: "routing", title: "Kurangi Ping 2" }}
      topBar={{ statusLabel: "Build", statusValue: "Foundation Showcase", meta: "KP-024 to KP-094" }}
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
