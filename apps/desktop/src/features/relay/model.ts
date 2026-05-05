import type { RelayHealthState } from "../../components/modules";

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
