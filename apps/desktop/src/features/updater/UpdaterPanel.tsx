import { useEffect, useState } from "react";
import { Button, Card, StatusBadge } from "../../components/primitives";
import "./UpdaterPanel.css";

export type UpdaterUiState =
  | "up_to_date"
  | "update_available"
  | "downloading"
  | "ready_to_restart"
  | "update_error";

export type UpdaterChannel = "stable" | "beta";

export type UpdaterViewModel = {
  state: UpdaterUiState;
  channel: UpdaterChannel;
  currentVersion: string;
  targetVersion?: string;
  reasonCode?: string;
  message?: string;
  canApply?: boolean;
  lastCheckedAtLabel?: string;
};

export type UpdaterActionResult = {
  ok: boolean;
  nextModel?: UpdaterViewModel;
  reasonCode?: string;
  message?: string;
};

export type UpdaterPanelProps = {
  model: UpdaterViewModel;
  title?: string;
  onCheckForUpdate?: () => Promise<UpdaterActionResult> | UpdaterActionResult;
  onDownloadUpdate?: () => Promise<UpdaterActionResult> | UpdaterActionResult;
  onApplyUpdate?: () => Promise<UpdaterActionResult> | UpdaterActionResult;
  onRetry?: () => Promise<UpdaterActionResult> | UpdaterActionResult;
};

type UpdaterAction = "check" | "download" | "apply" | "retry";

const statusBadgeByState: Record<UpdaterUiState, "off" | "connecting" | "on" | "degraded" | "error"> = {
  up_to_date: "on",
  update_available: "degraded",
  downloading: "connecting",
  ready_to_restart: "on",
  update_error: "error"
};

const statusLabelByState: Record<UpdaterUiState, string> = {
  up_to_date: "Up To Date",
  update_available: "Update Available",
  downloading: "Downloading...",
  ready_to_restart: "Ready To Restart",
  update_error: "Update Error"
};

const defaultMessageByState: Record<UpdaterUiState, string> = {
  up_to_date: "Kamu sudah di versi terbaru.",
  update_available: "Versi baru tersedia dan siap diunduh.",
  downloading: "Sedang mengunduh paket update. Jangan tutup aplikasi.",
  ready_to_restart: "Update sudah siap. Restart aplikasi untuk menerapkan update.",
  update_error: "Update gagal diproses. Coba ulangi lagi."
};

function normalizeVersion(raw?: string): string {
  if (!raw) {
    return "unknown";
  }
  const normalized = raw.trim();
  return normalized.length > 0 ? normalized : "unknown";
}

function defaultFailureMessage(action: UpdaterAction): string {
  switch (action) {
    case "check":
      return "Gagal cek update. Coba ulang beberapa saat lagi.";
    case "download":
      return "Gagal mengunduh update. Periksa koneksi lalu coba lagi.";
    case "apply":
      return "Gagal menerapkan update. Coba ulangi proses apply.";
    case "retry":
      return "Retry gagal. Coba lagi atau kembali cek update.";
    default:
      return "Update gagal diproses.";
  }
}

export function UpdaterPanel({
  model,
  title = "Updater Status",
  onCheckForUpdate,
  onDownloadUpdate,
  onApplyUpdate,
  onRetry
}: UpdaterPanelProps) {
  const [viewModel, setViewModel] = useState<UpdaterViewModel>(model);
  const [pendingAction, setPendingAction] = useState<UpdaterAction | null>(null);

  useEffect(() => {
    setViewModel(model);
  }, [model]);

  const badgeState = statusBadgeByState[viewModel.state];
  const badgeLabel = statusLabelByState[viewModel.state];
  const message = viewModel.message ?? defaultMessageByState[viewModel.state];

  const canCheck = pendingAction === null && viewModel.state !== "downloading";
  const canDownload =
    pendingAction === null && viewModel.state === "update_available" && !viewModel.canApply;
  const canApply =
    pendingAction === null && viewModel.state === "update_available" && viewModel.canApply === true;
  const canRetry = pendingAction === null && viewModel.state === "update_error";

  async function runAction(
    action: UpdaterAction,
    handler: (() => Promise<UpdaterActionResult> | UpdaterActionResult) | undefined
  ) {
    if (!handler || pendingAction !== null) {
      return;
    }

    setPendingAction(action);
    try {
      const result = await handler();
      const resolved = result ?? { ok: true };
      if (resolved.ok) {
        if (resolved.nextModel) {
          setViewModel(resolved.nextModel);
        }
        return;
      }

      setViewModel((previous) => ({
        ...previous,
        state: "update_error",
        reasonCode: resolved.reasonCode ?? "updater_action_failed",
        message: resolved.message ?? defaultFailureMessage(action)
      }));
    } catch {
      setViewModel((previous) => ({
        ...previous,
        state: "update_error",
        reasonCode: "updater_action_failed",
        message: defaultFailureMessage(action)
      }));
    } finally {
      setPendingAction(null);
    }
  }

  const retryHandler = onRetry ?? onCheckForUpdate;

  return (
    <Card
      as="section"
      className={`kp-updater-panel kp-updater-panel--${viewModel.state}`}
      aria-label="Updater status panel"
    >
      <header className="kp-updater-panel-header">
        <div className="kp-updater-panel-title-wrap">
          <h2 className="kp-updater-panel-title">{title}</h2>
          <p className="kp-updater-panel-meta">
            Channel <code>{viewModel.channel}</code> · v{normalizeVersion(viewModel.currentVersion)}
          </p>
        </div>
        <StatusBadge state={badgeState} label={badgeLabel} />
      </header>

      <p className="kp-updater-panel-message">{message}</p>

      {viewModel.targetVersion ? (
        <p className="kp-updater-panel-copy">
          Target version: <code>{normalizeVersion(viewModel.targetVersion)}</code>
        </p>
      ) : null}

      {viewModel.lastCheckedAtLabel ? (
        <p className="kp-updater-panel-copy">Last checked: {viewModel.lastCheckedAtLabel}</p>
      ) : null}

      {viewModel.reasonCode ? (
        <p className="kp-updater-panel-copy">
          Reason code: <code>{viewModel.reasonCode}</code>
        </p>
      ) : null}

      <div className="kp-updater-panel-actions">
        <Button
          intent="secondary"
          ariaLabel="Check updater status"
          onClick={() => {
            void runAction("check", onCheckForUpdate);
          }}
          disabled={!onCheckForUpdate || !canCheck}
          aria-busy={pendingAction === "check"}
        >
          {pendingAction === "check" ? "Checking..." : "Check Update"}
        </Button>

        {viewModel.state === "update_available" ? (
          <Button
            intent={viewModel.canApply ? "primary" : "secondary"}
            ariaLabel={viewModel.canApply ? "Apply downloaded update" : "Download update package"}
            onClick={() => {
              if (viewModel.canApply) {
                void runAction("apply", onApplyUpdate);
                return;
              }
              void runAction("download", onDownloadUpdate);
            }}
            disabled={
              viewModel.canApply
                ? !onApplyUpdate || !canApply
                : !onDownloadUpdate || !canDownload
            }
            aria-busy={pendingAction === "download" || pendingAction === "apply"}
          >
            {viewModel.canApply
              ? pendingAction === "apply"
                ? "Applying..."
                : "Apply Update"
              : pendingAction === "download"
                ? "Downloading..."
                : "Download Update"}
          </Button>
        ) : null}

        {viewModel.state === "update_error" ? (
          <Button
            intent="destructive"
            ariaLabel="Retry updater action"
            onClick={() => {
              void runAction("retry", retryHandler);
            }}
            disabled={!retryHandler || !canRetry}
            aria-busy={pendingAction === "retry"}
          >
            {pendingAction === "retry" ? "Retrying..." : "Retry"}
          </Button>
        ) : null}
      </div>
    </Card>
  );
}
