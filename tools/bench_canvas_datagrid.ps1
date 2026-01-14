param(
  [switch]$Release,
  [int]$ExitAfterFrames = 600,
  [int]$StatsWindow = 240,
  [switch]$AutoScroll = $true,
  [string]$OutDir = ""
)

$ErrorActionPreference = "Stop"

function Get-TimestampFolder() {
  return (Get-Date -Format "yyyyMMdd-HHmmss")
}

function Ensure-Dir([string]$Path) {
  if (-not (Test-Path $Path)) {
    New-Item -ItemType Directory -Path $Path | Out-Null
  }
}

function Set-Env([hashtable]$Vars, [ref]$Prev) {
  $Prev.Value = @{}
  foreach ($k in $Vars.Keys) {
    $Prev.Value[$k] = [Environment]::GetEnvironmentVariable($k)
    [Environment]::SetEnvironmentVariable($k, [string]$Vars[$k])
  }
}

function Restore-Env([hashtable]$Prev) {
  foreach ($k in $Prev.Keys) {
    [Environment]::SetEnvironmentVariable($k, $Prev[$k])
  }
}

function Parse-LastStatsLine([string]$LogPath) {
  $regex = [regex]'datagrid_canvas_stats: samples=(\d+) total_avg=([\d\.]+)ms total_p95=([\d\.]+)ms'
  $matches = Select-String -Path $LogPath -Pattern $regex -AllMatches
  if ($null -eq $matches -or $matches.Count -eq 0) {
    return $null
  }

  $last = $matches[-1].Matches[-1]
  return @{
    samples = [int]$last.Groups[1].Value
    total_avg_ms = [double]$last.Groups[2].Value
    total_p95_ms = [double]$last.Groups[3].Value
  }
}

function Parse-LastRendererPerfLine([string]$LogPath) {
  $regex = [regex]'renderer_perf: frames=(\d+) encode=([\d\.]+)ms prepare_text=([\d\.]+)ms draws=(\d+)'
  $matches = Select-String -Path $LogPath -Pattern $regex -AllMatches
  if ($null -eq $matches -or $matches.Count -eq 0) {
    return $null
  }

  $last = $matches[-1].Matches[-1]
  return @{
    frames = [int]$last.Groups[1].Value
    encode_ms = [double]$last.Groups[2].Value
    prepare_text_ms = [double]$last.Groups[3].Value
    draws = [int]$last.Groups[4].Value
  }
}

function Invoke-Case(
  [string]$Name,
  [int]$Rows,
  [int]$Cols,
  [bool]$VariableSizes
) {
  $caseDir = Join-Path $runDir $Name
  Ensure-Dir $caseDir
  $logPath = Join-Path $caseDir "run.log"

  $vars = @{
    FRET_CANVAS_GRID_ROWS = $Rows
    FRET_CANVAS_GRID_COLS = $Cols
    FRET_CANVAS_GRID_VARIABLE = $(if ($VariableSizes) { "1" } else { "0" })
    FRET_CANVAS_GRID_STATS_WINDOW = $StatsWindow
    FRET_CANVAS_GRID_EXIT_AFTER_FRAMES = $ExitAfterFrames
    FRET_CANVAS_GRID_AUTO_SCROLL = $(if ($AutoScroll) { "1" } else { "0" })
  }

  $prev = $null
  Set-Env $vars ([ref]$prev)
  try {
    $profileArgs = @()
    if ($Release) {
      $profileArgs += "--release"
    }

    $cargoArgs = @("run", "-p", "fret-demo", "--bin", "canvas_datagrid_stress_demo") + $profileArgs
    "case=$Name rows=$Rows cols=$Cols variable=$VariableSizes release=$Release frames=$ExitAfterFrames window=$StatsWindow autoscroll=$AutoScroll" | Out-File -FilePath $logPath -Encoding utf8

    & cargo @cargoArgs 2>&1 | Tee-Object -FilePath $logPath -Append | Out-Host
    if ($LASTEXITCODE -ne 0) {
      throw "cargo exited with code $LASTEXITCODE"
    }
  } finally {
    Restore-Env $prev
  }

  $stats = Parse-LastStatsLine $logPath
  $renderer = Parse-LastRendererPerfLine $logPath
  return @{
    name = $Name
    rows = $Rows
    cols = $Cols
    variable = $VariableSizes
    log = $logPath
    stats = $stats
    renderer = $renderer
  }
}

$root = (Resolve-Path ".").Path
$ts = Get-TimestampFolder
$runDir = $OutDir
if ([string]::IsNullOrWhiteSpace($runDir)) {
  $runDir = Join-Path $root (Join-Path ".bench/canvas-datagrid" $ts)
}
Ensure-Dir $runDir

$commit = (& git rev-parse HEAD).Trim()
$rustc = (& rustc -V).Trim()
$cargo = (& cargo -V).Trim()

$metaPath = Join-Path $runDir "meta.txt"
"commit=$commit" | Out-File -FilePath $metaPath -Encoding utf8
"rustc=$rustc" | Out-File -FilePath $metaPath -Append -Encoding utf8
"cargo=$cargo" | Out-File -FilePath $metaPath -Append -Encoding utf8

$summaryPath = Join-Path $runDir "summary.csv"
"case,rows,cols,variable,profile,exit_after_frames,stats_window,auto_scroll,samples,total_avg_ms,total_p95_ms,encode_ms,prepare_text_ms,draws,log" | Out-File -FilePath $summaryPath -Encoding utf8

$cases = @(
  @{ name = "200k_x_200_fixed"; rows = 200000; cols = 200; variable = $false },
  @{ name = "200k_x_200_variable"; rows = 200000; cols = 200; variable = $true },
  @{ name = "1m_x_200_fixed"; rows = 1000000; cols = 200; variable = $false },
  @{ name = "1m_x_200_variable"; rows = 1000000; cols = 200; variable = $true }
)

foreach ($c in $cases) {
  $result = Invoke-Case -Name $c.name -Rows $c.rows -Cols $c.cols -VariableSizes $c.variable

  $profile = $(if ($Release) { "release" } else { "debug" })
  $samples = ""
  $avg = ""
  $p95 = ""
  if ($null -ne $result.stats) {
    $samples = $result.stats.samples
    $avg = "{0:N3}" -f $result.stats.total_avg_ms
    $p95 = "{0:N3}" -f $result.stats.total_p95_ms
  }

  $encode = ""
  $prepareText = ""
  $draws = ""
  if ($null -ne $result.renderer) {
    $encode = "{0:N3}" -f $result.renderer.encode_ms
    $prepareText = "{0:N3}" -f $result.renderer.prepare_text_ms
    $draws = $result.renderer.draws
  }

  "$($result.name),$($result.rows),$($result.cols),$($result.variable),$profile,$ExitAfterFrames,$StatsWindow,$AutoScroll,$samples,$avg,$p95,$encode,$prepareText,$draws,$($result.log)" | Out-File -FilePath $summaryPath -Append -Encoding utf8
}

Write-Host ""
Write-Host "Wrote:"
Write-Host "  $summaryPath"
Write-Host "  $metaPath"
