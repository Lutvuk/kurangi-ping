import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { FoundationShowcasePage } from "./foundation-showcase";

describe("FoundationShowcasePage", () => {
  it("renders major foundation components and key states", () => {
    render(<FoundationShowcasePage />);

    expect(screen.getByText("UI Composition Showcase")).toBeInTheDocument();
    expect(screen.getByText("Compact Preview (<= 960px)")).toBeInTheDocument();
    expect(screen.getByText("Full Preview (>= 1280px)")).toBeInTheDocument();

    expect(screen.getAllByRole("button", { name: /routing toggle/i }).length).toBeGreaterThan(0);
    expect(screen.getAllByRole("status").length).toBeGreaterThan(0);
    expect(screen.getAllByLabelText("Ping metrics").length).toBeGreaterThan(0);
    expect(screen.getAllByRole("listitem").length).toBeGreaterThan(0);
    expect(screen.getAllByText("Detected").length).toBeGreaterThan(0);
    expect(screen.getAllByText("Not Detected").length).toBeGreaterThan(0);
    expect(screen.getAllByLabelText("Onboarding progress").length).toBeGreaterThan(0);
  });
});

