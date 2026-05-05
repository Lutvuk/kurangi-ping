ALTER TABLE telemetry_batch
  ADD COLUMN retry_backoff_ms INTEGER NOT NULL DEFAULT 0
  CHECK (retry_backoff_ms >= 0);

ALTER TABLE telemetry_batch
  ADD COLUMN last_attempt_at TEXT
  CHECK (
    last_attempt_at IS NULL
    OR (
      datetime(last_attempt_at) IS NOT NULL
      AND last_attempt_at GLOB '????-??-??T??:??:??*Z'
    )
  );

ALTER TABLE telemetry_batch
  ADD COLUMN next_retry_at TEXT
  CHECK (
    next_retry_at IS NULL
    OR (
      datetime(next_retry_at) IS NOT NULL
      AND next_retry_at GLOB '????-??-??T??:??:??*Z'
    )
  );

ALTER TABLE telemetry_batch
  ADD COLUMN expired_at TEXT
  CHECK (
    expired_at IS NULL
    OR (
      datetime(expired_at) IS NOT NULL
      AND expired_at GLOB '????-??-??T??:??:??*Z'
    )
  );

ALTER TABLE telemetry_batch
  ADD COLUMN last_error_code TEXT NOT NULL DEFAULT 'none'
  CHECK (
    last_error_code = lower(trim(last_error_code))
    AND length(last_error_code) BETWEEN 4 AND 64
  );

ALTER TABLE telemetry_event
  ADD COLUMN drop_reason_code TEXT NOT NULL DEFAULT 'none'
  CHECK (
    drop_reason_code = lower(trim(drop_reason_code))
    AND length(drop_reason_code) BETWEEN 4 AND 64
  );

ALTER TABLE telemetry_event
  ADD COLUMN dropped_at TEXT
  CHECK (
    dropped_at IS NULL
    OR (
      datetime(dropped_at) IS NOT NULL
      AND dropped_at GLOB '????-??-??T??:??:??*Z'
    )
  );
