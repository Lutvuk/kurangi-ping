import { Button } from "../../../components/primitives";
import type { OnboardingStepState } from "../../../components/modules";

export type GameDetectionTestStepProps = {
  state: OnboardingStepState;
  gameDetected?: boolean;
  onPrimaryAction?: () => void;
  isBusy?: boolean;
};

function detection_message(gameDetected: boolean | undefined, stepState: OnboardingStepState): string {
  if (stepState === "completed" || gameDetected === true) {
    return "Game sudah terdeteksi. Kita siap ke koneksi pertama.";
  }
  if (gameDetected === false) {
    return "Game belum terdeteksi. Buka game dulu lalu scan ulang.";
  }
  return "Buka game kamu, lalu jalankan scan agar routing tepat sasaran.";
}

export function GameDetectionTestStep({
  state,
  gameDetected,
  onPrimaryAction,
  isBusy = false
}: GameDetectionTestStepProps) {
  const isActive = state === "active";

  return (
    <section className="kp-onboarding-step-screen" aria-label="Game detection onboarding step">
      <h3 className="kp-onboarding-step-screen-title">Deteksi game</h3>
      <p className="kp-onboarding-step-screen-copy">{detection_message(gameDetected, state)}</p>

      <div className="kp-onboarding-step-screen-actions">
        <Button
          intent="secondary"
          ariaLabel="Scan game sekarang"
          onClick={onPrimaryAction}
          disabled={!isActive || isBusy}
        >
          {isBusy ? "Scanning..." : "Scan Game"}
        </Button>
      </div>
    </section>
  );
}

