import { ConnectionStatusBadge, PrimaryToggle } from "./components/modules";
import { AppShell } from "./layout/AppShell";
import { Panel } from "./layout/Panel";

export default function App() {
  return (
    <AppShell>
      <Panel eyebrow="Foundation" title="Kurangi Ping 2">
        <p className="kp-copy">Desktop shell regions are ready for feature injection.</p>
        <div
          style={{
            marginTop: "var(--space-4)",
            display: "flex",
            alignItems: "center",
            gap: "var(--space-4)"
          }}
        >
          <PrimaryToggle state="off" />
          <ConnectionStatusBadge state="off" />
        </div>
      </Panel>
    </AppShell>
  );
}
