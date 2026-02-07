param(
  [Parameter(Mandatory = $false)]
  [ValidateSet("lucide", "radix", "all")]
  [string]$Pack = "all",

  [Parameter(Mandatory = $false)]
  [switch]$SkipSync,

  [Parameter(Mandatory = $false)]
  [switch]$SkipVerify
)

$ErrorActionPreference = "Stop"

function Find-Python() {
  $candidates = @(
    [pscustomobject]@{ Cmd = "python"; Args = @() },
    [pscustomobject]@{ Cmd = "python3"; Args = @() },
    [pscustomobject]@{ Cmd = "py"; Args = @("-3") }
  )

  foreach ($candidate in $candidates) {
    $cmd = Get-Command $candidate.Cmd -ErrorAction SilentlyContinue
    if ($null -ne $cmd) {
      return $candidate
    }
  }

  throw "No Python interpreter found (tried: python, python3, py -3)."
}

function Invoke-Checked(
  [string]$Name,
  [string]$Program,
  [string[]]$Arguments
) {
  Write-Host "[check-icons] $Name"
  & $Program @Arguments
  if ($LASTEXITCODE -ne 0) {
    throw "Step failed: $Name (exit code: $LASTEXITCODE)"
  }
}

function Get-FileDigestOrMissing([string]$Path) {
  if (-not (Test-Path $Path)) {
    return "<missing>"
  }

  return (Get-FileHash -Algorithm SHA256 -Path $Path).Hash
}

function Get-PackGeneratedFiles([string]$PackName) {
  if ($PackName -eq "lucide") {
    return @(
      "ecosystem/fret-icons-lucide/icon-list.txt",
      "ecosystem/fret-icons-lucide/src/generated_ids.rs"
    )
  }

  if ($PackName -eq "radix") {
    return @(
      "ecosystem/fret-icons-radix/icon-list.txt",
      "ecosystem/fret-icons-radix/src/generated_ids.rs"
    )
  }

  throw "Unsupported pack: $PackName"
}

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$python = Find-Python

$packs = @()
if ($Pack -in @("lucide", "all")) {
  $packs += "lucide"
}
if ($Pack -in @("radix", "all")) {
  $packs += "radix"
}

$generatedFiles = @()
foreach ($packName in $packs) {
  $generatedFiles += Get-PackGeneratedFiles $packName
}

$beforeDigests = @{}
foreach ($file in $generatedFiles) {
  $abs = Join-Path $repoRoot $file
  $beforeDigests[$file] = Get-FileDigestOrMissing $abs
}

Invoke-Checked `
  "generate icon-list and generated_ids (pack=$Pack)" `
  $python.Cmd `
  @($python.Args + @((Join-Path $repoRoot "tools/gen_icons.py"), "--pack", $Pack))

if (-not $SkipSync) {
  foreach ($packName in $packs) {
    Invoke-Checked `
      "sync $packName assets" `
      $python.Cmd `
      @($python.Args + @((Join-Path $repoRoot "tools/sync_icons.py"), "--pack", $packName, "--clean"))
  }
}

if (-not $SkipVerify) {
  Invoke-Checked `
    "verify referenced vendor ids" `
    $python.Cmd `
    @($python.Args + @((Join-Path $repoRoot "tools/verify_icons.py"), "--strict"))
}

$changed = @()
foreach ($file in $generatedFiles) {
  $abs = Join-Path $repoRoot $file
  $after = Get-FileDigestOrMissing $abs
  if ($beforeDigests[$file] -ne $after) {
    $changed += $file
  }
}

if ($changed.Count -gt 0) {
  Write-Host "[check-icons] generated files changed after regeneration:"
  foreach ($file in $changed) {
    Write-Host "  - $file"
  }
  throw "Generated icon artifacts are not idempotent. Re-run generation and commit updated outputs."
}

Write-Host "[check-icons] done"

