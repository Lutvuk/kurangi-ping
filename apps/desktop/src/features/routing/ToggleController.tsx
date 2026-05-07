import { useCallback, useEffect, useMemo, useState } from "react";
import {
  ConnectionStatusBadge,
  type ConnectionStatusState,
  PrimaryToggle,
  type PrimaryToggleState
} from "../../components/modules";
import {
  ipcClient as defaultIpcClient,
  type IpcClient,
  type RoutingLifecycleResponse,
  type RoutingStateChangedEventPayload
} from "../../lib/ipc";
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
  enableIpcBridge?: boolean;
  ipcClient?: IpcClient;
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

function mapRoutingStateToStatus(state: RoutingLifecycleResponse["state"]): ConnectionStatusState {
  if (state === "active") {
    return "on";
  }
  if (state === "idle") {
    return "off";
  }
  if (state === "degraded") {
    return "degraded";
  }
  if (state === "connecting") {
    return "connecting";
  }
  return "error";
}

function mapRoutingResponseToResult(response: RoutingLifecycleResponse): ToggleControllerCommandResult {
  const nextState = mapRoutingStateToStatus(response.state);
  const rejected = response.reasonCode !== undefined || response.state === "error";
  return {
    ok: !rejected,
    nextState,
    reasonCode: response.reasonCode,
    message: response.message
  };
}

function mapRoutingEventToStatus(payload: RoutingStateChangedEventPayload): ConnectionStatusState {
  return mapRoutingStateToStatus(payload.state);
}

type RoutingIpcBridgeOptions = {
  enabled: boolean;
  client: IpcClient;
  onRoutingStateChanged: (payload: RoutingStateChangedEventPayload) => void;
};

type RoutingIpcBridge = {
  invokeEnableRouting: () => Promise<ToggleControllerCommandResult>;
  invokeDisableRouting: () => Promise<ToggleControllerCommandResult>;
};

export function useRoutingIpcBridge({
  enabled,
  client,
  onRoutingStateChanged
}: RoutingIpcBridgeOptions): RoutingIpcBridge {
  useEffect(() => {
    if (!enabled) {
      return;
    }

    let unsubscribe: (() => Promise<void>) | undefined;
    void (async () => {
      unsubscribe = await client.subscribeRoutingState(onRoutingStateChanged);
    })();

    return () => {
      if (unsubscribe) {
        void unsubscribe();
      }
    };
  }, [client, enabled, onRoutingStateChanged]);

  async function invokeEnableRouting(): Promise<ToggleControllerCommandResult> {
    try {
      const response = await client.invokeRoutingToggleOn({
        trigger: "user_toggle",
        requestedAtUnixMs: Date.now()
      });
      return mapRoutingResponseToResult(response);
    } catch {
      return {
        ok: false,
        nextState: "error",
        reasonCode: "ipc_unknown_failure",
        message: defaultFailureMessage("on")
      };
    }
  }

  async function invokeDisableRouting(): Promise<ToggleControllerCommandResult> {
    try {
      const response = await client.invokeRoutingToggleOff({
        trigger: "user_toggle",
        requestedAtUnixMs: Date.now()
      });
      return mapRoutingResponseToResult(response);
    } catch {
      return {
        ok: false,
        nextState: "error",
        reasonCode: "ipc_unknown_failure",
        message: defaultFailureMessage("off")
      };
    }
  }

  return {
    invokeEnableRouting,
    invokeDisableRouting
  };
}

export function ToggleController({
  initialState = "off",
  onEnableRouting,
  onDisableRouting,
  enableIpcBridge = false,
  ipcClient = defaultIpcClient
}: ToggleControllerProps) {
  const [statusState, setStatusState] = useState<ConnectionStatusState>(() =>
    normalizeInitialState(initialState)
  );
  const [pendingCommand, setPendingCommand] = useState<ToggleLifecycleCommand | null>(null);
  const [feedback, setFeedback] = useState<FeedbackState>(null);
  const handleRoutingStateChanged = useCallback((payload: RoutingStateChangedEventPayload) => {
    setStatusState(mapRoutingEventToStatus(payload));
  }, []);
  const ipcBridge = useRoutingIpcBridge({
    enabled: enableIpcBridge,
    client: ipcClient,
    onRoutingStateChanged: handleRoutingStateChanged
  });

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
      const result = await (command === "on"
        ? onEnableRouting?.() ?? (enableIpcBridge ? ipcBridge.invokeEnableRouting() : undefined)
        : onDisableRouting?.() ?? (enableIpcBridge ? ipcBridge.invokeDisableRouting() : undefined));
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
