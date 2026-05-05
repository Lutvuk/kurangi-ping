import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { MetricsStatePresenter, type MetricsStatePresenterState } from "./MetricsStatePresenter";

function renderState(state: MetricsStatePresenterState, reasonCode?: string) {
  render(
    <MetricsStatePresenter
      model={{
        state,
        reasonCode
      }}
    />
  );
}

describe("MetricsStatePresenter", () => {
  it("represents all five dashboard states", () => {
    const { rerender } = render(
      <MetricsStatePresenter model={{ state: "idle" }} title="Metrics State" />
    );
    expect(screen.getByLabelText("Metrics State")).toHaveClass("kp-metrics-state--idle");

    rerender(<MetricsStatePresenter model={{ state: "measuring" }} title="Metrics State" />);
    expect(screen.getByLabelText("Metrics State")).toHaveClass("kp-metrics-state--measuring");

    rerender(<MetricsStatePresenter model={{ state: "live" }} title="Metrics State" />);
    expect(screen.getByLabelText("Metrics State")).toHaveClass("kp-metrics-state--live");

    rerender(<MetricsStatePresenter model={{ state: "degraded" }} title="Metrics State" />);
    expect(screen.getByLabelText("Metrics State")).toHaveClass("kp-metrics-state--degraded");

    rerender(<MetricsStatePresenter model={{ state: "error" }} title="Metrics State" />);
    expect(screen.getByLabelText("Metrics State")).toHaveClass("kp-metrics-state--error");
  });

  it("shows actionable guidance for degraded and error states", () => {
    const { rerender } = render(
      <MetricsStatePresenter model={{ state: "degraded", reasonCode: "freshness_timeout" }} />
    );
    expect(
      screen.getByText("Sampel terlalu lama. Coba scan ulang relay dan cek koneksi.")
    ).toBeInTheDocument();
    expect(screen.getByText("(freshness_timeout)")).toBeInTheDocument();

    rerender(<MetricsStatePresenter model={{ state: "error", reasonCode: "invalid_sample_values" }} />);
    expect(
      screen.getByText("Nilai sampel tidak valid. Cek kondisi jaringan lalu ulangi.")
    ).toBeInTheDocument();
    expect(screen.getByText("(invalid_sample_values)")).toBeInTheDocument();
  });

  it("falls back to safe actionable message on unknown reason code", () => {
    renderState("error", "TRACE_PRIVATE_REASON");
    expect(
      screen.getByText("Pengukuran belum stabil. Coba lagi beberapa saat.")
    ).toBeInTheDocument();
  });

  it("includes accessibility announcement semantics", () => {
    const { rerender } = render(
      <MetricsStatePresenter model={{ state: "live" }} title="Metrics State" />
    );
    const live = screen.getByLabelText("Metrics State");
    expect(live).toHaveAttribute("role", "status");
    expect(live).toHaveAttribute("aria-live", "polite");

    rerender(<MetricsStatePresenter model={{ state: "error" }} title="Metrics State" />);
    const error = screen.getByLabelText("Metrics State");
    expect(error).toHaveAttribute("role", "alert");
    expect(error).toHaveAttribute("aria-live", "assertive");
  });
});
