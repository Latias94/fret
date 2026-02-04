param(
  [string]$Style = "v4/new-york-v4",
  [string]$RepoRoot,
  [switch]$TrackedOnly,
  [bool]$NormalizeOpenSuffix = $true,
  [string]$GateKind = "menu-height",
  [string]$ConstrainedViewportToken = "vp375x240",
  [string]$FilterKeyRegex = "(menu|dropdown|select|combobox|command)",
  [string]$ExcludeKeyRegex = "(focus-first|highlight-first|then-hover)",
  [string]$DumpKey,
  [int]$TopMissing = 50,
  [switch]$GroupMissingByPrefix,
  [int]$TopGroups = 25,
  [string]$GroupSplitPattern = "[\\.-]",
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

$goldenDir = Join-Path $RepoRoot ("goldens/shadcn-web/{0}" -f $Style)
if (-not (Test-Path $goldenDir)) {
  throw "Missing golden directory: $goldenDir"
}

$testDir = Join-Path $RepoRoot "ecosystem/fret-ui-shadcn/tests"
if (-not (Test-Path $testDir)) {
  throw "Missing test directory: $testDir"
}

function Get-TrackedGoldenNames([string]$repoRoot, [string]$dir) {
  $repoRootAbs = (Resolve-Path $repoRoot).Path
  $dirAbs = (Resolve-Path $dir).Path
  if (-not $dirAbs.StartsWith($repoRootAbs + [System.IO.Path]::DirectorySeparatorChar)) {
    throw "GoldenDir is not under RepoRoot (RepoRoot=$repoRootAbs, GoldenDir=$dirAbs)"
  }

  $rel = $dirAbs.Substring($repoRootAbs.Length + 1) -replace '\\', '/'
  $tracked = git -C $repoRoot ls-files -- $rel
  if ($LASTEXITCODE -ne 0) {
    throw "git ls-files failed (RepoRoot=$repoRoot, GoldenDir=$rel)"
  }
  return @($tracked) | Where-Object { $_ -like "*.json" } | ForEach-Object { $_ }
}

function Extract-OpenKeys([string[]]$paths, [bool]$normalizeOpenSuffix) {
  $keys = @()
  foreach ($p in $paths) {
    if ($p -notlike "*.open.json") { continue }
    $name = [System.IO.Path]::GetFileNameWithoutExtension($p)
    if ($normalizeOpenSuffix) {
      $name = $name -replace '\.open$', ''
    }
    $keys += $name
  }
  return $keys | Sort-Object -Unique
}

$goldenFiles = if ($TrackedOnly) {
  Get-TrackedGoldenNames $RepoRoot $goldenDir
} else {
  Get-ChildItem -Path $goldenDir -File -Filter "*.json" | ForEach-Object { $_.FullName }
}

$openKeys = Extract-OpenKeys $goldenFiles $NormalizeOpenSuffix

$keySet = [System.Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
foreach ($k in $openKeys) { [void]$keySet.Add($k) }

function Detect-GateKinds([string]$testName, [string]$gateKind) {
  $kinds = @()

  switch ($gateKind) {
    "menu-height" {
      if ($testName -match "panel_size") { $kinds += "panel-size" }
      if ($testName -match "menu_item_height") { $kinds += "menu-item-height" }
      if ($testName -match "listbox_height") { $kinds += "listbox-height" }
      if ($testName -match "option_height") { $kinds += "listbox-option-height" }
      if ($testName -match "content_insets") { $kinds += "content-insets" }
      if ($testName -match "wheel_scroll_matches_web_scrolled") { $kinds += "wheel-scroll" }
      if ($testName -match "viewport_height_matches") { $kinds += "viewport-height" }
      break
    }
    default {
      if ($testName -match "panel_size") { $kinds += "panel-size" }
      if ($testName -match "height") { $kinds += "height" }
      if ($testName -match "insets") { $kinds += "insets" }
      break
    }
  }

  return $kinds
}

$gateKindsByKey = @{}
foreach ($k in $openKeys) {
  $gateKindsByKey[$k] = [System.Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
}

function Extract-KeysFromBlock([string]$block, $keySet) {
  $hits = [System.Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
  $rx = [regex]'"([a-z0-9][a-z0-9_.-]{1,160})"' # shadcn keys are lowercase + punctuation.
  $matches = $rx.Matches($block)
  foreach ($m in $matches) {
    $s = $m.Groups[1].Value
    if ($keySet.Contains($s)) { [void]$hits.Add($s) }
  }
  return @($hits)
}

$testFiles = Get-ChildItem -Path $testDir -File -Filter "*.rs" |
  Where-Object { $_.Name -notlike "*_goldens_smoke.rs" } |
  ForEach-Object { $_.FullName }

foreach ($path in $testFiles) {
  $text = Get-Content -Raw -LiteralPath $path

  $parts = $text -split '(?m)^(?=\#\[test\])'
  foreach ($part in $parts) {
    if ($part -notmatch '(?m)^\#\[test\]') { continue }

    $testName = [regex]::Match($part, '(?m)^fn\s+([A-Za-z0-9_]+)\s*\(').Groups[1].Value
    if (-not $testName) { continue }

    $kinds = Detect-GateKinds $testName $GateKind
    if ($kinds.Count -eq 0) { continue }

    $keys = Extract-KeysFromBlock $part $keySet
    foreach ($k in $keys) {
      if (-not $gateKindsByKey.ContainsKey($k)) { continue }
      foreach ($kind in $kinds) {
        [void]$gateKindsByKey[$k].Add($kind)
      }
    }
  }
}

if ($DumpKey) {
  if (-not $gateKindsByKey.ContainsKey($DumpKey)) {
    throw "DumpKey not found in open keys: $DumpKey"
  }
  $kinds = @($gateKindsByKey[$DumpKey]) | Sort-Object
  if ($AsMarkdown) {
    Write-Output ('- `{0}` gate kinds: {1}' -f $DumpKey, ($kinds -join ", "))
  } else {
    Write-Host ("{0} gate kinds: {1}" -f $DumpKey, ($kinds -join ", "))
  }
  exit 0
}

function Has-MenuHeightGate([string]$key) {
  $kinds = $gateKindsByKey[$key]
  return $kinds.Contains("wheel-scroll") -or
    $kinds.Contains("panel-size") -or
    $kinds.Contains("viewport-height") -or
    $kinds.Contains("menu-item-height") -or
    $kinds.Contains("listbox-height") -or
    $kinds.Contains("listbox-option-height") -or
    $kinds.Contains("content-insets")
}

$candidates = $openKeys |
  Where-Object { $_ -match $FilterKeyRegex } |
  Where-Object { $_ -like ("*{0}*" -f $ConstrainedViewportToken) }

if ($ExcludeKeyRegex -and $ExcludeKeyRegex.Trim().Length -gt 0) {
  $candidates = $candidates | Where-Object { $_ -notmatch $ExcludeKeyRegex }
}

$missing = $candidates | Where-Object { -not (Has-MenuHeightGate $_) }

if ($AsMarkdown) {
  $trackedNote = if ($TrackedOnly) { " (tracked-only)" } else { "" }
  Write-Output ("- shadcn-web open overlay depth{0}: {1} open keys" -f $trackedNote, $openKeys.Count)
  Write-Output ('  - constrained token: `{0}`' -f $ConstrainedViewportToken)
  Write-Output ('  - filter: `{0}`' -f $FilterKeyRegex)
  if ($ExcludeKeyRegex -and $ExcludeKeyRegex.Trim().Length -gt 0) {
    Write-Output ('  - exclude: `{0}`' -f $ExcludeKeyRegex)
  }
  Write-Output ("  - candidates: {0}" -f $candidates.Count)
  Write-Output ("  - missing menu/listbox height gates: {0}" -f $missing.Count)
} else {
  Write-Host "Golden overlay depth (shadcn-web/$Style)"
  Write-Host ("  RepoRoot:  {0}" -f $RepoRoot)
  Write-Host ("  GoldenDir: {0}" -f $goldenDir)
  Write-Host ("  TestsDir:  {0}" -f $testDir)
  Write-Host ("  Tracked:   {0}" -f $(if ($TrackedOnly) { "yes" } else { "no" }))
  Write-Host ("  Open keys: {0}" -f $openKeys.Count)
  Write-Host ("  Constrained token: {0}" -f $ConstrainedViewportToken)
  Write-Host ("  Filter regex: {0}" -f $FilterKeyRegex)
  if ($ExcludeKeyRegex -and $ExcludeKeyRegex.Trim().Length -gt 0) {
    Write-Host ("  Exclude regex: {0}" -f $ExcludeKeyRegex)
  }
  Write-Host ("  Candidates: {0}" -f $candidates.Count)
  Write-Host ("  Missing menu/listbox height gates: {0}" -f $missing.Count)
}

if ($GroupMissingByPrefix) {
  $prefixes = $missing | ForEach-Object {
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

if ($TopMissing -gt 0) {
  if ($AsMarkdown) {
    Write-Output ""
    Write-Output ("- Missing keys (first {0}):" -f $TopMissing)
    $missing | Select-Object -First $TopMissing | ForEach-Object { Write-Output ('  - `{0}`' -f $_) }
  } else {
    Write-Host ""
    Write-Host ("Missing keys (first {0}):" -f $TopMissing)
    $missing | Select-Object -First $TopMissing | ForEach-Object { Write-Host ("  {0}" -f $_) }
  }
}
