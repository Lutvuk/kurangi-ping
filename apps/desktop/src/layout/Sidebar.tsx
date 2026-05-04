const NAV_ITEMS = ["Dashboard", "Routing", "Relays", "Settings"];

export function Sidebar() {
  return (
    <aside className="kp-sidebar" aria-label="Primary">
      <div className="kp-sidebar-title">Kurangi Ping</div>
      <nav className="kp-sidebar-nav">
        {NAV_ITEMS.map((item, index) => (
          <button
            key={item}
            type="button"
            className={`kp-nav-item ${index === 0 ? "is-active" : ""}`}
            aria-current={index === 0 ? "page" : undefined}
          >
            <span className="kp-nav-icon" aria-hidden="true">
              {item.charAt(0)}
            </span>
            <span className="kp-nav-label">{item}</span>
          </button>
        ))}
      </nav>
    </aside>
  );
}
