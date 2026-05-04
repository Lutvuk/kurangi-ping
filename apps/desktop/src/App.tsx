import { ConnectionStatusBadge, PrimaryToggle } from "./components/modules";
import { AppShell } from "./layout/AppShell";
import { Panel } from "./layout/Panel";
import { FoundationShowcasePage } from "./pages/foundation-showcase";

export default function App() {
  const isFoundationShowcaseEnabled =
    import.meta.env.DEV || import.meta.env.VITE_ENABLE_FOUNDATION_SHOWCASE === "1";

  if (isFoundationShowcaseEnabled) {
    return <FoundationShowcasePage />;
  }

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
