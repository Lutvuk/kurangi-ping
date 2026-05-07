import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "./App";

const mocks = vi.hoisted(() => ({
  invokeRoutingToggleOn: vi.fn(),
  invokeRoutingToggleOff: vi.fn(),
  invokeDetectionGetStatus: vi.fn(),
  subscribeRoutingState: vi.fn(),
  subscribeDetectionStatus: vi.fn()
}));

vi.mock("./lib/ipc", () => ({
  ipcClient: {
    invokeRoutingToggleOn: mocks.invokeRoutingToggleOn,
    invokeRoutingToggleOff: mocks.invokeRoutingToggleOff,
    invokeDetectionGetStatus: mocks.invokeDetectionGetStatus,
    subscribeRoutingState: mocks.subscribeRoutingState,
    subscribeDetectionStatus: mocks.subscribeDetectionStatus
  }
}));

describe("App IPC toggle wiring", () => {
  beforeEach(() => {
    mocks.invokeRoutingToggleOn.mockReset();
    mocks.invokeRoutingToggleOff.mockReset();
    mocks.invokeDetectionGetStatus.mockReset();
    mocks.subscribeRoutingState.mockReset();
    mocks.subscribeDetectionStatus.mockReset();
    mocks.invokeDetectionGetStatus.mockResolvedValue({
      state: "not_detected"
    });
    mocks.subscribeRoutingState.mockResolvedValue(async () => undefined);
    mocks.subscribeDetectionStatus.mockResolvedValue(async () => undefined);
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
      expect(screen.getByTestId("routing-connection-status")).toHaveTextContent("Connecting...")
    );
  });

  it("calls detection_get_status once on startup and maps detected response into panel", async () => {
    mocks.invokeDetectionGetStatus.mockResolvedValue({
      state: "detected",
      gameId: "ffxiv",
      processName: "ffxiv_dx11.exe",
      detectionTimeMs: 1_700_000_000_100
    });

    render(<App />);

    await waitFor(() => expect(mocks.invokeDetectionGetStatus).toHaveBeenCalledTimes(1));
    const detectionPanel = screen.getByLabelText("Detection status panel");
    await waitFor(() =>
      expect(within(detectionPanel).getByRole("status")).toHaveTextContent("Detected")
    );
    expect(within(detectionPanel).getByText("Ffxiv")).toBeInTheDocument();
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
    await waitFor(() =>
      expect(screen.getByTestId("routing-connection-status")).toHaveTextContent("Routing Active")
    );

    fireEvent.click(screen.getByRole("button", { name: "Routing toggle on" }));
    await waitFor(() => expect(mocks.invokeRoutingToggleOff).toHaveBeenCalledTimes(1));
    expect(mocks.invokeRoutingToggleOff).toHaveBeenCalledWith({
      trigger: "user_toggle",
      requestedAtUnixMs: expect.any(Number)
    });
    await waitFor(() =>
      expect(screen.getByTestId("routing-connection-status")).toHaveTextContent("Offline")
    );
  });

  it("keeps startup detection query idempotent across app-shell rerenders", async () => {
    mocks.invokeRoutingToggleOn.mockResolvedValue({
      state: "connecting"
    });

    render(<App />);
    await waitFor(() => expect(mocks.invokeDetectionGetStatus).toHaveBeenCalledTimes(1));

    fireEvent.click(screen.getByRole("button", { name: "Routing toggle off" }));
    await waitFor(() => expect(mocks.invokeRoutingToggleOn).toHaveBeenCalledTimes(1));
    expect(mocks.invokeDetectionGetStatus).toHaveBeenCalledTimes(1);
  });

  it("maps invoke failure to user-safe feedback and error badge", async () => {
    mocks.invokeRoutingToggleOn.mockRejectedValue(new Error("internal backend stacktrace"));

    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Routing toggle off" }));

    const alert = await screen.findByRole("alert");
    expect(alert).toHaveTextContent("Tidak bisa memproses perintah routing sekarang. Coba lagi.");
    expect(alert).not.toHaveTextContent("stacktrace");
    await waitFor(() =>
      expect(screen.getByTestId("routing-connection-status")).toHaveTextContent("No Relay Available")
    );
  });

  it("maps startup detection failure to ui-safe error state", async () => {
    mocks.invokeDetectionGetStatus.mockRejectedValue(
      new Error("backend stacktrace should not leak")
    );

    render(<App />);

    await waitFor(() => expect(mocks.invokeDetectionGetStatus).toHaveBeenCalledTimes(1));
    const detectionPanel = screen.getByLabelText("Detection status panel");
    await waitFor(() =>
      expect(within(detectionPanel).getByRole("status")).toHaveTextContent("Error")
    );
    expect(within(detectionPanel).getByText("ipc_unknown_failure")).toBeInTheDocument();
    expect(within(detectionPanel).getByText("Detection status tidak tersedia.")).toBeInTheDocument();
    expect(screen.queryByText("stacktrace")).not.toBeInTheDocument();
  });

  it("refreshes detection panel from detection_status_updated event and cleans listener on unmount", async () => {
    let detectionListener:
      | ((payload: {
          state: "detected" | "not_detected";
          gameId?: string;
          processName?: string;
          detectionTimeMs?: number;
          reasonCode?: string;
          message?: string;
        }) => void)
      | undefined;
    const detectionUnsubscribe = vi.fn(async () => undefined);
    mocks.subscribeDetectionStatus.mockImplementation(async (handler) => {
      detectionListener = handler;
      return detectionUnsubscribe;
    });

    const { unmount } = render(<App />);

    await waitFor(() => expect(mocks.subscribeDetectionStatus).toHaveBeenCalledTimes(1));
    const detectionPanel = screen.getByLabelText("Detection status panel");
    expect(within(detectionPanel).getByRole("status")).toHaveTextContent("Not Found");

    detectionListener?.({
      state: "detected",
      gameId: "valorant",
      processName: "valorant-win64-shipping.exe",
      detectionTimeMs: 1_700_000_002_000
    });
    await waitFor(() => expect(within(detectionPanel).getByRole("status")).toHaveTextContent("Detected"));
    expect(within(detectionPanel).getByText("Valorant")).toBeInTheDocument();

    detectionListener?.({
      state: "not_detected"
    });
    await waitFor(() => expect(within(detectionPanel).getByRole("status")).toHaveTextContent("Not Found"));

    unmount();
    await waitFor(() => expect(detectionUnsubscribe).toHaveBeenCalledTimes(1));
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
    await waitFor(() =>
      expect(screen.getByTestId("routing-connection-status")).toHaveTextContent("Routing Active")
    );

    routingListener?.({
      previousState: "idle",
      state: "connecting"
    });
    await waitFor(() =>
      expect(screen.getByTestId("routing-connection-status")).toHaveTextContent("Routing Active")
    );

    unmount();
    await waitFor(() => expect(unsubscribe).toHaveBeenCalledTimes(1));
  });
});
