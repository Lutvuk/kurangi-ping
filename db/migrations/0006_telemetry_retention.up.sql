CREATE INDEX IF NOT EXISTS idx_telemetry_batch_prune_expiry_status
  ON telemetry_batch(expires_at, expired_at, delivery_status);

CREATE INDEX IF NOT EXISTS idx_telemetry_batch_prune_next_retry
  ON telemetry_batch(next_retry_at, delivery_status, created_at);

CREATE INDEX IF NOT EXISTS idx_telemetry_event_prune_dropped_at
  ON telemetry_event(dropped_at, occurred_at);
