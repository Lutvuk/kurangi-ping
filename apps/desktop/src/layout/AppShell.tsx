import type { ReactNode } from "react";
import { Sidebar, type SidebarProps } from "./Sidebar";
import { TopBar, type TopBarProps } from "./TopBar";

export type AppShellProps = {
  children: ReactNode;
  sidebar?: SidebarProps;
  topBar?: TopBarProps;
  contentClassName?: string;
};

export function AppShell({ children, sidebar, topBar, contentClassName }: AppShellProps) {
  const shellContentClass = ["kp-shell-content", contentClassName].filter(Boolean).join(" ");

  return (
    <main className="kp-shell-root">
      <Sidebar {...sidebar} />
      <section className="kp-shell-main">
        <TopBar {...topBar} />
        <div className={shellContentClass}>{children}</div>
      </section>
    </main>
  );
}
