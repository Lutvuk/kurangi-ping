import { useMemo } from "react";
import {
  ConnectionStatusBadge,
  type ConnectionStatusState,
  RelayHealthListItem
} from "../../components/modules";
import { Card } from "../../components/primitives";
import { FailoverStatusNotice } from "./FailoverStatusNotice";
import type { RelayFailoverViewModel, RelayFailoverState, RelayHealthViewModel } from "./model";
import "./RelayHealthPanel.css";

export type RelayHealthPanelProps = {
  relays: RelayHealthViewModel[];
  failover: RelayFailoverViewModel;
  activeRelayId?: string | null;
  title?: string;
};

function resolveBadgeState(failoverState: RelayFailoverState): ConnectionStatusState {
  switch (failoverState) {
    case "stable":
      return "on";
    case "switching":
      return "connecting";
    case "recovered":
      return "on";
    case "failed":
      return "error";
    default:
      return "degraded";
  }
}

function resolveBadgeLabel(failoverState: RelayFailoverState): string {
  switch (failoverState) {
    case "stable":
      return "Relay Stable";
    case "switching":
      return "Switching Relay...";
    case "recovered":
      return "Relay Recovered";
    case "failed":
      return "No Relay Available";
    default:
      return "Relay Status Unknown";
  }
}

function resolveActiveRelayId(
  explicitActiveRelayId: string | null | undefined,
  failover: RelayFailoverViewModel
): string | null {
  if (explicitActiveRelayId) {
    return explicitActiveRelayId;
  }

  switch (failover.currentState) {
    case "switching":
      return failover.nextRelayId ?? failover.previousRelayId ?? null;
    case "recovered":
      return failover.nextRelayId ?? failover.previousRelayId ?? null;
    case "failed":
      return failover.previousRelayId ?? null;
    case "stable":
    default:
      return failover.nextRelayId ?? failover.previousRelayId ?? null;
  }
}

export function RelayHealthPanel({
  relays,
  failover,
  activeRelayId,
  title = "Relay Health"
}: RelayHealthPanelProps) {
  const badgeState = resolveBadgeState(failover.currentState);
  const badgeLabel = resolveBadgeLabel(failover.currentState);
  const activeId = useMemo(
    () => resolveActiveRelayId(activeRelayId, failover),
    [activeRelayId, failover]
  );

  return (
    <Card as="section" className="kp-relay-health-panel" aria-label="Relay health panel">
      <header className="kp-relay-health-panel-header">
        <h2 className="kp-relay-health-panel-title">{title}</h2>
        <ConnectionStatusBadge state={badgeState} label={badgeLabel} />
      </header>

      <p className="kp-relay-health-panel-copy">
        {failover.previousRelayId ? `Previous: ${failover.previousRelayId}` : "Previous: none"} •{" "}
        {failover.nextRelayId ? `Next: ${failover.nextRelayId}` : "Next: none"}
      </p>
      <FailoverStatusNotice failover={failover} />

      <div role="list" className="kp-relay-health-panel-list">
        {relays.map((relay) => (
          <RelayHealthListItem
            key={relay.relayId}
            hostname={relay.hostname}
            latencyMs={relay.latencyMs}
            region={relay.region}
            health={relay.health}
            active={activeId === relay.relayId}
            data-relay-id={relay.relayId}
          />
        ))}
      </div>
    </Card>
  );
}
