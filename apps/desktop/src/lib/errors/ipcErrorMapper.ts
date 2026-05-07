import type {
  DetectionStatusResponse,
  IpcReasonCode,
  MetricsPingSampledEventPayload,
  RoutingLifecycleResponse
} from "../ipc";

export type IpcFailureContext =
  | "routing_toggle_on"
  | "routing_toggle_off"
  | "detection_startup_query"
  | "listener_attach_routing"
  | "listener_attach_detection"
  | "listener_attach_metrics";

export type UiSafeIpcError = {
  reasonCode: IpcReasonCode;
  userMessage: string;
};

export type ShellErrorBannerState = {
  tone: "warning" | "error";
  message: string;
  reasonCode: IpcReasonCode;
};

export type IpcFailurePresentation = {
  uiError: UiSafeIpcError;
  banner?: ShellErrorBannerState;
  routingResponse?: RoutingLifecycleResponse;
  detectionResponse?: DetectionStatusResponse;
  metricsPayload?: MetricsPingSampledEventPayload;
};

type IpcErrorMappingInput = {
  context: IpcFailureContext;
  reasonCode?: IpcReasonCode;
};

function defaultReasonCodeByContext(context: IpcFailureContext): IpcReasonCode {
  if (context === "listener_attach_metrics") {
    return "ipc_metrics_stream_unavailable";
  }
  if (context === "listener_attach_routing") {
    return "ipc_timeout";
  }
  if (context === "detection_startup_query" || context === "listener_attach_detection") {
    return "ipc_detection_scan_failed";
  }
  return "ipc_unknown_failure";
}

function defaultMessageByContext(context: IpcFailureContext): string {
  if (context === "routing_toggle_on") {
    return "Tidak bisa memproses perintah ON routing sekarang. Coba lagi.";
  }
  if (context === "routing_toggle_off") {
    return "Tidak bisa memproses perintah OFF routing sekarang. Coba lagi.";
  }
  if (context === "detection_startup_query") {
    return "Detection status tidak tersedia.";
  }
  if (context === "listener_attach_routing") {
    return "Sinkronisasi status routing sedang tidak tersedia.";
  }
  if (context === "listener_attach_detection") {
    return "Sinkronisasi detection status tidak tersedia.";
  }
  return "Sinkronisasi metrics sedang tidak tersedia.";
}

export function mapIpcErrorToUiReason(input: IpcErrorMappingInput): UiSafeIpcError {
  const reasonCode = input.reasonCode ?? defaultReasonCodeByContext(input.context);
  return {
    reasonCode,
    userMessage: defaultMessageByContext(input.context)
  };
}

export function presentIpcFailureState(input: {
  context: IpcFailureContext;
  reasonCode?: IpcReasonCode;
  atUnixMs?: number;
}): IpcFailurePresentation {
  const uiError = mapIpcErrorToUiReason({
    context: input.context,
    reasonCode: input.reasonCode
  });

  if (input.context === "routing_toggle_on" || input.context === "routing_toggle_off") {
    return {
      uiError,
      banner: {
        tone: "error",
        message: uiError.userMessage,
        reasonCode: uiError.reasonCode
      },
      routingResponse: {
        state: "error",
        reasonCode: uiError.reasonCode,
        message: uiError.userMessage
      }
    };
  }

  if (input.context === "detection_startup_query" || input.context === "listener_attach_detection") {
    return {
      uiError,
      detectionResponse: {
        state: "not_detected",
        reasonCode: uiError.reasonCode,
        message: uiError.userMessage
      }
    };
  }

  if (input.context === "listener_attach_metrics") {
    return {
      uiError,
      metricsPayload: {
        sampledAtUnixMs: input.atUnixMs ?? Date.now(),
        state: "degraded",
        baselinePingMs: null,
        routedPingMs: null,
        jitterMs: null,
        packetLossPct: null,
        reasonCode: uiError.reasonCode
      }
    };
  }

  return {
    uiError,
    banner: {
      tone: "warning",
      message: uiError.userMessage,
      reasonCode: uiError.reasonCode
    }
  };
}
