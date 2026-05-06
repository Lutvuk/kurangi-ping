import { Button } from "../../../components/primitives";
import type { OnboardingStepState } from "../../../components/modules";

export type FirstConnectStepProps = {
  state: OnboardingStepState;
  connected?: boolean;
  onPrimaryAction?: () => void;
  isBusy?: boolean;
};

function first_connect_message(connected: boolean | undefined, stepState: OnboardingStepState): string {
  if (stepState === "completed" || connected === true) {
    return "Koneksi pertama berhasil. Kamu siap lanjut main.";
  }
  if (connected === false) {
    return "Belum tersambung. Coba connect lagi, biasanya cepat.";
  }
  return "Aktifkan routing sekali untuk lihat ping pertama kamu.";
}

export function FirstConnectStep({
  state,
  connected,
  onPrimaryAction,
  isBusy = false
}: FirstConnectStepProps) {
  const isActive = state === "active";

  return (
    <section className="kp-onboarding-step-screen" aria-label="First connect onboarding step">
      <h3 className="kp-onboarding-step-screen-title">Koneksi pertama</h3>
      <p className="kp-onboarding-step-screen-copy">{first_connect_message(connected, state)}</p>

      <div className="kp-onboarding-step-screen-actions">
        <Button
          intent="primary"
          ariaLabel="Aktifkan koneksi pertama"
          onClick={onPrimaryAction}
          disabled={!isActive || isBusy}
        >
          {isBusy ? "Connecting..." : "Connect Sekarang"}
        </Button>
      </div>
    </section>
  );
}

