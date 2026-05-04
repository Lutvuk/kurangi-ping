import { AppShell } from "./layout/AppShell";
import { Panel } from "./layout/Panel";

export default function App() {
  return (
    <AppShell>
      <Panel eyebrow="Foundation" title="Kurangi Ping 2">
        <p className="kp-copy">Desktop shell regions are ready for feature injection.</p>
      </Panel>
    </AppShell>
  );
}
