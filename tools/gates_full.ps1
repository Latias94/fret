param(
  [Parameter(Mandatory = $false)]
  [switch]$SkipPreRelease,

  [Parameter(Mandatory = $false)]
  [switch]$SkipWebGoldens,

  [Parameter(Mandatory = $false)]
  [switch]$WithIcons
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

function Invoke-Checked(
  [string]$Name,
  [string]$Program,
  [string[]]$Arguments
) {
  Write-Host "[gates-full] $Name"
  & $Program @Arguments
  if ($LASTEXITCODE -ne 0) {
    throw "Step failed: $Name (exit code: $LASTEXITCODE)"
  }
}

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")

if (-not $SkipPreRelease) {
  $preReleaseArgs = @(
    (Join-Path $repoRoot "tools/pre_release.py")
  )

  if (-not $WithIcons) {
    $preReleaseArgs += "--skip-icons"
  }

  Invoke-Checked "pre_release.py (workspace policies + fmt + clippy + nextest)" "python" $preReleaseArgs
}

if (-not $SkipWebGoldens) {
  $nextest = Get-Command cargo-nextest -ErrorAction SilentlyContinue

  if ($null -eq $nextest) {
    Write-Warning "cargo-nextest is not installed; falling back to cargo test -p fret-ui-shadcn --features web-goldens"
    Invoke-Checked "cargo test (web-goldens)" "cargo" @(
      "test",
      "-p",
      "fret-ui-shadcn",
      "--features",
      "web-goldens"
    )
  } else {
    Invoke-Checked "cargo nextest run -p fret-ui-shadcn --features web-goldens" "cargo" @(
      "nextest",
      "run",
      "-p",
      "fret-ui-shadcn",
      "--features",
      "web-goldens"
    )
  }
}

Write-Host "[gates-full] done"

