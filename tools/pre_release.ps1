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
  [switch]$SkipPortableTime,

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
  "python" `
  @(
    (Join-Path $repoRoot "tools/check_adr_numbers.py")
  )

Invoke-Checked `
  "Workspace layering policy" `
  "python" `
  @(
    (Join-Path $repoRoot "tools/check_layering.py")
  )

Invoke-Checked `
  "Execution surface policy" `
  "python" `
  @(
    (Join-Path $repoRoot "tools/check_execution_surface.py")
  )

Invoke-Checked `
  "Stringly command parsing policy" `
  "python" `
  @(
      (Join-Path $repoRoot "tools/check_stringly_command_parsing.py")
  )

Invoke-Checked `
  "Teaching surfaces policy (prefer action helpers)" `
  "python" `
  @(
    (Join-Path $repoRoot "tools/gate_no_on_action_in_teaching_surfaces.py")
  )

Invoke-Checked `
  "Teaching surfaces policy (no verbose models_mut action handlers)" `
  "python" `
  @(
    (Join-Path $repoRoot "tools/gate_no_models_mut_in_action_handlers.py")
  )

Invoke-Checked `
  "Teaching surfaces policy (only approved advanced on_action_notify cases)" `
  "python" `
  @(
    (Join-Path $repoRoot "tools/gate_only_allowed_on_action_notify_in_teaching_surfaces.py")
  )

Invoke-Checked `
  "Teaching surfaces policy (no legacy stack helpers)" `
  "python" `
  @(
    (Join-Path $repoRoot "tools/gate_no_stack_in_cookbook.py"),
    (Join-Path $repoRoot "tools/gate_no_stack_in_examples.py"),
    (Join-Path $repoRoot "tools/gate_no_public_stack_in_ui_kit.py")
  )

if (-not $SkipPortableTime) {
  Invoke-Checked `
    "Portable time sources (prefer fret_core::time::Instant)" `
    "python" `
    @(
      (Join-Path $repoRoot "tools/check_portable_time.py")
    )
}

if (-not $SkipReleaseClosure) {
  Invoke-Checked `
    "Release closure check" `
    "python" `
    @(
      (Join-Path $repoRoot "tools/release_closure_check.py"),
      "--config",
      "release-plz.toml",
      "--write-order",
      "docs/release/v0.1.0-publish-order.txt"
    )
}

if (-not $SkipIcons) {
  $iconArgs = @((Join-Path $repoRoot "tools/pre_release_icons.py"))
  if ($SkipDiffCheck) {
    $iconArgs += "--skip-diff-check"
  }

  Invoke-Checked "icons checks" "python" $iconArgs
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
