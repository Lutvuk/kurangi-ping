import type { ButtonHTMLAttributes } from "react";
import type { PrimitiveStatusState } from "../primitives/shared/types";

export type PrimaryToggleState = Extract<
  PrimitiveStatusState,
  "off" | "connecting" | "on" | "degraded"
>;

export type PrimaryToggleProps = Omit<ButtonHTMLAttributes<HTMLButtonElement>, "onToggle"> & {
  state: PrimaryToggleState;
  onToggle?: (nextEnabled: boolean) => void;
};

const labelByState: Record<PrimaryToggleState, string> = {
  off: "OFF",
  connecting: "CONNECTING",
  on: "ON",
  degraded: "DEGRADED"
};

export function PrimaryToggle({ state, onToggle, className, onClick, ...props }: PrimaryToggleProps) {
  const isEnabled = state !== "off";
  const composedClassName = ["kp-primary-toggle", `kp-primary-toggle--${state}`, className]
    .filter(Boolean)
    .join(" ");

  return (
    <button
      type="button"
      className={composedClassName}
      data-state={state}
      aria-pressed={isEnabled}
      aria-label={`Routing toggle ${state}`}
      onClick={(event) => {
        onClick?.(event);
        if (!event.defaultPrevented) {
          onToggle?.(!isEnabled);
        }
      }}
      {...props}
    >
      <span className="kp-primary-toggle-bracket kp-primary-toggle-bracket--tl" aria-hidden="true" />
      <span className="kp-primary-toggle-bracket kp-primary-toggle-bracket--tr" aria-hidden="true" />
      <span className="kp-primary-toggle-bracket kp-primary-toggle-bracket--bl" aria-hidden="true" />
      <span className="kp-primary-toggle-bracket kp-primary-toggle-bracket--br" aria-hidden="true" />
      <span className="kp-primary-toggle-label">{labelByState[state]}</span>
    </button>
  );
}
