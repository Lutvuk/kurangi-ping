Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$migrateRunner = Join-Path $PSScriptRoot "db-migrate.ps1"
$pruneRunner = Join-Path $PSScriptRoot "db-telemetry-prune.ps1"
$migrationsPath = Join-Path $repoRoot "db\migrations"
$tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("kp-telemetry-prune-test-" + [guid]::NewGuid().ToString("N"))
$dbPath = Join-Path $tempRoot "telemetry-prune.sqlite"
$nowUtc = "2026-08-01T02:00:00Z"

New-Item -ItemType Directory -Force -Path $tempRoot | Out-Null

try {
  & $migrateRunner -DatabasePath $dbPath -MigrationsPath $migrationsPath

  $seedData = @'
import sqlite3
import sys

db = sqlite3.connect(sys.argv[1])
db.execute("PRAGMA foreign_keys = ON")

db.execute(
    """
    INSERT INTO telemetry_batch(
      batch_id,
      created_at,
      retry_count,
      retry_backoff_ms,
      last_attempt_at,
      next_retry_at,
      expires_at,
      expired_at,
      delivery_status,
      last_error_code
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    """,
    (
      "batch-expired-status",
      "2026-08-01T00:00:00Z",
      2,
      5000,
      "2026-08-01T00:30:00Z",
      None,
      "2026-08-01T03:00:00Z",
      "2026-08-01T01:00:00Z",
      "expired",
      "retry_limit_hit"
    ),
)
db.execute(
    """
    INSERT INTO telemetry_batch(
      batch_id, created_at, retry_count, expires_at, delivery_status
    ) VALUES (?, ?, ?, ?, ?)
    """,
    ("batch-expired-time", "2026-08-01T00:00:00Z", 0, "2026-08-01T01:00:00Z", "queued"),
)
db.execute(
    """
    INSERT INTO telemetry_batch(
      batch_id, created_at, retry_count, next_retry_at, expires_at, delivery_status
    ) VALUES (?, ?, ?, ?, ?, ?)
    """,
    (
      "batch-retryable-queued",
      "2026-08-01T01:00:00Z",
      1,
      "2026-08-01T02:10:00Z",
      "2026-08-01T05:00:00Z",
      "queued",
    ),
)
db.execute(
    """
    INSERT INTO telemetry_batch(
      batch_id, created_at, retry_count, next_retry_at, expires_at, delivery_status, last_error_code
    ) VALUES (?, ?, ?, ?, ?, ?, ?)
    """,
    (
      "batch-retryable-failed",
      "2026-08-01T01:10:00Z",
      2,
      "2026-08-01T02:20:00Z",
      "2026-08-01T05:00:00Z",
      "failed",
      "transport_timeout",
    ),
)

db.execute(
    """
    INSERT INTO telemetry_event(
      event_id, batch_id, session_id, event_name, payload_json, occurred_at
    ) VALUES (?, ?, ?, ?, ?, ?)
    """,
    ("evt-1", "batch-expired-status", None, "app_opened", "{}", "2026-08-01T00:01:00Z"),
)
db.execute(
    """
    INSERT INTO telemetry_event(
      event_id, batch_id, session_id, event_name, payload_json, occurred_at
    ) VALUES (?, ?, ?, ?, ?, ?)
    """,
    ("evt-2", "batch-expired-time", None, "routing_enabled", "{}", "2026-08-01T00:02:00Z"),
)
db.execute(
    """
    INSERT INTO telemetry_event(
      event_id, batch_id, session_id, event_name, payload_json, occurred_at
    ) VALUES (?, ?, ?, ?, ?, ?)
    """,
    ("evt-3", "batch-retryable-queued", None, "ping_measured", "{}", "2026-08-01T01:05:00Z"),
)
db.execute(
    """
    INSERT INTO telemetry_event(
      event_id, batch_id, session_id, event_name, payload_json, occurred_at, drop_reason_code, dropped_at
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
    """,
    (
      "evt-4",
      "batch-retryable-queued",
      None,
      "relay_failed",
      "{}",
      "2026-08-01T01:20:00Z",
      "privacy_rejected",
      "2026-08-01T01:30:00Z",
    ),
)
db.execute(
    """
    INSERT INTO telemetry_event(
      event_id, batch_id, session_id, event_name, payload_json, occurred_at
    ) VALUES (?, ?, ?, ?, ?, ?)
    """,
    ("evt-5", "batch-retryable-failed", None, "relay_failed", "{}", "2026-08-01T01:25:00Z"),
)

db.commit()
'@

  $seedData | python - $dbPath | Out-Null
  if ($LASTEXITCODE -ne 0) {
    throw "failed to seed telemetry prune test fixtures"
  }

  $firstRun = & $pruneRunner -DatabasePath $dbPath -NowUtc $nowUtc | Out-String
  if ($firstRun -notmatch "telemetry_prune_batches_deleted=2") {
    throw "Expected first prune run to delete 2 expired telemetry batches."
  }
  if ($firstRun -notmatch "telemetry_prune_events_deleted_by_batch=2") {
    throw "Expected first prune run to delete 2 events through expired batches."
  }
  if ($firstRun -notmatch "telemetry_prune_events_deleted_dropped=1") {
    throw "Expected first prune run to delete 1 dropped telemetry event."
  }
  if ($firstRun -notmatch "telemetry_prune_retryable_batches_preserved=2") {
    throw "Expected prune summary to preserve 2 active/retryable batches."
  }
  if ($firstRun -notmatch "telemetry_prune_status=pass") {
    throw "Expected prune summary status pass."
  }

  $secondRun = & $pruneRunner -DatabasePath $dbPath -NowUtc $nowUtc | Out-String
  if ($secondRun -notmatch "telemetry_prune_batches_deleted=0") {
    throw "Expected second prune run to delete 0 telemetry batches."
  }
  if ($secondRun -notmatch "telemetry_prune_events_deleted_by_batch=0") {
    throw "Expected second prune run to delete 0 telemetry events by batch."
  }
  if ($secondRun -notmatch "telemetry_prune_events_deleted_dropped=0") {
    throw "Expected second prune run to delete 0 dropped telemetry events."
  }

  $verifyData = @'
import sqlite3
import sys

db = sqlite3.connect(sys.argv[1])

batch_ids = [row[0] for row in db.execute("SELECT batch_id FROM telemetry_batch ORDER BY batch_id")]
event_ids = [row[0] for row in db.execute("SELECT event_id FROM telemetry_event ORDER BY event_id")]

if batch_ids != ["batch-retryable-failed", "batch-retryable-queued"]:
    raise SystemExit(f"unexpected batch ids after prune: {batch_ids}")
if event_ids != ["evt-3", "evt-5"]:
    raise SystemExit(f"unexpected event ids after prune: {event_ids}")

print("db_telemetry_prune_test=pass")
'@

  $verifyData | python - $dbPath | ForEach-Object { Write-Host $_ }
  if ($LASTEXITCODE -ne 0) {
    throw "db-telemetry-prune data check failed."
  }
}
finally {
  if (Test-Path -LiteralPath $tempRoot) {
    Remove-Item -LiteralPath $tempRoot -Recurse -Force
  }
}
