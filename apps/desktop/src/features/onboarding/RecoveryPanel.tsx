import { Button, Card, StatusBadge } from "../../components/primitives";

export type RecoveryCategory = "permission" | "network" | "relay" | "detection" | "connect" | "generic";

export type RecoveryDescriptor = {
  title: string;
  message: string;
  guidanceActionLabel: string;
  category: RecoveryCategory;
  canContinueSafe: boolean;
};

const recoveryByReasonCode: Record<string, RecoveryDescriptor> = {
  permission_admin_required: {
    title: "Butuh izin administrator",
    message: "Aplikasi perlu izin admin untuk mengatur routing. Jalankan ulang sebagai Administrator.",
    guidanceActionLabel: "Lihat Cara Buka Admin",
    category: "permission",
    canContinueSafe: false
  },
  permission_route_table_denied: {
    title: "Akses route table ditolak",
    message: "Periksa pengaturan firewall atau kebijakan UAC, lalu coba lagi.",
    guidanceActionLabel: "Panduan Firewall & UAC",
    category: "permission",
    canContinueSafe: false
  },
  environment_network_unavailable: {
    title: "Jaringan belum tersedia",
    message: "Koneksi internet belum terdeteksi. Sambungkan jaringan lalu ulangi langkah ini.",
    guidanceActionLabel: "Tips Cek Koneksi",
    category: "network",
    canContinueSafe: false
  },
  environment_unsupported_os: {
    title: "Versi Windows belum didukung",
    message: "Versi sistem saat ini belum kompatibel untuk flow routing ini.",
    guidanceActionLabel: "Lihat Syarat Sistem",
    category: "network",
    canContinueSafe: false
  },
  relay_health_unavailable: {
    title: "Data relay belum tersedia",
    message: "Status relay belum bisa diambil sekarang. Kamu bisa lanjut tanpa routing atau coba lagi.",
    guidanceActionLabel: "Lihat Status Relay",
    category: "relay",
    canContinueSafe: true
  },
  relay_no_healthy_nodes: {
    title: "Relay sehat belum tersedia",
    message: "Tidak ada relay yang cukup stabil saat ini. Kamu bisa lanjut aman tanpa routing.",
    guidanceActionLabel: "Tips Stabilkan Relay",
    category: "relay",
    canContinueSafe: true
  },
  relay_only_degraded: {
    title: "Relay sedang degraded",
    message: "Relay tersedia tapi belum ideal. Lanjut aman tetap bisa sambil menunggu kondisi membaik.",
    guidanceActionLabel: "Lihat Cara Optimasi",
    category: "relay",
    canContinueSafe: true
  },
  detection_not_found: {
    title: "Game belum terdeteksi",
    message: "Buka game dulu, lalu scan ulang agar routing tepat sasaran.",
    guidanceActionLabel: "Buka Panduan Deteksi Game",
    category: "detection",
    canContinueSafe: true
  },
  connect_attempt_failed: {
    title: "Koneksi pertama gagal",
    message: "Koneksi belum berhasil di percobaan ini. Coba lagi atau lanjut tanpa routing dulu.",
    guidanceActionLabel: "Troubleshoot Koneksi",
    category: "connect",
    canContinueSafe: true
  }
};

const fallbackRecovery: RecoveryDescriptor = {
  title: "Perlu tindakan sebelum lanjut",
  message: "Langkah ini belum selesai. Coba ulangi atau buka panduan cepat untuk menyelesaikan masalah.",
  guidanceActionLabel: "Buka Troubleshoot",
  category: "generic",
  canContinueSafe: false
};

export type RecoveryPanelProps = {
  reasonCode: string;
  onRetry?: () => void;
  onGuidance?: () => void;
  onContinueSafe?: () => void;
  continueSafeAllowed?: boolean;
  isBusy?: boolean;
};

export function get_recovery_descriptor(reasonCode: string): RecoveryDescriptor {
  return recoveryByReasonCode[reasonCode] ?? fallbackRecovery;
}

export function RecoveryPanel({
  reasonCode,
  onRetry,
  onGuidance,
  onContinueSafe,
  continueSafeAllowed,
  isBusy = false
}: RecoveryPanelProps) {
  const descriptor = get_recovery_descriptor(reasonCode);
  const allowSafeContinue = continueSafeAllowed ?? descriptor.canContinueSafe;

  return (
    <Card as="section" className="kp-recovery-panel" aria-label="Onboarding recovery panel">
      <header className="kp-recovery-panel-header">
        <h3 className="kp-recovery-panel-title">{descriptor.title}</h3>
        <StatusBadge state="error" label="Recovery" />
      </header>

      <p className="kp-recovery-panel-copy">{descriptor.message}</p>
      <p className="kp-recovery-panel-code">
        Reason code: <code>{reasonCode}</code>
      </p>

      <div className="kp-recovery-panel-actions">
        <Button
          intent="secondary"
          ariaLabel="Retry current onboarding step"
          onClick={onRetry}
          disabled={!onRetry || isBusy}
        >
          {isBusy ? "Retrying..." : "Retry"}
        </Button>
        <Button
          intent="secondary"
          ariaLabel="Open troubleshooting guidance"
          onClick={onGuidance}
          disabled={!onGuidance || isBusy}
        >
          {descriptor.guidanceActionLabel}
        </Button>
        {allowSafeContinue ? (
          <Button
            intent="primary"
            ariaLabel="Continue onboarding without routing"
            onClick={onContinueSafe}
            disabled={!onContinueSafe || isBusy}
          >
            Continue Safe
          </Button>
        ) : null}
      </div>
    </Card>
  );
}

