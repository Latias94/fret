param(
  [int]$Port = 4020,
  [string]$AppUrl,
  [string]$V0Url = "https://v0.dev",
  [switch]$SkipInstall,
  [switch]$SkipShadcnBuild,
  [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"

function Resolve-RepoRoot([string]$startDir) {
  $dir = (Resolve-Path $startDir).Path
  while ($true) {
    if (Test-Path (Join-Path $dir "repo-ref\\ui")) {
      return $dir
    }
    $parent = Split-Path -Parent $dir
    if (-not $parent -or $parent -eq $dir) {
      break
    }
    $dir = $parent
  }
  throw "Unable to locate repo root from $startDir (expected repo-ref/ui)."
}

$repoRoot = Resolve-RepoRoot (Join-Path $PSScriptRoot "..\\..\\..")
$uiRoot = Join-Path $repoRoot "repo-ref\\ui"
$v4Root = Join-Path $uiRoot "apps\\v4"

if (-not (Test-Path $uiRoot)) {
  throw "Missing repo-ref/ui. Run: ./tools/fetch_repo_refs.ps1 -UiOnly"
}

if (-not $AppUrl) {
  $AppUrl = "http://localhost:$Port"
}

$env:NEXT_PUBLIC_APP_URL = $AppUrl
$env:NEXT_PUBLIC_V0_URL = $V0Url

Write-Host "shadcn/ui v4 server (production)"
Write-Host "  repoRoot: $repoRoot"
Write-Host "  uiRoot:   $uiRoot"
Write-Host "  v4Root:   $v4Root"
Write-Host "  appUrl:   $AppUrl"
Write-Host "  port:     $Port"

if (-not $SkipInstall) {
  Write-Host "Installing deps (pnpm -C repo-ref/ui install)..."
  pnpm -C $uiRoot install
}

if (-not $SkipShadcnBuild) {
  Write-Host "Building shadcn package (pnpm -C repo-ref/ui --filter shadcn build)..."
  pnpm -C $uiRoot --filter shadcn build
}

if (-not $SkipBuild) {
  Write-Host "Building v4 app with webpack (pnpm -C repo-ref/ui/apps/v4 exec next build --webpack)..."
  pnpm -C $v4Root exec next build --webpack
}

Write-Host "Starting server (pnpm -C repo-ref/ui/apps/v4 exec next start -p $Port)..."
pnpm -C $v4Root exec next start -p $Port
