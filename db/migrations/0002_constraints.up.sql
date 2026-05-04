PRAGMA foreign_keys = OFF;

BEGIN;

CREATE TABLE user_settings_new (
  installation_id TEXT PRIMARY KEY,
  preferred_region TEXT CHECK (preferred_region IS NULL OR preferred_region IN ('auto', 'sin', 'nrt', 'us')),
  auto_connect INTEGER NOT NULL CHECK (auto_connect IN (0, 1)),
  update_channel TEXT NOT NULL CHECK (update_channel IN ('beta', 'stable')),
  onboarding_state TEXT NOT NULL CHECK (onboarding_state IN ('not_started', 'in_progress', 'completed')),
  updated_at TEXT NOT NULL CHECK (datetime(updated_at) IS NOT NULL AND updated_at GLOB '????-??-??T??:??:??*Z')
);

CREATE TABLE supported_games_new (
  game_id TEXT PRIMARY KEY,
  display_name TEXT NOT NULL,
  executable_name TEXT NOT NULL UNIQUE,
  enabled INTEGER NOT NULL CHECK (enabled IN (0, 1)),
  updated_at TEXT NOT NULL CHECK (datetime(updated_at) IS NOT NULL AND updated_at GLOB '????-??-??T??:??:??*Z')
);

CREATE TABLE relay_manifest_new (
  manifest_version TEXT PRIMARY KEY,
  signature TEXT NOT NULL,
  fetched_at TEXT NOT NULL CHECK (datetime(fetched_at) IS NOT NULL AND fetched_at GLOB '????-??-??T??:??:??*Z'),
  valid_until TEXT NOT NULL CHECK (
    datetime(valid_until) IS NOT NULL
    AND valid_until GLOB '????-??-??T??:??:??*Z'
    AND datetime(valid_until) >= datetime(fetched_at)
  )
);

CREATE TABLE relay_node_new (
  relay_id TEXT PRIMARY KEY,
  manifest_version TEXT NOT NULL,
  region_code TEXT NOT NULL CHECK (region_code IN ('sin', 'nrt', 'us')),
  hostname TEXT NOT NULL,
  priority INTEGER NOT NULL CHECK (priority >= 0),
  is_active INTEGER NOT NULL CHECK (is_active IN (0, 1)),
  updated_at TEXT NOT NULL CHECK (datetime(updated_at) IS NOT NULL AND updated_at GLOB '????-??-??T??:??:??*Z'),
  FOREIGN KEY (manifest_version) REFERENCES relay_manifest(manifest_version)
);

CREATE TABLE route_session_new (
  session_id TEXT PRIMARY KEY,
  installation_id TEXT NOT NULL,
  game_id TEXT NOT NULL,
  relay_id TEXT,
  route_protocol TEXT NOT NULL CHECK (route_protocol IN ('wireguard', 'tcp_tls', 'quic')),
  started_at TEXT NOT NULL CHECK (datetime(started_at) IS NOT NULL AND started_at GLOB '????-??-??T??:??:??*Z'),
  ended_at TEXT CHECK (
    ended_at IS NULL
    OR (
      datetime(ended_at) IS NOT NULL
      AND ended_at GLOB '????-??-??T??:??:??*Z'
      AND datetime(ended_at) >= datetime(started_at)
    )
  ),
  end_reason TEXT,
  FOREIGN KEY (installation_id) REFERENCES user_settings(installation_id),
  FOREIGN KEY (game_id) REFERENCES supported_games(game_id),
  FOREIGN KEY (relay_id) REFERENCES relay_node(relay_id)
);

CREATE TABLE ping_sample_new (
  sample_id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL,
  baseline_ping_ms REAL NOT NULL CHECK (baseline_ping_ms >= 0),
  routed_ping_ms REAL CHECK (routed_ping_ms IS NULL OR routed_ping_ms >= 0),
  jitter_ms REAL CHECK (jitter_ms IS NULL OR jitter_ms >= 0),
  packet_loss_pct REAL CHECK (packet_loss_pct IS NULL OR (packet_loss_pct >= 0 AND packet_loss_pct <= 100)),
  sampled_at TEXT NOT NULL CHECK (datetime(sampled_at) IS NOT NULL AND sampled_at GLOB '????-??-??T??:??:??*Z'),
  FOREIGN KEY (session_id) REFERENCES route_session(session_id)
);

CREATE TABLE telemetry_batch_new (
  batch_id TEXT PRIMARY KEY,
  created_at TEXT NOT NULL CHECK (datetime(created_at) IS NOT NULL AND created_at GLOB '????-??-??T??:??:??*Z'),
  retry_count INTEGER NOT NULL CHECK (retry_count >= 0 AND retry_count <= 10),
  expires_at TEXT NOT NULL CHECK (
    datetime(expires_at) IS NOT NULL
    AND expires_at GLOB '????-??-??T??:??:??*Z'
    AND datetime(expires_at) >= datetime(created_at)
  ),
  delivery_status TEXT NOT NULL CHECK (delivery_status IN ('queued', 'sent', 'failed', 'expired'))
);

CREATE TABLE telemetry_event_new (
  event_id TEXT PRIMARY KEY,
  batch_id TEXT NOT NULL,
  session_id TEXT,
  event_name TEXT NOT NULL CHECK (
    event_name IN (
      'app_opened',
      'game_detected',
      'routing_enabled',
      'ping_measured',
      'routing_disabled',
      'crash_reported',
      'relay_failed',
      'onboarding_completed'
    )
  ),
  payload_json TEXT NOT NULL,
  occurred_at TEXT NOT NULL CHECK (datetime(occurred_at) IS NOT NULL AND occurred_at GLOB '????-??-??T??:??:??*Z'),
  FOREIGN KEY (batch_id) REFERENCES telemetry_batch(batch_id),
  FOREIGN KEY (session_id) REFERENCES route_session(session_id)
);

INSERT INTO user_settings_new (
  installation_id, preferred_region, auto_connect, update_channel, onboarding_state, updated_at
)
SELECT
  installation_id, preferred_region, auto_connect, update_channel, onboarding_state, updated_at
FROM user_settings;

INSERT INTO supported_games_new (
  game_id, display_name, executable_name, enabled, updated_at
)
SELECT
  game_id, display_name, executable_name, enabled, updated_at
FROM supported_games;

INSERT INTO relay_manifest_new (
  manifest_version, signature, fetched_at, valid_until
)
SELECT
  manifest_version, signature, fetched_at, valid_until
FROM relay_manifest;

INSERT INTO relay_node_new (
  relay_id, manifest_version, region_code, hostname, priority, is_active, updated_at
)
SELECT
  relay_id, manifest_version, region_code, hostname, priority, is_active, updated_at
FROM relay_node;

INSERT INTO route_session_new (
  session_id, installation_id, game_id, relay_id, route_protocol, started_at, ended_at, end_reason
)
SELECT
  session_id, installation_id, game_id, relay_id, route_protocol, started_at, ended_at, end_reason
FROM route_session;

INSERT INTO ping_sample_new (
  sample_id, session_id, baseline_ping_ms, routed_ping_ms, jitter_ms, packet_loss_pct, sampled_at
)
SELECT
  sample_id, session_id, baseline_ping_ms, routed_ping_ms, jitter_ms, packet_loss_pct, sampled_at
FROM ping_sample;

INSERT INTO telemetry_batch_new (
  batch_id, created_at, retry_count, expires_at, delivery_status
)
SELECT
  batch_id, created_at, retry_count, expires_at, delivery_status
FROM telemetry_batch;

INSERT INTO telemetry_event_new (
  event_id, batch_id, session_id, event_name, payload_json, occurred_at
)
SELECT
  event_id, batch_id, session_id, event_name, payload_json, occurred_at
FROM telemetry_event;

DROP TABLE telemetry_event;
DROP TABLE ping_sample;
DROP TABLE route_session;
DROP TABLE relay_node;
DROP TABLE telemetry_batch;
DROP TABLE relay_manifest;
DROP TABLE supported_games;
DROP TABLE user_settings;

ALTER TABLE user_settings_new RENAME TO user_settings;
ALTER TABLE supported_games_new RENAME TO supported_games;
ALTER TABLE relay_manifest_new RENAME TO relay_manifest;
ALTER TABLE relay_node_new RENAME TO relay_node;
ALTER TABLE route_session_new RENAME TO route_session;
ALTER TABLE ping_sample_new RENAME TO ping_sample;
ALTER TABLE telemetry_batch_new RENAME TO telemetry_batch;
ALTER TABLE telemetry_event_new RENAME TO telemetry_event;

COMMIT;

PRAGMA foreign_keys = ON;
