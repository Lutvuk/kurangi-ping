import { describe, expect, it } from "vitest";
import {
  mapIpcErrorToUiReason,
  presentIpcFailureState,
  type IpcFailureContext
} from "./ipcErrorMapper";

describe("ipcErrorMapper", () => {
  it("maps each context to approved ipc reason code taxonomy", () => {
    const contexts: IpcFailureContext[] = [
      "routing_toggle_on",
      "routing_toggle_off",
      "detection_startup_query",
      "listener_attach_routing",
      "listener_attach_detection",
      "listener_attach_metrics"
    ];

    const mapped = contexts.map((context) => mapIpcErrorToUiReason({ context }));
    expect(mapped).toEqual([
      expect.objectContaining({ reasonCode: "ipc_unknown_failure" }),
      expect.objectContaining({ reasonCode: "ipc_unknown_failure" }),
      expect.objectContaining({ reasonCode: "ipc_detection_scan_failed" }),
      expect.objectContaining({ reasonCode: "ipc_timeout" }),
      expect.objectContaining({ reasonCode: "ipc_detection_scan_failed" }),
      expect.objectContaining({ reasonCode: "ipc_metrics_stream_unavailable" })
    ]);
  });

  it("keeps backend-provided reason code when available", () => {
    const mapped = mapIpcErrorToUiReason({
      context: "routing_toggle_on",
      reasonCode: "ipc_command_rejected"
    });
    expect(mapped).toEqual({
      reasonCode: "ipc_command_rejected",
      userMessage: "Tidak bisa memproses perintah ON routing sekarang. Coba lagi."
    });
  });

  it("presents routing failure with safe banner and error response", () => {
    const presentation = presentIpcFailureState({
      context: "routing_toggle_off",
      reasonCode: "ipc_invalid_state"
    });
    expect(presentation.uiError.reasonCode).toBe("ipc_invalid_state");
    expect(presentation.banner).toEqual({
      tone: "error",
      message: "Tidak bisa memproses perintah OFF routing sekarang. Coba lagi.",
      reasonCode: "ipc_invalid_state"
    });
    expect(presentation.routingResponse).toEqual({
      state: "error",
      reasonCode: "ipc_invalid_state",
      message: "Tidak bisa memproses perintah OFF routing sekarang. Coba lagi."
    });
  });

  it("presents detection startup failure with safe fallback response", () => {
    const presentation = presentIpcFailureState({
      context: "detection_startup_query"
    });
    expect(presentation.detectionResponse).toEqual({
      state: "not_detected",
      reasonCode: "ipc_detection_scan_failed",
      message: "Detection status tidak tersedia."
    });
  });

  it("presents metrics listener attach failure as degraded metrics payload", () => {
    const presentation = presentIpcFailureState({
      context: "listener_attach_metrics",
      atUnixMs: 1_700_000_900_000
    });
    expect(presentation.metricsPayload).toEqual({
      sampledAtUnixMs: 1_700_000_900_000,
      state: "degraded",
      baselinePingMs: null,
      routedPingMs: null,
      jitterMs: null,
      packetLossPct: null,
      reasonCode: "ipc_metrics_stream_unavailable"
    });
  });
});
