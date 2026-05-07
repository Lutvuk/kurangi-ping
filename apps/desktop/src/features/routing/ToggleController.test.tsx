import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { IpcClient, RoutingStateChangedEventPayload } from "../../lib/ipc";
import { ToggleController, type ToggleControllerCommandResult } from "./ToggleController";

describe("ToggleController", () => {
  it("ON click triggers ON pipeline command", async () => {
    const onEnableRouting = vi.fn(async () => ({
      ok: true,
      nextState: "on"
    } satisfies ToggleControllerCommandResult));

    render(<ToggleController initialState="off" onEnableRouting={onEnableRouting} />);
    fireEvent.click(screen.getByRole("button", { name: "Routing toggle off" }));

    await waitFor(() => expect(onEnableRouting).toHaveBeenCalledTimes(1));
    await waitFor(() =>
      expect(screen.getByTestId("toggle-controller-status")).toHaveTextContent("Routing Active")
    );
  });

  it("OFF click triggers OFF pipeline command", async () => {
    const onDisableRouting = vi.fn(async () => ({
      ok: true,
      nextState: "off"
    } satisfies ToggleControllerCommandResult));

    render(<ToggleController initialState="on" onDisableRouting={onDisableRouting} />);
    fireEvent.click(screen.getByRole("button", { name: "Routing toggle on" }));

    await waitFor(() => expect(onDisableRouting).toHaveBeenCalledTimes(1));
    await waitFor(() =>
      expect(screen.getByTestId("toggle-controller-status")).toHaveTextContent("Offline")
    );
  });

  it("blocks duplicate command while previous command is in progress", async () => {
    let resolveCommand: ((value: ToggleControllerCommandResult) => void) | undefined;
    const onEnableRouting = vi.fn(
      () =>
        new Promise<ToggleControllerCommandResult>((resolve) => {
          resolveCommand = resolve;
        })
    );

    render(<ToggleController initialState="off" onEnableRouting={onEnableRouting} />);

    const button = screen.getByRole("button", { name: "Routing toggle off" });
    fireEvent.click(button);
    fireEvent.click(button);

    expect(onEnableRouting).toHaveBeenCalledTimes(1);
    expect(button).toBeDisabled();
    expect(screen.getByTestId("toggle-controller-status")).toHaveTextContent("Connecting...");

    resolveCommand?.({ ok: true, nextState: "on" });
    await waitFor(() => expect(button).not.toBeDisabled());
  });

  it("surfaces non-technical error feedback on failure", async () => {
    const onEnableRouting = vi.fn(async () => {
      throw new Error("internal stacktrace should not be shown");
    });

    render(<ToggleController initialState="off" onEnableRouting={onEnableRouting} />);
    fireEvent.click(screen.getByRole("button", { name: "Routing toggle off" }));

    const alert = await screen.findByRole("alert");
    expect(alert).toHaveTextContent("Belum bisa mengaktifkan routing. Coba lagi.");
    expect(alert).not.toHaveTextContent("stacktrace");
    expect(screen.getByTestId("toggle-controller-status")).toHaveTextContent("No Relay Available");
  });

  it("uses IPC bridge invoke and cleans routing subscription on unmount", async () => {
    let routingListener: ((payload: RoutingStateChangedEventPayload) => void) | undefined;
    const unsubscribe = vi.fn(async () => undefined);
    const ipcClient: IpcClient = {
      invokeRoutingToggleOn: vi.fn(async () => ({ state: "connecting" as const })),
      invokeRoutingToggleOff: vi.fn(async () => ({ state: "idle" as const })),
      invokeDetectionGetStatus: vi.fn(async () => ({ state: "not_detected" as const })),
      subscribeRoutingState: vi.fn(async (handler) => {
        routingListener = handler;
        return unsubscribe;
      }),
      subscribePingMetrics: vi.fn(async () => unsubscribe),
      subscribeDetectionStatus: vi.fn(async () => unsubscribe)
    };

    const { unmount } = render(
      <ToggleController initialState="off" enableIpcBridge ipcClient={ipcClient} />
    );
    fireEvent.click(screen.getByRole("button", { name: "Routing toggle off" }));

    await waitFor(() => expect(ipcClient.invokeRoutingToggleOn).toHaveBeenCalledTimes(1));
    routingListener?.({
      previousState: "idle",
      state: "active"
    });

    await waitFor(() =>
      expect(screen.getByTestId("toggle-controller-status")).toHaveTextContent("Routing Active")
    );

    unmount();
    await waitFor(() => expect(unsubscribe).toHaveBeenCalledTimes(1));
  });

  it("keeps ipc journey trace deterministic across repeated runs", async () => {
    async function runJourney(): Promise<string[]> {
      const trace: string[] = [];
      let routingListener: ((payload: RoutingStateChangedEventPayload) => void) | undefined;
      const unsubscribe = vi.fn(async () => {
        trace.push("cleanup:routing_subscription");
      });

      const ipcClient: IpcClient = {
        invokeRoutingToggleOn: vi.fn(async () => {
          trace.push("invoke:routing_toggle_on");
          return { state: "connecting" as const };
        }),
        invokeRoutingToggleOff: vi.fn(async () => ({ state: "idle" as const })),
        invokeDetectionGetStatus: vi.fn(async () => ({ state: "not_detected" as const })),
        subscribeRoutingState: vi.fn(async (handler) => {
          trace.push("subscribe:routing_state_changed");
          routingListener = handler;
          return unsubscribe;
        }),
        subscribePingMetrics: vi.fn(async () => unsubscribe),
        subscribeDetectionStatus: vi.fn(async () => unsubscribe)
      };

      const { unmount } = render(
        <ToggleController initialState="off" enableIpcBridge ipcClient={ipcClient} />
      );

      fireEvent.click(screen.getByRole("button", { name: "Routing toggle off" }));
      await waitFor(() => expect(ipcClient.invokeRoutingToggleOn).toHaveBeenCalledTimes(1));

      trace.push("event:routing_state_changed:idle->connecting");
      routingListener?.({
        previousState: "idle",
        state: "connecting"
      });
      trace.push("event:routing_state_changed:connecting->active");
      routingListener?.({
        previousState: "connecting",
        state: "active"
      });

      await waitFor(() =>
        expect(screen.getByTestId("toggle-controller-status")).toHaveTextContent("Routing Active")
      );

      unmount();
      await waitFor(() => expect(unsubscribe).toHaveBeenCalledTimes(1));
      return trace;
    }

    const first = await runJourney();
    const second = await runJourney();
    expect(first).toEqual(second);
  });
});
