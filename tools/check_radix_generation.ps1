param(
  [Parameter(Mandatory = $false)]
  [switch]$SkipSync,

  [Parameter(Mandatory = $false)]
  [switch]$SkipVerify
)

$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")

$args = @(
  "-NoProfile",
  "-ExecutionPolicy",
  "Bypass",
  "-File",
  (Join-Path $repoRoot "tools/check_icons_generation.ps1"),
  "-Pack",
  "radix"
)

if ($SkipSync) {
  $args += "-SkipSync"
}
if ($SkipVerify) {
  $args += "-SkipVerify"
}

& powershell.exe @args
if ($LASTEXITCODE -ne 0) {
  throw "check_icons_generation.ps1 failed for radix (exit code: $LASTEXITCODE)"
}

