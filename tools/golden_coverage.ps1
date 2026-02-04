param(
  [ValidateSet("shadcn-web", "radix-web")]
  [string]$Kind = "shadcn-web",
  [string]$Style = "v4/new-york-v4",
  [string]$RepoRoot,
  [bool]$NormalizeOpenSuffix = $true,
  [switch]$TrackedOnly,
  [int]$TopMissing = 50,
  [switch]$GroupMissingByPrefix,
  [switch]$GroupUsedByPrefix,
  [int]$TopGroups = 25,
  [string]$GroupSplitPattern = "[\\.-]",
  [string]$FilterMissingPrefix,
  [string]$FilterUsedPrefix,
  [switch]$ShowUsed,
  [switch]$ShowMissing,
  [switch]$ShowTargetedMissing,
  [string[]]$TargetedGateExcludeFiles = @("web_vs_fret_layout.rs", "snapshots.rs"),
  [switch]$ShowGateBreakdown,
  [switch]$GroupUntargetedByPrefix,
  [string]$FilterUntargetedPrefix,
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

$goldenNames = if ($TrackedOnly) {
  $repoRootAbs = (Resolve-Path $RepoRoot).Path
  $goldenDirAbs = (Resolve-Path $goldenDir).Path

  if (-not $goldenDirAbs.StartsWith($repoRootAbs + [System.IO.Path]::DirectorySeparatorChar)) {
    throw "GoldenDir is not under RepoRoot (RepoRoot=$repoRootAbs, GoldenDir=$goldenDirAbs)"
  }

  $relGoldenDir = $goldenDirAbs.Substring($repoRootAbs.Length + 1) -replace '\\', '/'
  $tracked = git -C $RepoRoot ls-files -- $relGoldenDir
  if ($LASTEXITCODE -ne 0) {
    throw "git ls-files failed (RepoRoot=$RepoRoot, GoldenDir=$relGoldenDir)"
  }

  @($tracked) |
    Where-Object { $_ -like "*.json" } |
    ForEach-Object { [System.IO.Path]::GetFileNameWithoutExtension($_) } |
    Sort-Object -Unique
} else {
  Get-ChildItem -Path $goldenDir -File -Filter "*.json" |
    ForEach-Object { $_.BaseName } |
    Sort-Object -Unique
}

$goldenKeys = $goldenNames
if ($Kind -eq "shadcn-web" -and $NormalizeOpenSuffix) {
  $goldenKeys = $goldenNames |
    ForEach-Object { $_ -replace '\.open$', '' } |
    Sort-Object -Unique
}

$testFiles = Get-ChildItem -Path $testDir -File -Filter "*.rs"

# "Gated" coverage aims to approximate "dedicated behavior gates", not "dynamic smoke traversal".
# Exclude `*_goldens_smoke.rs` from the string-literal heuristic so smoke-only keys do not inflate
# the gated percentage.
$gateTestFiles = $testFiles | Where-Object { $_.Name -notlike "*_goldens_smoke.rs" }

$used = [System.Collections.Generic.HashSet[string]]::new()
$usedTargeted = [System.Collections.Generic.HashSet[string]]::new()
$usedByFile = @{}

foreach ($file in $gateTestFiles) {
  $text = Get-Content -Raw -LiteralPath $file.FullName
  $fileUsed = [System.Collections.Generic.HashSet[string]]::new()

  foreach ($name in $goldenKeys) {
    $needle = '"' + $name + '"'
    if ($text.Contains($needle)) {
      [void]$fileUsed.Add($name)
      [void]$used.Add($name)

      if (-not ($TargetedGateExcludeFiles -contains $file.Name)) {
        [void]$usedTargeted.Add($name)
      }
    }
  }

  $usedByFile[$file.Name] = $fileUsed
}

$usedNames = @($used) | Sort-Object
$missingNames = $goldenKeys | Where-Object { -not $used.Contains($_) }

$usedTargetedNames = @($usedTargeted) | Sort-Object
$targetedMissingNames = $goldenKeys | Where-Object { -not $usedTargeted.Contains($_) }

$totalFiles = $goldenNames.Count
$total = $goldenKeys.Count
$usedCount = $usedNames.Count
$missingCount = $missingNames.Count
$usedTargetedCount = $usedTargetedNames.Count
$targetedMissingCount = $targetedMissingNames.Count

$coverage = 0.0
if ($total -gt 0) {
  $coverage = [Math]::Round(($usedCount * 100.0) / $total, 1)
}

$targetedCoverage = 0.0
if ($total -gt 0) {
  $targetedCoverage = [Math]::Round(($usedTargetedCount * 100.0) / $total, 1)
}

$smokeCandidate = $null
$smokeStyle = $null
function Get-SmokeStyleFromTest([string]$path, [string]$kind) {
  if (-not (Test-Path $path)) {
    return $null
  }

  $text = Get-Content -Raw -LiteralPath $path
  $matches = [regex]::Matches($text, '\\.join\\("([^"]+)"\\)')
  if (-not $matches -or $matches.Count -eq 0) {
    return $null
  }

  $tokens = @()
  foreach ($m in $matches) {
    $tokens += $m.Groups[1].Value
  }

  for ($i = 0; $i -lt ($tokens.Count - 1); $i++) {
    if ($tokens[$i] -eq "goldens" -and $tokens[$i + 1] -eq $kind) {
      $styleTokens = @()
      if ($i + 2 -le $tokens.Count - 1) {
        $styleTokens = $tokens[($i + 2)..($tokens.Count - 1)]
      }
      if ($styleTokens.Count -gt 0) {
        return ($styleTokens -join "/")
      }
      return $null
    }
  }

  return $null
}

$smokeTest = $null
$smokeCoverage = $null
if ($Kind -eq "shadcn-web") {
  $candidate = Join-Path $testDir "shadcn_web_goldens_smoke.rs"
  if (Test-Path $candidate) {
    $smokeCandidate = $candidate
    $smokeStyle = Get-SmokeStyleFromTest $candidate $Kind
  }
}
if ($Kind -eq "radix-web") {
  $candidate = Join-Path $testDir "radix_web_goldens_smoke.rs"
  if (Test-Path $candidate) {
    $smokeCandidate = $candidate
    $smokeStyle = Get-SmokeStyleFromTest $candidate $Kind
  }
}

if ($smokeCandidate) {
  if (-not $smokeStyle) {
    $smokeTest = $smokeCandidate
    $smokeCoverage = 100.0
  } elseif ($smokeStyle -eq $Style) {
    $smokeTest = $smokeCandidate
    $smokeCoverage = 100.0
  }
}

if ($AsMarkdown) {
  $trackedNote = if ($TrackedOnly) { " (tracked-only)" } else { "" }
  Write-Output ('- `{0}` goldens{1}: {2} files, {3} keys' -f $Kind, $trackedNote, $totalFiles, $total)
  Write-Output ('  - gated (any non-smoke test): {0} keys ({1}%) [string-literal heuristic], {2} missing' -f $usedCount, $coverage, $missingCount)
  Write-Output ('  - targeted gates (excluding {0}): {1} keys ({2}%), {3} missing' -f (($TargetedGateExcludeFiles | Sort-Object) -join ", "), $usedTargetedCount, $targetedCoverage, $targetedMissingCount)
  if ($smokeTest) {
    Write-Output ('  - smoke-parse coverage: {0}% (via `{1}`)' -f $smokeCoverage, (Split-Path -Leaf $smokeTest))
  } elseif ($smokeCandidate -and $smokeStyle -and $smokeStyle -ne $Style) {
    Write-Output ('  - smoke-parse coverage: n/a (smoke test targets `{0}`, not `{1}`)' -f $smokeStyle, $Style)
  }
} else {
  Write-Host ("Golden coverage ({0}/{1})" -f $Kind, $Style)
  Write-Host ("  RepoRoot:  {0}" -f $RepoRoot)
  Write-Host ("  GoldenDir: {0}" -f $goldenDir)
  Write-Host ("  TestsDir:  {0}" -f $testDir)
  Write-Host ("  Tracked:   {0}" -f $(if ($TrackedOnly) { "yes" } else { "no" }))
  Write-Host ("  Files:     {0}" -f $totalFiles)
  Write-Host ("  Keys:      {0} (NormalizeOpenSuffix={1})" -f $total, $NormalizeOpenSuffix)
  Write-Host ("  Gated:     {0} keys ({1}%) [any non-smoke test, string-literal heuristic]" -f $usedCount, $coverage)
  Write-Host ("  Ungated:   {0} keys [not referenced by non-smoke tests]" -f $missingCount)
  Write-Host ("  Targeted:  {0} keys ({1}%) [excluding: {2}]" -f $usedTargetedCount, $targetedCoverage, (($TargetedGateExcludeFiles | Sort-Object) -join ", "))
  Write-Host ("  Untargeted:{0} keys [only gated by excluded files]" -f $targetedMissingCount)
  if ($smokeTest) {
    Write-Host ("  Smoke:     yes ({0}%, {1})" -f $smokeCoverage, (Split-Path -Leaf $smokeTest))
  } elseif ($smokeCandidate -and $smokeStyle -and $smokeStyle -ne $Style) {
    Write-Host ("  Smoke:     n/a (smoke test targets {0}, not {1})" -f $smokeStyle, $Style)
  } else {
    Write-Host ("  Smoke:     n/a")
  }
}

if ($ShowUsed) {
  Write-Host ""
  Write-Host "Gated keys (unique):"
  $usedNames | ForEach-Object { Write-Host ("  {0}" -f $_) }
}

if ($ShowMissing) {
  Write-Host ""
  Write-Host ("Ungated keys (first {0}):" -f $TopMissing)
  $missingNames | Select-Object -First $TopMissing | ForEach-Object { Write-Host ("  {0}" -f $_) }
}

if ($ShowTargetedMissing) {
  Write-Host ""
  Write-Host ("Untargeted keys (first {0}) [only gated by excluded files]:" -f $TopMissing)
  $targetedMissingNames | Select-Object -First $TopMissing | ForEach-Object { Write-Host ("  {0}" -f $_) }
}

if ($ShowGateBreakdown) {
  $rows = @()
  foreach ($k in $usedByFile.Keys) {
    $rows += [pscustomobject]@{
      File = $k
      Keys = @($usedByFile[$k]).Count
    }
  }

  $rows = $rows | Sort-Object Keys -Descending

  if ($AsMarkdown) {
    Write-Output ""
    Write-Output "### Gate breakdown (keys referenced per test file)"
    foreach ($row in $rows) {
      $pct = 0.0
      if ($total -gt 0) { $pct = [Math]::Round(($row.Keys * 100.0) / $total, 1) }
      Write-Output ("- `{0}`: {1} ({2}%)" -f $row.File, $row.Keys, $pct)
    }
  } else {
    Write-Host ""
    Write-Host "Gate breakdown (keys referenced per test file):"
    foreach ($row in $rows) {
      $pct = 0.0
      if ($total -gt 0) { $pct = [Math]::Round(($row.Keys * 100.0) / $total, 1) }
      Write-Host ("  {0}: {1} ({2}%)" -f $row.File, $row.Keys, $pct)
    }
  }
}

if ($FilterUntargetedPrefix) {
  $filtered = $targetedMissingNames | Where-Object { $_ -like ($FilterUntargetedPrefix + "*") }
  if ($AsMarkdown) {
    Write-Output ""
    Write-Output ("### Untargeted keys: {0} (first {1})" -f $FilterUntargetedPrefix, $TopMissing)
    $filtered | Select-Object -First $TopMissing | ForEach-Object { Write-Output ("- {0}" -f $_) }
  } else {
    Write-Host ""
    Write-Host ("Untargeted keys: {0} (first {1})" -f $FilterUntargetedPrefix, $TopMissing)
    $filtered | Select-Object -First $TopMissing | ForEach-Object { Write-Host ("  {0}" -f $_) }
  }
}

if ($GroupUntargetedByPrefix) {
  $prefixes = $targetedMissingNames | ForEach-Object {
    $parts = $_ -split $GroupSplitPattern
    if ($parts.Length -gt 0) { $parts[0] } else { $_ }
  }

  $groups = $prefixes | Group-Object | Sort-Object Count -Descending
  if ($AsMarkdown) {
    Write-Output ""
    Write-Output "### Untargeted groups (heuristic)"
    $groups | Select-Object -First $TopGroups | ForEach-Object {
      Write-Output ('- `{0}` ({1})' -f $_.Name, $_.Count)
    }
  } else {
    Write-Host ""
    Write-Host "Untargeted groups (heuristic):"
    $groups | Select-Object -First $TopGroups | ForEach-Object {
      Write-Host ("  {0}: {1}" -f $_.Name, $_.Count)
    }
  }
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
