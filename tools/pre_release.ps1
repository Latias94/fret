param(
  [Parameter(Mandatory = $false)]
  [switch]$SkipFmt,

  [Parameter(Mandatory = $false)]
  [switch]$SkipClippy,

  [Parameter(Mandatory = $false)]
  [switch]$SkipNextest,

  [Parameter(Mandatory = $false)]
  [switch]$SkipIcons,

  [Parameter(Mandatory = $false)]
  [switch]$SkipReleaseClosure,

  [Parameter(Mandatory = $false)]
  [switch]$SkipDiffCheck
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

function Invoke-Checked(
  [string]$Name,
  [string]$Program,
  [string[]]$Arguments
) {
  Write-Host "[pre-release] $Name"
  & $Program @Arguments
  if ($LASTEXITCODE -ne 0) {
    throw "Step failed: $Name (exit code: $LASTEXITCODE)"
  }
}

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")

Invoke-Checked `
  "ADR ID uniqueness" `
  "pwsh" `
  @(
    "-NoProfile",
    "-File",
    (Join-Path $repoRoot "tools/check_adr_numbers.ps1")
  )

Invoke-Checked `
  "Workspace layering policy" `
  "pwsh" `
  @(
    "-NoProfile",
    "-File",
    (Join-Path $repoRoot "tools/check_layering.ps1")
  )

Invoke-Checked `
  "Execution surface policy" `
  "pwsh" `
  @(
    "-NoProfile",
    "-File",
    (Join-Path $repoRoot "tools/check_execution_surface.ps1")
  )

Invoke-Checked `
  "Stringly command parsing policy" `
  "pwsh" `
  @(
    "-NoProfile",
    "-File",
    (Join-Path $repoRoot "tools/check_stringly_command_parsing.ps1")
  )

if (-not $SkipReleaseClosure) {
  Invoke-Checked `
    "Release closure check" `
    "python" `
    @(
      (Join-Path $repoRoot "tools/release_closure_check.py"),
      "--write-order",
      "docs/release/v0.1.0-publish-order.txt"
    )
}

if (-not $SkipIcons) {
  $iconArgs = @(
    "-NoProfile",
    "-File",
    (Join-Path $repoRoot "tools/pre_release_icons.ps1")
  )
  if ($SkipDiffCheck) {
    $iconArgs += "-SkipDiffCheck"
  }

  Invoke-Checked "icons checks" "pwsh" $iconArgs
}

if (-not $SkipFmt) {
  Invoke-Checked "cargo fmt --check" "cargo" @("fmt", "--all", "--", "--check")
}

if (-not $SkipClippy) {
  Invoke-Checked "cargo clippy (workspace, all targets)" "cargo" @(
    "clippy",
    "--workspace",
    "--all-targets",
    "--",
    "-D",
    "warnings"
  )
}

if (-not $SkipNextest) {
  $nextest = Get-Command cargo-nextest -ErrorAction SilentlyContinue
  if ($null -eq $nextest) {
    Write-Warning "cargo-nextest is not installed; falling back to cargo test --workspace"
    Invoke-Checked "cargo test --workspace" "cargo" @("test", "--workspace")
  } else {
    Invoke-Checked "cargo nextest run" "cargo" @("nextest", "run")
  }
}

Write-Host "[pre-release] done"
