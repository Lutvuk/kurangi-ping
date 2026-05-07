import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "./App";

const mocks = vi.hoisted(() => ({
  invokeRoutingToggleOn: vi.fn(),
  invokeRoutingToggleOff: vi.fn(),
  subscribeRoutingState: vi.fn()
}));

vi.mock("./lib/ipc", () => ({
  ipcClient: {
    invokeRoutingToggleOn: mocks.invokeRoutingToggleOn,
    invokeRoutingToggleOff: mocks.invokeRoutingToggleOff,
    subscribeRoutingState: mocks.subscribeRoutingState
  }
}));

describe("App IPC toggle wiring", () => {
  beforeEach(() => {
    mocks.invokeRoutingToggleOn.mockReset();
    mocks.invokeRoutingToggleOff.mockReset();
    mocks.subscribeRoutingState.mockReset();
    mocks.subscribeRoutingState.mockResolvedValue(async () => undefined);
  });

  it("invokes routing_toggle_on via ipc client and sets connecting badge", async () => {
    mocks.invokeRoutingToggleOn.mockResolvedValue({
      state: "connecting"
    });

    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Routing toggle off" }));

    await waitFor(() => expect(mocks.invokeRoutingToggleOn).toHaveBeenCalledTimes(1));
    expect(mocks.invokeRoutingToggleOn).toHaveBeenCalledWith({
      trigger: "user_toggle",
      requestedAtUnixMs: expect.any(Number)
    });
    await waitFor(() =>
      expect(screen.getByRole("status")).toHaveTextContent("Connecting...")
    );
  });

  it("invokes routing_toggle_off and returns badge to offline", async () => {
    mocks.invokeRoutingToggleOn.mockResolvedValue({
      state: "active"
    });
    mocks.invokeRoutingToggleOff.mockResolvedValue({
      state: "idle"
    });

    render(<App />);

    fireEvent.click(screen.getByRole("button", { name: "Routing toggle off" }));
    await waitFor(() => expect(mocks.invokeRoutingToggleOn).toHaveBeenCalledTimes(1));
    await waitFor(() => expect(screen.getByRole("status")).toHaveTextContent("Routing Active"));

    fireEvent.click(screen.getByRole("button", { name: "Routing toggle on" }));
    await waitFor(() => expect(mocks.invokeRoutingToggleOff).toHaveBeenCalledTimes(1));
    expect(mocks.invokeRoutingToggleOff).toHaveBeenCalledWith({
      trigger: "user_toggle",
      requestedAtUnixMs: expect.any(Number)
    });
    await waitFor(() => expect(screen.getByRole("status")).toHaveTextContent("Offline"));
  });

  it("maps invoke failure to user-safe feedback and error badge", async () => {
    mocks.invokeRoutingToggleOn.mockRejectedValue(new Error("internal backend stacktrace"));

    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Routing toggle off" }));

    const alert = await screen.findByRole("alert");
    expect(alert).toHaveTextContent("Tidak bisa memproses perintah routing sekarang. Coba lagi.");
    expect(alert).not.toHaveTextContent("stacktrace");
    await waitFor(() =>
      expect(screen.getByRole("status")).toHaveTextContent("No Relay Available")
    );
  });

  it("syncs badge from routing_state_changed event and cleans listener on unmount", async () => {
    let routingListener:
      | ((payload: {
          previousState: "idle" | "connecting" | "active" | "degraded" | "error";
          state: "idle" | "connecting" | "active" | "degraded" | "error";
          reasonCode?: string;
          message?: string;
        }) => void)
      | undefined;
    const unsubscribe = vi.fn(async () => undefined);
    mocks.subscribeRoutingState.mockImplementation(async (handler) => {
      routingListener = handler;
      return unsubscribe;
    });

    const { unmount } = render(<App />);

    await waitFor(() => expect(mocks.subscribeRoutingState).toHaveBeenCalledTimes(1));
    routingListener?.({
      previousState: "idle",
      state: "active"
    });
    await waitFor(() => expect(screen.getByRole("status")).toHaveTextContent("Routing Active"));

    routingListener?.({
      previousState: "idle",
      state: "connecting"
    });
    await waitFor(() => expect(screen.getByRole("status")).toHaveTextContent("Routing Active"));

    unmount();
    await waitFor(() => expect(unsubscribe).toHaveBeenCalledTimes(1));
  });
});
