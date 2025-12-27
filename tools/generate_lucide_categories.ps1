param(
  [Parameter(Mandatory = $false)]
  [string]$LucideDir = "repo-ref/lucide",

  [Parameter(Mandatory = $false)]
  [string]$OutPath = "crates/fret-icons-lucide/categories.json"
)

$ErrorActionPreference = "Stop"

function Assert-Exists([string]$Path) {
  if (-not (Test-Path $Path)) {
    throw "Path not found: $Path"
  }
}

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$lucideRoot = Join-Path $repoRoot $LucideDir
$lucideCategories = Join-Path $lucideRoot "categories"
$lucideIconMeta = Join-Path $lucideRoot "icons"
$outFile = Join-Path $repoRoot $OutPath

Assert-Exists $lucideCategories
Assert-Exists $lucideIconMeta

$knownCategories = @{}
Get-ChildItem $lucideCategories -Filter "*.json" | ForEach-Object {
  $knownCategories[$_.BaseName] = $true
}

$categoryToIcons = @{}
Get-ChildItem $lucideIconMeta -Filter "*.json" | ForEach-Object {
  $iconName = $_.BaseName
  $meta = Get-Content $_.FullName -Raw | ConvertFrom-Json
  foreach ($cat in $meta.categories) {
    if (-not $knownCategories.ContainsKey($cat)) {
      throw "Unknown Lucide category '$cat' referenced by icon '$iconName' (update repo-ref/lucide/categories)"
    }

    if (-not $categoryToIcons.ContainsKey($cat)) {
      $categoryToIcons[$cat] = New-Object System.Collections.Generic.List[string]
    }
    $categoryToIcons[$cat].Add($iconName)
  }
}

$ordered = [ordered]@{}
foreach ($cat in ($knownCategories.Keys | Sort-Object)) {
  $icons = @()
  if ($categoryToIcons.ContainsKey($cat)) {
    $icons = $categoryToIcons[$cat] | Sort-Object -Unique
  }
  $ordered[$cat] = $icons
}

$json = $ordered | ConvertTo-Json -Depth 4
New-Item -ItemType Directory -Path (Split-Path $outFile) -Force | Out-Null
Set-Content -Path $outFile -Value $json -Encoding utf8

Write-Host "Wrote Lucide category map:" $outFile
