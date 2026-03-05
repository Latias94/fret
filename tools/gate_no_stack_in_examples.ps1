# Gate: prevent `stack::*` authoring helpers from drifting back into `apps/fret-examples`.
#
# Rationale:
# - Examples should teach the "golden path" authoring surface (`fret-ui-kit::ui::*` builders),
#   not the older `declarative::stack::*` helpers or `shadcn::stack::*` shortcuts.
#
# Usage:
#   pwsh tools/gate_no_stack_in_examples.ps1

[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$workspaceRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$targetRoot = Join-Path $workspaceRoot "apps/fret-examples"

$patterns = @(
    "\\bstack::(hstack|vstack)(_build)?\\b",
    "\\bshadcn::stack::(hstack|vstack)\\b",
    "\\buse\\s+fret_ui_kit::declarative::stack\\b",
    "\\buse\\s+fret_ui_shadcn::stack\\b"
)

function Assert-NoMatches(
    [string]$Name,
    [string]$Root,
    [string[]]$Patterns
) {
    Write-Host "[gate-no-stack] $Name"

    $files = Get-ChildItem -Path $Root -Recurse -File -Filter *.rs
    if (-not $files) {
        throw "No .rs files found under: $Root"
    }

    $failed = $false
    foreach ($pattern in $Patterns) {
        $hits = Select-String -Path $files.FullName -Pattern $pattern -List
        if ($hits) {
            Write-Host "[gate-no-stack] FAIL: pattern matched: $pattern"
            $hitsToPrint = $hits | Select-Object -First 20
            foreach ($hit in $hitsToPrint) {
                Write-Host ("  - {0}:{1}" -f $hit.Path, $hit.LineNumber)
            }
            if ($hits.Count -gt $hitsToPrint.Count) {
                Write-Host ("  ... and {0} more" -f ($hits.Count - $hitsToPrint.Count))
            }
            $failed = $true
        }
    }

    if ($failed) {
        throw "Gate failed: $Name"
    }
}

Push-Location $workspaceRoot
try {
    Assert-NoMatches `
        -Name "examples remain on the ui::* builder authoring surface" `
        -Root $targetRoot `
        -Patterns $patterns

    Write-Host "[gate-no-stack] ok"
    exit 0
} finally {
    Pop-Location
}

