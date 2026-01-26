$ErrorActionPreference = "Stop"

param(
  [switch]$Quick,
  [switch]$Full
)

if (-not $Quick -and -not $Full) {
  $Full = $true
}

$repoRoot = Split-Path -Parent $PSScriptRoot
Push-Location $repoRoot
try {
  cargo nextest run -p fret-ui view_cache
  cargo nextest run -p fret-ui-kit window_overlays

  if ($Full) {
    cargo nextest run -p fret-ui-shadcn tooltip hover_card dropdown_menu
  }
} finally {
  Pop-Location
}

