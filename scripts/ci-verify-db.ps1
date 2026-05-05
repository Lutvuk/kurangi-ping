param(
  [string]$DatabasePath = (Join-Path ([System.IO.Path]::GetTempPath()) ("kp-ci-db-" + [guid]::NewGuid().ToString("N") + ".sqlite")),
  [switch]$SkipSeedVerification
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$migrateScript = Join-Path $PSScriptRoot "db-migrate.ps1"
$migrateTestScript = Join-Path $PSScriptRoot "db-migrate.test.ps1"
$constraintTestScript = Join-Path $PSScriptRoot "db-constraints.test.ps1"
$indexTestScript = Join-Path $PSScriptRoot "db-indexes.test.ps1"
$seedScript = Join-Path $PSScriptRoot "db-seed.ps1"
$seedTestScript = Join-Path $PSScriptRoot "db-seed.test.ps1"
$telemetryPruneScript = Join-Path $PSScriptRoot "db-telemetry-prune.ps1"
$telemetryPruneTestScript = Join-Path $PSScriptRoot "db-telemetry-prune.test.ps1"
$migrationsPath = Join-Path $repoRoot "db\migrations"
$seedsPath = Join-Path $repoRoot "db\seeds"
$logsDir = Join-Path $PSScriptRoot "logs"
$logPath = Join-Path $logsDir "ci-db-verify.log"

New-Item -ItemType Directory -Force -Path $logsDir | Out-Null
if (Test-Path -LiteralPath $logPath) {
  Remove-Item -LiteralPath $logPath -Force
}

function Invoke-And-Capture {
  param(
    [Parameter(Mandatory = $true)][string]$Label,
    [Parameter(Mandatory = $true)][scriptblock]$ScriptBlock
  )

  "=== $Label ===" | Tee-Object -FilePath $logPath -Append | Out-Host
  & $ScriptBlock 2>&1 | Tee-Object -FilePath $logPath -Append | Out-Host
  if ($LASTEXITCODE -ne 0) {
    throw "$Label failed with exit code $LASTEXITCODE"
  }
}

if (Test-Path -LiteralPath $DatabasePath) {
  Remove-Item -LiteralPath $DatabasePath -Force
}

Invoke-And-Capture -Label "migrate-fresh-db" -ScriptBlock {
  & $migrateScript -DatabasePath $DatabasePath -MigrationsPath $migrationsPath
}

Invoke-And-Capture -Label "migrate-idempotent-rerun" -ScriptBlock {
  & $migrateScript -DatabasePath $DatabasePath -MigrationsPath $migrationsPath
}

if (-not $SkipSeedVerification) {
  Invoke-And-Capture -Label "seed-first-run" -ScriptBlock {
    & $seedScript -DatabasePath $DatabasePath -SeedsPath $seedsPath
  }

  Invoke-And-Capture -Label "seed-second-run-idempotent" -ScriptBlock {
    & $seedScript -DatabasePath $DatabasePath -SeedsPath $seedsPath
  }
}
else {
  "=== seed-verification-skipped ===" | Tee-Object -FilePath $logPath -Append | Out-Host
}

Invoke-And-Capture -Label "runner-migrate-smoke-test" -ScriptBlock {
  & $migrateTestScript
}

Invoke-And-Capture -Label "constraints-validation-test" -ScriptBlock {
  & $constraintTestScript
}

Invoke-And-Capture -Label "indexes-validation-test" -ScriptBlock {
  & $indexTestScript
}

Invoke-And-Capture -Label "telemetry-prune-dry-run-smoke" -ScriptBlock {
  & $telemetryPruneScript -DatabasePath $DatabasePath
}

Invoke-And-Capture -Label "telemetry-prune-validation-test" -ScriptBlock {
  & $telemetryPruneTestScript
}

if (-not $SkipSeedVerification) {
  Invoke-And-Capture -Label "seed-idempotency-test" -ScriptBlock {
    & $seedTestScript
  }
}

"=== summary ===" | Tee-Object -FilePath $logPath -Append | Out-Host
"db_verification=pass" | Tee-Object -FilePath $logPath -Append | Out-Host
"log_file=$logPath" | Tee-Object -FilePath $logPath -Append | Out-Host
