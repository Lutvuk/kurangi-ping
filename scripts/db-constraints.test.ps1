Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$runner = Join-Path $PSScriptRoot "db-migrate.ps1"
$migrationsPath = Join-Path $repoRoot "db\migrations"
$tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("kp-constraints-test-" + [guid]::NewGuid().ToString("N"))
$dbPath = Join-Path $tempRoot "constraints.sqlite"

New-Item -ItemType Directory -Force -Path $tempRoot | Out-Null

try {
  & $runner -DatabasePath $dbPath -MigrationsPath $migrationsPath

  $pythonCheck = @'
import sqlite3
import sys

db = sqlite3.connect(sys.argv[1])
db.execute("PRAGMA foreign_keys = ON")

def assert_fails(sql, params, expected):
    try:
        db.execute(sql, params)
        db.commit()
    except sqlite3.IntegrityError as exc:
        message = str(exc)
        if expected not in message:
            raise SystemExit(f"expected error containing '{expected}', got '{message}'")
        db.rollback()
        return
    raise SystemExit(f"expected failure for SQL: {sql}")

db.execute(
    """
    INSERT INTO user_settings(
      installation_id, preferred_region, auto_connect, update_channel, onboarding_state, updated_at
    ) VALUES (?, ?, ?, ?, ?, ?)
    """,
    ("inst-1", "auto", 1, "beta", "not_started", "2026-08-01T00:00:00Z"),
)
db.execute(
    """
    INSERT INTO supported_games(game_id, display_name, executable_name, enabled, updated_at)
    VALUES (?, ?, ?, ?, ?)
    """,
    ("ffxiv", "Final Fantasy XIV", "ffxiv_dx11.exe", 1, "2026-08-01T00:00:00Z"),
)
db.execute(
    """
    INSERT INTO relay_manifest(manifest_version, signature, fetched_at, valid_until)
    VALUES (?, ?, ?, ?)
    """,
    ("v1", "sig", "2026-08-01T00:00:00Z", "2026-08-02T00:00:00Z"),
)
db.execute(
    """
    INSERT INTO relay_node(relay_id, manifest_version, region_code, hostname, priority, is_active, updated_at)
    VALUES (?, ?, ?, ?, ?, ?, ?)
    """,
    ("sin-01", "v1", "sin", "sin-01.relay.local", 0, 1, "2026-08-01T00:00:00Z"),
)
db.execute(
    """
    INSERT INTO route_session(
      session_id, installation_id, game_id, relay_id, route_protocol, started_at, ended_at, end_reason
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
    """,
    ("sess-1", "inst-1", "ffxiv", "sin-01", "wireguard", "2026-08-01T00:00:01Z", None, None),
)
db.execute(
    """
    INSERT INTO telemetry_batch(batch_id, created_at, retry_count, expires_at, delivery_status)
    VALUES (?, ?, ?, ?, ?)
    """,
    ("batch-1", "2026-08-01T00:00:02Z", 0, "2026-08-01T01:00:02Z", "queued"),
)
db.execute(
    """
    INSERT INTO telemetry_event(event_id, batch_id, session_id, event_name, payload_json, occurred_at)
    VALUES (?, ?, ?, ?, ?, ?)
    """,
    ("evt-1", "batch-1", "sess-1", "routing_enabled", "{}", "2026-08-01T00:00:03Z"),
)
db.commit()

assert_fails(
    """
    INSERT INTO route_session(
      session_id, installation_id, game_id, relay_id, route_protocol, started_at
    ) VALUES (?, ?, ?, ?, ?, ?)
    """,
    ("sess-bad-fk", "inst-1", "missing-game", "sin-01", "wireguard", "2026-08-01T00:01:00Z"),
    "FOREIGN KEY constraint failed",
)

assert_fails(
    """
    INSERT INTO user_settings(
      installation_id, preferred_region, auto_connect, update_channel, onboarding_state, updated_at
    ) VALUES (?, ?, ?, ?, ?, ?)
    """,
    ("inst-bad-enum", "auto", 1, "beta", "done", "2026-08-01T00:00:00Z"),
    "CHECK constraint failed",
)

assert_fails(
    """
    INSERT INTO telemetry_batch(batch_id, created_at, retry_count, expires_at, delivery_status)
    VALUES (?, ?, ?, ?, ?)
    """,
    ("batch-bad-retry", "2026-08-01T00:00:02Z", -1, "2026-08-01T01:00:02Z", "queued"),
    "CHECK constraint failed",
)

assert_fails(
    """
    INSERT INTO ping_sample(
      sample_id, session_id, baseline_ping_ms, routed_ping_ms, jitter_ms, packet_loss_pct, sampled_at
    ) VALUES (?, ?, ?, ?, ?, ?, ?)
    """,
    ("sample-bad-loss", "sess-1", 45.0, 30.0, 3.0, 120.0, "2026-08-01T00:00:10Z"),
    "CHECK constraint failed",
)

assert_fails(
    """
    INSERT INTO supported_games(game_id, display_name, executable_name, enabled, updated_at)
    VALUES (?, ?, ?, ?, ?)
    """,
    ("wow", "World of Warcraft", "wow.exe", 1, "2026-08-01 00:00:00"),
    "CHECK constraint failed",
)

assert_fails(
    """
    INSERT INTO supported_games(game_id, display_name, executable_name, enabled, updated_at)
    VALUES (?, ?, ?, ?, ?)
    """,
    ("BadGame", "Bad Game", "badgame.exe", 1, "2026-08-01T00:00:00Z"),
    "CHECK constraint failed",
)

assert_fails(
    """
    INSERT INTO supported_games(game_id, display_name, executable_name, enabled, updated_at)
    VALUES (?, ?, ?, ?, ?)
    """,
    ("badgame2", "Bad Game 2", "games/badgame2.exe", 1, "2026-08-01T00:00:00Z"),
    "CHECK constraint failed",
)

assert_fails(
    """
    INSERT INTO supported_games(
      game_id, display_name, executable_name, match_mode, enabled, updated_at
    ) VALUES (?, ?, ?, ?, ?, ?)
    """,
    ("badgame3", "Bad Game 3", "badgame3.exe", "prefix", 1, "2026-08-01T00:00:00Z"),
    "CHECK constraint failed",
)

assert_fails(
    """
    INSERT INTO supported_games(
      game_id, display_name, executable_name, catalog_version, enabled, updated_at
    ) VALUES (?, ?, ?, ?, ?, ?)
    """,
    ("badgame4", "Bad Game 4", "badgame4.exe", "catalog-1", 1, "2026-08-01T00:00:00Z"),
    "CHECK constraint failed",
)

assert_fails(
    """
    INSERT INTO telemetry_event(event_id, batch_id, session_id, event_name, payload_json, occurred_at)
    VALUES (?, ?, ?, ?, ?, ?)
    """,
    ("evt-bad-event", "batch-1", "sess-1", "custom_event", "{}", "2026-08-01T00:00:04Z"),
    "CHECK constraint failed",
)

print("db_constraints_test=pass")
'@

  $pythonCheck | python - $dbPath | ForEach-Object { Write-Host $_ }
  if ($LASTEXITCODE -ne 0) {
    throw "db-constraints test failed."
  }
}
finally {
  if (Test-Path -LiteralPath $tempRoot) {
    Remove-Item -LiteralPath $tempRoot -Recurse -Force
  }
}
