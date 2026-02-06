$ErrorActionPreference = "Continue"
Set-StrictMode -Version Latest

function Normalize-RepoPath {
    param(
        [Parameter(Mandatory = $true)][string]$RepoRoot,
        [Parameter(Mandatory = $true)][string]$Path
    )

    $full = [System.IO.Path]::GetFullPath($Path)
    $root = [System.IO.Path]::GetFullPath($RepoRoot)
    if (-not $full.StartsWith($root)) {
        return ($Path -replace "\\", "/")
    }

    $rel = $full.Substring($root.Length).TrimStart("\", "/")
    return ($rel -replace "\\", "/")
}

function Write-Violation {
    param(
        [Parameter(Mandatory = $true)][string]$Rule,
        [Parameter(Mandatory = $true)][string]$Path,
        [Parameter(Mandatory = $true)][int]$LineNumber,
        [Parameter(Mandatory = $true)][string]$Line
    )

    Write-Error -ErrorAction Continue ("Stringly command parsing violation ({0}): {1}:{2}: {3}" -f $Rule, $Path, $LineNumber, $Line.Trim())
    $script:HadErrors = $true
}

$script:HadErrors = $false

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$roots = @(
    "apps/fret-examples/src",
    "apps/fretboard/src/scaffold"
)

$rule = @{
    Rule    = "no-command-strip-prefix-parsing"
    Pattern = "command\.as_str\(\)\.strip_prefix\("
}

$rootPaths = @()
foreach ($root in $roots) {
    $rootPaths += (Join-Path $repoRoot $root)
}

$files = Get-ChildItem -Path $rootPaths -Recurse -Filter *.rs -File
foreach ($file in $files) {
    $rel = Normalize-RepoPath -RepoRoot $repoRoot -Path $file.FullName

    $matches = Select-String -Path $file.FullName -Pattern $rule.Pattern -AllMatches
    foreach ($match in $matches) {
        Write-Violation -Rule $rule.Rule -Path $rel -LineNumber $match.LineNumber -Line $match.Line
    }
}

if ($script:HadErrors) {
    exit 1
}

Write-Host "Stringly command parsing check passed."

