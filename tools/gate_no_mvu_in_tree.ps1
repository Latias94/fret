# Gate: prevent legacy MVU identifiers from drifting back into the repo.
#
# This is a coarse but practical “hard delete” follow-up gate (M9).
#
# Usage:
#   pwsh tools/gate_no_mvu_in_tree.ps1

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
    Write-Host "[gate-no-mvu-in-tree] $Name"
    $failed = $false

    foreach ($pattern in $Patterns) {
        & rg -n $pattern @Paths
        if ($LASTEXITCODE -eq 0) {
            Write-Host "[gate-no-mvu-in-tree] FAIL: pattern matched: $pattern"
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
        -Name "repo remains MVU-free (code + scaffolds)" `
        -Paths @("apps", "crates", "ecosystem") `
        -Patterns @(
            "\\bimpl\\s+MvuProgram\\b",
            "\\bMvuProgram\\b",
            "\\bMessageRouter\\b",
            "\\bKeyedMessageRouter\\b",
            "\\brun_mvu\\b",
            "\\bMvuWindowState\\b",
            "\\bfret::mvu\\b",
            "\\bfret::mvu_router\\b",
            "\\bfret::legacy\\b",
            "\\blegacy[-_]mvu\\b"
        )

    Write-Host "[gate-no-mvu-in-tree] ok"
    exit 0
} finally {
    Pop-Location
}

