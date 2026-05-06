import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { get_recovery_descriptor, RecoveryPanel } from "./RecoveryPanel";

describe("RecoveryPanel", () => {
  it("maps backend reason code to actionable user message", () => {
    render(<RecoveryPanel reasonCode="permission_admin_required" />);

    expect(screen.getByText("Butuh izin administrator")).toBeInTheDocument();
    expect(
      screen.getByText(
        "Aplikasi perlu izin admin untuk mengatur routing. Jalankan ulang sebagai Administrator."
      )
    ).toBeInTheDocument();
    expect(screen.getByText("permission_admin_required")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Retry current onboarding step" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Open troubleshooting guidance" })).toBeDisabled();
  });

  it("always shows retry plus guidance action for failure recovery", () => {
    const onRetry = vi.fn();
    const onGuidance = vi.fn();

    render(
      <RecoveryPanel reasonCode="relay_no_healthy_nodes" onRetry={onRetry} onGuidance={onGuidance} />
    );

    const retryButton = screen.getByRole("button", { name: "Retry current onboarding step" });
    const guidanceButton = screen.getByRole("button", { name: "Open troubleshooting guidance" });
    fireEvent.click(retryButton);
    fireEvent.click(guidanceButton);

    expect(onRetry).toHaveBeenCalledTimes(1);
    expect(onGuidance).toHaveBeenCalledTimes(1);
  });

  it("shows continue safe action only on permitted non-blocking recovery paths", () => {
    const onContinueSafe = vi.fn();
    const { rerender } = render(
      <RecoveryPanel reasonCode="relay_no_healthy_nodes" onContinueSafe={onContinueSafe} />
    );

    const continueButton = screen.getByRole("button", { name: "Continue onboarding without routing" });
    fireEvent.click(continueButton);
    expect(onContinueSafe).toHaveBeenCalledTimes(1);

    rerender(<RecoveryPanel reasonCode="permission_admin_required" onContinueSafe={onContinueSafe} />);
    expect(screen.queryByRole("button", { name: "Continue onboarding without routing" })).toBeNull();
  });

  it("keeps recovery interactions accessible with explicit labels and keyboard focus", () => {
    render(
      <RecoveryPanel
        reasonCode="connect_attempt_failed"
        onRetry={() => undefined}
        onGuidance={() => undefined}
        onContinueSafe={() => undefined}
      />
    );

    const retryButton = screen.getByRole("button", { name: "Retry current onboarding step" });
    const guidanceButton = screen.getByRole("button", { name: "Open troubleshooting guidance" });
    const continueButton = screen.getByRole("button", { name: "Continue onboarding without routing" });

    retryButton.focus();
    expect(retryButton).toHaveFocus();
    guidanceButton.focus();
    expect(guidanceButton).toHaveFocus();
    continueButton.focus();
    expect(continueButton).toHaveFocus();
    expect(screen.getByLabelText("Onboarding recovery panel")).toBeInTheDocument();
  });

  it("uses deterministic fallback descriptor for unknown reason code", () => {
    const descriptor = get_recovery_descriptor("unknown_case");
    expect(descriptor.title).toBe("Perlu tindakan sebelum lanjut");
    expect(descriptor.canContinueSafe).toBe(false);
  });
});

