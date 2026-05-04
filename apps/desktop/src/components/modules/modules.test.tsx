import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ConnectionStatusBadge, PingMetricCard, PrimaryToggle, RelayHealthListItem } from "./index";

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

  it("renders ping metric card with distinct values and units", () => {
    render(<PingMetricCard state="on" currentPingMs={42} baselinePingMs={71} reductionPct={40.8} />);

    const card = screen.getByLabelText("Ping metrics");
    expect(card).toHaveClass("kp-ping-card");
    expect(card).toHaveClass("kp-ping-card--on");
    expect(screen.getByText("Current")).toBeInTheDocument();
    expect(screen.getByText("Baseline")).toBeInTheDocument();
    expect(screen.getByText("Reduction")).toBeInTheDocument();
    expect(screen.getByText("42")).toBeInTheDocument();
    expect(screen.getByText("71")).toBeInTheDocument();
    expect(screen.getByText("40.8")).toBeInTheDocument();
    expect(screen.getAllByText("ms")).toHaveLength(2);
    expect(screen.getByText("%")).toBeInTheDocument();
  });

  it("renders relay list item active and health states", () => {
    render(
      <div role="list">
        <RelayHealthListItem hostname="sin-01.relay.local" latencyMs={38} region="sin" health="ok" />
        <RelayHealthListItem
          hostname="nrt-01.relay.local"
          latencyMs={125}
          region="nrt"
          health="warn"
          active
        />
      </div>
    );

    const items = screen.getAllByRole("listitem");
    expect(items[0]).toHaveClass("kp-relay-item--ok");
    expect(items[1]).toHaveClass("kp-relay-item--warn");
    expect(items[1]).toHaveClass("kp-relay-item--active");
    expect(screen.getByText("sin-01.relay.local")).toBeInTheDocument();
    expect(screen.getByText("nrt-01.relay.local")).toBeInTheDocument();
    expect(screen.getByText("SIN")).toBeInTheDocument();
    expect(screen.getByText("NRT")).toBeInTheDocument();
    expect(screen.getAllByText("ms")).toHaveLength(2);
  });
});
