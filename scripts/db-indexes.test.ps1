Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$runner = Join-Path $PSScriptRoot "db-migrate.ps1"
$migrationsPath = Join-Path $repoRoot "db\migrations"
$tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("kp-indexes-test-" + [guid]::NewGuid().ToString("N"))
$dbPath = Join-Path $tempRoot "indexes.sqlite"

New-Item -ItemType Directory -Force -Path $tempRoot | Out-Null

try {
  & $runner -DatabasePath $dbPath -MigrationsPath $migrationsPath

  $pythonCheck = @'
import sqlite3
import sys

db = sqlite3.connect(sys.argv[1])
db.execute("PRAGMA foreign_keys = ON")

expected_indexes = {
    "idx_supported_games_enabled_executable",
    "idx_route_session_installation_started_at",
    "idx_route_session_game_started_at",
    "idx_ping_sample_session_sampled_at",
    "idx_telemetry_batch_delivery_retry_expiry_created",
    "idx_telemetry_batch_prune_expiry_status",
    "idx_telemetry_batch_prune_next_retry",
    "idx_telemetry_event_batch_occurred_at",
    "idx_telemetry_event_prune_dropped_at",
    "idx_telemetry_event_session_occurred_at",
}

existing = {row[0] for row in db.execute("SELECT name FROM sqlite_master WHERE type='index'")}
missing = sorted(expected_indexes - existing)
if missing:
    raise SystemExit(f"missing indexes: {missing}")

def assert_uses_index(sql, expected_index):
    plan_rows = list(db.execute("EXPLAIN QUERY PLAN " + sql))
    detail_text = " | ".join(row[3] for row in plan_rows)
    if expected_index not in detail_text:
        raise SystemExit(f"query plan does not use {expected_index}: {detail_text}")

assert_uses_index(
    "SELECT game_id FROM supported_games WHERE enabled = 1 ORDER BY executable_name LIMIT 5",
    "idx_supported_games_enabled_executable",
)
assert_uses_index(
    "SELECT session_id FROM route_session WHERE installation_id = 'inst-1' ORDER BY started_at DESC LIMIT 5",
    "idx_route_session_installation_started_at",
)
assert_uses_index(
    "SELECT sample_id FROM ping_sample WHERE session_id = 'sess-1' ORDER BY sampled_at DESC LIMIT 20",
    "idx_ping_sample_session_sampled_at",
)
assert_uses_index(
    "SELECT batch_id FROM telemetry_batch WHERE delivery_status = 'queued' AND retry_count <= 3 AND expires_at > '2026-08-01T00:00:00Z' ORDER BY created_at LIMIT 10",
    "idx_telemetry_batch_delivery_retry_expiry_created",
)
assert_uses_index(
    "SELECT batch_id FROM telemetry_batch WHERE delivery_status = 'expired' OR expires_at <= '2026-08-01T00:00:00Z'",
    "idx_telemetry_batch_prune_expiry_status",
)
assert_uses_index(
    "SELECT event_id FROM telemetry_event WHERE batch_id = 'batch-1' ORDER BY occurred_at DESC LIMIT 50",
    "idx_telemetry_event_batch_occurred_at",
)
assert_uses_index(
    "SELECT event_id FROM telemetry_event WHERE dropped_at IS NOT NULL AND dropped_at <= '2026-08-01T00:00:00Z' ORDER BY occurred_at DESC LIMIT 50",
    "idx_telemetry_event_prune_dropped_at",
)

print("db_indexes_test=pass")
'@

  $pythonCheck | python - $dbPath | ForEach-Object { Write-Host $_ }
  if ($LASTEXITCODE -ne 0) {
    throw "db-indexes test failed."
  }
}
finally {
  if (Test-Path -LiteralPath $tempRoot) {
    Remove-Item -LiteralPath $tempRoot -Recurse -Force
  }
}
