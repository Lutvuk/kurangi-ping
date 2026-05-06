import { OnboardingStepper } from "../../components/modules";
import { Card, StatusBadge } from "../../components/primitives";
import {
  current_onboarding_step_id,
  onboarding_reason_code,
  to_onboarding_flow_steps,
  type OnboardingStateMachineView,
  type OnboardingStepId
} from "./model";
import {
  FirstConnectStep,
  GameDetectionTestStep,
  PermissionCheckStep,
  RelayTestStep,
  WelcomeStep
} from "./steps";
import "./OnboardingFlow.css";

export type OnboardingFlowSignals = {
  permissionGranted?: boolean;
  relayReady?: boolean;
  gameDetected?: boolean;
  connected?: boolean;
};

export type OnboardingFlowActions = {
  onStart?: () => void;
  onRunPermissionCheck?: () => void;
  onRunRelayTest?: () => void;
  onRunDetectionTest?: () => void;
  onRunFirstConnect?: () => void;
};

export type OnboardingFlowProps = {
  machine: OnboardingStateMachineView;
  signals?: OnboardingFlowSignals;
  actions?: OnboardingFlowActions;
  isBusy?: boolean;
  className?: string;
};

function flow_status(machine: OnboardingStateMachineView): {
  badgeState: "off" | "connecting" | "on" | "error";
  badgeLabel: string;
} {
  if (machine.state.state === "completed") {
    return { badgeState: "on", badgeLabel: "Ready" };
  }

  if (machine.state.state === "blocked" || machine.state.state === "failed") {
    return { badgeState: "error", badgeLabel: "Action Needed" };
  }

  if (machine.state.state === "in_progress") {
    return { badgeState: "connecting", badgeLabel: "In Progress" };
  }

  return { badgeState: "off", badgeLabel: "Not Started" };
}

function render_step_screen(
  stepId: OnboardingStepId,
  stepState: "inactive" | "active" | "completed",
  signals: OnboardingFlowSignals,
  actions: OnboardingFlowActions,
  isBusy: boolean
) {
  if (stepId === "welcome") {
    return <WelcomeStep state={stepState} onPrimaryAction={actions.onStart} isBusy={isBusy} />;
  }

  if (stepId === "permission_check") {
    return (
      <PermissionCheckStep
        state={stepState}
        permissionGranted={signals.permissionGranted}
        onPrimaryAction={actions.onRunPermissionCheck}
        isBusy={isBusy}
      />
    );
  }

  if (stepId === "relay_test") {
    return (
      <RelayTestStep
        state={stepState}
        relayReady={signals.relayReady}
        onPrimaryAction={actions.onRunRelayTest}
        isBusy={isBusy}
      />
    );
  }

  if (stepId === "game_detection_test") {
    return (
      <GameDetectionTestStep
        state={stepState}
        gameDetected={signals.gameDetected}
        onPrimaryAction={actions.onRunDetectionTest}
        isBusy={isBusy}
      />
    );
  }

  return (
    <FirstConnectStep
      state={stepState}
      connected={signals.connected}
      onPrimaryAction={actions.onRunFirstConnect}
      isBusy={isBusy}
    />
  );
}

export function OnboardingFlow({
  machine,
  signals = {},
  actions = {},
  isBusy = false,
  className
}: OnboardingFlowProps) {
  const steps = to_onboarding_flow_steps(machine);
  const focusedStepId = current_onboarding_step_id(machine);
  const focusedStep = steps.find((step) => step.id === focusedStepId) ?? steps[0];
  const status = flow_status(machine);
  const reasonCode = onboarding_reason_code(machine);
  const composedClassName = ["kp-onboarding-flow", className].filter(Boolean).join(" ");

  return (
    <Card as="section" className={composedClassName} aria-label="Onboarding flow">
      <header className="kp-onboarding-flow-header">
        <div className="kp-onboarding-flow-title-wrap">
          <h2 className="kp-onboarding-flow-title">First Run Setup</h2>
          <p className="kp-onboarding-flow-copy">Lima langkah cepat sampai koneksi pertama aktif.</p>
        </div>
        <StatusBadge state={status.badgeState} label={status.badgeLabel} />
      </header>

      <OnboardingStepper
        steps={steps.map((step) => ({
          id: step.id,
          label: step.label,
          state: step.state
        }))}
      />

      {render_step_screen(focusedStep.id, focusedStep.state, signals, actions, isBusy)}

      {reasonCode ? (
        <p className="kp-onboarding-flow-reason">
          Kendala saat ini: <code>{reasonCode}</code>
        </p>
      ) : null}
    </Card>
  );
}

