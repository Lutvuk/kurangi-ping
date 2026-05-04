import type { HTMLAttributes } from "react";

export type PingMetricCardState = "off" | "connecting" | "on" | "degraded";

export type PingMetricCardProps = HTMLAttributes<HTMLElement> & {
  state: PingMetricCardState;
  currentPingMs: number | null;
  baselinePingMs: number | null;
  reductionPct: number | null;
};

function formatNumber(value: number | null): string {
  if (value === null || Number.isNaN(value)) {
    return "--";
  }
  return Number.isInteger(value) ? String(value) : value.toFixed(1);
}

export function PingMetricCard({
  state,
  currentPingMs,
  baselinePingMs,
  reductionPct,
  className,
  ...props
}: PingMetricCardProps) {
  const composedClassName = ["kp-ping-card", `kp-ping-card--${state}`, className]
    .filter(Boolean)
    .join(" ");

  return (
    <section className={composedClassName} aria-label="Ping metrics" {...props}>
      <div className="kp-ping-card-col">
        <p className="kp-ping-card-label">Current</p>
        <p className="kp-ping-card-value kp-ping-card-value--current">
          <span>{formatNumber(currentPingMs)}</span>
          <span className="kp-ping-card-unit">ms</span>
        </p>
      </div>
      <div className="kp-ping-card-col">
        <p className="kp-ping-card-label">Baseline</p>
        <p className="kp-ping-card-value kp-ping-card-value--baseline">
          <span>{formatNumber(baselinePingMs)}</span>
          <span className="kp-ping-card-unit">ms</span>
        </p>
      </div>
      <div className="kp-ping-card-col">
        <p className="kp-ping-card-label">Reduction</p>
        <p className="kp-ping-card-value kp-ping-card-value--reduction">
          <span>{formatNumber(reductionPct)}</span>
          <span className="kp-ping-card-unit">%</span>
        </p>
      </div>
    </section>
  );
}
