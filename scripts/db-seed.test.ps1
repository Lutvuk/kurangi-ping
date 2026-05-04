Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$migrateRunner = Join-Path $PSScriptRoot "db-migrate.ps1"
$seedRunner = Join-Path $PSScriptRoot "db-seed.ps1"
$migrationsPath = Join-Path $repoRoot "db\migrations"
$seedsPath = Join-Path $repoRoot "db\seeds"
$tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("kp-seed-test-" + [guid]::NewGuid().ToString("N"))
$dbPath = Join-Path $tempRoot "seed.sqlite"

New-Item -ItemType Directory -Force -Path $tempRoot | Out-Null

try {
  & $migrateRunner -DatabasePath $dbPath -MigrationsPath $migrationsPath

  $firstRun = & $seedRunner -DatabasePath $dbPath -SeedsPath $seedsPath | Out-String

  if ($firstRun -notmatch "seed_files=1") {
    throw "Expected version-aware seed runner to apply one catalog seed file."
  }
  if ($firstRun -notmatch "inserted_count=5") {
    throw "Expected first seed run inserted_count=5"
  }
  if ($firstRun -notmatch "updated_count=0") {
    throw "Expected first seed run updated_count=0"
  }
  if ($firstRun -notmatch "skipped_count=0") {
    throw "Expected first seed run skipped_count=0"
  }
  if ($firstRun -notmatch "catalog_version=v2") {
    throw "Expected first seed run to report catalog version v2"
  }
  if ($firstRun -notmatch "catalog_seed_file=0002_supported_games_catalog.sql") {
    throw "Expected first seed run to report selected catalog seed file"
  }

  $mutateForUpdate = @'
import sqlite3
import sys

db = sqlite3.connect(sys.argv[1])
db.execute(
    """
    UPDATE supported_games
    SET enabled = 1, updated_at = '2026-05-01T00:00:00Z', catalog_version = 'v1'
    WHERE game_id = 'swtor'
    """
)
db.commit()
'@

  $mutateForUpdate | python - $dbPath | Out-Null
  if ($LASTEXITCODE -ne 0) {
    throw "failed to prepare update-path seed scenario"
  }

  $secondRun = & $seedRunner -DatabasePath $dbPath -SeedsPath $seedsPath | Out-String
  $thirdRun = & $seedRunner -DatabasePath $dbPath -SeedsPath $seedsPath | Out-String

  if ($secondRun -notmatch "inserted_count=0") {
    throw "Expected second seed run inserted_count=0"
  }
  if ($secondRun -notmatch "updated_count=1") {
    throw "Expected second seed run updated_count=1"
  }
  if ($secondRun -notmatch "skipped_count=4") {
    throw "Expected second seed run skipped_count=4"
  }
  if ($thirdRun -notmatch "inserted_count=0") {
    throw "Expected third seed run inserted_count=0"
  }
  if ($thirdRun -notmatch "updated_count=0") {
    throw "Expected third seed run updated_count=0"
  }
  if ($thirdRun -notmatch "skipped_count=5") {
    throw "Expected third seed run skipped_count=5"
  }

  $pythonCheck = @'
import sqlite3
import sys

db = sqlite3.connect(sys.argv[1])
count = db.execute("SELECT COUNT(*) FROM supported_games").fetchone()[0]
ids = [row[0] for row in db.execute("SELECT game_id FROM supported_games ORDER BY game_id")]
mode_versions = db.execute(
    "SELECT COUNT(*) FROM supported_games WHERE match_mode = 'exact' AND catalog_version = 'v2'"
).fetchone()[0]
swtor_enabled = db.execute(
    "SELECT enabled FROM supported_games WHERE game_id = 'swtor'"
).fetchone()[0]
catalog_state = db.execute(
    """
    SELECT catalog_version, source_file
    FROM supported_games_catalog_state
    WHERE catalog_name = 'supported_games'
    """
).fetchone()
if count != 5:
    raise SystemExit(f"expected 5 supported_games rows, got {count}")
expected = ["ffxiv", "gta_online", "swtor", "valorant", "wow"]
if ids != expected:
    raise SystemExit(f"unexpected game_id set: {ids}")
if mode_versions != 5:
    raise SystemExit(f"expected 5 rows with exact/v2 catalog metadata, got {mode_versions}")
if swtor_enabled != 0:
    raise SystemExit(f"expected swtor enabled=0 from catalog, got {swtor_enabled}")
if catalog_state is None:
    raise SystemExit("expected supported_games_catalog_state row")
if catalog_state[0] != "v2":
    raise SystemExit(f"expected catalog state version v2, got {catalog_state[0]}")
if catalog_state[1] != "0002_supported_games_catalog.sql":
    raise SystemExit(f"expected catalog state source file 0002_supported_games_catalog.sql, got {catalog_state[1]}")
print("db_seed_test=pass")
'@

  $pythonCheck | python - $dbPath | ForEach-Object { Write-Host $_ }
  if ($LASTEXITCODE -ne 0) {
    throw "db-seed data check failed."
  }
}
finally {
  if (Test-Path -LiteralPath $tempRoot) {
    Remove-Item -LiteralPath $tempRoot -Recurse -Force
  }
}
