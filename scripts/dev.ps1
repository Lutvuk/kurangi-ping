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
$logsDir = Join-Path $PSScriptRoot "logs"
New-Item -ItemType Directory -Force -Path $logsDir | Out-Null

Assert-Command pnpm
Assert-Command go

$cargoPath = Join-Path $env:USERPROFILE ".cargo\bin"
if (Test-Path -LiteralPath $cargoPath) {
  $env:PATH = "$cargoPath;$env:PATH"
}
Assert-Command cargo

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
