import type { HTMLAttributes } from "react";

export type OnboardingStepState = "inactive" | "active" | "completed";

export type OnboardingStep = {
  id: string;
  label: string;
  state: OnboardingStepState;
};

export type OnboardingStepperProps = Omit<HTMLAttributes<HTMLOListElement>, "onSelect"> & {
  steps: OnboardingStep[];
  onStepSelect?: (stepId: string) => void;
};

function findNextStepId(steps: OnboardingStep[], currentIndex: number, direction: 1 | -1): string | null {
  const nextIndex = currentIndex + direction;
  if (nextIndex < 0 || nextIndex >= steps.length) {
    return null;
  }
  return steps[nextIndex]?.id ?? null;
}

export function OnboardingStepper({
  steps,
  onStepSelect,
  className,
  ...props
}: OnboardingStepperProps) {
  const composedClassName = ["kp-onboarding-stepper", className].filter(Boolean).join(" ");

  return (
    <ol className={composedClassName} aria-label="Onboarding progress" {...props}>
      {steps.map((step, index) => {
        const stepClassName = ["kp-onboarding-step", `kp-onboarding-step--${step.state}`].join(" ");
        const stepNumber = String(index + 1);
        const indicator = step.state === "completed" ? "✓" : stepNumber;
        const isInteractive = Boolean(onStepSelect);

        return (
          <li className={stepClassName} key={step.id}>
            {isInteractive ? (
              <button
                type="button"
                className="kp-onboarding-step-button"
                aria-current={step.state === "active" ? "step" : undefined}
                onClick={() => onStepSelect?.(step.id)}
                onKeyDown={(event) => {
                  if (event.key === "ArrowRight") {
                    const nextStepId = findNextStepId(steps, index, 1);
                    if (nextStepId) {
                      event.preventDefault();
                      onStepSelect?.(nextStepId);
                    }
                  }
                  if (event.key === "ArrowLeft") {
                    const previousStepId = findNextStepId(steps, index, -1);
                    if (previousStepId) {
                      event.preventDefault();
                      onStepSelect?.(previousStepId);
                    }
                  }
                }}
              >
                <span className="kp-onboarding-step-indicator" aria-hidden="true">
                  {indicator}
                </span>
                <span className="kp-onboarding-step-label">{step.label}</span>
              </button>
            ) : (
              <span className="kp-onboarding-step-static" aria-current={step.state === "active" ? "step" : undefined}>
                <span className="kp-onboarding-step-indicator" aria-hidden="true">
                  {indicator}
                </span>
                <span className="kp-onboarding-step-label">{step.label}</span>
              </span>
            )}
          </li>
        );
      })}
    </ol>
  );
}

