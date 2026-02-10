param(
  [Parameter(Mandatory = $false)]
  [switch]$SkipCheck,

  [Parameter(Mandatory = $false)]
  [switch]$SkipFmt,

  [Parameter(Mandatory = $false)]
  [switch]$SkipNextest
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

function Invoke-Checked(
  [string]$Name,
  [string]$Program,
  [string[]]$Arguments
) {
  Write-Host "[gates-delinea-fast] $Name"
  & $Program @Arguments
  if ($LASTEXITCODE -ne 0) {
    throw "Step failed: $Name (exit code: $LASTEXITCODE)"
  }
}

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$packages = @(
  "delinea",
  "fret-chart",
  "fret-ui-kit"
)

Push-Location $repoRoot
try {
  if (-not $SkipFmt) {
    foreach ($pkg in $packages) {
      Invoke-Checked "cargo fmt -p $pkg -- --check" "cargo" @("fmt", "-p", $pkg, "--", "--check")
    }
  }

  if (-not $SkipCheck) {
    Invoke-Checked `
      "cargo check (subset)" `
      "cargo" `
      @(
        "check",
        "--all-targets",
        "-p",
        "delinea",
        "-p",
        "fret-chart",
        "-p",
        "fret-ui-kit"
      )
  }

  if (-not $SkipNextest) {
    $nextest = Get-Command cargo-nextest -ErrorAction SilentlyContinue
    if ($null -eq $nextest) {
      Write-Warning "cargo-nextest is not installed; falling back to cargo test (subset)"
      foreach ($pkg in $packages) {
        Invoke-Checked "cargo test -p $pkg --tests" "cargo" @("test", "-p", $pkg, "--tests")
      }
    } else {
      Invoke-Checked `
        "cargo nextest run (subset)" `
        "cargo" `
        @(
          "nextest",
          "run",
          "--tests",
          "-p",
          "delinea",
          "-p",
          "fret-chart",
          "-p",
          "fret-ui-kit"
        )
    }
  }

  Write-Host "[gates-delinea-fast] done"
} finally {
  Pop-Location
}

