import { act, renderHook } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { useAppShellIpcState } from "./useAppShellIpcState";

describe("useAppShellIpcState", () => {
  it("provides default shell view model compatible with current UI contracts", () => {
    const { result } = renderHook(() => useAppShellIpcState());

    expect(result.current.viewModel.routing).toEqual({
      lifecycleState: "idle",
      toggleState: "off",
      badgeState: "off"
    });
    expect(result.current.viewModel.detection).toEqual({
      state: "not_found"
    });
    expect(result.current.viewModel.metrics).toEqual({
      state: "idle",
      baselinePingMs: null,
      routedPingMs: null,
      jitterMs: null,
      packetLossPct: null
    });
  });

  it("applies routing invoke response transitions deterministically", () => {
    const { result } = renderHook(() => useAppShellIpcState());

    act(() => {
      result.current.actions.markRoutingCommandStarted("on");
    });
    expect(result.current.viewModel.routing).toMatchObject({
      lifecycleState: "connecting",
      toggleState: "connecting",
      badgeState: "connecting"
    });

    act(() => {
      result.current.actions.applyRoutingInvokeResponse({
        state: "active"
      });
    });
    expect(result.current.viewModel.routing).toMatchObject({
      lifecycleState: "active",
      toggleState: "on",
      badgeState: "on"
    });
  });

  it("updates routing from engine event payload and preserves reason metadata", () => {
    const { result } = renderHook(() => useAppShellIpcState());

    act(() => {
      result.current.actions.applyRoutingStateEvent({
        previousState: "connecting",
        state: "error",
        reasonCode: "ipc_command_rejected",
        message: "rejected by backend policy"
      });
    });

    expect(result.current.viewModel.routing).toEqual({
      lifecycleState: "error",
      toggleState: "off",
      badgeState: "error",
      reasonCode: "ipc_command_rejected",
      message: "rejected by backend policy"
    });
  });

  it("maps startup detection response and subsequent detection event into detection panel model", () => {
    const { result } = renderHook(() => useAppShellIpcState());

    act(() => {
      result.current.actions.applyDetectionQueryResponse({
        state: "not_detected"
      });
    });
    expect(result.current.viewModel.detection).toEqual({
      state: "not_found",
      gameId: undefined,
      processName: undefined,
      detectionTimeMs: undefined,
      reasonCode: undefined,
      message: undefined
    });

    act(() => {
      result.current.actions.applyDetectionStatusEvent({
        state: "detected",
        gameId: "ffxiv",
        processName: "ffxiv_dx11.exe",
        detectionTimeMs: 1_700_000_001_000
      });
    });
    expect(result.current.viewModel.detection).toMatchObject({
      state: "detected",
      gameId: "ffxiv",
      processName: "ffxiv_dx11.exe",
      detectionTimeMs: 1_700_000_001_000
    });
  });

  it("maps metrics event payload into live metrics view model", () => {
    const { result } = renderHook(() => useAppShellIpcState());

    act(() => {
      result.current.actions.applyMetricsSampleEvent({
        sampledAtUnixMs: 1_700_000_002_000,
        state: "live",
        baselinePingMs: 220,
        routedPingMs: 160,
        jitterMs: 3.6,
        packetLossPct: 0.3
      });
    });

    expect(result.current.viewModel.metrics).toEqual({
      sampledAtUnixMs: 1_700_000_002_000,
      state: "live",
      baselinePingMs: 220,
      routedPingMs: 160,
      jitterMs: 3.6,
      packetLossPct: 0.3,
      reasonCode: undefined
    });
  });
});
