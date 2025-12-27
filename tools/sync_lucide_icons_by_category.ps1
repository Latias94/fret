param(
  [Parameter(Mandatory = $true)]
  [string[]]$Categories,

  [Parameter(Mandatory = $false)]
  [string]$LucideDir = "repo-ref/lucide",

  [Parameter(Mandatory = $false)]
  [string]$CategoryMap = "crates/fret-icons-lucide/categories.json",

  [Parameter(Mandatory = $false)]
  [string]$DestDir = "crates/fret-icons-lucide/assets/icons"
)

$ErrorActionPreference = "Stop"

function Assert-Exists([string]$Path) {
  if (-not (Test-Path $Path)) {
    throw "Path not found: $Path"
  }
}

$Categories = @(
  $Categories |
    ForEach-Object { $_.Split(",") } |
    ForEach-Object { $_.Trim() } |
    Where-Object { $_ }
)

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$lucideRoot = Join-Path $repoRoot $LucideDir
$lucideSvgs = Join-Path $lucideRoot "icons"
$mapFile = Join-Path $repoRoot $CategoryMap
$dest = Join-Path $repoRoot $DestDir

Assert-Exists $lucideSvgs
Assert-Exists $mapFile
New-Item -ItemType Directory -Path $dest -Force | Out-Null

$map = Get-Content $mapFile -Raw | ConvertFrom-Json

$copied = 0
$missing = 0

foreach ($cat in $Categories) {
  if (-not ($map.PSObject.Properties.Name -contains $cat)) {
    throw "Category '$cat' not found in $mapFile"
  }

  foreach ($icon in $map.$cat) {
    $src = Join-Path $lucideSvgs "$icon.svg"
    $dst = Join-Path $dest "$icon.svg"
    if (Test-Path $src) {
      Copy-Item -Path $src -Destination $dst -Force
      $copied++
    }
    else {
      $missing++
    }
  }
}

Write-Host "Synced Lucide categories:" ($Categories -join ", ")
Write-Host "Copied:" $copied "Missing SVGs:" $missing
