import { describe, expect, it } from "vitest";
import { IPC_COMMAND_LIST, IPC_COMMANDS, IPC_EVENT_LIST, IPC_EVENTS } from "./contracts";

describe("IPC contract registry", () => {
  it("centralizes all command names for routing toggle and detection status", () => {
    expect(IPC_COMMANDS).toEqual({
      routingToggleOn: "routing_toggle_on",
      routingToggleOff: "routing_toggle_off",
      detectionGetStatus: "detection_get_status"
    });
    expect(IPC_COMMAND_LIST).toEqual([
      "routing_toggle_on",
      "routing_toggle_off",
      "detection_get_status"
    ]);
  });

  it("centralizes all event names for routing, metrics, and detection streams", () => {
    expect(IPC_EVENTS).toEqual({
      routingStateChanged: "routing_state_changed",
      metricsPingSampled: "metrics_ping_sampled",
      detectionStatusUpdated: "detection_status_updated"
    });
    expect(IPC_EVENT_LIST).toEqual([
      "routing_state_changed",
      "metrics_ping_sampled",
      "detection_status_updated"
    ]);
  });

  it("keeps command and event registries deterministic and unique", () => {
    expect(new Set(IPC_COMMAND_LIST).size).toBe(IPC_COMMAND_LIST.length);
    expect(new Set(IPC_EVENT_LIST).size).toBe(IPC_EVENT_LIST.length);
  });
});

