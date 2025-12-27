param(
  [Parameter(Mandatory = $false)]
  [string]$ListPath = "crates/fret-icons-lucide/icon-list.txt",

  [Parameter(Mandatory = $false)]
  [string]$DestDir = "crates/fret-icons-lucide/assets/icons",

  [Parameter(Mandatory = $false)]
  [switch]$Apply
)

$ErrorActionPreference = "Stop"

function Assert-Exists([string]$Path) {
  if (-not (Test-Path $Path)) {
    throw "Path not found: $Path"
  }
}

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$listFile = Join-Path $repoRoot $ListPath
$dest = Join-Path $repoRoot $DestDir

Assert-Exists $listFile
Assert-Exists $dest

$keep = New-Object "System.Collections.Generic.HashSet[string]"

foreach ($line in (Get-Content $listFile)) {
  $line = $line.Trim()
  if (-not $line) { continue }
  if ($line.StartsWith("#")) { continue }
  if (-not $line.EndsWith(".svg")) {
    throw "Invalid entry (expected *.svg): $line"
  }
  [void]$keep.Add($line)
}

$all = Get-ChildItem $dest -Filter "*.svg"
$toDelete = $all | Where-Object { -not $keep.Contains($_.Name) }

Write-Host "Lucide icon cleanup"
Write-Host "List file:" $ListPath
Write-Host "Total SVGs:" $all.Count
Write-Host "Keep SVGs:" $keep.Count
Write-Host "Delete SVGs:" $toDelete.Count

if (-not $Apply) {
  Write-Host "Dry run (pass -Apply to actually delete)."
  $toDelete | Select-Object -First 40 Name
  exit 0
}

foreach ($f in $toDelete) {
  Remove-Item -Force -Path $f.FullName
}

Write-Host "Deleted:" $toDelete.Count
