param(
  [ValidateSet("shadcn-web", "radix-web")]
  [string]$Kind = "shadcn-web",
  [string]$Style = "v4/new-york-v4",
  [string]$RepoRoot,
  [bool]$NormalizeOpenSuffix = $true,
  [int]$TopMissing = 50,
  [switch]$GroupMissingByPrefix,
  [switch]$GroupUsedByPrefix,
  [int]$TopGroups = 25,
  [string]$GroupSplitPattern = "[\\.-]",
  [string]$FilterMissingPrefix,
  [string]$FilterUsedPrefix,
  [switch]$ShowUsed,
  [switch]$ShowMissing,
  [switch]$AsMarkdown
)

$ErrorActionPreference = "Stop"

function Resolve-RepoRoot([string]$startDir) {
  $dir = (Resolve-Path $startDir).Path
  while ($true) {
    if ((Test-Path (Join-Path $dir "Cargo.toml")) -and (Test-Path (Join-Path $dir "goldens"))) {
      return $dir
    }
    $parent = Split-Path -Parent $dir
    if (-not $parent -or $parent -eq $dir) {
      break
    }
    $dir = $parent
  }
  throw "Unable to locate repo root from $startDir (expected Cargo.toml + goldens/)."
}

if (-not $RepoRoot) {
  $RepoRoot = Resolve-RepoRoot (Join-Path $PSScriptRoot "..")
}

$goldenDir = Join-Path $RepoRoot ("goldens/{0}/{1}" -f $Kind, $Style)
if (-not (Test-Path $goldenDir)) {
  throw "Missing golden directory: $goldenDir"
}

$testDir = Join-Path $RepoRoot "ecosystem/fret-ui-shadcn/tests"
if (-not (Test-Path $testDir)) {
  throw "Missing test directory: $testDir"
}

$goldenNames = Get-ChildItem -Path $goldenDir -File -Filter "*.json" |
  ForEach-Object { $_.BaseName } |
  Sort-Object -Unique

$goldenKeys = $goldenNames
if ($Kind -eq "shadcn-web" -and $NormalizeOpenSuffix) {
  $goldenKeys = $goldenNames |
    ForEach-Object { $_ -replace '\.open$', '' } |
    Sort-Object -Unique
}

$testFiles = Get-ChildItem -Path $testDir -File -Filter "*.rs"
$testText = ($testFiles | ForEach-Object { Get-Content -Raw -LiteralPath $_.FullName }) -join "`n"

$used = [System.Collections.Generic.HashSet[string]]::new()

foreach ($name in $goldenKeys) {
  $needle = '"' + $name + '"'
  if ($testText.Contains($needle)) {
    [void]$used.Add($name)
  }
}

$usedNames = @($used) | Sort-Object
$missingNames = $goldenKeys | Where-Object { -not $used.Contains($_) }

$totalFiles = $goldenNames.Count
$total = $goldenKeys.Count
$usedCount = $usedNames.Count
$missingCount = $missingNames.Count

$coverage = 0.0
if ($total -gt 0) {
  $coverage = [Math]::Round(($usedCount * 100.0) / $total, 1)
}

if ($AsMarkdown) {
  Write-Output ('- `{0}` goldens: {1} files, {2} keys; {3} keys referenced ({4}%), {5} keys not referenced' -f $Kind, $totalFiles, $total, $usedCount, $coverage, $missingCount)
} else {
  Write-Host ("Golden coverage ({0}/{1})" -f $Kind, $Style)
  Write-Host ("  RepoRoot:  {0}" -f $RepoRoot)
  Write-Host ("  GoldenDir: {0}" -f $goldenDir)
  Write-Host ("  TestsDir:  {0}" -f $testDir)
  Write-Host ("  Files:     {0}" -f $totalFiles)
  Write-Host ("  Keys:      {0} (NormalizeOpenSuffix={1})" -f $total, $NormalizeOpenSuffix)
  Write-Host ("  Used keys: {0} ({1}%)" -f $usedCount, $coverage)
  Write-Host ("  Missing:   {0} keys" -f $missingCount)
}

if ($ShowUsed) {
  Write-Host ""
  Write-Host "Referenced (unique):"
  $usedNames | ForEach-Object { Write-Host ("  {0}" -f $_) }
}

if ($ShowMissing) {
  Write-Host ""
  Write-Host ("Not referenced (first {0}):" -f $TopMissing)
  $missingNames | Select-Object -First $TopMissing | ForEach-Object { Write-Host ("  {0}" -f $_) }
}

if ($GroupMissingByPrefix) {
  $prefixes = $missingNames | ForEach-Object {
    $parts = $_ -split $GroupSplitPattern
    if ($parts.Length -gt 0) { $parts[0] } else { $_ }
  }

  $groups = $prefixes | Group-Object | Sort-Object Count -Descending
  if ($AsMarkdown) {
    Write-Output ""
    Write-Output ("- Missing keys grouped by prefix (Top {0}):" -f $TopGroups)
    $groups | Select-Object -First $TopGroups | ForEach-Object {
      Write-Output ('  - `{0}`: {1}' -f $_.Name, $_.Count)
    }
  } else {
    Write-Host ""
    Write-Host ("Missing keys grouped by prefix (Top {0}):" -f $TopGroups)
    $groups | Select-Object -First $TopGroups | Format-Table -AutoSize Count, Name
  }
}

if ($GroupUsedByPrefix) {
  $prefixes = $usedNames | ForEach-Object {
    $parts = $_ -split $GroupSplitPattern
    if ($parts.Length -gt 0) { $parts[0] } else { $_ }
  }

  $groups = $prefixes | Group-Object | Sort-Object Count -Descending
  if ($AsMarkdown) {
    Write-Output ""
    Write-Output ("- Referenced keys grouped by prefix (Top {0}):" -f $TopGroups)
    $groups | Select-Object -First $TopGroups | ForEach-Object {
      Write-Output ('  - `{0}`: {1}' -f $_.Name, $_.Count)
    }
  } else {
    Write-Host ""
    Write-Host ("Referenced keys grouped by prefix (Top {0}):" -f $TopGroups)
    $groups | Select-Object -First $TopGroups | Format-Table -AutoSize Count, Name
  }
}

if ($FilterMissingPrefix) {
  $prefix = $FilterMissingPrefix.Trim()
  if ($prefix.Length -eq 0) {
    throw "FilterMissingPrefix is empty."
  }

  $filtered = $missingNames | Where-Object { $_ -like ("{0}*" -f $prefix) }

  if ($AsMarkdown) {
    Write-Output ""
    Write-Output ('- Missing keys with prefix `{0}`: {1}' -f $prefix, $filtered.Count)
    $filtered | ForEach-Object { Write-Output ('  - `{0}`' -f $_) }
  } else {
    Write-Host ""
    Write-Host ("Missing keys with prefix {0}: {1}" -f $prefix, $filtered.Count)
    $filtered | ForEach-Object { Write-Host ("  {0}" -f $_) }
  }
}

if ($FilterUsedPrefix) {
  $prefix = $FilterUsedPrefix.Trim()
  if ($prefix.Length -eq 0) {
    throw "FilterUsedPrefix is empty."
  }

  $filtered = $usedNames | Where-Object { $_ -like ("{0}*" -f $prefix) }

  if ($AsMarkdown) {
    Write-Output ""
    Write-Output ('- Referenced keys with prefix `{0}`: {1}' -f $prefix, $filtered.Count)
    $filtered | ForEach-Object { Write-Output ('  - `{0}`' -f $_) }
  } else {
    Write-Host ""
    Write-Host ("Referenced keys with prefix {0}: {1}" -f $prefix, $filtered.Count)
    $filtered | ForEach-Object { Write-Host ("  {0}" -f $_) }
  }
}
