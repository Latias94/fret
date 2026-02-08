# Run a small, CI-friendly imui v3 gate matrix:
# - nextest contracts (imui + docking handshake)
# - cheap perf guards (source-level smoke tests)
# - scripted diag coverage for floating/popup coexistence
#
# Usage:
#   pwsh tools/diag_gate_imui_v3.ps1
#   pwsh tools/diag_gate_imui_v3.ps1 -OutDir target/fret-diag-imui-v3 -Release

[CmdletBinding()]
param(
    [string] $OutDir = "target/fret-diag-imui-v3",
    [int] $TimeoutMs = 180000,
    [int] $PollMs = 50,
    [switch] $Release
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$workspaceRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path

Push-Location $workspaceRoot
try {
    # Build the demo once and launch the produced exe for diag runs. Avoid nested `cargo run` under
    # `--launch`, which can exceed the script timeout on cold builds.
    $demoBuild = @("cargo", "build", "-j", "1", "-p", "fret-demo", "--bin", "imui_floating_windows_demo")
    if ($Release) { $demoBuild += "--release" }

    & $demoBuild
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    $demoExe = if ($Release) { "target/release/imui_floating_windows_demo.exe" } else { "target/debug/imui_floating_windows_demo.exe" }
    if (!(Test-Path $demoExe)) {
        throw "imui demo exe not found: $demoExe"
    }

    & cargo nextest run -p fret-imui
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    & cargo nextest run -p fret-ui-kit --features imui --test imui_response_contract_smoke --test imui_adapter_seam_smoke --test imui_perf_guard_smoke
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    & cargo nextest run -p fret-docking --features imui --test imui_handshake_smoke
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    $scripts = @(
        "tools/diag-scripts/imui-float-window-drag-resize-context-menu.json",
        "tools/diag-scripts/imui-float-window-select-popup-coexistence.json",
        "tools/diag-scripts/imui-float-window-activate-on-content-bring-to-front.json"
    )

    foreach ($script in $scripts) {
        & cargo run -j 1 -p fretboard -- `
            diag run $script `
            --dir $OutDir `
            --timeout-ms $TimeoutMs `
            --poll-ms $PollMs `
            --pack `
            --env "FRET_DIAG_SEMANTICS=1" `
            --launch -- $demoExe
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    }

    exit 0
} finally {
    Pop-Location
}
