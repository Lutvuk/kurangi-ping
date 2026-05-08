import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import App from "./App";

const mocks = vi.hoisted(() => ({
  invokeRoutingToggleOn: vi.fn(),
  invokeRoutingToggleOff: vi.fn(),
  invokeDetectionGetStatus: vi.fn(),
  subscribeRoutingState: vi.fn(),
  subscribeDetectionStatus: vi.fn(),
  subscribePingMetrics: vi.fn()
}));

vi.mock("./lib/ipc", () => ({
  ipcClient: {
    invokeRoutingToggleOn: mocks.invokeRoutingToggleOn,
    invokeRoutingToggleOff: mocks.invokeRoutingToggleOff,
    invokeDetectionGetStatus: mocks.invokeDetectionGetStatus,
    subscribeRoutingState: mocks.subscribeRoutingState,
    subscribeDetectionStatus: mocks.subscribeDetectionStatus,
    subscribePingMetrics: mocks.subscribePingMetrics
  },
  createListenerRegistryMetadata: () => ({
    attachedKeys: new Set()
  }),
  attachShellListeners: async ({
    client,
    handlers,
    onAttachError
  }: {
    client: {
      subscribeRoutingState: (handler: (payload: unknown) => void) => Promise<() => Promise<void>>;
      subscribeDetectionStatus: (handler: (payload: unknown) => void) => Promise<() => Promise<void>>;
      subscribePingMetrics: (handler: (payload: unknown) => void) => Promise<() => Promise<void>>;
    };
    handlers: {
      onRoutingStateChanged: (payload: unknown) => void;
      onDetectionStatusUpdated: (payload: unknown) => void;
      onMetricsPingSampled: (payload: unknown) => void;
    };
    onAttachError?: (context: { key: string; error: unknown }) => void;
  }) => {
    const detach: Array<() => Promise<void>> = [];
    try {
      detach.push(await client.subscribeRoutingState(handlers.onRoutingStateChanged));
    } catch (error) {
      onAttachError?.({ key: "routing_state_changed", error });
    }
    try {
      detach.push(await client.subscribeDetectionStatus(handlers.onDetectionStatusUpdated));
    } catch (error) {
      onAttachError?.({ key: "detection_status_updated", error });
    }
    try {
      detach.push(await client.subscribePingMetrics(handlers.onMetricsPingSampled));
    } catch (error) {
      onAttachError?.({ key: "metrics_ping_sampled", error });
    }
    return async () => {
      for (const unsubscribe of detach) {
        await unsubscribe();
      }
    };
  }
}));

type RoutingStatePayload = {
  previousState: "idle" | "connecting" | "active" | "degraded" | "error";
  state: "idle" | "connecting" | "active" | "degraded" | "error";
  reasonCode?: string;
  message?: string;
};

type DetectionStatusPayload = {
  state: "detected" | "not_detected";
  gameId?: string;
  processName?: string;
  detectionTimeMs?: number;
  reasonCode?: string;
  message?: string;
};

type MetricsPayload = {
  sampledAtUnixMs: number;
  state: "live" | "measuring" | "degraded";
  baselinePingMs: number | null;
  routedPingMs: number | null;
  jitterMs: number | null;
  packetLossPct: number | null;
  reasonCode?: string;
};

function buildJourneyHarness(trace: string[]) {
  let routingListener: ((payload: RoutingStatePayload) => void) | undefined;
  let detectionListener: ((payload: DetectionStatusPayload) => void) | undefined;
  let metricsListener: ((payload: MetricsPayload) => void) | undefined;

  const routingUnsubscribe = vi.fn(async () => {
    trace.push("cleanup:routing");
  });
  const detectionUnsubscribe = vi.fn(async () => {
    trace.push("cleanup:detection");
  });
  const metricsUnsubscribe = vi.fn(async () => {
    trace.push("cleanup:metrics");
  });

  mocks.invokeRoutingToggleOn.mockImplementation(async () => {
    trace.push("invoke:routing_toggle_on");
    return { state: "connecting" as const };
  });
  mocks.invokeRoutingToggleOff.mockImplementation(async () => ({ state: "idle" as const }));
  mocks.invokeDetectionGetStatus.mockImplementation(async () => {
    trace.push("invoke:detection_get_status");
    return {
      state: "detected" as const,
      gameId: "ffxiv",
      processName: "ffxiv_dx11.exe",
      detectionTimeMs: 1_700_001_100_000
    };
  });
  mocks.subscribeRoutingState.mockImplementation(async (handler) => {
    trace.push("subscribe:routing_state_changed");
    routingListener = handler;
    return routingUnsubscribe;
  });
  mocks.subscribeDetectionStatus.mockImplementation(async (handler) => {
    trace.push("subscribe:detection_status_updated");
    detectionListener = handler;
    return detectionUnsubscribe;
  });
  mocks.subscribePingMetrics.mockImplementation(async (handler) => {
    trace.push("subscribe:metrics_ping_sampled");
    metricsListener = handler;
    return metricsUnsubscribe;
  });

  return {
    emitRouting(payload: RoutingStatePayload) {
      trace.push(`event:routing_state_changed:${payload.previousState}->${payload.state}`);
      routingListener?.(payload);
    },
    emitDetection(payload: DetectionStatusPayload) {
      trace.push(`event:detection_status_updated:${payload.state}`);
      detectionListener?.(payload);
    },
    emitMetrics(payload: MetricsPayload) {
      trace.push(`event:metrics_ping_sampled:${payload.state}`);
      metricsListener?.(payload);
    },
    counts() {
      return {
        routing: routingUnsubscribe.mock.calls.length,
        detection: detectionUnsubscribe.mock.calls.length,
        metrics: metricsUnsubscribe.mock.calls.length
      };
    }
  };
}

type JourneySuiteResult = {
  trace: string[];
  cleanup: {
    routing: number;
    detection: number;
    metrics: number;
  };
};

async function runAppShellIpcJourneySuite(): Promise<JourneySuiteResult> {
  const trace: string[] = [];
  const harness = buildJourneyHarness(trace);

  const { unmount } = render(<App />);
  const detectionPanel = screen.getByLabelText("Detection status panel");
  const metricsPanel = screen.getByLabelText("Ping Metrics");

  await waitFor(() => expect(mocks.invokeDetectionGetStatus).toHaveBeenCalledTimes(1));
  await waitFor(() =>
    expect(within(detectionPanel).getByRole("status")).toHaveTextContent("Detected")
  );

  fireEvent.click(screen.getByRole("button", { name: "Routing toggle off" }));
  await waitFor(() => expect(mocks.invokeRoutingToggleOn).toHaveBeenCalledTimes(1));
  await waitFor(() =>
    expect(screen.getByTestId("routing-connection-status")).toHaveTextContent("Connecting...")
  );

  harness.emitRouting({
    previousState: "connecting",
    state: "active"
  });
  await waitFor(() =>
    expect(screen.getByTestId("routing-connection-status")).toHaveTextContent("Routing Active")
  );

  harness.emitDetection({
    state: "not_detected"
  });
  await waitFor(() =>
    expect(within(detectionPanel).getByRole("status")).toHaveTextContent("Not Found")
  );

  harness.emitMetrics({
    sampledAtUnixMs: 1_700_001_100_900,
    state: "live",
    baselinePingMs: 209.4,
    routedPingMs: 150.2,
    jitterMs: 3.4,
    packetLossPct: 0.1
  });
  await waitFor(() =>
    expect(within(metricsPanel).getByTestId("metrics-panel-status")).toHaveTextContent("Live")
  );

  unmount();
  await waitFor(() => expect(harness.counts()).toEqual({ routing: 1, detection: 1, metrics: 1 }));

  return {
    trace,
    cleanup: harness.counts()
  };
}

describe("App IPC wiring journey harness", () => {
  it("runAppShellIpcJourneySuite validates integrated invoke/event path", async () => {
    const result = await runAppShellIpcJourneySuite();
    expect(result.cleanup).toEqual({ routing: 1, detection: 1, metrics: 1 });
    expect(result.trace).toEqual([
      "invoke:detection_get_status",
      "subscribe:routing_state_changed",
      "subscribe:detection_status_updated",
      "subscribe:metrics_ping_sampled",
      "invoke:routing_toggle_on",
      "event:routing_state_changed:connecting->active",
      "event:detection_status_updated:not_detected",
      "event:metrics_ping_sampled:live",
      "cleanup:routing",
      "cleanup:detection",
      "cleanup:metrics"
    ]);
  });

  it("repeated mount/unmount stays deterministic without duplicate listener side-effects", async () => {
    async function runCycle(): Promise<string[]> {
      const trace: string[] = [];
      const harness = buildJourneyHarness(trace);

      const { unmount } = render(<App />);
      await waitFor(() => expect(mocks.invokeDetectionGetStatus).toHaveBeenCalledTimes(1));

      harness.emitRouting({
        previousState: "idle",
        state: "active"
      });
      await waitFor(() =>
        expect(screen.getByTestId("routing-connection-status")).toHaveTextContent("Routing Active")
      );

      unmount();
      await waitFor(() =>
        expect(harness.counts()).toEqual({ routing: 1, detection: 1, metrics: 1 })
      );
      return trace;
    }

    mocks.invokeRoutingToggleOn.mockReset();
    mocks.invokeRoutingToggleOff.mockReset();
    mocks.invokeDetectionGetStatus.mockReset();
    mocks.subscribeRoutingState.mockReset();
    mocks.subscribeDetectionStatus.mockReset();
    mocks.subscribePingMetrics.mockReset();
    const first = await runCycle();

    mocks.invokeRoutingToggleOn.mockReset();
    mocks.invokeRoutingToggleOff.mockReset();
    mocks.invokeDetectionGetStatus.mockReset();
    mocks.subscribeRoutingState.mockReset();
    mocks.subscribeDetectionStatus.mockReset();
    mocks.subscribePingMetrics.mockReset();
    const second = await runCycle();

    expect(first).toEqual(second);
  });
});
