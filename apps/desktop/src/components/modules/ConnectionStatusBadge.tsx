import type { HTMLAttributes } from "react";
import type { PrimitiveStatusState } from "../primitives/shared/types";

export type ConnectionStatusState = Extract<
  PrimitiveStatusState,
  "off" | "connecting" | "on" | "degraded"
>;

export type ConnectionStatusBadgeProps = HTMLAttributes<HTMLSpanElement> & {
  state: ConnectionStatusState;
  label?: string;
};

const labelByState: Record<ConnectionStatusState, string> = {
  off: "Offline",
  connecting: "Connecting...",
  on: "Routing Active",
  degraded: "Routing Degraded"
};

export function ConnectionStatusBadge({
  state,
  label,
  className,
  ...props
}: ConnectionStatusBadgeProps) {
  const composedClassName = ["kp-connection-badge", `kp-connection-badge--${state}`, className]
    .filter(Boolean)
    .join(" ");

  return (
    <span className={composedClassName} role="status" aria-live="polite" {...props}>
      <span className="kp-connection-badge-dot" aria-hidden="true" />
      <span className="kp-connection-badge-label">{label ?? labelByState[state]}</span>
    </span>
  );
}
