import type { HTMLAttributes } from "react";

export type RelayHealthState = "ok" | "warn" | "dead";

export type RelayHealthListItemProps = HTMLAttributes<HTMLDivElement> & {
  hostname: string;
  latencyMs: number | null;
  region: string;
  active?: boolean;
  health: RelayHealthState;
};

function formatLatency(value: number | null): string {
  if (value === null || Number.isNaN(value)) {
    return "--";
  }
  return Number.isInteger(value) ? String(value) : value.toFixed(1);
}

export function RelayHealthListItem({
  hostname,
  latencyMs,
  region,
  active = false,
  health,
  className,
  ...props
}: RelayHealthListItemProps) {
  const composedClassName = [
    "kp-relay-item",
    `kp-relay-item--${health}`,
    active ? "kp-relay-item--active" : "",
    className
  ]
    .filter(Boolean)
    .join(" ");

  return (
    <div className={composedClassName} role="listitem" {...props}>
      <span className="kp-relay-item-indicator" aria-hidden="true" />
      <span className="kp-relay-item-hostname">{hostname}</span>
      <span className="kp-relay-item-latency">
        {formatLatency(latencyMs)}
        <span className="kp-relay-item-unit">ms</span>
      </span>
      <span className="kp-relay-item-region">{region.toUpperCase()}</span>
    </div>
  );
}
