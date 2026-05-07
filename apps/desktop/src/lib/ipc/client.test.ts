import { describe, expect, it, vi } from "vitest";
import { createIpcClient, type IpcClientDeps } from "./client";

function createStubDeps(): IpcClientDeps {
  return {
    invoke: vi.fn(),
    listen: vi.fn()
  };
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
});
