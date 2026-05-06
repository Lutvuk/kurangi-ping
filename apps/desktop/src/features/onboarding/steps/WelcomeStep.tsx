import { Button } from "../../../components/primitives";
import type { OnboardingStepState } from "../../../components/modules";

export type WelcomeStepProps = {
  state: OnboardingStepState;
  onPrimaryAction?: () => void;
  isBusy?: boolean;
};

export function WelcomeStep({ state, onPrimaryAction, isBusy = false }: WelcomeStepProps) {
  const isActive = state === "active";
  const isCompleted = state === "completed";
  const summary = isCompleted
    ? "Welcome selesai. Setup berjalan sesuai urutan."
    : "Lima langkah singkat untuk siap bermain tanpa akun dan tanpa form panjang.";

  return (
    <section className="kp-onboarding-step-screen" aria-label="Welcome onboarding step">
      <h3 className="kp-onboarding-step-screen-title">Selamat datang di Kurangi Ping</h3>
      <p className="kp-onboarding-step-screen-copy">{summary}</p>

      <div className="kp-onboarding-step-screen-actions">
        <Button
          intent="primary"
          ariaLabel="Mulai onboarding"
          onClick={onPrimaryAction}
          disabled={!isActive || isBusy}
        >
          {isBusy ? "Memulai..." : "Mulai Setup"}
        </Button>
      </div>
    </section>
  );
}

