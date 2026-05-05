import { useMemo, useState } from "react";
import {
  ConnectionStatusBadge,
  type ConnectionStatusState,
  PrimaryToggle,
  type PrimaryToggleState
} from "../../components/modules";
import "./ToggleController.css";

type ToggleLifecycleCommand = "on" | "off";

export type ToggleControllerCommandResult = {
  ok: boolean;
  nextState?: ConnectionStatusState;
  reasonCode?: string;
  message?: string;
};

export type ToggleControllerProps = {
  initialState?: ConnectionStatusState;
  onEnableRouting?: () => Promise<ToggleControllerCommandResult> | ToggleControllerCommandResult;
  onDisableRouting?: () => Promise<ToggleControllerCommandResult> | ToggleControllerCommandResult;
};

type FeedbackState = {
  tone: "info" | "error";
  message: string;
} | null;

function normalizeInitialState(state: ConnectionStatusState): ConnectionStatusState {
  if (state === "error") {
    return "off";
  }
  return state;
}

function toToggleState(
  statusState: ConnectionStatusState,
  pendingCommand: ToggleLifecycleCommand | null
): PrimaryToggleState {
  if (pendingCommand !== null) {
    return "connecting";
  }

  if (statusState === "on") {
    return "on";
  }
  if (statusState === "degraded") {
    return "degraded";
  }
  if (statusState === "connecting") {
    return "connecting";
  }
  return "off";
}

function isDuplicateIntent(command: ToggleLifecycleCommand, state: ConnectionStatusState): boolean {
  if (command === "on") {
    return state === "on" || state === "connecting" || state === "degraded";
  }
  return state === "off" || state === "error";
}

function defaultFailureMessage(command: ToggleLifecycleCommand): string {
  if (command === "on") {
    return "Belum bisa mengaktifkan routing. Coba lagi.";
  }
  return "Belum bisa menonaktifkan routing. Coba lagi.";
}

export function ToggleController({
  initialState = "off",
  onEnableRouting,
  onDisableRouting
}: ToggleControllerProps) {
  const [statusState, setStatusState] = useState<ConnectionStatusState>(() =>
    normalizeInitialState(initialState)
  );
  const [pendingCommand, setPendingCommand] = useState<ToggleLifecycleCommand | null>(null);
  const [feedback, setFeedback] = useState<FeedbackState>(null);

  const toggleState = toToggleState(statusState, pendingCommand);
  const badgeLabel = useMemo(() => {
    if (pendingCommand === "on") {
      return "Connecting...";
    }
    if (pendingCommand === "off") {
      return "Disconnecting...";
    }
    return undefined;
  }, [pendingCommand]);

  async function handleCommand(command: ToggleLifecycleCommand) {
    if (pendingCommand !== null) {
      setFeedback({
        tone: "info",
        message: "Tunggu sebentar, perintah sebelumnya masih diproses."
      });
      return;
    }

    if (isDuplicateIntent(command, statusState)) {
      setFeedback({
        tone: "info",
        message:
          command === "on" ? "Routing sudah aktif." : "Routing sudah nonaktif."
      });
      return;
    }

    setFeedback(null);
    setPendingCommand(command);
    setStatusState("connecting");

    try {
      const result = await (command === "on" ? onEnableRouting?.() : onDisableRouting?.());
      const resolvedResult: ToggleControllerCommandResult = result ?? {
        ok: true,
        nextState: command === "on" ? "on" : "off"
      };

      if (resolvedResult.ok) {
        setStatusState(resolvedResult.nextState ?? (command === "on" ? "on" : "off"));
        return;
      }

      setStatusState("error");
      setFeedback({
        tone: "error",
        message: resolvedResult.message ?? defaultFailureMessage(command)
      });
    } catch {
      setStatusState("error");
      setFeedback({
        tone: "error",
        message: defaultFailureMessage(command)
      });
    } finally {
      setPendingCommand(null);
    }
  }

  return (
    <div className="kp-toggle-controller">
      <div className="kp-toggle-controller-row">
        <PrimaryToggle
          state={toggleState}
          disabled={pendingCommand !== null}
          aria-busy={pendingCommand !== null}
          onToggle={(nextEnabled) => {
            void handleCommand(nextEnabled ? "on" : "off");
          }}
        />
        <ConnectionStatusBadge
          state={statusState}
          label={badgeLabel}
          data-testid="toggle-controller-status"
        />
      </div>

      {feedback ? (
        <p
          className={`kp-toggle-controller-feedback kp-toggle-controller-feedback--${feedback.tone}`}
          role={feedback.tone === "error" ? "alert" : "status"}
          aria-live={feedback.tone === "error" ? "assertive" : "polite"}
        >
          {feedback.message}
        </p>
      ) : null}
    </div>
  );
}
