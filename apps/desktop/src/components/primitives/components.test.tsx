import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { Button, Card, Input, Modal, Select, StatusBadge, Toast } from "./index";

describe("primitive components", () => {
  it("renders button variants with token classes", () => {
    render(
      <div>
        <Button intent="primary">Primary</Button>
        <Button intent="secondary">Secondary</Button>
        <Button intent="destructive">Destructive</Button>
      </div>
    );

    expect(screen.getByRole("button", { name: "Primary" })).toHaveClass("kp-button--primary");
    expect(screen.getByRole("button", { name: "Secondary" })).toHaveClass("kp-button--secondary");
    expect(screen.getByRole("button", { name: "Destructive" })).toHaveClass(
      "kp-button--destructive"
    );
  });

  it("renders explicit disabled states for input/select/button", () => {
    render(
      <div>
        <Button disabled>Disabled Button</Button>
        <Input aria-label="Ping host" disabled placeholder="Host" />
        <Select
          aria-label="Relay region"
          disabled
          options={[
            { value: "auto", label: "Auto" },
            { value: "sin", label: "Singapore" }
          ]}
        />
      </div>
    );

    expect(screen.getByRole("button", { name: "Disabled Button" })).toBeDisabled();
    expect(screen.getByRole("textbox", { name: "Ping host" })).toBeDisabled();
    expect(screen.getByRole("combobox", { name: "Relay region" })).toBeDisabled();
  });

  it("maps status badge semantics for off connecting on degraded", () => {
    render(
      <div>
        <StatusBadge state="off" />
        <StatusBadge state="connecting" />
        <StatusBadge state="on" />
        <StatusBadge state="degraded" />
      </div>
    );

    const badges = screen.getAllByRole("status");
    expect(badges[0]).toHaveClass("kp-status-badge--off");
    expect(badges[1]).toHaveClass("kp-status-badge--connecting");
    expect(badges[2]).toHaveClass("kp-status-badge--on");
    expect(badges[3]).toHaveClass("kp-status-badge--degraded");
  });

  it("renders card with sharp surface class", () => {
    render(<Card data-testid="metric-card">Ping metrics</Card>);
    expect(screen.getByTestId("metric-card")).toHaveClass("kp-card");
  });

  it("supports modal keyboard close and basic tab trap", () => {
    const onClose = vi.fn();
    render(
      <Modal open title="Disconnect?" onClose={onClose}>
        <button type="button">Retry</button>
        <button type="button">Continue</button>
      </Modal>
    );

    const dialog = screen.getByRole("dialog", { name: "Disconnect?" });
    fireEvent.keyDown(dialog, { key: "Escape" });
    expect(onClose).toHaveBeenCalledTimes(1);

    const continueButton = screen.getByRole("button", { name: "Continue" });
    continueButton.focus();
    fireEvent.keyDown(dialog, { key: "Tab" });
    expect(screen.getByRole("button", { name: "Close modal" })).toHaveFocus();
  });

  it("maps toast tones to semantic accents", () => {
    render(
      <div>
        <Toast tone="info" title="Info" />
        <Toast tone="warning" title="Warning" />
        <Toast tone="error" title="Error" />
        <Toast tone="success" title="Success" />
      </div>
    );

    expect(screen.getByText("Info").closest("div")).toHaveClass("kp-toast--info");
    expect(screen.getByText("Warning").closest("div")).toHaveClass("kp-toast--warning");
    expect(screen.getByText("Error").closest("div")).toHaveClass("kp-toast--error");
    expect(screen.getByText("Success").closest("div")).toHaveClass("kp-toast--success");
  });
});
