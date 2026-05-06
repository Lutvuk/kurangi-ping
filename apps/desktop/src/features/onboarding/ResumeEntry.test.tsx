import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ResumeEntry } from "./ResumeEntry";

describe("ResumeEntry", () => {
  it("shows checkpoint context clearly for resumed onboarding", () => {
    render(<ResumeEntry stepLabel="Relay Test" completedCount={2} totalCount={5} />);

    expect(screen.getByLabelText("Onboarding resume entry")).toBeInTheDocument();
    expect(screen.getByText("Lanjut dari checkpoint terakhir")).toBeInTheDocument();
    expect(screen.getByText(/Kamu sudah menyelesaikan 2\/5 langkah/i)).toBeInTheDocument();
    expect(screen.getByText("Relay Test")).toBeInTheDocument();
  });

  it("fires resume callback from primary action", () => {
    const onResume = vi.fn();
    render(
      <ResumeEntry stepLabel="Game Detect" completedCount={3} totalCount={5} onResume={onResume} />
    );

    fireEvent.click(screen.getByRole("button", { name: "Resume onboarding from checkpoint" }));
    expect(onResume).toHaveBeenCalledTimes(1);
  });

  it("requires restart confirmation before destructive reset callback", () => {
    const onRestart = vi.fn();
    render(
      <ResumeEntry
        stepLabel="Permission"
        completedCount={1}
        totalCount={5}
        onResume={() => undefined}
        onRestart={onRestart}
      />
    );

    fireEvent.click(screen.getByRole("button", { name: "Restart onboarding setup" }));
    expect(screen.getByRole("alertdialog", { name: "Confirm onboarding restart" })).toBeInTheDocument();
    expect(onRestart).toHaveBeenCalledTimes(0);

    fireEvent.click(screen.getByRole("button", { name: "Confirm restart onboarding" }));
    expect(onRestart).toHaveBeenCalledTimes(1);
  });

  it("allows canceling restart confirmation safely", () => {
    render(
      <ResumeEntry
        stepLabel="First Connect"
        completedCount={4}
        totalCount={5}
        onResume={() => undefined}
        onRestart={() => undefined}
      />
    );

    fireEvent.click(screen.getByRole("button", { name: "Restart onboarding setup" }));
    expect(screen.getByRole("alertdialog", { name: "Confirm onboarding restart" })).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Cancel restart onboarding" }));

    expect(screen.queryByRole("alertdialog", { name: "Confirm onboarding restart" })).toBeNull();
    expect(screen.getByRole("button", { name: "Restart onboarding setup" })).toBeInTheDocument();
  });
});

