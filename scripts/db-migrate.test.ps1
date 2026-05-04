Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$runner = Join-Path $PSScriptRoot "db-migrate.ps1"
$tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("kp-migrate-test-" + [guid]::NewGuid().ToString("N"))
$migrationsPath = Join-Path $tempRoot "migrations"
$dbPath = Join-Path $tempRoot "test.sqlite"

New-Item -ItemType Directory -Force -Path $migrationsPath | Out-Null

@'
CREATE TABLE IF NOT EXISTS alpha (
  id INTEGER PRIMARY KEY,
  value TEXT NOT NULL
);
'@ | Set-Content -LiteralPath (Join-Path $migrationsPath "0001_create_alpha.up.sql")

@'
CREATE TABLE IF NOT EXISTS beta (
  id INTEGER PRIMARY KEY,
  alpha_id INTEGER NOT NULL
);
'@ | Set-Content -LiteralPath (Join-Path $migrationsPath "0002_create_beta.up.sql")

try {
  & $runner -DatabasePath $dbPath -MigrationsPath $migrationsPath
  & $runner -DatabasePath $dbPath -MigrationsPath $migrationsPath

  $pythonCheck = @'
import sqlite3
import sys

db = sqlite3.connect(sys.argv[1])
count = db.execute("SELECT COUNT(*) FROM schema_migrations").fetchone()[0]
tables = {row[0] for row in db.execute("SELECT name FROM sqlite_master WHERE type='table'")}
if count != 2:
    raise SystemExit(f"expected 2 applied migrations, got {count}")
if "alpha" not in tables or "beta" not in tables:
    raise SystemExit("expected alpha and beta tables to exist")
print("db_migrate_test=pass")
'@
  $pythonCheck | python - $dbPath | ForEach-Object { Write-Host $_ }
  if ($LASTEXITCODE -ne 0) {
    throw "db-migrate test failed."
  }
}
finally {
  if (Test-Path -LiteralPath $tempRoot) {
    Remove-Item -LiteralPath $tempRoot -Recurse -Force
  }
}
