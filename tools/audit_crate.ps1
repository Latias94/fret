param(
    [Parameter(Mandatory = $true)]
    [string]$Crate,

    [int]$TopFiles = 12,

    [int]$MinLines = 200,

    [switch]$EnforceKernelForbiddenDeps
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

function Get-LineCount {
    param([Parameter(Mandatory = $true)][string]$Path)
    $count = 0
    foreach ($line in [System.IO.File]::ReadLines($Path)) {
        $count++
    }
    return $count
}

function MatchesAnyPattern {
    param(
        [Parameter(Mandatory = $true)][string]$Value,
        [Parameter(Mandatory = $true)][string[]]$Patterns
    )
    foreach ($pattern in $Patterns) {
        if ($Value -match $pattern) {
            return $true
        }
    }
    return $false
}

$metadataJson = & cargo metadata --format-version 1
$metadata = $metadataJson | ConvertFrom-Json

$pkg = $metadata.packages | Where-Object { $_.name -eq $Crate } | Select-Object -First 1
if (-not $pkg) {
    Write-Error "crate not found in cargo metadata: $Crate"
    exit 2
}

$manifestPath = [string]$pkg.manifest_path
$crateDir = Split-Path -Parent $manifestPath
$srcDir = Join-Path $crateDir "src"

Write-Host ("crate: {0}" -f $pkg.name)
Write-Host ("version: {0}" -f $pkg.version)
Write-Host ("manifest: {0}" -f $manifestPath)
Write-Host ("dir: {0}" -f $crateDir)

Write-Host ""
Write-Host "top files (src/, by lines):"
if (Test-Path $srcDir) {
    $files = Get-ChildItem -Path $srcDir -Recurse -File -Filter *.rs
    $rows = foreach ($f in $files) {
        $rel = $f.FullName.Substring($crateDir.Length + 1)
        [pscustomobject]@{
            path  = $rel
            lines = (Get-LineCount -Path $f.FullName)
        }
    }
    $rows |
        Where-Object { $_.lines -ge $MinLines } |
        Sort-Object lines -Descending |
        Select-Object -First $TopFiles |
        Format-Table -AutoSize
} else {
    Write-Host "  (no src/ directory)"
}

Write-Host ""
Write-Host "public surface (src/lib.rs quick scan):"
$libRs = Join-Path $srcDir "lib.rs"
if (Test-Path $libRs) {
    $libLines = [System.IO.File]::ReadAllLines($libRs)
    $pubUse = @($libLines | Where-Object { $_ -match "^\s*pub\s+use\s+" }).Count
    $pubMod = @($libLines | Where-Object { $_ -match "^\s*pub\s+mod\s+" }).Count
    Write-Host ("  pub mod: {0}" -f $pubMod)
    Write-Host ("  pub use: {0}" -f $pubUse)
} else {
    Write-Host "  (no src/lib.rs)"
}

Write-Host ""
Write-Host "dependencies (direct, from cargo metadata):"
$directDeps = $pkg.dependencies | Where-Object { $_.kind -in @($null, "normal") }
$workspaceNames = @{}
foreach ($memberId in $metadata.workspace_members) {
    $p = $metadata.packages | Where-Object { $_.id -eq $memberId } | Select-Object -First 1
    if ($p) { $workspaceNames[$p.name] = $true }
}

$workspaceDirect = @()
$externalDirect = @()
foreach ($d in $directDeps) {
    if ($workspaceNames.ContainsKey($d.name)) {
        $workspaceDirect += $d.name
    } else {
        $externalDirect += $d.name
    }
}

Write-Host "  workspace:"
if ($workspaceDirect.Count -eq 0) {
    Write-Host "    (none)"
} else {
    $workspaceDirect | Sort-Object | ForEach-Object { Write-Host ("    - {0}" -f $_) }
}

Write-Host "  external:"
if ($externalDirect.Count -eq 0) {
    Write-Host "    (none)"
} else {
    $externalDirect | Sort-Object | ForEach-Object { Write-Host ("    - {0}" -f $_) }
}

Write-Host ""
Write-Host "kernel forbidden deps spot check (name patterns):"
$kernelCrates = @("fret-core", "fret-runtime", "fret-app", "fret-ui")
$forbidden = @(
    "^winit($|[-_])",
    "^wgpu($|[-_])",
    "^web-sys$",
    "^js-sys$",
    "^wasm-bindgen($|[-_])",
    "^tokio($|[-_])",
    "^reqwest($|[-_])"
)

if ($kernelCrates -contains $pkg.name) {
    $hits = $externalDirect | Where-Object { MatchesAnyPattern -Value $_ -Patterns $forbidden }
    if (@($hits).Count -eq 0) {
        Write-Host "  ok (no obvious forbidden deps)"
    } else {
        Write-Host "  potential violations:"
        $hits | Sort-Object | ForEach-Object { Write-Host ("    - {0}" -f $_) }
        if ($EnforceKernelForbiddenDeps) {
            Write-Error "kernel forbidden deps check failed for $Crate"
            exit 3
        }
    }
} else {
    Write-Host "  (skipped: not a kernel crate)"
}

Write-Host ""
Write-Host "suggested gates (fast):"
Write-Host ("  - cargo fmt")
Write-Host ("  - cargo nextest run -p {0}" -f $pkg.name)
Write-Host ("  - pwsh -NoProfile -File tools/check_layering.ps1")
