param(
  [Parameter(Mandatory = $false)]
  [string]$GpuiList = "tools/gpui-icon-list.txt",

  [Parameter(Mandatory = $false)]
  [string]$LucideDir = "repo-ref/lucide/icons",

  [Parameter(Mandatory = $false)]
  [string]$DestDir = "crates/fret-icons-lucide/assets/icons"
)

$ErrorActionPreference = "Stop"

function Assert-Exists([string]$Path) {
  if (-not (Test-Path $Path)) {
    throw "Path not found: $Path"
  }
}

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$listFile = Join-Path $repoRoot $GpuiList
$src = Join-Path $repoRoot $LucideDir
$dst = Join-Path $repoRoot $DestDir

Assert-Exists $listFile
Assert-Exists $src
New-Item -ItemType Directory -Path $dst -Force | Out-Null

$items = Get-Content $listFile |
  Where-Object { $_ -and (-not $_.StartsWith("#")) } |
  ForEach-Object { $_.Trim() } |
  Where-Object { $_ }

$copied = 0
$missing = New-Object System.Collections.Generic.List[string]

foreach ($name in $items) {
  $from = Join-Path $src $name
  $to = Join-Path $dst $name
  if (Test-Path $from) {
    Copy-Item -Path $from -Destination $to -Force
    $copied++
  }
  else {
    $missing.Add($name)
  }
}

Write-Host "Synced Lucide icons from gpui-component list"
Write-Host "Copied:" $copied
Write-Host "Missing:" $missing.Count
if ($missing.Count -gt 0) {
  Write-Host "Missing examples:"
  $missing | Select-Object -First 20
}
