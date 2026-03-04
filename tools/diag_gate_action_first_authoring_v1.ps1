# Run a small, CI-friendly diagnostics gate set for the Action-first authoring v1 workstream:
# - commands/keymap typed-action dispatch + availability gating
# - overlay modal barrier shortcut gating
# - cross-frontend action dispatch (declarative + GenUI + imui)
# - editor-grade surface adoption (workspace shell demo tab close dispatch trace)
#
# Usage:
#   pwsh tools/diag_gate_action_first_authoring_v1.ps1
#   pwsh tools/diag_gate_action_first_authoring_v1.ps1 -OutDir target/dfa-v1 -Release

[CmdletBinding()]
param(
    # Keep the base output directory short to avoid Windows path-length issues when bundles are
    # dumped (schema2 + chunking can create deep directory trees).
    [string] $OutDir = "target/dfa-v1",
    [int] $TimeoutMs = 180000,
    [int] $PollMs = 50,
    [switch] $Release
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Invoke-Checked(
    [string]$Name,
    [string]$Program,
    [string[]]$Arguments
) {
    Write-Host "[diag-gate-afa-v1] $Name"
    & $Program @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "Step failed: $Name (exit code: $LASTEXITCODE)"
    }
}

$workspaceRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path

Push-Location $workspaceRoot
try {
    # Avoid cross-run contamination: diag uses `--dir` as a stateful rendezvous with the launched
    # demo process. Reusing the same directory across runs can cause scripts to talk to a different
    # (previous) demo instance or to pick up stale bundles.
    $runId = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
    $runOutDir = Join-Path $OutDir $runId

    $profileDir = if ($Release) { "release" } else { "debug" }

    # Build fretboard once and invoke the built exe directly.
    #
    # Rationale: `cargo run -p fretboard` can dominate gate runtime (and make timeouts flaky)
    # even when the diagnostics scripts themselves are fast.
    $fretboardBuildArgs = @("build", "-j", "1", "-p", "fretboard")
    if ($Release) {
        $fretboardBuildArgs += "--release"
    }
    Invoke-Checked "cargo build -p fretboard" "cargo" $fretboardBuildArgs

    $fretboardExe = Join-Path $workspaceRoot (Join-Path "target" (Join-Path $profileDir "fretboard.exe"))
    if (!(Test-Path $fretboardExe)) {
        throw "fretboard exe not found: $fretboardExe"
    }

    $gates = @(
        @{
            Name = "cookbook-hello-click-count"
            DirName = "h"
            ScriptPath = "tools/diag-scripts/cookbook/hello/cookbook-hello-click-count.json"
            ExampleName = "hello"
        },
        @{
            Name = "cookbook-commands-keymap-basics-shortcut-and-gating"
            DirName = "k"
            ScriptPath = "tools/diag-scripts/cookbook/commands-keymap-basics/cookbook-commands-keymap-basics-shortcut-and-gating.json"
            ExampleName = "commands_keymap_basics"
        },
        @{
            Name = "cookbook-overlay-basics-modal-barrier-shortcut-gating"
            DirName = "o"
            ScriptPath = "tools/diag-scripts/cookbook/overlay-basics/cookbook-overlay-basics-modal-barrier-shortcut-gating.json"
            ExampleName = "overlay_basics"
        },
        @{
            Name = "cookbook-imui-action-basics-cross-frontend"
            DirName = "i"
            ScriptPath = "tools/diag-scripts/cookbook/imui-action-basics/cookbook-imui-action-basics-cross-frontend.json"
            ExampleName = "imui_action_basics"
        },
        @{
            Name = "cookbook-payload-actions-basics-remove"
            DirName = "p"
            ScriptPath = "tools/diag-scripts/cookbook/payload-actions-basics/cookbook-payload-actions-basics-remove.json"
            ExampleName = "payload_actions_basics"
        },
        @{
            Name = "workspace-shell-demo-tab-close-button-command-dispatch-trace"
            DirName = "w"
            ScriptPath = "tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-close-button-closes-tab-smoke.json"
            PackageName = "fret-demo"
            BinName = "workspace_shell_demo"
        }
    )

    foreach ($gate in $gates) {
        $scriptPath = $gate.ScriptPath
        $gateName = $gate.Name
        $demoExe = $null

        if ($gate.ContainsKey("ExampleName")) {
            $exampleName = $gate.ExampleName
            $buildArgs = @("build", "-j", "1", "-p", "fret-cookbook", "--example", $exampleName)
            if ($Release) {
                $buildArgs += "--release"
            }
            Invoke-Checked "cargo build -p fret-cookbook --example $exampleName" "cargo" $buildArgs

            $demoExe = Join-Path $workspaceRoot (Join-Path "target" (Join-Path $profileDir (Join-Path "examples" "$exampleName.exe")))
            if (!(Test-Path $demoExe)) {
                throw "cookbook example exe not found: $demoExe"
            }
        } elseif ($gate.ContainsKey("PackageName") -and $gate.ContainsKey("BinName")) {
            $packageName = $gate.PackageName
            $binName = $gate.BinName
            $buildArgs = @("build", "-j", "1", "-p", $packageName, "--bin", $binName)
            if ($Release) {
                $buildArgs += "--release"
            }
            Invoke-Checked "cargo build -p $packageName --bin $binName" "cargo" $buildArgs

            $demoExe = Join-Path $workspaceRoot (Join-Path "target" (Join-Path $profileDir "$binName.exe"))
            if (!(Test-Path $demoExe)) {
                throw "demo exe not found: $demoExe"
            }
        } else {
            throw "Invalid gate entry (missing ExampleName or (PackageName + BinName)): $($gateName)"
        }

        # Keep directory names short to avoid Windows path-length issues when bundles are dumped.
        $scriptOutDir = Join-Path $runOutDir $gate.DirName
        Invoke-Checked "fretboard diag run $gateName" $fretboardExe @(
            "diag",
            "run",
            $scriptPath,
            "--dir",
            $scriptOutDir,
            "--timeout-ms",
            "$TimeoutMs",
            "--poll-ms",
            "$PollMs",
            "--pack",
            "--env",
            "FRET_DIAG_SEMANTICS=1",
            "--env",
            "FRET_DIAG_REDACT_TEXT=0",
            "--env",
            "FRET_DIAG_FIXED_FRAME_DELTA_MS=16",
            "--env",
            "RUST_LOG=warn",
            "--launch",
            "--",
            $demoExe
        )
    }

    Write-Host "[diag-gate-afa-v1] done (out_dir=$runOutDir)"
    exit 0
} finally {
    Pop-Location
}
