param(
  [string]$DatabasePath = (Join-Path $PSScriptRoot "..\db\kurangi-ping.sqlite"),
  [string]$NowUtc = ([DateTime]::UtcNow.ToString("yyyy-MM-ddTHH:mm:ssZ"))
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Assert-Command {
  param([Parameter(Mandatory = $true)][string]$Name)
  if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
    throw "Missing required command: $Name"
  }
}

function Invoke-TelemetryPrune {
  [CmdletBinding()]
  param(
    [Parameter(Mandatory = $true)][string]$DatabasePath,
    [Parameter(Mandatory = $true)][string]$NowUtc
  )

  Assert-Command python

  if (-not (Test-Path -LiteralPath $DatabasePath)) {
    throw "Database file not found: $DatabasePath"
  }

  $pythonScript = @'
import sqlite3
import sys

db_path = sys.argv[1]
now_utc = sys.argv[2]

conn = sqlite3.connect(db_path)
conn.execute("PRAGMA foreign_keys = ON")

batch_count_before = conn.execute("SELECT COUNT(*) FROM telemetry_batch").fetchone()[0]
event_count_before = conn.execute("SELECT COUNT(*) FROM telemetry_event").fetchone()[0]

prune_batch_ids = [
    row[0]
    for row in conn.execute(
        """
        SELECT batch_id
        FROM telemetry_batch
        WHERE
          delivery_status = 'expired'
          OR datetime(expires_at) <= datetime(?1)
          OR (
            expired_at IS NOT NULL
            AND datetime(expired_at) <= datetime(?1)
          )
        """,
        (now_utc,),
    )
]

events_deleted_by_batch = 0
batches_deleted = 0
dropped_events_deleted = 0
retryable_batches_preserved = conn.execute(
    """
    SELECT COUNT(*)
    FROM telemetry_batch
    WHERE
      delivery_status IN ('queued', 'failed')
      AND datetime(expires_at) > datetime(?1)
      AND (
        next_retry_at IS NULL
        OR datetime(next_retry_at) > datetime(?1)
      )
    """,
    (now_utc,),
).fetchone()[0]

try:
    conn.execute("BEGIN")

    if prune_batch_ids:
        placeholders = ",".join("?" for _ in prune_batch_ids)
        delete_events_sql = f"DELETE FROM telemetry_event WHERE batch_id IN ({placeholders})"
        cursor = conn.execute(delete_events_sql, prune_batch_ids)
        events_deleted_by_batch = cursor.rowcount

        delete_batches_sql = f"DELETE FROM telemetry_batch WHERE batch_id IN ({placeholders})"
        cursor = conn.execute(delete_batches_sql, prune_batch_ids)
        batches_deleted = cursor.rowcount

    cursor = conn.execute(
        """
        DELETE FROM telemetry_event
        WHERE
          dropped_at IS NOT NULL
          AND datetime(dropped_at) <= datetime(?1)
        """,
        (now_utc,),
    )
    dropped_events_deleted = cursor.rowcount

    conn.commit()
except Exception:
    conn.rollback()
    raise

batch_count_after = conn.execute("SELECT COUNT(*) FROM telemetry_batch").fetchone()[0]
event_count_after = conn.execute("SELECT COUNT(*) FROM telemetry_event").fetchone()[0]

print(f"telemetry_prune_now={now_utc}")
print(f"telemetry_prune_batches_candidate={len(prune_batch_ids)}")
print(f"telemetry_prune_batches_deleted={batches_deleted}")
print(f"telemetry_prune_events_deleted_by_batch={events_deleted_by_batch}")
print(f"telemetry_prune_events_deleted_dropped={dropped_events_deleted}")
print(f"telemetry_prune_batches_before={batch_count_before}")
print(f"telemetry_prune_batches_after={batch_count_after}")
print(f"telemetry_prune_events_before={event_count_before}")
print(f"telemetry_prune_events_after={event_count_after}")
print(f"telemetry_prune_retryable_batches_preserved={retryable_batches_preserved}")
print("telemetry_prune_status=pass")
'@

  $pythonOutput = $pythonScript | python - $DatabasePath $NowUtc
  if ($LASTEXITCODE -ne 0) {
    throw "Telemetry prune failed."
  }

  $pythonOutput | ForEach-Object { Write-Output $_ }
}

if ($MyInvocation.InvocationName -ne ".") {
  Invoke-TelemetryPrune -DatabasePath $DatabasePath -NowUtc $NowUtc
}
