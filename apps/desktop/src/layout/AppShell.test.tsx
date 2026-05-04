import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { AppShell } from "./AppShell";

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

  it("renders placeholder content slot for feature injection", () => {
    render(
      <AppShell>
        <div>Injected Content</div>
      </AppShell>,
    );

    expect(screen.getByText("Injected Content")).toBeInTheDocument();
  });
});
