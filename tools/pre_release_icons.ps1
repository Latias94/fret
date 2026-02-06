param(
  [Parameter(Mandatory = $false)]
  [switch]$SkipDiffCheck
)

$ErrorActionPreference = "Stop"

function Invoke-Checked(
  [string]$Name,
  [string]$Program,
  [string[]]$Arguments
) {
  Write-Host "[pre-release/icons] $Name"
  & $Program @Arguments
  if ($LASTEXITCODE -ne 0) {
    throw "Step failed: $Name (exit code: $LASTEXITCODE)"
  }
}

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")

Invoke-Checked `
  "lucide generate + sync + verify" `
  "powershell.exe" `
  @(
    "-NoProfile",
    "-ExecutionPolicy",
    "Bypass",
    "-File",
    (Join-Path $repoRoot "tools/check_lucide_generation.ps1")
  )

Invoke-Checked `
  "radix generate + sync + verify" `
  "powershell.exe" `
  @(
    "-NoProfile",
    "-ExecutionPolicy",
    "Bypass",
    "-File",
    (Join-Path $repoRoot "tools/check_radix_generation.ps1")
  )

if (-not $SkipDiffCheck) {
  Invoke-Checked `
    "diff check icon-related paths" `
    "git" `
    @(
      "diff",
      "--exit-code",
      "--",
      "ecosystem/fret-icons-lucide",
      "ecosystem/fret-icons-radix",
      "tools",
      ".gitmodules",
      "third_party/lucide",
      "third_party/radix-icons"
    )
}

Write-Host "[pre-release/icons] done"
