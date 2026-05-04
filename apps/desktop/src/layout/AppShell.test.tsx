import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { AppShell } from "./AppShell";
import { Panel } from "./Panel";

describe("AppShell", () => {
  it("renders sidebar and top status region", () => {
    render(
      <AppShell>
        <div>Injected Content</div>
      </AppShell>,
    );

    expect(screen.getByLabelText("Primary")).toBeInTheDocument();
    expect(screen.getByText("Status")).toBeInTheDocument();
  });

  it("renders panel slot for future feature mounting", () => {
    render(
      <AppShell>
        <Panel eyebrow="Metrics" title="Live Data">
          <div>Injected Content</div>
        </Panel>
      </AppShell>,
    );

    expect(screen.getByText("Metrics")).toBeInTheDocument();
    expect(screen.getByText("Live Data")).toBeInTheDocument();
    expect(screen.getByText("Injected Content")).toBeInTheDocument();
  });

  it("supports typed sidebar/topbar overrides", () => {
    render(
      <AppShell
        sidebar={{
          title: "Ops Console",
          items: [
            { id: "home", label: "Home" },
            { id: "diag", label: "Diagnostics" }
          ],
          activeId: "diag"
        }}
        topBar={{ statusLabel: "Link", statusValue: "Active", meta: "sin-01 selected" }}
      >
        <div>Custom Shell</div>
      </AppShell>,
    );

    expect(screen.getByText("Ops Console")).toBeInTheDocument();
    expect(screen.getByText("Diagnostics")).toBeInTheDocument();
    expect(screen.getByText("Link")).toBeInTheDocument();
    expect(screen.getByText("Active")).toBeInTheDocument();
    expect(screen.getByText("sin-01 selected")).toBeInTheDocument();
  });
});
