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

function Is-AllowListedPath {
    param(
        [Parameter(Mandatory = $true)][string]$Path,
        [Parameter(Mandatory = $true)][string[]]$AllowRegexes
    )

    foreach ($pattern in $AllowRegexes) {
        if ($Path -match $pattern) {
            return $true
        }
    }

    return $false
}

function Write-Violation {
    param(
        [Parameter(Mandatory = $true)][string]$Rule,
        [Parameter(Mandatory = $true)][string]$Path,
        [Parameter(Mandatory = $true)][string]$Detail
    )

    Write-Error -ErrorAction Continue ("Component state coupling violation ({0}): {1}: {2}" -f $Rule, $Path, $Detail)
    $script:HadErrors = $true
}

$script:HadErrors = $false

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$sourceRoots = @(
    "ecosystem/fret-ui-kit/src",
    "ecosystem/fret-ui-shadcn/src",
    "ecosystem/fret-ui-material3/src",
    "ecosystem/fret-imui/src"
)

$manifestPaths = @(
    "ecosystem/fret-ui-kit/Cargo.toml",
    "ecosystem/fret-ui-shadcn/Cargo.toml",
    "ecosystem/fret-ui-material3/Cargo.toml",
    "ecosystem/fret-imui/Cargo.toml"
)

$allowRegexes = @(
    "^ecosystem/fret-ui-kit/src/state(?:/|\.rs)",
    "^ecosystem/fret-ui-shadcn/src/state(?:/|\.rs)",
    "^ecosystem/fret-ui-material3/src/state(?:/|\.rs)",
    "^ecosystem/fret-imui/src/state(?:/|\.rs)"
)

$codeRules = @(
    @{
        Rule = "no-fret-query-import-in-primitives"
        Pattern = "\bfret_query::"
    },
    @{
        Rule = "no-fret-selector-import-in-primitives"
        Pattern = "\bfret_selector::"
    },
    @{
        Rule = "no-use-query-sugar-in-primitives"
        Pattern = "\.use_query(?:_async|_async_local)?\s*\("
    },
    @{
        Rule = "no-use-selector-sugar-in-primitives"
        Pattern = "\.use_selector(?:_keyed)?\s*\("
    }
)

$depPolicies = @(
    @{
        Name = "fret-query"
        Feature = "state-query"
    },
    @{
        Name = "fret-selector"
        Feature = "state-selector"
    }
)

$rootPaths = @()
foreach ($root in $sourceRoots) {
    $rootPaths += (Join-Path $repoRoot $root)
}

$sourceFiles = Get-ChildItem -Path $rootPaths -Recurse -Filter *.rs -File
foreach ($file in $sourceFiles) {
    $rel = Normalize-RepoPath -RepoRoot $repoRoot -Path $file.FullName
    if (Is-AllowListedPath -Path $rel -AllowRegexes $allowRegexes) {
        continue
    }

    foreach ($rule in $codeRules) {
        $matches = Select-String -Path $file.FullName -Pattern $rule.Pattern -AllMatches -CaseSensitive
        foreach ($match in $matches) {
            Write-Violation -Rule $rule.Rule -Path $rel -Detail ("line {0}: {1}" -f $match.LineNumber, $match.Line.Trim())
        }
    }
}

foreach ($manifest in $manifestPaths) {
    $fullPath = Join-Path $repoRoot $manifest
    if (-not (Test-Path $fullPath)) {
        continue
    }

    $rel = Normalize-RepoPath -RepoRoot $repoRoot -Path $fullPath
    $content = Get-Content -Raw -Path $fullPath

    foreach ($policy in $depPolicies) {
        $name = $policy.Name
        $feature = $policy.Feature
        $depPattern = "(?m)^\s*" + [Regex]::Escape($name) + "\s*=\s*(.+)$"
        $depMatches = [Regex]::Matches($content, $depPattern)

        if ($depMatches.Count -eq 0) {
            continue
        }

        foreach ($m in $depMatches) {
            $line = $m.Groups[0].Value.Trim()
            if ($line -notmatch "optional\s*=\s*true") {
                Write-Violation -Rule ("{0}-must-be-optional" -f $name) -Path $rel -Detail $line
            }
        }

        if ($content -notmatch ("(?m)^\s*" + [Regex]::Escape($feature) + "\s*=")) {
            Write-Violation -Rule ("missing-{0}-feature" -f $feature) -Path $rel -Detail ("dependency `{0}` exists but feature `{1}` is missing" -f $name, $feature)
        }
    }

    $hasStateSelector = $content -match "(?m)^\s*state-selector\s*="
    $hasStateQuery = $content -match "(?m)^\s*state-query\s*="
    if (($hasStateSelector -or $hasStateQuery) -and ($content -notmatch "(?m)^\s*state\s*=")) {
                Write-Violation -Rule "missing-state-umbrella-feature" -Path $rel -Detail 'define state = ["state-selector", "state-query"]'
    }
}

if ($script:HadErrors) {
    exit 1
}

Write-Host "Component state coupling check passed."