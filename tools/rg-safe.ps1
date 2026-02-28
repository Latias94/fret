# Bounded ripgrep wrapper for the Fret workspace.
#
# Purpose: avoid accidental output explosions by excluding diagnostics bundle artifacts
# (e.g. `target/fret-diag/**/bundle.json`) from repository searches.
#
# Usage:
#   tools/rg-safe.ps1 <rg args...>
# Example:
#   tools/rg-safe.ps1 -n -F "Demo"
#
# Notes:
# - This script still delegates to `rg` (ripgrep). Ensure it is installed and on PATH.
# - If you intentionally want to search bundle artifacts, call `rg` directly.

param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$Args
)

$ErrorActionPreference = "Stop"

$rg = Get-Command rg -ErrorAction SilentlyContinue
if (-not $rg) {
    Write-Error "rg (ripgrep) not found in PATH"
    exit 2
}

$excludes = @(
    "--glob", "!target/fret-diag/**",
    "--glob", "!.fret/diag/**",
    "--glob", "!**/bundle.json",
    "--glob", "!**/*.bundle.json",
    "--glob", "!**/bundle.schema2.json",
    "--glob", "!**/*.bundle.schema2.json"
)

& rg @Args @excludes
exit $LASTEXITCODE

