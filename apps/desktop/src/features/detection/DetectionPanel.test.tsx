import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { DetectionPanel, type DetectionViewModel } from "./DetectionPanel";

function model(state: DetectionViewModel["state"], overrides?: Partial<DetectionViewModel>): DetectionViewModel {
  return {
    state,
    gameId: "ffxiv",
    processName: "ffxiv_dx11.exe",
    detectionTimeMs: 1200,
    ...overrides
  };
}

describe("DetectionPanel", () => {
  function expectStatusText(text: string) {
    const status = screen.getByRole("status");
    expect(status).toHaveTextContent(text);
  }

  it("renders normalized states with semantic status labels", () => {
    const { rerender } = render(<DetectionPanel model={model("not_found")} />);
    expectStatusText("Not Found");

    rerender(<DetectionPanel model={model("detected")} />);
    expectStatusText("Detected");

    rerender(<DetectionPanel model={model("stale")} />);
    expectStatusText("Stale");

    rerender(<DetectionPanel model={model("error", { reasonCode: "permission_denied" })} />);
    expectStatusText("Error");
    expect(screen.getByText("permission_denied")).toBeInTheDocument();
  });

  it("rescan action triggers command and refreshes detection state", async () => {
    const onTriggerRescan = vi.fn(async () =>
      model("detected", {
        gameId: "valorant",
        processName: "valorant-win64-shipping.exe",
        detectionTimeMs: 980
      })
    );

    render(
      <DetectionPanel
        model={model("not_found", {
          gameId: "unknown",
          processName: "process unavailable"
        })}
        onTriggerRescan={onTriggerRescan}
      />
    );

    fireEvent.click(screen.getByRole("button", { name: "Rescan detection status" }));

    await waitFor(() => expect(onTriggerRescan).toHaveBeenCalledTimes(1));
    await waitFor(() => expectStatusText("Detected"));
    expect(screen.getByText("Valorant")).toBeInTheDocument();
  });

  it("debounces concurrent rescans and keeps keyboard-focusable action", async () => {
    let resolveRescan: ((value: DetectionViewModel) => void) | undefined;
    const onTriggerRescan = vi.fn(
      () =>
        new Promise<DetectionViewModel>((resolve) => {
          resolveRescan = resolve;
        })
    );

    render(<DetectionPanel model={model("not_found")} onTriggerRescan={onTriggerRescan} />);

    const button = screen.getByRole("button", { name: "Rescan detection status" });
    button.focus();
    expect(button).toHaveFocus();

    fireEvent.click(button);
    fireEvent.click(button);

    expect(onTriggerRescan).toHaveBeenCalledTimes(1);
    expect(button).toBeDisabled();

    resolveRescan?.(model("detected"));
    await waitFor(() => expect(button).not.toBeDisabled());
  });

  it("shows fallback error state when rescan command throws", async () => {
    const onTriggerRescan = vi.fn(async () => {
      throw new Error("boom");
    });

    render(<DetectionPanel model={model("not_found")} onTriggerRescan={onTriggerRescan} />);
    fireEvent.click(screen.getByRole("button", { name: "Rescan detection status" }));

    await waitFor(() => expectStatusText("Error"));
    expect(screen.getByText("rescan_failed")).toBeInTheDocument();
  });
});
