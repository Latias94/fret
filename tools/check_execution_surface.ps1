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

function IsAllowedPath {
    param(
        [Parameter(Mandatory = $true)][string]$Path,
        [Parameter(Mandatory = $true)][string[]]$AllowList
    )

    foreach ($allowed in $AllowList) {
        if ($allowed.EndsWith("/")) {
            if ($Path.StartsWith($allowed)) {
                return $true
            }
            continue
        }
        if ($Path -eq $allowed) {
            return $true
        }
    }

    return $false
}

function Write-Violation {
    param(
        [Parameter(Mandatory = $true)][string]$Rule,
        [Parameter(Mandatory = $true)][string]$Path,
        [Parameter(Mandatory = $true)][int]$LineNumber,
        [Parameter(Mandatory = $true)][string]$Line
    )

    Write-Error -ErrorAction Continue ("Execution surface violation ({0}): {1}:{2}: {3}" -f $Rule, $Path, $LineNumber, $Line.Trim())
    $script:HadErrors = $true
}

$script:HadErrors = $false

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$roots = @("crates", "ecosystem", "apps")

$rules = @(
    @{
        Rule      = "no-raw-thread-spawn"
        Pattern   = "\b(std::thread::spawn|thread::spawn)\b"
        Scope     = @("crates/", "ecosystem/", "apps/")
        AllowList = @(
            "crates/fret-launch/src/runner/desktop/dispatcher.rs",
            "crates/fret-launch/src/runner/desktop/hotpatch.rs"
        )
    },
    @{
        Rule      = "no-raw-thread-sleep"
        Pattern   = "\b(std::thread::sleep|thread::sleep)\b"
        Scope     = @("crates/", "ecosystem/", "apps/")
        AllowList = @(
            "apps/fretboard/",
            "crates/fret-launch/src/runner/desktop/hotpatch.rs"
        )
    },
    @{
        Rule      = "no-bespoke-channels"
        Pattern   = "\b(std::sync::mpsc|crossbeam_channel|async_channel|flume)\b"
        Scope     = @("ecosystem/", "apps/")
        AllowList = @()
    },
    @{
        Rule      = "no-bespoke-futures-channels"
        Pattern   = "\b(futures::channel::(mpsc|oneshot)|futures_channel)\b"
        Scope     = @("ecosystem/", "apps/")
        AllowList = @()
    },
    @{
        Rule      = "no-split-brain-timers"
        Pattern   = "\b(gloo_timers|futures_timer|wasm_timer|tokio::time::sleep|async_std::task::sleep)\b"
        Scope     = @("ecosystem/", "apps/")
        AllowList = @()
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
        $inScope = $false
        foreach ($prefix in $rule.Scope) {
            if ($rel.StartsWith($prefix)) {
                $inScope = $true
                break
            }
        }
        if (-not $inScope) {
            continue
        }

        $matches = Select-String -Path $file.FullName -Pattern $rule.Pattern -AllMatches
        foreach ($match in $matches) {
            if (IsAllowedPath -Path $rel -AllowList $rule.AllowList) {
                continue
            }
            Write-Violation -Rule $rule.Rule -Path $rel -LineNumber $match.LineNumber -Line $match.Line
        }
    }
}

if ($script:HadErrors) {
    exit 1
}

Write-Host "Execution surface check passed."
