import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
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
});
