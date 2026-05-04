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
$engineManifest = Join-Path $repoRoot "crates/client-engine/Cargo.toml"

Assert-Command pnpm
Assert-Command go

$cargoPath = Join-Path $env:USERPROFILE ".cargo\bin"
if (Test-Path -LiteralPath $cargoPath) {
  $env:PATH = "$cargoPath;$env:PATH"
}
Assert-Command cargo

Write-Host "[build] desktop ui build..."
pnpm --dir $desktopDir build

Write-Host "[build] rust client-engine build..."
cargo build --manifest-path $engineManifest

Write-Host "[build] relay-controller go build..."
Push-Location $relayDir
try {
  go build ./... | Out-Host
}
finally {
  Pop-Location
}

Write-Host "[build] all build flows completed."
