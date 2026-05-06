import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { RestartPrompt, type RestartPromptModel } from "./RestartPrompt";

function model(overrides?: Partial<RestartPromptModel>): RestartPromptModel {
  return {
    updaterState: "up_to_date",
    currentVersion: "1.0.0",
    targetVersion: "1.1.0",
    ...overrides
  };
}

describe("RestartPrompt", () => {
  it("prompt appears only when update is ready-to-restart", () => {
    const { rerender } = render(<RestartPrompt model={model({ updaterState: "up_to_date" })} />);
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();

    rerender(<RestartPrompt model={model({ updaterState: "ready_to_restart" })} />);
    expect(screen.getByRole("dialog", { name: "Restart Required" })).toBeInTheDocument();

    rerender(<RestartPrompt model={model({ updaterState: "downloading" })} />);
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });

  it("user can defer restart safely", async () => {
    const onDeferRestart = vi.fn(async () => undefined);
    render(
      <RestartPrompt model={model({ updaterState: "ready_to_restart" })} onDeferRestart={onDeferRestart} />
    );

    fireEvent.click(screen.getByRole("button", { name: "Defer restart until later" }));
    await waitFor(() => expect(onDeferRestart).toHaveBeenCalledTimes(1));
    await waitFor(() => expect(screen.queryByRole("dialog")).not.toBeInTheDocument());
    expect(screen.getByLabelText("Restart deferred notice")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Open restart prompt again" })).toBeInTheDocument();
  });

  it("post-update confirmation state is visible on relaunch", () => {
    render(
      <RestartPrompt
        model={model({
          updaterState: "up_to_date",
          currentVersion: "1.1.0",
          relaunchConfirmationVersion: "1.1.0"
        })}
      />
    );
    expect(screen.getByLabelText("Post-update confirmation")).toBeInTheDocument();
    expect(screen.getByText(/versi/i)).toHaveTextContent("1.1.0");
  });

  it("accessibility checks pass for modal interactions", async () => {
    const onDeferRestart = vi.fn(async () => undefined);
    render(
      <RestartPrompt model={model({ updaterState: "ready_to_restart" })} onDeferRestart={onDeferRestart} />
    );

    const dialog = screen.getByRole("dialog", { name: "Restart Required" });
    expect(dialog).toHaveAttribute("aria-modal", "true");

    const closeButton = screen.getByRole("button", { name: "Close modal" });
    await waitFor(() => expect(closeButton).toHaveFocus());

    fireEvent.keyDown(dialog, { key: "Escape" });
    await waitFor(() => expect(onDeferRestart).toHaveBeenCalledTimes(1));
    await waitFor(() => expect(screen.queryByRole("dialog")).not.toBeInTheDocument());
  });
});
