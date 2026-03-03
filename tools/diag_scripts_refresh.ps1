param(
  [switch]$Staged,
  [switch]$NoIndex
)

$ErrorActionPreference = "Stop"

function Get-ChangedScriptPaths {
  param([switch]$Staged)

  $diffArgs = @("diff", "--name-only", "--diff-filter=ACMR")
  if ($Staged) {
    $diffArgs += "--cached"
  }

  $paths = & git @diffArgs
  if ($LASTEXITCODE -ne 0) {
    throw "git diff failed"
  }

  $paths `
    | Where-Object { $_ -match '^tools/diag-scripts/.*\.json$' } `
    | Sort-Object -Unique
}

function Invoke-NormalizeScripts {
  param([string[]]$Scripts)

  if (-not $Scripts -or $Scripts.Count -eq 0) {
    return
  }

  # Keep command lines reasonably sized.
  $chunkSize = 40
  for ($i = 0; $i -lt $Scripts.Count; $i += $chunkSize) {
    $chunk = $Scripts[$i..([Math]::Min($Scripts.Count - 1, $i + $chunkSize - 1))]
    & cargo run -p fretboard --quiet -- diag script normalize @chunk --write
    if ($LASTEXITCODE -ne 0) {
      throw "diag script normalize failed"
    }
  }
}

$scripts = Get-ChangedScriptPaths -Staged:$Staged
if (-not $scripts -or $scripts.Count -eq 0) {
  Write-Host "no changed scripts under tools/diag-scripts/*.json"
  exit 0
}

Write-Host ("normalize scripts ({0}):" -f $scripts.Count)
$scripts | ForEach-Object { Write-Host ("  - {0}" -f $_) }

Invoke-NormalizeScripts -Scripts $scripts

if (-not $NoIndex) {
  & python tools/check_diag_scripts_registry.py --write
  if ($LASTEXITCODE -ne 0) {
    throw "check_diag_scripts_registry.py --write failed"
  }

  & python tools/check_diag_scripts_registry.py
  if ($LASTEXITCODE -ne 0) {
    throw "check_diag_scripts_registry.py failed"
  }
}

Write-Host "ok"

