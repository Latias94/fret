# Gate: prevent legacy MVU usage from drifting back into the cookbook.
#
# Usage:
#   pwsh tools/gate_no_mvu_in_cookbook.ps1

[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$workspaceRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path

function Assert-NoMatches(
    [string]$Name,
    [string[]]$Paths,
    [string[]]$Patterns
) {
    Write-Host "[gate-no-mvu] $Name"
    $failed = $false

    foreach ($pattern in $Patterns) {
        & rg -n $pattern @Paths
        if ($LASTEXITCODE -eq 0) {
            Write-Host "[gate-no-mvu] FAIL: pattern matched: $pattern"
            $failed = $true
        } elseif ($LASTEXITCODE -eq 1) {
            # No matches.
        } else {
            throw "rg failed (exit code: $LASTEXITCODE) for pattern: $pattern"
        }
    }

    if ($failed) {
        throw "Gate failed: $Name"
    }
}

Push-Location $workspaceRoot
try {
    Assert-NoMatches `
        -Name "cookbook remains action-first authoring" `
        -Paths @("apps/fret-cookbook") `
        -Patterns @(
            "\\bimpl\\s+MvuProgram\\b",
            "\\brun_mvu\\b",
            "\\bMessageRouter\\b",
            "\\bKeyedMessageRouter\\b",
            "\\bfret::mvu\\b",
            "\\bfret::mvu_router\\b"
        )

    Write-Host "[gate-no-mvu] ok"
    exit 0
} finally {
    Pop-Location
}

