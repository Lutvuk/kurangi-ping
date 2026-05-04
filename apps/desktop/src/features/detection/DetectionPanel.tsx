import { useEffect, useMemo, useState } from "react";
import { GameDetectionRow } from "../../components/modules";
import { Button, Card, StatusBadge } from "../../components/primitives";
import "./DetectionPanel.css";

export type DetectionUiState = "not_found" | "detected" | "stale" | "error";

export type DetectionViewModel = {
  state: DetectionUiState;
  gameId?: string;
  processName?: string;
  detectionTimeMs?: number;
  reasonCode?: string;
  message?: string;
};

export type DetectionPanelProps = {
  model: DetectionViewModel;
  onTriggerRescan?: () => Promise<DetectionViewModel> | DetectionViewModel;
  title?: string;
};

const statusBadgeByState: Record<DetectionUiState, "off" | "on" | "degraded" | "error"> = {
  not_found: "off",
  detected: "on",
  stale: "degraded",
  error: "error"
};

const statusLabelByState: Record<DetectionUiState, string> = {
  not_found: "Not Found",
  detected: "Detected",
  stale: "Stale",
  error: "Error"
};

const messageByState: Record<DetectionUiState, string> = {
  not_found: "Game belum terdeteksi. Jalankan game lalu lakukan scan ulang.",
  detected: "Game terdeteksi dan siap untuk aktivasi routing.",
  stale: "Hasil deteksi sudah stale. Lakukan scan ulang sebelum routing.",
  error: "Scan gagal. Coba scan ulang atau cek izin akses proses."
};

function formatGameName(gameId?: string): string {
  if (!gameId) {
    return "Unknown Game";
  }

  return gameId
    .split("_")
    .map((part) => {
      if (!part) {
        return part;
      }
      return part[0].toUpperCase() + part.slice(1);
    })
    .join(" ");
}

function formatDetectionTime(detectionTimeMs?: number): string {
  if (detectionTimeMs === undefined) {
    return "Latest scan unavailable";
  }

  return `${detectionTimeMs} ms`;
}

export function DetectionPanel({ model, onTriggerRescan, title = "Detection Status" }: DetectionPanelProps) {
  const [viewModel, setViewModel] = useState<DetectionViewModel>(model);
  const [isRescanning, setIsRescanning] = useState(false);

  useEffect(() => {
    setViewModel(model);
  }, [model]);

  const effectiveState: DetectionUiState = isRescanning ? "stale" : viewModel.state;
  const badgeState = isRescanning ? "connecting" : statusBadgeByState[effectiveState];
  const badgeLabel = isRescanning ? "Scanning..." : statusLabelByState[effectiveState];
  const message = viewModel.message ?? messageByState[effectiveState];

  const rowState = effectiveState === "detected" ? "detected" : "not-detected";
  const rowStatusLabel = isRescanning ? "Scanning..." : statusLabelByState[effectiveState];
  const rowServerInfo = useMemo(() => {
    const process = viewModel.processName ?? "process unavailable";
    const timing = formatDetectionTime(viewModel.detectionTimeMs);
    return `${process} · ${timing}`;
  }, [viewModel.processName, viewModel.detectionTimeMs]);

  async function handleRescan() {
    if (!onTriggerRescan || isRescanning) {
      return;
    }

    setIsRescanning(true);
    try {
      const nextModel = await onTriggerRescan();
      setViewModel(nextModel);
    } catch {
      setViewModel((previous) => ({
        ...previous,
        state: "error",
        reasonCode: "rescan_failed",
        message: "Rescan gagal dijalankan. Coba ulang beberapa saat lagi."
      }));
    } finally {
      setIsRescanning(false);
    }
  }

  return (
    <Card as="section" className="kp-detection-panel" aria-label="Detection status panel">
      <header className="kp-detection-panel-header">
        <h2 className="kp-detection-panel-title">{title}</h2>
        <StatusBadge className="kp-detection-panel-status" state={badgeState} label={badgeLabel} />
      </header>

      <GameDetectionRow
        gameName={formatGameName(viewModel.gameId)}
        serverInfo={rowServerInfo}
        state={rowState}
        statusLabel={rowStatusLabel}
      />

      <p className={`kp-detection-panel-message kp-detection-panel-message--${badgeState}`}>{message}</p>

      {viewModel.reasonCode ? (
        <p className="kp-detection-panel-copy">
          Reason code: <code>{viewModel.reasonCode}</code>
        </p>
      ) : null}

      <div>
        <Button
          intent="secondary"
          ariaLabel="Rescan detection status"
          onClick={handleRescan}
          disabled={!onTriggerRescan || isRescanning}
          aria-busy={isRescanning}
        >
          {isRescanning ? "Scanning..." : "Scan Ulang"}
        </Button>
      </div>
    </Card>
  );
}
