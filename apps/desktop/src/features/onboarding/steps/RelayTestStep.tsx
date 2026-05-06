import { Button } from "../../../components/primitives";
import type { OnboardingStepState } from "../../../components/modules";

export type RelayTestStepProps = {
  state: OnboardingStepState;
  relayReady?: boolean;
  onPrimaryAction?: () => void;
  isBusy?: boolean;
};

function relay_message(relayReady: boolean | undefined, stepState: OnboardingStepState): string {
  if (stepState === "completed" || relayReady === true) {
    return "Relay terbaik sudah dipilih. Lanjut ke deteksi game.";
  }
  if (relayReady === false) {
    return "Relay belum stabil. Coba test ulang untuk hasil paling aman.";
  }
  return "Kami tes beberapa relay dan pilih yang paling cepat secara otomatis.";
}

export function RelayTestStep({ state, relayReady, onPrimaryAction, isBusy = false }: RelayTestStepProps) {
  const isActive = state === "active";

  return (
    <section className="kp-onboarding-step-screen" aria-label="Relay test onboarding step">
      <h3 className="kp-onboarding-step-screen-title">Test relay otomatis</h3>
      <p className="kp-onboarding-step-screen-copy">{relay_message(relayReady, state)}</p>

      <div className="kp-onboarding-step-screen-actions">
        <Button
          intent="secondary"
          ariaLabel="Jalankan test relay"
          onClick={onPrimaryAction}
          disabled={!isActive || isBusy}
        >
          {isBusy ? "Mengetes..." : "Test Relay"}
        </Button>
      </div>
    </section>
  );
}

