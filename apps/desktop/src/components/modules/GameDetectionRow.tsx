import type { HTMLAttributes, ReactNode } from "react";

export type GameDetectionState = "detected" | "not-detected";

export type GameDetectionRowProps = HTMLAttributes<HTMLDivElement> & {
  gameName: string;
  serverInfo: string;
  state: GameDetectionState;
  icon?: ReactNode;
  statusLabel?: string;
};

const defaultLabelByState: Record<GameDetectionState, string> = {
  detected: "Detected",
  "not-detected": "Not Detected"
};

export function GameDetectionRow({
  gameName,
  serverInfo,
  state,
  icon,
  statusLabel,
  className,
  ...props
}: GameDetectionRowProps) {
  const composedClassName = ["kp-game-row", `kp-game-row--${state}`, className].filter(Boolean).join(" ");

  return (
    <div className={composedClassName} {...props}>
      <span className="kp-game-row-icon" aria-hidden="true">
        {icon ?? gameName.slice(0, 1).toUpperCase()}
      </span>
      <span className="kp-game-row-meta">
        <span className="kp-game-row-name">{gameName}</span>
        <span className="kp-game-row-server">{serverInfo}</span>
      </span>
      <span className="kp-game-row-status">{statusLabel ?? defaultLabelByState[state]}</span>
    </div>
  );
}

