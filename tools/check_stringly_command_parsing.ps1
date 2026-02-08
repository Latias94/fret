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
    "apps/fretboard/src/scaffold",
    "apps/fret-ui-gallery/src"
)

$rules = @(
    @{
        Rule    = "no-command-strip-prefix-parsing"
        Pattern = "command\.as_str\(\)\.strip_prefix\("
    },
    @{
        Rule    = "no-command-cmd-prefix-strip-prefix-parsing"
        Pattern = "strip_prefix\((?:crate::commands::)?CMD_[A-Z0-9_]+_PREFIX\)"
    }
)

$rootPaths = @()
foreach ($root in $roots) {
    $rootPaths += (Join-Path $repoRoot $root)
}

$files = Get-ChildItem -Path $rootPaths -Recurse -Filter *.rs -File
foreach ($file in $files) {
    $rel = Normalize-RepoPath -RepoRoot $repoRoot -Path $file.FullName

    foreach ($rule in $rules) {
        $matches = Select-String -Path $file.FullName -Pattern $rule.Pattern -AllMatches -CaseSensitive
        foreach ($match in $matches) {
            Write-Violation -Rule $rule.Rule -Path $rel -LineNumber $match.LineNumber -Line $match.Line
        }
    }
}

if ($script:HadErrors) {
    exit 1
}

Write-Host "Stringly command parsing check passed."
