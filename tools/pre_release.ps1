param(
  [Parameter(Mandatory = $false)]
  [switch]$SkipIcons,

  [Parameter(Mandatory = $false)]
  [switch]$SkipDiffCheck
)

$ErrorActionPreference = "Stop"

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

if (-not $SkipIcons) {
  $iconArgs = @(
    "-NoProfile",
    "-ExecutionPolicy",
    "Bypass",
    "-File",
    (Join-Path $repoRoot "tools/pre_release_icons.ps1")
  )
  if ($SkipDiffCheck) {
    $iconArgs += "-SkipDiffCheck"
  }

  Invoke-Checked "icons checks" "powershell.exe" $iconArgs
}

Write-Host "[pre-release] done"

