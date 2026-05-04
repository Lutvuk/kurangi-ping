import type { ReactNode } from "react";
import { Sidebar } from "./Sidebar";
import { TopBar } from "./TopBar";

type AppShellProps = {
  children: ReactNode;
};

export function AppShell({ children }: AppShellProps) {
  return (
    <main className="kp-shell-root">
      <Sidebar />
      <section className="kp-shell-main">
        <TopBar />
        <div className="kp-shell-content">{children}</div>
      </section>
    </main>
  );
}
