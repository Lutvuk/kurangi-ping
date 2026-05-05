import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { RelayHealthPanel } from "./RelayHealthPanel";
import type { RelayFailoverViewModel, RelayHealthViewModel } from "./model";

function relays(): RelayHealthViewModel[] {
  return [
    {
      relayId: "sin-01",
      hostname: "sin-01.relay.local",
      region: "sin",
      latencyMs: 38,
      health: "ok"
    },
    {
      relayId: "nrt-01",
      hostname: "nrt-01.relay.local",
      region: "nrt",
      latencyMs: 122,
      health: "warn"
    },
    {
      relayId: "lax-01",
      hostname: "lax-01.relay.local",
      region: "lax",
      latencyMs: null,
      health: "dead"
    }
  ];
}

function failover(
  currentState: RelayFailoverViewModel["currentState"],
  overrides?: Partial<RelayFailoverViewModel>
): RelayFailoverViewModel {
  return {
    currentState,
    previousRelayId: "sin-01",
    nextRelayId: "nrt-01",
    reasonCode: "dead_relay_detected",
    ...overrides
  };
}

describe("RelayHealthPanel", () => {
  it("renders relay health classes that reflect live classification", () => {
    render(<RelayHealthPanel relays={relays()} failover={failover("stable")} />);

    const relayItems = screen.getAllByRole("listitem");
    expect(relayItems[0]).toHaveClass("kp-relay-item--ok");
    expect(relayItems[1]).toHaveClass("kp-relay-item--warn");
    expect(relayItems[2]).toHaveClass("kp-relay-item--dead");
  });

  it("syncs active relay highlight with engine failover state", () => {
    const { rerender } = render(
      <RelayHealthPanel relays={relays()} failover={failover("switching", { nextRelayId: "nrt-01" })} />
    );
    let relayItems = screen.getAllByRole("listitem");
    expect(relayItems[1]).toHaveClass("kp-relay-item--active");

    rerender(
      <RelayHealthPanel
        relays={relays()}
        failover={failover("stable")}
        activeRelayId="sin-01"
      />
    );
    relayItems = screen.getAllByRole("listitem");
    expect(relayItems[0]).toHaveClass("kp-relay-item--active");
  });

  it("maps failover lifecycle transitions to badge states and labels", () => {
    const { rerender } = render(
      <RelayHealthPanel relays={relays()} failover={failover("switching")} />
    );
    let status = screen.getAllByRole("status")[0];
    expect(status).toHaveClass("kp-connection-badge--connecting");
    expect(status).toHaveTextContent("Switching Relay...");

    rerender(<RelayHealthPanel relays={relays()} failover={failover("recovered")} />);
    status = screen.getAllByRole("status")[0];
    expect(status).toHaveClass("kp-connection-badge--on");
    expect(status).toHaveTextContent("Relay Recovered");

    rerender(<RelayHealthPanel relays={relays()} failover={failover("failed")} />);
    status = screen.getByRole("status");
    expect(status).toHaveClass("kp-connection-badge--error");
    expect(status).toHaveTextContent("No Relay Available");
  });

  it("shows non-technical reason text for failover messaging", () => {
    render(
      <RelayHealthPanel
        relays={relays()}
        failover={failover("failed", { reasonCode: "switch_retry_exhausted" })}
      />
    );

    expect(screen.getByText(/percobaan perpindahan relay sudah habis/i)).toBeInTheDocument();
  });
});
