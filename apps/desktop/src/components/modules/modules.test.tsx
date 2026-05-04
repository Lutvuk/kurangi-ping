import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ConnectionStatusBadge, PrimaryToggle } from "./index";

describe("module components", () => {
  it("renders primary toggle states clearly", () => {
    render(
      <div>
        <PrimaryToggle state="off" />
        <PrimaryToggle state="connecting" />
        <PrimaryToggle state="on" />
        <PrimaryToggle state="degraded" />
      </div>
    );

    expect(screen.getByRole("button", { name: "Routing toggle off" })).toHaveClass(
      "kp-primary-toggle--off"
    );
    expect(screen.getByRole("button", { name: "Routing toggle connecting" })).toHaveClass(
      "kp-primary-toggle--connecting"
    );
    expect(screen.getByRole("button", { name: "Routing toggle on" })).toHaveClass(
      "kp-primary-toggle--on"
    );
    expect(screen.getByRole("button", { name: "Routing toggle degraded" })).toHaveClass(
      "kp-primary-toggle--degraded"
    );
  });

  it("emits next enabled intent on toggle click", () => {
    const onToggle = vi.fn();
    render(<PrimaryToggle state="off" onToggle={onToggle} />);

    fireEvent.click(screen.getByRole("button", { name: "Routing toggle off" }));
    expect(onToggle).toHaveBeenCalledWith(true);
  });

  it("maps connection status badge states", () => {
    render(
      <div>
        <ConnectionStatusBadge state="off" />
        <ConnectionStatusBadge state="connecting" />
        <ConnectionStatusBadge state="on" />
        <ConnectionStatusBadge state="degraded" />
      </div>
    );

    const badges = screen.getAllByRole("status");
    expect(badges[0]).toHaveClass("kp-connection-badge--off");
    expect(badges[1]).toHaveClass("kp-connection-badge--connecting");
    expect(badges[2]).toHaveClass("kp-connection-badge--on");
    expect(badges[3]).toHaveClass("kp-connection-badge--degraded");
  });
});
