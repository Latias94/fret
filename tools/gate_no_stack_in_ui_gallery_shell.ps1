# Gate: prevent `stack::*` authoring helpers from drifting back into the UI gallery shell.
#
# Rationale:
# - The UI gallery's *shell* (navigation + layout chrome) is part of the repo's teaching surface.
# - Keep it aligned with the golden path layout authoring surface (`fret-ui-kit::ui::*` builders),
#   even while preview pages/snippets are migrated in batches.
#
# Scope (intentional):
# - Only scans `apps/fret-ui-gallery/src/ui/*.rs` (not `src/ui/previews/**`).
#
# Usage:
#   pwsh tools/gate_no_stack_in_ui_gallery_shell.ps1

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
        & rg -n --max-depth 1 --glob "*.rs" $pattern @Paths
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
        -Name "ui-gallery shell remains on the ui::* builder authoring surface" `
        -Paths @("apps/fret-ui-gallery/src/ui") `
        -Patterns @(
            "\\bstack::(hstack|vstack)(_build)?\\b",
            "\\buse\\s+fret_ui_kit::declarative::stack\\b",
            "\\buse\\s+fret_ui_shadcn::stack\\b",
            "\\bshadcn::stack::(hstack|vstack)\\b"
        )

    Write-Host "[gate-no-stack] ok"
    exit 0
} finally {
    Pop-Location
}
