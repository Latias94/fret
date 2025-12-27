param(
  [Parameter(Mandatory = $false)]
  [ValidateSet("lucide", "radix", "all")]
  [string]$Pack = "all",

  [Parameter(Mandatory = $false)]
  [switch]$Clean
)

$ErrorActionPreference = "Stop"

function Find-Python() {
  $candidates = @(
    [pscustomobject]@{ Cmd = "python"; Args = @() },
    [pscustomobject]@{ Cmd = "python3"; Args = @() },
    [pscustomobject]@{ Cmd = "py"; Args = @("-3") }
  )

  foreach ($c in $candidates) {
    $cmd = Get-Command $c.Cmd -ErrorAction SilentlyContinue
    if ($null -ne $cmd) {
      return $c
    }
  }

  return $null
}

function Assert-Exists([string]$Path) {
  if (-not (Test-Path $Path)) {
    throw "Path not found: $Path"
  }
}

function Sync-Pack(
  [string]$Name,
  [string]$RepoRefDir,
  [string]$ListPath,
  [string]$DestDir,
  [switch]$Clean
) {
  Assert-Exists $RepoRefDir
  Assert-Exists $ListPath
  if (-not (Test-Path $DestDir)) {
    New-Item -ItemType Directory -Path $DestDir | Out-Null
  }

  $items = Get-Content $ListPath | Where-Object { $_ -and (-not $_.StartsWith("#")) } | ForEach-Object { $_.Trim() }
  $keep = @{}
  foreach ($item in $items) {
    $keep[$item] = $true
  }
  foreach ($item in $items) {
    $src = Join-Path $RepoRefDir $item
    $dst = Join-Path $DestDir $item
    Assert-Exists $src
    Copy-Item -Path $src -Destination $dst -Force
  }

  $deleted = 0
  if ($Clean) {
    Get-ChildItem -Path $DestDir -Filter "*.svg" | ForEach-Object {
      if (-not $keep.ContainsKey($_.Name)) {
        Remove-Item -Path $_.FullName -Force
        $deleted++
      }
    }
  }

  Write-Host "Synced $Name icons: $($items.Count) files"
  if ($Clean) {
    Write-Host "Cleaned $Name icons: $deleted files"
  }
}

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")

$py = Find-Python
if ($null -ne $py) {
  $args = @(
    (Join-Path $repoRoot "tools/sync_icons.py"),
    "--pack", $Pack
  )
  if ($Clean) {
    $args += "--clean"
  }

  & $py.Cmd @($py.Args + $args)
  exit $LASTEXITCODE
}

if ($Pack -eq "lucide" -or $Pack -eq "all") {
  Sync-Pack `
    "lucide" `
    (Join-Path $repoRoot "repo-ref/lucide/icons") `
    (Join-Path $repoRoot "crates/fret-icons-lucide/icon-list.txt") `
    (Join-Path $repoRoot "crates/fret-icons-lucide/assets/icons") `
    -Clean:$Clean
}

if ($Pack -eq "radix" -or $Pack -eq "all") {
  Sync-Pack `
    "radix" `
    (Join-Path $repoRoot "repo-ref/icons/packages/radix-icons/icons") `
    (Join-Path $repoRoot "crates/fret-icons-radix/icon-list.txt") `
    (Join-Path $repoRoot "crates/fret-icons-radix/assets/icons") `
    -Clean:$Clean
}
