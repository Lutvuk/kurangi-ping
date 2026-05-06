import { useEffect, useState } from "react";
import { Button, Card, Modal, StatusBadge } from "../../components/primitives";
import type { UpdaterUiState } from "./UpdaterPanel";
import "./RestartPrompt.css";

export type RestartPromptModel = {
  updaterState: UpdaterUiState;
  currentVersion: string;
  targetVersion?: string;
  relaunchConfirmationVersion?: string;
};

export type RestartPromptProps = {
  model: RestartPromptModel;
  onRestartNow?: () => Promise<void> | void;
  onDeferRestart?: () => Promise<void> | void;
};

type PromptAction = "restart" | "defer" | null;

function normalizeVersion(raw?: string): string {
  if (!raw) {
    return "unknown";
  }
  const normalized = raw.trim();
  return normalized.length > 0 ? normalized : "unknown";
}

export function RestartPrompt({ model, onRestartNow, onDeferRestart }: RestartPromptProps) {
  const [isDeferred, setIsDeferred] = useState(false);
  const [pendingAction, setPendingAction] = useState<PromptAction>(null);

  useEffect(() => {
    if (model.updaterState !== "ready_to_restart") {
      setIsDeferred(false);
    }
  }, [model.updaterState]);

  const shouldShowPrompt = model.updaterState === "ready_to_restart" && !isDeferred;

  async function handleDefer() {
    if (pendingAction !== null) {
      return;
    }
    setPendingAction("defer");
    try {
      await onDeferRestart?.();
      setIsDeferred(true);
    } finally {
      setPendingAction(null);
    }
  }

  async function handleRestartNow() {
    if (!onRestartNow || pendingAction !== null) {
      return;
    }
    setPendingAction("restart");
    try {
      await onRestartNow();
    } finally {
      setPendingAction(null);
    }
  }

  return (
    <>
      {model.relaunchConfirmationVersion ? (
        <Card as="section" className="kp-restart-prompt-confirmation" aria-label="Post-update confirmation">
          <header className="kp-restart-prompt-confirmation-header">
            <h3 className="kp-restart-prompt-confirmation-title">Update Applied</h3>
            <StatusBadge state="on" label="Up To Date" />
          </header>
          <p className="kp-restart-prompt-confirmation-copy">
            Aplikasi berhasil relaunch di versi{" "}
            <code>{normalizeVersion(model.relaunchConfirmationVersion)}</code>.
          </p>
        </Card>
      ) : null}

      {model.updaterState === "ready_to_restart" && isDeferred ? (
        <Card as="section" className="kp-restart-prompt-deferred" aria-label="Restart deferred notice">
          <header className="kp-restart-prompt-deferred-header">
            <h3 className="kp-restart-prompt-deferred-title">Restart Deferred</h3>
            <StatusBadge state="degraded" label="Restart Pending" />
          </header>
          <p className="kp-restart-prompt-deferred-copy">
            Update versi <code>{normalizeVersion(model.targetVersion)}</code> siap diterapkan kapan saja.
          </p>
          <div className="kp-restart-prompt-deferred-actions">
            <Button
              intent="secondary"
              ariaLabel="Open restart prompt again"
              onClick={() => setIsDeferred(false)}
              disabled={pendingAction !== null}
            >
              Show Restart Prompt
            </Button>
          </div>
        </Card>
      ) : null}

      <Modal open={shouldShowPrompt} title="Restart Required" onClose={() => void handleDefer()}>
        <div className="kp-restart-prompt-modal">
          <p className="kp-restart-prompt-copy">
            Update siap diterapkan dari v<code>{normalizeVersion(model.currentVersion)}</code> ke v
            <code>{normalizeVersion(model.targetVersion)}</code>.
          </p>
          <p className="kp-restart-prompt-copy">
            Restart sekarang untuk mengaktifkan update, atau pilih nanti kalau kamu masih bermain.
          </p>

          <div className="kp-restart-prompt-actions">
            <Button
              intent="primary"
              ariaLabel="Restart app now"
              onClick={() => {
                void handleRestartNow();
              }}
              disabled={!onRestartNow || pendingAction !== null}
              aria-busy={pendingAction === "restart"}
            >
              {pendingAction === "restart" ? "Restarting..." : "Restart Now"}
            </Button>
            <Button
              intent="secondary"
              ariaLabel="Defer restart until later"
              onClick={() => {
                void handleDefer();
              }}
              disabled={pendingAction !== null}
              aria-busy={pendingAction === "defer"}
            >
              {pendingAction === "defer" ? "Saving..." : "Later"}
            </Button>
          </div>
        </div>
      </Modal>
    </>
  );
}
