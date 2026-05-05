import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { MetricsTrendMiniView, type MetricsTrendSample } from "./MetricsTrendMiniView";

function samples(values: Array<number | null>): MetricsTrendSample[] {
  return values.map((value, index) => ({
    sampledAtUnixMs: 1_700_000_000_000 + index * 1_000,
    routedPingMs: value
  }));
}

describe("MetricsTrendMiniView", () => {
  it("renders recent sample sequence as lightweight sparkline", () => {
    render(<MetricsTrendMiniView state="live" samples={samples([158.4, 155.1, 152.8, 150.5])} />);

    const svg = screen.getByTestId("metrics-trend-svg");
    expect(svg).toBeInTheDocument();
    const polyline = svg.querySelector("polyline");
    expect(polyline).not.toBeNull();
    expect(polyline?.getAttribute("points")).not.toBe("");
    expect(screen.getByText(/Latest 150.5 ms/)).toBeInTheDocument();
  });

  it("gracefully handles low sample counts", () => {
    const { rerender } = render(<MetricsTrendMiniView samples={samples([])} />);
    expect(screen.getByTestId("metrics-trend-empty")).toHaveTextContent("No trend data yet");

    rerender(<MetricsTrendMiniView samples={samples([153.2])} />);
    expect(screen.getByTestId("metrics-trend-empty")).toHaveTextContent("Collecting more samples...");
  });

  it("trims to maxPoints to keep overhead minimal", () => {
    render(<MetricsTrendMiniView maxPoints={3} samples={samples([170, 165, 160, 155, 150])} />);
    expect(screen.getByText(/Latest 150 ms/)).toBeInTheDocument();
    expect(screen.getByTestId("metrics-trend-svg")).toBeInTheDocument();
  });
});
