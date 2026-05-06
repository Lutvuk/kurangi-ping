param(
  [Parameter(Mandatory = $true)]
  [ValidateSet("stable", "beta")]
  [string]$Channel,
  [Parameter(Mandatory = $true)]
  [string]$ArtifactPath,
  [string]$SignaturePath = "",
  [ValidateSet("allow_unsigned", "require_signed")]
  [string]$BetaSigningPolicy = "allow_unsigned",
  [string]$ExpectedSigner = "",
  [string]$LogPath = (Join-Path $PSScriptRoot "logs/ci-signing-verify.log")
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Write-GateLog {
  param(
    [Parameter(Mandatory = $true)]
    [string]$Message
  )

  $line = "[{0}] {1}" -f (Get-Date).ToString("yyyy-MM-ddTHH:mm:ssK"), $Message
  $line | Tee-Object -FilePath $LogPath -Append | Out-Host
}

$logDir = Split-Path -Parent $LogPath
if (-not (Test-Path -LiteralPath $logDir)) {
  New-Item -ItemType Directory -Force -Path $logDir | Out-Null
}
if (Test-Path -LiteralPath $LogPath) {
  Remove-Item -LiteralPath $LogPath -Force
}

$resolvedArtifactPath = (Resolve-Path -LiteralPath $ArtifactPath).Path
if (-not (Test-Path -LiteralPath $resolvedArtifactPath -PathType Leaf)) {
  throw "Artifact not found: $ArtifactPath"
}

$artifactFileName = Split-Path -Leaf $resolvedArtifactPath
$artifactHash = (Get-FileHash -LiteralPath $resolvedArtifactPath -Algorithm SHA256).Hash.ToLowerInvariant()
$requiresSignature = $Channel -eq "stable" -or ($Channel -eq "beta" -and $BetaSigningPolicy -eq "require_signed")
$effectiveSignaturePath = $SignaturePath

Write-GateLog "SIGNING_GATE_CHANNEL=$Channel"
Write-GateLog "SIGNING_GATE_BETA_POLICY=$BetaSigningPolicy"
Write-GateLog "SIGNING_GATE_REQUIRE_SIGNATURE=$requiresSignature"
Write-GateLog "SIGNING_GATE_ARTIFACT=$resolvedArtifactPath"
Write-GateLog "SIGNING_GATE_ARTIFACT_SHA256=$artifactHash"

if ([string]::IsNullOrWhiteSpace($effectiveSignaturePath)) {
  $effectiveSignaturePath = "$resolvedArtifactPath.sig.json"
}

if (-not (Test-Path -LiteralPath $effectiveSignaturePath -PathType Leaf)) {
  Write-GateLog "SIGNING_GATE_SIGNATURE_PATH=$effectiveSignaturePath"
  Write-GateLog "SIGNING_GATE_SIGNATURE_PRESENT=false"
  if ($requiresSignature) {
    Write-GateLog "SIGNING_GATE_RESULT=fail"
    Write-GateLog "SIGNING_GATE_REASON=signature_missing"
    throw "Signing verification failed: signature is required but missing."
  }

  Write-GateLog "SIGNING_GATE_RESULT=pass"
  Write-GateLog "SIGNING_GATE_REASON=unsigned_allowed_by_policy"
  return
}

Write-GateLog "SIGNING_GATE_SIGNATURE_PATH=$effectiveSignaturePath"
Write-GateLog "SIGNING_GATE_SIGNATURE_PRESENT=true"

try {
  $signatureDocument = Get-Content -LiteralPath $effectiveSignaturePath -Raw | ConvertFrom-Json
}
catch {
  Write-GateLog "SIGNING_GATE_RESULT=fail"
  Write-GateLog "SIGNING_GATE_REASON=signature_parse_failed"
  throw "Signing verification failed: signature file is not valid JSON."
}

$signatureArtifactName = [string]$signatureDocument.artifactName
$signatureSha256 = [string]$signatureDocument.sha256
$signatureSigner = [string]$signatureDocument.signer

if ([string]::IsNullOrWhiteSpace($signatureArtifactName) -or [string]::IsNullOrWhiteSpace($signatureSha256)) {
  Write-GateLog "SIGNING_GATE_RESULT=fail"
  Write-GateLog "SIGNING_GATE_REASON=signature_fields_missing"
  throw "Signing verification failed: signature file is missing required fields."
}

Write-GateLog "SIGNING_GATE_SIGNATURE_ARTIFACT=$signatureArtifactName"
Write-GateLog "SIGNING_GATE_SIGNATURE_SHA256=$signatureSha256"
Write-GateLog "SIGNING_GATE_SIGNATURE_SIGNER=$signatureSigner"

if ($signatureArtifactName -ne $artifactFileName) {
  Write-GateLog "SIGNING_GATE_RESULT=fail"
  Write-GateLog "SIGNING_GATE_REASON=artifact_name_mismatch"
  throw "Signing verification failed: signature artifact name does not match."
}

if ($signatureSha256.ToLowerInvariant() -ne $artifactHash) {
  Write-GateLog "SIGNING_GATE_RESULT=fail"
  Write-GateLog "SIGNING_GATE_REASON=artifact_hash_mismatch"
  throw "Signing verification failed: artifact hash mismatch."
}

if (-not [string]::IsNullOrWhiteSpace($ExpectedSigner) -and $signatureSigner -ne $ExpectedSigner) {
  Write-GateLog "SIGNING_GATE_EXPECTED_SIGNER=$ExpectedSigner"
  Write-GateLog "SIGNING_GATE_RESULT=fail"
  Write-GateLog "SIGNING_GATE_REASON=signer_mismatch"
  throw "Signing verification failed: signer identity mismatch."
}

Write-GateLog "SIGNING_GATE_RESULT=pass"
Write-GateLog "SIGNING_GATE_REASON=signature_verified"
