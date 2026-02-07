Param(
    [switch]$SkipFmt,
    [switch]$SkipClippy,
    [switch]$SkipNextest
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")

function Invoke-Step {
    param(
        [Parameter(Mandatory = $true)][string]$Name,
        [Parameter(Mandatory = $true)][scriptblock]$Run
    )

    Write-Host ("==> {0}" -f $Name)
    & $Run
}

Invoke-Step -Name "ADR ID uniqueness" -Run {
    pwsh -NoProfile -File (Join-Path $repoRoot "tools/check_adr_numbers.ps1")
}

Invoke-Step -Name "Workspace layering policy" -Run {
    pwsh -NoProfile -File (Join-Path $repoRoot "tools/check_layering.ps1")
}

Invoke-Step -Name "Execution surface policy" -Run {
    pwsh -NoProfile -File (Join-Path $repoRoot "tools/check_execution_surface.ps1")
}

Invoke-Step -Name "Stringly command parsing policy" -Run {
    pwsh -NoProfile -File (Join-Path $repoRoot "tools/check_stringly_command_parsing.ps1")
}

if (-not $SkipFmt) {
    Invoke-Step -Name "cargo fmt --check" -Run {
        cargo fmt --all -- --check
    }
}

if (-not $SkipClippy) {
    Invoke-Step -Name "cargo clippy (workspace, all targets)" -Run {
        cargo clippy --workspace --all-targets -- -D warnings
    }
}

if (-not $SkipNextest) {
    $nextest = Get-Command cargo-nextest -ErrorAction SilentlyContinue
    if ($null -eq $nextest) {
        Write-Warning "cargo-nextest is not installed; falling back to cargo test --workspace"
        Invoke-Step -Name "cargo test --workspace" -Run {
            cargo test --workspace
        }
    } else {
        Invoke-Step -Name "cargo nextest run" -Run {
            cargo nextest run
        }
    }
}

Write-Host "Pre-release checks passed."
