import { useMemo } from "react";
import {
  ConnectionStatusBadge,
  type ConnectionStatusState,
  RelayHealthListItem,
  type RelayHealthState
} from "../../components/modules";
import { Card } from "../../components/primitives";
import "./RelayHealthPanel.css";

export type RelayHealthViewModel = {
  relayId: string;
  hostname: string;
  region: string;
  latencyMs: number | null;
  health: RelayHealthState;
};

export type RelayFailoverState = "stable" | "switching" | "recovered" | "failed";

export type RelayFailoverViewModel = {
  currentState: RelayFailoverState;
  previousRelayId?: string | null;
  nextRelayId?: string | null;
  reasonCode?: string;
};

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

function normalizeReason(reasonCode?: string): string {
  if (!reasonCode) {
    return "Status update unavailable";
  }

  return reasonCode
    .split("_")
    .filter(Boolean)
    .map((segment) => segment[0].toUpperCase() + segment.slice(1))
    .join(" ");
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

  const reasonMessage = normalizeReason(failover.reasonCode);

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
      <p className="kp-relay-health-panel-copy kp-relay-health-panel-copy--reason">
        Reason: <code>{reasonMessage}</code>
      </p>

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
