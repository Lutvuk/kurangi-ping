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
  $secondRun = & $seedRunner -DatabasePath $dbPath -SeedsPath $seedsPath | Out-String

  if ($firstRun -notmatch "inserted_count=5") {
    throw "Expected first seed run inserted_count=5"
  }
  if ($firstRun -notmatch "skipped_count=0") {
    throw "Expected first seed run skipped_count=0"
  }
  if ($secondRun -notmatch "inserted_count=0") {
    throw "Expected second seed run inserted_count=0"
  }
  if ($secondRun -notmatch "skipped_count=5") {
    throw "Expected second seed run skipped_count=5"
  }

  $pythonCheck = @'
import sqlite3
import sys

db = sqlite3.connect(sys.argv[1])
count = db.execute("SELECT COUNT(*) FROM supported_games").fetchone()[0]
ids = [row[0] for row in db.execute("SELECT game_id FROM supported_games ORDER BY game_id")]
mode_versions = db.execute(
    "SELECT COUNT(*) FROM supported_games WHERE match_mode = 'exact' AND catalog_version GLOB 'v[0-9]*'"
).fetchone()[0]
if count != 5:
    raise SystemExit(f"expected 5 supported_games rows, got {count}")
expected = ["ffxiv", "gta_online", "swtor", "valorant", "wow"]
if ids != expected:
    raise SystemExit(f"unexpected game_id set: {ids}")
if mode_versions != 5:
    raise SystemExit(f"expected 5 rows with valid match_mode/catalog_version defaults, got {mode_versions}")
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
