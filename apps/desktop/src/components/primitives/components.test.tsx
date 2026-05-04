import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { Button, Input, Select, StatusBadge } from "./index";

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
});
