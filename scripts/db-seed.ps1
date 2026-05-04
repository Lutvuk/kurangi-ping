param(
  [string]$DatabasePath = (Join-Path $PSScriptRoot "..\db\kurangi-ping.sqlite"),
  [string]$SeedsPath = (Join-Path $PSScriptRoot "..\db\seeds")
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Assert-Command {
  param([Parameter(Mandatory = $true)][string]$Name)
  if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
    throw "Missing required command: $Name"
  }
}

function Invoke-DbSeed {
  [CmdletBinding()]
  param(
    [Parameter(Mandatory = $true)][string]$DatabasePath,
    [Parameter(Mandatory = $true)][string]$SeedsPath
  )

  Assert-Command python

  $resolvedSeedsPath = (Resolve-Path -LiteralPath $SeedsPath).Path
  $dbDirectory = Split-Path -Parent $DatabasePath
  if (-not (Test-Path -LiteralPath $dbDirectory)) {
    New-Item -ItemType Directory -Force -Path $dbDirectory | Out-Null
  }

  $pythonScript = @'
import pathlib
import sqlite3
import sys

db_path = pathlib.Path(sys.argv[1])
seeds_path = pathlib.Path(sys.argv[2])

if not seeds_path.exists():
    raise SystemExit(f"Seeds path not found: {seeds_path}")

conn = sqlite3.connect(str(db_path))
conn.execute("PRAGMA foreign_keys = ON")

table_exists = conn.execute(
    "SELECT 1 FROM sqlite_master WHERE type='table' AND name='supported_games'"
).fetchone()
if not table_exists:
    raise SystemExit("supported_games table not found. Run migrations first.")

files = sorted(seeds_path.glob("*.sql"), key=lambda p: p.name)
if not files:
    print(f"db={db_path}")
    print("seed_files=0")
    print("inserted_count=0")
    print("skipped_count=0")
    raise SystemExit(0)

total_inserted = 0
total_skipped = 0

for seed_file in files:
    sql = seed_file.read_text(encoding="utf-8")
    try:
        conn.execute("BEGIN")
        conn.executescript(sql)

        temp_exists = conn.execute(
            "SELECT 1 FROM sqlite_temp_master WHERE type='table' AND name='_seed_supported_games_rows'"
        ).fetchone()
        if not temp_exists:
            raise RuntimeError(
                f"{seed_file.name} must define temp table _seed_supported_games_rows"
            )

        invalid_rows = conn.execute(
            """
            SELECT COUNT(*)
            FROM _seed_supported_games_rows
            WHERE game_id != lower(game_id)
               OR executable_name != lower(executable_name)
               OR executable_name NOT LIKE '%.exe'
            """
        ).fetchone()[0]
        if invalid_rows:
            raise RuntimeError(
                f"{seed_file.name} has rows violating allowlist convention (lowercase ids/exe names, .exe extension)"
            )

        total_rows = conn.execute(
            "SELECT COUNT(*) FROM _seed_supported_games_rows"
        ).fetchone()[0]
        before_count = conn.execute(
            """
            SELECT COUNT(*)
            FROM supported_games
            WHERE game_id IN (SELECT game_id FROM _seed_supported_games_rows)
            """
        ).fetchone()[0]

        conn.execute(
            """
            INSERT OR IGNORE INTO supported_games (
              game_id, display_name, executable_name, enabled, updated_at
            )
            SELECT
              game_id, display_name, executable_name, enabled, updated_at
            FROM _seed_supported_games_rows
            """
        )

        after_count = conn.execute(
            """
            SELECT COUNT(*)
            FROM supported_games
            WHERE game_id IN (SELECT game_id FROM _seed_supported_games_rows)
            """
        ).fetchone()[0]

        inserted = after_count - before_count
        skipped = total_rows - inserted
        total_inserted += inserted
        total_skipped += skipped

        conn.execute("DROP TABLE IF EXISTS temp._seed_supported_games_rows")
        conn.commit()

        print(f"seed={seed_file.name}")
        print(f"seed_inserted={inserted}")
        print(f"seed_skipped={skipped}")
    except Exception:
        conn.rollback()
        raise

print(f"db={db_path}")
print(f"seed_files={len(files)}")
print(f"inserted_count={total_inserted}")
print(f"skipped_count={total_skipped}")
'@

  $pythonOutput = $pythonScript | python - $DatabasePath $resolvedSeedsPath
  if ($LASTEXITCODE -ne 0) {
    throw "Seed runner failed."
  }

  $pythonOutput | ForEach-Object { Write-Output $_ }
}

if ($MyInvocation.InvocationName -ne ".") {
  Invoke-DbSeed -DatabasePath $DatabasePath -SeedsPath $SeedsPath
}
