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

function moveSidebarFocus(current: HTMLButtonElement, direction: 1 | -1): void {
  const nav = current.closest(".kp-sidebar-nav");
  if (!nav) {
    return;
  }
  const buttons = Array.from(nav.querySelectorAll<HTMLButtonElement>(".kp-nav-item"));
  const index = buttons.indexOf(current);
  if (index === -1) {
    return;
  }

  const nextIndex = index + direction;
  if (nextIndex < 0 || nextIndex >= buttons.length) {
    return;
  }

  buttons[nextIndex]?.focus();
}

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
            onKeyDown={(event) => {
              if (event.key === "ArrowDown") {
                event.preventDefault();
                moveSidebarFocus(event.currentTarget, 1);
              }
              if (event.key === "ArrowUp") {
                event.preventDefault();
                moveSidebarFocus(event.currentTarget, -1);
              }
            }}
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
