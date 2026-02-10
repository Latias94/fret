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
    # Avoid cross-run contamination: `diag run` uses `--dir` as a stateful rendezvous with the
    # launched demo process. Reusing the same directory across runs can cause scripts to talk to
    # a different (previous) demo instance or to pick up stale bundles.
    $runId = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
    $runOutDir = Join-Path $OutDir $runId

    # Build the demo once and launch the produced exe for diag runs. Avoid nested `cargo run` under
    # `--launch`, which can exceed the script timeout on cold builds.
    if ($Release) {
        & cargo build -j 1 -p fret-demo --bin imui_floating_windows_demo --release
    } else {
        & cargo build -j 1 -p fret-demo --bin imui_floating_windows_demo
    }
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
        @{
            Path = "tools/diag-scripts/imui-float-window-titlebar-drag-screenshots.json"
            ExtraArgs = @(
                "--check-stale-paint", "imui-float-demo.a.activate",
                "--check-stale-paint-eps", "0.5",
                "--env", "FRET_DIAG_SCREENSHOTS=1",
                "--env", "FRET_DIAG_REDACT_TEXT=0"
            )
        },
        @{
            Path = "tools/diag-scripts/imui-float-window-text-wrap-no-overlap-150.json"
            ExtraArgs = @()
        },
        @{
            Path = "tools/diag-scripts/imui-float-window-drag-resize-context-menu.json"
            ExtraArgs = @()
        },
        @{
            Path = "tools/diag-scripts/imui-float-window-select-popup-coexistence.json"
            ExtraArgs = @()
        },
        @{
            Path = "tools/diag-scripts/imui-float-window-activate-on-content-bring-to-front.json"
            ExtraArgs = @()
        }
    )

    foreach ($script in $scripts) {
        $scriptPath = $script.Path
        $extraArgs = $script.ExtraArgs
        $scriptName = [IO.Path]::GetFileNameWithoutExtension($scriptPath)
        $scriptOutDir = Join-Path $runOutDir $scriptName
        & cargo run -j 1 -p fretboard -- `
            diag run $scriptPath `
            --dir $scriptOutDir `
            --timeout-ms $TimeoutMs `
            --poll-ms $PollMs `
            @extraArgs `
            --pack `
            --env "FRET_DIAG_SEMANTICS=1" `
            --launch -- $demoExe
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    }

    exit 0
} finally {
    Pop-Location
}
