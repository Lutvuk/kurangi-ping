import type { OnboardingStepState } from "../../components/modules";

export type OnboardingStepId =
  | "welcome"
  | "permission_check"
  | "relay_test"
  | "game_detection_test"
  | "first_connect";

export type OnboardingLifecycleStateView =
  | { state: "not_started" }
  | { state: "in_progress"; current_step: OnboardingStepId; completed_steps: OnboardingStepId[] }
  | { state: "completed"; completed_steps: OnboardingStepId[] }
  | {
      state: "blocked";
      blocked_step: OnboardingStepId;
      completed_steps: OnboardingStepId[];
      reason_code: string;
    }
  | {
      state: "failed";
      failed_step: OnboardingStepId;
      completed_steps: OnboardingStepId[];
      reason_code: string;
    };

export type OnboardingStateMachineView = {
  state: OnboardingLifecycleStateView;
};

export type OnboardingStepMeta = {
  id: OnboardingStepId;
  label: string;
};

export type OnboardingFlowStepView = OnboardingStepMeta & {
  state: OnboardingStepState;
};

export const ONBOARDING_STEP_SEQUENCE: OnboardingStepMeta[] = [
  { id: "welcome", label: "Welcome" },
  { id: "permission_check", label: "Permission" },
  { id: "relay_test", label: "Relay Test" },
  { id: "game_detection_test", label: "Game Detect" },
  { id: "first_connect", label: "First Connect" }
];

const stepLabelById: Record<OnboardingStepId, string> = ONBOARDING_STEP_SEQUENCE.reduce(
  (accumulator, step) => {
    accumulator[step.id] = step.label;
    return accumulator;
  },
  {} as Record<OnboardingStepId, string>
);

function active_step_from_state(lifecycle: OnboardingLifecycleStateView): OnboardingStepId {
  if (lifecycle.state === "in_progress") {
    return lifecycle.current_step;
  }

  if (lifecycle.state === "blocked") {
    return lifecycle.blocked_step;
  }

  if (lifecycle.state === "failed") {
    return lifecycle.failed_step;
  }

  if (lifecycle.state === "completed") {
    return "first_connect";
  }

  return "welcome";
}

function completed_set_from_state(lifecycle: OnboardingLifecycleStateView): Set<OnboardingStepId> {
  if (lifecycle.state === "in_progress" || lifecycle.state === "blocked" || lifecycle.state === "failed") {
    return new Set(lifecycle.completed_steps);
  }

  if (lifecycle.state === "completed") {
    return new Set(ONBOARDING_STEP_SEQUENCE.map((step) => step.id));
  }

  return new Set();
}

export function to_onboarding_flow_steps(machine: OnboardingStateMachineView): OnboardingFlowStepView[] {
  const activeStep = active_step_from_state(machine.state);
  const completedSet = completed_set_from_state(machine.state);

  return ONBOARDING_STEP_SEQUENCE.map((step) => {
    let state: OnboardingStepState = "inactive";
    if (completedSet.has(step.id)) {
      state = "completed";
    } else if (step.id === activeStep) {
      state = "active";
    }

    return {
      ...step,
      state
    };
  });
}

export function current_onboarding_step_id(machine: OnboardingStateMachineView): OnboardingStepId {
  return active_step_from_state(machine.state);
}

export function onboarding_reason_code(machine: OnboardingStateMachineView): string | null {
  if (machine.state.state === "blocked" || machine.state.state === "failed") {
    return machine.state.reason_code;
  }
  return null;
}

export function onboarding_step_label(stepId: OnboardingStepId): string {
  return stepLabelById[stepId];
}
