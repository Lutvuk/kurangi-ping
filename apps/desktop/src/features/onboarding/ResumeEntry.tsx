import { useState } from "react";
import { Button, Card, StatusBadge } from "../../components/primitives";

export type ResumeEntryProps = {
  stepLabel: string;
  completedCount: number;
  totalCount: number;
  onResume?: () => void;
  onRestart?: () => void;
  isBusy?: boolean;
};

export function ResumeEntry({
  stepLabel,
  completedCount,
  totalCount,
  onResume,
  onRestart,
  isBusy = false
}: ResumeEntryProps) {
  const [confirmRestart, setConfirmRestart] = useState(false);

  return (
    <Card as="section" className="kp-resume-entry" aria-label="Onboarding resume entry">
      <header className="kp-resume-entry-header">
        <h3 className="kp-resume-entry-title">Lanjut dari checkpoint terakhir</h3>
        <StatusBadge state="connecting" label="Resume" />
      </header>

      <p className="kp-resume-entry-copy">
        Kamu sudah menyelesaikan {completedCount}/{totalCount} langkah. Lanjut ke{" "}
        <strong>{stepLabel}</strong>.
      </p>

      {confirmRestart ? (
        <div className="kp-resume-entry-confirm" role="alertdialog" aria-label="Confirm onboarding restart">
          <p className="kp-resume-entry-confirm-copy">
            Mulai ulang setup dari awal? Progress checkpoint saat ini akan diganti.
          </p>
          <div className="kp-resume-entry-actions">
            <Button
              intent="destructive"
              ariaLabel="Confirm restart onboarding"
              onClick={onRestart}
              disabled={!onRestart || isBusy}
            >
              {isBusy ? "Restarting..." : "Ya, Restart"}
            </Button>
            <Button
              intent="secondary"
              ariaLabel="Cancel restart onboarding"
              onClick={() => setConfirmRestart(false)}
              disabled={isBusy}
            >
              Batal
            </Button>
          </div>
        </div>
      ) : (
        <div className="kp-resume-entry-actions">
          <Button
            intent="primary"
            ariaLabel="Resume onboarding from checkpoint"
            onClick={onResume}
            disabled={!onResume || isBusy}
          >
            {isBusy ? "Melanjutkan..." : "Lanjutkan"}
          </Button>
          <Button
            intent="secondary"
            ariaLabel="Restart onboarding setup"
            onClick={() => setConfirmRestart(true)}
            disabled={!onRestart || isBusy}
          >
            Restart Setup
          </Button>
        </div>
      )}
    </Card>
  );
}

