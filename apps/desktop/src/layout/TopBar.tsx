export function TopBar() {
  return (
    <header className="kp-topbar">
      <div className="kp-topbar-left">
        <span className="kp-label">Status</span>
        <span className="kp-topbar-value">Idle</span>
      </div>
      <div className="kp-topbar-right">
        <span className="kp-topbar-meta">No relay selected</span>
      </div>
    </header>
  );
}
