<#
.SYNOPSIS
  Reports the largest source files in the repository by line count.

.DESCRIPTION
  This is a lightweight drift detector for "god files" that tend to appear during rapid refactors.
  It intentionally does not depend on Cargo metadata so it can be used before the workspace builds.

.PARAMETER Root
  Repository root (defaults to current directory).

.PARAMETER Top
  Number of entries to print (defaults to 50).

.PARAMETER MinLines
  Only include files with at least this many lines (defaults to 0).

.PARAMETER Include
  Only include paths that start with one of these prefixes (defaults to common workspace buckets).

.PARAMETER Exclude
  Exclude any file whose path contains one of these segments (defaults to: target, .git, repo-ref).

.PARAMETER Json
  Emit JSON instead of a table.

.EXAMPLE
  pwsh -NoProfile -File tools/report_largest_files.ps1 -Top 30

.EXAMPLE
  pwsh -NoProfile -File tools/report_largest_files.ps1 -Include crates,ecosystem -MinLines 800
#>

[CmdletBinding()]
param(
    [Parameter(Mandatory = $false)]
    [string] $Root = ".",

    [Parameter(Mandatory = $false)]
    [int] $Top = 50,

    [Parameter(Mandatory = $false)]
    [int] $MinLines = 0,

    [Parameter(Mandatory = $false)]
    [string[]] $Include = @("crates", "ecosystem", "apps", "tools", "themes", "docs"),

    [Parameter(Mandatory = $false)]
    [string[]] $Exclude = @("target", ".git", "repo-ref"),

    [Parameter(Mandatory = $false)]
    [switch] $Json
)

$ErrorActionPreference = "Stop"

function Get-RepoRelativePath {
    param(
        [Parameter(Mandatory = $true)]
        [string] $Base,
        [Parameter(Mandatory = $true)]
        [string] $FullPath
    )

    $baseNorm = [System.IO.Path]::GetFullPath($Base)
    $fullNorm = [System.IO.Path]::GetFullPath($FullPath)

    if (-not $fullNorm.StartsWith($baseNorm, [System.StringComparison]::OrdinalIgnoreCase)) {
        return $fullNorm
    }

    $rel = $fullNorm.Substring($baseNorm.Length).TrimStart([System.IO.Path]::DirectorySeparatorChar, [System.IO.Path]::AltDirectorySeparatorChar)
    return $rel -replace "\\", "/"
}

function Get-LineCount {
    param(
        [Parameter(Mandatory = $true)]
        [string] $Path
    )

    $count = 0
    foreach ($line in [System.IO.File]::ReadLines($Path)) {
        $count++
    }
    return $count
}

$rootPath = (Resolve-Path -LiteralPath $Root).Path

$includePrefixes = $Include | ForEach-Object { $_.Trim().TrimEnd([char[]]"/\\") } | Where-Object { $_ -ne "" }
$excludeSegments = $Exclude | ForEach-Object { $_.Trim().Trim([char[]]"/\\") } | Where-Object { $_ -ne "" }

$files = Get-ChildItem -LiteralPath $rootPath -Recurse -File -Force -Filter "*.rs"

$rows = foreach ($file in $files) {
    $relative = Get-RepoRelativePath -Base $rootPath -FullPath $file.FullName

    $isIncluded = $false
    foreach ($prefix in $includePrefixes) {
        if ($relative.StartsWith($prefix + "/", [System.StringComparison]::OrdinalIgnoreCase) -or $relative.Equals($prefix, [System.StringComparison]::OrdinalIgnoreCase)) {
            $isIncluded = $true
            break
        }
    }
    if (-not $isIncluded) { continue }

    $isExcluded = $false
    foreach ($seg in $excludeSegments) {
        if ($relative -match "(^|/)$([regex]::Escape($seg))(/|$)") {
            $isExcluded = $true
            break
        }
        if ($relative -like "*$seg*") {
            # Fallback: segment may not align with separators (e.g. ".git").
            $isExcluded = $true
            break
        }
    }
    if ($isExcluded) { continue }

    $lines = Get-LineCount -Path $file.FullName
    if ($lines -lt $MinLines) { continue }

    [pscustomobject]@{
        path  = $relative
        lines = $lines
    }
}

$topRows = $rows | Sort-Object -Property lines -Descending | Select-Object -First $Top

if ($Json) {
    $topRows | ConvertTo-Json -Depth 3
}
else {
    $topRows | Format-Table -AutoSize
}
