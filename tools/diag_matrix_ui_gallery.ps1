# Run the UI Gallery scripted regression matrix (uncached vs cached) with shell reuse enabled.
#
# This is intended for CI/automation and AI-driven regression checks:
# - Forces view-cache reuse to be exercised (`FRET_UI_GALLERY_VIEW_CACHE_SHELL=1`).
# - Gates on view-cache reuse events and (overlay-only) cached-synthesis events.
# - Compares bundles via stable semantics anchors (test_id) and ignores paint fingerprint/bounds by default.
#
# Usage:
#   pwsh tools/diag_matrix_ui_gallery.ps1
#   pwsh tools/diag_matrix_ui_gallery.ps1 -OutDir target/fret-diag -WarmupFrames 5 -Release -Json

[CmdletBinding()]
param(
    [string] $OutDir = "target/fret-diag",
    [int] $WarmupFrames = 5,
    [int] $TimeoutMs = 180000,
    [int] $PollMs = 50,
    [switch] $Release,
    [switch] $Json
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$workspaceRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path

Push-Location $workspaceRoot
try {
    $launch = @("cargo", "run", "-p", "fret-ui-gallery")
    if ($Release) {
        $launch += "--release"
    }

    $args = @(
        "run", "-p", "fretboard", "--",
        "diag", "matrix", "ui-gallery",
        "--dir", $OutDir,
        "--timeout-ms", $TimeoutMs,
        "--poll-ms", $PollMs,
        "--warmup-frames", $WarmupFrames,
        "--compare-ignore-bounds",
        "--compare-ignore-scene-fingerprint",
        "--check-view-cache-reuse-min", "1",
        "--check-overlay-synthesis-min", "1",
        "--env", "FRET_UI_GALLERY_VIEW_CACHE_SHELL=1",
        "--env", "FRET_DIAG_SEMANTICS=1",
        "--launch", "--"
    ) + $launch

    if ($Json) {
        $args = $args + @("--json")
    }

    & cargo @args
    exit $LASTEXITCODE
} finally {
    Pop-Location
}

