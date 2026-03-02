param(
  [Parameter(Mandatory = $false)]
  [string]$Target = "wasm32-unknown-unknown"
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

function Invoke-Checked(
  [string]$Name,
  [string]$Program,
  [string[]]$Arguments
) {
  Write-Host "[gates-wasm-smoke] $Name"
  & $Program @Arguments
  if ($LASTEXITCODE -ne 0) {
    throw "Step failed: $Name (exit code: $LASTEXITCODE)"
  }
}

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
Push-Location $repoRoot
try {
  Invoke-Checked `
    "cargo check -p fret (no default features) for $Target" `
    "cargo" `
    @("check", "-p", "fret", "--no-default-features", "--target", $Target)
} finally {
  Pop-Location
}

Write-Host "[gates-wasm-smoke] done"
