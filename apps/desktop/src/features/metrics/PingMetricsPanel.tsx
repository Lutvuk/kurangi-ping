import { useEffect, useMemo, useState } from "react";
import { PingMetricCard, type PingMetricCardState } from "../../components/modules";
import { MetricsStatePresenter } from "./MetricsStatePresenter";
import "./PingMetricsPanel.css";

export type MetricsPanelState = "idle" | "measuring" | "live" | "degraded" | "error";

export type MetricsViewModel = {
  state: MetricsPanelState;
  baselinePingMs: number | null;
  routedPingMs: number | null;
  jitterMs: number | null;
  packetLossPct: number | null;
  sampledAtUnixMs?: number;
  reasonCode?: string;
};

export type PingMetricsPanelProps = {
  model: MetricsViewModel;
  title?: string;
};

type StabilizedMetricsView = {
  state: MetricsPanelState;
  baselinePingMs: number | null;
  routedPingMs: number | null;
  jitterMs: number | null;
  packetLossPct: number | null;
  sampledAtUnixMs?: number;
  reasonCode?: string;
};

const statusLabelByState: Record<MetricsPanelState, string> = {
  idle: "Idle",
  measuring: "Measuring",
  live: "Live",
  degraded: "Degraded",
  error: "Error"
};

const cardStateByMetricsState: Record<MetricsPanelState, PingMetricCardState> = {
  idle: "off",
  measuring: "connecting",
  live: "on",
  degraded: "degraded",
  error: "degraded"
};

function stabilizeNumber(nextValue: number | null, previousValue: number | null): number | null {
  if (nextValue === null || Number.isNaN(nextValue)) {
    return previousValue;
  }
  return nextValue;
}

function formatNumber(value: number | null, digits = 1, unit = ""): string {
  if (value === null || Number.isNaN(value)) {
    return "--";
  }

  const formatter = new Intl.NumberFormat("en-US", {
    minimumFractionDigits: 0,
    maximumFractionDigits: digits
  });
  return `${formatter.format(value)}${unit}`;
}

function computeReductionPct(baselinePingMs: number | null, routedPingMs: number | null): number | null {
  if (
    baselinePingMs === null ||
    routedPingMs === null ||
    baselinePingMs <= 0 ||
    Number.isNaN(baselinePingMs) ||
    Number.isNaN(routedPingMs)
  ) {
    return null;
  }

  return ((baselinePingMs - routedPingMs) / baselinePingMs) * 100;
}

function toInitialView(model: MetricsViewModel): StabilizedMetricsView {
  return {
    ...model
  };
}

export function PingMetricsPanel({ model, title = "Ping Metrics" }: PingMetricsPanelProps) {
  const [view, setView] = useState<StabilizedMetricsView>(() => toInitialView(model));

  useEffect(() => {
    setView((previous) => {
      const next: StabilizedMetricsView = {
        state: model.state,
        baselinePingMs: stabilizeNumber(model.baselinePingMs, previous.baselinePingMs),
        routedPingMs: stabilizeNumber(model.routedPingMs, previous.routedPingMs),
        jitterMs: stabilizeNumber(model.jitterMs, previous.jitterMs),
        packetLossPct: stabilizeNumber(model.packetLossPct, previous.packetLossPct),
        sampledAtUnixMs: model.sampledAtUnixMs ?? previous.sampledAtUnixMs,
        reasonCode: model.reasonCode
      };

      const unchanged =
        previous.state === next.state &&
        previous.baselinePingMs === next.baselinePingMs &&
        previous.routedPingMs === next.routedPingMs &&
        previous.jitterMs === next.jitterMs &&
        previous.packetLossPct === next.packetLossPct &&
        previous.sampledAtUnixMs === next.sampledAtUnixMs &&
        previous.reasonCode === next.reasonCode;

      return unchanged ? previous : next;
    });
  }, [model]);

  const reductionPct = useMemo(
    () => computeReductionPct(view.baselinePingMs, view.routedPingMs),
    [view.baselinePingMs, view.routedPingMs]
  );
  const updatedLabel = view.sampledAtUnixMs
    ? `Updated ${new Date(view.sampledAtUnixMs).toLocaleTimeString("en-US", {
        hour12: false
      })}`
    : "No samples yet";

  return (
    <section
      className={`kp-metrics-panel kp-metrics-panel--${view.state}`}
      aria-label={title}
      data-state={view.state}
    >
      <header className="kp-metrics-panel-header">
        <h2 className="kp-metrics-panel-title">{title}</h2>
        <p className="kp-metrics-panel-status" data-testid="metrics-panel-status">
          {statusLabelByState[view.state]}
        </p>
      </header>

      <MetricsStatePresenter model={{ state: view.state, reasonCode: view.reasonCode }} />

      <PingMetricCard
        state={cardStateByMetricsState[view.state]}
        currentPingMs={view.routedPingMs}
        baselinePingMs={view.baselinePingMs}
        reductionPct={reductionPct}
      />

      <div className="kp-metrics-panel-meta">
        <p className="kp-metrics-panel-meta-item" data-testid="metrics-jitter">
          Jitter: <span>{formatNumber(view.jitterMs, 1, " ms")}</span>
        </p>
        <p className="kp-metrics-panel-meta-item" data-testid="metrics-packet-loss">
          Packet Loss: <span>{formatNumber(view.packetLossPct, 1, "%")}</span>
        </p>
        <p className="kp-metrics-panel-updated" data-testid="metrics-updated-at">
          {updatedLabel}
        </p>
      </div>

      {view.reasonCode ? (
        <p className="kp-metrics-panel-reason">
          Reason code: <code>{view.reasonCode}</code>
        </p>
      ) : null}
    </section>
  );
}
