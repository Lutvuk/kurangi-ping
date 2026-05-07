import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  IPC_COMMANDS,
  IPC_EVENTS,
  type DetectionStatusResponse,
  type DetectionStatusUpdatedEventPayload,
  type IpcReasonCode,
  type MetricsEmissionState,
  type MetricsPingSampledEventPayload,
  type RoutingLifecycleResponse,
  type RoutingStateChangedEventPayload,
  type RoutingToggleOffRequest,
  type RoutingToggleOnRequest
} from "./contracts";

type WireRoutingLifecycleResponse = {
  state: RoutingLifecycleResponse["state"];
  reason_code?: string;
  message?: string;
};

type WireDetectionStatusResponse = {
  state: DetectionStatusResponse["state"];
  game_id?: string;
  process_name?: string;
  detection_time_ms?: number;
  reason_code?: string;
  message?: string;
};

type WireRoutingStateChangedEventPayload = {
  previous_state: RoutingStateChangedEventPayload["previousState"];
  state: RoutingStateChangedEventPayload["state"];
  reason_code?: string;
  message?: string;
};

type WireMetricsPingSampledEventPayload = {
  sampled_at_unix_ms: number;
  state: MetricsEmissionState;
  baseline_ping_ms: number | null;
  routed_ping_ms: number | null;
  jitter_ms: number | null;
  packet_loss_pct: number | null;
  reason_code?: string;
};

type WireDetectionStatusUpdatedEventPayload = {
  state: DetectionStatusUpdatedEventPayload["state"];
  game_id?: string;
  process_name?: string;
  detection_time_ms?: number;
  reason_code?: string;
  message?: string;
};

type RoutingToggleOnInvokeArgs = {
  request: {
    trigger: RoutingToggleOnRequest["trigger"];
    requested_at_unix_ms: number;
  };
};

type RoutingToggleOffInvokeArgs = {
  request: {
    trigger: RoutingToggleOffRequest["trigger"];
    requested_at_unix_ms: number;
  };
};

type RawUnsubscribe = () => void | Promise<void>;
export type IpcUnsubscribe = () => Promise<void>;

export type IpcInvoke = <TResponse>(
  command: string,
  args?: Record<string, unknown>
) => Promise<TResponse>;

export type IpcListen = <TPayload>(
  eventName: string,
  handler: (event: { payload: TPayload }) => void
) => Promise<RawUnsubscribe>;

export type IpcClientDeps = {
  invoke: IpcInvoke;
  listen: IpcListen;
};

export type IpcClient = {
  invokeRoutingToggleOn: (request: RoutingToggleOnRequest) => Promise<RoutingLifecycleResponse>;
  invokeRoutingToggleOff: (request: RoutingToggleOffRequest) => Promise<RoutingLifecycleResponse>;
  invokeDetectionGetStatus: () => Promise<DetectionStatusResponse>;
  subscribeRoutingState: (
    handler: (payload: RoutingStateChangedEventPayload) => void
  ) => Promise<IpcUnsubscribe>;
  subscribePingMetrics: (
    handler: (payload: MetricsPingSampledEventPayload) => void
  ) => Promise<IpcUnsubscribe>;
  subscribeDetectionStatus: (
    handler: (payload: DetectionStatusUpdatedEventPayload) => void
  ) => Promise<IpcUnsubscribe>;
};

const REASON_CODES: readonly IpcReasonCode[] = [
  "ipc_invalid_state",
  "ipc_command_rejected",
  "ipc_detection_scan_failed",
  "ipc_metrics_stream_unavailable",
  "ipc_timeout",
  "ipc_unknown_failure"
];

const DEFAULT_DEPS: IpcClientDeps = {
  invoke: (command, args) => invoke(command, args),
  listen: (eventName, handler) => listen(eventName, handler)
};

export function createIpcClient(deps: IpcClientDeps = DEFAULT_DEPS): IpcClient {
  return {
    async invokeRoutingToggleOn(request) {
      const response = await deps.invoke<WireRoutingLifecycleResponse>(IPC_COMMANDS.routingToggleOn, {
        request: {
          trigger: request.trigger,
          requested_at_unix_ms: request.requestedAtUnixMs
        }
      } satisfies RoutingToggleOnInvokeArgs);
      return mapRoutingLifecycleResponse(response);
    },

    async invokeRoutingToggleOff(request) {
      const response = await deps.invoke<WireRoutingLifecycleResponse>(IPC_COMMANDS.routingToggleOff, {
        request: {
          trigger: request.trigger,
          requested_at_unix_ms: request.requestedAtUnixMs
        }
      } satisfies RoutingToggleOffInvokeArgs);
      return mapRoutingLifecycleResponse(response);
    },

    async invokeDetectionGetStatus() {
      const response = await deps.invoke<WireDetectionStatusResponse>(IPC_COMMANDS.detectionGetStatus);
      return mapDetectionStatusResponse(response);
    },

    async subscribeRoutingState(handler) {
      const rawUnsubscribe = await deps.listen<WireRoutingStateChangedEventPayload>(
        IPC_EVENTS.routingStateChanged,
        (event) => {
          handler(mapRoutingStateChangedEventPayload(event.payload));
        }
      );
      return once(rawUnsubscribe);
    },

    async subscribePingMetrics(handler) {
      const rawUnsubscribe = await deps.listen<WireMetricsPingSampledEventPayload>(
        IPC_EVENTS.metricsPingSampled,
        (event) => {
          handler(mapMetricsPingSampledEventPayload(event.payload));
        }
      );
      return once(rawUnsubscribe);
    },

    async subscribeDetectionStatus(handler) {
      const rawUnsubscribe = await deps.listen<WireDetectionStatusUpdatedEventPayload>(
        IPC_EVENTS.detectionStatusUpdated,
        (event) => {
          handler(mapDetectionStatusUpdatedEventPayload(event.payload));
        }
      );
      return once(rawUnsubscribe);
    }
  };
}

function once(rawUnsubscribe: RawUnsubscribe): IpcUnsubscribe {
  let released = false;
  return async () => {
    if (released) {
      return;
    }
    released = true;
    await rawUnsubscribe();
  };
}

function normalizeReasonCode(code: string | undefined): IpcReasonCode | undefined {
  if (!code) {
    return undefined;
  }
  if (REASON_CODES.includes(code as IpcReasonCode)) {
    return code as IpcReasonCode;
  }
  return "ipc_unknown_failure";
}

function mapRoutingLifecycleResponse(
  payload: WireRoutingLifecycleResponse
): RoutingLifecycleResponse {
  return {
    state: payload.state,
    reasonCode: normalizeReasonCode(payload.reason_code),
    message: payload.message
  };
}

function mapDetectionStatusResponse(
  payload: WireDetectionStatusResponse
): DetectionStatusResponse {
  return {
    state: payload.state,
    gameId: payload.game_id,
    processName: payload.process_name,
    detectionTimeMs: payload.detection_time_ms,
    reasonCode: normalizeReasonCode(payload.reason_code),
    message: payload.message
  };
}

function mapRoutingStateChangedEventPayload(
  payload: WireRoutingStateChangedEventPayload
): RoutingStateChangedEventPayload {
  return {
    previousState: payload.previous_state,
    state: payload.state,
    reasonCode: normalizeReasonCode(payload.reason_code),
    message: payload.message
  };
}

function mapMetricsPingSampledEventPayload(
  payload: WireMetricsPingSampledEventPayload
): MetricsPingSampledEventPayload {
  return {
    sampledAtUnixMs: payload.sampled_at_unix_ms,
    state: payload.state,
    baselinePingMs: payload.baseline_ping_ms,
    routedPingMs: payload.routed_ping_ms,
    jitterMs: payload.jitter_ms,
    packetLossPct: payload.packet_loss_pct,
    reasonCode: normalizeReasonCode(payload.reason_code)
  };
}

function mapDetectionStatusUpdatedEventPayload(
  payload: WireDetectionStatusUpdatedEventPayload
): DetectionStatusUpdatedEventPayload {
  return {
    state: payload.state,
    gameId: payload.game_id,
    processName: payload.process_name,
    detectionTimeMs: payload.detection_time_ms,
    reasonCode: normalizeReasonCode(payload.reason_code),
    message: payload.message
  };
}

export const ipcClient = createIpcClient();
