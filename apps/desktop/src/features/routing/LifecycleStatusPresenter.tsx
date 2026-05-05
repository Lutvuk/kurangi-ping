import "./LifecycleStatusPresenter.css";

export type LifecycleStatusState = "idle" | "arming" | "active" | "disarming" | "error";

export type LifecycleStatusViewModel = {
  state: LifecycleStatusState;
  reasonCode?: string;
  detail?: string;
};

export type LifecycleStatusPresenterProps = {
  model: LifecycleStatusViewModel;
  title?: string;
};

const titleByState: Record<LifecycleStatusState, string> = {
  idle: "Routing Idle",
  arming: "Routing Arming",
  active: "Routing Active",
  disarming: "Routing Disarming",
  error: "Routing Error"
};

const descriptionByState: Record<LifecycleStatusState, string> = {
  idle: "Routing belum aktif. Tekan ON untuk memulai jalur teroptimasi.",
  arming: "Sedang memverifikasi game, manifest, dan jalur relay terbaik.",
  active: "Routing aktif dan koneksi sedang dijaga tetap stabil.",
  disarming: "Sedang menutup jalur routing dengan aman.",
  error: "Routing gagal distabilkan. Cek pesan reason lalu coba lagi."
};

const reasonMessageMap: Record<string, string> = {
  detection_not_found: "Game belum terdeteksi. Jalankan game lalu aktifkan lagi.",
  detection_stale: "Status deteksi sudah stale. Scan ulang sebelum mengaktifkan routing.",
  manifest_signature_invalid: "Manifest relay tidak valid. Tunggu update konfigurasi terbaru.",
  manifest_expired: "Manifest relay sudah kedaluwarsa. Coba lagi sebentar lagi.",
  precheck_no_protocols_configured: "Konfigurasi protokol belum siap.",
  precheck_no_candidates_available: "Belum ada kandidat relay yang bisa dipakai.",
  off_teardown_failed: "Penutupan routing belum bersih. Sistem menjaga koneksi tetap aman.",
  arming_timeout: "Waktu aktivasi habis. Coba ON lagi beberapa saat lagi.",
  disarming_timeout: "Waktu penonaktifan habis. Coba OFF lagi beberapa saat lagi.",
  safe_generic_issue: "Terjadi kendala koneksi. Coba ulang dari tombol utama."
};

function toFriendlyReason(reasonCode?: string): string {
  if (!reasonCode) {
    return reasonMessageMap.safe_generic_issue;
  }
  return reasonMessageMap[reasonCode] ?? reasonMessageMap.safe_generic_issue;
}

export function LifecycleStatusPresenter({
  model,
  title = "Lifecycle Status"
}: LifecycleStatusPresenterProps) {
  const announcementRole = model.state === "error" ? "alert" : "status";
  const announcementLive = model.state === "error" ? "assertive" : "polite";
  const reasonText = model.reasonCode ? toFriendlyReason(model.reasonCode) : null;
  const detail = model.detail ?? descriptionByState[model.state];

  return (
    <section
      className={`kp-lifecycle-status kp-lifecycle-status--${model.state}`}
      aria-label={title}
      data-state={model.state}
      role={announcementRole}
      aria-live={announcementLive}
      aria-atomic="true"
    >
      <p className="kp-lifecycle-status-eyebrow">{title}</p>
      <p className="kp-lifecycle-status-title">{titleByState[model.state]}</p>
      <p className="kp-lifecycle-status-detail">{detail}</p>
      {reasonText ? (
        <p className="kp-lifecycle-status-reason">
          {reasonText} <span className="kp-lifecycle-status-reason-code">({model.reasonCode})</span>
        </p>
      ) : null}
    </section>
  );
}
