import { describe, expect, it, vi } from "vitest";
import {
  attachShellListeners,
  createIpcClient,
  createListenerRegistryMetadata,
  listenerRegistryGuard,
  type IpcClient,
  type IpcClientDeps
} from "./client";

function createStubDeps(): IpcClientDeps {
  return {
    invoke: vi.fn(),
    listen: vi.fn()
  };
}

function validateWirePayloadConformance(
  payload: unknown,
  requiredKeys: readonly string[]
): string[] {
  if (!payload || typeof payload !== "object") {
    return ["payload must be object"];
  }

  const record = payload as Record<string, unknown>;
  const missing = requiredKeys.filter((key) => !(key in record));
  if (missing.length > 0) {
    return missing.map((key) => `missing required key '${key}'`);
  }
  return [];
}

describe("ipc client adapter", () => {
  it("invokes routing toggle commands with typed payload mapping", async () => {
    const deps = createStubDeps();
    vi.mocked(deps.invoke)
      .mockResolvedValueOnce({
        state: "connecting",
        reason_code: undefined,
        message: undefined
      })
      .mockResolvedValueOnce({
        state: "idle",
        reason_code: "ipc_command_rejected",
        message: "toggle rejected"
      });

    const client = createIpcClient(deps);

    const onResponse = await client.invokeRoutingToggleOn({
      trigger: "user_toggle",
      requestedAtUnixMs: 1_700_000_000_000
    });
    const offResponse = await client.invokeRoutingToggleOff({
      trigger: "shutdown",
      requestedAtUnixMs: 1_700_000_001_000
    });

    expect(vi.mocked(deps.invoke).mock.calls[0]).toEqual([
      "routing_toggle_on",
      {
        request: {
          trigger: "user_toggle",
          requested_at_unix_ms: 1_700_000_000_000
        }
      }
    ]);
    expect(vi.mocked(deps.invoke).mock.calls[1]).toEqual([
      "routing_toggle_off",
      {
        request: {
          trigger: "shutdown",
          requested_at_unix_ms: 1_700_000_001_000
        }
      }
    ]);

    expect(onResponse).toEqual({
      state: "connecting",
      reasonCode: undefined,
      message: undefined
    });
    expect(offResponse).toEqual({
      state: "idle",
      reasonCode: "ipc_command_rejected",
      message: "toggle rejected"
    });
  });

  it("maps detection query response and normalizes unknown reason code fallback", async () => {
    const deps = createStubDeps();
    vi.mocked(deps.invoke).mockResolvedValueOnce({
      state: "not_detected",
      game_id: undefined,
      process_name: undefined,
      detection_time_ms: 1_700_000_002_000,
      reason_code: "some-new-code",
      message: "temporary failure"
    });

    const client = createIpcClient(deps);
    const response = await client.invokeDetectionGetStatus();

    expect(vi.mocked(deps.invoke).mock.calls[0]).toEqual(["detection_get_status"]);
    expect(response).toEqual({
      state: "not_detected",
      gameId: undefined,
      processName: undefined,
      detectionTimeMs: 1_700_000_002_000,
      reasonCode: "ipc_unknown_failure",
      message: "temporary failure"
    });
  });

  it("subscribes to bridge events with deterministic idempotent cleanup", async () => {
    const deps = createStubDeps();
    let routingHandler: ((event: { payload: unknown }) => void) | undefined;
    const rawUnsubscribe = vi.fn();

    vi.mocked(deps.listen).mockImplementationOnce(async (_eventName, handler) => {
      routingHandler = handler as (event: { payload: unknown }) => void;
      return rawUnsubscribe;
    });

    const client = createIpcClient(deps);
    const callback = vi.fn();
    const cleanup = await client.subscribeRoutingState(callback);

    expect(vi.mocked(deps.listen).mock.calls[0][0]).toBe("routing_state_changed");

    routingHandler?.({
      payload: {
        previous_state: "idle",
        state: "connecting",
        reason_code: "ipc_invalid_state",
        message: "transition applied"
      }
    });
    expect(callback).toHaveBeenCalledWith({
      previousState: "idle",
      state: "connecting",
      reasonCode: "ipc_invalid_state",
      message: "transition applied"
    });

    await cleanup();
    await cleanup();
    expect(rawUnsubscribe).toHaveBeenCalledTimes(1);
  });

  it("maps metrics and detection event payloads to frontend model shape", async () => {
    const deps = createStubDeps();
    const rawUnsubscribe = vi.fn();
    const listeners: Array<(event: { payload: unknown }) => void> = [];

    vi.mocked(deps.listen).mockImplementation(async (_eventName, handler) => {
      listeners.push(handler as (event: { payload: unknown }) => void);
      return rawUnsubscribe;
    });

    const client = createIpcClient(deps);
    const onMetrics = vi.fn();
    const onDetection = vi.fn();

    const cleanupMetrics = await client.subscribePingMetrics(onMetrics);
    const cleanupDetection = await client.subscribeDetectionStatus(onDetection);

    listeners[0]?.({
      payload: {
        sampled_at_unix_ms: 1_700_000_003_000,
        state: "degraded",
        baseline_ping_ms: 210,
        routed_ping_ms: 160,
        jitter_ms: 4,
        packet_loss_pct: 0.5,
        reason_code: "ipc_timeout"
      }
    });

    listeners[1]?.({
      payload: {
        state: "detected",
        game_id: "ffxiv",
        process_name: "ffxiv_dx11.exe",
        detection_time_ms: 1_700_000_004_000,
        reason_code: undefined,
        message: undefined
      }
    });

    expect(onMetrics).toHaveBeenCalledWith({
      sampledAtUnixMs: 1_700_000_003_000,
      state: "degraded",
      baselinePingMs: 210,
      routedPingMs: 160,
      jitterMs: 4,
      packetLossPct: 0.5,
      reasonCode: "ipc_timeout"
    });
    expect(onDetection).toHaveBeenCalledWith({
      state: "detected",
      gameId: "ffxiv",
      processName: "ffxiv_dx11.exe",
      detectionTimeMs: 1_700_000_004_000,
      reasonCode: undefined,
      message: undefined
    });

    await cleanupMetrics();
    await cleanupDetection();
    expect(rawUnsubscribe).toHaveBeenCalledTimes(2);
  });

  it("fails conformance assertions for missing and invalid payload fixtures", () => {
    const routingValid = {
      previous_state: "idle",
      state: "active"
    };
    const routingInvalid = {
      state: "active"
    };
    const metricsValid = {
      sampled_at_unix_ms: 1_700_000_010_000,
      state: "live",
      baseline_ping_ms: 210.5,
      routed_ping_ms: 152.2,
      jitter_ms: 3.1,
      packet_loss_pct: 0.0
    };
    const metricsInvalid = {
      sampled_at_unix_ms: 1_700_000_010_000,
      baseline_ping_ms: 210.5
    };
    const detectionValid = {
      state: "detected",
      game_id: "ffxiv"
    };
    const detectionInvalid = {
      reason_code: "ipc_detection_scan_failed"
    };

    expect(
      validateWirePayloadConformance(routingValid, ["previous_state", "state"])
    ).toEqual([]);
    expect(
      validateWirePayloadConformance(metricsValid, [
        "sampled_at_unix_ms",
        "state",
        "baseline_ping_ms",
        "routed_ping_ms",
        "jitter_ms",
        "packet_loss_pct"
      ])
    ).toEqual([]);
    expect(validateWirePayloadConformance(detectionValid, ["state"])).toEqual([]);

    expect(validateWirePayloadConformance(routingInvalid, ["previous_state", "state"])).toEqual([
      "missing required key 'previous_state'"
    ]);
    expect(
      validateWirePayloadConformance(metricsInvalid, [
        "sampled_at_unix_ms",
        "state",
        "baseline_ping_ms",
        "routed_ping_ms",
        "jitter_ms",
        "packet_loss_pct"
      ])
    ).toEqual([
      "missing required key 'state'",
      "missing required key 'routed_ping_ms'",
      "missing required key 'jitter_ms'",
      "missing required key 'packet_loss_pct'"
    ]);
    expect(validateWirePayloadConformance(detectionInvalid, ["state"])).toEqual([
      "missing required key 'state'"
    ]);
  });

  it("keeps deterministic event behavior across repeated subscribe-unsubscribe cycles", async () => {
    const deps = createStubDeps();
    const trace: string[] = [];
    let cycle = 0;

    vi.mocked(deps.listen).mockImplementation(async (eventName, handler) => {
      cycle += 1;
      const currentCycle = cycle;
      trace.push(`subscribe:${eventName}:${currentCycle}`);
      void handler({
        payload: {
          previous_state: "idle",
          state: "connecting",
          reason_code: undefined,
          message: undefined
        }
      });
      return async () => {
        trace.push(`unsubscribe:${eventName}:${currentCycle}`);
      };
    });

    const client = createIpcClient(deps);
    const onEvent = vi.fn();

    for (let run = 0; run < 3; run += 1) {
      const cleanup = await client.subscribeRoutingState(onEvent);
      await cleanup();
    }

    expect(onEvent).toHaveBeenCalledTimes(3);
    expect(trace).toEqual([
      "subscribe:routing_state_changed:1",
      "unsubscribe:routing_state_changed:1",
      "subscribe:routing_state_changed:2",
      "unsubscribe:routing_state_changed:2",
      "subscribe:routing_state_changed:3",
      "unsubscribe:routing_state_changed:3"
    ]);
  });

  it("listener registry guard blocks duplicate attachment attempts for same key", () => {
    const metadata = createListenerRegistryMetadata();
    expect(listenerRegistryGuard(metadata, "routing_state_changed")).toBe(true);
    expect(listenerRegistryGuard(metadata, "routing_state_changed")).toBe(false);
  });

  it("attachShellListeners keeps subscribe-unsubscribe cycle deterministic across repeated runs", async () => {
    const trace: string[] = [];
    const routingUnsubscribe = vi.fn(async () => {
      trace.push("unsubscribe:routing");
    });
    const detectionUnsubscribe = vi.fn(async () => {
      trace.push("unsubscribe:detection");
    });
    const metricsUnsubscribe = vi.fn(async () => {
      trace.push("unsubscribe:metrics");
    });

    const client: IpcClient = {
      invokeRoutingToggleOn: vi.fn(),
      invokeRoutingToggleOff: vi.fn(),
      invokeDetectionGetStatus: vi.fn(),
      subscribeRoutingState: vi.fn(async () => {
        trace.push("subscribe:routing");
        return routingUnsubscribe;
      }),
      subscribeDetectionStatus: vi.fn(async () => {
        trace.push("subscribe:detection");
        return detectionUnsubscribe;
      }),
      subscribePingMetrics: vi.fn(async () => {
        trace.push("subscribe:metrics");
        return metricsUnsubscribe;
      })
    };

    async function runCycle() {
      const metadata = createListenerRegistryMetadata();
      const detach = await attachShellListeners({
        client,
        metadata,
        handlers: {
          onRoutingStateChanged: vi.fn(),
          onDetectionStatusUpdated: vi.fn(),
          onMetricsPingSampled: vi.fn()
        }
      });
      await detach();
    }

    await runCycle();
    await runCycle();

    expect(trace).toEqual([
      "subscribe:routing",
      "subscribe:detection",
      "subscribe:metrics",
      "unsubscribe:routing",
      "unsubscribe:detection",
      "unsubscribe:metrics",
      "subscribe:routing",
      "subscribe:detection",
      "subscribe:metrics",
      "unsubscribe:routing",
      "unsubscribe:detection",
      "unsubscribe:metrics"
    ]);
  });

  it("attachShellListeners isolates attach failure and keeps app lifecycle operational", async () => {
    const onAttachError = vi.fn();
    const routingUnsubscribe = vi.fn(async () => undefined);
    const detectionUnsubscribe = vi.fn(async () => undefined);

    const client: IpcClient = {
      invokeRoutingToggleOn: vi.fn(),
      invokeRoutingToggleOff: vi.fn(),
      invokeDetectionGetStatus: vi.fn(),
      subscribeRoutingState: vi.fn(async () => routingUnsubscribe),
      subscribeDetectionStatus: vi.fn(async () => detectionUnsubscribe),
      subscribePingMetrics: vi.fn(async () => {
        throw new Error("metrics listener unavailable");
      })
    };

    const detach = await attachShellListeners({
      client,
      metadata: createListenerRegistryMetadata(),
      handlers: {
        onRoutingStateChanged: vi.fn(),
        onDetectionStatusUpdated: vi.fn(),
        onMetricsPingSampled: vi.fn()
      },
      onAttachError
    });

    await detach();
    expect(onAttachError).toHaveBeenCalledTimes(1);
    expect(onAttachError).toHaveBeenCalledWith({
      key: "metrics_ping_sampled",
      error: expect.any(Error)
    });
    expect(routingUnsubscribe).toHaveBeenCalledTimes(1);
    expect(detectionUnsubscribe).toHaveBeenCalledTimes(1);
  });
});
