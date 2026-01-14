param(
  [switch]$Release,
  [int]$ExitAfterFrames = 600,
  [int]$StatsWindow = 240,
  [switch]$AutoScroll = $true,
  [int]$Iterations = 1,
  [switch]$FullLog = $false,
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

function Parse-LastVisibleLine([string]$LogPath) {
  $regex = [regex]'datagrid_canvas: visible_rows=(\d+) visible_cols=(\d+) visible_cells=(\d+)'
  $matches = Select-String -Path $LogPath -Pattern $regex -AllMatches
  if ($null -eq $matches -or $matches.Count -eq 0) {
    return $null
  }

  $last = $matches[-1].Matches[-1]
  return @{
    visible_rows = [int]$last.Groups[1].Value
    visible_cols = [int]$last.Groups[2].Value
    visible_cells = [int]$last.Groups[3].Value
  }
}

function Invoke-Case(
  [string]$Name,
  [int]$Rows,
  [int]$Cols,
  [bool]$VariableSizes,
  [int]$Iteration
) {
  $caseDir = Join-Path $runDir $Name
  Ensure-Dir $caseDir
  $logBase = Join-Path $caseDir ("run_iter{0}" -f $Iteration)
  $logPath = "${logBase}.log"
  $fullLogPath = "${logBase}.full.log"

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

    $cargoArgs = @("run")
    if (-not $FullLog) {
      $cargoArgs += "-q"
    }
    $cargoArgs += @("-p", "fret-demo", "--bin", "canvas_datagrid_stress_demo")
    $cargoArgs += $profileArgs
    "case=$Name rows=$Rows cols=$Cols variable=$VariableSizes release=$Release frames=$ExitAfterFrames window=$StatsWindow autoscroll=$AutoScroll" | Out-File -FilePath $logPath -Encoding utf8

    if ($FullLog) {
      & cargo @cargoArgs 2>&1 | Tee-Object -FilePath $fullLogPath | Tee-Object -FilePath $logPath -Append | Out-Host
    } else {
      & cargo @cargoArgs 2>&1 | Tee-Object -FilePath $logPath -Append | Out-Host
    }
    if ($LASTEXITCODE -ne 0) {
      throw "cargo exited with code $LASTEXITCODE"
    }
  } finally {
    Restore-Env $prev
  }

  $stats = Parse-LastStatsLine $logPath
  $renderer = Parse-LastRendererPerfLine $logPath
  $visible = Parse-LastVisibleLine $logPath
  return @{
    name = $Name
    rows = $Rows
    cols = $Cols
    variable = $VariableSizes
    log = $logPath
    full_log = $fullLogPath
    stats = $stats
    renderer = $renderer
    visible = $visible
  }
}

$root = (Resolve-Path ".").Path
$ts = Get-TimestampFolder
$runDir = ""
if (-not [string]::IsNullOrWhiteSpace($OutDir)) {
  $runDir = $OutDir
} elseif (-not [string]::IsNullOrWhiteSpace($env:FRET_BENCH_OUT_DIR)) {
  $runDir = $env:FRET_BENCH_OUT_DIR
} elseif (-not [string]::IsNullOrWhiteSpace($env:SCCACHE_DIR)) {
  $runDir = Join-Path (Split-Path $env:SCCACHE_DIR -Parent) "bench"
} elseif (-not [string]::IsNullOrWhiteSpace($env:TEMP)) {
  $runDir = Join-Path $env:TEMP "fret-bench"
} else {
  $runDir = Join-Path $root ".bench"
}
$runDir = Join-Path $runDir (Join-Path "canvas-datagrid" $ts)
Ensure-Dir $runDir

$commit = (& git rev-parse HEAD).Trim()
$rustc = (& rustc -V).Trim()
$cargo = (& cargo -V).Trim()

$metaPath = Join-Path $runDir "meta.txt"
"commit=$commit" | Out-File -FilePath $metaPath -Encoding utf8
"rustc=$rustc" | Out-File -FilePath $metaPath -Append -Encoding utf8
"cargo=$cargo" | Out-File -FilePath $metaPath -Append -Encoding utf8
"sccache=$env:SCCACHE_DIR" | Out-File -FilePath $metaPath -Append -Encoding utf8

$summaryPath = Join-Path $runDir "summary.csv"
"case,iteration,rows,cols,variable,profile,exit_after_frames,stats_window,auto_scroll,visible_rows,visible_cols,visible_cells,samples,total_avg_ms,total_p95_ms,encode_ms,prepare_text_ms,draws,log,full_log" | Out-File -FilePath $summaryPath -Encoding utf8

function Get-Median([double[]]$Values) {
  if ($null -eq $Values -or $Values.Count -eq 0) {
    return $null
  }
  $sorted = $Values | Sort-Object
  $n = $sorted.Count
  if (($n % 2) -eq 1) {
    return [double]$sorted[($n - 1) / 2]
  }
  $a = [double]$sorted[($n / 2) - 1]
  $b = [double]$sorted[$n / 2]
  return ($a + $b) / 2.0
}

$cases = @(
  @{ name = "200k_x_200_fixed"; rows = 200000; cols = 200; variable = $false },
  @{ name = "200k_x_200_variable"; rows = 200000; cols = 200; variable = $true },
  @{ name = "1m_x_200_fixed"; rows = 1000000; cols = 200; variable = $false },
  @{ name = "1m_x_200_variable"; rows = 1000000; cols = 200; variable = $true }
)

if ($Iterations -lt 1) {
  throw "-Iterations must be >= 1"
}

$profile = $(if ($Release) { "release" } else { "debug" })
$allRows = @()

foreach ($c in $cases) {
  for ($iter = 1; $iter -le $Iterations; $iter++) {
    $result = Invoke-Case -Name $c.name -Rows $c.rows -Cols $c.cols -VariableSizes $c.variable -Iteration $iter

    $vrows = ""
    $vcols = ""
    $vcells = ""
    if ($null -ne $result.visible) {
      $vrows = $result.visible.visible_rows
      $vcols = $result.visible.visible_cols
      $vcells = $result.visible.visible_cells
    }

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

    "$($result.name),$iter,$($result.rows),$($result.cols),$($result.variable),$profile,$ExitAfterFrames,$StatsWindow,$AutoScroll,$vrows,$vcols,$vcells,$samples,$avg,$p95,$encode,$prepareText,$draws,$($result.log),$($result.full_log)" | Out-File -FilePath $summaryPath -Append -Encoding utf8

    $allRows += [pscustomobject]@{
      case = $result.name
      iteration = $iter
      rows = $result.rows
      cols = $result.cols
      variable = $result.variable
      visible_rows = $(if ($null -ne $result.visible) { $result.visible.visible_rows } else { $null })
      visible_cols = $(if ($null -ne $result.visible) { $result.visible.visible_cols } else { $null })
      visible_cells = $(if ($null -ne $result.visible) { $result.visible.visible_cells } else { $null })
      samples = $result.stats.samples
      total_avg_ms = $result.stats.total_avg_ms
      total_p95_ms = $result.stats.total_p95_ms
      encode_ms = $(if ($null -ne $result.renderer) { $result.renderer.encode_ms } else { $null })
      prepare_text_ms = $(if ($null -ne $result.renderer) { $result.renderer.prepare_text_ms } else { $null })
      draws = $(if ($null -ne $result.renderer) { $result.renderer.draws } else { $null })
      log = $result.log
    }
  }
}

if ($Iterations -gt 1) {
  $aggPath = Join-Path $runDir "summary_agg.csv"
  "case,rows,cols,variable,profile,iterations,visible_rows_median,visible_cols_median,visible_cells_median,total_avg_median_ms,total_p95_median_ms,prepare_text_median_ms,draws_median" | Out-File -FilePath $aggPath -Encoding utf8

  $grouped = $allRows | Group-Object -Property case
  foreach ($g in $grouped) {
    $first = $g.Group[0]
    $vrMed = Get-Median ($g.Group | Where-Object { $null -ne $_.visible_rows } | ForEach-Object { [double]$_.visible_rows })
    $vcMed = Get-Median ($g.Group | Where-Object { $null -ne $_.visible_cols } | ForEach-Object { [double]$_.visible_cols })
    $vcellMed = Get-Median ($g.Group | Where-Object { $null -ne $_.visible_cells } | ForEach-Object { [double]$_.visible_cells })
    $avgMed = Get-Median ($g.Group | ForEach-Object { [double]$_.total_avg_ms })
    $p95Med = Get-Median ($g.Group | ForEach-Object { [double]$_.total_p95_ms })
    $prepMed = Get-Median ($g.Group | Where-Object { $null -ne $_.prepare_text_ms } | ForEach-Object { [double]$_.prepare_text_ms })
    $drawsMed = Get-Median ($g.Group | Where-Object { $null -ne $_.draws } | ForEach-Object { [double]$_.draws })

    $vrOut = $(if ($null -ne $vrMed) { "{0:N0}" -f $vrMed } else { "" })
    $vcOut = $(if ($null -ne $vcMed) { "{0:N0}" -f $vcMed } else { "" })
    $vcellOut = $(if ($null -ne $vcellMed) { "{0:N0}" -f $vcellMed } else { "" })
    $avgOut = $(if ($null -ne $avgMed) { "{0:N3}" -f $avgMed } else { "" })
    $p95Out = $(if ($null -ne $p95Med) { "{0:N3}" -f $p95Med } else { "" })
    $prepOut = $(if ($null -ne $prepMed) { "{0:N3}" -f $prepMed } else { "" })
    $drawsOut = $(if ($null -ne $drawsMed) { "{0:N0}" -f $drawsMed } else { "" })

    "$($first.case),$($first.rows),$($first.cols),$($first.variable),$profile,$Iterations,$vrOut,$vcOut,$vcellOut,$avgOut,$p95Out,$prepOut,$drawsOut" | Out-File -FilePath $aggPath -Append -Encoding utf8
  }
}

Write-Host ""
Write-Host "Wrote:"
Write-Host "  $summaryPath"
if ($Iterations -gt 1) {
  Write-Host "  $(Join-Path $runDir "summary_agg.csv")"
}
Write-Host "  $metaPath"
