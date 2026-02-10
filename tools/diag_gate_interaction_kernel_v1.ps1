# Run a small, CI-friendly gate matrix for the interaction-kernel v1 workstream:
# - kernel unit tests (including runtime-backed drag session helpers)
# - imui floating windows regression scripts (drag/resize + DPI + stale paint)
# - fret-node conformance tests (viewport + thresholds)
#
# Usage:
#   pwsh tools/diag_gate_interaction_kernel_v1.ps1
#   pwsh tools/diag_gate_interaction_kernel_v1.ps1 -OutDir target/fret-diag-interaction-kernel-v1 -Release

[CmdletBinding()]
param(
    [string] $OutDir = "target/fret-diag-interaction-kernel-v1",
    [int] $TimeoutMs = 180000,
    [int] $PollMs = 50,
    [switch] $Release
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$workspaceRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path

Push-Location $workspaceRoot
try {
    & cargo nextest run -p fret-interaction --features runtime
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    & cargo nextest run -p fret-ui-kit --features imui
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    & cargo nextest run -p fret-node
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    $demoBuild = @("cargo", "build", "-j", "1", "-p", "fret-demo", "--bin", "imui_floating_windows_demo")
    if ($Release) { $demoBuild += "--release" }
    & $demoBuild
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    $demoExe = if ($Release) { "target/release/imui_floating_windows_demo.exe" } else { "target/debug/imui_floating_windows_demo.exe" }
    if (!(Test-Path $demoExe)) {
        throw "imui floating windows demo exe not found: $demoExe"
    }

    $scripts = @(
        @{
            Path = "tools/diag-scripts/imui-float-window-titlebar-drag-screenshots.json"
            ExtraArgs = @(
                "--check-stale-paint", "imui-float-demo.a.activate",
                "--check-stale-paint-eps", "0.5",
                "--env", "FRET_DIAG_SCREENSHOTS=1",
                "--env", "FRET_DIAG_REDACT_TEXT=0"
            )
            DemoExe = $demoExe
        },
        @{
            Path = "tools/diag-scripts/imui-float-window-text-wrap-no-overlap-150.json"
            ExtraArgs = @()
            DemoExe = $demoExe
        }
    )

    foreach ($script in $scripts) {
        $scriptPath = $script.Path
        $extraArgs = $script.ExtraArgs
        $launchExe = $script.DemoExe
        & cargo run -j 1 -p fretboard -- `
            diag run $scriptPath `
            --dir $OutDir `
            --timeout-ms $TimeoutMs `
            --poll-ms $PollMs `
            @extraArgs `
            --pack `
            --env "FRET_DIAG_SEMANTICS=1" `
            --launch -- $launchExe
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    }

    # M3 repro: multi-window hover arbitration during dock drag.
    $editorBuild = @("cargo", "build", "-j", "1", "-p", "fret-demo", "--bin", "imui_editor_proof_demo")
    if ($Release) { $editorBuild += "--release" }
    & $editorBuild
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    $editorExe = if ($Release) { "target/release/imui_editor_proof_demo.exe" } else { "target/debug/imui_editor_proof_demo.exe" }
    if (!(Test-Path $editorExe)) {
        throw "imui editor proof demo exe not found: $editorExe"
    }

    & cargo run -j 1 -p fretboard -- `
        diag run tools/diag-scripts/imui-editor-proof-multiwindow-overlap-topmost-hover.json `
        --dir $OutDir `
        --timeout-ms $TimeoutMs `
        --poll-ms $PollMs `
        --check-dock-drag-min 1 `
        --check-dock-drag-source-windows-min 2 `
        --pack `
        --env "FRET_DIAG_SEMANTICS=1" `
        --launch -- $editorExe
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    exit 0
} finally {
    Pop-Location
}
