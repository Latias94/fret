# Gate wrapper (PowerShell): prefer the Python implementation for portability.
#
# Usage:
#   pwsh tools/gate_fret_launch_surface_contract.ps1

[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$workspaceRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path

Push-Location $workspaceRoot
try {
    python tools/gate_fret_launch_surface_contract.py
    exit $LASTEXITCODE
} finally {
    Pop-Location
}
