# Gate: prevent legacy `declarative::stack` helpers from being re-exposed as public API.
#
# Rationale:
# - Teaching surfaces have converged on `fret-ui-kit::ui::*` builders.
# - The old `declarative::stack` helpers are kept temporarily for internal implementation code,
#   but should not be publicly exported (or reintroduced into `prelude::*`).
#
# Usage:
#   pwsh tools/gate_no_public_stack_in_ui_kit.ps1

[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$workspaceRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path

function Assert-NoFileMatches(
    [string]$Name,
    [string]$Path,
    [string[]]$Patterns
) {
    Write-Host "[gate-no-public-stack] $Name"

    if (-not (Test-Path $Path)) {
        throw "File not found: $Path"
    }

    $failed = $false
    foreach ($pattern in $Patterns) {
        $hits = Select-String -Path $Path -Pattern $pattern
        if ($hits) {
            Write-Host "[gate-no-public-stack] FAIL: $pattern"
            foreach ($hit in ($hits | Select-Object -First 20)) {
                Write-Host ("  - {0}:{1}" -f $hit.Path, $hit.LineNumber)
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
    Assert-NoFileMatches `
        -Name "ui-kit prelude does not export declarative::stack" `
        -Path "ecosystem/fret-ui-kit/src/lib.rs" `
        -Patterns @(
            "\\bpub\\s+use\\s+crate::declarative::stack\\b",
            "\\bpub\\s+use\\s+crate::declarative::\\{\\s*stack\\b",
            "\\bpub\\s+use\\s+crate::declarative::\\{[^\\}]*\\bstack\\b"
        )

    Assert-NoFileMatches `
        -Name "ui-kit declarative module does not expose a public stack module" `
        -Path "ecosystem/fret-ui-kit/src/declarative/mod.rs" `
        -Patterns @(
            "\\bpub\\s+mod\\s+stack\\b",
            "\\bpub\\(crate\\)\\s+mod\\s+stack\\b",
            "\\bpub\\(super\\)\\s+mod\\s+stack\\b"
        )

    Write-Host "[gate-no-public-stack] ok"
    exit 0
} finally {
    Pop-Location
}

