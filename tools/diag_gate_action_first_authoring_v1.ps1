# Run a small, CI-friendly diagnostics gate set for the Action-first authoring v1 workstream:
# - commands/keymap typed-action dispatch + availability gating
# - overlay modal barrier shortcut gating
# - cross-frontend action dispatch (declarative + GenUI + imui)
# - editor-grade surface adoption (workspace shell demo tab close dispatch trace)
#
# Usage:
#   pwsh tools/diag_gate_action_first_authoring_v1.ps1
#   pwsh tools/diag_gate_action_first_authoring_v1.ps1 -OutDir target/fret-diag-afa-v1 -Release

[CmdletBinding()]
param(
    [string] $OutDir = "target/fret-diag-action-first-authoring-v1",
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

    $gates = @(
        @{
            Name = "cookbook-commands-keymap-basics-shortcut-and-gating"
            ScriptPath = "tools/diag-scripts/cookbook/commands-keymap-basics/cookbook-commands-keymap-basics-shortcut-and-gating.json"
            ExampleName = "commands_keymap_basics"
        },
        @{
            Name = "cookbook-overlay-basics-modal-barrier-shortcut-gating"
            ScriptPath = "tools/diag-scripts/cookbook/overlay-basics/cookbook-overlay-basics-modal-barrier-shortcut-gating.json"
            ExampleName = "overlay_basics"
        },
        @{
            Name = "cookbook-imui-action-basics-cross-frontend"
            ScriptPath = "tools/diag-scripts/cookbook/imui-action-basics/cookbook-imui-action-basics-cross-frontend.json"
            ExampleName = "imui_action_basics"
        },
        @{
            Name = "workspace-shell-demo-tab-close-button-command-dispatch-trace"
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

        $scriptOutDir = Join-Path $runOutDir $gateName
        Invoke-Checked "fretboard diag run $gateName" "cargo" @(
            "run",
            "-j",
            "1",
            "-p",
            "fretboard",
            "--",
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
