CREATE INDEX IF NOT EXISTS idx_route_session_installation_started_at
  ON route_session(installation_id, started_at DESC);

CREATE INDEX IF NOT EXISTS idx_route_session_game_started_at
  ON route_session(game_id, started_at DESC);

CREATE INDEX IF NOT EXISTS idx_ping_sample_session_sampled_at
  ON ping_sample(session_id, sampled_at DESC);

CREATE INDEX IF NOT EXISTS idx_telemetry_batch_delivery_retry_expiry_created
  ON telemetry_batch(delivery_status, retry_count, expires_at, created_at);

CREATE INDEX IF NOT EXISTS idx_telemetry_event_batch_occurred_at
  ON telemetry_event(batch_id, occurred_at DESC);

CREATE INDEX IF NOT EXISTS idx_telemetry_event_session_occurred_at
  ON telemetry_event(session_id, occurred_at DESC);
