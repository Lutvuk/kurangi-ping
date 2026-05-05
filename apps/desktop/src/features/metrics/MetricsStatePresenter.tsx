import "./MetricsStatePresenter.css";

export type MetricsStatePresenterState = "idle" | "measuring" | "live" | "degraded" | "error";

export type MetricsStatePresenterModel = {
  state: MetricsStatePresenterState;
  reasonCode?: string;
  detail?: string;
};

export type MetricsStatePresenterProps = {
  model: MetricsStatePresenterModel;
  title?: string;
};

const titleByState: Record<MetricsStatePresenterState, string> = {
  idle: "Metrics Idle",
  measuring: "Measuring",
  live: "Metrics Live",
  degraded: "Metrics Degraded",
  error: "Metrics Error"
};

const descriptionByState: Record<MetricsStatePresenterState, string> = {
  idle: "Belum ada sampel baru. Aktifkan routing untuk mulai pengukuran.",
  measuring: "Probe sedang mengumpulkan sampel terbaru.",
  live: "Metrik stabil dan terus diperbarui secara real-time.",
  degraded: "Update metrik tidak stabil. Menampilkan nilai terakhir yang aman.",
  error: "Perhitungan metrik gagal. Silakan cek reason lalu lanjutkan."
};

const reasonMessageMap: Record<string, string> = {
  no_samples_yet: "Belum ada sampel. Tunggu probe pertama selesai.",
  freshness_timeout: "Sampel terlalu lama. Coba scan ulang relay dan cek koneksi.",
  probe_failed: "Probe gagal merespons. Coba lanjutkan lalu pantau beberapa detik.",
  invalid_computation_window: "Window perhitungan tidak valid. Tunggu siklus sampel berikutnya.",
  invalid_sample_values: "Nilai sampel tidak valid. Cek kondisi jaringan lalu ulangi.",
  safe_generic_issue: "Pengukuran belum stabil. Coba lagi beberapa saat."
};

function toFriendlyReason(reasonCode?: string): string {
  if (!reasonCode) {
    return reasonMessageMap.safe_generic_issue;
  }
  return reasonMessageMap[reasonCode] ?? reasonMessageMap.safe_generic_issue;
}

export function MetricsStatePresenter({
  model,
  title = "Metrics State"
}: MetricsStatePresenterProps) {
  const announcementRole = model.state === "error" ? "alert" : "status";
  const announcementLive = model.state === "error" ? "assertive" : "polite";
  const detail = model.detail ?? descriptionByState[model.state];
  const showReason = model.state === "degraded" || model.state === "error";
  const reasonText = showReason ? toFriendlyReason(model.reasonCode) : null;

  return (
    <section
      className={`kp-metrics-state kp-metrics-state--${model.state}`}
      aria-label={title}
      data-state={model.state}
      role={announcementRole}
      aria-live={announcementLive}
      aria-atomic="true"
    >
      <p className="kp-metrics-state-eyebrow">{title}</p>
      <p className="kp-metrics-state-title">{titleByState[model.state]}</p>
      <p className="kp-metrics-state-detail">{detail}</p>
      {reasonText ? (
        <p className="kp-metrics-state-reason">
          {reasonText}{" "}
          {model.reasonCode ? (
            <span className="kp-metrics-state-reason-code">({model.reasonCode})</span>
          ) : null}
        </p>
      ) : null}
    </section>
  );
}
