import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { OnboardingFlow } from "./OnboardingFlow";
import type { OnboardingStateMachineView } from "./model";

function render_flow(machine: OnboardingStateMachineView) {
  return render(<OnboardingFlow machine={machine} />);
}

describe("OnboardingFlow", () => {
  it("renders five steps in canonical onboarding order", () => {
    render_flow({ state: { state: "not_started" } });

    const labels = screen.getAllByText(/Welcome|Permission|Relay Test|Game Detect|First Connect/i);
    expect(labels.map((node) => node.textContent)).toEqual([
      "Welcome",
      "Permission",
      "Relay Test",
      "Game Detect",
      "First Connect"
    ]);
  });

  it("maps backend onboarding state machine into stepper states", () => {
    render_flow({
      state: {
        state: "in_progress",
        current_step: "relay_test",
        completed_steps: ["welcome", "permission_check"]
      }
    });

    const steps = screen.getAllByRole("listitem");
    expect(steps[0]).toHaveClass("kp-onboarding-step--completed");
    expect(steps[1]).toHaveClass("kp-onboarding-step--completed");
    expect(steps[2]).toHaveClass("kp-onboarding-step--active");
    expect(steps[3]).toHaveClass("kp-onboarding-step--inactive");
    expect(steps[4]).toHaveClass("kp-onboarding-step--inactive");
    expect(screen.getByRole("status")).toHaveTextContent("In Progress");
  });

  it("shows concise non-technical step copy for each main screen", () => {
    render_flow({
      state: {
        state: "in_progress",
        current_step: "game_detection_test",
        completed_steps: ["welcome", "permission_check", "relay_test"]
      }
    });

    expect(screen.getByText("Deteksi game")).toBeInTheDocument();
    expect(
      screen.getByText("Buka game kamu, lalu jalankan scan agar routing tepat sasaran.")
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Scan game sekarang" })).toBeInTheDocument();
  });

  it("keeps onboarding layout aligned with stepper + status + active step pattern", () => {
    const { container } = render_flow({
      state: {
        state: "failed",
        failed_step: "first_connect",
        completed_steps: ["welcome", "permission_check", "relay_test", "game_detection_test"],
        reason_code: "connect_attempt_failed"
      }
    });

    expect(screen.getByLabelText("Onboarding flow")).toBeInTheDocument();
    expect(screen.getByRole("status")).toHaveTextContent("Action Needed");
    expect(screen.getByText("Koneksi pertama")).toBeInTheDocument();
    expect(screen.getByText("connect_attempt_failed")).toBeInTheDocument();
    expect(container.querySelector(".kp-onboarding-flow-header")).toBeTruthy();
    expect(container.querySelector(".kp-onboarding-step-screen")).toBeTruthy();
  });
});

