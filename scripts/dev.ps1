param(
  [switch]$Seed
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Assert-Command {
  param([Parameter(Mandatory = $true)][string]$Name)
  if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
    throw "Missing required command: $Name"
  }
}

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$desktopDir = Join-Path $repoRoot "apps/desktop"
$relayDir = Join-Path $repoRoot "apps/relay-controller"
$dbMigrateScript = Join-Path $PSScriptRoot "db-migrate.ps1"
$dbSeedScript = Join-Path $PSScriptRoot "db-seed.ps1"
$logsDir = Join-Path $PSScriptRoot "logs"
New-Item -ItemType Directory -Force -Path $logsDir | Out-Null

Assert-Command pnpm
Assert-Command go

$cargoPath = Join-Path $env:USERPROFILE ".cargo\bin"
if (Test-Path -LiteralPath $cargoPath) {
  $env:PATH = "$cargoPath;$env:PATH"
}
Assert-Command cargo

$seedFromEnv = $env:KP_DB_SEED_ON_BOOTSTRAP -eq "1"
$shouldSeed = $Seed -or $seedFromEnv

Write-Host "[dev] running local db migrations..."
& $dbMigrateScript

if ($shouldSeed) {
  Write-Host "[dev] running optional supported_games seed..."
  & $dbSeedScript
}
else {
  Write-Host "[dev] skipping seed (use -Seed or KP_DB_SEED_ON_BOOTSTRAP=1 to enable)"
}

$relayPort = if ($env:KP_RELAY_CONTROLLER_PORT) { $env:KP_RELAY_CONTROLLER_PORT } else { "8080" }
$env:KP_RELAY_CONTROLLER_PORT = $relayPort
$relayOutLog = Join-Path $logsDir "relay-controller.out.log"
$relayErrLog = Join-Path $logsDir "relay-controller.err.log"

Write-Host "[dev] starting relay-controller on port $relayPort..."
$relay = Start-Process -FilePath "go" `
  -ArgumentList @("run", "./cmd/server") `
  -WorkingDirectory $relayDir `
  -RedirectStandardOutput $relayOutLog `
  -RedirectStandardError $relayErrLog `
  -WindowStyle Hidden `
  -PassThru

Start-Sleep -Seconds 2
if ($relay.HasExited) {
  throw "relay-controller exited early. Check $relayErrLog"
}

Write-Host "[dev] starting desktop tauri runtime..."
try {
  pnpm --dir $desktopDir tauri:dev
}
finally {
  if (-not $relay.HasExited) {
    Write-Host "[dev] stopping relay-controller (pid=$($relay.Id))..."
    Stop-Process -Id $relay.Id -Force -ErrorAction SilentlyContinue
  }
}
