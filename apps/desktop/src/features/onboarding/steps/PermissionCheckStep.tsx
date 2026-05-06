import { Button } from "../../../components/primitives";
import type { OnboardingStepState } from "../../../components/modules";

export type PermissionCheckStepProps = {
  state: OnboardingStepState;
  permissionGranted?: boolean;
  onPrimaryAction?: () => void;
  isBusy?: boolean;
};

function permission_message(permissionGranted: boolean | undefined, stepState: OnboardingStepState): string {
  if (stepState === "completed" || permissionGranted === true) {
    return "Izin sistem siap. Kita bisa lanjut aman ke test relay.";
  }
  if (permissionGranted === false) {
    return "Aplikasi butuh izin routing. Ikuti petunjuk lalu coba lagi.";
  }
  return "Cek izin Windows untuk memastikan koneksi bisa dialihkan dengan aman.";
}

export function PermissionCheckStep({
  state,
  permissionGranted,
  onPrimaryAction,
  isBusy = false
}: PermissionCheckStepProps) {
  const isActive = state === "active";

  return (
    <section className="kp-onboarding-step-screen" aria-label="Permission onboarding step">
      <h3 className="kp-onboarding-step-screen-title">Cek izin sistem</h3>
      <p className="kp-onboarding-step-screen-copy">{permission_message(permissionGranted, state)}</p>

      <div className="kp-onboarding-step-screen-actions">
        <Button
          intent="secondary"
          ariaLabel="Jalankan cek izin"
          onClick={onPrimaryAction}
          disabled={!isActive || isBusy}
        >
          {isBusy ? "Mengecek..." : "Cek Izin"}
        </Button>
      </div>
    </section>
  );
}

