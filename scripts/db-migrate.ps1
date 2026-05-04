param(
  [string]$DatabasePath = (Join-Path $PSScriptRoot "..\db\kurangi-ping.sqlite"),
  [string]$MigrationsPath = (Join-Path $PSScriptRoot "..\db\migrations")
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Assert-Command {
  param([Parameter(Mandatory = $true)][string]$Name)
  if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
    throw "Missing required command: $Name"
  }
}

function Invoke-DbMigrate {
  [CmdletBinding()]
  param(
    [Parameter(Mandatory = $true)][string]$DatabasePath,
    [Parameter(Mandatory = $true)][string]$MigrationsPath
  )

  Assert-Command python

  $resolvedMigrationsPath = (Resolve-Path -LiteralPath $MigrationsPath).Path
  $dbDirectory = Split-Path -Parent $DatabasePath
  if (-not (Test-Path -LiteralPath $dbDirectory)) {
    New-Item -ItemType Directory -Force -Path $dbDirectory | Out-Null
  }

  $pythonScript = @'
import pathlib
import sqlite3
import sys

db_path = pathlib.Path(sys.argv[1])
migrations_path = pathlib.Path(sys.argv[2])

if not migrations_path.exists():
    raise SystemExit(f"Migrations path not found: {migrations_path}")

conn = sqlite3.connect(str(db_path))
conn.execute("""
CREATE TABLE IF NOT EXISTS schema_migrations (
  version TEXT PRIMARY KEY,
  filename TEXT NOT NULL,
  applied_at TEXT NOT NULL DEFAULT (datetime('now'))
)
""")

applied = {row[0] for row in conn.execute("SELECT version FROM schema_migrations")}
files = sorted(migrations_path.glob("*.up.sql"), key=lambda p: p.name)

applied_now = []
skipped = []

for migration in files:
    version = migration.name[:-7]
    if version in applied:
        skipped.append(migration.name)
        continue

    sql = migration.read_text(encoding="utf-8")
    try:
        conn.execute("BEGIN")
        conn.executescript(sql)
        conn.execute(
            "INSERT INTO schema_migrations(version, filename, applied_at) VALUES (?, ?, datetime('now'))",
            (version, migration.name),
        )
        conn.commit()
        applied_now.append(migration.name)
    except Exception:
        conn.rollback()
        raise

print(f"db={db_path}")
print(f"applied_count={len(applied_now)}")
print(f"skipped_count={len(skipped)}")
for item in applied_now:
    print(f"applied={item}")
for item in skipped:
    print(f"skipped={item}")
'@

  $pythonOutput = $pythonScript | python - $DatabasePath $resolvedMigrationsPath
  if ($LASTEXITCODE -ne 0) {
    throw "Migration runner failed."
  }

  $pythonOutput | ForEach-Object { Write-Host $_ }
}

if ($MyInvocation.InvocationName -ne ".") {
  Invoke-DbMigrate -DatabasePath $DatabasePath -MigrationsPath $MigrationsPath
}
