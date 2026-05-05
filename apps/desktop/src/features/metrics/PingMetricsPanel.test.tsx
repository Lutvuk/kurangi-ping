import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { PingMetricsPanel, type MetricsViewModel } from "./PingMetricsPanel";

function model(state: MetricsViewModel["state"], overrides?: Partial<MetricsViewModel>): MetricsViewModel {
  return {
    state,
    baselinePingMs: 210.4,
    routedPingMs: 154.2,
    jitterMs: 4.1,
    packetLossPct: 0.0,
    sampledAtUnixMs: 1_700_000_000_000,
    ...overrides
  };
}

describe("PingMetricsPanel", () => {
  it("updates baseline routed and reduction values in near realtime", () => {
    const { rerender } = render(<PingMetricsPanel model={model("live")} />);
    expect(screen.getByLabelText("Ping metrics")).toHaveTextContent("154.2");
    expect(screen.getByLabelText("Ping metrics")).toHaveTextContent("210.4");

    rerender(
      <PingMetricsPanel
        model={model("live", {
          baselinePingMs: 205.0,
          routedPingMs: 148.6
        })}
      />
    );
    expect(screen.getByLabelText("Ping metrics")).toHaveTextContent("148.6");
    expect(screen.getByLabelText("Ping metrics")).toHaveTextContent("205");
  });

  it("keeps numeric formatting stable and legible", () => {
    render(
      <PingMetricsPanel
        model={model("live", {
          baselinePingMs: 230.0,
          routedPingMs: 160.0,
          jitterMs: 2.35,
          packetLossPct: 11.05
        })}
      />
    );

    expect(screen.getByTestId("metrics-jitter")).toHaveTextContent("Jitter: 2.4 ms");
    expect(screen.getByTestId("metrics-packet-loss")).toHaveTextContent("Packet Loss: 11.1%");
  });

  it("aligns color semantics with live and degraded state", () => {
    const { rerender } = render(<PingMetricsPanel model={model("live")} />);
    expect(screen.getByLabelText("Ping metrics").className).toContain("kp-ping-card--on");
    expect(screen.getByTestId("metrics-panel-status")).toHaveTextContent("Live");

    rerender(<PingMetricsPanel model={model("measuring")} />);
    expect(screen.getByLabelText("Ping metrics").className).toContain("kp-ping-card--connecting");
    expect(screen.getByTestId("metrics-panel-status")).toHaveTextContent("Measuring");

    rerender(<PingMetricsPanel model={model("degraded")} />);
    expect(screen.getByLabelText("Ping metrics").className).toContain("kp-ping-card--degraded");
    expect(screen.getByTestId("metrics-panel-status")).toHaveTextContent("Degraded");
  });

  it("avoids flicker under rapid samples with temporary null fields", () => {
    const { rerender } = render(<PingMetricsPanel model={model("live")} />);

    rerender(
      <PingMetricsPanel
        model={model("degraded", {
          routedPingMs: null,
          jitterMs: null,
          packetLossPct: null,
          reasonCode: "freshness_timeout"
        })}
      />
    );

    expect(screen.getByLabelText("Ping metrics")).toHaveTextContent("154.2");
    expect(screen.getByTestId("metrics-jitter")).toHaveTextContent("4.1 ms");
    expect(screen.getByTestId("metrics-packet-loss")).toHaveTextContent("0%");
    expect(screen.getByText("freshness_timeout")).toBeInTheDocument();
  });

  it("keeps core dashboard delivery unblocked when optional trend module is omitted", () => {
    render(<PingMetricsPanel model={model("live")} />);
    expect(screen.getByLabelText("Ping metrics")).toBeInTheDocument();
    expect(screen.queryByTestId("metrics-trend-mini")).not.toBeInTheDocument();
  });
});
