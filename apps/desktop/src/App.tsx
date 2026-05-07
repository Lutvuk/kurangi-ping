import { useEffect, useMemo, useRef, useState } from "react";
import { ConnectionStatusBadge, PrimaryToggle } from "./components/modules";
import { DetectionPanel } from "./features/detection";
import { PingMetricsPanel } from "./features/metrics";
import { useAppShellIpcState, type AppShellIpcActions } from "./features/shell";
import {
  attachShellListeners,
  createListenerRegistryMetadata,
  ipcClient,
  type ShellListenerKey
} from "./lib/ipc";
import { AppShell } from "./layout/AppShell";
import { Panel } from "./layout/Panel";
import { FoundationShowcasePage } from "./pages/foundation-showcase";

export default function App() {
  const { viewModel, actions } = useAppShellIpcState();
  const [inFlightCommand, setInFlightCommand] = useState<"on" | "off" | null>(null);
  const [commandFeedback, setCommandFeedback] = useState<string | null>(null);
  const startupDetectionRequestedRef = useRef(false);
  const listenerRegistryRef = useRef(createListenerRegistryMetadata());
  const isFoundationShowcaseEnabled =
    (import.meta.env.DEV && import.meta.env.MODE !== "test") ||
    import.meta.env.VITE_ENABLE_FOUNDATION_SHOWCASE === "1";

  if (isFoundationShowcaseEnabled) {
    return <FoundationShowcasePage />;
  }

  async function invokeRoutingToggle(command: "on" | "off") {
    if (inFlightCommand !== null) {
      return;
    }

    setCommandFeedback(null);
    setInFlightCommand(command);
    actions.markRoutingCommandStarted(command);

    try {
      const response =
        command === "on"
          ? await ipcClient.invokeRoutingToggleOn({
              trigger: "user_toggle",
              requestedAtUnixMs: Date.now()
            })
          : await ipcClient.invokeRoutingToggleOff({
              trigger: "user_toggle",
              requestedAtUnixMs: Date.now()
            });
      actions.applyRoutingInvokeResponse(response);
      if (response.reasonCode) {
        setCommandFeedback("Perintah routing ditolak sistem. Coba lagi.");
      }
    } catch {
      actions.applyRoutingInvokeResponse({
        state: "error",
        reasonCode: "ipc_unknown_failure",
        message: "Toggle command failed."
      });
      setCommandFeedback("Tidak bisa memproses perintah routing sekarang. Coba lagi.");
    } finally {
      setInFlightCommand(null);
    }
  }

  const statusLabel = useMemo(() => {
    if (inFlightCommand === "on") {
      return "Connecting...";
    }
    if (inFlightCommand === "off") {
      return "Disconnecting...";
    }
    return undefined;
  }, [inFlightCommand]);

  useEffect(() => {
    if (startupDetectionRequestedRef.current) {
      return;
    }
    startupDetectionRequestedRef.current = true;

    let active = true;
    void (async () => {
      try {
        const response = await ipcClient.invokeDetectionGetStatus();
        if (!active) {
          return;
        }
        actions.applyDetectionQueryResponse(response);
      } catch {
        if (!active) {
          return;
        }
        actions.applyDetectionQueryResponse({
          state: "not_detected",
          reasonCode: "ipc_unknown_failure",
          message: "Detection status tidak tersedia."
        });
      }
    })();

    return () => {
      active = false;
    };
  }, [actions]);

  useEffect(() => {
    let active = true;
    let detachListeners: (() => Promise<void>) | undefined;

    void (async () => {
      detachListeners = await attachShellListeners({
        client: ipcClient,
        metadata: listenerRegistryRef.current,
        handlers: {
          onRoutingStateChanged(payload) {
            if (!active) {
              return;
            }
            actions.applyRoutingStateEvent(payload);
          },
          onDetectionStatusUpdated(payload) {
            if (!active) {
              return;
            }
            actions.applyDetectionStatusEvent(payload);
          },
          onMetricsPingSampled(payload) {
            if (!active) {
              return;
            }
            actions.applyMetricsSampleEvent(payload);
          }
        },
        onAttachError(context) {
          if (!active) {
            return;
          }
          handleListenerAttachError(context.key, actions, setCommandFeedback);
        }
      });
    })();

    return () => {
      active = false;
      if (detachListeners) {
        void detachListeners();
      }
    };
  }, [actions]);

  return (
    <AppShell>
      <Panel eyebrow="Foundation" title="Kurangi Ping 2">
        <p className="kp-copy">Desktop shell regions are ready for feature injection.</p>
        <div
          style={{
            marginTop: "var(--space-4)",
            display: "flex",
            alignItems: "center",
            gap: "var(--space-4)"
          }}
        >
          <PrimaryToggle
            state={viewModel.routing.toggleState}
            disabled={inFlightCommand !== null}
            aria-busy={inFlightCommand !== null}
            onToggle={(nextEnabled) => {
              void invokeRoutingToggle(nextEnabled ? "on" : "off");
            }}
          />
          <ConnectionStatusBadge
            state={viewModel.routing.badgeState}
            label={statusLabel}
            data-testid="routing-connection-status"
          />
        </div>
        {commandFeedback ? (
          <p role="alert" className="kp-copy" style={{ marginTop: "var(--space-3)" }}>
            {commandFeedback}
          </p>
        ) : null}
      </Panel>
      <DetectionPanel model={viewModel.detection} title="Detection Status" />
      <PingMetricsPanel model={viewModel.metrics} title="Ping Metrics" />
    </AppShell>
  );
}

function handleListenerAttachError(
  key: ShellListenerKey,
  actions: AppShellIpcActions,
  setCommandFeedback: (value: string | null) => void
) {
  if (key === "routing_state_changed") {
    setCommandFeedback("Sinkronisasi status routing sedang tidak tersedia.");
    return;
  }

  if (key === "detection_status_updated") {
    actions.applyDetectionQueryResponse({
      state: "not_detected",
      reasonCode: "ipc_unknown_failure",
      message: "Sinkronisasi detection status tidak tersedia."
    });
    return;
  }

  actions.applyMetricsSampleEvent({
    sampledAtUnixMs: Date.now(),
    state: "degraded",
    baselinePingMs: null,
    routedPingMs: null,
    jitterMs: null,
    packetLossPct: null,
    reasonCode: "ipc_metrics_stream_unavailable"
  });
}
