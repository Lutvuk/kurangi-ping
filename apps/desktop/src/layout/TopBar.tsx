export type TopBarProps = {
  statusLabel?: string;
  statusValue?: string;
  meta?: string;
};

export function TopBar({
  statusLabel = "Status",
  statusValue = "Idle",
  meta = "No relay selected"
}: TopBarProps) {
  return (
    <header className="kp-topbar">
      <div className="kp-topbar-left">
        <span className="kp-label">{statusLabel}</span>
        <span className="kp-topbar-value">{statusValue}</span>
      </div>
      <div className="kp-topbar-right">
        <span className="kp-topbar-meta">{meta}</span>
      </div>
    </header>
  );
}
