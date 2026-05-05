import { memo, useMemo } from "react";
import type { MetricsPanelState } from "./PingMetricsPanel";
import "./MetricsTrendMiniView.css";

export type MetricsTrendSample = {
  sampledAtUnixMs: number;
  routedPingMs: number | null;
};

export type MetricsTrendMiniViewProps = {
  samples: MetricsTrendSample[];
  state?: MetricsPanelState;
  title?: string;
  maxPoints?: number;
};

const VIEWBOX_WIDTH = 100;
const VIEWBOX_HEIGHT = 28;

function formatMs(value: number | null): string {
  if (value === null || Number.isNaN(value)) {
    return "--";
  }
  const formatter = new Intl.NumberFormat("en-US", {
    minimumFractionDigits: 0,
    maximumFractionDigits: 1
  });
  return `${formatter.format(value)} ms`;
}

function clampTrendSample(value: number | null): number | null {
  if (value === null || Number.isNaN(value) || !Number.isFinite(value) || value < 0) {
    return null;
  }
  return value;
}

function buildPolylinePoints(values: number[]): string {
  if (values.length < 2) {
    return "";
  }

  const min = Math.min(...values);
  const max = Math.max(...values);
  const range = Math.max(max - min, 1);
  const xStep = VIEWBOX_WIDTH / (values.length - 1);

  return values
    .map((value, index) => {
      const x = index * xStep;
      const y = VIEWBOX_HEIGHT - ((value - min) / range) * VIEWBOX_HEIGHT;
      return `${x.toFixed(2)},${y.toFixed(2)}`;
    })
    .join(" ");
}

export const MetricsTrendMiniView = memo(function MetricsTrendMiniView({
  samples,
  state = "idle",
  title = "Recent Trend",
  maxPoints = 24
}: MetricsTrendMiniViewProps) {
  const trimmed = useMemo(() => samples.slice(-Math.max(maxPoints, 2)), [samples, maxPoints]);
  const values = useMemo(
    () =>
      trimmed
        .map((sample) => clampTrendSample(sample.routedPingMs))
        .filter((value): value is number => value !== null),
    [trimmed]
  );
  const polylinePoints = useMemo(() => buildPolylinePoints(values), [values]);

  const latestSample = trimmed.length > 0 ? trimmed[trimmed.length - 1] : null;
  const latestLabel = latestSample
    ? `Latest ${formatMs(clampTrendSample(latestSample.routedPingMs))}`
    : "No trend data yet";

  return (
    <section
      className={`kp-trend-mini kp-trend-mini--${state}`}
      aria-label={title}
      data-state={state}
      data-testid="metrics-trend-mini"
    >
      <header className="kp-trend-mini-header">
        <p className="kp-trend-mini-title">{title}</p>
        <p className="kp-trend-mini-meta">{latestLabel}</p>
      </header>

      {values.length >= 2 ? (
        <svg
          className="kp-trend-mini-chart"
          viewBox={`0 0 ${VIEWBOX_WIDTH} ${VIEWBOX_HEIGHT}`}
          preserveAspectRatio="none"
          role="img"
          aria-label="Routed ping trend sparkline"
          data-testid="metrics-trend-svg"
        >
          <polyline
            className="kp-trend-mini-line"
            points={polylinePoints}
            fill="none"
            strokeWidth="1.5"
            strokeLinecap="round"
            strokeLinejoin="round"
          />
        </svg>
      ) : (
        <p className="kp-trend-mini-empty" data-testid="metrics-trend-empty">
          {trimmed.length === 0 ? "No trend data yet" : "Collecting more samples..."}
        </p>
      )}
    </section>
  );
});
