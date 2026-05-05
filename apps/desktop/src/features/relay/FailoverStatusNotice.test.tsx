import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { FailoverStatusNotice } from "./FailoverStatusNotice";
import type { RelayFailoverViewModel } from "./model";

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

describe("FailoverStatusNotice", () => {
  it("shows reconnecting/recovered/no-relay states with clear text", () => {
    const { rerender } = render(<FailoverStatusNotice failover={failover("switching")} />);
    let status = screen.getByRole("status");
    expect(status).toHaveTextContent("Reconnecting...");

    rerender(<FailoverStatusNotice failover={failover("recovered", { reasonCode: "switch_successful" })} />);
    status = screen.getByRole("status");
    expect(status).toHaveTextContent("Connection Recovered");

    rerender(
      <FailoverStatusNotice failover={failover("failed", { reasonCode: "switch_retry_exhausted" })} />
    );
    const alert = screen.getByRole("alert");
    expect(alert).toHaveTextContent("No Relay Available");
  });

  it("maps backend reason codes consistently into user-safe messages", () => {
    const { rerender } = render(
      <FailoverStatusNotice failover={failover("switching", { reasonCode: "relay_unreachable" })} />
    );
    expect(screen.getByText(/relay aktif tidak responsif/i)).toBeInTheDocument();

    rerender(
      <FailoverStatusNotice failover={failover("switching", { reasonCode: "hysteresis_window_active" })} />
    );
    expect(screen.getByText(/sedang distabilkan/i)).toBeInTheDocument();
  });

  it("avoids exposing technical internals in error states", () => {
    render(
      <FailoverStatusNotice
        failover={failover("failed", { reasonCode: "KERNEL_ERR_0xC0000005" })}
      />
    );
    expect(screen.getByText(/terjadi kendala relay/i)).toBeInTheDocument();
    expect(screen.queryByText(/0xC0000005/)).not.toBeInTheDocument();
  });

  it("uses accessible state announcements", () => {
    const { rerender } = render(<FailoverStatusNotice failover={failover("switching")} />);
    let status = screen.getByRole("status");
    expect(status).toHaveAttribute("aria-live", "polite");
    expect(status).toHaveAttribute("aria-atomic", "true");

    rerender(
      <FailoverStatusNotice failover={failover("failed", { reasonCode: "switch_retry_exhausted" })} />
    );
    const alert = screen.getByRole("alert");
    expect(alert).toHaveAttribute("aria-live", "assertive");
    expect(alert).toHaveAttribute("aria-atomic", "true");
  });
});
