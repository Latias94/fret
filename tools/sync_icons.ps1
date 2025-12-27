param(
  [Parameter(Mandatory = $false)]
  [ValidateSet("lucide", "radix", "all")]
  [string]$Pack = "all"
)

$ErrorActionPreference = "Stop"

function Assert-Exists([string]$Path) {
  if (-not (Test-Path $Path)) {
    throw "Path not found: $Path"
  }
}

function Sync-Pack([string]$Name, [string]$RepoRefDir, [string]$ListPath, [string]$DestDir) {
  Assert-Exists $RepoRefDir
  Assert-Exists $ListPath
  if (-not (Test-Path $DestDir)) {
    New-Item -ItemType Directory -Path $DestDir | Out-Null
  }

  $items = Get-Content $ListPath | Where-Object { $_ -and (-not $_.StartsWith("#")) } | ForEach-Object { $_.Trim() }
  foreach ($item in $items) {
    $src = Join-Path $RepoRefDir $item
    $dst = Join-Path $DestDir $item
    Assert-Exists $src
    Copy-Item -Path $src -Destination $dst -Force
  }

  Write-Host "Synced $Name icons: $($items.Count) files"
}

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")

if ($Pack -eq "lucide" -or $Pack -eq "all") {
  Sync-Pack `
    "lucide" `
    (Join-Path $repoRoot "repo-ref/lucide/icons") `
    (Join-Path $repoRoot "crates/fret-icons-lucide/icon-list.txt") `
    (Join-Path $repoRoot "crates/fret-icons-lucide/assets/icons")
}

if ($Pack -eq "radix" -or $Pack -eq "all") {
  Sync-Pack `
    "radix" `
    (Join-Path $repoRoot "repo-ref/icons/packages/radix-icons/icons") `
    (Join-Path $repoRoot "crates/fret-icons-radix/icon-list.txt") `
    (Join-Path $repoRoot "crates/fret-icons-radix/assets/icons")
}
