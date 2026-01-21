$ErrorActionPreference = "Stop"

# Builds all `apps/fret-demo` binaries on Windows with conservative parallelism.
#
# Why: building many wgpu-heavy bins in parallel can exhaust virtual memory on Windows, leading to
# linker OOM (`LNK1102`) or metadata mmap failures (`os error 1455`).
#
# Usage:
#   pwsh tools/windows/build-fret-demo-bins.ps1
#

$repoRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
Push-Location $repoRoot
try {
  if (-not $env:CARGO_TARGET_DIR) {
    $env:CARGO_TARGET_DIR = (Join-Path $repoRoot "target")
  }

  if (-not $env:CARGO_BUILD_JOBS) {
    $env:CARGO_BUILD_JOBS = "4"
  }

  cargo build -p fret-demo --bins
} finally {
  Pop-Location
}
