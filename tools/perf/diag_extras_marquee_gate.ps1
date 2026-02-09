param(
  [Parameter(Mandatory = $true)]
  [string]$Baseline,

  [string]$OutDir = "",
  [string]$LaunchBin = "target/release/extras_marquee_perf_demo",
  [int]$TimeoutMs = 300000,
  [int]$Repeat = 7,
  [int]$WarmupFrames = 60
)

$ErrorActionPreference = "Stop"

function Usage {
  @"
Usage:
  pwsh -File tools/perf/diag_extras_marquee_gate.ps1 `
    -Baseline <path> `
    [-OutDir <path>] `
    [-LaunchBin <path>] `
    [-TimeoutMs <n>] `
    [-Repeat <n>] `
    [-WarmupFrames <n>]

Notes:
  - Runs the `extras-marquee-steady` perf suite via `fretboard diag perf`.
  - Baselines are machine-dependent; generate one via:
      cargo run -p fretboard -- diag perf extras-marquee-steady `
        --repeat 7 --warmup-frames 5 `
        --perf-baseline-out docs/workstreams/perf-baselines/extras-marquee-steady.<machine-tag>.v1.json `
        --perf-baseline-headroom-pct 20 `
        --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 `
        --env FRET_DIAG_SEMANTICS=0 `
        --launch -- target/release/extras_marquee_perf_demo
"@ | Write-Host
}

if ([string]::IsNullOrWhiteSpace($OutDir)) {
  $ts = [int](Get-Date -UFormat %s)
  $OutDir = "target/fret-diag-perf/extras-marquee-steady.$ts"
}

if (-not (Test-Path $Baseline)) {
  Write-Host "error: baseline not found: $Baseline"
  Write-Host ""
  Usage
  exit 2
}

New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

Write-Host "[gate] extras-marquee-steady -> $OutDir"
Write-Host "[gate] baseline: $Baseline"
Write-Host "[gate] launch-bin: $LaunchBin"

# Ensure the demo binary exists (fast no-op if already built).
& cargo build -q -p fret-demo --release --bin extras_marquee_perf_demo

$cmd = @(
  "cargo", "run", "-q", "-p", "fretboard", "--",
  "diag", "perf", "extras-marquee-steady",
  "--dir", $OutDir,
  "--timeout-ms", $TimeoutMs,
  "--reuse-launch",
  "--repeat", $Repeat,
  "--warmup-frames", $WarmupFrames,
  "--sort", "time",
  "--top", "15",
  "--json",
  "--perf-baseline", $Baseline,
  "--env", "FRET_DIAG_SCRIPT_AUTO_DUMP=0",
  "--env", "FRET_DIAG_SEMANTICS=0",
  "--launch", "--", $LaunchBin
)

Write-Host ("[gate] cmd: " + ($cmd -join " "))

$stdoutPath = Join-Path $OutDir "stdout.json"
$stderrPath = Join-Path $OutDir "stderr.log"

& $cmd[0] $cmd[1..($cmd.Length - 1)] 1> $stdoutPath 2> $stderrPath
$rc = $LASTEXITCODE
if ($rc -ne 0) {
  Write-Host "FAIL (rc=$rc). See: $stderrPath"
  exit $rc
}

$checkFile = Join-Path $OutDir "check.perf_thresholds.json"
if (-not (Test-Path $checkFile)) {
  Write-Host "FAIL (missing $checkFile). See: $stderrPath"
  exit 1
}

$checkJson = Get-Content $checkFile -Raw | ConvertFrom-Json
$failures = @()
if ($null -ne $checkJson.failures) {
  $failures = @($checkJson.failures)
}

if ($failures.Count -ne 0) {
  Write-Host "FAIL (perf threshold failures=$($failures.Count)). See: $checkFile"
  exit 1
}

Write-Host "PASS (extras-marquee-steady)"
