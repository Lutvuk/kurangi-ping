export type SidebarItem = {
  id: string;
  label: string;
  shortLabel?: string;
};

export type SidebarProps = {
  title?: string;
  items?: SidebarItem[];
  activeId?: string;
};

const DEFAULT_ITEMS: SidebarItem[] = [
  { id: "dashboard", label: "Dashboard", shortLabel: "D" },
  { id: "routing", label: "Routing", shortLabel: "R" },
  { id: "relays", label: "Relays", shortLabel: "L" },
  { id: "settings", label: "Settings", shortLabel: "S" }
];

export function Sidebar({
  title = "Kurangi Ping",
  items = DEFAULT_ITEMS,
  activeId = DEFAULT_ITEMS[0]?.id
}: SidebarProps) {
  return (
    <aside className="kp-sidebar" aria-label="Primary">
      <div className="kp-sidebar-title">{title}</div>
      <nav className="kp-sidebar-nav">
        {items.map((item) => (
          <button
            key={item.id}
            type="button"
            className={`kp-nav-item ${item.id === activeId ? "is-active" : ""}`}
            aria-current={item.id === activeId ? "page" : undefined}
          >
            <span className="kp-nav-icon" aria-hidden="true">
              {item.shortLabel ?? item.label.charAt(0)}
            </span>
            <span className="kp-nav-label">{item.label}</span>
          </button>
        ))}
      </nav>
    </aside>
  );
}
