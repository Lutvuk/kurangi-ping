import { useReducer } from "react";
import type { ConnectionStatusState, PrimaryToggleState } from "../../components/modules";
import type {
  DetectionStatusResponse,
  DetectionStatusUpdatedEventPayload,
  IpcReasonCode,
  MetricsPingSampledEventPayload,
  RoutingLifecycleResponse,
  RoutingLifecycleState,
  RoutingStateChangedEventPayload
} from "../../lib/ipc";
import type { DetectionViewModel } from "../detection";
import type { MetricsViewModel } from "../metrics";

export type AppShellRoutingViewModel = {
  lifecycleState: RoutingLifecycleState;
  toggleState: PrimaryToggleState;
  badgeState: ConnectionStatusState;
  reasonCode?: IpcReasonCode;
  message?: string;
};

export type AppShellIpcViewModel = {
  routing: AppShellRoutingViewModel;
  detection: DetectionViewModel;
  metrics: MetricsViewModel;
};

type AppShellIpcAction =
  | { type: "routing.command.started"; command: "on" | "off" }
  | { type: "routing.invoke.resolved"; response: RoutingLifecycleResponse }
  | { type: "routing.event.received"; payload: RoutingStateChangedEventPayload }
  | { type: "detection.query.resolved"; response: DetectionStatusResponse }
  | { type: "detection.event.received"; payload: DetectionStatusUpdatedEventPayload }
  | { type: "metrics.event.received"; payload: MetricsPingSampledEventPayload };

export type AppShellIpcActions = {
  markRoutingCommandStarted: (command: "on" | "off") => void;
  applyRoutingInvokeResponse: (response: RoutingLifecycleResponse) => void;
  applyRoutingStateEvent: (payload: RoutingStateChangedEventPayload) => void;
  applyDetectionQueryResponse: (response: DetectionStatusResponse) => void;
  applyDetectionStatusEvent: (payload: DetectionStatusUpdatedEventPayload) => void;
  applyMetricsSampleEvent: (payload: MetricsPingSampledEventPayload) => void;
};

export type AppShellIpcStateResult = {
  viewModel: AppShellIpcViewModel;
  actions: AppShellIpcActions;
};

const INITIAL_VIEW_MODEL: AppShellIpcViewModel = {
  routing: {
    lifecycleState: "idle",
    toggleState: "off",
    badgeState: "off"
  },
  detection: {
    state: "not_found"
  },
  metrics: {
    state: "idle",
    baselinePingMs: null,
    routedPingMs: null,
    jitterMs: null,
    packetLossPct: null
  }
};

function toRoutingVisualState(state: RoutingLifecycleState): {
  toggleState: PrimaryToggleState;
  badgeState: ConnectionStatusState;
} {
  if (state === "active") {
    return {
      toggleState: "on",
      badgeState: "on"
    };
  }
  if (state === "degraded") {
    return {
      toggleState: "degraded",
      badgeState: "degraded"
    };
  }
  if (state === "connecting") {
    return {
      toggleState: "connecting",
      badgeState: "connecting"
    };
  }
  if (state === "error") {
    return {
      toggleState: "off",
      badgeState: "error"
    };
  }
  return {
    toggleState: "off",
    badgeState: "off"
  };
}

function mapDetectionToViewModel(
  payload: DetectionStatusResponse | DetectionStatusUpdatedEventPayload
): DetectionViewModel {
  const nextState = payload.state === "detected" ? "detected" : payload.reasonCode ? "error" : "not_found";
  return {
    state: nextState,
    gameId: payload.gameId,
    processName: payload.processName,
    detectionTimeMs: payload.detectionTimeMs,
    reasonCode: payload.reasonCode,
    message: payload.message
  };
}

function mapRoutingToViewModel(
  state: RoutingLifecycleState,
  reasonCode?: IpcReasonCode,
  message?: string
): AppShellRoutingViewModel {
  const visual = toRoutingVisualState(state);
  return {
    lifecycleState: state,
    toggleState: visual.toggleState,
    badgeState: visual.badgeState,
    reasonCode,
    message
  };
}

function reduceAppShellIpcState(
  state: AppShellIpcViewModel,
  action: AppShellIpcAction
): AppShellIpcViewModel {
  if (action.type === "routing.command.started") {
    return {
      ...state,
      routing: mapRoutingToViewModel("connecting")
    };
  }

  if (action.type === "routing.invoke.resolved") {
    return {
      ...state,
      routing: mapRoutingToViewModel(
        action.response.state,
        action.response.reasonCode,
        action.response.message
      )
    };
  }

  if (action.type === "routing.event.received") {
    return {
      ...state,
      routing: mapRoutingToViewModel(
        action.payload.state,
        action.payload.reasonCode,
        action.payload.message
      )
    };
  }

  if (action.type === "detection.query.resolved") {
    return {
      ...state,
      detection: mapDetectionToViewModel(action.response)
    };
  }

  if (action.type === "detection.event.received") {
    return {
      ...state,
      detection: mapDetectionToViewModel(action.payload)
    };
  }

  return {
    ...state,
    metrics: {
      state: action.payload.state,
      baselinePingMs: action.payload.baselinePingMs,
      routedPingMs: action.payload.routedPingMs,
      jitterMs: action.payload.jitterMs,
      packetLossPct: action.payload.packetLossPct,
      sampledAtUnixMs: action.payload.sampledAtUnixMs,
      reasonCode: action.payload.reasonCode
    }
  };
}

export function useAppShellIpcState(): AppShellIpcStateResult {
  const [viewModel, dispatch] = useReducer(reduceAppShellIpcState, INITIAL_VIEW_MODEL);

  return {
    viewModel,
    actions: {
      markRoutingCommandStarted(command) {
        dispatch({
          type: "routing.command.started",
          command
        });
      },
      applyRoutingInvokeResponse(response) {
        dispatch({
          type: "routing.invoke.resolved",
          response
        });
      },
      applyRoutingStateEvent(payload) {
        dispatch({
          type: "routing.event.received",
          payload
        });
      },
      applyDetectionQueryResponse(response) {
        dispatch({
          type: "detection.query.resolved",
          response
        });
      },
      applyDetectionStatusEvent(payload) {
        dispatch({
          type: "detection.event.received",
          payload
        });
      },
      applyMetricsSampleEvent(payload) {
        dispatch({
          type: "metrics.event.received",
          payload
        });
      }
    }
  };
}

export function createInitialAppShellIpcViewModel(): AppShellIpcViewModel {
  return INITIAL_VIEW_MODEL;
}
