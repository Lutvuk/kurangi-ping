import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { UpdaterPanel, type UpdaterViewModel } from "./UpdaterPanel";

function model(state: UpdaterViewModel["state"], overrides?: Partial<UpdaterViewModel>): UpdaterViewModel {
  return {
    state,
    channel: "stable",
    currentVersion: "1.0.0",
    ...overrides
  };
}

describe("UpdaterPanel", () => {
  function expectStatusText(text: string) {
    const status = screen.getByRole("status");
    expect(status).toHaveTextContent(text);
  }

  it("panel reflects normalized updater states", () => {
    const { rerender } = render(<UpdaterPanel model={model("up_to_date")} />);
    expectStatusText("Up To Date");

    rerender(<UpdaterPanel model={model("update_available", { targetVersion: "1.1.0" })} />);
    expectStatusText("Update Available");

    rerender(<UpdaterPanel model={model("downloading", { targetVersion: "1.1.0" })} />);
    expectStatusText("Downloading...");

    rerender(<UpdaterPanel model={model("ready_to_restart", { targetVersion: "1.1.0" })} />);
    expectStatusText("Ready To Restart");

    rerender(
      <UpdaterPanel
        model={model("update_error", { reasonCode: "updater_download_transport_failed" })}
      />
    );
    expectStatusText("Update Error");
    expect(screen.getByText("updater_download_transport_failed")).toBeInTheDocument();
  });

  it("controls support check, download, and apply actions as allowed by state", () => {
    const onCheckForUpdate = vi.fn();
    const onDownloadUpdate = vi.fn();
    const onApplyUpdate = vi.fn();
    const { rerender } = render(
      <UpdaterPanel model={model("up_to_date")} onCheckForUpdate={onCheckForUpdate} />
    );

    expect(screen.getByRole("button", { name: "Check updater status" })).toBeEnabled();
    expect(screen.queryByRole("button", { name: "Download update package" })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Apply downloaded update" })).not.toBeInTheDocument();

    rerender(
      <UpdaterPanel
        model={model("update_available", { targetVersion: "1.1.0", canApply: false })}
        onCheckForUpdate={onCheckForUpdate}
        onDownloadUpdate={onDownloadUpdate}
      />
    );
    expect(screen.getByRole("button", { name: "Download update package" })).toBeEnabled();
    expect(screen.queryByRole("button", { name: "Apply downloaded update" })).not.toBeInTheDocument();

    rerender(
      <UpdaterPanel
        model={model("update_available", { targetVersion: "1.1.0", canApply: true })}
        onCheckForUpdate={onCheckForUpdate}
        onApplyUpdate={onApplyUpdate}
      />
    );
    expect(screen.getByRole("button", { name: "Apply downloaded update" })).toBeEnabled();

    rerender(
      <UpdaterPanel
        model={model("downloading", { targetVersion: "1.1.0" })}
        onCheckForUpdate={onCheckForUpdate}
      />
    );
    expect(screen.getByRole("button", { name: "Check updater status" })).toBeDisabled();
  });

  it("error state provides actionable retry path", async () => {
    const onRetry = vi.fn(async () => ({
      ok: true,
      nextModel: model("up_to_date", {
        currentVersion: "1.1.0",
        lastCheckedAtLabel: "just now"
      })
    }));

    render(
      <UpdaterPanel
        model={model("update_error", {
          reasonCode: "updater_apply_failed",
          targetVersion: "1.1.0"
        })}
        onRetry={onRetry}
      />
    );

    fireEvent.click(screen.getByRole("button", { name: "Retry updater action" }));
    await waitFor(() => expect(onRetry).toHaveBeenCalledTimes(1));
    await waitFor(() => expectStatusText("Up To Date"));
    expect(screen.getByText(/last checked:\s*just now/i)).toBeInTheDocument();
  });

  it("styling remains aligned with design-system class contract", () => {
    const { container } = render(
      <UpdaterPanel model={model("ready_to_restart", { targetVersion: "1.1.0" })} />
    );
    const panel = container.querySelector(".kp-updater-panel");
    expect(panel).toHaveClass("kp-updater-panel--ready_to_restart");
  });
});
