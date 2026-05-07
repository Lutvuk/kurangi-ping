import { useEffect, useMemo, useRef, useState } from "react";
import { ConnectionStatusBadge, PrimaryToggle } from "./components/modules";
import { DetectionPanel } from "./features/detection";
import { PingMetricsPanel } from "./features/metrics";
import { useAppShellIpcState, type AppShellIpcActions } from "./features/shell";
import { presentIpcFailureState } from "./lib/errors/ipcErrorMapper";
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
        const rejected = presentIpcFailureState({
          context: command === "on" ? "routing_toggle_on" : "routing_toggle_off",
          reasonCode: response.reasonCode
        });
        setCommandFeedback(rejected.banner?.message ?? rejected.uiError.userMessage);
      }
    } catch {
      const failure = presentIpcFailureState({
        context: command === "on" ? "routing_toggle_on" : "routing_toggle_off"
      });
      actions.applyRoutingInvokeResponse(
        failure.routingResponse ?? {
          state: "error",
          reasonCode: "ipc_unknown_failure",
          message: failure.uiError.userMessage
        }
      );
      setCommandFeedback(failure.banner?.message ?? failure.uiError.userMessage);
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
        const failure = presentIpcFailureState({
          context: "detection_startup_query"
        });
        actions.applyDetectionQueryResponse(
          failure.detectionResponse ?? {
            state: "not_detected",
            reasonCode: failure.uiError.reasonCode,
            message: failure.uiError.userMessage
          }
        );
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
    const failure = presentIpcFailureState({
      context: "listener_attach_routing"
    });
    setCommandFeedback(failure.banner?.message ?? failure.uiError.userMessage);
    return;
  }

  if (key === "detection_status_updated") {
    const failure = presentIpcFailureState({
      context: "listener_attach_detection"
    });
    actions.applyDetectionQueryResponse(
      failure.detectionResponse ?? {
        state: "not_detected",
        reasonCode: failure.uiError.reasonCode,
        message: failure.uiError.userMessage
      }
    );
    return;
  }

  const failure = presentIpcFailureState({
    context: "listener_attach_metrics",
    atUnixMs: Date.now()
  });
  actions.applyMetricsSampleEvent(
    failure.metricsPayload ?? {
      sampledAtUnixMs: Date.now(),
      state: "degraded",
      baselinePingMs: null,
      routedPingMs: null,
      jitterMs: null,
      packetLossPct: null,
      reasonCode: failure.uiError.reasonCode
    }
  );
}
