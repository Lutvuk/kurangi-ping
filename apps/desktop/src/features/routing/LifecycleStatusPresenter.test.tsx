import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { LifecycleStatusPresenter, type LifecycleStatusState } from "./LifecycleStatusPresenter";

function renderState(state: LifecycleStatusState, reasonCode?: string) {
  render(
    <LifecycleStatusPresenter
      model={{
        state,
        reasonCode
      }}
    />
  );
}

describe("LifecycleStatusPresenter", () => {
  it("renders all five lifecycle states with semantic styling", () => {
    const { rerender } = render(
      <LifecycleStatusPresenter model={{ state: "idle" }} title="Lifecycle Status" />
    );
    expect(screen.getByLabelText("Lifecycle Status")).toHaveClass("kp-lifecycle-status--idle");

    rerender(<LifecycleStatusPresenter model={{ state: "arming" }} title="Lifecycle Status" />);
    expect(screen.getByLabelText("Lifecycle Status")).toHaveClass("kp-lifecycle-status--arming");

    rerender(<LifecycleStatusPresenter model={{ state: "active" }} title="Lifecycle Status" />);
    expect(screen.getByLabelText("Lifecycle Status")).toHaveClass("kp-lifecycle-status--active");

    rerender(
      <LifecycleStatusPresenter model={{ state: "disarming" }} title="Lifecycle Status" />
    );
    expect(screen.getByLabelText("Lifecycle Status")).toHaveClass(
      "kp-lifecycle-status--disarming"
    );

    rerender(<LifecycleStatusPresenter model={{ state: "error" }} title="Lifecycle Status" />);
    expect(screen.getByLabelText("Lifecycle Status")).toHaveClass("kp-lifecycle-status--error");
  });

  it("maps reason code to user-friendly text", () => {
    renderState("error", "detection_not_found");
    expect(
      screen.getByText("Game belum terdeteksi. Jalankan game lalu aktifkan lagi.")
    ).toBeInTheDocument();
    expect(screen.getByText("(detection_not_found)")).toBeInTheDocument();
  });

  it("falls back to safe generic reason for unknown codes", () => {
    renderState("error", "INTERNAL_TRACE_0xBAD");
    expect(
      screen.getByText("Terjadi kendala koneksi. Coba ulang dari tombol utama.")
    ).toBeInTheDocument();
  });

  it("provides accessibility announcements with polite vs assertive modes", () => {
    const { rerender } = render(
      <LifecycleStatusPresenter model={{ state: "active" }} title="Lifecycle Status" />
    );

    const active = screen.getByLabelText("Lifecycle Status");
    expect(active).toHaveAttribute("role", "status");
    expect(active).toHaveAttribute("aria-live", "polite");

    rerender(<LifecycleStatusPresenter model={{ state: "error" }} title="Lifecycle Status" />);
    const error = screen.getByLabelText("Lifecycle Status");
    expect(error).toHaveAttribute("role", "alert");
    expect(error).toHaveAttribute("aria-live", "assertive");
  });
});
