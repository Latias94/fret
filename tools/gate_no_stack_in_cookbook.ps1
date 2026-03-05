# Gate: prevent `stack::*` authoring helpers from drifting back into the cookbook.
#
# Rationale:
# - The cookbook should teach the "golden path" authoring surface (`fret-ui-kit::ui::*` builders),
#   not the older `declarative::stack::*` helpers that require an outer `cx` and props plumbing.
#
# Usage:
#   pwsh tools/gate_no_stack_in_cookbook.ps1

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
    Write-Host "[gate-no-stack] $Name"
    $failed = $false

    foreach ($pattern in $Patterns) {
        & rg -n $pattern @Paths
        if ($LASTEXITCODE -eq 0) {
            Write-Host "[gate-no-stack] FAIL: pattern matched: $pattern"
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
        -Name "cookbook remains on the ui::* builder authoring surface" `
        -Paths @("apps/fret-cookbook") `
        -Patterns @(
            "\\bstack::(hstack|vstack)(_build)?\\b",
            "\\buse\\s+fret_ui_kit::declarative::stack\\b",
            "\\buse\\s+fret_ui_shadcn::stack\\b"
        )

    Write-Host "[gate-no-stack] ok"
    exit 0
} finally {
    Pop-Location
}

