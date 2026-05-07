export const IPC_COMMANDS = {
  routingToggleOn: "routing_toggle_on",
  routingToggleOff: "routing_toggle_off",
  detectionGetStatus: "detection_get_status"
} as const;

export type IpcCommand = (typeof IPC_COMMANDS)[keyof typeof IPC_COMMANDS];

export const IPC_EVENTS = {
  routingStateChanged: "routing_state_changed",
  metricsPingSampled: "metrics_ping_sampled",
  detectionStatusUpdated: "detection_status_updated"
} as const;

export type IpcEvent = (typeof IPC_EVENTS)[keyof typeof IPC_EVENTS];

export const IPC_COMMAND_LIST: readonly IpcCommand[] = Object.freeze(Object.values(IPC_COMMANDS));
export const IPC_EVENT_LIST: readonly IpcEvent[] = Object.freeze(Object.values(IPC_EVENTS));

// Non-sensitive reason codes only. Never include process args, full paths, usernames, or IP addresses.
export type IpcReasonCode =
  | "ipc_invalid_state"
  | "ipc_command_rejected"
  | "ipc_detection_scan_failed"
  | "ipc_metrics_stream_unavailable"
  | "ipc_timeout"
  | "ipc_unknown_failure";

export type RoutingLifecycleState = "idle" | "connecting" | "active" | "degraded" | "error";

export type RoutingToggleOnRequest = {
  trigger: "user_toggle" | "startup_resume";
  requestedAtUnixMs: number;
};

export type RoutingToggleOffRequest = {
  trigger: "user_toggle" | "shutdown" | "session_end";
  requestedAtUnixMs: number;
};

export type RoutingLifecycleResponse = {
  state: RoutingLifecycleState;
  reasonCode?: IpcReasonCode;
  message?: string;
};

export type DetectionStatusState = "detected" | "not_detected";

export type DetectionStatusResponse = {
  state: DetectionStatusState;
  gameId?: string;
  processName?: string;
  detectionTimeMs?: number;
  reasonCode?: IpcReasonCode;
  message?: string;
};

export type RoutingStateChangedEventPayload = {
  previousState: RoutingLifecycleState;
  state: RoutingLifecycleState;
  reasonCode?: IpcReasonCode;
  message?: string;
};

export type MetricsEmissionState = "live" | "measuring" | "degraded";

export type MetricsPingSampledEventPayload = {
  sampledAtUnixMs: number;
  state: MetricsEmissionState;
  baselinePingMs: number | null;
  routedPingMs: number | null;
  jitterMs: number | null;
  packetLossPct: number | null;
  reasonCode?: IpcReasonCode;
};

export type DetectionStatusUpdatedEventPayload = {
  state: DetectionStatusState;
  gameId?: string;
  processName?: string;
  detectionTimeMs?: number;
  reasonCode?: IpcReasonCode;
  message?: string;
};

