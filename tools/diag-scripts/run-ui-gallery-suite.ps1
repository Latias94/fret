param(
  [string]$DiagDir = "target/fret-diag",
  [int]$TimeoutMs = 60000,
  [int]$PollMs = 50,
  [int]$StartupDelaySeconds = 3,
  [switch]$StartApp,
  [switch]$EnableInspect
)

$ErrorActionPreference = "Stop"

$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\\..")
Set-Location $RepoRoot

$env:FRET_DIAG_DIR = $DiagDir

if ($StartApp) {
  Write-Host "Starting UI Gallery (diagnostics enabled)..."
  $app = Start-Process -FilePath "cargo" -ArgumentList @("run", "-p", "fret-ui-gallery") -WorkingDirectory $RepoRoot -PassThru
  Write-Host "UI Gallery pid=$($app.Id)"
  Start-Sleep -Seconds ([Math]::Max(0, $StartupDelaySeconds))

  if ($EnableInspect) {
    & cargo run -p fretboard -- diag inspect on --dir $DiagDir | Out-Host
  }
} else {
  Write-Host "Assuming the app is already running (UI Gallery or another diagnostics-enabled app)."
}

Write-Host "Running diag suite: ui-gallery"
& cargo run -p fretboard -- diag suite ui-gallery --dir $DiagDir --timeout-ms $TimeoutMs --poll-ms $PollMs | Out-Host
exit $LASTEXITCODE

