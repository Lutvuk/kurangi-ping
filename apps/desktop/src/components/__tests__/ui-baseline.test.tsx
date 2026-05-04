import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import {
  Button,
  Input,
  Modal,
  Select,
  StatusBadge
} from "../primitives";
import {
  ConnectionStatusBadge,
  GameDetectionRow,
  OnboardingStepper,
  PingMetricCard,
  PrimaryToggle,
  RelayHealthListItem
} from "../modules";
import { AppShell } from "../../layout/AppShell";

describe("ui baseline suite", () => {
  it("renders key primitives and module components in a deterministic baseline", () => {
    render(
      <div>
        <Button intent="primary">Connect</Button>
        <Input aria-label="Executable path" value="ffxiv_dx11.exe" readOnly />
        <Select
          aria-label="Relay region"
          value="sin"
          options={[
            { value: "auto", label: "Auto" },
            { value: "sin", label: "Singapore" }
          ]}
          onChange={() => undefined}
        />
        <StatusBadge state="on" label="Connected" />

        <PrimaryToggle state="on" />
        <ConnectionStatusBadge state="on" />
        <PingMetricCard state="on" currentPingMs={44} baselinePingMs={72} reductionPct={38.9} />
        <div role="list">
          <RelayHealthListItem hostname="sin-01.relay.local" latencyMs={38} region="sin" health="ok" />
        </div>
        <GameDetectionRow
          gameName="Final Fantasy XIV"
          serverInfo="Elemental - Tonberry"
          state="detected"
          icon="FF"
        />
      </div>
    );

    expect(screen.getByRole("button", { name: "Connect" })).toHaveClass("kp-button--primary");
    expect(screen.getByRole("textbox", { name: "Executable path" })).toBeInTheDocument();
    expect(screen.getByRole("combobox", { name: "Relay region" })).toBeInTheDocument();
    expect(screen.getByText("Connected").closest(".kp-status-badge")).toHaveClass(
      "kp-status-badge--on"
    );
    expect(screen.getByRole("button", { name: "Routing toggle on" })).toHaveClass("kp-primary-toggle--on");
    expect(screen.getByText("Routing Active")).toBeInTheDocument();
    expect(screen.getByLabelText("Ping metrics")).toBeInTheDocument();
    expect(screen.getByText("sin-01.relay.local")).toBeInTheDocument();
    expect(screen.getByText("Final Fantasy XIV")).toBeInTheDocument();
  });

  it("covers keyboard interactions for toggle-adjacent flows modal and navigation", () => {
    const onClose = vi.fn();
    const onStepSelect = vi.fn();

    render(
      <div>
        <PrimaryToggle state="off" />
        <Modal open title="Disconnect?" onClose={onClose}>
          <button type="button">Retry</button>
          <button type="button">Continue</button>
        </Modal>
        <OnboardingStepper
          steps={[
            { id: "welcome", label: "Welcome", state: "completed" },
            { id: "permission", label: "Permission", state: "active" },
            { id: "relay-test", label: "Relay Test", state: "inactive" }
          ]}
          onStepSelect={onStepSelect}
        />
        <AppShell>
          <div>Shell baseline content</div>
        </AppShell>
      </div>
    );

    const toggle = screen.getByRole("button", { name: "Routing toggle off" });
    expect(toggle).toHaveAttribute("aria-pressed", "false");

    const dialog = screen.getByRole("dialog", { name: "Disconnect?" });
    fireEvent.keyDown(dialog, { key: "Escape" });
    expect(onClose).toHaveBeenCalledTimes(1);

    const stepButtons = screen.getAllByRole("button", { name: /Welcome|Permission|Relay Test/i });
    fireEvent.keyDown(stepButtons[1], { key: "ArrowRight" });
    expect(onStepSelect).toHaveBeenCalledWith("relay-test");
    fireEvent.keyDown(stepButtons[1], { key: "ArrowLeft" });
    expect(onStepSelect).toHaveBeenCalledWith("welcome");

    const dashboardButton = screen.getByRole("button", { name: "Dashboard" });
    const routingButton = screen.getByRole("button", { name: "Routing" });
    dashboardButton.focus();
    fireEvent.keyDown(dashboardButton, { key: "ArrowDown" });
    expect(routingButton).toHaveFocus();
    fireEvent.keyDown(routingButton, { key: "ArrowUp" });
    expect(dashboardButton).toHaveFocus();
  });

  it("guards semantic status and health mappings against class regressions", () => {
    render(
      <div>
        <ConnectionStatusBadge state="off" />
        <ConnectionStatusBadge state="connecting" />
        <ConnectionStatusBadge state="on" />
        <ConnectionStatusBadge state="degraded" />
        <div role="list">
          <RelayHealthListItem hostname="sin-01" latencyMs={35} region="sin" health="ok" />
          <RelayHealthListItem hostname="nrt-01" latencyMs={80} region="nrt" health="warn" />
          <RelayHealthListItem hostname="lax-01" latencyMs={null} region="lax" health="dead" />
        </div>
      </div>
    );

    const statusBadges = screen.getAllByRole("status");
    expect(statusBadges[0]).toHaveClass("kp-connection-badge--off");
    expect(statusBadges[1]).toHaveClass("kp-connection-badge--connecting");
    expect(statusBadges[2]).toHaveClass("kp-connection-badge--on");
    expect(statusBadges[3]).toHaveClass("kp-connection-badge--degraded");

    const relayItems = screen.getAllByRole("listitem");
    expect(relayItems[0]).toHaveClass("kp-relay-item--ok");
    expect(relayItems[1]).toHaveClass("kp-relay-item--warn");
    expect(relayItems[2]).toHaveClass("kp-relay-item--dead");
  });
});
