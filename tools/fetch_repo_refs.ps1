<#!
Bootstrap helper for local `repo-ref/` checkouts.

Why this exists:
- `repo-ref/` is intentionally ignored by git in this repo (see `.gitignore`).
- Many docs (especially `docs/audits/*`) reference paths under `repo-ref/` for upstream reading.
- This script makes those paths reproducible on a new machine without committing large upstream repos.

Usage (PowerShell):
  ./tools/fetch_repo_refs.ps1               # fetch the recommended set (ui + primitives)
  ./tools/fetch_repo_refs.ps1 -UiOnly       # fetch only shadcn/ui
  ./tools/fetch_repo_refs.ps1 -PrimitivesOnly
  ./tools/fetch_repo_refs.ps1 -Force        # re-point origin + checkout pinned commit

Proxy (optional):
  $env:HTTP_PROXY='http://127.0.0.1:10809'
  $env:HTTPS_PROXY='http://127.0.0.1:10809'
  $env:ALL_PROXY='http://127.0.0.1:10809'
!#>

[CmdletBinding()]
param(
  [switch]$UiOnly,
  [switch]$PrimitivesOnly,
  [switch]$Force,

  # Override URLs if you maintain internal mirrors/forks.
  [string]$UiUrl = "https://github.com/shadcn-ui/ui.git",
  [string]$PrimitivesUrl = "https://github.com/radix-ui/primitives.git",

  # Pinned commits recorded in `docs/repo-ref.md`.
  [string]$UiCommit = "d07a7af8",
  [string]$PrimitivesCommit = "90751370"
)

$ErrorActionPreference = "Stop"

function Ensure-RepoCheckout {
  param(
    [Parameter(Mandatory = $true)][string]$Name,
    [Parameter(Mandatory = $true)][string]$Url,
    [Parameter(Mandatory = $true)][string]$Commit
  )

  $root = Join-Path (Get-Location) "repo-ref"
  if (-not (Test-Path $root)) {
    New-Item -ItemType Directory -Path $root | Out-Null
  }

  $path = Join-Path $root $Name
  if (-not (Test-Path $path)) {
    Write-Host "Cloning $Name -> $path"
    git clone $Url $path | Out-Null
  }

  $gitDir = Join-Path $path ".git"
  if (-not (Test-Path $gitDir)) {
    Write-Warning "Skipping ${Name}: $path exists but is not a git repo."
    Write-Warning "If this is a checkout you manage manually, keep it; otherwise delete the folder and re-run."
    return
  }

  $origin = (& git -C $path remote get-url origin 2>$null)
  $head = (& git -C $path rev-parse --short=12 HEAD 2>$null)
  Write-Host "Found ${Name}: origin=$origin head=$head"

  if ($Force -and $origin -ne $Url) {
    Write-Host "Updating ${Name} origin -> $Url"
    git -C $path remote set-url origin $Url | Out-Null
  }

  Write-Host "Fetching ${Name}..."
  git -C $path fetch --tags origin | Out-Null

  # Accept short SHAs (as recorded in docs) and full SHAs.
  $target = $Commit
  Write-Host "Checking out ${Name} @ $target"
  git -C $path checkout $target | Out-Null
}

$wantUi = $true
$wantPrimitives = $true
if ($UiOnly) {
  $wantPrimitives = $false
}
if ($PrimitivesOnly) {
  $wantUi = $false
}

if ($wantUi) {
  Ensure-RepoCheckout -Name "ui" -Url $UiUrl -Commit $UiCommit
}
if ($wantPrimitives) {
  Ensure-RepoCheckout -Name "primitives" -Url $PrimitivesUrl -Commit $PrimitivesCommit
}

