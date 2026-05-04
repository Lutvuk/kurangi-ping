param(
  [string]$DatabasePath = (Join-Path $PSScriptRoot "..\db\kurangi-ping.sqlite"),
  [string]$SeedsPath = (Join-Path $PSScriptRoot "..\db\seeds"),
  [string]$CatalogVersionPath = (Join-Path $PSScriptRoot "..\db\seeds\catalog_version.json")
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
    [Parameter(Mandatory = $true)][string]$SeedsPath,
    [Parameter(Mandatory = $true)][string]$CatalogVersionPath
  )

  Assert-Command python

  $resolvedSeedsPath = (Resolve-Path -LiteralPath $SeedsPath).Path
  $dbDirectory = Split-Path -Parent $DatabasePath
  if (-not (Test-Path -LiteralPath $dbDirectory)) {
    New-Item -ItemType Directory -Force -Path $dbDirectory | Out-Null
  }

  $resolvedCatalogVersionPath = (Resolve-Path -LiteralPath $CatalogVersionPath).Path

  $pythonScript = @'
import json
import pathlib
import sqlite3
import sys

db_path = pathlib.Path(sys.argv[1])
seeds_path = pathlib.Path(sys.argv[2])
catalog_version_path = pathlib.Path(sys.argv[3])

if not seeds_path.exists():
    raise SystemExit(f"Seeds path not found: {seeds_path}")
if not catalog_version_path.exists():
    raise SystemExit(f"Catalog version file not found: {catalog_version_path}")

conn = sqlite3.connect(str(db_path))
conn.execute("PRAGMA foreign_keys = ON")

table_exists = conn.execute(
    "SELECT 1 FROM sqlite_master WHERE type='table' AND name='supported_games'"
).fetchone()
if not table_exists:
    raise SystemExit("supported_games table not found. Run migrations first.")

catalog_meta = json.loads(catalog_version_path.read_text(encoding="utf-8"))
catalog_version = str(catalog_meta.get("version", "")).strip()
if not catalog_version:
    raise SystemExit("catalog_version.json must include a non-empty 'version'.")
catalog_seed_file = str(catalog_meta.get("seed_file", "")).strip()
if not catalog_seed_file:
    raise SystemExit("catalog_version.json must include a non-empty 'seed_file'.")

files = [seeds_path / catalog_seed_file]
if not files[0].exists():
    raise SystemExit(f"Catalog seed file not found: {files[0]}")

conn.execute(
    """
    CREATE TABLE IF NOT EXISTS supported_games_catalog_state (
      catalog_name TEXT PRIMARY KEY,
      catalog_version TEXT NOT NULL CHECK (catalog_version GLOB 'v[0-9]*'),
      source_file TEXT NOT NULL,
      applied_at TEXT NOT NULL CHECK (
        datetime(applied_at) IS NOT NULL
        AND applied_at GLOB '????-??-??T??:??:??*Z'
      )
    )
    """
)

current_state = conn.execute(
    """
    SELECT catalog_version
    FROM supported_games_catalog_state
    WHERE catalog_name = 'supported_games'
    """
).fetchone()
previous_catalog_version = current_state[0] if current_state else None

total_inserted = 0
total_updated = 0
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
        inserted = conn.execute(
            """
            SELECT COUNT(*)
            FROM _seed_supported_games_rows src
            LEFT JOIN supported_games dest ON dest.game_id = src.game_id
            WHERE dest.game_id IS NULL
            """
        ).fetchone()[0]
        updated = conn.execute(
            """
            SELECT COUNT(*)
            FROM _seed_supported_games_rows src
            JOIN supported_games dest ON dest.game_id = src.game_id
            WHERE dest.display_name != src.display_name
               OR dest.executable_name != src.executable_name
               OR dest.enabled != src.enabled
               OR dest.updated_at != src.updated_at
               OR dest.match_mode != 'exact'
               OR dest.catalog_version != ?
            """,
            (catalog_version,),
        ).fetchone()[0]

        conn.execute(
            """
            UPDATE supported_games
            SET
              display_name = (
                SELECT src.display_name
                FROM _seed_supported_games_rows src
                WHERE src.game_id = supported_games.game_id
              ),
              executable_name = (
                SELECT src.executable_name
                FROM _seed_supported_games_rows src
                WHERE src.game_id = supported_games.game_id
              ),
              match_mode = 'exact',
              catalog_version = ?,
              enabled = (
                SELECT src.enabled
                FROM _seed_supported_games_rows src
                WHERE src.game_id = supported_games.game_id
              ),
              updated_at = (
                SELECT src.updated_at
                FROM _seed_supported_games_rows src
                WHERE src.game_id = supported_games.game_id
              )
            WHERE game_id IN (SELECT game_id FROM _seed_supported_games_rows)
              AND (
                display_name != (
                  SELECT src.display_name
                  FROM _seed_supported_games_rows src
                  WHERE src.game_id = supported_games.game_id
                )
                OR executable_name != (
                  SELECT src.executable_name
                  FROM _seed_supported_games_rows src
                  WHERE src.game_id = supported_games.game_id
                )
                OR enabled != (
                  SELECT src.enabled
                  FROM _seed_supported_games_rows src
                  WHERE src.game_id = supported_games.game_id
                )
                OR updated_at != (
                  SELECT src.updated_at
                  FROM _seed_supported_games_rows src
                  WHERE src.game_id = supported_games.game_id
                )
                OR match_mode != 'exact'
                OR catalog_version != ?
              )
            """
            ,
            (catalog_version, catalog_version),
        )
        conn.execute(
            """
            INSERT INTO supported_games (
              game_id,
              display_name,
              executable_name,
              match_mode,
              catalog_version,
              enabled,
              updated_at
            )
            SELECT
              src.game_id,
              src.display_name,
              src.executable_name,
              'exact',
              ?,
              src.enabled,
              src.updated_at
            FROM _seed_supported_games_rows src
            WHERE NOT EXISTS (
              SELECT 1
              FROM supported_games dest
              WHERE dest.game_id = src.game_id
            )
            """
            ,
            (catalog_version,),
        )

        skipped = total_rows - inserted - updated
        total_inserted += inserted
        total_updated += updated
        total_skipped += skipped

        conn.execute("DROP TABLE IF EXISTS temp._seed_supported_games_rows")
        conn.commit()

        print(f"seed={seed_file.name}")
        print(f"seed_inserted={inserted}")
        print(f"seed_updated={updated}")
        print(f"seed_skipped={skipped}")
    except Exception:
        conn.rollback()
        raise

applied_at = conn.execute(
    "SELECT strftime('%Y-%m-%dT%H:%M:%SZ', 'now')"
).fetchone()[0]
conn.execute(
    """
    INSERT INTO supported_games_catalog_state (
      catalog_name, catalog_version, source_file, applied_at
    )
    SELECT 'supported_games', ?, ?, ?
    WHERE NOT EXISTS (
      SELECT 1
      FROM supported_games_catalog_state
      WHERE catalog_name = 'supported_games'
    )
    """,
    (catalog_version, files[-1].name, applied_at),
)
conn.execute(
    """
    UPDATE supported_games_catalog_state
    SET
      catalog_version = ?,
      source_file = ?,
      applied_at = ?
    WHERE catalog_name = 'supported_games'
    """,
    (catalog_version, files[-1].name, applied_at),
)
conn.commit()

print(f"db={db_path}")
print(f"seed_files={len(files)}")
print(f"inserted_count={total_inserted}")
print(f"updated_count={total_updated}")
print(f"skipped_count={total_skipped}")
print(f"catalog_version={catalog_version}")
print(f"catalog_seed_file={catalog_seed_file}")
print(f"catalog_version_changed={1 if previous_catalog_version != catalog_version else 0}")
'@

  $pythonOutput = $pythonScript | python - $DatabasePath $resolvedSeedsPath $resolvedCatalogVersionPath
  if ($LASTEXITCODE -ne 0) {
    throw "Seed runner failed."
  }

  $pythonOutput | ForEach-Object { Write-Output $_ }
}

if ($MyInvocation.InvocationName -ne ".") {
  Invoke-DbSeed -DatabasePath $DatabasePath -SeedsPath $SeedsPath -CatalogVersionPath $CatalogVersionPath
}
