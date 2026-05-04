CREATE TABLE IF NOT EXISTS user_settings (
  installation_id TEXT PRIMARY KEY,
  preferred_region TEXT,
  auto_connect INTEGER NOT NULL,
  update_channel TEXT NOT NULL,
  onboarding_state TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS supported_games (
  game_id TEXT PRIMARY KEY,
  display_name TEXT NOT NULL,
  executable_name TEXT NOT NULL UNIQUE,
  enabled INTEGER NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS relay_manifest (
  manifest_version TEXT PRIMARY KEY,
  signature TEXT NOT NULL,
  fetched_at TEXT NOT NULL,
  valid_until TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS relay_node (
  relay_id TEXT PRIMARY KEY,
  manifest_version TEXT NOT NULL,
  region_code TEXT NOT NULL,
  hostname TEXT NOT NULL,
  priority INTEGER NOT NULL,
  is_active INTEGER NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (manifest_version) REFERENCES relay_manifest(manifest_version)
);

CREATE TABLE IF NOT EXISTS route_session (
  session_id TEXT PRIMARY KEY,
  installation_id TEXT NOT NULL,
  game_id TEXT NOT NULL,
  relay_id TEXT,
  route_protocol TEXT NOT NULL,
  started_at TEXT NOT NULL,
  ended_at TEXT,
  end_reason TEXT,
  FOREIGN KEY (installation_id) REFERENCES user_settings(installation_id),
  FOREIGN KEY (game_id) REFERENCES supported_games(game_id),
  FOREIGN KEY (relay_id) REFERENCES relay_node(relay_id)
);

CREATE TABLE IF NOT EXISTS ping_sample (
  sample_id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL,
  baseline_ping_ms REAL NOT NULL,
  routed_ping_ms REAL,
  jitter_ms REAL,
  packet_loss_pct REAL,
  sampled_at TEXT NOT NULL,
  FOREIGN KEY (session_id) REFERENCES route_session(session_id)
);

CREATE TABLE IF NOT EXISTS telemetry_batch (
  batch_id TEXT PRIMARY KEY,
  created_at TEXT NOT NULL,
  retry_count INTEGER NOT NULL,
  expires_at TEXT NOT NULL,
  delivery_status TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS telemetry_event (
  event_id TEXT PRIMARY KEY,
  batch_id TEXT NOT NULL,
  session_id TEXT,
  event_name TEXT NOT NULL,
  payload_json TEXT NOT NULL,
  occurred_at TEXT NOT NULL,
  FOREIGN KEY (batch_id) REFERENCES telemetry_batch(batch_id),
  FOREIGN KEY (session_id) REFERENCES route_session(session_id)
);
