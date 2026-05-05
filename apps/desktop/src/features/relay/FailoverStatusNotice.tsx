import type { HTMLAttributes } from "react";
import type { RelayFailoverViewModel } from "./model";
import "./FailoverStatusNotice.css";

type NoticeState = "neutral" | "probe" | "signal" | "error";

type NoticeContent = {
  title: string;
  message: string;
  tone: NoticeState;
  live: "polite" | "assertive";
  role: "status" | "alert";
};

const reasonMessageMap: Record<string, string> = {
  dead_relay_detected: "Relay aktif tidak responsif. Sedang pindah ke relay cadangan.",
  relay_unreachable: "Relay aktif tidak responsif. Sedang pindah ke relay cadangan.",
  degraded_grace_exceeded: "Kualitas relay menurun. Sedang memilih jalur yang lebih stabil.",
  relay_unstable: "Kualitas relay menurun. Sedang memilih jalur yang lebih stabil.",
  poll_failure_streak_exceeded: "Pemeriksaan relay gagal berulang. Mencoba jalur alternatif.",
  relay_probe_failed: "Pemeriksaan relay gagal berulang. Mencoba jalur alternatif.",
  hysteresis_window_active: "Koneksi sedang distabilkan sebelum percobaan berikutnya.",
  failover_cooldown: "Koneksi sedang distabilkan sebelum percobaan berikutnya.",
  switch_successful: "Jalur baru aktif. Koneksi kamu sudah pulih.",
  no_alternative_relay: "Belum ada relay alternatif yang siap digunakan.",
  switch_retry_exhausted: "Percobaan perpindahan relay sudah habis. Coba lagi sebentar lagi.",
  switch_preparation_failed: "Persiapan perpindahan relay gagal. Coba aktifkan ulang routing.",
  switch_apply_failed: "Perpindahan relay belum berhasil. Sistem akan mencoba lagi saat stabil.",
  permission_required: "Perlu izin tambahan untuk menyelesaikan perpindahan relay.",
  network_timeout: "Waktu koneksi habis saat pindah relay. Menunggu percobaan berikutnya.",
  safe_generic_issue: "Terjadi kendala relay. Sistem menjaga koneksi tetap aman."
};

function reasonMessage(reasonCode?: string): string {
  if (!reasonCode) {
    return reasonMessageMap.safe_generic_issue;
  }

  return reasonMessageMap[reasonCode] ?? reasonMessageMap.safe_generic_issue;
}

function buildNoticeContent(failover: RelayFailoverViewModel): NoticeContent {
  if (failover.currentState === "switching") {
    return {
      title: "Reconnecting...",
      message: reasonMessage(failover.reasonCode),
      tone: "probe",
      live: "polite",
      role: "status"
    };
  }

  if (failover.currentState === "recovered") {
    return {
      title: "Connection Recovered",
      message: reasonMessage(failover.reasonCode),
      tone: "signal",
      live: "polite",
      role: "status"
    };
  }

  if (failover.currentState === "failed") {
    return {
      title: "No Relay Available",
      message: reasonMessage(failover.reasonCode),
      tone: "error",
      live: "assertive",
      role: "alert"
    };
  }

  return {
    title: "Relay Stable",
    message: "Koneksi relay dalam kondisi stabil.",
    tone: "neutral",
    live: "polite",
    role: "status"
  };
}

export type FailoverStatusNoticeProps = HTMLAttributes<HTMLDivElement> & {
  failover: RelayFailoverViewModel;
};

export function FailoverStatusNotice({ failover, className, ...props }: FailoverStatusNoticeProps) {
  const content = buildNoticeContent(failover);
  const composedClassName = ["kp-failover-notice", `kp-failover-notice--${content.tone}`, className]
    .filter(Boolean)
    .join(" ");

  return (
    <div
      className={composedClassName}
      role={content.role}
      aria-live={content.live}
      aria-atomic="true"
      {...props}
    >
      <p className="kp-failover-notice-title">{content.title}</p>
      <p className="kp-failover-notice-message">{content.message}</p>
    </div>
  );
}
