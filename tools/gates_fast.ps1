param(
  [Parameter(Mandatory = $false)]
  [switch]$SkipLayering,

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
  Write-Host "[gates-fast] $Name"
  & $Program @Arguments
  if ($LASTEXITCODE -ne 0) {
    throw "Step failed: $Name (exit code: $LASTEXITCODE)"
  }
}

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")

if (-not $SkipLayering) {
  Invoke-Checked `
    "Workspace layering policy" `
    "pwsh" `
    @(
      "-NoProfile",
      "-File",
      (Join-Path $repoRoot "tools/check_layering.ps1")
    )
}

if (-not $SkipFmt) {
  Invoke-Checked "cargo fmt --check" "cargo" @("fmt", "--all", "--", "--check")
}

if (-not $SkipNextest) {
  $nextest = Get-Command cargo-nextest -ErrorAction SilentlyContinue
  $packages = @(
    "fret-core",
    "fret-runtime",
    "fret-ui",
    "fret-ui-kit",
    "fret-runner-winit",
    "fret-ui-shadcn"
  )

  if ($null -eq $nextest) {
    Write-Warning "cargo-nextest is not installed; falling back to cargo test (subset)"
    foreach ($pkg in $packages) {
      Invoke-Checked "cargo test -p $pkg" "cargo" @("test", "-p", $pkg)
    }
  } else {
    foreach ($pkg in $packages) {
      Invoke-Checked "cargo nextest run -p $pkg" "cargo" @("nextest", "run", "-p", $pkg)
    }
  }
}

Write-Host "[gates-fast] done"
