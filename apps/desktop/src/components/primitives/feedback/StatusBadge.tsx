import type { HTMLAttributes } from "react";
import type { PrimitiveStatusState } from "../shared/types";

export type StatusBadgeProps = HTMLAttributes<HTMLSpanElement> & {
  state: PrimitiveStatusState;
  label?: string;
};

const defaultLabelByState: Record<PrimitiveStatusState, string> = {
  off: "Off",
  connecting: "Connecting",
  on: "Routing Active",
  degraded: "Degraded",
  error: "Error"
};

const classByState: Record<PrimitiveStatusState, string> = {
  off: "kp-status-badge--off",
  connecting: "kp-status-badge--connecting",
  on: "kp-status-badge--on",
  degraded: "kp-status-badge--degraded",
  error: "kp-status-badge--error"
};

export function StatusBadge({ state, label, className, ...props }: StatusBadgeProps) {
  const composedClassName = ["kp-status-badge", classByState[state], className]
    .filter(Boolean)
    .join(" ");
  const text = label ?? defaultLabelByState[state];

  return (
    <span className={composedClassName} role="status" aria-live="polite" {...props}>
      <span className="kp-status-badge-dot" aria-hidden="true" />
      <span>{text}</span>
    </span>
  );
}
